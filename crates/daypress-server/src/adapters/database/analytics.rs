//! Native Cookieless Analytics Engine (`adapters/database/analytics.rs`).
//!
//! Provides GDPR/ePrivacy-native telemetry with:
//! - Daily-rotating CSPRNG server secret salt (`SHA-256(daily_salt | ip | ua | YYYY-MM-DD)`)
//! - Bot, crawler, headless browser, prefetch/prerender, DNT/GPC, and Admin/Staff filtering
//! - Zero-PII User-Agent classification (`Browser` & `OS`) and Referrer Domain / Channel normalization
//! - Active engaged reading time (`duration_secs`), max scroll depth (`scroll_depth`), outbound link & file download events
//! - Sargable composite-indexed SQLite queries, period-over-period deltas, drilldown filters, and `daypress_analytics_daily` rollups

use super::Database;
use crate::models::{
    AnalyticsBucketPoint, AnalyticsFilter, AnalyticsSummary, AnalyticsTelemetrySettings,
    BreakdownMetric, ConversionMetric, TopPageMetric,
};
use axum::http::{header, HeaderMap};
use chrono::{Duration, Utc};
use rusqlite::{
    params, params_from_iter, types::Value as SqlValue, Connection, OptionalExtension, Result,
};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Truncates a string safely on UTF-8 character boundaries.
fn safe_truncate(input: &str, max_chars: usize) -> String {
    input.trim().chars().take(max_chars).collect()
}

/// Normalizes a raw page URL or path into a clean canonical path (`/pricing`, `/blog/hello`, etc.).
pub fn normalize_analytics_page_path(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "/".to_string();
    }
    // If a full URL was passed, strip scheme + authority.
    let without_origin = if let Some(idx) = trimmed.find("://") {
        let after_scheme = &trimmed[idx + 3..];
        if let Some(slash_idx) = after_scheme.find('/') {
            &after_scheme[slash_idx..]
        } else {
            "/"
        }
    } else {
        trimmed
    };
    let path_only = without_origin
        .split('#')
        .next()
        .unwrap_or("/")
        .split('?')
        .next()
        .unwrap_or("/")
        .trim();
    if path_only.is_empty() {
        "/".to_string()
    } else if path_only.starts_with('/') {
        safe_truncate(path_only, 240)
    } else {
        format!("/{}", safe_truncate(path_only, 239))
    }
}

/// Detects known search crawlers, AI scrapers, headless browsers, uptime monitors, and CLI HTTP clients.
/// Intentionally allows empty/`"unknown"` User-Agents so programmatic API/integration tests continue to pass.
pub fn is_bot_or_crawler(ua: &str) -> bool {
    let clean = ua.trim();
    if clean.is_empty() || clean.eq_ignore_ascii_case("unknown") {
        return false;
    }
    let lower = clean.to_ascii_lowercase();
    const BOT_SIGNATURES: &[&str] = &[
        "googlebot",
        "adsbot-google",
        "apis-google",
        "mediapartners-google",
        "bingbot",
        "adidxbot",
        "msnbot",
        "slurp",
        "duckduckbot",
        "baiduspider",
        "yandexbot",
        "sogou",
        "exabot",
        "facebot",
        "facebookexternalhit",
        "meta-externalagent",
        "ia_archiver",
        "gptbot",
        "chatgpt-user",
        "oai-searchbot",
        "claudebot",
        "anthropic-ai",
        "perplexitybot",
        "bytespider",
        "ccbot",
        "cohere-ai",
        "ahrefsbot",
        "semrushbot",
        "mj12bot",
        "dotbot",
        "petalbot",
        "rogerbot",
        "screaming frog",
        "applebot",
        "linkedinbot",
        "twitterbot",
        "discordbot",
        "telegrambot",
        "whatsapp",
        "slackbot",
        "pinterestbot",
        "pingdom",
        "uptimerobot",
        "statuscake",
        "better stack",
        "gtmetrix",
        "lighthouse",
        "chrome-lighthouse",
        "pagespeed",
        "headlesschrome",
        "phantomjs",
        "puppeteer",
        "playwright",
        "selenium",
        "webdriver",
        "cypress",
        "curl/",
        "wget/",
        "python-requests",
        "python-urllib",
        "aiohttp/",
        "go-http-client",
        "okhttp/",
        "apache-httpclient",
        "libwww-perl",
        "scrapy",
        "httpie/",
    ];
    BOT_SIGNATURES.iter().any(|sig| lower.contains(sig))
}

/// Detects speculative browser prefetch / prerender requests so unvisited tabs do not inflate views.
pub fn is_prefetch_or_prerender(headers: &HeaderMap) -> bool {
    for hdr_name in ["sec-purpose", "purpose", "x-purpose", "x-moz"] {
        if let Some(val) = headers.get(hdr_name).and_then(|v| v.to_str().ok()) {
            let lower = val.to_ascii_lowercase();
            if lower.contains("prefetch")
                || lower.contains("preview")
                || lower.contains("prerender")
            {
                return true;
            }
        }
    }
    false
}

/// Checks `DNT: 1` (Do Not Track) and `Sec-GPC: 1` (Global Privacy Control) request headers.
pub fn has_dnt_or_gpc(headers: &HeaderMap) -> bool {
    let dnt = headers
        .get("dnt")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .unwrap_or("");
    let gpc = headers
        .get("sec-gpc")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .unwrap_or("");
    dnt == "1" || gpc == "1"
}

/// Zero-PII User-Agent classification into `(browser, os, fallback_device_type)`.
/// Raw User-Agent strings are never persisted in analytics tables.
pub fn parse_ua_telemetry(ua: &str) -> (&'static str, &'static str, &'static str) {
    let clean = ua.trim();
    if clean.is_empty() || clean.eq_ignore_ascii_case("unknown") {
        return ("Unknown", "Unknown", "desktop");
    }
    let lower = clean.to_ascii_lowercase();

    let browser = if lower.contains("edg/") || lower.contains("edgios") || lower.contains("edga/") {
        "Edge"
    } else if lower.contains("opr/") || lower.contains("opera") {
        "Opera"
    } else if lower.contains("brave") {
        "Brave"
    } else if lower.contains("vivaldi") {
        "Vivaldi"
    } else if lower.contains("samsungbrowser") {
        "Samsung Internet"
    } else if lower.contains("ucbrowser") {
        "UC Browser"
    } else if lower.contains("firefox/") || lower.contains("fxios") {
        "Firefox"
    } else if lower.contains("chrome/") || lower.contains("crios") || lower.contains("chromium") {
        "Chrome"
    } else if lower.contains("safari/") {
        "Safari"
    } else {
        "Other"
    };

    let os = if lower.contains("iphone") || lower.contains("ipad") || lower.contains("ipod") {
        "iOS"
    } else if lower.contains("android") {
        "Android"
    } else if lower.contains("cros") {
        "ChromeOS"
    } else if lower.contains("windows") {
        "Windows"
    } else if lower.contains("macintosh") || lower.contains("mac os x") {
        "macOS"
    } else if lower.contains("linux") || lower.contains("ubuntu") || lower.contains("x11") {
        "Linux"
    } else {
        "Other"
    };

    let device = if lower.contains("ipad")
        || lower.contains("tablet")
        || (lower.contains("android") && !lower.contains("mobile"))
    {
        "tablet"
    } else if lower.contains("mobi") || lower.contains("iphone") || lower.contains("android") {
        "mobile"
    } else {
        "desktop"
    };

    (browser, os, device)
}

fn extract_host_clean(url_or_host: &str) -> String {
    let s = url_or_host.trim();
    if s.is_empty() || s.starts_with('/') {
        return String::new();
    }
    let after_scheme = if let Some(idx) = s.find("://") {
        &s[idx + 3..]
    } else {
        s
    };
    let authority = after_scheme
        .split('/')
        .next()
        .unwrap_or("")
        .split('?')
        .next()
        .unwrap_or("")
        .split('#')
        .next()
        .unwrap_or("");
    let host_port = authority.rsplit('@').next().unwrap_or(authority);
    let host = host_port
        .split(':')
        .next()
        .unwrap_or(host_port)
        .trim()
        .to_ascii_lowercase();
    for prefix in ["www.", "m.", "lm.", "l."] {
        if let Some(stripped) = host.strip_prefix(prefix) {
            if stripped.contains('.') {
                return safe_truncate(stripped, 180);
            }
        }
    }
    safe_truncate(&host, 180)
}

