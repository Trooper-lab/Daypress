//! v0.9 Pillar 1: Built-in Newsletters, Member-Gated Posts & Automated Digests (`adapters/database/newsletters.rs`).

use super::Database;
use crate::email::{dispatch_newsletter_email, EmailMessage};
use crate::errors::AppError;
use crate::models::{
    CreateNewsletterCampaignRequest, DaypressPostResponse, NewsletterCampaignRecord,
    NewsletterStatsSummary, NewsletterSubscribeRequest, SubscriberRecord,
};
use crate::templates::utils::escape_html;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use sha2::{Digest, Sha256};

fn generate_subscriber_token(seed: &str, purpose: &str) -> String {
    let random_part = crate::auth::generate_random_password();
    let now_ns = Utc::now().timestamp_nanos_opt().unwrap_or(0);
    let mut hasher = Sha256::new();
    hasher.update(format!("{seed}:{purpose}:{random_part}:{now_ns}").as_bytes());
    let hex = format!("{:x}", hasher.finalize());
    hex.chars().take(40).collect()
}

fn parse_lists_json(raw: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(raw)
        .ok()
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| vec!["general".to_string()])
}

fn sanitize_internal_return_url(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.starts_with('/') && !trimmed.starts_with("//") && !trimmed.starts_with("/\\") {
        trimmed.chars().take(500).collect()
    } else {
        String::new()
    }
}

impl Database {
    /// Checks whether Double Opt-In is enabled for public newsletter signups (default: `true`).
    pub fn is_newsletter_double_opt_in_enabled(&self) -> bool {
        if let Ok(Some(val)) = self.get_option("daypress_newsletter_double_opt_in") {
            let v = val.trim().to_ascii_lowercase();
            if v == "0" || v == "false" || v == "no" {
                return false;
            }
            if v == "1" || v == "true" || v == "yes" {
                return true;
            }
        }
        if let Ok(Some(nl)) = self.get_installed_plugin("daypress-newsletters") {
            if let Some(cfg_val) = nl.config.get("double_opt_in") {
                let v = cfg_val.trim().to_ascii_lowercase();
                if v == "0" || v == "false" || v == "no" {
                    return false;
                }
            }
        }
        true
    }

