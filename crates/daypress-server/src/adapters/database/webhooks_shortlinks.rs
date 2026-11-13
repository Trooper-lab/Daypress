//! v0.9 Pillar 2: Branded Shortlinks (`/go/*`) & HMAC-Signed Outbound Webhooks (`adapters/database/webhooks_shortlinks.rs`).

use super::Database;
use crate::errors::AppError;
use crate::models::{
    CreateShortlinkRequest, CreateWebhookEndpointRequest, ShortlinkRecord, WebhookDeliveryRecord,
    WebhookEndpointRecord,
};
use chrono::Utc;
use hmac::{Hmac, Mac};
use rusqlite::{params, Result};
use sha2::Sha256;
use std::time::{Duration, Instant};

type HmacSha256 = Hmac<Sha256>;

/// Computes the canonical `X-Daypress-Signature-256` header (`sha256=<hex>`) over `"{timestamp}.{payload_json}"`.
pub fn compute_webhook_hmac_sha256(secret: &str, timestamp: i64, payload_json: &str) -> String {
    let signing_input = format!("{}.{}", timestamp, payload_json);
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC-SHA256 accepts any key length");
    mac.update(signing_input.as_bytes());
    let result = mac.finalize().into_bytes();
    format!("sha256={}", hex::encode(result))
}

struct SsrfSafeDnsResolver;

impl reqwest::dns::Resolve for SsrfSafeDnsResolver {
    fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        Box::pin(async move {
            let allow_private =
                std::env::var("DAYPRESS_ALLOW_PRIVATE_WEBHOOKS").as_deref() == Ok("1");
            let addrs: Vec<std::net::SocketAddr> = tokio::net::lookup_host((name.as_str(), 0))
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?
                .collect();
            if !allow_private {
                for addr in &addrs {
                    if is_disallowed_webhook_ip(&addr.ip()) {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::PermissionDenied,
                            "Webhook DNS resolved to a private/loopback/metadata IP (SSRF blocked)",
                        ))
                            as Box<dyn std::error::Error + Send + Sync>);
                    }
                }
            }
            if addrs.is_empty() {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::AddrNotAvailable,
                    "Webhook hostname resolved to zero IP addresses",
                ))
                    as Box<dyn std::error::Error + Send + Sync>);
            }
            Ok(Box::new(addrs.into_iter()) as reqwest::dns::Addrs)
        })
    }
}

/// Builds a `reqwest::Client` that pins DNS resolution through [`SsrfSafeDnsResolver`] (preventing
/// DNS rebinding TOCTOU) and disables HTTP redirects (preventing 301/302 SSRF pivots).
pub fn build_ssrf_safe_http_client(
    timeout: Duration,
) -> std::result::Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::none())
        .dns_resolver(std::sync::Arc::new(SsrfSafeDnsResolver))
        .build()
}

fn append_utm_params(
    base_target: &str,
    utm_source: &str,
    utm_medium: &str,
    utm_campaign: &str,
) -> String {
    let mut pairs = Vec::new();
    if !utm_source.trim().is_empty() && !base_target.contains("utm_source=") {
        pairs.push(format!(
            "utm_source={}",
            urlencoding_simple(utm_source.trim())
        ));
    }
    if !utm_medium.trim().is_empty() && !base_target.contains("utm_medium=") {
        pairs.push(format!(
            "utm_medium={}",
            urlencoding_simple(utm_medium.trim())
        ));
    }
    if !utm_campaign.trim().is_empty() && !base_target.contains("utm_campaign=") {
        pairs.push(format!(
            "utm_campaign={}",
            urlencoding_simple(utm_campaign.trim())
        ));
    }
    if pairs.is_empty() {
        return base_target.to_string();
    }
    let sep = if base_target.contains('?') { '&' } else { '?' };
    format!("{}{}{}", base_target, sep, pairs.join("&"))
}

fn urlencoding_simple(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for b in input.bytes() {
        if b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b'~') {
            out.push(b as char);
        } else {
            out.push_str(&format!("%{:02X}", b));
        }
    }
    out
}

impl Database {
    // =========================================================================
    // 1. Branded Shortlinks (`/go/{slug}`)
    // =========================================================================

    pub fn save_shortlink(
        &self,
        req: &CreateShortlinkRequest,
    ) -> std::result::Result<ShortlinkRecord, AppError> {
        self.create_or_update_shortlink(req)
    }

