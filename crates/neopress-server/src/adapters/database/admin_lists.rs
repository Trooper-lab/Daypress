//! Admin list-screen queries (`/n-admin/edit` and `/n-admin/edit-tags`).
//!
//! These power the WordPress-style list tables: server-side status tabs with real
//! per-status counts, search, taxonomy / month filters, column sorting and pagination.
//! Only the visible page of rows is hydrated, so large migrated sites stay fast.

use super::Database;
use crate::models::*;
use rusqlite::{params, Result};
use std::collections::HashMap;

/// Statuses that WordPress hides from the "All" tab of `/n-admin/edit`.
pub const ADMIN_HIDDEN_STATUSES: [&str; 3] = ["trash", "auto-draft", "inherit"];

#[derive(Debug, Default, Clone)]
pub struct AdminPostListQuery {
    pub post_type: String,
    /// `None` / `"all"` = every status except trash / auto-draft / inherit.
    pub status: Option<String>,
    pub search: Option<String>,
    /// Taxonomy filter (`category`, `post_tag`, or a custom taxonomy slug).
    pub taxonomy: Option<String>,
    pub term_id: Option<i64>,
    pub term_slug: Option<String>,
    /// `YYYYMM` month filter (WordPress `m` query var).
    pub month: Option<String>,
    /// `title` | `date` | `author` | `modified` | `id`
    pub orderby: Option<String>,
    /// `asc` | `desc`
    pub order: Option<String>,
    pub offset: usize,
    pub limit: usize,
    /// Default ordering: these IDs float to the top (e.g. the static front page).
    pub priority_ids: Vec<i64>,
    /// Default ordering: published items with these slugs come next (e.g. main-menu pages).
    pub priority_slugs: Vec<String>,
}

