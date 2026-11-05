//! Editorial Workflows (`v0.8`):
//! - 1-Click Post/Page Duplication (`duplicate_post`)
//! - Multi-Author Bylines & Co-Authors Plus Migration (`daypress_post_authors`)
//! - Shareable Secret Draft Preview Links (`daypress_preview_tokens`)
//! - Scheduled Revisions to Already-Published Pages (`post_status = 'future_revision'`)
//! - Visual Side-by-Side AST Revision Diffs & Per-Node Cherry-Pick Restore

use super::Database;
use crate::models::{
    DaypressCoAuthor, DaypressPostResponse, DaypressPreviewTokenRecord, ScheduleRevisionRequest,
    ScheduledRevisionSummary,
};
use chrono::Utc;
use rusqlite::{params, Connection, OptionalExtension, Result};
use sha2::{Digest, Sha256};

fn sha256_hex(input: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    format!("{:x}", hasher.finalize())
}

impl Database {
    // =========================================================================
    // 1. MULTI-AUTHOR BYLINES (`Co-Authors Plus` PARITY)
    // =========================================================================

    pub(crate) fn set_post_coauthors_with_conn(
        conn: &Connection,
        post_id: i64,
        user_ids: &[i64],
    ) -> Result<()> {
        conn.execute(
            "DELETE FROM daypress_post_authors WHERE post_id = ?1",
            params![post_id],
        )?;
        let mut seen = std::collections::HashSet::new();
        for (idx, &uid) in user_ids.iter().enumerate() {
            if uid > 0 && seen.insert(uid) {
                conn.execute(
                    "INSERT INTO daypress_post_authors (post_id, user_id, author_order) VALUES (?1, ?2, ?3)",
                    params![post_id, uid, idx as i64],
                )?;
            }
        }
        Ok(())
    }