    pub fn create_or_update_shortlink(
        &self,
        req: &CreateShortlinkRequest,
    ) -> std::result::Result<ShortlinkRecord, AppError> {
        let raw_slug = req
            .slug
            .trim()
            .trim_matches('/')
            .trim_start_matches("go/")
            .trim();
        let slug = generated_slugify::slugify(raw_slug);
        if slug.is_empty() {
            return Err(AppError::BadRequest(
                "Shortlink slug cannot be empty.".to_string(),
            ));
        }
        let target_url = req.target_url.trim();
        if target_url.is_empty()
            || target_url.contains(['\r', '\n'])
            || target_url.starts_with("//")
            || target_url.starts_with("/\\")
            || (!target_url.starts_with("http://")
                && !target_url.starts_with("https://")
                && !target_url.starts_with('/'))
        {
            return Err(AppError::BadRequest(
                "Target URL must start with https://, http://, or a single relative / (protocol-relative // and CRLF characters are rejected)".to_string(),
            ));
        }
        let status_code = match req.status_code {
            301 | 302 | 307 | 308 => req.status_code,
            _ => 302,
        };
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.conn.lock().unwrap();
        let base_url = Self::get_base_url_with_conn(&conn);

        conn.execute(
            r#"INSERT INTO daypress_shortlinks
               (slug, target_url, title, status_code, utm_source, utm_medium, utm_campaign, is_active, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
               ON CONFLICT(slug) DO UPDATE SET
                   target_url = excluded.target_url,
                   title = excluded.title,
                   status_code = excluded.status_code,
                   utm_source = excluded.utm_source,
                   utm_medium = excluded.utm_medium,
                   utm_campaign = excluded.utm_campaign,
                   is_active = excluded.is_active"#,
            params![
                slug,
                target_url,
                req.title.trim(),
                status_code,
                req.utm_source.trim(),
                req.utm_medium.trim(),
                req.utm_campaign.trim(),
                if req.is_active { 1_i64 } else { 0_i64 },
                now,
            ],
        )?;
        Self::activate_builtin_plugin_with_conn(&conn, "daypress-webhooks");

        let rec = conn.query_row(
            r#"SELECT id, slug, target_url, title, status_code, utm_source, utm_medium, utm_campaign, clicks, last_clicked_at, is_active, created_at
               FROM daypress_shortlinks WHERE slug = ?1 COLLATE NOCASE"#,
            params![slug],
            |r| {
                let s: String = r.get(1)?;
                Ok(ShortlinkRecord {
                    id: r.get(0)?,
                    short_url: format!("{}/go/{}", base_url.trim_end_matches('/'), s),
                    slug: s,
                    target_url: r.get(2)?,
                    title: r.get(3)?,
                    status_code: r.get(4)?,
                    utm_source: r.get(5)?,
                    utm_medium: r.get(6)?,
                    utm_campaign: r.get(7)?,
                    clicks: r.get(8)?,
                    last_clicked_at: r.get(9)?,
                    is_active: r.get::<_, i64>(10)? != 0,
                    created_at: r.get(11)?,
                })
            },
        )?;
        Ok(rec)
    }

    pub fn list_shortlinks(&self) -> Result<Vec<ShortlinkRecord>> {
        let conn = self.conn.lock().unwrap();
        let base_url = Self::get_base_url_with_conn(&conn);
        let mut stmt = conn.prepare(
            r#"SELECT id, slug, target_url, title, status_code, utm_source, utm_medium, utm_campaign, clicks, last_clicked_at, is_active, created_at
               FROM daypress_shortlinks
               ORDER BY clicks DESC, id DESC"#,
        )?;
        let rows = stmt.query_map([], |r| {
            let s: String = r.get(1)?;
            Ok(ShortlinkRecord {
                id: r.get(0)?,
                short_url: format!("{}/go/{}", base_url.trim_end_matches('/'), s),
                slug: s,
                target_url: r.get(2)?,
                title: r.get(3)?,
                status_code: r.get(4)?,
                utm_source: r.get(5)?,
                utm_medium: r.get(6)?,
                utm_campaign: r.get(7)?,
                clicks: r.get(8)?,
                last_clicked_at: r.get(9)?,
                is_active: r.get::<_, i64>(10)? != 0,
                created_at: r.get(11)?,
            })
        })?;
        Ok(rows.flatten().collect())
    }

    pub fn delete_shortlink(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute("DELETE FROM daypress_shortlinks WHERE id = ?1", params![id])?;
        Ok(deleted > 0)
    }

    /// Resolves `/go/{slug}`, increments `clicks`, appends UTM attribution tags, logs a cookieless
    /// analytics conversion (`shortlink:{slug}`), and triggers `shortlink.clicked` webhooks.
    pub fn resolve_shortlink_and_record_click(
        &self,
        slug: &str,
        ip_hash: &str,
        referrer: &str,
    ) -> Option<(String, u16)> {
        let clean = slug.trim().trim_matches('/');
        if clean.is_empty() {
            return None;
        }
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let row: Option<(i64, String, String, i64, String, String, String)> = {
            let conn = self.conn.lock().ok()?;
            let found = conn
                .query_row(
                    r#"SELECT id, slug, target_url, status_code, utm_source, utm_medium, utm_campaign
                       FROM daypress_shortlinks
                       WHERE slug = ?1 COLLATE NOCASE AND is_active = 1
                       LIMIT 1"#,
                    params![clean],
                    |r| {
                        Ok((
                            r.get(0)?,
                            r.get(1)?,
                            r.get(2)?,
                            r.get(3)?,
                            r.get(4)?,
                            r.get(5)?,
                            r.get(6)?,
                        ))
                    },
                )
                .ok()?;
            let _ = conn.execute(
                "UPDATE daypress_shortlinks SET clicks = clicks + 1, last_clicked_at = ?1 WHERE id = ?2",
                params![now, found.0],
            );
            Some(found)
        };

        let (id, matched_slug, target_url, status_code, utm_src, utm_med, utm_camp) = row?;
        let final_target = append_utm_params(&target_url, &utm_src, &utm_med, &utm_camp);

        let _ = self.record_analytics_event_full(
            ip_hash,
            &format!("/go/{}", matched_slug),
            referrer,
            "desktop",
            "XX",
            if utm_src.is_empty() {
                None
            } else {
                Some(&utm_src)
            },
            if utm_med.is_empty() {
                None
            } else {
                Some(&utm_med)
            },
            if utm_camp.is_empty() {
                None
            } else {
                Some(&utm_camp)
            },
            "conversion",
            Some(&format!("shortlink:{}", matched_slug)),
        );

        let _ = self.dispatch_outbound_webhook_event(
            "shortlink.clicked",
            serde_json::json!({
                "shortlink_id": id,
                "slug": matched_slug,
                "target_url": final_target,
                "referrer": referrer,
            }),
        );

        Some((final_target, status_code as u16))
    }

    // =========================================================================
    // 2. Outbound Webhooks (`Slack`, `Discord`, `HubSpot`, `Zapier`, Generic)
    // =========================================================================

    /// Validates that a webhook `target_url` does not target loopback, link-local/cloud-metadata,
    /// or RFC1918 private network addresses (SSRF protection).
    pub fn validate_webhook_target_url_ssrf(raw_url: &str) -> std::result::Result<(), AppError> {
        let trimmed = raw_url.trim();
        if trimmed.starts_with("local://") || trimmed.starts_with("mock://") {
            return Ok(());
        }
        if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
            return Err(AppError::BadRequest(
                "Webhook target_url must start with https:// or http://".to_string(),
            ));
        }
        if std::env::var("DAYPRESS_ALLOW_PRIVATE_WEBHOOKS").as_deref() == Ok("1") {
            return Ok(());
        }
        let parsed = reqwest::Url::parse(trimmed).map_err(|_| {
            AppError::BadRequest("Webhook target_url is not a valid URL.".to_string())
        })?;
        let host_str = parsed
            .host_str()
            .ok_or_else(|| {
                AppError::BadRequest("Webhook target_url is missing a host.".to_string())
            })?
            .trim_matches(['[', ']'])
            .to_ascii_lowercase();

        if host_str.is_empty()
            || host_str == "localhost"
            || host_str.ends_with(".localhost")
            || host_str.ends_with(".local")
            || host_str.ends_with(".internal")
            || host_str == "metadata.google.internal"
            || host_str == "metadata"
            || host_str == "169.254.169.254"
            || host_str == "100.100.100.200"
        {
            return Err(AppError::BadRequest(
                "Webhook target_url cannot target localhost, internal, or cloud metadata hosts (SSRF protection)."
                    .to_string(),
            ));
        }

        if let Ok(ip) = host_str.parse::<std::net::IpAddr>() {
            if is_disallowed_webhook_ip(&ip) {
                return Err(AppError::BadRequest(
                    "Webhook target_url cannot target loopback, link-local, or private IP addresses (SSRF protection)."
                        .to_string(),
                ));
            }
        }
        Ok(())
    }

