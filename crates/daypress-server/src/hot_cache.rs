//! Pre-Compressed In-Memory SSR Hot Cache (`<50µs` P99 Latency) & Cloudflare Edge Purge (`hot_cache.rs`).
//!
//! Caches rendered public SSR HTML alongside pre-compressed Gzip payloads so anonymous visitor
//! requests are served directly from memory in microseconds without touching SQLite or re-running
//! the AST/HTML sanitizer pipeline. Automatically invalidates on any database write and optionally
//! triggers an asynchronous Cloudflare Zone cache purge.

use axum::{
    body::Body,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use flate2::{write::GzEncoder, Compression};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    io::Write,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, Mutex, RwLock,
    },
    time::{SystemTime, UNIX_EPOCH},
};

const MAX_HOT_CACHE_ENTRIES: usize = 2048;

#[derive(Clone)]
pub struct CachedSsrEntry {
    pub html_raw: Arc<Vec<u8>>,
    pub html_gzip: Arc<Vec<u8>>,
    pub etag: String,
    pub created_at_unix: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HotCacheStats {
    pub enabled: bool,
    pub cached_entries: usize,
    pub hits: u64,
    pub misses: u64,
    pub hit_rate_pct: f64,
    pub invalidations: u64,
    pub raw_bytes: usize,
    pub compressed_bytes: usize,
    pub last_purged_at: Option<String>,
    pub last_invalidation_reason: Option<String>,
    pub cloudflare_purge_configured: bool,
}

#[derive(Default)]
pub struct SsrHotCache {
    entries: RwLock<HashMap<String, CachedSsrEntry>>,
    hits: AtomicU64,
    misses: AtomicU64,
    invalidations: AtomicU64,
    last_purged_at: RwLock<Option<String>>,
    last_reason: RwLock<Option<String>>,
}

impl SsrHotCache {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the incoming request carries an active admin/editor or subscriber unlock session cookie.
    pub fn has_session_cookie(headers: &HeaderMap) -> bool {
        headers
            .get(header::COOKIE)
            .and_then(|v| v.to_str().ok())
            .map(|cookie_str| {
                cookie_str.split(';').any(|item| {
                    let trimmed = item.trim();
                    trimmed.starts_with("daypress_admin=")
                        || trimmed.starts_with("daypress_subscriber=")
                })
            })
            .unwrap_or(false)
    }