    /// Checks whether Automated New-Post Email Digests are enabled globally.
    pub fn is_newsletter_auto_digest_enabled(&self) -> bool {
        if let Ok(Some(val)) = self.get_option("daypress_newsletter_auto_digest") {
            let v = val.trim().to_ascii_lowercase();
            if v == "1" || v == "true" || v == "yes" {
                return true;
            }
            if v == "0" || v == "false" || v == "no" {
                return false;
            }
        }
        if let Ok(Some(nl)) = self.get_installed_plugin("daypress-newsletters") {
            if nl.active {
                if let Some(cfg_val) = nl.config.get("auto_digest_on_publish") {
                    let v = cfg_val.trim().to_ascii_lowercase();
                    if v == "1" || v == "true" || v == "yes" {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn map_subscriber_row(
        row: &rusqlite::Row,
        include_tokens: bool,
    ) -> rusqlite::Result<SubscriberRecord> {
        let lists_raw: String = row.get(4)?;
        let confirm_token: String = row.get(5)?;
        let unsubscribe_token: String = row.get(6)?;
        Ok(SubscriberRecord {
            id: row.get(0)?,
            email: row.get(1)?,
            name: row.get(2)?,
            status: row.get(3)?,
            lists: parse_lists_json(&lists_raw),
            confirm_token: if include_tokens {
                Some(confirm_token)
            } else {
                None
            },
            unsubscribe_token: if include_tokens {
                Some(unsubscribe_token)
            } else {
                None
            },
            source_url: row.get(7)?,
            confirmed_at: row.get(8)?,
            unsubscribed_at: row.get(9)?,
            created_at: row.get(10)?,
        })
    }

    /// Subscribes a visitor or admin-imported contact to one or more newsletter lists.
    /// Sends a Double Opt-In confirmation email when `status == "pending"`.
    pub fn subscribe_newsletter_contact(
        &self,
        req: &NewsletterSubscribeRequest,
        ip_hash: &str,
    ) -> std::result::Result<SubscriberRecord, AppError> {
        let (rec, _) = self.subscribe_newsletter_contact_checked(req, ip_hash)?;
        Ok(rec)
    }

    pub(crate) fn subscribe_newsletter_contact_checked(
        &self,
        req: &NewsletterSubscribeRequest,
        ip_hash: &str,
    ) -> std::result::Result<(SubscriberRecord, bool), AppError> {
        let email = req.email.trim().to_ascii_lowercase();
        if email.is_empty()
            || !email.contains('@')
            || !email.contains('.')
            || email.contains([' ', ',', ';', '<', '>'])
        {
            return Err(AppError::BadRequest(
                "Please enter a valid email address.".to_string(),
            ));
        }
        let name: String = req.name.trim().chars().take(120).collect();
        let source_url = sanitize_internal_return_url(&req.source_url);
        let mut lists: Vec<String> = req
            .lists
            .iter()
            .map(|s| s.trim().to_ascii_lowercase())
            .filter(|s| !s.is_empty())
            .collect();
        if lists.is_empty() {
            lists.push("general".to_string());
        }
        lists.sort();
        lists.dedup();

        let require_double_opt_in =
            !req.skip_double_opt_in && self.is_newsletter_double_opt_in_enabled();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let (record, already_active_before, should_send_confirm_email, base_url, site_title) = {
            let conn = self.conn.lock().unwrap();
            Self::activate_builtin_plugin_with_conn(&conn, "daypress-newsletters");
            let base_url = Self::get_base_url_with_conn(&conn);
            let site_title: String = conn
                .query_row(
                    "SELECT option_value FROM options WHERE option_name = 'blogname'",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or_else(|_| "Daypress".to_string());

            let existing: Option<(i64, String, String, String, String)> = conn
                .query_row(
                    "SELECT id, status, lists_json, confirm_token, unsubscribe_token FROM daypress_subscribers WHERE email = ?1 COLLATE NOCASE",
                    params![email],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
                )
                .ok();

            if let Some((sub_id, old_status, old_lists_raw, existing_confirm, existing_unsub)) =
                existing
            {
                let mut merged_lists = parse_lists_json(&old_lists_raw);
                for l in &lists {
                    if !merged_lists.contains(l) {
                        merged_lists.push(l.clone());
                    }
                }
                let lists_json = serde_json::to_string(&merged_lists)
                    .unwrap_or_else(|_| "[\"general\"]".to_string());

                let (new_status, confirmed_at_sql, send_confirm) =
                    if old_status == "subscribed" || !require_double_opt_in {
                        ("subscribed", Some(now.clone()), false)
                    } else {
                        ("pending", None, true)
                    };

                conn.execute(
                    r#"UPDATE daypress_subscribers
                       SET name = CASE WHEN ?1 != '' THEN ?1 ELSE name END,
                           status = ?2,
                           lists_json = ?3,
                           confirmed_at = COALESCE(confirmed_at, ?4),
                           unsubscribed_at = CASE WHEN ?2 = 'subscribed' THEN NULL ELSE unsubscribed_at END
                       WHERE id = ?5"#,
                    params![name, new_status, lists_json, confirmed_at_sql, sub_id],
                )?;

                let rec = conn.query_row(
                    r#"SELECT id, email, name, status, lists_json, confirm_token, unsubscribe_token, source_url, confirmed_at, unsubscribed_at, created_at
                       FROM daypress_subscribers WHERE id = ?1"#,
                    params![sub_id],
                    |r| Self::map_subscriber_row(r, true),
                )?;
                let _ = (existing_confirm, existing_unsub);
                (
                    rec,
                    old_status == "subscribed",
                    send_confirm,
                    base_url,
                    site_title,
                )
            } else {
                let confirm_token = generate_subscriber_token(&email, "confirm");
                let unsubscribe_token = generate_subscriber_token(&email, "unsubscribe");
                let lists_json =
                    serde_json::to_string(&lists).unwrap_or_else(|_| "[\"general\"]".to_string());
                let (init_status, confirmed_at) = if require_double_opt_in {
                    ("pending", None)
                } else {
                    ("subscribed", Some(now.clone()))
                };

                conn.execute(
                    r#"INSERT INTO daypress_subscribers
                       (email, name, status, lists_json, confirm_token, unsubscribe_token, source_url, ip_hash, confirmed_at, created_at)
                       VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)"#,
                    params![
                        email,
                        name,
                        init_status,
                        lists_json,
                        confirm_token,
                        unsubscribe_token,
                        source_url,
                        ip_hash,
                        confirmed_at,
                        now
                    ],
                )?;
                let sub_id = conn.last_insert_rowid();
                let rec = conn.query_row(
                    r#"SELECT id, email, name, status, lists_json, confirm_token, unsubscribe_token, source_url, confirmed_at, unsubscribed_at, created_at
                       FROM daypress_subscribers WHERE id = ?1"#,
                    params![sub_id],
                    |r| Self::map_subscriber_row(r, true),
                )?;
                (rec, false, require_double_opt_in, base_url, site_title)
            }
        };

        if should_send_confirm_email {
            if let Some(ref c_token) = record.confirm_token {
                let confirm_url = format!(
                    "{}/newsletter/confirm/{}",
                    base_url.trim_end_matches('/'),
                    c_token
                );
                let safe_site = escape_html(&site_title);
                let greeting = if record.name.trim().is_empty() {
                    "Hello".to_string()
                } else {
                    format!("Hi {}", escape_html(&record.name))
                };
                let body_html = format!(
                    r#"<div style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 560px; margin: 0 auto; padding: 28px; color: #18181b; border: 1px solid #e4e4e7; border-radius: 12px;">
                        <h2 style="margin: 0 0 12px; font-size: 20px;">Confirm your subscription to {safe_site}</h2>
                        <p style="font-size: 15px; line-height: 1.6; color: #3f3f46;">{greeting}, please confirm your email address to start receiving updates and unlock subscriber-only posts on <strong>{safe_site}</strong>.</p>
                        <p style="margin: 24px 0;">
                            <a href="{confirm_url}" style="display: inline-block; background: #a8401b; color: #ffffff; font-weight: 600; text-decoration: none; padding: 12px 22px; border-radius: 8px;">Confirm Subscription &rarr;</a>
                        </p>
                        <p style="font-size: 12px; color: #71717a;">If you did not request this subscription, you can safely ignore this email.</p>
                    </div>"#
                );
                let _ = dispatch_newsletter_email(
                    self,
                    EmailMessage {
                        to: record.email.clone(),
                        subject: format!("Confirm your subscription to {}", site_title),
                        body_html,
                        body_text: format!(
                            "Confirm your subscription to {}: {}",
                            site_title, confirm_url
                        ),
                    },
                );
            }
        }

        // Record conversion in cookieless analytics
        let _ = self.record_analytics_event_full(
            ip_hash,
            if source_url.is_empty() {
                "/"
            } else {
                &source_url
            },
            "",
            "desktop",
            "XX",
            None,
            None,
            None,
            "conversion",
            Some("newsletter_signup"),
        );

        Ok((record, already_active_before))
    }

    fn send_confirmation_or_magic_link_email(
        &self,
        base_url: &str,
        site_title: &str,
        recipient_email: &str,
        recipient_name: &str,
        token: &str,
        is_magic_unlock: bool,
    ) {
        let confirm_url = format!(
            "{}/newsletter/confirm/{}",
            base_url.trim_end_matches('/'),
            token
        );
        let safe_site = escape_html(site_title);
        let greeting = if recipient_name.trim().is_empty() {
            "Hello".to_string()
        } else {
            format!("Hi {}", escape_html(recipient_name))
        };
        let (subject, heading, cta, desc) = if is_magic_unlock {
            (
                format!("Your access link for {}", site_title),
                format!("Sign in to read subscriber posts on {}", safe_site),
                "Unlock Article &rarr;",
                format!("{greeting}, click the one-time link below to verify your email and unlock subscriber-only posts on <strong>{safe_site}</strong>."),
            )
        } else {
            (
                format!("Confirm your subscription to {}", site_title),
                format!("Confirm your subscription to {}", safe_site),
                "Confirm Subscription &rarr;",
                format!("{greeting}, please confirm your email address to start receiving updates and unlock subscriber-only posts on <strong>{safe_site}</strong>."),
            )
        };
        let body_html = format!(
            r#"<div style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 560px; margin: 0 auto; padding: 28px; color: #18181b; border: 1px solid #e4e4e7; border-radius: 12px;">
                <h2 style="margin: 0 0 12px; font-size: 20px;">{heading}</h2>
                <p style="font-size: 15px; line-height: 1.6; color: #3f3f46;">{desc}</p>
                <p style="margin: 24px 0;">
                    <a href="{confirm_url}" style="display: inline-block; background: #a8401b; color: #ffffff; font-weight: 600; text-decoration: none; padding: 12px 22px; border-radius: 8px;">{cta}</a>
                </p>
                <p style="font-size: 12px; color: #71717a;">If you did not request this link, you can safely ignore this email.</p>
            </div>"#
        );
        let _ = dispatch_newsletter_email(
            self,
            EmailMessage {
                to: recipient_email.to_string(),
                subject,
                body_html,
                body_text: format!("{}: {}", heading, confirm_url),
            },
        );
    }

    /// Confirms a pending subscriber or verifies a returning subscriber magic-link token
    /// via `/newsletter/confirm/{token}`, rotates the one-time `confirm_token`, and returns the record.
    pub fn confirm_subscriber_by_token(&self, token: &str) -> Result<Option<SubscriberRecord>> {
        let clean = token.trim();
        if clean.is_empty() {
            return Ok(None);
        }
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let sub_row: Option<(i64, String)> = conn
            .query_row(
                "SELECT id, email FROM daypress_subscribers WHERE confirm_token = ?1",
                params![clean],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        let Some((id, sub_email)) = sub_row else {
            return Ok(None);
        };
        let rotated_token = generate_subscriber_token(&sub_email, "used_confirm");
        conn.execute(
            r#"UPDATE daypress_subscribers
               SET status = 'subscribed',
                   confirm_token = ?1,
                   confirmed_at = COALESCE(confirmed_at, ?2),
                   unsubscribed_at = NULL
               WHERE id = ?3"#,
            params![rotated_token, now, id],
        )?;
        let rec = conn.query_row(
            r#"SELECT id, email, name, status, lists_json, confirm_token, unsubscribe_token, source_url, confirmed_at, unsubscribed_at, created_at
               FROM daypress_subscribers WHERE id = ?1"#,
            params![id],
            |r| Self::map_subscriber_row(r, true),
        )?;
        drop(conn);
        let _ = self.dispatch_outbound_webhook_event(
            "subscriber.confirmed",
            serde_json::json!({
                "subscriber_id": rec.id,
                "email": rec.email,
                "name": rec.name,
                "lists": rec.lists,
                "confirmed_at": rec.confirmed_at
            }),
        );
        Ok(Some(rec))
    }

    /// Unsubscribes a subscriber via `/newsletter/unsubscribe/{token}` (RFC-8058 1-click parity).
    pub fn unsubscribe_by_token(&self, token: &str) -> Result<Option<SubscriberRecord>> {
        let clean = token.trim();
        if clean.is_empty() {
            return Ok(None);
        }
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let sub_id: Option<i64> = conn
            .query_row(
                "SELECT id FROM daypress_subscribers WHERE unsubscribe_token = ?1",
                params![clean],
                |r| r.get(0),
            )
            .ok();
        let Some(id) = sub_id else {
            return Ok(None);
        };
        conn.execute(
            r#"UPDATE daypress_subscribers
               SET status = 'unsubscribed',
                   unsubscribed_at = ?1
               WHERE id = ?2"#,
            params![now, id],
        )?;
        let rec = conn.query_row(
            r#"SELECT id, email, name, status, lists_json, confirm_token, unsubscribe_token, source_url, confirmed_at, unsubscribed_at, created_at
               FROM daypress_subscribers WHERE id = ?1"#,
            params![id],
            |r| Self::map_subscriber_row(r, true),
        )?;
        drop(conn);
        let _ = self.dispatch_outbound_webhook_event(
            "subscriber.unsubscribed",
            serde_json::json!({
                "subscriber_id": rec.id,
                "email": rec.email,
                "name": rec.name,
                "lists": rec.lists,
                "unsubscribed_at": rec.unsubscribed_at
            }),
        );
        Ok(Some(rec))
    }

    /// Issues an HMAC-SHA256 signed `daypress_subscriber` session cookie token (`sub.<id>.<exp>.<sig>`),
    /// keeping `unsubscribe_token` and `confirm_token` strictly out of cookies.
    pub fn issue_subscriber_cookie_token(&self, sub_id: i64) -> String {
        let secret = self.get_auth_secret().unwrap_or_default();
        let exp = Utc::now().timestamp() + 31_536_000;
        let payload = format!("daypress-subscriber-v1|{}|{}", sub_id, exp);
        let sig = hex::encode(crate::auth::hmac_sha256(
            secret.as_bytes(),
            payload.as_bytes(),
        ));
        format!("sub.{}.{}.{}", sub_id, exp, sig)
    }

    /// Returns `true` when `token` is a valid HMAC-signed subscriber session token for an active
    /// (`status = 'subscribed'`) newsletter subscriber.
    pub fn verify_subscriber_token(&self, token: &str) -> bool {
        let clean = token.trim();
        let mut parts = clean.splitn(4, '.');
        let (Some("sub"), Some(id_raw), Some(exp_raw), Some(sig)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return false;
        };
        let Ok(sub_id) = id_raw.parse::<i64>() else {
            return false;
        };
        let Ok(exp) = exp_raw.parse::<i64>() else {
            return false;
        };
        if sub_id <= 0 || exp < Utc::now().timestamp() {
            return false;
        }
        let Ok(secret) = self.get_auth_secret() else {
            return false;
        };
        let expected = hex::encode(crate::auth::hmac_sha256(
            secret.as_bytes(),
            format!("daypress-subscriber-v1|{}|{}", sub_id, exp).as_bytes(),
        ));
        if !crate::auth::constant_time_eq(expected.as_bytes(), sig.as_bytes()) {
            return false;
        }
        let Ok(conn) = self.conn.lock() else {
            return false;
        };
        conn.query_row(
            r#"SELECT 1 FROM daypress_subscribers
               WHERE id = ?1 AND status = 'subscribed'
               LIMIT 1"#,
            params![sub_id],
            |_| Ok(true),
        )
        .unwrap_or(false)
    }

    /// Sends a one-time emailed confirmation / magic unlock link for a gated post instead of
    /// granting unlock access or revealing subscriber presence synchronously from an email string.
    pub fn request_subscriber_magic_unlock(
        &self,
        email_raw: &str,
        source_url_raw: &str,
        ip_hash: &str,
    ) -> std::result::Result<(), AppError> {
        let email = email_raw.trim().to_ascii_lowercase();
        if email.is_empty()
            || !email.contains('@')
            || !email.contains('.')
            || email.contains([' ', ',', ';', '<', '>'])
        {
            return Err(AppError::BadRequest(
                "Please enter a valid email address.".to_string(),
            ));
        }
        let safe_source = sanitize_internal_return_url(source_url_raw);
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let (base_url, site_title, name, magic_token, is_existing_subscribed) = {
            let conn = self.conn.lock().unwrap();
            Self::activate_builtin_plugin_with_conn(&conn, "daypress-newsletters");
            let base_url = Self::get_base_url_with_conn(&conn);
            let site_title: String = conn
                .query_row(
                    "SELECT option_value FROM options WHERE option_name = 'blogname'",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or_else(|_| "Daypress".to_string());

            let existing: Option<(i64, String, String)> = conn
                .query_row(
                    "SELECT id, status, name FROM daypress_subscribers WHERE email = ?1 COLLATE NOCASE",
                    params![email],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .ok();

            let magic_token = generate_subscriber_token(&email, "magic_unlock");
            if let Some((sub_id, old_status, sub_name)) = existing {
                let is_sub = old_status == "subscribed";
                let target_status = if is_sub { "subscribed" } else { "pending" };
                conn.execute(
                    r#"UPDATE daypress_subscribers
                       SET status = ?1,
                           confirm_token = ?2,
                           source_url = CASE WHEN ?3 != '' THEN ?3 ELSE source_url END
                       WHERE id = ?4"#,
                    params![target_status, magic_token, safe_source, sub_id],
                )?;
                (base_url, site_title, sub_name, magic_token, is_sub)
            } else {
                let unsub_token = generate_subscriber_token(&email, "unsubscribe");
                conn.execute(
                    r#"INSERT INTO daypress_subscribers
                       (email, name, status, lists_json, confirm_token, unsubscribe_token, source_url, ip_hash, confirmed_at, created_at)
                       VALUES (?1, '', 'pending', '["general"]', ?2, ?3, ?4, ?5, NULL, ?6)"#,
                    params![email, magic_token, unsub_token, safe_source, ip_hash, now],
                )?;
                (base_url, site_title, String::new(), magic_token, false)
            }
        };

        self.send_confirmation_or_magic_link_email(
            &base_url,
            &site_title,
            &email,
            &name,
            &magic_token,
            is_existing_subscribed,
        );
        Ok(())
    }

    /// Lists subscribers for the Admin Portal (`/d-admin/newsletters`), REST API, and MCP tools.
    pub fn list_subscribers(
        &self,
        status_filter: Option<&str>,
        list_filter: Option<&str>,
        search: Option<&str>,
        include_tokens: bool,
    ) -> Result<Vec<SubscriberRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, email, name, status, lists_json, confirm_token, unsubscribe_token, source_url, confirmed_at, unsubscribed_at, created_at
               FROM daypress_subscribers
               ORDER BY id DESC"#,
        )?;
        let rows = stmt.query_map([], |r| Self::map_subscriber_row(r, include_tokens))?;
        let status_norm = status_filter
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("all"))
            .map(|s| s.to_ascii_lowercase());
        let list_norm = list_filter
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("all"))
            .map(|s| s.to_ascii_lowercase());
        let search_norm = search
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(|s| s.to_ascii_lowercase());

        let mut out = Vec::new();
        for r in rows.flatten() {
            if let Some(ref st) = status_norm {
                if !r.status.eq_ignore_ascii_case(st) {
                    continue;
                }
            }
            if let Some(ref lst) = list_norm {
                if !r.lists.iter().any(|l| l.eq_ignore_ascii_case(lst)) {
                    continue;
                }
            }
            if let Some(ref q) = search_norm {
                if !r.email.to_ascii_lowercase().contains(q)
                    && !r.name.to_ascii_lowercase().contains(q)
                {
                    continue;
                }
            }
            out.push(r);
        }
        Ok(out)
    }

    pub fn update_subscriber_status(&self, id: i64, status: &str) -> Result<bool> {
        let norm = match status.trim().to_ascii_lowercase().as_str() {
            "subscribed" | "active" | "confirmed" => "subscribed",
            "unsubscribed" => "unsubscribed",
            _ => "pending",
        };
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.conn.lock().unwrap();
        let changed = conn.execute(
            r#"UPDATE daypress_subscribers
               SET status = ?1,
                   confirmed_at = CASE WHEN ?1 = 'subscribed' THEN COALESCE(confirmed_at, ?2) ELSE confirmed_at END,
                   unsubscribed_at = CASE WHEN ?1 = 'unsubscribed' THEN ?2 ELSE NULL END
               WHERE id = ?3"#,
            params![norm, now, id],
        )?;
        Ok(changed > 0)
    }

    pub fn delete_subscriber(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let deleted = conn.execute(
            "DELETE FROM daypress_subscribers WHERE id = ?1",
            params![id],
        )?;
        Ok(deleted > 0)
    }

    pub fn export_subscribers_csv(&self) -> Result<String> {
        let subs = self.list_subscribers(None, None, None, false)?;
        let mut csv =
            String::from("id,email,name,status,lists,source_url,confirmed_at,created_at\n");
        let esc = |s: &str| crate::templates::utils::csv_safe_cell(s);
        for s in subs {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{}\n",
                s.id,
                esc(&s.email),
                esc(&s.name),
                esc(&s.status),
                esc(&s.lists.join(";")),
                esc(&s.source_url),
                esc(s.confirmed_at.as_deref().unwrap_or("")),
                esc(&s.created_at),
            ));
        }
        Ok(csv)
    }

    pub fn get_newsletter_stats(&self) -> Result<NewsletterStatsSummary> {
        let conn = self.conn.lock().unwrap();
        Self::get_newsletter_stats_with_conn(&conn)
    }

    pub(crate) fn get_newsletter_stats_with_conn(
        conn: &Connection,
    ) -> Result<NewsletterStatsSummary> {
        let total_subscribers: i64 = conn
            .query_row("SELECT COUNT(*) FROM daypress_subscribers", [], |r| {
                r.get(0)
            })
            .unwrap_or(0);
        let confirmed_subscribers: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM daypress_subscribers WHERE status = 'subscribed'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let pending_subscribers: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM daypress_subscribers WHERE status = 'pending'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let unsubscribed_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM daypress_subscribers WHERE status = 'unsubscribed'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let (total_campaigns_sent, total_emails_delivered): (i64, i64) = conn
            .query_row(
                "SELECT COUNT(*), COALESCE(SUM(sent_count), 0) FROM daypress_newsletter_campaigns WHERE status = 'sent'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap_or((0, 0));

        Ok(NewsletterStatsSummary {
            total_subscribers,
            confirmed_subscribers,
            pending_subscribers,
            unsubscribed_count,
            total_campaigns_sent,
            total_emails_delivered,
        })
    }

    pub fn list_newsletter_campaigns(&self) -> Result<Vec<NewsletterCampaignRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, subject, body_html, target_list, campaign_type, post_id, status, recipient_count, sent_count, failed_count, sent_at, created_at
               FROM daypress_newsletter_campaigns
               ORDER BY id DESC"#,
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(NewsletterCampaignRecord {
                id: r.get(0)?,
                subject: r.get(1)?,
                body_html: r.get(2)?,
                target_list: r.get(3)?,
                campaign_type: r.get(4)?,
                post_id: r.get(5)?,
                status: r.get(6)?,
                recipient_count: r.get(7)?,
                sent_count: r.get(8)?,
                failed_count: r.get(9)?,
                sent_at: r.get(10)?,
                created_at: r.get(11)?,
            })
        })?;
        Ok(rows.flatten().collect())
    }