impl Database {
    /// Paged, filtered and sorted admin listing for a single post type.
    /// Returns the hydrated rows of the requested page plus the total matching row count.
    pub fn admin_list_posts(&self, q: &AdminPostListQuery) -> Result<(Vec<WpPostResponse>, usize)> {
        let conn = self.conn.lock().unwrap();
        let base_url = Self::get_base_url_with_conn(&conn);

        let mut where_sql: Vec<String> = vec!["p.post_type = ?".to_string()];
        let mut args: Vec<rusqlite::types::Value> = vec![q.post_type.clone().into()];

        match q
            .status
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty() && *s != "all")
        {
            Some(st) => {
                where_sql.push("p.post_status = ?".to_string());
                args.push(st.to_string().into());
            }
            None => {
                where_sql.push(format!(
                    "p.post_status NOT IN ({})",
                    ADMIN_HIDDEN_STATUSES
                        .iter()
                        .map(|s| format!("'{}'", s))
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        if let Some(s) = q.search.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
            where_sql.push(
                "(p.post_title LIKE ? OR p.post_content LIKE ? OR p.post_excerpt LIKE ? OR p.post_name LIKE ?)"
                    .to_string(),
            );
            let like = format!("%{}%", s);
            for _ in 0..4 {
                args.push(like.clone().into());
            }
        }

        if let Some(m) = q
            .month
            .as_deref()
            .map(str::trim)
            .filter(|m| m.len() == 6 && m.chars().all(|c| c.is_ascii_digit()))
        {
            where_sql
                .push("(substr(p.post_date, 1, 4) || substr(p.post_date, 6, 2)) = ?".to_string());
            args.push(m.to_string().into());
        }

        if let Some(tax) = q.taxonomy.as_deref().filter(|t| !t.is_empty()) {
            if let Some(tid) = q.term_id.filter(|id| *id > 0) {
                where_sql.push(
                    "p.ID IN (SELECT r.object_id FROM wp_term_relationships r JOIN wp_term_taxonomy tt ON r.term_taxonomy_id = tt.term_taxonomy_id WHERE tt.taxonomy = ? AND tt.term_id = ?)"
                        .to_string(),
                );
                args.push(tax.to_string().into());
                args.push(tid.into());
            } else if let Some(slug) = q.term_slug.as_deref().filter(|s| !s.is_empty()) {
                where_sql.push(
                    "p.ID IN (SELECT r.object_id FROM wp_term_relationships r JOIN wp_term_taxonomy tt ON r.term_taxonomy_id = tt.term_taxonomy_id JOIN wp_terms t ON t.term_id = tt.term_id WHERE tt.taxonomy = ? AND t.slug = ?)"
                        .to_string(),
                );
                args.push(tax.to_string().into());
                args.push(slug.to_string().into());
            }
        }

        let where_clause = format!("WHERE {}", where_sql.join(" AND "));

        let total: i64 = conn.query_row(
            &format!("SELECT COUNT(*) FROM wp_posts p {}", where_clause),
            rusqlite::params_from_iter(args.iter()),
            |r| r.get(0),
        )?;

        let is_asc = q
            .order
            .as_deref()
            .map(|o| o.eq_ignore_ascii_case("asc"))
            .unwrap_or(false);
        let dir = if is_asc { "ASC" } else { "DESC" };
        let mut order_args: Vec<rusqlite::types::Value> = Vec::new();
        let order_sql = match q.orderby.as_deref().map(|o| o.to_ascii_lowercase()).as_deref() {
            Some("title") => format!("LOWER(p.post_title) {dir}, p.ID {dir}"),
            Some("date") => format!("p.post_date {dir}, p.ID {dir}"),
            Some("modified") => format!("p.post_modified {dir}, p.ID {dir}"),
            Some("id") => format!("p.ID {dir}"),
            Some("author") => format!(
                "LOWER(COALESCE((SELECT u.display_name FROM wp_users u WHERE u.ID = p.post_author), '')) {dir}, p.post_date DESC"
            ),
            _ => {
                // Default editorial ordering: front page, main-menu pages, published, titled, newest.
                let mut parts: Vec<String> = Vec::new();
                if !q.priority_ids.is_empty() {
                    parts.push(format!(
                        "(p.ID IN ({})) DESC",
                        vec!["?"; q.priority_ids.len()].join(", ")
                    ));
                    for id in &q.priority_ids {
                        order_args.push((*id).into());
                    }
                }
                if !q.priority_slugs.is_empty() {
                    parts.push(format!(
                        "(p.post_status = 'publish' AND LOWER(p.post_name) IN ({})) DESC",
                        vec!["?"; q.priority_slugs.len()].join(", ")
                    ));
                    for s in &q.priority_slugs {
                        order_args.push(s.to_lowercase().into());
                    }
                }
                parts.push("(p.post_status = 'publish') DESC".to_string());
                parts.push("(TRIM(p.post_title) <> '') DESC".to_string());
                parts.push("p.post_date DESC".to_string());
                parts.push("p.ID DESC".to_string());
                parts.join(", ")
            }
        };

        let sql = format!(
            "SELECT p.ID, p.post_date, p.post_date_gmt, p.post_modified, p.post_modified_gmt, p.post_title, p.post_content, p.post_excerpt, p.post_name, p.post_status, p.post_type, p.guid, p.post_author, p.comment_status, p.ping_status, COALESCE(p.post_parent, 0), COALESCE(p.menu_order, 0), COALESCE(p.is_sticky, 0) FROM wp_posts p {} ORDER BY {} LIMIT ? OFFSET ?",
            where_clause, order_sql
        );
        let mut all_args = args;
        all_args.extend(order_args);
        all_args.push((q.limit.max(1) as i64).into());
        all_args.push((q.offset as i64).into());

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(
            rusqlite::params_from_iter(all_args.iter()),
            Self::map_post_row,
        )?;
        let mut posts = Vec::new();
        for r in rows {
            let mut post = r?;
            Self::hydrate_post_relations(&conn, &mut post, &base_url);
            posts.push(post);
        }
        Ok((posts, total.max(0) as usize))
    }

    /// Per-status row counts for one post type (drives the status tabs of `/n-admin/edit`).
    pub fn admin_post_status_counts(&self, post_type: &str) -> Result<HashMap<String, i64>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT post_status, COUNT(*) FROM wp_posts WHERE post_type = ?1 GROUP BY post_status",
        )?;
        let rows = stmt.query_map(params![post_type], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
        })?;
        let mut map = HashMap::new();
        for r in rows {
            let (k, v) = r?;
            map.insert(k, v);
        }
        Ok(map)
    }

    /// Distinct `YYYY-MM` months that contain non-trashed rows of a post type (month filter dropdown).
    pub fn admin_post_months(&self, post_type: &str) -> Result<Vec<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT DISTINCT substr(post_date, 1, 7) AS ym FROM wp_posts
             WHERE post_type = ?1 AND post_status NOT IN ('trash', 'auto-draft', 'inherit') AND length(post_date) >= 7
             ORDER BY ym DESC",
        )?;
        let rows = stmt.query_map(params![post_type], |r| r.get::<_, String>(0))?;
        Ok(rows.filter_map(|r| r.ok()).collect())
    }

    /// Returns the `term_id` currently owning `slug` in `wp_terms` (slugs are globally unique here).
    pub fn term_id_for_slug(&self, slug: &str) -> Result<Option<i64>> {
        let conn = self.conn.lock().unwrap();
        Ok(conn
            .query_row(
                "SELECT term_id FROM wp_terms WHERE slug = ?1",
                params![slug],
                |r| r.get::<_, i64>(0),
            )
            .ok())
    }

    /// Finds a term in `taxonomy` whose name matches case-insensitively (WordPress `term_exists`).
    pub fn term_id_for_name(&self, taxonomy: &str, name: &str, parent: i64) -> Result<Option<i64>> {
        let conn = self.conn.lock().unwrap();
        Ok(conn
            .query_row(
                "SELECT t.term_id FROM wp_terms t JOIN wp_term_taxonomy tt ON t.term_id = tt.term_id
                 WHERE tt.taxonomy = ?1 AND LOWER(t.name) = LOWER(?2) AND tt.parent = ?3",
                params![taxonomy, name.trim(), parent],
                |r| r.get::<_, i64>(0),
            )
            .ok())
    }
}