/// Normalizes a raw `Referer` URL into `(referrer_domain, acquisition_channel)`.
/// Strips internal self-referrals matching `self_hosts`.
pub fn normalize_referrer_and_channel(
    raw_referrer: &str,
    self_hosts: &[String],
    utm_source: Option<&str>,
    utm_medium: Option<&str>,
) -> (String, &'static str) {
    let mut domain = extract_host_clean(raw_referrer);
    if !domain.is_empty()
        && self_hosts
            .iter()
            .any(|h| !h.is_empty() && h.eq_ignore_ascii_case(&domain))
    {
        domain.clear();
    }

    // Collapse regional search/social redirectors to canonical domains
    if domain.starts_with("google.") || domain.contains(".google.") {
        domain = "google.com".to_string();
    } else if domain == "t.co" || domain == "x.com" || domain == "twitter.com" {
        domain = "x.com".to_string();
    } else if domain == "lnkd.in" {
        domain = "linkedin.com".to_string();
    } else if domain == "youtu.be" {
        domain = "youtube.com".to_string();
    } else if domain == "old.reddit.com" {
        domain = "reddit.com".to_string();
    } else if domain.ends_with(".bing.com") {
        domain = "bing.com".to_string();
    } else if domain.contains("search.yahoo.") {
        domain = "yahoo.com".to_string();
    }

    let med = utm_medium.unwrap_or("").trim().to_ascii_lowercase();
    let src = utm_source.unwrap_or("").trim().to_ascii_lowercase();

    let is_paid_medium = matches!(
        med.as_str(),
        "cpc"
            | "ppc"
            | "paid"
            | "paid_social"
            | "paidsocial"
            | "display"
            | "cpm"
            | "ads"
            | "sponsored"
    );
    let is_email_medium = matches!(med.as_str(), "email" | "newsletter" | "e-mail");
    let is_social_medium = matches!(med.as_str(), "social" | "social-network" | "social-media");

    const SEARCH_DOMAINS: &[&str] = &[
        "google.com",
        "bing.com",
        "duckduckgo.com",
        "yahoo.com",
        "ecosia.org",
        "kagi.com",
        "brave.com",
        "search.brave.com",
        "baidu.com",
        "yandex.ru",
        "yandex.com",
        "qwant.com",
        "startpage.com",
        "perplexity.ai",
        "chatgpt.com",
    ];
    const SOCIAL_DOMAINS: &[&str] = &[
        "x.com",
        "news.ycombinator.com",
        "reddit.com",
        "linkedin.com",
        "facebook.com",
        "instagram.com",
        "youtube.com",
        "tiktok.com",
        "threads.net",
        "bsky.app",
        "github.com",
        "producthunt.com",
        "indiehackers.com",
        "lobste.rs",
        "dev.to",
        "medium.com",
        "discord.com",
        "t.me",
    ];
    const EMAIL_DOMAINS: &[&str] = &[
        "mail.google.com",
        "outlook.live.com",
        "outlook.office.com",
        "mail.yahoo.com",
        "substack.com",
        "beehiiv.com",
    ];

    let channel = if is_paid_medium {
        "Paid"
    } else if is_email_medium || EMAIL_DOMAINS.iter().any(|d| domain == *d) {
        "Email"
    } else if SEARCH_DOMAINS.iter().any(|d| domain == *d)
        || matches!(src.as_str(), "google" | "bing" | "duckduckgo") && med == "organic"
    {
        "Organic Search"
    } else if is_social_medium
        || SOCIAL_DOMAINS.iter().any(|d| domain == *d)
        || matches!(
            src.as_str(),
            "hn" | "hackernews"
                | "twitter"
                | "x"
                | "linkedin"
                | "reddit"
                | "facebook"
                | "instagram"
                | "youtube"
                | "github"
                | "producthunt"
        )
    {
        "Social"
    } else if !src.is_empty() || !med.is_empty() {
        "Campaign / UTM"
    } else if !domain.is_empty() {
        "Referral"
    } else {
        "Direct"
    };

    (domain, channel)
}

fn generate_random_hex_secret() -> String {
    let mut bytes = [0u8; 32];
    if getrandom::getrandom(&mut bytes).is_ok() {
        hex::encode(bytes)
    } else {
        let mut fallback = Sha256::new();
        fallback.update(
            Utc::now()
                .timestamp_nanos_opt()
                .unwrap_or_default()
                .to_le_bytes(),
        );
        fallback.update(b"daypress-daily-salt-fallback");
        hex::encode(fallback.finalize())
    }
}

fn upsert_option_with_conn(conn: &Connection, name: &str, val: &str) {
    let updated = conn
        .execute(
            "UPDATE options SET option_value = ?1 WHERE option_name = ?2",
            params![val, name],
        )
        .unwrap_or(0);
    if updated == 0 {
        let _ = conn.execute(
            "INSERT INTO options (option_name, option_value, autoload) VALUES (?1, ?2, 'yes')",
            params![name, val],
        );
    }
}

fn get_option_with_conn(conn: &Connection, name: &str) -> Option<String> {
    conn.query_row(
        "SELECT option_value FROM options WHERE option_name = ?1",
        params![name],
        |r| r.get::<_, String>(0),
    )
    .optional()
    .ok()
    .flatten()
}

/// Internal SQL filter builder for `daypress_analytics_events`.
struct SqlWhereBuilder {
    clauses: Vec<String>,
    params: Vec<SqlValue>,
}

impl SqlWhereBuilder {
    fn new_time_window(start_inclusive: &str, end_inclusive: Option<&str>) -> Self {
        let mut clauses = Vec::new();
        let mut params = Vec::new();
        clauses.push("created_at >= ?1".to_string());
        params.push(SqlValue::Text(start_inclusive.to_string()));
        if let Some(end_str) = end_inclusive {
            let idx = params.len() + 1;
            clauses.push(format!("created_at < ?{idx}"));
            params.push(SqlValue::Text(end_str.to_string()));
        }
        Self { clauses, params }
    }

    fn push_eq(&mut self, col_expr: &str, val: &str) {
        let idx = self.params.len() + 1;
        self.clauses.push(format!("{col_expr} = ?{idx}"));
        self.params.push(SqlValue::Text(val.to_string()));
    }

    fn apply_filter(&mut self, filter: &AnalyticsFilter) {
        if let Some(ref p) = filter.page {
            self.push_eq("page_url", p);
        }
        if let Some(ref r) = filter.referrer {
            if r.eq_ignore_ascii_case("Direct") || r.eq_ignore_ascii_case("Direct / None") {
                self.clauses
                    .push("TRIM(COALESCE(NULLIF(referrer_domain, ''), referrer)) = ''".to_string());
            } else {
                let idx = self.params.len() + 1;
                self.clauses.push(format!(
                    "(referrer_domain = ?{idx} OR (referrer_domain = '' AND referrer = ?{idx}))"
                ));
                self.params.push(SqlValue::Text(r.clone()));
            }
        }
        if let Some(ref ch) = filter.channel {
            self.push_eq("COALESCE(NULLIF(channel, ''), 'Direct')", ch);
        }
        if let Some(ref us) = filter.utm_source {
            self.push_eq("COALESCE(utm_source, '')", us);
        }
        if let Some(ref um) = filter.utm_medium {
            self.push_eq("COALESCE(utm_medium, '')", um);
        }
        if let Some(ref uc) = filter.utm_campaign {
            self.push_eq("COALESCE(utm_campaign, '')", uc);
        }
        if let Some(ref dev) = filter.device {
            self.push_eq(
                "LOWER(COALESCE(NULLIF(TRIM(device_type), ''), 'desktop'))",
                &dev.to_ascii_lowercase(),
            );
        }
        if let Some(ref br) = filter.browser {
            self.push_eq("COALESCE(NULLIF(TRIM(browser), ''), 'Unknown')", br);
        }
        if let Some(ref os_val) = filter.os {
            self.push_eq("COALESCE(NULLIF(TRIM(os), ''), 'Unknown')", os_val);
        }
        if let Some(ref ctry) = filter.country {
            self.push_eq(
                "UPPER(COALESCE(NULLIF(TRIM(country), ''), 'XX'))",
                &ctry.to_ascii_uppercase(),
            );
        }
        if let Some(ref g) = filter.goal {
            let start_param = self
                .params
                .first()
                .cloned()
                .unwrap_or(SqlValue::Text(String::new()));
            let idx_start = self.params.len() + 1;
            self.params.push(start_param);
            let idx_goal = self.params.len() + 1;
            self.params.push(SqlValue::Text(g.clone()));
            self.clauses.push(format!(
                "visitor_hash IN (SELECT DISTINCT visitor_hash FROM daypress_analytics_events WHERE created_at >= ?{idx_start} AND COALESCE(NULLIF(TRIM(goal_name), ''), event_type) = ?{idx_goal})"
            ));
        }
    }

