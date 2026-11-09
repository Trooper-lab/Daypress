//! v0.9 Pillar 4: Sandboxed WebAssembly (`.wasm`) Extension Crates (`adapters/database/wasm_extensions.rs`).

use super::Database;
use crate::errors::AppError;
use crate::models::{InstallWasmExtensionRequest, WasmExtensionRecord};
use base64::Engine;
use chrono::Utc;
use daypress_wasm::{
    compile_template_to_wasm_binary, execute_wasm_export, inspect_wasm_binary, WasmCapabilities,
    WasmExecutionOutput, WasmHostEnvironment, DEFAULT_FUEL_LIMIT, DEFAULT_MAX_MEMORY_PAGES,
};
use rusqlite::{params, Result};
use std::collections::HashMap;

struct SqliteWasmHost<'a> {
    db: &'a Database,
    extension_slug: String,
}

impl<'a> WasmHostEnvironment for SqliteWasmHost<'a> {
    fn kv_get(&self, key: &str) -> std::result::Result<Option<String>, String> {
        let conn = self.db.conn.lock().map_err(|e| e.to_string())?;
        let val: Option<String> = conn
            .query_row(
                "SELECT kv_value FROM daypress_wasm_kv WHERE extension_slug = ?1 AND kv_key = ?2",
                params![self.extension_slug, key],
                |r| r.get(0),
            )
            .ok();
        Ok(val)
    }

    fn kv_set(&mut self, key: &str, value: &str) -> std::result::Result<(), String> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let conn = self.db.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            r#"INSERT INTO daypress_wasm_kv (extension_slug, kv_key, kv_value, updated_at)
               VALUES (?1, ?2, ?3, ?4)
               ON CONFLICT(extension_slug, kv_key) DO UPDATE SET
                   kv_value = excluded.kv_value,
                   updated_at = excluded.updated_at"#,
            params![self.extension_slug, key, value, now],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn get_option(&self, option_name: &str) -> Option<String> {
        self.db.get_option(option_name).ok().flatten()
    }
}