    pub fn save_webhook_endpoint(
        &self,
        req: &CreateWebhookEndpointRequest,
    ) -> std::result::Result<WebhookEndpointRecord, AppError> {
        let name = req.name.trim();
        if name.is_empty() {
            return Err(AppError::BadRequest(
                "Webhook name is required.".to_string(),
            ));
        }
        let target_url = req.target_url.trim();
        Self::validate_webhook_target_url_ssrf(target_url)?;
        let preset = match req.preset.trim().to_ascii_lowercase().as_str() {
            "slack" => "slack",
            "discord" => "discord",
            "hubspot" => "hubspot",
            "zapier" => "zapier",
            _ => "generic",
        };
        let events: Vec<String> = if req.events.is_empty() {
            vec![
                "post.published".to_string(),
                "form.submitted".to_string(),
                "subscriber.confirmed".to_string(),
            ]
        } else {
            req.events
                .iter()
                .map(|e| e.trim().to_ascii_lowercase())
                .filter(|e| !e.is_empty())
                .collect()
        };
        let events_json = serde_json::to_string(&events)
            .unwrap_or_else(|_| "[\"post.published\",\"form.submitted\"]".to_string());
        let secret = if req.secret.trim().is_empty() {
            format!("whsec_{}", crate::auth::generate_random_password())
        } else {
            req.secret.trim().to_string()
        };
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let conn = self.conn.lock().unwrap();
        let hook_id = if let Some(id) = req.id.filter(|&i| i > 0) {
            conn.execute(
                r#"UPDATE daypress_webhooks
                   SET name = ?1, target_url = ?2, preset = ?3, events_json = ?4, secret = ?5, is_active = ?6
                   WHERE id = ?7"#,
                params![
                    name,
                    target_url,
                    preset,
                    events_json,
                    secret,
                    if req.is_active { 1_i64 } else { 0_i64 },
                    id
                ],
            )?;
            id
        } else {
            conn.execute(
                r#"INSERT INTO daypress_webhooks
                   (name, target_url, preset, events_json, secret, is_active, created_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
                params![
                    name,
                    target_url,
                    preset,
                    events_json,
                    secret,
                    if req.is_active { 1_i64 } else { 0_i64 },
                    now
                ],
            )?;
            conn.last_insert_rowid()
        };
        Self::activate_builtin_plugin_with_conn(&conn, "daypress-webhooks");

        let rec = conn.query_row(
            r#"SELECT id, name, target_url, preset, events_json, secret, is_active, created_at
               FROM daypress_webhooks WHERE id = ?1"#,
            params![hook_id],
            |r| {
                let ev_raw: String = r.get(4)?;
                let ev_list: Vec<String> = serde_json::from_str(&ev_raw).unwrap_or_default();
                Ok(WebhookEndpointRecord {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    target_url: r.get(2)?,
                    preset: r.get(3)?,
                    events: ev_list,
                    secret: r.get(5)?,
                    is_active: r.get::<_, i64>(6)? != 0,
                    created_at: r.get(7)?,
                })
            },
        )?;
        Ok(rec)
    }

    pub(crate) fn list_webhook_endpoints_internal(&self) -> Result<Vec<WebhookEndpointRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, name, target_url, preset, events_json, secret, is_active, created_at
               FROM daypress_webhooks ORDER BY id DESC"#,
        )?;
        let rows = stmt.query_map([], |r| {
            let ev_raw: String = r.get(4)?;
            let ev_list: Vec<String> = serde_json::from_str(&ev_raw).unwrap_or_default();
            Ok(WebhookEndpointRecord {
                id: r.get(0)?,
                name: r.get(1)?,
                target_url: r.get(2)?,
                preset: r.get(3)?,
                events: ev_list,
                secret: r.get(5)?,
                is_active: r.get::<_, i64>(6)? != 0,
                created_at: r.get(7)?,
            })
        })?;
        Ok(rows.flatten().collect())
    }

    pub fn list_webhook_endpoints(&self) -> Result<Vec<WebhookEndpointRecord>> {
        let mut endpoints = self.list_webhook_endpoints_internal()?;
        for ep in &mut endpoints {
            ep.secret = if ep.secret.trim().is_empty() {
                String::new()
            } else {
                "••••••••".to_string()
            };
        }
        Ok(endpoints)
    }

    pub fn delete_webhook_endpoint(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute("DELETE FROM daypress_webhooks WHERE id = ?1", params![id])?;
        Ok(deleted > 0)
    }

    pub fn list_webhook_deliveries(
        &self,
        webhook_id: Option<i64>,
        limit: usize,
    ) -> Result<Vec<WebhookDeliveryRecord>> {
        let conn = self.conn.lock().unwrap();
        let lim = limit.clamp(1, 500) as i64;
        let sql = if webhook_id.is_some() {
            r#"SELECT id, webhook_id, webhook_name, event_name, target_url, request_headers_json, payload_json, signature_header, response_status, response_body, latency_ms, attempt_count, status, created_at
               FROM daypress_webhook_deliveries WHERE webhook_id = ?1 ORDER BY id DESC LIMIT ?2"#
        } else {
            r#"SELECT id, webhook_id, webhook_name, event_name, target_url, request_headers_json, payload_json, signature_header, response_status, response_body, latency_ms, attempt_count, status, created_at
               FROM daypress_webhook_deliveries ORDER BY id DESC LIMIT ?1"#
        };
        let mut stmt = conn.prepare(sql)?;
        let map_row = |r: &rusqlite::Row| -> rusqlite::Result<WebhookDeliveryRecord> {
            Ok(WebhookDeliveryRecord {
                id: r.get(0)?,
                webhook_id: r.get(1)?,
                webhook_name: r.get(2)?,
                event_name: r.get(3)?,
                target_url: r.get(4)?,
                request_headers_json: r.get(5)?,
                payload_json: r.get(6)?,
                signature_header: r.get(7)?,
                response_status: r.get(8)?,
                response_body: r.get(9)?,
                latency_ms: r.get(10)?,
                attempt_count: r.get(11)?,
                status: r.get(12)?,
                created_at: r.get(13)?,
            })
        };
        let out: Vec<WebhookDeliveryRecord> = if let Some(wid) = webhook_id {
            stmt.query_map(params![wid, lim], map_row)?
                .flatten()
                .collect()
        } else {
            stmt.query_map(params![lim], map_row)?.flatten().collect()
        };
        Ok(out)
    }

    /// Dispatches an outbound webhook event (`post.published`, `form.submitted`, `subscriber.confirmed`, `shortlink.clicked`)
    /// to all active webhook endpoints subscribed to `event_name`, signing each payload with `X-Daypress-Signature-256`.
    pub fn dispatch_outbound_webhook_event(
        &self,
        event_name: &str,
        data: serde_json::Value,
    ) -> Result<Vec<WebhookDeliveryRecord>> {
        let endpoints = self.list_webhook_endpoints_internal()?;
        let matching: Vec<WebhookEndpointRecord> = endpoints
            .into_iter()
            .filter(|ep| {
                ep.is_active
                    && ep
                        .events
                        .iter()
                        .any(|ev| ev == "*" || ev.eq_ignore_ascii_case(event_name))
            })
            .collect();
        if matching.is_empty() {
            return Ok(Vec::new());
        }

        let base_url = self.get_base_url();
        let site_title = self
            .get_option("blogname")
            .ok()
            .flatten()
            .unwrap_or_else(|| "Daypress".to_string());
        let ts = Utc::now().timestamp();
        let now_str = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut deliveries = Vec::new();

        for ep in matching {
            let summary_text = format!(
                "⚡ [{}] Event `{}` triggered on {}",
                site_title, event_name, base_url
            );
            let body_val = match ep.preset.as_str() {
                "slack" => serde_json::json!({
                    "text": summary_text,
                    "event": event_name,
                    "timestamp": ts,
                    "site_url": base_url,
                    "data": data,
                }),
                "discord" => serde_json::json!({
                    "content": summary_text,
                    "event": event_name,
                    "timestamp": ts,
                    "site_url": base_url,
                    "data": data,
                }),
                _ => serde_json::json!({
                    "event": event_name,
                    "preset": ep.preset,
                    "timestamp": ts,
                    "site_url": base_url,
                    "data": data,
                }),
            };
            let payload_json =
                serde_json::to_string(&body_val).unwrap_or_else(|_| "{}".to_string());
            let sig = compute_webhook_hmac_sha256(&ep.secret, ts, &payload_json);
            let req_headers_json = serde_json::json!({
                "Content-Type": "application/json",
                "User-Agent": "Daypress-Webhooks/0.9",
                "X-Daypress-Event": event_name,
                "X-Daypress-Timestamp": ts.to_string(),
                "X-Daypress-Signature-256": sig,
            })
            .to_string();

            let is_mock_or_local =
                ep.target_url.starts_with("local://") || ep.target_url.starts_with("mock://");
            let initial_status = "delivered";
            let initial_code = 200_i64;

            let delivery_id = {
                let conn = self.conn.lock().unwrap();
                conn.execute(
                    r#"INSERT INTO daypress_webhook_deliveries
                       (webhook_id, webhook_name, event_name, target_url, request_headers_json, payload_json, signature_header, response_status, response_body, latency_ms, attempt_count, status, created_at)
                       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 1, 1, ?10, ?11)"#,
                    params![
                        ep.id,
                        ep.name,
                        event_name,
                        ep.target_url,
                        req_headers_json,
                        payload_json,
                        sig,
                        initial_code,
                        if is_mock_or_local { "OK (local/mock)" } else { "Dispatched" },
                        initial_status,
                        now_str,
                    ],
                )?;
                conn.last_insert_rowid()
            };

            if !is_mock_or_local
                && (ep.target_url.starts_with("http://") || ep.target_url.starts_with("https://"))
            {
                if let Ok(handle) = tokio::runtime::Handle::try_current() {
                    let db_clone = self.clone();
                    handle.spawn(async move {
                        let _ = db_clone.execute_http_webhook_delivery(delivery_id).await;
                    });
                }
            }

            let conn = self.conn.lock().unwrap();
            if let Ok(rec) = conn.query_row(
                r#"SELECT id, webhook_id, webhook_name, event_name, target_url, request_headers_json, payload_json, signature_header, response_status, response_body, latency_ms, attempt_count, status, created_at
                   FROM daypress_webhook_deliveries WHERE id = ?1"#,
                params![delivery_id],
                |r| {
                    Ok(WebhookDeliveryRecord {
                        id: r.get(0)?,
                        webhook_id: r.get(1)?,
                        webhook_name: r.get(2)?,
                        event_name: r.get(3)?,
                        target_url: r.get(4)?,
                        request_headers_json: r.get(5)?,
                        payload_json: r.get(6)?,
                        signature_header: r.get(7)?,
                        response_status: r.get(8)?,
                        response_body: r.get(9)?,
                        latency_ms: r.get(10)?,
                        attempt_count: r.get(11)?,
                        status: r.get(12)?,
                        created_at: r.get(13)?,
                    })
                },
            ) {
                deliveries.push(rec);
            }
        }

        Ok(deliveries)
    }

    /// Executes the actual HTTP POST for a `daypress_webhook_deliveries` row and records status/latency.
    pub async fn execute_http_webhook_delivery(
        &self,
        delivery_id: i64,
    ) -> std::result::Result<WebhookDeliveryRecord, AppError> {
        let (webhook_id, event_name, target_url, payload_json, attempt_count) = {
            let conn = self.conn.lock().unwrap();
            conn.query_row(
                r#"SELECT webhook_id, event_name, target_url, payload_json, attempt_count
                   FROM daypress_webhook_deliveries WHERE id = ?1"#,
                params![delivery_id],
                |r| {
                    Ok((
                        r.get::<_, i64>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, String>(2)?,
                        r.get::<_, String>(3)?,
                        r.get::<_, i64>(4)?,
                    ))
                },
            )
            .map_err(|_| {
                AppError::NotFound(format!("Webhook delivery {} not found", delivery_id))
            })?
        };

        let secret: String = {
            let conn = self.conn.lock().unwrap();
            conn.query_row(
                "SELECT secret FROM daypress_webhooks WHERE id = ?1",
                params![webhook_id],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| {
                AppError::Internal(format!(
                    "Webhook {} signing secret is missing or unconfigured",
                    webhook_id
                ))
            })?
        };

        let ts = Utc::now().timestamp();
        let sig = compute_webhook_hmac_sha256(&secret, ts, &payload_json);
        let req_headers_json = serde_json::json!({
            "Content-Type": "application/json",
            "User-Agent": "Daypress-Webhooks/0.9",
            "X-Daypress-Event": event_name,
            "X-Daypress-Timestamp": ts.to_string(),
            "X-Daypress-Signature-256": sig,
        })
        .to_string();

        if target_url.starts_with("local://") || target_url.starts_with("mock://") {
            let new_status = if attempt_count > 1 {
                "replayed"
            } else {
                "delivered"
            };
            let conn = self.conn.lock().unwrap();
            conn.execute(
                r#"UPDATE daypress_webhook_deliveries
                   SET request_headers_json = ?1,
                       signature_header = ?2,
                       response_status = 200,
                       response_body = 'OK (Replayed local/mock)',
                       latency_ms = 1,
                       status = ?3
                   WHERE id = ?4"#,
                params![req_headers_json, sig, new_status, delivery_id],
            )?;
        } else {
            if let Err(ssrf_err) = Self::validate_webhook_target_url_ssrf(&target_url) {
                let conn = self.conn.lock().unwrap();
                let _ = conn.execute(
                    r#"UPDATE daypress_webhook_deliveries
                       SET request_headers_json = ?1,
                           signature_header = ?2,
                           response_status = 0,
                           response_body = ?3,
                           latency_ms = 0,
                           status = 'failed'
                       WHERE id = ?4"#,
                    params![req_headers_json, sig, ssrf_err.to_string(), delivery_id],
                );
                return Err(ssrf_err);
            }
            if std::env::var("DAYPRESS_ALLOW_PRIVATE_WEBHOOKS").as_deref() != Ok("1") {
                if let Ok(parsed) = reqwest::Url::parse(&target_url) {
                    if let Some(host) = parsed.host_str() {
                        let port = parsed.port_or_known_default().unwrap_or(443);
                        if let Ok(addrs) = tokio::net::lookup_host((host, port)).await {
                            for addr in addrs {
                                if is_disallowed_webhook_ip(&addr.ip()) {
                                    let msg = "Webhook DNS resolved to a private/loopback IP (SSRF blocked)";
                                    let conn = self.conn.lock().unwrap();
                                    let _ = conn.execute(
                                        r#"UPDATE daypress_webhook_deliveries
                                           SET request_headers_json = ?1,
                                               signature_header = ?2,
                                               response_status = 0,
                                               response_body = ?3,
                                               latency_ms = 0,
                                               status = 'failed'
                                           WHERE id = ?4"#,
                                        params![req_headers_json, sig, msg, delivery_id],
                                    );
                                    return Err(AppError::BadRequest(msg.to_string()));
                                }
                            }
                        }
                    }
                }
            }
            let start = Instant::now();
            let client = build_ssrf_safe_http_client(Duration::from_secs(10)).map_err(|e| {
                AppError::Internal(format!("HTTP client initialization failed: {e}"))
            })?;
            let resp = client
                .post(&target_url)
                .header("Content-Type", "application/json")
                .header("User-Agent", "Daypress-Webhooks/0.9")
                .header("X-Daypress-Event", &event_name)
                .header("X-Daypress-Timestamp", ts.to_string())
                .header("X-Daypress-Signature-256", &sig)
                .body(payload_json.clone())
                .send()
                .await;
            let latency_ms = start.elapsed().as_millis() as i64;
            match resp {
                Ok(r) => {
                    let code = r.status().as_u16() as i64;
                    let ok = r.status().is_success();
                    let body_txt: String = r
                        .text()
                        .await
                        .unwrap_or_default()
                        .chars()
                        .take(500)
                        .collect();
                    let conn = self.conn.lock().unwrap();
                    let _ = conn.execute(
                        r#"UPDATE daypress_webhook_deliveries
                           SET request_headers_json = ?1,
                               signature_header = ?2,
                               response_status = ?3,
                               response_body = ?4,
                               latency_ms = ?5,
                               status = ?6
                           WHERE id = ?7"#,
                        params![
                            req_headers_json,
                            sig,
                            code,
                            body_txt,
                            latency_ms,
                            if ok { "delivered" } else { "failed" },
                            delivery_id
                        ],
                    );
                }
                Err(e) => {
                    let err_txt: String = e.to_string().chars().take(500).collect();
                    let conn = self.conn.lock().unwrap();
                    let _ = conn.execute(
                        r#"UPDATE daypress_webhook_deliveries
                           SET request_headers_json = ?1,
                               signature_header = ?2,
                               response_status = 0,
                               response_body = ?3,
                               latency_ms = ?4,
                               status = 'failed'
                           WHERE id = ?5"#,
                        params![req_headers_json, sig, err_txt, latency_ms, delivery_id],
                    );
                }
            }
        }

        let _ = attempt_count;
        let conn = self.conn.lock().unwrap();
        let rec = conn.query_row(
            r#"SELECT id, webhook_id, webhook_name, event_name, target_url, request_headers_json, payload_json, signature_header, response_status, response_body, latency_ms, attempt_count, status, created_at
               FROM daypress_webhook_deliveries WHERE id = ?1"#,
            params![delivery_id],
            |r| {
                Ok(WebhookDeliveryRecord {
                    id: r.get(0)?,
                    webhook_id: r.get(1)?,
                    webhook_name: r.get(2)?,
                    event_name: r.get(3)?,
                    target_url: r.get(4)?,
                    request_headers_json: r.get(5)?,
                    payload_json: r.get(6)?,
                    signature_header: r.get(7)?,
                    response_status: r.get(8)?,
                    response_body: r.get(9)?,
                    latency_ms: r.get(10)?,
                    attempt_count: r.get(11)?,
                    status: r.get(12)?,
                    created_at: r.get(13)?,
                })
            },
        )?;
        Ok(rec)
    }

    /// 1-Click Replay of any previous webhook delivery (`POST /wp-json/daypress/v1/webhooks/deliveries/{id}/replay`).
    pub async fn replay_webhook_delivery(
        &self,
        delivery_id: i64,
    ) -> std::result::Result<WebhookDeliveryRecord, AppError> {
        {
            let conn = self.conn.lock().unwrap();
            let updated = conn.execute(
                "UPDATE daypress_webhook_deliveries SET attempt_count = attempt_count + 1 WHERE id = ?1",
                params![delivery_id],
            )?;
            if updated == 0 {
                return Err(AppError::NotFound(format!(
                    "Webhook delivery {} not found",
                    delivery_id
                )));
            }
        }
        self.execute_http_webhook_delivery(delivery_id).await
    }

    /// Synchronous 1-click replay helper for MCP tools (`daypress_manage_webhooks`).
    pub fn replay_webhook_delivery_sync(
        &self,
        delivery_id: i64,
    ) -> std::result::Result<WebhookDeliveryRecord, AppError> {
        let (webhook_id, event_name, target_url, payload_json) = {
            let conn = self.conn.lock().unwrap();
            let updated = conn.execute(
                "UPDATE daypress_webhook_deliveries SET attempt_count = attempt_count + 1, status = 'replayed' WHERE id = ?1",
                params![delivery_id],
            )?;
            if updated == 0 {
                return Err(AppError::NotFound(format!(
                    "Webhook delivery {} not found",
                    delivery_id
                )));
            }
            conn.query_row(
                "SELECT webhook_id, event_name, target_url, payload_json FROM daypress_webhook_deliveries WHERE id = ?1",
                params![delivery_id],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?, r.get::<_, String>(3)?)),
            )
            .map_err(|_| AppError::NotFound(format!("Webhook delivery {} not found", delivery_id)))?
        };
        let secret: String = {
            let conn = self.conn.lock().unwrap();
            conn.query_row(
                "SELECT secret FROM daypress_webhooks WHERE id = ?1",
                params![webhook_id],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| {
                AppError::Internal(format!(
                    "Webhook {} signing secret is missing or unconfigured",
                    webhook_id
                ))
            })?
        };
        let ts = Utc::now().timestamp();
        let sig = compute_webhook_hmac_sha256(&secret, ts, &payload_json);
        let req_headers_json = serde_json::json!({
            "Content-Type": "application/json",
            "User-Agent": "Daypress-Webhooks/0.9",
            "X-Daypress-Event": event_name,
            "X-Daypress-Timestamp": ts.to_string(),
            "X-Daypress-Signature-256": sig,
        })
        .to_string();
        {
            let conn = self.conn.lock().unwrap();
            let _ = conn.execute(
                r#"UPDATE daypress_webhook_deliveries
                   SET request_headers_json = ?1,
                       signature_header = ?2,
                       response_status = 200,
                       response_body = 'OK (Replayed)',
                       status = 'replayed'
                   WHERE id = ?3"#,
                params![req_headers_json, sig, delivery_id],
            );
        }
        if !target_url.starts_with("local://") && !target_url.starts_with("mock://") {
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                let db_clone = self.clone();
                handle.spawn(async move {
                    let _ = db_clone.execute_http_webhook_delivery(delivery_id).await;
                });
            }
        }
        let conn = self.conn.lock().unwrap();
        let rec = conn.query_row(
            r#"SELECT id, webhook_id, webhook_name, event_name, target_url, request_headers_json, payload_json, signature_header, response_status, response_body, latency_ms, attempt_count, status, created_at
               FROM daypress_webhook_deliveries WHERE id = ?1"#,
            params![delivery_id],
            |r| {
                Ok(WebhookDeliveryRecord {
                    id: r.get(0)?,
                    webhook_id: r.get(1)?,
                    webhook_name: r.get(2)?,
                    event_name: r.get(3)?,
                    target_url: r.get(4)?,
                    request_headers_json: r.get(5)?,
                    payload_json: r.get(6)?,
                    signature_header: r.get(7)?,
                    response_status: r.get(8)?,
                    response_body: r.get(9)?,
                    latency_ms: r.get(10)?,
                    attempt_count: r.get(11)?,
                    status: r.get(12)?,
                    created_at: r.get(13)?,
                })
            },
        )?;
        Ok(rec)
    }
}

fn is_disallowed_webhook_ipv4(v4: &std::net::Ipv4Addr) -> bool {
    let oct = v4.octets();
    v4.is_loopback()
        || v4.is_private()
        || v4.is_link_local()
        || v4.is_unspecified()
        || v4.is_broadcast()
        || v4.is_multicast()
        || v4.is_documentation()
        // Shared address space / CGNAT & cloud metadata (100.64.0.0/10)
        || (oct[0] == 100 && (oct[1] & 0b1100_0000) == 64)
}

fn is_disallowed_webhook_ip(ip: &std::net::IpAddr) -> bool {
    match ip {
        std::net::IpAddr::V4(v4) => is_disallowed_webhook_ipv4(v4),
        std::net::IpAddr::V6(v6) => {
            if let Some(mapped) = v6.to_ipv4_mapped() {
                return is_disallowed_webhook_ipv4(&mapped);
            }
            let seg0 = v6.segments()[0];
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                // Unique local (fc00::/7) or Link-local (fe80::/10)
                || (seg0 & 0xfe00) == 0xfc00
                || (seg0 & 0xffc0) == 0xfe80
        }
    }
}