    /// Creates a newsletter campaign (`broadcast` or `digest`) and immediately dispatches it to
    /// all confirmed subscribers in `target_list` when `req.send_now` is `true`.
    pub fn create_and_send_campaign(
        &self,
        req: &CreateNewsletterCampaignRequest,
        campaign_type: &str,
        post_id: Option<i64>,
    ) -> std::result::Result<NewsletterCampaignRecord, AppError> {
        let subject = req.subject.trim();
        if subject.is_empty() {
            return Err(AppError::BadRequest(
                "Campaign subject is required.".to_string(),
            ));
        }
        let target_list = if req.target_list.trim().is_empty() {
            "general".to_string()
        } else {
            req.target_list.trim().to_ascii_lowercase()
        };
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let subscribers = self.list_subscribers(
            Some("subscribed"),
            if target_list == "all" || target_list == "general" {
                None
            } else {
                Some(&target_list)
            },
            None,
            true,
        )?;
        let recipient_count = subscribers.len() as i64;
        let base_url = self.get_base_url();

        let campaign_id = {
            let conn = self.conn.lock().unwrap();
            Self::activate_builtin_plugin_with_conn(&conn, "daypress-newsletters");
            conn.execute(
                r#"INSERT INTO daypress_newsletter_campaigns
                   (subject, body_html, target_list, campaign_type, post_id, status, recipient_count, sent_count, failed_count, created_at)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 0, 0, ?8)"#,
                params![
                    subject,
                    req.body_html,
                    target_list,
                    campaign_type,
                    post_id,
                    if req.send_now { "sending" } else { "draft" },
                    recipient_count,
                    now
                ],
            )?;
            conn.last_insert_rowid()
        };

