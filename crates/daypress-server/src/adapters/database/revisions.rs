//! Post revision snapshotting, restoration, and cleanup (`adapters/database/revisions.rs`).

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
    // POST REVISIONS & REVISION CLEANING ENGINE
    // =========================================================================

    pub(crate) fn snapshot_post_revision_with_conn(
        conn: &Connection,
        post_id: i64,
        now_local: &str,
        now_gmt: &str,
    ) -> Result<Option<i64>> {
        let row: Option<(i64, String, String, String, String, String)> = conn
            .query_row(
                "SELECT post_author, post_title, post_content, post_excerpt, post_type, COALESCE(layout_json, '') FROM posts WHERE ID = ?1",
                params![post_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)),
            )
            .ok();
        let Some((author, title, content, excerpt, ptype, layout_json)) = row else {
            return Ok(None);
        };
        if !matches!(ptype.as_str(), "post" | "page") {
            return Ok(None);
        }
        // Skip empty initial auto-drafts
        if title.trim().is_empty() && content.trim().is_empty() && layout_json.trim().is_empty() {
            return Ok(None);
        }

        // Deduplicate against the most recent revision for this post
        let last_rev: Option<(String, String, String, String)> = conn
            .query_row(
                "SELECT post_title, post_content, post_excerpt, COALESCE(layout_json, '') FROM posts WHERE post_parent = ?1 AND post_type = 'revision' ORDER BY ID DESC LIMIT 1",
                params![post_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .ok();
        if let Some((lt, lc, le, ll)) = last_rev {
            if lt == title && lc == content && le == excerpt && ll == layout_json {
                return Ok(None);
            }
        }

        let slug = format!("{}-revision-v1", post_id);
        let layout_opt = if layout_json.trim().is_empty() {
            None
        } else {
            Some(layout_json.as_str())
        };
        conn.execute(
            r#"INSERT INTO posts (post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, comment_status, ping_status, post_name, post_modified, post_modified_gmt, guid, post_type, post_parent, menu_order, layout_json)
               VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'inherit', 'closed', 'closed', ?7, ?2, ?3, '', 'revision', ?8, 0, ?9)"#,
            params![author, now_local, now_gmt, content, title, excerpt, slug, post_id, layout_opt],
        )?;
        let rev_id = conn.last_insert_rowid();
        let base_url = Self::get_base_url_with_conn(conn);
        let guid = format!("{}/?p={}", base_url.trim_end_matches('/'), rev_id);
        let _ = conn.execute(
            "UPDATE posts SET guid = ?1 WHERE ID = ?2",
            params![guid, rev_id],
        );
        if let Some(l) = layout_opt {
            let _ = conn.execute(
                "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_layout', ?2)",
                params![rev_id, l],
            );
        }
        Ok(Some(rev_id))
    }

    fn map_revision_row(
        conn: &Connection,
        row: &rusqlite::Row,
    ) -> Result<DaypressRevisionResponse> {
        let id: i64 = row.get(0)?;
        let author: i64 = row.get(1)?;
        let date: String = row.get(2)?;
        let date_gmt: String = row.get(3)?;
        let modified: String = row.get(4)?;
        let modified_gmt: String = row.get(5)?;
        let parent: i64 = row.get(6)?;
        let slug: String = row.get(7)?;
        let guid: String = row.get(8)?;
        let title: String = row.get(9)?;
        let content: String = row.get(10)?;
        let excerpt: String = row.get(11)?;
        let layout_json: String = row.get(12)?;

        let author_name: String = conn
            .query_row(
                "SELECT display_name FROM users WHERE ID = ?1",
                params![author],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "admin".to_string());

        Ok(DaypressRevisionResponse {
            id,
            author,
            author_name,
            date: date.replace(' ', "T"),
            date_gmt: date_gmt.replace(' ', "T"),
            modified: modified.replace(' ', "T"),
            modified_gmt: modified_gmt.replace(' ', "T"),
            parent,
            slug,
            guid: DaypressGuid { rendered: guid },
            title: DaypressRenderedText {
                rendered: title,
                protected: false,
            },
            content: DaypressRenderedText {
                rendered: content,
                protected: false,
            },
            excerpt: DaypressRenderedText {
                rendered: excerpt,
                protected: false,
            },
            has_layout: !layout_json.trim().is_empty(),
        })
    }

    pub fn get_post_revisions(&self, post_id: i64) -> Result<Vec<DaypressRevisionResponse>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT ID, post_author, post_date, post_date_gmt, post_modified, post_modified_gmt, COALESCE(post_parent, 0), post_name, guid, post_title, post_content, post_excerpt, COALESCE(layout_json, '')
             FROM posts WHERE post_parent = ?1 AND post_type = 'revision' ORDER BY ID DESC",
        )?;
        let rows = stmt.query_map(params![post_id], |r| Self::map_revision_row(&conn, r))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    pub fn get_revision_by_id(
        &self,
        post_id: i64,
        rev_id: i64,
    ) -> Result<Option<DaypressRevisionResponse>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT ID, post_author, post_date, post_date_gmt, post_modified, post_modified_gmt, COALESCE(post_parent, 0), post_name, guid, post_title, post_content, post_excerpt, COALESCE(layout_json, '')
             FROM posts WHERE ID = ?1 AND post_parent = ?2 AND post_type = 'revision'",
        )?;
        let mut rows = stmt.query_map(params![rev_id, post_id], |r| {
            Self::map_revision_row(&conn, r)
        })?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    pub fn restore_post_revision(
        &self,
        post_id: i64,
        rev_id: i64,
    ) -> Result<Option<DaypressPostResponse>> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let rev_row: Option<(String, String, String, Option<String>)> = tx
            .query_row(
                "SELECT post_title, post_content, post_excerpt, layout_json FROM posts WHERE ID = ?1 AND post_parent = ?2 AND post_type = 'revision'",
                params![rev_id, post_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .ok();
        let Some((title, content, excerpt, layout_json)) = rev_row else {
            return Ok(None);
        };

        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let site_offset_secs = super::scheduling::read_site_timezone_offset(&tx, None);
        let now_local = (Utc::now().naive_utc()
            + chrono::Duration::seconds(site_offset_secs as i64))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

        // Snapshot current state before restoring
        let _ = Self::snapshot_post_revision_with_conn(&tx, post_id, &now_local, &now);

        tx.execute(
            "UPDATE posts SET post_title = ?1, post_content = ?2, post_excerpt = ?3, layout_json = ?4, post_modified = ?5, post_modified_gmt = ?6 WHERE ID = ?7",
            params![title, content, excerpt, layout_json, now_local, now, post_id],
        )?;
        tx.execute(
            "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_layout'",
            params![post_id],
        )?;
        if let Some(ref l) = layout_json {
            if !l.trim().is_empty() {
                let _ = tx.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_layout', ?2)",
                    params![post_id, l],
                );
            }
        } else {
            let _ = super::post_ast::synchronize_post_ast_layout(&tx, post_id, &content, None);
        }

        // Snapshot restored state as latest revision
        let _ = Self::snapshot_post_revision_with_conn(&tx, post_id, &now_local, &now);

        tx.commit()?;
        drop(conn);
        self.get_post_by_id(post_id)
    }

    pub fn restore_revision_by_id(&self, rev_id: i64) -> Result<Option<DaypressPostResponse>> {
        let parent_id: Option<i64> = {
            let conn = self.conn.lock().unwrap();
            conn.query_row(
                "SELECT post_parent FROM posts WHERE ID = ?1 AND post_type = 'revision'",
                params![rev_id],
                |r| r.get(0),
            )
            .ok()
        };
        let Some(post_id) = parent_id.filter(|id| *id > 0) else {
            return Ok(None);
        };
        self.restore_post_revision(post_id, rev_id)
    }

    pub fn delete_post_revision(&self, post_id: i64, rev_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM posts WHERE ID = ?1 AND post_parent = ?2 AND post_type = 'revision'",
                params![rev_id, post_id],
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n > 0)
            .unwrap_or(false);
        if !exists {
            return Ok(false);
        }
        let _ = conn.execute("DELETE FROM postmeta WHERE post_id = ?1", params![rev_id]);
        let deleted = conn.execute(
            "DELETE FROM posts WHERE ID = ?1 AND post_parent = ?2 AND post_type = 'revision'",
            params![rev_id, post_id],
        )?;
        Ok(deleted > 0)
    }

    /// Cleans post revisions across the site, optionally keeping the N most recent revisions per post
    /// and/or only deleting revisions older than `older_than_days`.
    pub fn clean_post_revisions(
        &self,
        keep_per_post: Option<usize>,
        older_than_days: Option<i64>,
    ) -> Result<CleanRevisionsResponse> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let keep = keep_per_post.unwrap_or(0);
        let cutoff_gmt = older_than_days.filter(|d| *d > 0).map(|days| {
            (Utc::now().naive_utc() - chrono::Duration::days(days))
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        });

        let parent_ids: Vec<i64> = {
            let mut stmt = tx.prepare(
                "SELECT DISTINCT COALESCE(post_parent, 0) FROM posts WHERE post_type = 'revision'",
            )?;
            let rows = stmt.query_map([], |r| r.get(0))?;
            rows.filter_map(|r| r.ok()).collect()
        };

        let mut to_delete: Vec<i64> = Vec::new();
        for parent_id in parent_ids {
            let rev_rows: Vec<(i64, String)> = {
                let mut stmt = tx.prepare(
                    "SELECT ID, post_date_gmt FROM posts WHERE post_parent = ?1 AND post_type = 'revision' ORDER BY ID DESC",
                )?;
                let rows = stmt.query_map(params![parent_id], |r| Ok((r.get(0)?, r.get(1)?)))?;
                rows.filter_map(|r| r.ok()).collect()
            };
            for (idx, (rev_id, rev_date_gmt)) in rev_rows.into_iter().enumerate() {
                if idx < keep {
                    continue;
                }
                if let Some(ref cutoff) = cutoff_gmt {
                    if rev_date_gmt.replace('T', " ") > *cutoff {
                        continue;
                    }
                }
                to_delete.push(rev_id);
            }
        }

        let mut deleted_count = 0usize;
        for rev_id in to_delete {
            let _ = tx.execute("DELETE FROM postmeta WHERE post_id = ?1", params![rev_id]);
            deleted_count += tx.execute(
                "DELETE FROM posts WHERE ID = ?1 AND post_type = 'revision'",
                params![rev_id],
            )?;
        }

        let remaining_count: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM posts WHERE post_type = 'revision'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);

        tx.commit()?;
        Ok(CleanRevisionsResponse {
            deleted_count,
            remaining_count,
        })
    }
}