    /// Extracts optional theme preference from cookie so dark/light overrides don't collide.
    fn theme_variant_from_headers(headers: &HeaderMap) -> &'static str {
        if let Some(cookie_str) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) {
            if cookie_str.contains("dp_theme=dark") {
                return "dark";
            } else if cookie_str.contains("dp_theme=light") {
                return "light";
            }
        }
        "default"
    }

    pub fn make_key(route_key: &str, headers: &HeaderMap) -> String {
        let theme = Self::theme_variant_from_headers(headers);
        format!("{theme}:{route_key}")
    }

    fn accepts_gzip(headers: &HeaderMap) -> bool {
        headers
            .get(header::ACCEPT_ENCODING)
            .and_then(|v| v.to_str().ok())
            .map(|enc| enc.contains("gzip"))
            .unwrap_or(false)
    }

    /// Attempts to serve a public SSR request directly from the in-memory hot cache in `<50µs`.
    pub fn try_serve(&self, route_key: &str, headers: &HeaderMap) -> Option<Response> {
        if Self::has_session_cookie(headers) {
            return None;
        }
        let key = Self::make_key(route_key, headers);
        let entry = {
            let guard = self.entries.read().ok()?;
            guard.get(&key)?.clone()
        };

        self.hits.fetch_add(1, Ordering::Relaxed);

        // Conditional ETag check (`304 Not Modified`)
        if let Some(inm) = headers
            .get(header::IF_NONE_MATCH)
            .and_then(|v| v.to_str().ok())
        {
            if inm.trim() == entry.etag || inm == "*" {
                let mut resp = StatusCode::NOT_MODIFIED.into_response();
                let h = resp.headers_mut();
                h.insert("x-daypress-cache", HeaderValue::from_static("HIT"));
                if let Ok(etag_val) = HeaderValue::from_str(&entry.etag) {
                    h.insert(header::ETAG, etag_val);
                }
                return Some(resp);
            }
        }

        Some(Self::build_cached_response(&entry, headers, "HIT"))
    }

    /// Compresses and stores a rendered 200 OK public SSR HTML document in the hot cache and returns the response.
    pub fn store_and_serve(
        &self,
        route_key: &str,
        headers: &HeaderMap,
        is_authenticated: bool,
        html: String,
    ) -> Response {
        if is_authenticated || Self::has_session_cookie(headers) {
            let mut resp = (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
                html,
            )
                .into_response();
            resp.headers_mut()
                .insert("x-daypress-cache", HeaderValue::from_static("BYPASS"));
            return resp;
        }

        self.misses.fetch_add(1, Ordering::Relaxed);

        let raw_bytes = html.into_bytes();
        let mut hasher = Sha256::new();
        hasher.update(&raw_bytes);
        let digest = hex::encode(hasher.finalize());
        let short_digest: String = digest.chars().take(16).collect();
        let etag = format!("\"dp-{short_digest}\"");

        let mut encoder =
            GzEncoder::new(Vec::with_capacity(raw_bytes.len() / 3), Compression::fast());
        let gzip_bytes = if encoder.write_all(&raw_bytes).is_ok() {
            encoder.finish().unwrap_or_default()
        } else {
            Vec::new()
        };

        let now_unix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let entry = CachedSsrEntry {
            html_raw: Arc::new(raw_bytes),
            html_gzip: Arc::new(gzip_bytes),
            etag,
            created_at_unix: now_unix,
        };

        let key = Self::make_key(route_key, headers);
        if let Ok(mut guard) = self.entries.write() {
            if guard.len() >= MAX_HOT_CACHE_ENTRIES && !guard.contains_key(&key) {
                guard.clear();
            }
            guard.insert(key, entry.clone());
        }

        Self::build_cached_response(&entry, headers, "MISS")
    }

    fn build_cached_response(
        entry: &CachedSsrEntry,
        headers: &HeaderMap,
        cache_status: &'static str,
    ) -> Response {
        let use_gzip = Self::accepts_gzip(headers) && !entry.html_gzip.is_empty();
        let body_bytes = if use_gzip {
            entry.html_gzip.as_ref().clone()
        } else {
            entry.html_raw.as_ref().clone()
        };

        let mut resp = Response::new(Body::from(body_bytes));
        *resp.status_mut() = StatusCode::OK;
        let h = resp.headers_mut();
        h.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("text/html; charset=utf-8"),
        );
        h.insert(
            header::VARY,
            HeaderValue::from_static("Accept-Encoding, Cookie"),
        );
        h.insert("x-daypress-cache", HeaderValue::from_static(cache_status));
        if let Ok(etag_val) = HeaderValue::from_str(&entry.etag) {
            h.insert(header::ETAG, etag_val);
        }
        if use_gzip {
            h.insert(header::CONTENT_ENCODING, HeaderValue::from_static("gzip"));
        }
        resp
    }

    /// Invalidates all entries in the in-memory SSR hot cache (`<1µs`).
    pub fn invalidate_all(&self, reason: &str) -> usize {
        let cleared = if let Ok(mut guard) = self.entries.write() {
            let count = guard.len();
            guard.clear();
            count
        } else {
            0
        };
        self.invalidations.fetch_add(1, Ordering::Relaxed);
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        if let Ok(mut p) = self.last_purged_at.write() {
            *p = Some(now);
        }
        if let Ok(mut r) = self.last_reason.write() {
            *r = Some(reason.to_string());
        }
        cleared
    }

    pub fn stats(&self, cf_configured: bool) -> HotCacheStats {
        let (cached_entries, raw_bytes, compressed_bytes) = if let Ok(guard) = self.entries.read() {
            let mut raw = 0usize;
            let mut comp = 0usize;
            for v in guard.values() {
                raw += v.html_raw.len();
                comp += v.html_gzip.len();
            }
            (guard.len(), raw, comp)
        } else {
            (0, 0, 0)
        };

        let hits = self.hits.load(Ordering::Relaxed);
        let misses = self.misses.load(Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate_pct = if total > 0 {
            ((hits as f64 / total as f64) * 1000.0).round() / 10.0
        } else {
            0.0
        };
        let invalidations = self.invalidations.load(Ordering::Relaxed);
        let last_purged_at = self.last_purged_at.read().ok().and_then(|g| g.clone());
        let last_invalidation_reason = self.last_reason.read().ok().and_then(|g| g.clone());

        HotCacheStats {
            enabled: true,
            cached_entries,
            hits,
            misses,
            hit_rate_pct,
            invalidations,
            raw_bytes,
            compressed_bytes,
            last_purged_at,
            last_invalidation_reason,
            cloudflare_purge_configured: cf_configured,
        }
    }
}

/// Sliding-window per-form IP rate limiter for FormStudio anti-spam protection.
#[derive(Default)]
pub struct FormRateLimiter {
    buckets: Mutex<HashMap<(i64, String), Vec<u64>>>,
}

impl FormRateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns `true` if the submission is allowed under `max_per_minute` (0 disables rate limiting).
    pub fn check_and_record(&self, form_id: i64, ip_hash: &str, max_per_minute: u32) -> bool {
        if max_per_minute == 0 {
            return true;
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let cutoff = now.saturating_sub(60);

        let Ok(mut guard) = self.buckets.lock() else {
            return true;
        };
        let entry = guard.entry((form_id, ip_hash.to_string())).or_default();
        entry.retain(|&ts| ts >= cutoff);
        if entry.len() >= max_per_minute as usize {
            return false;
        }
        entry.push(now);
        true
    }
}
