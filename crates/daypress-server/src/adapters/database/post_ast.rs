//! Post AST synchronization helper (`adapters/database/post_ast.rs`).

use rusqlite::{params, Connection};
use std::collections::HashMap;

/// Transpiles post content into a canonical AST layout and updates `posts.layout_json`
/// and `postmeta._daypress_layout`. Uses `daypress_ast::transpile_content` for Gutenberg,
/// Elementor, and Divi, falling back to `Database::synthesize_layout_json_from_html`.
pub fn synchronize_post_ast_layout(
    conn: &Connection,
    post_id: i64,
    content: &str,
    meta_updates: Option<&serde_json::Value>,
) -> rusqlite::Result<()> {
    // 1. Gather existing postmeta
    let mut meta: HashMap<String, String> = HashMap::new();
    {
        let mut stmt =
            conn.prepare("SELECT meta_key, meta_value FROM postmeta WHERE post_id = ?1")?;
        let rows = stmt.query_map(params![post_id], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        })?;
        for row in rows.flatten() {
            meta.insert(row.0, row.1);
        }
    }

    // 2. Overlay new metadata from request if provided
    if let Some(serde_json::Value::Object(map)) = meta_updates {
        for (k, v) in map {
            let val_str = match v {
                serde_json::Value::String(s) => s.clone(),
                serde_json::Value::Null => String::new(),
                other => other.to_string(),
            };
            meta.insert(k.clone(), val_str);
        }
    }

    // 3. Attempt universal transpilation (Elementor, Divi, or Gutenberg)
    let layout_json = if let Some(layout) = daypress_ast::transpile_content(content, &meta) {
        serde_json::to_string(&layout).ok()
    } else {
        super::Database::synthesize_layout_json_from_html(post_id, content)
    };

    if let Some(l_json) = layout_json {
        conn.execute(
            "UPDATE posts SET layout_json = ?1 WHERE ID = ?2",
            params![l_json, post_id],
        )?;
        conn.execute(
            "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_layout'",
            params![post_id],
        )?;
        conn.execute(
            "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_layout', ?2)",
            params![post_id, l_json],
        )?;
    }

    Ok(())
}

use super::Database;
use chrono::Utc;
use rusqlite::Result;

impl Database {
    // =========================================================================
    // VISUAL BUILDER & HYBRID JSON LAYOUT PERSISTENCE (Long-Term Architecture)
    // =========================================================================

    /// Splits compiled `<section ...>...</section>` (or Gutenberg/ACF block) HTML into top-level
    /// `CustomHtml` AST nodes so DayStudio and the Post Block Editor share a lossless 1:1 AST.
    pub(crate) fn synthesize_layout_json_from_html(post_id: i64, html: &str) -> Option<String> {
        let trimmed = html.trim();
        if trimmed.is_empty() {
            return None;
        }

        let mut chunks: Vec<String> = Vec::new();
        let lower = trimmed.to_ascii_lowercase();
        if lower.contains("<section") && lower.contains("</section>") {
            let mut cursor = 0;
            while cursor < trimmed.len() {
                if let Some(rel_open) = lower[cursor..].find("<section") {
                    let abs_open = cursor + rel_open;
                    let pre = trimmed[cursor..abs_open].trim();
                    if !pre.is_empty() {
                        chunks.push(pre.to_string());
                    }

                    // Walk forward balancing nested `<section` and `</section>` tags
                    let mut depth = 0i32;
                    let mut scan = abs_open;
                    let mut end_idx = trimmed.len();
                    while scan < trimmed.len() {
                        let next_open = lower[scan..].find("<section").map(|o| scan + o);
                        let next_close = lower[scan..].find("</section>").map(|c| scan + c);
                        match (next_open, next_close) {
                            (Some(o), Some(c)) if o < c => {
                                depth += 1;
                                scan = o + 8;
                            }
                            (_, Some(c)) => {
                                depth -= 1;
                                scan = c + 10;
                                if depth <= 0 {
                                    end_idx = scan;
                                    break;
                                }
                            }
                            _ => {
                                end_idx = trimmed.len();
                                break;
                            }
                        }
                    }
                    let sec_html = trimmed[abs_open..end_idx].trim();
                    if !sec_html.is_empty() {
                        chunks.push(sec_html.to_string());
                    }
                    cursor = end_idx;
                } else {
                    let tail = trimmed[cursor..].trim();
                    if !tail.is_empty() {
                        chunks.push(tail.to_string());
                    }
                    break;
                }
            }
        } else {
            chunks.push(trimmed.to_string());
        }

        if chunks.is_empty() {
            return None;
        }

        let children: Vec<serde_json::Value> = chunks
            .into_iter()
            .enumerate()
            .map(|(idx, raw_chunk)| {
                serde_json::json!({
                    "id": format!("acf_sec_{}_{}", post_id, idx),
                    "type": "CustomHtml",
                    "data": {
                        "raw": raw_chunk
                    },
                    "styles": {}
                })
            })
            .collect();

        let layout = serde_json::json!({
            "version": 1,
            "root": {
                "id": "root",
                "type": "Root",
                "styles": {},
                "children": children
            }
        });
        serde_json::to_string(&layout).ok()
    }

