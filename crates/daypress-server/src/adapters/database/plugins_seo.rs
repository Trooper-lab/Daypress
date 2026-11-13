#![allow(unused_imports)]
use super::Database;
use crate::auth::AuthUser;
use crate::domain::errors::{DomainError, DomainResult};
use crate::errors::AppError;
use crate::models::*;
use base64::Engine;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

impl Database {
    // =========================================================================
    // NATIVE SUBSYSTEMS: DECLARATIVE FORMS & WEBHOOKS
    // =========================================================================

    pub fn create_form(&self, title: &str, schema_json: &str, settings_json: &str) -> Result<i64> {
        let id = {
            let conn = self.conn.lock().unwrap();
            let now = Utc::now().to_rfc3339();
            conn.execute(
                "INSERT INTO daypress_forms (title, schema_json, settings_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
                params![title, schema_json, settings_json, now],
            )?;
            Self::activate_builtin_plugin_with_conn(&conn, "daypress-forms");
            conn.last_insert_rowid()
        };
        self.invalidate_hot_cache("form_created");
        Ok(id)
    }

    pub fn get_form(&self, id: i64) -> Result<Option<DaypressForm>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, schema_json, settings_json, created_at, updated_at FROM daypress_forms WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |r| {
            Ok(DaypressForm {
                id: r.get(0)?,
                title: r.get(1)?,
                schema_json: r.get(2)?,
                settings_json: r.get(3)?,
                created_at: r.get(4)?,
                updated_at: r.get(5)?,
            })
        })?;
        match rows.next() {
            Some(res) => Ok(Some(res?)),
            None => Ok(None),
        }
    }

    pub fn submit_form_entry(
        &self,
        form_id: i64,
        data_json: &str,
        ip_hash: &str,
        user_agent: &str,
    ) -> Result<i64> {
        self.submit_form_entry_full(
            form_id, data_json, "[]", "unread", "queued", "none", "", ip_hash, user_agent,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn submit_form_entry_full(
        &self,
        form_id: i64,
        data_json: &str,
        attachments_json: &str,
        status: &str,
        email_status: &str,
        webhook_status: &str,
        source_url: &str,
        ip_hash: &str,
        user_agent: &str,
    ) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            r#"INSERT INTO daypress_form_entries
               (form_id, data_json, attachments_json, status, is_starred, email_status, email_error, webhook_status, source_url, ip_hash, user_agent, created_at)
               VALUES (?1, ?2, ?3, ?4, 0, ?5, '', ?6, ?7, ?8, ?9, ?10)"#,
            params![
                form_id,
                data_json,
                attachments_json,
                status,
                email_status,
                webhook_status,
                source_url,
                ip_hash,
                user_agent,
                now
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update_form_entry_delivery_status(
        &self,
        entry_id: i64,
        email_status: &str,
        email_error: &str,
        webhook_status: &str,
    ) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "UPDATE daypress_form_entries SET email_status = ?1, email_error = ?2, webhook_status = ?3 WHERE id = ?4",
            params![email_status, email_error, webhook_status, entry_id],
        )?;
        Ok(rows > 0)
    }

    pub fn update_form_entry_workflow(
        &self,
        entry_id: i64,
        status: Option<&str>,
        is_starred: Option<bool>,
    ) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let mut changed = 0usize;
        if let Some(st) = status {
            let clean = match st {
                "unread" | "read" | "spam" | "trash" => st,
                _ => "read",
            };
            changed += conn.execute(
                "UPDATE daypress_form_entries SET status = ?1 WHERE id = ?2",
                params![clean, entry_id],
            )?;
        }
        if let Some(star) = is_starred {
            changed += conn.execute(
                "UPDATE daypress_form_entries SET is_starred = ?1 WHERE id = ?2",
                params![if star { 1 } else { 0 }, entry_id],
            )?;
        }
        Ok(changed > 0)
    }

    pub fn list_forms(&self) -> Result<Vec<(DaypressForm, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT f.id, f.title, f.schema_json, f.settings_json, f.created_at, f.updated_at,
                      (SELECT COUNT(*) FROM daypress_form_entries e WHERE e.form_id = f.id AND COALESCE(e.status, 'unread') NOT IN ('spam', 'trash'))
               FROM daypress_forms f
               ORDER BY f.id ASC"#,
        )?;
        let rows = stmt.query_map([], |r| {
            Ok((
                DaypressForm {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    schema_json: r.get(2)?,
                    settings_json: r.get(3)?,
                    created_at: r.get(4)?,
                    updated_at: r.get(5)?,
                },
                r.get::<_, i64>(6)?,
            ))
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn save_form_record(&self, req: &CreateFormRequest) -> Result<i64> {
        let wh_url = req.settings.webhook_url.trim();
        if !wh_url.is_empty() {
            Self::validate_webhook_target_url_ssrf(wh_url).map_err(|e| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    e.to_string(),
                )))
            })?;
        }
        let saved_id = {
            let conn = self.conn.lock().unwrap();
            let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let schema_json =
                serde_json::to_string(&req.fields).unwrap_or_else(|_| "[]".to_string());
            let settings_json =
                serde_json::to_string(&req.settings).unwrap_or_else(|_| "{}".to_string());
            let title = if req.title.trim().is_empty() {
                "Untitled Form"
            } else {
                req.title.trim()
            };

            let id = if let Some(id) = req.id.filter(|&id| id > 0) {
                let updated = conn.execute(
                    "UPDATE daypress_forms SET title = ?1, schema_json = ?2, settings_json = ?3, updated_at = ?4 WHERE id = ?5",
                    params![title, schema_json, settings_json, now, id],
                )?;
                if updated > 0 {
                    id
                } else {
                    conn.execute(
                        "INSERT INTO daypress_forms (id, title, schema_json, settings_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?5)",
                        params![id, title, schema_json, settings_json, now],
                    )?;
                    id
                }
            } else {
                conn.execute(
                    "INSERT INTO daypress_forms (title, schema_json, settings_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
                    params![title, schema_json, settings_json, now],
                )?;
                conn.last_insert_rowid()
            };
            Self::activate_builtin_plugin_with_conn(&conn, "daypress-forms");
            id
        };
        self.invalidate_hot_cache("form_saved");
        Ok(saved_id)
    }

    pub fn delete_form(&self, id: i64) -> Result<bool> {
        let rows = {
            let conn = self.conn.lock().unwrap();
            let _ = conn.execute(
                "DELETE FROM daypress_form_entries WHERE form_id = ?1",
                params![id],
            );
            conn.execute("DELETE FROM daypress_forms WHERE id = ?1", params![id])?
        };
        for root_dir in ["data", "uploads"] {
            let upload_dir = Path::new(root_dir)
                .join("form-submissions")
                .join(id.to_string());
            if upload_dir.exists() {
                let _ = fs::remove_dir_all(upload_dir);
            }
        }
        self.invalidate_hot_cache("form_deleted");
        Ok(rows > 0)
    }

    pub fn get_form_entry_by_id(&self, entry_id: i64) -> Result<Option<FormEntryItem>> {
        let conn = self.conn.lock().unwrap();
        let sql = r#"SELECT e.id, e.form_id, COALESCE(f.title, 'Form'), e.data_json,
                            COALESCE(e.attachments_json, '[]'), COALESCE(e.status, 'unread'),
                            COALESCE(e.is_starred, 0), COALESCE(e.email_status, 'disabled'),
                            COALESCE(e.email_error, ''), COALESCE(e.webhook_status, 'none'),
                            COALESCE(e.source_url, ''), e.ip_hash, e.user_agent, e.created_at
                     FROM daypress_form_entries e
                     LEFT JOIN daypress_forms f ON e.form_id = f.id
                     WHERE e.id = ?1"#;
        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query_map(params![entry_id], Self::map_form_entry_row)?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    fn map_form_entry_row(r: &rusqlite::Row) -> rusqlite::Result<FormEntryItem> {
        let attachments_raw: String = r.get(4)?;
        let attachments: Vec<FormAttachmentMeta> =
            serde_json::from_str(&attachments_raw).unwrap_or_default();
        Ok(FormEntryItem {
            id: r.get(0)?,
            form_id: r.get(1)?,
            form_title: r.get(2)?,
            data_json: r.get(3)?,
            attachments,
            status: r.get(5)?,
            is_starred: r.get::<_, i64>(6).unwrap_or(0) != 0,
            email_status: r.get(7)?,
            email_error: r.get(8)?,
            webhook_status: r.get(9)?,
            source_url: r.get(10)?,
            ip_hash: r.get(11)?,
            user_agent: r.get(12)?,
            created_at: r.get(13)?,
        })
    }

    pub fn list_form_entries(
        &self,
        form_id: Option<i64>,
        limit: i64,
    ) -> Result<Vec<FormEntryItem>> {
        self.list_form_entries_filtered(form_id, None, None, limit)
    }

    pub fn list_form_entries_filtered(
        &self,
        form_id: Option<i64>,
        status_filter: Option<&str>,
        search: Option<&str>,
        limit: i64,
    ) -> Result<Vec<FormEntryItem>> {
        let conn = self.conn.lock().unwrap();
        let sql = if form_id.is_some() {
            r#"SELECT e.id, e.form_id, COALESCE(f.title, 'Form'), e.data_json,
                      COALESCE(e.attachments_json, '[]'), COALESCE(e.status, 'unread'),
                      COALESCE(e.is_starred, 0), COALESCE(e.email_status, 'disabled'),
                      COALESCE(e.email_error, ''), COALESCE(e.webhook_status, 'none'),
                      COALESCE(e.source_url, ''), e.ip_hash, e.user_agent, e.created_at
               FROM daypress_form_entries e
               LEFT JOIN daypress_forms f ON e.form_id = f.id
               WHERE e.form_id = ?1
               ORDER BY e.id DESC LIMIT ?2"#
        } else {
            r#"SELECT e.id, e.form_id, COALESCE(f.title, 'Form'), e.data_json,
                      COALESCE(e.attachments_json, '[]'), COALESCE(e.status, 'unread'),
                      COALESCE(e.is_starred, 0), COALESCE(e.email_status, 'disabled'),
                      COALESCE(e.email_error, ''), COALESCE(e.webhook_status, 'none'),
                      COALESCE(e.source_url, ''), e.ip_hash, e.user_agent, e.created_at
               FROM daypress_form_entries e
               LEFT JOIN daypress_forms f ON e.form_id = f.id
               ORDER BY e.id DESC LIMIT ?1"#
        };

        let mut stmt = conn.prepare(sql)?;
        let fetch_limit = limit.max(200);
        let raw_entries: Vec<FormEntryItem> = if let Some(fid) = form_id {
            stmt.query_map(params![fid, fetch_limit], Self::map_form_entry_row)?
                .filter_map(|r| r.ok())
                .collect()
        } else {
            stmt.query_map(params![fetch_limit], Self::map_form_entry_row)?
                .filter_map(|r| r.ok())
                .collect()
        };

        let status_clean = status_filter.map(str::trim).unwrap_or("");
        let search_lower = search.map(|s| s.trim().to_lowercase()).unwrap_or_default();

        let filtered = raw_entries
            .into_iter()
            .filter(|item| match status_clean {
                "" | "all" => item.status != "spam" && item.status != "trash",
                "starred" => item.is_starred && item.status != "trash",
                "unread" | "read" | "spam" | "trash" => item.status == status_clean,
                "failed_email" => item.email_status == "failed",
                _ => true,
            })
            .filter(|item| {
                if search_lower.is_empty() {
                    true
                } else {
                    item.data_json.to_lowercase().contains(&search_lower)
                        || item.form_title.to_lowercase().contains(&search_lower)
                        || item.source_url.to_lowercase().contains(&search_lower)
                }
            })
            .take(limit.max(1) as usize)
            .collect();

        Ok(filtered)
    }

    pub fn delete_form_entry(&self, entry_id: i64) -> Result<bool> {
        if let Ok(Some(entry)) = self.get_form_entry_by_id(entry_id) {
            for att in &entry.attachments {
                let p = Path::new(&att.stored_path);
                let has_traversal = p.components().any(|c| {
                    matches!(
                        c,
                        std::path::Component::ParentDir
                            | std::path::Component::RootDir
                            | std::path::Component::Prefix(_)
                    )
                });
                if !has_traversal
                    && (p.starts_with("data/form-submissions")
                        || p.starts_with("uploads/form-submissions"))
                    && p.exists()
                {
                    let _ = fs::remove_file(p);
                }
            }
        }
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "DELETE FROM daypress_form_entries WHERE id = ?1",
            params![entry_id],
        )?;
        Ok(rows > 0)
    }

    // =========================================================================
    // NATIVE SUBSYSTEMS: SEO REDIRECTS, 404 MONITOR & GLOBAL SEO SETTINGS
    // =========================================================================

    pub fn get_global_seo_settings(&self) -> Result<GlobalSeoSettings> {
        Ok(GlobalSeoSettings {
            title_separator: self
                .get_option("daypress_seo_separator")?
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "–".to_string()),
            default_og_image: self
                .get_option("daypress_seo_default_og_image")?
                .unwrap_or_default(),
            org_name: self
                .get_option("daypress_seo_org_name")?
                .unwrap_or_default(),
            org_logo_url: self
                .get_option("daypress_seo_org_logo")?
                .unwrap_or_default(),
            twitter_site: self
                .get_option("daypress_seo_twitter_handle")?
                .unwrap_or_default(),
            enable_breadcrumbs: self
                .get_option("daypress_seo_enable_json_ld")?
                .map(|v| v != "0" && v != "false")
                .unwrap_or(true),
            auto_redirect_slug_changes: self
                .get_option("daypress_seo_auto_redirect")?
                .map(|v| v != "0" && v != "false")
                .unwrap_or(true),
            custom_robots_txt: self
                .get_option("daypress_seo_custom_robots")?
                .unwrap_or_default(),
        })
    }

    pub fn save_global_seo_settings(&self, s: &GlobalSeoSettings) -> Result<()> {
        self.set_option("daypress_seo_separator", s.title_separator.trim())?;
        self.set_option("daypress_seo_default_og_image", s.default_og_image.trim())?;
        self.set_option("daypress_seo_org_name", s.org_name.trim())?;
        self.set_option("daypress_seo_org_logo", s.org_logo_url.trim())?;
        self.set_option("daypress_seo_twitter_handle", s.twitter_site.trim())?;
        self.set_option(
            "daypress_seo_enable_json_ld",
            if s.enable_breadcrumbs { "1" } else { "0" },
        )?;
        self.set_option(
            "daypress_seo_auto_redirect",
            if s.auto_redirect_slug_changes {
                "1"
            } else {
                "0"
            },
        )?;
        self.set_option("daypress_seo_custom_robots", s.custom_robots_txt.trim())?;
        Ok(())
    }

    pub fn list_redirects(&self) -> Result<Vec<RedirectRule>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source_path, target_url, status_code, hits, is_active, created_at FROM daypress_redirects ORDER BY id DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(RedirectRule {
                id: r.get(0)?,
                source_path: r.get(1)?,
                target_url: r.get(2)?,
                status_code: r.get::<_, i64>(3).unwrap_or(301),
                hits: r.get(4)?,
                active: r.get::<_, i64>(5)? != 0,
                created_at: r.get(6)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn create_redirect(&self, req: &CreateRedirectRequest) -> Result<RedirectRule> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut src = req.source_path.trim().to_string();
        if !src.starts_with('/') {
            src = format!("/{src}");
        }
        let dst = req.target_url.trim().to_string();
        let code = if req.status_code == 302 || req.status_code == 307 || req.status_code == 308 {
            req.status_code
        } else {
            301
        };

        conn.execute(
            r#"INSERT INTO daypress_redirects (source_path, target_url, status_code, hits, is_active, created_at)
               VALUES (?1, ?2, ?3, 0, 1, ?4)
               ON CONFLICT(source_path) DO UPDATE SET
                   target_url = excluded.target_url,
                   status_code = excluded.status_code,
                   is_active = 1"#,
            params![src, dst, code, now],
        )?;

        let rule = conn.query_row(
            "SELECT id, source_path, target_url, status_code, hits, is_active, created_at FROM daypress_redirects WHERE source_path = ?1",
            params![src],
            |r| {
                Ok(RedirectRule {
                    id: r.get(0)?,
                    source_path: r.get(1)?,
                    target_url: r.get(2)?,
                    status_code: r.get::<_, i64>(3).unwrap_or(301),
                    hits: r.get(4)?,
                    active: r.get::<_, i64>(5)? != 0,
                    created_at: r.get(6)?,
                })
            },
        )?;
        Ok(rule)
    }

    pub fn delete_redirect(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM daypress_redirects WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    /// Checks if an incoming request path matches an active redirect rule (with or without trailing slash).
    /// Increments the hit counter when matched.
    pub fn match_redirect_and_increment(&self, request_path: &str) -> Option<(String, u16)> {
        let conn = self.conn.lock().ok()?;
        let normalized = if request_path.starts_with('/') {
            request_path.to_string()
        } else {
            format!("/{request_path}")
        };
        let with_slash = if normalized.ends_with('/') {
            normalized.clone()
        } else {
            format!("{normalized}/")
        };
        let without_slash = normalized.trim_end_matches('/');
        let without_slash = if without_slash.is_empty() {
            "/"
        } else {
            without_slash
        };

        let found: Option<(i64, String, u16)> = conn
            .query_row(
                "SELECT id, target_url, status_code FROM daypress_redirects WHERE is_active = 1 AND (source_path = ?1 OR source_path = ?2 OR source_path = ?3) LIMIT 1",
                params![normalized, with_slash, without_slash],
                |r| Ok((r.get(0)?, r.get(1)?, r.get::<_, u16>(2).unwrap_or(301))),
            )
            .ok();

        if let Some((id, target, code)) = found {
            let _ = conn.execute(
                "UPDATE daypress_redirects SET hits = hits + 1 WHERE id = ?1",
                params![id],
            );
            Some((target, code))
        } else {
            None
        }
    }

    pub fn record_404_hit(&self, request_path: &str, referrer: &str, user_agent: &str) {
        if let Ok(conn) = self.conn.lock() {
            let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let _ = conn.execute(
                r#"INSERT INTO daypress_404_log (request_path, referrer, user_agent, hits, last_seen_at)
                   VALUES (?1, ?2, ?3, 1, ?4)
                   ON CONFLICT(request_path) DO UPDATE SET
                       hits = daypress_404_log.hits + 1,
                       referrer = CASE WHEN excluded.referrer != '' THEN excluded.referrer ELSE daypress_404_log.referrer END,
                       user_agent = CASE WHEN excluded.user_agent != '' THEN excluded.user_agent ELSE daypress_404_log.user_agent END,
                       last_seen_at = excluded.last_seen_at"#,
                params![request_path, referrer, user_agent, now],
            );
        }
    }

    pub fn list_404_logs(&self, limit: i64) -> Result<Vec<NotFoundLogEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, request_path, referrer, user_agent, hits, last_seen_at FROM daypress_404_log ORDER BY hits DESC, last_seen_at DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |r| {
            Ok(NotFoundLogEntry {
                id: r.get(0)?,
                path: r.get(1)?,
                referrer: r.get(2)?,
                user_agent: r.get(3)?,
                hits: r.get(4)?,
                last_seen: r.get(5)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn clear_404_logs(&self) -> Result<usize> {
        let conn = self.conn.lock().unwrap();
        let count = conn.execute("DELETE FROM daypress_404_log", [])?;
        Ok(count)
    }

    // =========================================================================
    // NATIVE SUBSYSTEMS: PLUGIN ECOSYSTEM REGISTRY & COMMUNITY EXTENSIONS
    // =========================================================================

    pub(crate) fn activate_builtin_plugin_with_conn(conn: &Connection, slug: &str) {
        if crate::plugins::is_core_feature_slug(slug) {
            return;
        }
        let _ = Self::ensure_builtin_plugins_seeded_with_conn(conn);
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let _ = conn.execute(
            "UPDATE daypress_plugins SET is_active = 1, updated_at = ?2 WHERE slug = ?1",
            params![slug, now],
        );
        if let Ok(mut stmt) = conn.prepare(
            "SELECT slug FROM daypress_plugins WHERE is_active = 1 ORDER BY is_builtin DESC, name ASC",
        ) {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                let active: Vec<String> = rows.flatten().collect();
                if let Ok(json_str) = serde_json::to_string(&active) {
                    let _ = conn.execute(
                        "INSERT INTO options (option_name, option_value) VALUES ('daypress_active_plugins', ?1)
                         ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                        params![json_str],
                    );
                }
            }
        }
    }

    pub fn ensure_builtin_plugins_seeded_with_conn(conn: &Connection) -> Result<()> {
        let catalog_ready: Option<String> = conn
            .query_row(
                "SELECT option_value FROM options WHERE option_name = 'daypress_plugins_catalog_v100_core_split'",
                [],
                |r| r.get(0),
            )
            .ok();
        let preexisting_builtin_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM daypress_plugins WHERE is_builtin = 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if catalog_ready.as_deref() == Some("1") && preexisting_builtin_count == 6 {
            return Ok(());
        }

        // Preserve active cookie-consent state on upgraded DBs before purging core rows from `daypress_plugins`.
        let cookie_plugin_was_active: bool = conn
            .query_row(
                "SELECT is_active FROM daypress_plugins WHERE slug = 'daypress-cookie-consent'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|v| v != 0)
            .unwrap_or(false);
        if cookie_plugin_was_active {
            let existing_banner_opt: Option<String> = conn
                .query_row(
                    "SELECT option_value FROM options WHERE option_name = 'daypress_cookie_banner_enabled'",
                    [],
                    |r| r.get(0),
                )
                .ok();
            if existing_banner_opt.is_none() {
                let _ = conn.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('daypress_cookie_banner_enabled', '1')
                     ON CONFLICT(option_name) DO NOTHING",
                    [],
                );
            }
        }

        // Purge built-in Core features that are no longer treated as plugins.
        for core_slug in crate::plugins::CORE_FEATURE_SLUGS {
            let _ = conn.execute(
                "DELETE FROM daypress_plugins WHERE slug = ?1",
                params![core_slug],
            );
        }

        let remaining_preexisting_builtin: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM daypress_plugins WHERE is_builtin = 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        for manifest in crate::plugins::builtin_plugins_catalog() {
            let mut default_cfg = std::collections::HashMap::new();
            for field in &manifest.settings_schema {
                default_cfg.insert(field.key.clone(), field.default_value.clone());
            }
            let default_active = false;
            let manifest_json =
                serde_json::to_string(&manifest).unwrap_or_else(|_| "{}".to_string());
            let config_json =
                serde_json::to_string(&default_cfg).unwrap_or_else(|_| "{}".to_string());

            let _ = conn.execute(
                r#"INSERT INTO daypress_plugins (slug, name, version, description, category, is_builtin, is_active, manifest_json, config_json, updated_at)
                   VALUES (?1, ?2, ?3, ?4, 'builtin', 1, ?5, ?6, ?7, ?8)
                   ON CONFLICT(slug) DO UPDATE SET
                       name = excluded.name,
                       version = excluded.version,
                       description = excluded.description,
                       is_builtin = 1,
                       manifest_json = excluded.manifest_json"#,
                params![
                    manifest.slug,
                    manifest.name,
                    manifest.version,
                    manifest.description,
                    if default_active { 1 } else { 0 },
                    manifest_json,
                    config_json,
                    now,
                ],
            );
        }

        // One-time migration: ensure pre-0.9 databases reset built-in plugins to deactivated by default.
        let migrated: Option<String> = conn
            .query_row(
                "SELECT option_value FROM options WHERE option_name = 'daypress_plugins_default_inactive_v09'",
                [],
                |r| r.get(0),
            )
            .ok();
        if migrated.is_none() {
            if remaining_preexisting_builtin > 0 {
                let _ = conn.execute(
                    "UPDATE daypress_plugins SET is_active = 0 WHERE is_builtin = 1",
                    [],
                );
            }
            let _ = conn.execute(
                "INSERT INTO options (option_name, option_value) VALUES ('daypress_plugins_default_inactive_v09', '1')
                 ON CONFLICT(option_name) DO UPDATE SET option_value = '1'",
                [],
            );
        }

        // Re-synchronize `active_plugins` and `daypress_active_plugins` so purged Core slugs are removed.
        if let Ok(mut stmt) = conn.prepare(
            "SELECT slug FROM daypress_plugins WHERE is_active = 1 ORDER BY is_builtin DESC, name ASC",
        ) {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                let active: Vec<String> = rows.flatten().collect();
                if let Ok(json_str) = serde_json::to_string(&active) {
                    let _ = conn.execute(
                        "INSERT INTO options (option_name, option_value) VALUES ('active_plugins', ?1)
                         ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                        params![json_str],
                    );
                    let _ = conn.execute(
                        "INSERT INTO options (option_name, option_value) VALUES ('daypress_active_plugins', ?1)
                         ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                        params![json_str],
                    );
                }
            }
        }

        let _ = conn.execute(
            "INSERT INTO options (option_name, option_value) VALUES ('daypress_plugins_catalog_v100_core_split', '1')
             ON CONFLICT(option_name) DO UPDATE SET option_value = '1'",
            [],
        );

        Ok(())
    }

    pub fn get_active_installed_plugins(&self) -> Result<Vec<InstalledPluginRecord>> {
        let conn = self.conn.lock().unwrap();
        let _ = Self::ensure_builtin_plugins_seeded_with_conn(&conn);
        let mut stmt = conn.prepare_cached(
            "SELECT slug, name, version, description, is_builtin, is_active, manifest_json, config_json, updated_at FROM daypress_plugins WHERE is_active = 1 ORDER BY is_builtin DESC, name ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            let slug: String = r.get(0)?;
            let name: String = r.get(1)?;
            let version: String = r.get(2)?;
            let description: String = r.get(3)?;
            let is_builtin: i64 = r.get(4)?;
            let is_active: i64 = r.get(5)?;
            let manifest_raw: String = r.get(6)?;
            let config_raw: String = r.get(7)?;
            let updated_at: String = r.get(8)?;

            let manifest: PluginManifest =
                serde_json::from_str(&manifest_raw).unwrap_or_else(|_| PluginManifest {
                    slug: slug.clone(),
                    name: name.clone(),
                    version: version.clone(),
                    author: "Community".to_string(),
                    description: description.clone(),
                    settings_url: None,
                    is_builtin: is_builtin != 0,
                    settings_schema: vec![],
                    hooks: PluginHooksManifest::default(),
                });
            let config: std::collections::HashMap<String, String> =
                serde_json::from_str(&config_raw).unwrap_or_default();

            Ok(InstalledPluginRecord {
                manifest,
                active: is_active != 0,
                config,
                installed_at: updated_at,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_all_installed_plugins(&self) -> Result<Vec<InstalledPluginRecord>> {
        let conn = self.conn.lock().unwrap();
        let _ = Self::ensure_builtin_plugins_seeded_with_conn(&conn);
        let mut stmt = conn.prepare(
            "SELECT slug, name, version, description, is_builtin, is_active, manifest_json, config_json, updated_at FROM daypress_plugins ORDER BY is_builtin DESC, name ASC",
        )?;
        let rows = stmt.query_map([], |r| {
            let slug: String = r.get(0)?;
            let name: String = r.get(1)?;
            let version: String = r.get(2)?;
            let description: String = r.get(3)?;
            let is_builtin: i64 = r.get(4)?;
            let is_active: i64 = r.get(5)?;
            let manifest_raw: String = r.get(6)?;
            let config_raw: String = r.get(7)?;
            let updated_at: String = r.get(8)?;

            let manifest: PluginManifest =
                serde_json::from_str(&manifest_raw).unwrap_or_else(|_| PluginManifest {
                    slug: slug.clone(),
                    name: name.clone(),
                    version: version.clone(),
                    author: "Community".to_string(),
                    description: description.clone(),
                    settings_url: None,
                    is_builtin: is_builtin != 0,
                    settings_schema: vec![],
                    hooks: PluginHooksManifest::default(),
                });
            let config: std::collections::HashMap<String, String> =
                serde_json::from_str(&config_raw).unwrap_or_default();

            Ok(InstalledPluginRecord {
                manifest,
                active: is_active != 0,
                config,
                installed_at: updated_at,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    pub fn get_installed_plugin(&self, slug: &str) -> Result<Option<InstalledPluginRecord>> {
        let all = self.get_all_installed_plugins()?;
        Ok(all.into_iter().find(|p| p.manifest.slug == slug))
    }

    pub fn is_plugin_active(&self, slug: &str) -> bool {
        if crate::plugins::is_core_feature_slug(slug) {
            return false;
        }
        if let Ok(conn) = self.conn.lock() {
            let res: Result<i64> = conn.query_row(
                "SELECT is_active FROM daypress_plugins WHERE slug = ?1",
                params![slug],
                |r| r.get(0),
            );
            matches!(res, Ok(v) if v != 0)
        } else {
            false
        }
    }

    pub fn toggle_plugin_status(&self, slug: &str, active: Option<bool>) -> Result<bool> {
        if crate::plugins::is_core_feature_slug(slug) {
            return Err(rusqlite::Error::InvalidParameterName(format!(
                "'{}' is a built-in Daypress Core feature and is not managed as a plugin.",
                slug
            )));
        }
        let conn = self.conn.lock().unwrap();
        let _ = Self::ensure_builtin_plugins_seeded_with_conn(&conn);
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let current: Option<i64> = conn
            .query_row(
                "SELECT is_active FROM daypress_plugins WHERE slug = ?1",
                params![slug],
                |r| r.get(0),
            )
            .ok();

        let new_state = match (active, current) {
            (Some(explicit), _) => explicit,
            (None, Some(cur)) => cur == 0,
            (None, None) => true,
        };

        let updated = conn.execute(
            "UPDATE daypress_plugins SET is_active = ?1, updated_at = ?2 WHERE slug = ?3",
            params![if new_state { 1 } else { 0 }, now, slug],
        )?;

        if updated == 0 {
            // If toggling a known catalog plugin not yet in daypress_plugins, insert it
            let manifest = if let Some(m) = crate::plugins::builtin_manifest(slug) {
                m
            } else {
                PluginManifest {
                    slug: slug.to_string(),
                    name: slug.replace('-', " "),
                    version: "1.0.0".to_string(),
                    author: "Daypress Community".to_string(),
                    description: "Installed via Daypress Plugin Directory.".to_string(),
                    settings_url: None,
                    is_builtin: false,
                    settings_schema: vec![],
                    hooks: PluginHooksManifest::default(),
                }
            };
            let is_builtin = manifest.is_builtin;
            let manifest_json =
                serde_json::to_string(&manifest).unwrap_or_else(|_| "{}".to_string());
            conn.execute(
                "INSERT INTO daypress_plugins (slug, name, version, description, category, is_builtin, is_active, manifest_json, config_json, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, '{}', ?9)",
                params![
                    slug,
                    manifest.name,
                    manifest.version,
                    manifest.description,
                    if is_builtin { "builtin" } else { "community" },
                    if is_builtin { 1 } else { 0 },
                    if new_state { 1 } else { 0 },
                    manifest_json,
                    now
                ],
            )?;
        }

        // Also keep legacy `active_plugins` wp_option synchronized
        let active_slugs: Vec<String> = conn
            .prepare("SELECT slug FROM daypress_plugins WHERE is_active = 1")
            .and_then(|mut s| {
                let rows = s.query_map([], |r| r.get::<_, String>(0))?;
                Ok(rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();
        if let Ok(json_str) = serde_json::to_string(&active_slugs) {
            let _ = conn.execute(
                "INSERT INTO options (option_name, option_value) VALUES ('active_plugins', ?1)
                 ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                params![json_str],
            );
        }

        Ok(new_state)
    }

    pub fn save_plugin_config(
        &self,
        slug: &str,
        config: &std::collections::HashMap<String, String>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let config_json = serde_json::to_string(config).unwrap_or_else(|_| "{}".to_string());
        conn.execute(
            "UPDATE daypress_plugins SET config_json = ?1, updated_at = ?2 WHERE slug = ?3",
            params![config_json, now, slug],
        )?;
        Ok(())
    }

    pub fn install_custom_plugin(
        &self,
        manifest: &PluginManifest,
        initial_config: Option<&std::collections::HashMap<String, String>>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let mut cfg = std::collections::HashMap::new();
        for field in &manifest.settings_schema {
            cfg.insert(field.key.clone(), field.default_value.clone());
        }
        if let Some(overrides) = initial_config {
            for (k, v) in overrides {
                cfg.insert(k.clone(), v.clone());
            }
        }
        let manifest_json = serde_json::to_string(manifest).unwrap_or_else(|_| "{}".to_string());
        let config_json = serde_json::to_string(&cfg).unwrap_or_else(|_| "{}".to_string());

        conn.execute(
            r#"INSERT INTO daypress_plugins (slug, name, version, description, category, is_builtin, is_active, manifest_json, config_json, updated_at)
               VALUES (?1, ?2, ?3, ?4, 'community', 0, 1, ?5, ?6, ?7)
               ON CONFLICT(slug) DO UPDATE SET
                   name = excluded.name,
                   version = excluded.version,
                   description = excluded.description,
                   is_active = 1,
                   manifest_json = excluded.manifest_json,
                   config_json = excluded.config_json,
                   updated_at = excluded.updated_at"#,
            params![
                manifest.slug,
                manifest.name,
                manifest.version,
                manifest.description,
                manifest_json,
                config_json,
                now
            ],
        )?;
        Ok(())
    }

    pub fn uninstall_custom_plugin(&self, slug: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute(
            "DELETE FROM daypress_plugins WHERE slug = ?1 AND is_builtin = 0",
            params![slug],
        )?;
        Ok(rows > 0)
    }

    // =========================================================================
    // NATIVE SUBSYSTEMS: HOT SQLITE SNAPSHOT & BACKUP MANAGER
    // =========================================================================

    pub fn list_backup_snapshots(&self) -> Result<Vec<BackupSnapshotItem>> {
        let _ = fs::create_dir_all("backups");
        let mut items = Vec::new();
        if let Ok(entries) = fs::read_dir("backups") {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(fname) = path.file_name().and_then(|s| s.to_str()) {
                    if fname.ends_with(".sqlite") || fname.ends_with(".db") {
                        let meta = entry.metadata().ok();
                        let size_bytes = meta.as_ref().map(|m| m.len()).unwrap_or(0);
                        let created_at = meta
                            .and_then(|m| m.modified().ok())
                            .map(|t| {
                                let dt: chrono::DateTime<Utc> = t.into();
                                dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
                            })
                            .unwrap_or_else(|| "Unknown".to_string());
                        items.push(BackupSnapshotItem {
                            filename: fname.to_string(),
                            size_bytes,
                            created_at,
                            format: "sqlite".to_string(),
                        });
                    }
                }
            }
        }
        items.sort_by(|a, b| b.filename.cmp(&a.filename));
        Ok(items)
    }

    pub fn create_sqlite_snapshot(&self, label: Option<&str>) -> Result<BackupSnapshotItem> {
        let _ = fs::create_dir_all("backups");
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S").to_string();
        let clean_label: String = label
            .unwrap_or("snapshot")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .take(24)
            .collect();
        let prefix = if clean_label.is_empty() {
            "snapshot"
        } else {
            &clean_label
        };
        let filename = format!("daypress_{}_{}.sqlite", prefix, timestamp);
        let target_path = Path::new("backups").join(&filename);
        if target_path.exists() {
            let _ = fs::remove_file(&target_path);
        }

        let escaped_path = target_path
            .to_string_lossy()
            .replace('\\', "/")
            .replace('\'', "''");
        {
            let conn = self.conn.lock().unwrap();
            conn.execute_batch(&format!("VACUUM INTO '{}';", escaped_path))?;
        }

        let size_bytes = fs::metadata(&target_path).map(|m| m.len()).unwrap_or(0);
        Ok(BackupSnapshotItem {
            filename,
            size_bytes,
            created_at: Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            format: "sqlite".to_string(),
        })
    }

    pub fn delete_backup_snapshot(&self, filename: &str) -> Result<bool> {
        let Some(safe_name) = crate::media::sanitize_filename(filename) else {
            return Ok(false);
        };
        let path = Path::new("backups").join(safe_name);
        if path.exists() {
            Ok(fs::remove_file(path).is_ok())
        } else {
            Ok(false)
        }
    }

    pub fn restore_sqlite_snapshot(&self, filename: &str) -> Result<bool> {
        let Some(safe_name) = crate::media::sanitize_filename(filename) else {
            return Ok(false);
        };
        let backup_path = Path::new("backups").join(safe_name);
        if !backup_path.exists() {
            return Ok(false);
        }

        let conn = self.conn.lock().unwrap();
        let escaped_path = backup_path
            .to_string_lossy()
            .replace('\\', "/")
            .replace('\'', "''");
        conn.execute(
            &format!("ATTACH DATABASE '{}' AS snap_db;", escaped_path),
            [],
        )?;

        let tables = [
            "options",
            "posts",
            "postmeta",
            "terms",
            "term_taxonomy",
            "term_relationships",
            "comments",
            "users",
            "usermeta",
            "daypress_forms",
            "daypress_form_entries",
            "daypress_redirects",
            "daypress_plugins",
        ];
        let mut from_neopress = false;
        for tbl in tables {
            // A table the snapshot lacks is left alone instead of being emptied.
            let Some(source) = crate::legacy::snapshot_table(&conn, "snap_db", tbl)
                .ok()
                .flatten()
            else {
                continue;
            };
            from_neopress |= source != tbl;
            let _ = conn.execute(&format!("DELETE FROM main.{tbl};"), []);
            let _ = conn.execute(
                &format!("INSERT OR IGNORE INTO main.{tbl} SELECT * FROM snap_db.\"{source}\";"),
                [],
            );
        }
        let _ = conn.execute("DETACH DATABASE snap_db;", []);
        if from_neopress {
            crate::legacy::upgrade_data(&conn)?;
        }
        Ok(true)
    }

    // =========================================================================
    // NATIVE SUBSYSTEMS: COOKIELESS ANALYTICS
    // =========================================================================

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
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let ev_type = if event_type.trim().is_empty() {
            "pageview"
        } else {
            event_type.trim()
        };
        conn.execute(
            r#"INSERT INTO daypress_analytics_events 
               (visitor_hash, page_url, referrer, device_type, country, utm_source, utm_medium, utm_campaign, event_type, goal_name, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"#,
            params![
                visitor_hash,
                page_url,
                referrer,
                device_type,
                country,
                utm_source.filter(|s| !s.trim().is_empty()),
                utm_medium.filter(|s| !s.trim().is_empty()),
                utm_campaign.filter(|s| !s.trim().is_empty()),
                ev_type,
                goal_name.filter(|s| !s.trim().is_empty()),
                now
            ],
        )?;
        drop(conn);
        self.record_ab_conversion_for_visitor(visitor_hash, ev_type, goal_name);
        Ok(())
    }

    pub fn get_analytics_summary(&self) -> Result<AnalyticsSummary> {
        self.get_analytics_dashboard("30d")
    }

    pub fn get_analytics_dashboard(&self, range: &str) -> Result<AnalyticsSummary> {
        use chrono::Duration;
        let conn = self.conn.lock().unwrap();
        let now = Utc::now();
        let clean_range = match range.trim() {
            "24h" | "7d" | "30d" | "90d" | "12m" => range.trim(),
            _ => "30d",
        };

        let cutoff_dt = match clean_range {
            "24h" => now - Duration::hours(24),
            "7d" => now - Duration::days(7),
            "90d" => now - Duration::days(90),
            "12m" => now - Duration::days(365),
            _ => now - Duration::days(30),
        };
        let cutoff_str = cutoff_dt.format("%Y-%m-%d %H:%M:%S").to_string();

        let total_visitors: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT visitor_hash) FROM daypress_analytics_events WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1",
                params![cutoff_str],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let total_pageviews: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM daypress_analytics_events WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1 AND COALESCE(event_type, 'pageview') = 'pageview'",
                params![cutoff_str],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let total_conversions: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM daypress_analytics_events WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1 AND COALESCE(event_type, 'pageview') IN ('form_submit', 'goal')",
                params![cutoff_str],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let single_hit_visitors: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM (SELECT visitor_hash FROM daypress_analytics_events WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1 GROUP BY visitor_hash HAVING COUNT(*) = 1)",
                params![cutoff_str],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let bounce_rate_pct = if total_visitors == 0 {
            0.0
        } else {
            (single_hit_visitors as f64 / total_visitors as f64) * 100.0
        };

        // Time-series buckets
        let mut bucket_map: std::collections::BTreeMap<String, (i64, i64, i64)> =
            std::collections::BTreeMap::new();
        match clean_range {
            "24h" => {
                for h in (0..24).rev() {
                    let b = (now - Duration::hours(h))
                        .format("%Y-%m-%d %H:00")
                        .to_string();
                    bucket_map.insert(b, (0, 0, 0));
                }
            }
            "12m" => {
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

        let bucket_sql = match clean_range {
            "24h" => {
                r#"SELECT substr(replace(created_at, 'T', ' '), 1, 13) || ':00' AS b,
                          COUNT(DISTINCT visitor_hash),
                          SUM(CASE WHEN COALESCE(event_type, 'pageview') = 'pageview' THEN 1 ELSE 0 END),
                          SUM(CASE WHEN COALESCE(event_type, 'pageview') IN ('form_submit', 'goal') THEN 1 ELSE 0 END)
                   FROM daypress_analytics_events
                   WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1
                   GROUP BY b ORDER BY b ASC"#
            }
            "12m" => {
                r#"SELECT substr(replace(created_at, 'T', ' '), 1, 7) AS b,
                          COUNT(DISTINCT visitor_hash),
                          SUM(CASE WHEN COALESCE(event_type, 'pageview') = 'pageview' THEN 1 ELSE 0 END),
                          SUM(CASE WHEN COALESCE(event_type, 'pageview') IN ('form_submit', 'goal') THEN 1 ELSE 0 END)
                   FROM daypress_analytics_events
                   WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1
                   GROUP BY b ORDER BY b ASC"#
            }
            _ => {
                r#"SELECT substr(replace(created_at, 'T', ' '), 1, 10) AS b,
                          COUNT(DISTINCT visitor_hash),
                          SUM(CASE WHEN COALESCE(event_type, 'pageview') = 'pageview' THEN 1 ELSE 0 END),
                          SUM(CASE WHEN COALESCE(event_type, 'pageview') IN ('form_submit', 'goal') THEN 1 ELSE 0 END)
                   FROM daypress_analytics_events
                   WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1
                   GROUP BY b ORDER BY b ASC"#
            }
        };

        if let Ok(mut b_stmt) = conn.prepare(bucket_sql) {
            if let Ok(rows) = b_stmt.query_map(params![cutoff_str], |r| {
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

        // Top Pages
        let mut stmt = conn.prepare(
            r#"SELECT page_url, COUNT(*) as views, COUNT(DISTINCT visitor_hash) as visitors
               FROM daypress_analytics_events
               WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1
                 AND COALESCE(event_type, 'pageview') = 'pageview'
               GROUP BY page_url ORDER BY views DESC LIMIT 10"#,
        )?;
        let top_pages = stmt
            .query_map(params![cutoff_str], |r| {
                Ok(TopPageMetric {
                    page_url: r.get(0)?,
                    views: r.get(1)?,
                    visitors: r.get(2).unwrap_or(0),
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        let query_breakdown = |sql: &str| -> Vec<BreakdownMetric> {
            let Ok(mut s) = conn.prepare(sql) else {
                return Vec::new();
            };
            let Ok(rows) = s.query_map(params![cutoff_str], |r| {
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

        let top_referrers = query_breakdown(
            r#"SELECT CASE WHEN TRIM(COALESCE(referrer, '')) = '' THEN 'Direct / None' ELSE referrer END AS lbl,
                      COUNT(DISTINCT visitor_hash) AS vis, COUNT(*) AS vws
               FROM daypress_analytics_events
               WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1
               GROUP BY lbl ORDER BY vws DESC LIMIT 10"#,
        );

        let utm_sources = query_breakdown(
            r#"SELECT utm_source AS lbl, COUNT(DISTINCT visitor_hash) AS vis, COUNT(*) AS vws
               FROM daypress_analytics_events
               WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1
                 AND utm_source IS NOT NULL AND TRIM(utm_source) != ''
               GROUP BY lbl ORDER BY vws DESC LIMIT 10"#,
        );

        let utm_mediums = query_breakdown(
            r#"SELECT utm_medium AS lbl, COUNT(DISTINCT visitor_hash) AS vis, COUNT(*) AS vws
               FROM daypress_analytics_events
               WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1
                 AND utm_medium IS NOT NULL AND TRIM(utm_medium) != ''
               GROUP BY lbl ORDER BY vws DESC LIMIT 10"#,
        );

        let utm_campaigns = query_breakdown(
            r#"SELECT utm_campaign AS lbl, COUNT(DISTINCT visitor_hash) AS vis, COUNT(*) AS vws
               FROM daypress_analytics_events
               WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1
                 AND utm_campaign IS NOT NULL AND TRIM(utm_campaign) != ''
               GROUP BY lbl ORDER BY vws DESC LIMIT 10"#,
        );

        let devices = query_breakdown(
            r#"SELECT COALESCE(NULLIF(TRIM(device_type), ''), 'desktop') AS lbl,
                      COUNT(DISTINCT visitor_hash) AS vis, COUNT(*) AS vws
               FROM daypress_analytics_events
               WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1
               GROUP BY lbl ORDER BY vws DESC LIMIT 10"#,
        );

        let countries = query_breakdown(
            r#"SELECT COALESCE(NULLIF(TRIM(country), ''), 'XX') AS lbl,
                      COUNT(DISTINCT visitor_hash) AS vis, COUNT(*) AS vws
               FROM daypress_analytics_events
               WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1
               GROUP BY lbl ORDER BY vws DESC LIMIT 10"#,
        );

        let mut conversions = Vec::new();
        if let Ok(mut c_stmt) = conn.prepare(
            r#"SELECT COALESCE(NULLIF(TRIM(goal_name), ''), COALESCE(event_type, 'goal')) AS gname,
                      COUNT(*) AS cnt,
                      COUNT(DISTINCT visitor_hash) AS vis
               FROM daypress_analytics_events
               WHERE substr(replace(created_at, 'T', ' '), 1, 19) >= ?1
                 AND COALESCE(event_type, 'pageview') IN ('form_submit', 'goal')
               GROUP BY gname ORDER BY cnt DESC LIMIT 15"#,
        ) {
            if let Ok(rows) = c_stmt.query_map(params![cutoff_str], |r| {
                let goal_name: String = r.get(0)?;
                let count: i64 = r.get(1).unwrap_or(0);
                let unique_visitors: i64 = r.get(2).unwrap_or(0);
                let conversion_rate_pct = if total_visitors == 0 {
                    0.0
                } else {
                    (unique_visitors as f64 / total_visitors as f64) * 100.0
                };
                Ok(ConversionMetric {
                    goal_name,
                    count,
                    unique_visitors,
                    conversion_rate_pct,
                })
            }) {
                conversions = rows.filter_map(|r| r.ok()).collect();
            }
        }

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
        })
    }

    // =========================================================================
    // NATIVE SUBSYSTEMS: TRANSACTIONAL EMAIL LOGGING
    // =========================================================================

    pub fn log_email(
        &self,
        recipient: &str,
        subject: &str,
        transport: &str,
        status: &str,
        latency_ms: i64,
        error_message: Option<&str>,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            r#"INSERT INTO daypress_email_log 
               (recipient, subject, transport, status, latency_ms, error_message, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                recipient,
                subject,
                transport,
                status,
                latency_ms,
                error_message,
                now
            ],
        )?;
        Ok(())
    }

    pub fn get_email_logs(&self, limit: i64) -> Result<Vec<EmailLogEntry>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, recipient, subject, transport, status, latency_ms, error_message, created_at FROM daypress_email_log ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit], |r| {
            Ok(EmailLogEntry {
                id: r.get(0)?,
                recipient: r.get(1)?,
                subject: r.get(2)?,
                transport: r.get(3)?,
                status: r.get(4)?,
                latency_ms: r.get(5)?,
                error_message: r.get(6)?,
                created_at: r.get(7)?,
            })
        })?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }
}
