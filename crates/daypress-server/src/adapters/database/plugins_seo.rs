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
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO daypress_forms (title, schema_json, settings_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
            params![title, schema_json, settings_json, now],
        )?;
        Ok(conn.last_insert_rowid())
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
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO daypress_form_entries (form_id, data_json, ip_hash, user_agent, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![form_id, data_json, ip_hash, user_agent, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_forms(&self) -> Result<Vec<(DaypressForm, i64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT f.id, f.title, f.schema_json, f.settings_json, f.created_at, f.updated_at,
                      (SELECT COUNT(*) FROM daypress_form_entries e WHERE e.form_id = f.id)
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
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let schema_json = serde_json::to_string(&req.fields).unwrap_or_else(|_| "[]".to_string());
        let settings_json =
            serde_json::to_string(&req.settings).unwrap_or_else(|_| "{}".to_string());
        let title = if req.title.trim().is_empty() {
            "Untitled Form"
        } else {
            req.title.trim()
        };

        if let Some(id) = req.id.filter(|&id| id > 0) {
            let updated = conn.execute(
                "UPDATE daypress_forms SET title = ?1, schema_json = ?2, settings_json = ?3, updated_at = ?4 WHERE id = ?5",
                params![title, schema_json, settings_json, now, id],
            )?;
            if updated > 0 {
                return Ok(id);
            }
        }

        conn.execute(
            "INSERT INTO daypress_forms (title, schema_json, settings_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
            params![title, schema_json, settings_json, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn delete_form(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "DELETE FROM daypress_form_entries WHERE form_id = ?1",
            params![id],
        );
        let rows = conn.execute("DELETE FROM daypress_forms WHERE id = ?1", params![id])?;
        Ok(rows > 0)
    }

    pub fn list_form_entries(
        &self,
        form_id: Option<i64>,
        limit: i64,
    ) -> Result<Vec<FormEntryItem>> {
        let conn = self.conn.lock().unwrap();
        let sql = if form_id.is_some() {
            r#"SELECT e.id, e.form_id, COALESCE(f.title, 'Form'), e.data_json, e.ip_hash, e.user_agent, e.created_at
               FROM daypress_form_entries e
               LEFT JOIN daypress_forms f ON e.form_id = f.id
               WHERE e.form_id = ?1
               ORDER BY e.id DESC LIMIT ?2"#
        } else {
            r#"SELECT e.id, e.form_id, COALESCE(f.title, 'Form'), e.data_json, e.ip_hash, e.user_agent, e.created_at
               FROM daypress_form_entries e
               LEFT JOIN daypress_forms f ON e.form_id = f.id
               ORDER BY e.id DESC LIMIT ?1"#
        };

        let mut stmt = conn.prepare(sql)?;
        let map_row = |r: &rusqlite::Row| -> rusqlite::Result<FormEntryItem> {
            Ok(FormEntryItem {
                id: r.get(0)?,
                form_id: r.get(1)?,
                form_title: r.get(2)?,
                data_json: r.get(3)?,
                ip_hash: r.get(4)?,
                user_agent: r.get(5)?,
                created_at: r.get(6)?,
            })
        };

        let entries: Vec<FormEntryItem> = if let Some(fid) = form_id {
            stmt.query_map(params![fid, limit], map_row)?
                .filter_map(|r| r.ok())
                .collect()
        } else {
            stmt.query_map(params![limit], map_row)?
                .filter_map(|r| r.ok())
                .collect()
        };
        Ok(entries)
    }

    pub fn delete_form_entry(&self, entry_id: i64) -> Result<bool> {
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

    pub fn ensure_builtin_plugins_seeded_with_conn(conn: &Connection) -> Result<()> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        for manifest in crate::plugins::builtin_plugins_catalog() {
            let mut default_cfg = std::collections::HashMap::new();
            for field in &manifest.settings_schema {
                default_cfg.insert(field.key.clone(), field.default_value.clone());
            }
            let is_always_on = crate::plugins::is_core_always_on(&manifest.slug);
            let default_active = is_always_on
                || !matches!(manifest.slug.as_str(), "whatsapp-chat" | "code-snippets");
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
                       manifest_json = excluded.manifest_json,
                       is_active = CASE WHEN ?9 = 1 THEN 1 ELSE daypress_plugins.is_active END"#,
                params![
                    manifest.slug,
                    manifest.name,
                    manifest.version,
                    manifest.description,
                    if default_active { 1 } else { 0 },
                    manifest_json,
                    config_json,
                    now,
                    if is_always_on { 1 } else { 0 },
                ],
            );
        }
        Ok(())
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
        if crate::plugins::is_core_always_on(slug) {
            return true;
        }
        if let Ok(conn) = self.conn.lock() {
            let res: Result<i64> = conn.query_row(
                "SELECT is_active FROM daypress_plugins WHERE slug = ?1",
                params![slug],
                |r| r.get(0),
            );
            match res {
                Ok(v) => v != 0,
                Err(_) => {
                    // Default active for built-in catalog plugins before table seed
                    matches!(
                        slug,
                        "daypress-seo" | "daypress-forms" | "daypress-wal-optimizer"
                    )
                }
            }
        } else {
            matches!(
                slug,
                "daypress-seo" | "daypress-forms" | "daypress-wal-optimizer"
            )
        }
    }

    pub fn toggle_plugin_status(&self, slug: &str, active: Option<bool>) -> Result<bool> {
        if crate::plugins::is_core_always_on(slug) {
            if active == Some(false) {
                return Err(rusqlite::Error::InvalidParameterName(format!(
                    "Core subsystem plugin '{}' is always active and cannot be deactivated.",
                    slug
                )));
            }
            return Ok(true);
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
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();
        conn.execute(
            r#"INSERT INTO daypress_analytics_events 
               (visitor_hash, page_url, referrer, device_type, country, utm_source, utm_medium, utm_campaign, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
            params![
                visitor_hash,
                page_url,
                referrer,
                device_type,
                country,
                utm_source,
                utm_medium,
                utm_campaign,
                now
            ],
        )?;
        Ok(())
    }

    pub fn get_analytics_summary(&self) -> Result<AnalyticsSummary> {
        let conn = self.conn.lock().unwrap();
        let total_visitors: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT visitor_hash) FROM daypress_analytics_events",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        let total_pageviews: i64 = conn
            .query_row("SELECT COUNT(*) FROM daypress_analytics_events", [], |r| {
                r.get(0)
            })
            .unwrap_or(0);

        let mut stmt = conn.prepare(
            "SELECT page_url, COUNT(*) as views FROM daypress_analytics_events GROUP BY page_url ORDER BY views DESC LIMIT 10",
        )?;
        let top_pages = stmt
            .query_map([], |r| {
                Ok(TopPageMetric {
                    page_url: r.get(0)?,
                    views: r.get(1)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(AnalyticsSummary {
            total_visitors,
            total_pageviews,
            top_pages,
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