        let mut sent_count = 0_i64;
        let mut failed_count = 0_i64;

        if req.send_now {
            for sub in &subscribers {
                let unsub_token = sub.unsubscribe_token.as_deref().unwrap_or("");
                let unsub_url = format!(
                    "{}/newsletter/unsubscribe/{}",
                    base_url.trim_end_matches('/'),
                    unsub_token
                );
                let full_html = format!(
                    r#"{body}
                    <hr style="border: none; border-top: 1px solid #e4e4e7; margin: 32px 0 16px;" />
                    <p style="font-size: 12px; color: #71717a; font-family: sans-serif;">
                        You are receiving this email because you subscribed at <a href="{base}" style="color: #71717a;">{base}</a>.
                        <a href="{unsub_url}" style="color: #a8401b; margin-left: 8px;">Unsubscribe in 1 click</a>
                    </p>"#,
                    body = req.body_html,
                    base = escape_html(&base_url),
                    unsub_url = unsub_url
                );
                match dispatch_newsletter_email(
                    self,
                    EmailMessage {
                        to: sub.email.clone(),
                        subject: subject.to_string(),
                        body_html: full_html,
                        body_text: String::new(),
                    },
                ) {
                    Ok(()) => sent_count += 1,
                    Err(_) => failed_count += 1,
                }
            }

            let conn = self.conn.lock().unwrap();
            conn.execute(
                r#"UPDATE daypress_newsletter_campaigns
                   SET status = 'sent',
                       sent_count = ?1,
                       failed_count = ?2,
                       sent_at = ?3
                   WHERE id = ?4"#,
                params![sent_count, failed_count, now, campaign_id],
            )?;
        }