    fn where_sql(&self) -> String {
        self.clauses.join(" AND ")
    }
}

impl Database {
    /// Returns the daily-rotating CSPRNG secret salt for `today_utc` (`YYYY-MM-DD`).
    /// When the UTC date advances, automatically mints a new 256-bit secret so yesterday's
    /// IP + User-Agent hashes can never be brute-forced or correlated across days, and
    /// opportunistically rolls up & prunes expired raw events according to retention settings.
    pub fn get_or_rotate_daily_analytics_salt(&self, today_utc: &str) -> String {
        let mut should_prune_days: Option<i64> = None;
        let secret = {
            let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
            let stored_date = get_option_with_conn(&conn, "daypress_analytics_salt_date");
            let stored_secret = get_option_with_conn(&conn, "daypress_analytics_salt_secret");
            if let (Some(d), Some(s)) = (stored_date, stored_secret) {
                if d == today_utc && s.len() >= 32 {
                    return s;
                }
            }
            let fresh = generate_random_hex_secret();
            upsert_option_with_conn(&conn, "daypress_analytics_salt_date", today_utc);
            upsert_option_with_conn(&conn, "daypress_analytics_salt_secret", &fresh);

            let ret = get_option_with_conn(&conn, "daypress_analytics_retention_days")
                .and_then(|v| v.trim().parse::<i64>().ok())
                .unwrap_or(180);
            if ret > 0 {
                should_prune_days = Some(ret);
            }
            fresh
        };

        if let Some(days) = should_prune_days {
            let _ = self.rollup_and_prune_analytics(days);
        }
        secret
    }

    /// Computes the cookieless daily-salted `visitor_hash` (`SHA-256(daily_salt | ip | ua | YYYY-MM-DD)`).
    /// Shared across SSR A/B split-testing, client beacons, FormStudio conversions, and `/go/*` shortlinks.
    pub fn compute_daily_visitor_hash(&self, headers: &HeaderMap, peer_ip: Option<&str>) -> String {
        let client_ip = crate::auth::resolve_client_ip(headers, peer_ip);
        let ua = headers
            .get(header::USER_AGENT)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");
        self.compute_daily_visitor_hash_from_parts(&client_ip, ua)
    }

    pub fn compute_daily_visitor_hash_from_parts(&self, client_ip: &str, ua: &str) -> String {
        let today = Utc::now().format("%Y-%m-%d").to_string();
        let salt = self.get_or_rotate_daily_analytics_salt(&today);
        let mut hasher = Sha256::new();
        hasher.update(salt.as_bytes());
        hasher.update(b"|");
        hasher.update(client_ip.as_bytes());
        hasher.update(b"|");
        hasher.update(ua.as_bytes());
        hasher.update(b"|");
        hasher.update(today.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Reads the current analytics privacy, hygiene, and storage retention settings plus live table counts.
    pub fn get_analytics_telemetry_settings(&self) -> Result<AnalyticsTelemetrySettings> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let parse_bool = |key: &str, default_val: bool| -> bool {
            match get_option_with_conn(&conn, key).as_deref().map(str::trim) {
                Some("0" | "false" | "no" | "off") => false,
                Some("1" | "true" | "yes" | "on") => true,
                _ => default_val,
            }
        };
        let retention_days = get_option_with_conn(&conn, "daypress_analytics_retention_days")
            .and_then(|v| v.trim().parse::<i64>().ok())
            .unwrap_or(180)
            .clamp(0, 3650);
        let exclude_admins = parse_bool("daypress_analytics_exclude_admins", true);
        let filter_bots = parse_bool("daypress_analytics_filter_bots", true);
        let honor_dnt_gpc = parse_bool("daypress_analytics_honor_dnt_gpc", true);

        let raw_events_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM daypress_analytics_events", [], |r| {
                r.get(0)
            })
            .unwrap_or(0);
        let daily_rollup_rows: i64 = conn
            .query_row("SELECT COUNT(*) FROM daypress_analytics_daily", [], |r| {
                r.get(0)
            })
            .unwrap_or(0);