    pub fn set_post_coauthors(&self, post_id: i64, user_ids: &[i64]) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        Self::set_post_coauthors_with_conn(&conn, post_id, user_ids)?;
        drop(conn);
        self.invalidate_hot_cache("set_post_coauthors");
        Ok(())
    }

    pub(crate) fn get_post_coauthors_with_conn(
        conn: &Connection,
        post_id: i64,
        primary_author_id: i64,
    ) -> (Vec<DaypressCoAuthor>, String) {
        let mut coauthors: Vec<DaypressCoAuthor> = conn
            .prepare_cached(
                r#"SELECT u.ID, u.display_name, u.user_nicename, u.user_login
                   FROM daypress_post_authors pa
                   JOIN users u ON u.ID = pa.user_id
                   WHERE pa.post_id = ?1
                   ORDER BY pa.author_order ASC"#,
            )
            .and_then(|mut stmt| {
                let rows = stmt.query_map(params![post_id], |r| {
                    let id: i64 = r.get(0)?;
                    let display: String = r.get(1)?;
                    let nicename: String = r.get(2)?;
                    let login: String = r.get(3)?;
                    Ok(DaypressCoAuthor {
                        id,
                        display_name: if display.trim().is_empty() {
                            login.clone()
                        } else {
                            display
                        },
                        slug: if nicename.trim().is_empty() {
                            login
                        } else {
                            nicename
                        },
                    })
                })?;
                Ok(rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        // Ensure primary author is always present first
        if !coauthors.iter().any(|ca| ca.id == primary_author_id) {
            let primary = conn
                .query_row(
                    "SELECT ID, display_name, user_nicename, user_login FROM users WHERE ID = ?1",
                    params![primary_author_id],
                    |r| {
                        let id: i64 = r.get(0)?;
                        let display: String = r.get(1)?;
                        let nicename: String = r.get(2)?;
                        let login: String = r.get(3)?;
                        Ok(DaypressCoAuthor {
                            id,
                            display_name: if display.trim().is_empty() {
                                login.clone()
                            } else {
                                display
                            },
                            slug: if nicename.trim().is_empty() {
                                login
                            } else {
                                nicename
                            },
                        })
                    },
                )
                .unwrap_or(DaypressCoAuthor {
                    id: primary_author_id,
                    display_name: "admin".to_string(),
                    slug: "admin".to_string(),
                });
            coauthors.insert(0, primary);
        }

        let formatted_byline = match coauthors.len() {
            0 => "admin".to_string(),
            1 => coauthors[0].display_name.clone(),
            2 => format!(
                "{} & {}",
                coauthors[0].display_name, coauthors[1].display_name
            ),
            n => {
                let head: Vec<&str> = coauthors[..n - 1]
                    .iter()
                    .map(|c| c.display_name.as_str())
                    .collect();
                format!("{} & {}", head.join(", "), coauthors[n - 1].display_name)
            }
        };

        (coauthors, formatted_byline)
    }

    /// Migrates WordPress `Co-Authors Plus` (`author` taxonomy terms / `cap-*` slugs / `_coauthor` postmeta)
    /// into `daypress_post_authors` during 1-click JSON or WXR imports.
    pub(crate) fn import_coauthors_with_conn(conn: &Connection) {
        // 1. `_coauthor` or `_daypress_coauthors` postmeta rows
        let meta_rows: Vec<(i64, String)> = conn
            .prepare(
                "SELECT post_id, COALESCE(meta_value, '') FROM postmeta WHERE meta_key IN ('_coauthor', 'coauthor', '_daypress_coauthors')",
            )
            .and_then(|mut stmt| {
                let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
                Ok(rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        let mut order_by_post: std::collections::HashMap<i64, i64> =
            std::collections::HashMap::new();
        for (pid, raw_val) in meta_rows {
            for token in raw_val.split(',') {
                let clean = token.trim().trim_start_matches("cap-");
                if clean.is_empty() {
                    continue;
                }
                let uid: Option<i64> = if let Ok(id) = clean.parse::<i64>() {
                    Some(id)
                } else {
                    conn.query_row(
                        "SELECT ID FROM users WHERE LOWER(user_login) = LOWER(?1) OR LOWER(user_nicename) = LOWER(?1) OR LOWER(display_name) = LOWER(?1) LIMIT 1",
                        params![clean],
                        |r| r.get(0),
                    )
                    .ok()
                };
                if let Some(user_id) = uid {
                    let ord = order_by_post.entry(pid).or_insert(0);
                    let _ = conn.execute(
                        "INSERT OR IGNORE INTO daypress_post_authors (post_id, user_id, author_order) VALUES (?1, ?2, ?3)",
                        params![pid, user_id, *ord],
                    );
                    *ord += 1;
                }
            }
        }

        // 2. Co-Authors Plus `author` taxonomy terms
        let tax_rows: Vec<(i64, String, String)> = conn
            .prepare(
                r#"SELECT r.object_id, t.slug, t.name
                   FROM term_relationships r
                   JOIN term_taxonomy tt ON r.term_taxonomy_id = tt.term_taxonomy_id
                   JOIN terms t ON tt.term_id = t.term_id
                   WHERE tt.taxonomy = 'author'
                   ORDER BY r.term_order ASC"#,
            )
            .and_then(|mut stmt| {
                let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
                Ok(rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        for (pid, slug, name) in tax_rows {
            let clean_slug = slug.trim_start_matches("cap-");
            let uid: Option<i64> = conn
                .query_row(
                    "SELECT ID FROM users WHERE LOWER(user_nicename) = LOWER(?1) OR LOWER(user_login) = LOWER(?1) OR LOWER(display_name) = LOWER(?2) LIMIT 1",
                    params![clean_slug, name],
                    |r| r.get(0),
                )
                .ok();
            if let Some(user_id) = uid {
                let ord = order_by_post.entry(pid).or_insert(0);
                let _ = conn.execute(
                    "INSERT OR IGNORE INTO daypress_post_authors (post_id, user_id, author_order) VALUES (?1, ?2, ?3)",
                    params![pid, user_id, *ord],
                );
                *ord += 1;
            }
        }
    }

    // =========================================================================
    // 3. SHAREABLE SECRET DRAFT PREVIEW LINKS (NO LOGIN REQUIRED)
    // =========================================================================

    pub fn create_preview_token(
        &self,
        post_id: i64,
        ttl_hours: i64,
        label: Option<&str>,
        created_by: i64,
    ) -> Result<DaypressPreviewTokenRecord> {
        let raw_token = format!("dp_prev_{}", crate::auth::random_hex_64());
        let token_hash = sha256_hex(&raw_token);
        let token_preview: String = raw_token.chars().take(16).collect();
        let now_ts = Utc::now().timestamp();
        let hours = ttl_hours.clamp(1, 24 * 365);
        let expires_at = now_ts + hours * 3600;
        let clean_label = label.unwrap_or("Reviewer Link").trim().to_string();

        let conn = self.conn.lock().unwrap();
        let base_url = Self::get_base_url_with_conn(&conn);
        conn.execute(
            r#"INSERT INTO daypress_preview_tokens (
                   post_id, token_hash, token_preview, label, created_by, created_at, expires_at
               ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
            params![
                post_id,
                token_hash,
                token_preview,
                clean_label,
                created_by,
                now_ts,
                expires_at
            ],
        )?;
        let id = conn.last_insert_rowid();
        let preview_url = format!(
            "{}/?p={}&preview_token={}",
            base_url.trim_end_matches('/'),
            post_id,
            raw_token
        );

        Ok(DaypressPreviewTokenRecord {
            id,
            post_id,
            token_preview,
            label: clean_label,
            created_by,
            created_at: now_ts,
            expires_at,
            revoked_at: None,
            raw_token: Some(raw_token),
            preview_url,
        })
    }

    pub fn list_preview_tokens(&self, post_id: i64) -> Result<Vec<DaypressPreviewTokenRecord>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT id, post_id, token_preview, label, created_by, created_at, expires_at, revoked_at
               FROM daypress_preview_tokens
               WHERE post_id = ?1
               ORDER BY id DESC"#,
        )?;
        let rows = stmt.query_map(params![post_id], |r| {
            Ok(DaypressPreviewTokenRecord {
                id: r.get(0)?,
                post_id: r.get(1)?,
                token_preview: r.get(2)?,
                label: r.get(3)?,
                created_by: r.get(4)?,
                created_at: r.get(5)?,
                expires_at: r.get(6)?,
                revoked_at: r.get(7)?,
                raw_token: None,
                preview_url: String::new(),
            })
        })?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    pub fn revoke_preview_token(&self, post_id: i64, token_id: i64) -> Result<bool> {
        let now_ts = Utc::now().timestamp();
        let conn = self.conn.lock().unwrap();
        let updated = conn.execute(
            "UPDATE daypress_preview_tokens SET revoked_at = ?1 WHERE id = ?2 AND post_id = ?3 AND revoked_at IS NULL",
            params![now_ts, token_id, post_id],
        )?;
        Ok(updated > 0)
    }

    /// Verifies a raw secret preview token. If valid, unrevoked, and unexpired, returns
    /// `Some((post_id, expires_at))`. If `expected_post_id` is `Some(pid)`, also verifies that
    /// the token belongs to `pid`.
    pub fn verify_preview_token(
        &self,
        raw_token: &str,
        expected_post_id: Option<i64>,
    ) -> Option<(i64, i64)> {
        let trimmed = raw_token.trim();
        if trimmed.is_empty() {
            return None;
        }
        let token_hash = sha256_hex(trimmed);
        let now_ts = Utc::now().timestamp();
        let conn = self.conn.lock().ok()?;
        let row: Option<(i64, i64)> = conn
            .query_row(
                r#"SELECT post_id, expires_at
                   FROM daypress_preview_tokens
                   WHERE token_hash = ?1 AND revoked_at IS NULL AND expires_at > ?2
                   LIMIT 1"#,
                params![token_hash, now_ts],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        let (post_id, expires_at) = row?;
        if let Some(exp) = expected_post_id {
            if post_id != exp {
                return None;
            }
        }
        Some((post_id, expires_at))
    }

    // =========================================================================
    // 4. SCHEDULED REVISIONS TO ALREADY-PUBLISHED PAGES
    // =========================================================================

    pub fn schedule_published_post_revision(
        &self,
        parent_post_id: i64,
        req: &ScheduleRevisionRequest,
        author_id: Option<i64>,
    ) -> Result<ScheduledRevisionSummary> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let parent_row: Option<(i64, String, String, String, Option<String>)> = tx
            .query_row(
                "SELECT post_author, post_title, post_content, post_excerpt, layout_json FROM posts WHERE ID = ?1",
                params![parent_post_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )
            .optional()?;
        let Some((orig_author, cur_title, cur_content, cur_excerpt, cur_layout)) = parent_row
        else {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        };

        let author = author_id.unwrap_or(orig_author);
        let title = req.title.clone().unwrap_or(cur_title);
        let mut content = req.content.clone().unwrap_or(cur_content);
        let excerpt = req.excerpt.clone().unwrap_or(cur_excerpt);

        let layout_str: Option<String> = match &req.layout {
            Some(v) if !v.is_null() => {
                if let Ok(parsed_ast) =
                    serde_json::from_value::<daypress_ast::DaypressLayout>(v.clone())
                {
                    content = daypress_ast::render_layout_to_html(&parsed_ast);
                }
                serde_json::to_string(v).ok()
            }
            _ => cur_layout,
        };

        let now_gmt = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let (sched_local, sched_gmt, _) = super::scheduling::schedule_for_new_post_in_site_tz(
            &tx,
            "future",
            "post",
            req.date.as_deref(),
            req.date_gmt.as_deref(),
            &now_gmt,
        );

        // Replace any existing pending `future_revision` for this parent post
        let existing_rev_id: Option<i64> = tx
            .query_row(
                "SELECT ID FROM posts WHERE post_parent = ?1 AND post_status = 'future_revision' ORDER BY ID DESC LIMIT 1",
                params![parent_post_id],
                |r| r.get(0),
            )
            .optional()?;

        let rev_post_id = if let Some(eid) = existing_rev_id {
            tx.execute(
                r#"UPDATE posts
                   SET post_author = ?1, post_date = ?2, post_date_gmt = ?3, post_modified = ?2, post_modified_gmt = ?3,
                       post_title = ?4, post_content = ?5, post_excerpt = ?6, layout_json = ?7
                   WHERE ID = ?8"#,
                params![
                    author,
                    sched_local,
                    sched_gmt,
                    title,
                    content,
                    excerpt,
                    layout_str,
                    eid
                ],
            )?;
            eid
        } else {
            let slug = format!("{}-scheduled-revision", parent_post_id);
            tx.execute(
                r#"INSERT INTO posts (
                       post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt,
                       post_status, comment_status, ping_status, post_name, post_modified,
                       post_modified_gmt, guid, post_parent, menu_order, post_type, layout_json
                   ) VALUES (
                       ?1, ?2, ?3, ?4, ?5, ?6,
                       'future_revision', 'closed', 'closed', ?7, ?2,
                       ?3, '', ?8, 0, 'revision', ?9
                   )"#,
                params![
                    author,
                    sched_local,
                    sched_gmt,
                    content,
                    title,
                    excerpt,
                    slug,
                    parent_post_id,
                    layout_str
                ],
            )?;
            tx.last_insert_rowid()
        };

        tx.execute(
            "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_layout'",
            params![rev_post_id],
        )?;
        if let Some(ref l) = layout_str {
            if !l.trim().is_empty() {
                let _ = tx.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_layout', ?2)",
                    params![rev_post_id, l],
                );
            }
        }

        tx.commit()?;

        Ok(ScheduledRevisionSummary {
            revision_post_id: rev_post_id,
            parent_post_id,
            title,
            scheduled_date: sched_local.replace(' ', "T"),
            scheduled_date_gmt: sched_gmt.replace(' ', "T"),
            has_layout: layout_str.is_some_and(|s| !s.trim().is_empty()),
        })
    }

    pub(crate) fn get_scheduled_revision_for_post_with_conn(
        conn: &Connection,
        parent_post_id: i64,
    ) -> Option<ScheduledRevisionSummary> {
        conn.query_row(
            r#"SELECT ID, post_parent, post_title, post_date, post_date_gmt, COALESCE(layout_json, '')
               FROM posts
               WHERE post_parent = ?1 AND post_status = 'future_revision'
               ORDER BY ID DESC LIMIT 1"#,
            params![parent_post_id],
            |r| {
                let rev_id: i64 = r.get(0)?;
                let parent_id: i64 = r.get(1)?;
                let title: String = r.get(2)?;
                let date: String = r.get(3)?;
                let date_gmt: String = r.get(4)?;
                let layout: String = r.get(5)?;
                Ok(ScheduledRevisionSummary {
                    revision_post_id: rev_id,
                    parent_post_id: parent_id,
                    title,
                    scheduled_date: date.replace(' ', "T"),
                    scheduled_date_gmt: date_gmt.replace(' ', "T"),
                    has_layout: !layout.trim().is_empty(),
                })
            },
        )
        .ok()
    }

    pub fn get_scheduled_revision_for_post(
        &self,
        parent_post_id: i64,
    ) -> Result<Option<ScheduledRevisionSummary>> {
        let conn = self.conn.lock().unwrap();
        Ok(Self::get_scheduled_revision_for_post_with_conn(
            &conn,
            parent_post_id,
        ))
    }

    pub fn cancel_scheduled_revision(&self, parent_post_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rev_ids: Vec<i64> = {
            let mut stmt = conn.prepare(
                "SELECT ID FROM posts WHERE post_parent = ?1 AND post_status = 'future_revision'",
            )?;
            let rows = stmt.query_map(params![parent_post_id], |r| r.get(0))?;
            rows.filter_map(|r| r.ok()).collect()
        };
        if rev_ids.is_empty() {
            return Ok(false);
        }
        for rid in &rev_ids {
            let _ = conn.execute("DELETE FROM postmeta WHERE post_id = ?1", params![rid]);
            let _ = conn.execute("DELETE FROM posts WHERE ID = ?1", params![rid]);
        }
        Ok(true)
    }

    /// Promotes any due `future_revision` rows whose `post_date_gmt <= now_gmt` into their live
    /// parent post atomically, snapshotting the live post first and preserving the live post's ID & URL.
    pub(crate) fn promote_due_scheduled_revisions_with_conn(
        conn: &Connection,
        now_gmt: &str,
    ) -> Result<usize> {
        let due_rows: Vec<(i64, i64, String, String, String, Option<String>, String)> = {
            let mut stmt = conn.prepare_cached(
                r#"SELECT ID, post_parent, post_title, post_content, post_excerpt, layout_json, post_date
                   FROM posts
                   WHERE post_status = 'future_revision' AND post_date_gmt <= ?1
                   ORDER BY post_date_gmt ASC, ID ASC"#,
            )?;
            let rows = stmt.query_map(params![now_gmt], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                ))
            })?;
            rows.filter_map(|r| r.ok()).collect()
        };

        if due_rows.is_empty() {
            return Ok(0);
        }

        let mut promoted = 0usize;
        for (rev_id, parent_id, new_title, new_content, new_excerpt, new_layout, sched_local) in
            due_rows
        {
            if parent_id <= 0 {
                continue;
            }
            // 1. Snapshot the current live post state before applying the scheduled revision
            let _ = Self::snapshot_post_revision_with_conn(conn, parent_id, &sched_local, now_gmt);

            // 2. Apply the scheduled revision's content and layout into the live parent post
            conn.execute(
                r#"UPDATE posts
                   SET post_title = ?1, post_content = ?2, post_excerpt = ?3, layout_json = ?4,
                       post_modified = ?5, post_modified_gmt = ?6
                   WHERE ID = ?7"#,
                params![
                    new_title,
                    new_content,
                    new_excerpt,
                    new_layout,
                    sched_local,
                    now_gmt,
                    parent_id
                ],
            )?;

            conn.execute(
                "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_layout'",
                params![parent_id],
            )?;
            if let Some(ref l) = new_layout {
                if !l.trim().is_empty() {
                    let _ = conn.execute(
                        "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_layout', ?2)",
                        params![parent_id, l],
                    );
                }
            }

            // 3. Convert the applied `future_revision` into a standard historical `'inherit'` revision
            let hist_slug = format!("{}-revision-v1", parent_id);
            conn.execute(
                "UPDATE posts SET post_status = 'inherit', post_type = 'revision', post_name = ?1 WHERE ID = ?2",
                params![hist_slug, rev_id],
            )?;

            promoted += 1;
        }

        Ok(promoted)
    }

    // =========================================================================
    // 5. VISUAL SIDE-BY-SIDE AST REVISION DIFFS & CHERRY-PICK RESTORE
    // =========================================================================

    fn load_post_or_revision_ast_with_conn(
        conn: &Connection,
        row_id: i64,
    ) -> Result<daypress_ast::DaypressLayout> {
        let (content, layout_json): (String, String) = conn.query_row(
            "SELECT post_content, COALESCE(layout_json, '') FROM posts WHERE ID = ?1",
            params![row_id],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        if !layout_json.trim().is_empty() {
            if let Ok(ast) = serde_json::from_str::<daypress_ast::DaypressLayout>(&layout_json) {
                return Ok(ast);
            }
        }
        Ok(daypress_ast::transpile_html_to_layout(&content))
    }

    /// Computes a structural `AstLayoutDiffReport` between `base_rev_id` and `target_rev_id`
    /// (when `target_rev_id` is `None` or `0` or equals `post_id`, compares against the live `post_id`).
    pub fn diff_post_revisions(
        &self,
        post_id: i64,
        base_rev_id: i64,
        target_rev_id: Option<i64>,
    ) -> Result<daypress_ast::AstLayoutDiffReport> {
        let conn = self.conn.lock().unwrap();
        let base_ast = Self::load_post_or_revision_ast_with_conn(&conn, base_rev_id)?;
        let target_id = target_rev_id.filter(|&id| id > 0).unwrap_or(post_id);
        let target_ast = Self::load_post_or_revision_ast_with_conn(&conn, target_id)?;
        Ok(daypress_ast::diff_layouts(&base_ast, &target_ast))
    }

    /// Restores a single node (`node_id`) from `rev_id` into `post_id`, snapshotting `post_id` first,
    /// recompiling SSR HTML, and invalidating the hot cache.
    pub fn cherry_pick_revision_node(
        &self,
        post_id: i64,
        rev_id: i64,
        node_id: &str,
    ) -> Result<DaypressPostResponse> {
        {
            let mut conn = self.conn.lock().unwrap();
            let tx = conn.transaction()?;
            let current_ast = Self::load_post_or_revision_ast_with_conn(&tx, post_id)?;
            let rev_ast = Self::load_post_or_revision_ast_with_conn(&tx, rev_id)?;
            let patched_ast =
                daypress_ast::cherry_pick_node_from_revision(&current_ast, &rev_ast, node_id)
                    .map_err(|msg| {
                        rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            msg,
                        )))
                    })?;

            let now_gmt = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let offset_secs = super::scheduling::read_site_timezone_offset(&tx, None);
            let now_local = (Utc::now().naive_utc()
                + chrono::Duration::seconds(offset_secs as i64))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();

            let _ = Self::snapshot_post_revision_with_conn(&tx, post_id, &now_local, &now_gmt);

            let new_layout_json = serde_json::to_string(&patched_ast).unwrap_or_default();
            let rendered_html = daypress_ast::render_layout_to_html(&patched_ast);

            tx.execute(
                "UPDATE posts SET layout_json = ?1, post_content = ?2, post_modified = ?3, post_modified_gmt = ?4 WHERE ID = ?5",
                params![new_layout_json, rendered_html, now_local, now_gmt, post_id],
            )?;
            tx.execute(
                "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_layout'",
                params![post_id],
            )?;
            tx.execute(
                "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_layout', ?2)",
                params![post_id, new_layout_json],
            )?;

            let _ = Self::snapshot_post_revision_with_conn(&tx, post_id, &now_local, &now_gmt);
            tx.commit()?;
        }
        self.invalidate_hot_cache("cherry_pick_revision_node");
        self.get_post_by_id(post_id)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)
    }
}