        let conn = self.conn.lock().unwrap();
        let rec = conn.query_row(
            r#"SELECT id, subject, body_html, target_list, campaign_type, post_id, status, recipient_count, sent_count, failed_count, sent_at, created_at
               FROM daypress_newsletter_campaigns WHERE id = ?1"#,
            params![campaign_id],
            |r| {
                Ok(NewsletterCampaignRecord {
                    id: r.get(0)?,
                    subject: r.get(1)?,
                    body_html: r.get(2)?,
                    target_list: r.get(3)?,
                    campaign_type: r.get(4)?,
                    post_id: r.get(5)?,
                    status: r.get(6)?,
                    recipient_count: r.get(7)?,
                    sent_count: r.get(8)?,
                    failed_count: r.get(9)?,
                    sent_at: r.get(10)?,
                    created_at: r.get(11)?,
                })
            },
        )?;
        Ok(rec)
    }

    /// Automatically sends a New-Post Email Digest to confirmed subscribers when a post is published
    /// and either `explicit_flag == Some(true)` or global auto-digest is enabled.
    pub fn maybe_send_new_post_digest(
        &self,
        post: &DaypressPostResponse,
        explicit_flag: Option<bool>,
    ) -> Result<Option<NewsletterCampaignRecord>> {
        if post.status != "publish"
            || post.post_type == "revision"
            || post.post_type == "nav_menu_item"
        {
            return Ok(None);
        }
        let should_send = match explicit_flag {
            Some(true) => true,
            Some(false) => false,
            None => post.post_type == "post" && self.is_newsletter_auto_digest_enabled(),
        };
        if !should_send {
            return Ok(None);
        }

        // Prevent duplicate automated digests for the same post unless explicitly forced
        if explicit_flag != Some(true) {
            let conn = self.conn.lock().unwrap();
            let already_sent: bool = conn
                .query_row(
                    "SELECT 1 FROM daypress_newsletter_campaigns WHERE post_id = ?1 AND campaign_type = 'digest' LIMIT 1",
                    params![post.id],
                    |_| Ok(true),
                )
                .unwrap_or(false);
            if already_sent {
                return Ok(None);
            }
        }

        let site_title = self
            .get_option("blogname")
            .ok()
            .flatten()
            .unwrap_or_else(|| "Daypress".to_string());
        let post_title = if post.title.rendered.trim().is_empty() {
            "New Post".to_string()
        } else {
            post.title.rendered.clone()
        };
        let excerpt_preview: String = if !post.excerpt.rendered.trim().is_empty() {
            post.excerpt.rendered.chars().take(320).collect()
        } else {
            post.content.rendered.chars().take(360).collect()
        };
        let post_url = if post.link.trim().is_empty() {
            format!(
                "{}/{}/",
                self.get_base_url().trim_end_matches('/'),
                post.slug
            )
        } else {
            post.link.clone()
        };
        let gate_badge = if post.member_gate == "subscribers_only" {
            r#"<span style="display:inline-block;background:#fef3c7;color:#92400e;font-size:11px;font-weight:700;padding:3px 8px;border-radius:999px;margin-bottom:10px;">Subscriber-Exclusive Post</span>"#
        } else {
            ""
        };
        let body_html = format!(
            r#"<div style="font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; max-width: 600px; margin: 0 auto; padding: 24px; color: #18181b;">
                <div style="font-size: 12px; text-transform: uppercase; letter-spacing: 0.08em; color: #71717a; margin-bottom: 8px;">New on {site_title}</div>
                {gate_badge}
                <h1 style="margin: 0 0 14px; font-size: 24px; line-height: 1.3;">{title}</h1>
                <div style="font-size: 15px; line-height: 1.65; color: #3f3f46; margin-bottom: 24px;">{excerpt}</div>
                <a href="{post_url}" style="display: inline-block; background: #a8401b; color: #ffffff; font-weight: 600; text-decoration: none; padding: 12px 22px; border-radius: 8px;">Read Full Post &rarr;</a>
            </div>"#,
            site_title = escape_html(&site_title),
            gate_badge = gate_badge,
            title = escape_html(&post_title),
            excerpt = excerpt_preview,
            post_url = post_url,
        );

        let req = CreateNewsletterCampaignRequest {
            subject: format!("[{}] {}", site_title, post_title),
            body_html,
            target_list: "general".to_string(),
            send_now: true,
        };
        match self.create_and_send_campaign(&req, "digest", Some(post.id)) {
            Ok(campaign) => Ok(Some(campaign)),
            Err(_) => Ok(None),
        }
    }
}