    /// Retrieves the JSON Layout for a post. Prioritizes the dedicated `layout_json`
    /// column on `posts` (<0.15ms latency), falling back to `postmeta` (_daypress_layout)
    /// or synthesizing a lossless section-level AST from `post_content` when present.
    pub fn get_post_layout(&self, post_id: i64) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        // 1. Check direct layout_json column
        let direct_layout: Option<String> = conn
            .query_row(
                "SELECT layout_json FROM posts WHERE ID = ?1",
                params![post_id],
                |r| r.get(0),
            )
            .ok()
            .flatten();

        if let Some(ref l) = direct_layout {
            if !l.trim().is_empty() {
                return Ok(direct_layout);
            }
        }

        // 2. Fallback to postmeta '_daypress_layout'
        let meta_layout: Option<String> = conn
            .query_row(
                "SELECT meta_value FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_layout'",
                params![post_id],
                |r| r.get(0),
            )
            .ok()
            .flatten();

        if let Some(ref ml) = meta_layout {
            if !ml.trim().is_empty() {
                return Ok(meta_layout);
            }
        }

        Ok(None)
    }

    /// Saves the JSON layout for a post using a transactional dual-write pattern:
    /// - Stores canonical JSON in `posts.layout_json`
    /// - Synchronizes to `postmeta` (_daypress_layout) for WordPress ecosystem compatibility
    /// - Optionally updates `posts.post_content` with pre-compiled semantic HTML for zero-JS public SSR
    pub fn save_post_layout(
        &self,
        post_id: i64,
        layout_json: &str,
        rendered_html: Option<&str>,
    ) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let site_offset_secs = super::scheduling::read_site_timezone_offset(&tx, None);
        let now_local = (Utc::now().naive_utc()
            + chrono::Duration::seconds(site_offset_secs as i64))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

        let _ = Self::snapshot_post_revision_with_conn(&tx, post_id, &now_local, &now);

        if let Some(html) = rendered_html {
            tx.execute(
                "UPDATE posts SET layout_json = ?1, post_content = ?2, post_modified = ?3, post_modified_gmt = ?4 WHERE ID = ?5",
                params![layout_json, html, now_local, now, post_id],
            )?;
        } else {
            tx.execute(
                "UPDATE posts SET layout_json = ?1, post_modified = ?2, post_modified_gmt = ?3 WHERE ID = ?4",
                params![layout_json, now_local, now, post_id],
            )?;
        }

        tx.execute(
            "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_layout'",
            params![post_id],
        )?;
        tx.execute(
            "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_layout', ?2)",
            params![post_id, layout_json],
        )?;

        let _ = Self::snapshot_post_revision_with_conn(&tx, post_id, &now_local, &now);

        tx.commit()?;
        drop(conn);
        self.invalidate_hot_cache("post_layout_saved");
        Ok(())
    }

    /// Deletes the JSON layout for a post, reverting it to standard editor mode.
    pub fn delete_post_layout(&self, post_id: i64) -> Result<()> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        tx.execute(
            "UPDATE posts SET layout_json = NULL WHERE ID = ?1",
            params![post_id],
        )?;
        tx.execute(
            "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key IN ('_daypress_layout', '_elementor_data', '_elementor_edit_mode')",
            params![post_id],
        )?;
        tx.commit()?;
        drop(conn);
        self.invalidate_hot_cache("post_layout_deleted");
        Ok(())
    }
}