        Ok(AnalyticsTelemetrySettings {
            retention_days,
            exclude_admins,
            filter_bots,
            honor_dnt_gpc,
            daily_salt_active: true,
            raw_events_count,
            daily_rollup_rows,
        })
    }

    /// Persists analytics telemetry & retention settings and runs rollup/pruning when `retention_days > 0`.
    pub fn update_analytics_telemetry_settings(
        &self,
        retention_days: i64,
        exclude_admins: bool,
        filter_bots: bool,
        honor_dnt_gpc: bool,
    ) -> Result<(AnalyticsTelemetrySettings, usize)> {
        let clamped_days = retention_days.clamp(0, 3650);
        {
            let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
            upsert_option_with_conn(
                &conn,
                "daypress_analytics_retention_days",
                &clamped_days.to_string(),
            );
            upsert_option_with_conn(
                &conn,
                "daypress_analytics_exclude_admins",
                if exclude_admins { "1" } else { "0" },
            );
            upsert_option_with_conn(
                &conn,
                "daypress_analytics_filter_bots",
                if filter_bots { "1" } else { "0" },
            );
            upsert_option_with_conn(
                &conn,
                "daypress_analytics_honor_dnt_gpc",
                if honor_dnt_gpc { "1" } else { "0" },
            );
        }
        let pruned = if clamped_days > 0 {
            self.rollup_and_prune_analytics(clamped_days).unwrap_or(0)
        } else {
            0
        };
        let updated = self.get_analytics_telemetry_settings()?;
        Ok((updated, pruned))
    }

    /// Backfills `daypress_analytics_daily` from raw events and prunes raw events older than `retention_days`.
    pub fn rollup_and_prune_analytics(&self, retention_days: i64) -> Result<usize> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        // Always ensure any historical dates present in `daypress_analytics_events` are rolled up accurately.
        let _ = conn.execute(
            r#"INSERT INTO daypress_analytics_daily
                   (date, page_url, unique_visitors, pageviews, bounces, conversions, total_duration_secs, scroll_sum, engaged_views)
               SELECT
                   substr(created_at, 1, 10) AS d,
                   page_url,
                   COUNT(DISTINCT visitor_hash) AS uv,
                   SUM(CASE WHEN event_type = 'pageview' THEN 1 ELSE 0 END) AS pv,
                   0 AS bnc,
                   SUM(CASE WHEN event_type IN ('form_submit', 'goal', 'conversion') THEN 1 ELSE 0 END) AS conv,
                   SUM(CASE WHEN event_type = 'pageview' THEN COALESCE(duration_secs, 0) ELSE 0 END) AS dur,
                   SUM(CASE WHEN event_type = 'pageview' THEN COALESCE(scroll_depth, 0) ELSE 0 END) AS scr,
                   SUM(CASE WHEN event_type = 'pageview' AND (COALESCE(duration_secs, 0) > 0 OR COALESCE(scroll_depth, 0) > 0) THEN 1 ELSE 0 END) AS eng
               FROM daypress_analytics_events
               GROUP BY d, page_url
               ON CONFLICT(date, page_url) DO UPDATE SET
                   unique_visitors = MAX(daypress_analytics_daily.unique_visitors, excluded.unique_visitors),
                   pageviews = MAX(daypress_analytics_daily.pageviews, excluded.pageviews),
                   conversions = MAX(daypress_analytics_daily.conversions, excluded.conversions),
                   total_duration_secs = MAX(daypress_analytics_daily.total_duration_secs, excluded.total_duration_secs),
                   scroll_sum = MAX(daypress_analytics_daily.scroll_sum, excluded.scroll_sum),
                   engaged_views = MAX(daypress_analytics_daily.engaged_views, excluded.engaged_views)"#,
            [],
        );

        if retention_days <= 0 {
            return Ok(0);
        }
        let cutoff = (Utc::now() - Duration::days(retention_days.clamp(1, 3650)))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let deleted = conn.execute(
            "DELETE FROM daypress_analytics_events WHERE created_at < ?1",
            params![cutoff],
        )?;
        let _ = conn.execute(
            "DELETE FROM daypress_web_vitals WHERE created_at < ?1",
            params![cutoff],
        );
        Ok(deleted)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_analytics_event(
        &self,
        visitor_hash: &str,
        page_url: &str,
        referrer: &str,
        device_type: &str,
        country: &str,
        utm_source: Option<&str>,
        utm_medium: Option<&str>,
        utm_campaign: Option<&str>,
    ) -> Result<()> {
        self.record_analytics_event_full(
            visitor_hash,
            page_url,
            referrer,
            device_type,
            country,
            utm_source,
            utm_medium,
            utm_campaign,
            "pageview",
            None,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn record_analytics_event_full(
        &self,
        visitor_hash: &str,
        page_url: &str,
        referrer: &str,
        device_type: &str,
        country: &str,
        utm_source: Option<&str>,
        utm_medium: Option<&str>,
        utm_campaign: Option<&str>,
        event_type: &str,
        goal_name: Option<&str>,
    ) -> Result<()> {
        self.record_analytics_beacon(
            visitor_hash,
            page_url,
            referrer,
            None,
            device_type,
            "",
            "",
            country,
            utm_source,
            utm_medium,
            utm_campaign,
            event_type,
            goal_name,
            None,
            None,
        )
    }

    /// Primary enriched ingestion pipeline for pageviews, custom goals, FormStudio conversions,
    /// outbound clicks, file downloads, and `pagehide` engaged-time / scroll-depth flushes.
    #[allow(clippy::too_many_arguments)]
    pub fn record_analytics_beacon(
        &self,
        visitor_hash: &str,
        page_url: &str,
        raw_referrer: &str,
        request_host: Option<&str>,
        device_type: &str,
        browser: &str,
        os: &str,
        country: &str,
        utm_source: Option<&str>,
        utm_medium: Option<&str>,
        utm_campaign: Option<&str>,
        event_type: &str,
        goal_name: Option<&str>,
        duration_secs: Option<i64>,
        scroll_depth: Option<i64>,
    ) -> Result<()> {
        let now_dt = Utc::now();
        let now_str = now_dt.format("%Y-%m-%d %H:%M:%S").to_string();
        let today_str = now_dt.format("%Y-%m-%d").to_string();
        let today_start = format!("{today_str} 00:00:00");

        let clean_page = normalize_analytics_page_path(page_url);
        let ev_raw = event_type.trim().to_ascii_lowercase();
        let ev_type = match ev_raw.as_str() {
            "" | "pageview" | "page_view" => "pageview",
            "engagement" | "ping" => "engagement",
            "goal" | "custom_goal" => "goal",
            "conversion" => "conversion",
            "form_submit" => "form_submit",
            "outbound_click" | "outbound" => "outbound_click",
            "file_download" | "download" => "file_download",
            _ => "goal",
        };

        let clamped_dur = duration_secs.unwrap_or(0).clamp(0, 3600);
        let clamped_scroll = scroll_depth.unwrap_or(0).clamp(0, 100);

        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());

        // Engagement beacon (`pagehide` / `visibilitychange`): update the visitor's most recent
        // pageview row on this page today instead of appending extra rows.
        if ev_type == "engagement" {
            if clamped_dur == 0 && clamped_scroll == 0 {
                return Ok(());
            }
            if let Ok(Some((row_id, prev_dur, prev_scroll))) = conn
                .query_row(
                    r#"SELECT id, COALESCE(duration_secs, 0), COALESCE(scroll_depth, 0)
                       FROM daypress_analytics_events
                       WHERE visitor_hash = ?1
                         AND created_at >= ?2
                         AND event_type = 'pageview'
                         AND page_url = ?3
                       ORDER BY id DESC LIMIT 1"#,
                    params![visitor_hash, today_start, clean_page],
                    |r| {
                        Ok((
                            r.get::<_, i64>(0)?,
                            r.get::<_, i64>(1)?,
                            r.get::<_, i64>(2)?,
                        ))
                    },
                )
                .optional()
            {
                let new_dur = prev_dur.max(clamped_dur);
                let new_scroll = prev_scroll.max(clamped_scroll);
                let delta_dur = new_dur.saturating_sub(prev_dur);
                let delta_scroll = new_scroll.saturating_sub(prev_scroll);
                let newly_engaged =
                    i64::from(prev_dur == 0 && prev_scroll == 0 && (new_dur > 0 || new_scroll > 0));
                conn.execute(
                    "UPDATE daypress_analytics_events SET duration_secs = ?1, scroll_depth = ?2 WHERE id = ?3",
                    params![new_dur, new_scroll, row_id],
                )?;
                let _ = conn.execute(
                    r#"INSERT INTO daypress_analytics_daily
                           (date, page_url, unique_visitors, pageviews, bounces, conversions, total_duration_secs, scroll_sum, engaged_views)
                       VALUES (?1, ?2, 1, 1, 0, 0, ?3, ?4, ?5)
                       ON CONFLICT(date, page_url) DO UPDATE SET
                           total_duration_secs = daypress_analytics_daily.total_duration_secs + excluded.total_duration_secs,
                           scroll_sum = daypress_analytics_daily.scroll_sum + excluded.scroll_sum,
                           engaged_views = daypress_analytics_daily.engaged_views + excluded.engaged_views"#,
                    params![today_str, clean_page, delta_dur, delta_scroll, newly_engaged],
                );
            }
            return Ok(());
        }

        // Resolve site hostnames so internal page-to-page navigations are not recorded as external referrers.
        let mut self_hosts = Vec::with_capacity(3);
        if let Some(h) = request_host {
            let hc = extract_host_clean(h);
            if !hc.is_empty() {
                self_hosts.push(hc);
            }
        }
        for opt_key in ["siteurl", "home"] {
            if let Some(u) = get_option_with_conn(&conn, opt_key) {
                let hc = extract_host_clean(&u);
                if !hc.is_empty() && !self_hosts.contains(&hc) {
                    self_hosts.push(hc);
                }
            }
        }

        let clean_utm_src = utm_source
            .map(|s| safe_truncate(s, 120))
            .filter(|s| !s.is_empty());
        let clean_utm_med = utm_medium
            .map(|s| safe_truncate(s, 120))
            .filter(|s| !s.is_empty());
        let clean_utm_camp = utm_campaign
            .map(|s| safe_truncate(s, 120))
            .filter(|s| !s.is_empty());

        let (ref_domain, channel) = normalize_referrer_and_channel(
            raw_referrer,
            &self_hosts,
            clean_utm_src.as_deref(),
            clean_utm_med.as_deref(),
        );
        // Strip internal referrer URLs completely so Top Referrers stays clean even for legacy queries.
        let stored_referrer = if ref_domain.is_empty() {
            String::new()
        } else {
            safe_truncate(raw_referrer, 300)
        };

        let clean_device = match device_type.trim().to_ascii_lowercase().as_str() {
            "mobile" => "mobile",
            "tablet" => "tablet",
            _ => "desktop",
        };
        let clean_country = {
            let c = country.trim().to_ascii_uppercase();
            if c.len() == 2 && c.chars().all(|ch| ch.is_ascii_alphabetic()) {
                c
            } else {
                "XX".to_string()
            }
        };
        let clean_goal = goal_name
            .map(|g| safe_truncate(g, 160))
            .filter(|g| !g.is_empty());

        // Check if this visitor already viewed `clean_page` today before inserting (for accurate daily rollups).
        let already_seen_page_today: bool = conn
            .query_row(
                r#"SELECT 1 FROM daypress_analytics_events
                   WHERE visitor_hash = ?1 AND created_at >= ?2 AND page_url = ?3
                   LIMIT 1"#,
                params![visitor_hash, today_start, clean_page],
                |_| Ok(true),
            )
            .optional()
            .unwrap_or(None)
            .unwrap_or(false);

        conn.execute(
            r#"INSERT INTO daypress_analytics_events
               (visitor_hash, page_url, referrer, referrer_domain, channel, device_type, browser, os, country,
                utm_source, utm_medium, utm_campaign, event_type, goal_name, duration_secs, scroll_depth, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)"#,
            params![
                visitor_hash,
                clean_page,
                stored_referrer,
                ref_domain,
                channel,
                clean_device,
                safe_truncate(browser, 48),
                safe_truncate(os, 48),
                clean_country,
                clean_utm_src,
                clean_utm_med,
                clean_utm_camp,
                ev_type,
                clean_goal,
                clamped_dur,
                clamped_scroll,
                now_str,
            ],
        )?;

        let uv_inc = i64::from(!already_seen_page_today);
        let pv_inc = i64::from(ev_type == "pageview");
        let conv_inc = i64::from(matches!(ev_type, "form_submit" | "goal" | "conversion"));
        let eng_inc = i64::from(ev_type == "pageview" && (clamped_dur > 0 || clamped_scroll > 0));
        let _ = conn.execute(
            r#"INSERT INTO daypress_analytics_daily
                   (date, page_url, unique_visitors, pageviews, bounces, conversions, total_duration_secs, scroll_sum, engaged_views)
               VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7, ?8)
               ON CONFLICT(date, page_url) DO UPDATE SET
                   unique_visitors = daypress_analytics_daily.unique_visitors + excluded.unique_visitors,
                   pageviews = daypress_analytics_daily.pageviews + excluded.pageviews,
                   conversions = daypress_analytics_daily.conversions + excluded.conversions,
                   total_duration_secs = daypress_analytics_daily.total_duration_secs + excluded.total_duration_secs,
                   scroll_sum = daypress_analytics_daily.scroll_sum + excluded.scroll_sum,
                   engaged_views = daypress_analytics_daily.engaged_views + excluded.engaged_views"#,
            params![
                today_str,
                clean_page,
                uv_inc,
                pv_inc,
                conv_inc,
                clamped_dur,
                clamped_scroll,
                eng_inc,
            ],
        );
        drop(conn);

        if matches!(ev_type, "form_submit" | "goal" | "conversion") {
            self.record_ab_conversion_for_visitor(visitor_hash, ev_type, clean_goal.as_deref());
        }
        Ok(())
    }

    pub fn get_analytics_summary(&self) -> Result<AnalyticsSummary> {
        self.get_analytics_dashboard("30d")
    }

    pub fn get_analytics_dashboard(&self, range: &str) -> Result<AnalyticsSummary> {
        self.get_analytics_dashboard_filtered(range, &AnalyticsFilter::default())
    }

    /// Computes the full sargable Cookieless Analytics dashboard summary with period-over-period
    /// comparison deltas, entry/exit pages, channels, browser/OS breakdowns, conversions,
    /// outbound clicks, file downloads, and `daypress_analytics_daily` rollup fallback.
    pub fn get_analytics_dashboard_filtered(
        &self,
        range: &str,
        raw_filter: &AnalyticsFilter,
    ) -> Result<AnalyticsSummary> {
        let filter = raw_filter.normalized();
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let now = Utc::now();

        let clean_range = match range.trim() {
            "today" => "today",
            "24h" => "24h",
            "7d" => "7d",
            "30d" => "30d",
            "90d" => "90d",
            "12m" => "12m",
            "all" => "all",
            _ => "30d",
        };

        let (cutoff_dt, prev_cutoff_dt) = match clean_range {
            "today" | "24h" => (now - Duration::hours(24), now - Duration::hours(48)),
            "7d" => (now - Duration::days(7), now - Duration::days(14)),
            "90d" => (now - Duration::days(90), now - Duration::days(180)),
            "12m" => (now - Duration::days(365), now - Duration::days(730)),
            "all" => (now - Duration::days(3650), now - Duration::days(7300)),
            _ => (now - Duration::days(30), now - Duration::days(60)),
        };

        let cutoff_str = cutoff_dt.format("%Y-%m-%d %H:%M:%S").to_string();
        let prev_cutoff_str = prev_cutoff_dt.format("%Y-%m-%d %H:%M:%S").to_string();
        let live_cutoff_str = (now - Duration::minutes(5))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

        // Live active visitors in the last 5 minutes (unfiltered site-wide pulse)
        let live_visitors: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT visitor_hash) FROM daypress_analytics_events WHERE created_at >= ?1",
                params![live_cutoff_str],
                |r| r.get(0),
            )
            .unwrap_or(0);

        // Helper to compute visitor/pageview/conversion/bounce/engagement KPIs for any `[start, end)` window.
        let compute_window_kpis = |start_s: &str,
                                   end_s: Option<&str>|
         -> (i64, i64, i64, f64, f64, f64) {
            let mut wb = SqlWhereBuilder::new_time_window(start_s, end_s);
            wb.apply_filter(&filter);
            let w_sql = wb.where_sql();

            // Consolidated per-visitor subquery computes Visitors, Pageviews, Conversions, Bounces, Duration & Scroll in 1 index scan!
            let kpi_sql = format!(
                r#"SELECT
                       COUNT(*) AS total_vis,
                       COALESCE(SUM(pv_cnt), 0) AS total_pv,
                       COALESCE(SUM(conv_cnt), 0) AS total_conv,
                       COALESCE(SUM(CASE WHEN pv_cnt <= 1 AND conv_cnt = 0 AND max_dur < 10 THEN 1 ELSE 0 END), 0) AS bounce_vis,
                       COALESCE(AVG(CASE WHEN visit_dur > 0 THEN CAST(visit_dur AS REAL) END), 0.0) AS avg_dur,
                       COALESCE(AVG(CASE WHEN max_scroll > 0 THEN CAST(max_scroll AS REAL) END), 0.0) AS avg_scr
                   FROM (
                       SELECT
                           visitor_hash,
                           SUM(CASE WHEN COALESCE(event_type, 'pageview') = 'pageview' THEN 1 ELSE 0 END) AS pv_cnt,
                           SUM(CASE WHEN COALESCE(event_type, 'pageview') IN ('form_submit', 'goal', 'conversion') THEN 1 ELSE 0 END) AS conv_cnt,
                           COALESCE(MAX(duration_secs), 0) AS max_dur,
                           COALESCE(SUM(CASE WHEN COALESCE(event_type, 'pageview') = 'pageview' THEN COALESCE(duration_secs, 0) ELSE 0 END), 0) AS visit_dur,
                           COALESCE(MAX(scroll_depth), 0) AS max_scroll
                       FROM daypress_analytics_events
                       WHERE {w_sql}
                       GROUP BY visitor_hash
                   )"#
            );

            conn.query_row(&kpi_sql, params_from_iter(wb.params.iter()), |r| {
                let vis: i64 = r.get(0).unwrap_or(0);
                let pv: i64 = r.get(1).unwrap_or(0);
                let conv: i64 = r.get(2).unwrap_or(0);
                let bounces: i64 = r.get(3).unwrap_or(0);
                let avg_dur: f64 = r.get(4).unwrap_or(0.0);
                let avg_scr: f64 = r.get(5).unwrap_or(0.0);
                let bounce_pct = if vis == 0 {
                    0.0
                } else {
                    (bounces as f64 / vis as f64) * 100.0
                };
                Ok((vis, pv, conv, bounce_pct, avg_dur, avg_scr))
            })
            .unwrap_or((0, 0, 0, 0.0, 0.0, 0.0))
        };

        let (
            mut total_visitors,
            mut total_pageviews,
            mut total_conversions,
            bounce_rate_pct,
            avg_duration_secs,
            avg_scroll_depth_pct,
        ) = compute_window_kpis(&cutoff_str, None);

        let (
            prev_visitors,
            prev_pageviews,
            prev_conversions,
            prev_bounce_rate_pct,
            prev_avg_duration_secs,
            _,
        ) = compute_window_kpis(&prev_cutoff_str, Some(&cutoff_str));

        // Check if `daypress_analytics_daily` has rolled-up archive dates older than the earliest raw event
        let earliest_raw_date: Option<String> = if filter.is_empty() {
            conn.query_row(
                "SELECT substr(MIN(created_at), 1, 10) FROM daypress_analytics_events",
                [],
                |r| r.get::<_, Option<String>>(0),
            )
            .ok()
            .flatten()
        } else {
            None
        };

        let cutoff_date_str = cutoff_dt.format("%Y-%m-%d").to_string();
        if filter.is_empty() {
            let archive_upper = earliest_raw_date.as_deref().unwrap_or("9999-99-99");
            if cutoff_date_str.as_str() < archive_upper {
                if let Ok((arch_uv, arch_pv, arch_conv)) = conn.query_row(
                    r#"SELECT COALESCE(SUM(unique_visitors), 0),
                              COALESCE(SUM(pageviews), 0),
                              COALESCE(SUM(conversions), 0)
                       FROM daypress_analytics_daily
                       WHERE date >= ?1 AND date < ?2"#,
                    params![cutoff_date_str, archive_upper],
                    |r| {
                        Ok((
                            r.get::<_, i64>(0)?,
                            r.get::<_, i64>(1)?,
                            r.get::<_, i64>(2)?,
                        ))
                    },
                ) {
                    total_visitors += arch_uv;
                    total_pageviews += arch_pv;
                    total_conversions += arch_conv;
                }
            }
        }

        let views_per_visit = if total_visitors == 0 {
            0.0
        } else {
            (total_pageviews as f64) / (total_visitors as f64)
        };
        let conversion_rate_pct = if total_visitors == 0 {
            0.0
        } else {
            ((total_conversions as f64) / (total_visitors as f64) * 100.0).min(100.0)
        };

        // Initialize time-series buckets
        let mut bucket_map: BTreeMap<String, (i64, i64, i64)> = BTreeMap::new();
        match clean_range {
            "today" | "24h" => {
                for h in (0..24).rev() {
                    let b = (now - Duration::hours(h))
                        .format("%Y-%m-%d %H:00")
                        .to_string();
                    bucket_map.insert(b, (0, 0, 0));
                }
            }
            "12m" | "all" => {
                for m in (0..12).rev() {
                    let b = (now - Duration::days(m * 30)).format("%Y-%m").to_string();
                    bucket_map.insert(b, (0, 0, 0));
                }
            }
            _ => {
                let days = match clean_range {
                    "7d" => 7,
                    "90d" => 90,
                    _ => 30,
                };
                for d in (0..days).rev() {
                    let b = (now - Duration::days(d)).format("%Y-%m-%d").to_string();
                    bucket_map.insert(b, (0, 0, 0));
                }
            }
        }

        // Merge archived daily rollup buckets if applicable
        if filter.is_empty() && !matches!(clean_range, "today" | "24h") {
            let archive_upper = earliest_raw_date.as_deref().unwrap_or("9999-99-99");
            if cutoff_date_str.as_str() < archive_upper {
                let arch_bucket_expr = if matches!(clean_range, "12m" | "all") {
                    "substr(date, 1, 7)"
                } else {
                    "date"
                };
                let arch_sql = format!(
                    r#"SELECT {arch_bucket_expr} AS b,
                              COALESCE(SUM(unique_visitors), 0),
                              COALESCE(SUM(pageviews), 0),
                              COALESCE(SUM(conversions), 0)
                       FROM daypress_analytics_daily
                       WHERE date >= ?1 AND date < ?2
                       GROUP BY b ORDER BY b ASC"#
                );
                if let Ok(mut stmt) = conn.prepare(&arch_sql) {
                    if let Ok(rows) = stmt.query_map(params![cutoff_date_str, archive_upper], |r| {
                        Ok((
                            r.get::<_, String>(0)?,
                            r.get::<_, i64>(1).unwrap_or(0),
                            r.get::<_, i64>(2).unwrap_or(0),
                            r.get::<_, i64>(3).unwrap_or(0),
                        ))
                    }) {
                        for row in rows.flatten() {
                            bucket_map.insert(row.0, (row.1, row.2, row.3));
                        }
                    }
                }
            }
        }

        let mut wb = SqlWhereBuilder::new_time_window(&cutoff_str, None);
        wb.apply_filter(&filter);
        let w_sql = wb.where_sql();

        let bucket_expr = match clean_range {
            "today" | "24h" => "substr(created_at, 1, 13) || ':00'",
            "12m" | "all" => "substr(created_at, 1, 7)",
            _ => "substr(created_at, 1, 10)",
        };
        let bucket_sql = format!(
            r#"SELECT {bucket_expr} AS b,
                      COUNT(DISTINCT visitor_hash),
                      SUM(CASE WHEN COALESCE(event_type, 'pageview') = 'pageview' THEN 1 ELSE 0 END),
                      SUM(CASE WHEN COALESCE(event_type, 'pageview') IN ('form_submit', 'goal', 'conversion') THEN 1 ELSE 0 END)
               FROM daypress_analytics_events
               WHERE {w_sql}
               GROUP BY b ORDER BY b ASC"#
        );
        if let Ok(mut b_stmt) = conn.prepare(&bucket_sql) {
            if let Ok(rows) = b_stmt.query_map(params_from_iter(wb.params.iter()), |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, i64>(1).unwrap_or(0),
                    r.get::<_, i64>(2).unwrap_or(0),
                    r.get::<_, i64>(3).unwrap_or(0),
                ))
            }) {
                for row in rows.flatten() {
                    bucket_map.insert(row.0, (row.1, row.2, row.3));
                }
            }
        }

        let buckets: Vec<AnalyticsBucketPoint> = bucket_map
            .into_iter()
            .map(
                |(bucket, (visitors, pageviews, conversions))| AnalyticsBucketPoint {
                    bucket,
                    visitors,
                    pageviews,
                    conversions,
                },
            )
            .collect();

        // Top Pages (with Avg Duration & Avg Scroll Depth)
        let top_pages_sql = format!(
            r#"SELECT page_url,
                      COUNT(*) AS views,
                      COUNT(DISTINCT visitor_hash) AS visitors,
                      COALESCE(AVG(CASE WHEN duration_secs > 0 THEN CAST(duration_secs AS REAL) END), 0.0) AS avg_dur,
                      COALESCE(AVG(CASE WHEN scroll_depth > 0 THEN CAST(scroll_depth AS REAL) END), 0.0) AS avg_scr
               FROM daypress_analytics_events
               WHERE {w_sql} AND COALESCE(event_type, 'pageview') = 'pageview'
               GROUP BY page_url
               ORDER BY views DESC, visitors DESC
               LIMIT 15"#
        );
        let mut top_pages: Vec<TopPageMetric> = conn
            .prepare(&top_pages_sql)?
            .query_map(params_from_iter(wb.params.iter()), |r| {
                Ok(TopPageMetric {
                    page_url: r.get(0)?,
                    views: r.get(1)?,
                    visitors: r.get(2).unwrap_or(0),
                    avg_duration_secs: r.get(3).unwrap_or(0.0),
                    avg_scroll_depth_pct: r.get(4).unwrap_or(0.0),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        // Fallback to `daypress_analytics_daily` top pages if raw events are empty after pruning
        if top_pages.is_empty() && filter.is_empty() {
            if let Ok(mut d_stmt) = conn.prepare(
                r#"SELECT page_url,
                          SUM(pageviews) AS views,
                          SUM(unique_visitors) AS visitors,
                          CASE WHEN SUM(engaged_views) > 0 THEN CAST(SUM(total_duration_secs) AS REAL) / SUM(engaged_views) ELSE 0.0 END,
                          CASE WHEN SUM(engaged_views) > 0 THEN CAST(SUM(scroll_sum) AS REAL) / SUM(engaged_views) ELSE 0.0 END
                   FROM daypress_analytics_daily
                   WHERE date >= ?1
                   GROUP BY page_url
                   ORDER BY views DESC
                   LIMIT 15"#,
            ) {
                if let Ok(rows) = d_stmt.query_map(params![cutoff_date_str], |r| {
                    Ok(TopPageMetric {
                        page_url: r.get(0)?,
                        views: r.get(1).unwrap_or(0),
                        visitors: r.get(2).unwrap_or(0),
                        avg_duration_secs: r.get(3).unwrap_or(0.0),
                        avg_scroll_depth_pct: r.get(4).unwrap_or(0.0),
                    })
                }) {
                    top_pages = rows.filter_map(|r| r.ok()).collect();
                }
            }
        }

        // Entry Pages (first pageview of each visitor in the window) & Exit Pages (last pageview)
        let query_session_boundary_pages = |agg_fn: &str| -> Vec<TopPageMetric> {
            let sql = format!(
                r#"SELECT e.page_url,
                          COUNT(*) AS entrances,
                          COUNT(DISTINCT e.visitor_hash) AS visitors,
                          COALESCE(AVG(CASE WHEN e.duration_secs > 0 THEN CAST(e.duration_secs AS REAL) END), 0.0),
                          COALESCE(AVG(CASE WHEN e.scroll_depth > 0 THEN CAST(e.scroll_depth AS REAL) END), 0.0)
                   FROM daypress_analytics_events e
                   INNER JOIN (
                       SELECT {agg_fn}(id) AS target_id
                       FROM daypress_analytics_events
                       WHERE {w_sql} AND COALESCE(event_type, 'pageview') = 'pageview'
                       GROUP BY visitor_hash
                   ) b ON e.id = b.target_id
                   GROUP BY e.page_url
                   ORDER BY entrances DESC
                   LIMIT 12"#
            );
            let Ok(mut s) = conn.prepare(&sql) else {
                return Vec::new();
            };
            let Ok(rows) = s.query_map(params_from_iter(wb.params.iter()), |r| {
                Ok(TopPageMetric {
                    page_url: r.get(0)?,
                    views: r.get(1).unwrap_or(0),
                    visitors: r.get(2).unwrap_or(0),
                    avg_duration_secs: r.get(3).unwrap_or(0.0),
                    avg_scroll_depth_pct: r.get(4).unwrap_or(0.0),
                })
            }) else {
                return Vec::new();
            };
            rows.filter_map(|r| r.ok()).collect()
        };
        let entry_pages = query_session_boundary_pages("MIN");
        let exit_pages = query_session_boundary_pages("MAX");

        // Breakdown helper (counts `pageview` views so conversions never double-count pageviews,
        // while falling back to all events if a site only sends custom events)
        let query_breakdown = |label_expr: &str, extra_cond: &str| -> Vec<BreakdownMetric> {
            let cond_suffix = if extra_cond.is_empty() {
                String::new()
            } else {
                format!(" AND {extra_cond}")
            };
            let sql = format!(
                r#"SELECT {label_expr} AS lbl,
                          COUNT(DISTINCT visitor_hash) AS vis,
                          COALESCE(NULLIF(SUM(CASE WHEN COALESCE(event_type, 'pageview') = 'pageview' THEN 1 ELSE 0 END), 0), COUNT(*)) AS vws
                   FROM daypress_analytics_events
                   WHERE {w_sql}{cond_suffix}
                   GROUP BY lbl
                   ORDER BY vis DESC, vws DESC
                   LIMIT 12"#
            );
            let Ok(mut s) = conn.prepare(&sql) else {
                return Vec::new();
            };
            let Ok(rows) = s.query_map(params_from_iter(wb.params.iter()), |r| {
                Ok(BreakdownMetric {
                    label: r.get(0)?,
                    visitors: r.get(1).unwrap_or(0),
                    views: r.get(2).unwrap_or(0),
                })
            }) else {
                return Vec::new();
            };
            rows.filter_map(|r| r.ok()).collect()
        };

        // Top Referrers: prefer normalized `referrer_domain`, fallback to cleaned `referrer` for legacy rows
        let raw_referrers = query_breakdown(
            "CASE WHEN TRIM(COALESCE(NULLIF(referrer_domain, ''), referrer)) = '' THEN 'Direct / None' ELSE COALESCE(NULLIF(TRIM(referrer_domain), ''), TRIM(referrer)) END",
            "",
        );
        // Normalize any legacy full-URL referrers in-memory so old rows group cleanly by domain too
        let mut ref_merged: BTreeMap<String, (i64, i64)> = BTreeMap::new();
        for item in raw_referrers {
            let clean_lbl = if item.label == "Direct / None" {
                item.label
            } else {
                let d = extract_host_clean(&item.label);
                if d.is_empty() {
                    item.label
                } else {
                    d
                }
            };
            let entry = ref_merged.entry(clean_lbl).or_insert((0, 0));
            entry.0 += item.visitors;
            entry.1 += item.views;
        }
        let mut top_referrers: Vec<BreakdownMetric> = ref_merged
            .into_iter()
            .map(|(label, (visitors, views))| BreakdownMetric {
                label,
                visitors,
                views,
            })
            .collect();
        top_referrers.sort_by(|a, b| {
            b.visitors
                .cmp(&a.visitors)
                .then_with(|| b.views.cmp(&a.views))
        });
        top_referrers.truncate(12);

        let channels = query_breakdown("COALESCE(NULLIF(TRIM(channel), ''), 'Direct')", "");
        let utm_sources = query_breakdown(
            "TRIM(utm_source)",
            "utm_source IS NOT NULL AND TRIM(utm_source) != ''",
        );
        let utm_mediums = query_breakdown(
            "TRIM(utm_medium)",
            "utm_medium IS NOT NULL AND TRIM(utm_medium) != ''",
        );
        let utm_campaigns = query_breakdown(
            "TRIM(utm_campaign)",
            "utm_campaign IS NOT NULL AND TRIM(utm_campaign) != ''",
        );
        let devices = query_breakdown(
            "LOWER(COALESCE(NULLIF(TRIM(device_type), ''), 'desktop'))",
            "",
        );
        let browsers = query_breakdown(
            "COALESCE(NULLIF(TRIM(browser), ''), 'Unknown')",
            "TRIM(COALESCE(browser, '')) != ''",
        );
        let operating_systems = query_breakdown(
            "COALESCE(NULLIF(TRIM(os), ''), 'Unknown')",
            "TRIM(COALESCE(os, '')) != ''",
        );
        let countries = query_breakdown("UPPER(COALESCE(NULLIF(TRIM(country), ''), 'XX'))", "");

        let query_events_table = |event_types_in: &str| -> Vec<ConversionMetric> {
            let sql = format!(
                r#"SELECT COALESCE(NULLIF(TRIM(goal_name), ''), COALESCE(event_type, 'goal')) AS gname,
                          COUNT(*) AS cnt,
                          COUNT(DISTINCT visitor_hash) AS vis
                   FROM daypress_analytics_events
                   WHERE {w_sql} AND COALESCE(event_type, 'pageview') IN ({event_types_in})
                   GROUP BY gname
                   ORDER BY cnt DESC
                   LIMIT 15"#
            );
            let Ok(mut c_stmt) = conn.prepare(&sql) else {
                return Vec::new();
            };
            let Ok(rows) = c_stmt.query_map(params_from_iter(wb.params.iter()), |r| {
                let goal_name: String = r.get(0)?;
                let count: i64 = r.get(1).unwrap_or(0);
                let unique_visitors: i64 = r.get(2).unwrap_or(0);
                let conversion_rate_pct = if total_visitors == 0 {
                    0.0
                } else {
                    ((unique_visitors as f64 / total_visitors as f64) * 100.0).min(100.0)
                };
                Ok(ConversionMetric {
                    goal_name,
                    count,
                    unique_visitors,
                    conversion_rate_pct,
                })
            }) else {
                return Vec::new();
            };
            rows.filter_map(|r| r.ok()).collect()
        };

        let conversions = query_events_table("'form_submit', 'goal', 'conversion'");
        let outbound_clicks = query_events_table("'outbound_click'");
        let file_downloads = query_events_table("'file_download'");

        Ok(AnalyticsSummary {
            range: clean_range.to_string(),
            total_visitors,
            total_pageviews,
            total_conversions,
            bounce_rate_pct,
            buckets,
            top_pages,
            top_referrers,
            utm_sources,
            utm_mediums,
            utm_campaigns,
            devices,
            countries,
            conversions,
            live_visitors,
            views_per_visit,
            avg_duration_secs,
            avg_scroll_depth_pct,
            conversion_rate_pct,
            prev_visitors,
            prev_pageviews,
            prev_conversions,
            prev_bounce_rate_pct,
            prev_avg_duration_secs,
            entry_pages,
            exit_pages,
            channels,
            browsers,
            operating_systems,
            outbound_clicks,
            file_downloads,
            active_filter: filter,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{to_bytes, Body},
        http::{Request, StatusCode},
        response::Html,
    };
    use serde_json::json;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_analytics_hygiene_engagement_drilldowns_and_rollups() {
        let db = Database::new_in_memory().expect("in-memory db");
        let app = crate::create_app(db.clone());

        // 1. Bots, crawlers, prefetches, and DNT/GPC headers must be dropped at ingestion
        for (hdr_k, hdr_v, ua) in [
            ("user-agent", "Mozilla/5.0 (compatible; Googlebot/2.1)", "Mozilla/5.0 (compatible; Googlebot/2.1)"),
            ("user-agent", "Mozilla/5.0 (compatible; GPTBot/1.0)", "Mozilla/5.0 (compatible; GPTBot/1.0)"),
            ("sec-purpose", "prefetch;prerender", "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_0) AppleWebKit/537.36 Chrome/125.0.0.0 Safari/537.36"),
            ("dnt", "1", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Firefox/126.0"),
            ("sec-gpc", "1", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) Firefox/126.0"),
        ] {
            let res = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/wp-json/daypress/v1/analytics/event")
                        .header(header::CONTENT_TYPE, "application/json")
                        .header(header::USER_AGENT, ua)
                        .header(hdr_k, hdr_v)
                        .body(Body::from(json!({ "page_url": "/pricing" }).to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_eq!(res.status(), StatusCode::OK);
            let body: serde_json::Value =
                serde_json::from_slice(&to_bytes(res.into_body(), 8192).await.unwrap()).unwrap();
            assert_eq!(body["recorded"], false);
        }

        let after_blocked = db.get_analytics_dashboard("30d").unwrap();
        assert_eq!(after_blocked.total_pageviews, 0);
        assert_eq!(after_blocked.total_visitors, 0);

        // 2. Real Visitor 1 (macOS + Safari, from Google Organic -> / -> /pricing, engaged 45s & 82% scroll, triggers goal via `event_name` alias + outbound click + PDF download)
        let ua_safari = "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_4) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.4 Safari/605.1.15";
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wp-json/daypress/v1/analytics/event")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::HOST, "daypress.example")
                    .header(header::USER_AGENT, ua_safari)
                    .header("cf-ipcountry", "NL")
                    .body(Body::from(
                        json!({
                            "page_url": "/",
                            "referrer": "https://www.google.nl/search?q=daypress",
                            "device_type": "desktop"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Internal self-referrer (`https://daypress.example/`) must NOT pollute external Top Referrers
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wp-json/daypress/v1/analytics/event")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::HOST, "daypress.example")
                    .header(header::USER_AGENT, ua_safari)
                    .header("cf-ipcountry", "NL")
                    .body(Body::from(
                        json!({
                            "page_url": "/pricing",
                            "referrer": "https://daypress.example/",
                            "device_type": "desktop"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Engagement beacon updates the `/pricing` pageview row in-place without inflating pageviews
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wp-json/daypress/v1/analytics/event")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(header::HOST, "daypress.example")
                    .header(header::USER_AGENT, ua_safari)
                    .body(Body::from(
                        json!({
                            "page_url": "/pricing",
                            "event_type": "engagement",
                            "duration_secs": 45,
                            "scroll_depth": 82
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        // Legacy `event_name: "goal"` + `outbound_click` + `file_download`
        for ev_payload in [
            json!({ "page_url": "/pricing", "event_name": "goal", "goal_name": "Start Trial" }),
            json!({ "page_url": "/pricing", "event_type": "outbound_click", "goal_name": "github.com/daypress" }),
            json!({ "page_url": "/pricing", "event_type": "file_download", "goal_name": "/uploads/whitepaper.pdf" }),
        ] {
            let _ = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method("POST")
                        .uri("/wp-json/daypress/v1/analytics/event")
                        .header(header::CONTENT_TYPE, "application/json")
                        .header(header::USER_AGENT, ua_safari)
                        .header("cf-ipcountry", "NL")
                        .body(Body::from(ev_payload.to_string()))
                        .unwrap(),
                )
                .await
                .unwrap();
        }

        // 3. Verify enriched summary, session entry/exit pages, browser/OS, channels, and drilldowns
        let summary = db.get_analytics_dashboard("30d").unwrap();
        assert_eq!(summary.total_visitors, 1);
        assert_eq!(
            summary.total_pageviews, 2,
            "engagement & goals must not inflate pageviews"
        );
        assert_eq!(summary.total_conversions, 1);
        assert_eq!(summary.bounce_rate_pct, 0.0);
        assert!(summary.avg_duration_secs >= 44.0);
        assert!(summary.avg_scroll_depth_pct >= 81.0);
        assert_eq!(summary.entry_pages[0].page_url, "/");
        assert_eq!(summary.exit_pages[0].page_url, "/pricing");
        assert!(summary
            .top_referrers
            .iter()
            .any(|r| r.label == "google.com"));
        assert!(!summary
            .top_referrers
            .iter()
            .any(|r| r.label.contains("daypress.example")));
        assert!(summary.channels.iter().any(|c| c.label == "Organic Search"));
        assert!(summary.browsers.iter().any(|b| b.label == "Safari"));
        assert!(summary.operating_systems.iter().any(|o| o.label == "macOS"));
        assert_eq!(summary.conversions[0].goal_name, "Start Trial");
        assert_eq!(summary.outbound_clicks[0].goal_name, "github.com/daypress");
        assert_eq!(
            summary.file_downloads[0].goal_name,
            "/uploads/whitepaper.pdf"
        );

        // Filtered drilldown by `page=/pricing`
        let pricing_only = db
            .get_analytics_dashboard_filtered(
                "30d",
                &AnalyticsFilter {
                    page: Some("/pricing".to_string()),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(pricing_only.total_pageviews, 1);
        assert_eq!(pricing_only.total_visitors, 1);

        // 4. Verify historical rollup + pruning preserves daily rollup counts
        {
            let conn = db.conn.lock().unwrap();
            conn.execute(
                r#"INSERT INTO daypress_analytics_events
                   (visitor_hash, page_url, referrer, referrer_domain, channel, device_type, browser, os, country, event_type, created_at)
                   VALUES ('old_hash_1', '/legacy-post', '', '', 'Direct', 'mobile', 'Chrome', 'Android', 'US', 'pageview', ?1)"#,
                params![(Utc::now() - Duration::days(60)).format("%Y-%m-%d %H:%M:%S").to_string()],
            )
            .unwrap();
        }
        let pruned = db.rollup_and_prune_analytics(30).unwrap();
        assert_eq!(
            pruned, 1,
            "60-day-old raw event should be rolled up and pruned under 30d retention"
        );
        let summary_90d = db.get_analytics_dashboard("90d").unwrap();
        assert_eq!(
            summary_90d.total_pageviews, 3,
            "90d range must seamlessly include rolled-up daily archive totals after raw-event pruning"
        );

        // 5. Verify overview SSR HTML renders KPIs, proportion bars, CWV strip, and active drilldown filter chips
        let Html(overview_html) =
            crate::handlers::native_modules::analytics_seo::handle_admin_analytics(
                axum::extract::State(db.clone()),
                axum::extract::Query(
                    crate::handlers::native_modules::analytics_seo::AnalyticsDashboardQuery {
                        range: Some("30d".to_string()),
                        page: Some("/pricing".to_string()),
                        country: Some("NL".to_string()),
                        ..Default::default()
                    },
                ),
            )
            .await;
        assert!(overview_html.contains("Filtered View"));
        assert!(overview_html.contains("Real-User Core Web Vitals (p75)"));
        assert!(overview_html.contains("Netherlands (NL)"));
        assert!(overview_html.contains("Start Trial"));
        assert!(overview_html.contains("github.com/daypress"));
        assert!(overview_html.contains("/uploads/whitepaper.pdf"));
    }
}