fn parse_shortcode_attrs(attr_str: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let normalized = attr_str
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#039;", "'");
    if let Ok(re) =
        regex::Regex::new(r#"([a-zA-Z0-9_-]+)\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s\]]+))"#)
    {
        for caps in re.captures_iter(&normalized) {
            let key = caps.get(1).map(|m| m.as_str()).unwrap_or("").to_string();
            let val = caps
                .get(2)
                .or_else(|| caps.get(3))
                .or_else(|| caps.get(4))
                .map(|m| m.as_str())
                .unwrap_or("")
                .to_string();
            if !key.is_empty() {
                map.insert(key, val);
            }
        }
    }
    map
}

impl Database {
    fn map_wasm_extension_row(r: &rusqlite::Row) -> rusqlite::Result<WasmExtensionRecord> {
        let caps_json: String = r.get(6)?;
        let capabilities: WasmCapabilities = serde_json::from_str(&caps_json).unwrap_or_default();
        let wasm_bytes: Vec<u8> = r.get(10)?;
        let exports = inspect_wasm_binary(&wasm_bytes)
            .map(|i| i.exports)
            .unwrap_or_default();
        Ok(WasmExtensionRecord {
            id: r.get(0)?,
            slug: r.get(1)?,
            name: r.get(2)?,
            version: r.get(3)?,
            description: r.get(4)?,
            shortcode: r.get(5)?,
            capabilities,
            fuel_limit: r.get::<_, i64>(7)?.max(100) as u64,
            max_memory_pages: r.get::<_, i64>(8)?.clamp(1, 256) as u32,
            wasm_sha256: r.get(9)?,
            wasm_size_bytes: wasm_bytes.len(),
            exports,
            is_active: r.get::<_, i64>(11)? != 0,
            executions: r.get(12)?,
            last_error: r.get(13)?,
            created_at: r.get(14)?,
        })
    }

    pub fn install_or_update_wasm_extension(
        &self,
        req: &InstallWasmExtensionRequest,
    ) -> std::result::Result<WasmExtensionRecord, AppError> {
        let slug = generated_slugify::slugify(req.slug.trim());
        if slug.is_empty() {
            return Err(AppError::BadRequest(
                "WebAssembly extension slug cannot be empty.".to_string(),
            ));
        }
        let name = if req.name.trim().is_empty() {
            slug.clone()
        } else {
            req.name.trim().to_string()
        };
        let version = if req.version.trim().is_empty() {
            "1.0.0".to_string()
        } else {
            req.version.trim().to_string()
        };
        let shortcode = req
            .shortcode
            .trim()
            .trim_matches(|c| c == '[' || c == ']')
            .to_string();

        let wasm_bytes = if let Some(b64) =
            req.wasm_base64.as_ref().filter(|s| !s.trim().is_empty())
        {
            base64::engine::general_purpose::STANDARD
                .decode(b64.trim())
                .map_err(|e| AppError::BadRequest(format!("Invalid base64 .wasm payload: {e}")))?
        } else if let Some(tpl) = req
            .template_source
            .as_ref()
            .filter(|s| !s.trim().is_empty())
        {
            compile_template_to_wasm_binary(tpl, 1)
        } else {
            return Err(AppError::BadRequest(
                "Provide either `wasm_base64` (raw .wasm binary) or `template_source`.".to_string(),
            ));
        };

        let inspection =
            inspect_wasm_binary(&wasm_bytes).map_err(|e| AppError::BadRequest(e.to_string()))?;

        let fuel_limit = req
            .fuel_limit
            .unwrap_or(DEFAULT_FUEL_LIMIT)
            .clamp(100, 5_000_000);
        let max_memory_pages = req
            .max_memory_pages
            .unwrap_or(DEFAULT_MAX_MEMORY_PAGES)
            .clamp(1, 256);
        if inspection.initial_memory_pages > max_memory_pages {
            return Err(AppError::BadRequest(format!(
                "Wasm module requests {} initial memory pages, exceeding max_memory_pages ({})",
                inspection.initial_memory_pages, max_memory_pages
            )));
        }

        let caps_json = serde_json::to_string(&req.capabilities)
            .unwrap_or_else(|_| "{\"shortcodes\":true}".to_string());
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let conn = self.conn.lock().unwrap();
        conn.execute(
            r#"INSERT INTO daypress_wasm_extensions
               (slug, name, version, description, shortcode, capabilities_json, fuel_limit, max_memory_pages, wasm_sha256, wasm_bytes, is_active, created_at)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
               ON CONFLICT(slug) DO UPDATE SET
                   name = excluded.name,
                   version = excluded.version,
                   description = excluded.description,
                   shortcode = excluded.shortcode,
                   capabilities_json = excluded.capabilities_json,
                   fuel_limit = excluded.fuel_limit,
                   max_memory_pages = excluded.max_memory_pages,
                   wasm_sha256 = excluded.wasm_sha256,
                   wasm_bytes = excluded.wasm_bytes,
                   is_active = excluded.is_active,
                   last_error = NULL"#,
            params![
                slug,
                name,
                version,
                req.description.trim(),
                shortcode,
                caps_json,
                fuel_limit as i64,
                max_memory_pages as i64,
                inspection.sha256,
                wasm_bytes,
                if req.is_active { 1_i64 } else { 0_i64 },
                now,
            ],
        )?;

        let rec = conn.query_row(
            r#"SELECT id, slug, name, version, description, shortcode, capabilities_json, fuel_limit, max_memory_pages, wasm_sha256, wasm_bytes, is_active, executions, last_error, created_at
               FROM daypress_wasm_extensions WHERE slug = ?1 COLLATE NOCASE"#,
            params![slug],
            Self::map_wasm_extension_row,
        )?;
        drop(conn);
        self.invalidate_hot_cache("wasm_extension_installed");
        Ok(rec)
    }

    pub fn list_wasm_extensions(&self) -> Result<Vec<WasmExtensionRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, slug, name, version, description, shortcode, capabilities_json, fuel_limit, max_memory_pages, wasm_sha256, wasm_bytes, is_active, executions, last_error, created_at
               FROM daypress_wasm_extensions
               ORDER BY id DESC"#,
        )?;
        let rows = stmt.query_map([], Self::map_wasm_extension_row)?;
        Ok(rows.flatten().collect())
    }

    pub fn delete_wasm_extension(&self, slug: &str) -> Result<bool> {
        let clean = slug.trim();
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "DELETE FROM daypress_wasm_kv WHERE extension_slug = ?1 COLLATE NOCASE",
            params![clean],
        );
        let deleted = conn.execute(
            "DELETE FROM daypress_wasm_extensions WHERE slug = ?1 COLLATE NOCASE",
            params![clean],
        )?;
        drop(conn);
        if deleted > 0 {
            self.invalidate_hot_cache("wasm_extension_deleted");
        }
        Ok(deleted > 0)
    }

    /// Executes an installed `.wasm` extension inside the fuel-metered, capability-gated sandbox.
    pub fn execute_wasm_extension(
        &self,
        slug: &str,
        export_name: &str,
        attrs: &HashMap<String, String>,
        input_content: &str,
    ) -> std::result::Result<WasmExecutionOutput, AppError> {
        let clean = slug.trim();
        let (ext_slug, caps_json, fuel_limit, max_pages, wasm_bytes): (
            String,
            String,
            u64,
            u32,
            Vec<u8>,
        ) = {
            let conn = self.conn.lock().unwrap();
            conn.query_row(
                r#"SELECT slug, capabilities_json, fuel_limit, max_memory_pages, wasm_bytes
                   FROM daypress_wasm_extensions
                   WHERE slug = ?1 COLLATE NOCASE AND is_active = 1"#,
                params![clean],
                |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?.max(100) as u64,
                        r.get::<_, i64>(3)?.clamp(1, 256) as u32,
                        r.get::<_, Vec<u8>>(4)?,
                    ))
                },
            )
            .map_err(|_| {
                AppError::NotFound(format!(
                    "Active WebAssembly extension '{}' not found",
                    clean
                ))
            })?
        };

        let capabilities: WasmCapabilities = serde_json::from_str(&caps_json).unwrap_or_default();
        let mut host = SqliteWasmHost {
            db: self,
            extension_slug: ext_slug.clone(),
        };

        match execute_wasm_export(
            &wasm_bytes,
            export_name,
            attrs,
            input_content,
            &capabilities,
            fuel_limit,
            max_pages,
            &mut host,
        ) {
            Ok(output) => {
                if let Ok(conn) = self.conn.lock() {
                    let _ = conn.execute(
                        "UPDATE daypress_wasm_extensions SET executions = executions + 1, last_error = NULL WHERE slug = ?1 COLLATE NOCASE",
                        params![ext_slug],
                    );
                }
                Ok(output)
            }
            Err(err) => {
                let err_msg = err.to_string();
                if let Ok(conn) = self.conn.lock() {
                    let _ = conn.execute(
                        "UPDATE daypress_wasm_extensions SET last_error = ?1 WHERE slug = ?2 COLLATE NOCASE",
                        params![err_msg, ext_slug],
                    );
                }
                Err(AppError::BadRequest(err_msg))
            }
        }
    }

    /// Expands any `[shortcode attr="val"]` belonging to an active `.wasm` extension in `content`.
    pub fn expand_active_wasm_shortcodes(&self, content: &str) -> String {
        if !content.contains('[') {
            return content.to_string();
        }
        let active_exts: Vec<(String, String)> = {
            let Ok(conn) = self.conn.lock() else {
                return content.to_string();
            };
            let Ok(mut stmt) = conn.prepare(
                "SELECT slug, shortcode FROM daypress_wasm_extensions WHERE is_active = 1 AND shortcode != ''",
            ) else {
                return content.to_string();
            };
            stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .ok()
                .map(|rows| rows.flatten().collect())
                .unwrap_or_default()
        };
        if active_exts.is_empty() {
            return content.to_string();
        }

        let mut out = content.to_string();
        for (slug, sc) in active_exts {
            let marker = format!("[{}", sc);
            if !out.contains(&marker) {
                continue;
            }
            let pattern = format!(r#"\[{}\b([^\]]*)\]"#, regex::escape(&sc));
            if let Ok(re) = regex::Regex::new(&pattern) {
                out = re
                    .replace_all(&out, |caps: &regex::Captures| {
                        let attr_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                        let attrs = parse_shortcode_attrs(attr_str);
                        match self.execute_wasm_extension(&slug, "render_shortcode", &attrs, "") {
                            Ok(res) => res.output_html,
                            Err(err) => format!(
                                "<!-- daypress-wasm [{}] trapped: {} -->",
                                crate::templates::utils::escape_html(&slug),
                                crate::templates::utils::escape_html(&err.to_string())
                            ),
                        }
                    })
                    .into_owned();
            }
        }
        out
    }
}
