//! Post creation, update, duplication, trash, and deletion (`adapters/database/posts_write.rs`).

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
    /// Slugs that would be shadowed by a built-in route, so content can never claim them.
    const RESERVED_SLUGS: [&'static str; 17] = [
        "admin",
        "author",
        "category",
        "feed",
        "login",
        "logout",
        "d-admin",
        "page",
        "robots.txt",
        "search",
        "sitemap.xml",
        "static",
        "studio",
        "tag",
        "uploads",
        "wp-admin",
        "wp-json",
    ];

    /// Returns `desired` (or `untitled`) made unique across all routable content, appending
    /// `-2`, `-3`, … like WordPress, since every post type shares the `/{slug}/` URL space.
    pub(crate) fn unique_post_slug(
        conn: &Connection,
        desired: &str,
        exclude_id: Option<i64>,
    ) -> Result<String> {
        let base = if desired.trim().is_empty() {
            "untitled".to_string()
        } else {
            desired.trim().to_string()
        };
        let taken = |candidate: &str| -> Result<bool> {
            if Self::RESERVED_SLUGS.contains(&candidate) {
                return Ok(true);
            }
            conn.query_row(
                "SELECT COUNT(*) FROM posts WHERE post_name = ?1 AND ID <> ?2
                   AND post_type NOT IN ('nav_menu_item', 'revision', 'elementor_library')
                   AND post_status <> 'trash'",
                params![candidate, exclude_id.unwrap_or(-1)],
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n > 0)
        };
        if !taken(&base)? {
            return Ok(base);
        }
        let mut n = 2;
        loop {
            let candidate = format!("{}-{}", base, n);
            if !taken(&candidate)? {
                return Ok(candidate);
            }
            n += 1;
        }
    }

    pub fn create_post(&self, req: &CreatePostRequest) -> Result<DaypressPostResponse> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let desired = match &req.slug {
            Some(s) if !s.trim().is_empty() => generated_slugify::slugify(s.trim()),
            _ => generated_slugify::slugify(&req.title),
        };
        let slug = Self::unique_post_slug(&conn, &desired, None)?;

        let excerpt = req.excerpt.clone().unwrap_or_else(|| {
            if req.content.chars().count() > 140 {
                let prefix: String = req.content.chars().take(140).collect();
                format!("{}...", prefix)
            } else {
                req.content.clone()
            }
        });
        let parent = req.parent.unwrap_or(0);
        let menu_order = req.menu_order.unwrap_or(0);
        let author = req.author.unwrap_or(1);
        // `publish` dated ahead becomes `future` (scheduled), like WordPress, using site timezone.
        let (post_date_local, post_date_gmt, status) =
            super::scheduling::schedule_for_new_post_in_site_tz(
                &conn,
                &req.status,
                &req.post_type,
                req.date.as_deref(),
                req.date_gmt.as_deref(),
                &now,
            );
        let site_offset_secs = super::scheduling::read_site_timezone_offset(&conn, None);
        let now_local = (Utc::now().naive_utc()
            + chrono::Duration::seconds(site_offset_secs as i64))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

        conn.execute(
            r#"INSERT INTO posts (post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type, post_parent, menu_order)
               VALUES (?1, ?11, ?12, ?3, ?4, ?5, ?6, ?7, ?13, ?2, '', ?8, ?9, ?10)"#,
            params![author, now, req.content, req.title, excerpt, status, slug, req.post_type, parent, menu_order, post_date_local, post_date_gmt, now_local],
        )?;

        let id = conn.last_insert_rowid();
        let base_url = Self::get_base_url_with_conn(&conn);
        let guid = format!("{}/?p={}", base_url.trim_end_matches('/'), id);
        conn.execute(
            "UPDATE posts SET guid = ?1 WHERE ID = ?2",
            params![guid, id],
        )?;

        // Settings → Discussion "Allow people to submit comments on new posts" and
        // Settings → Writing "Default Post Category" apply to new blog posts.
        let read_option = |name: &str| -> Option<String> {
            conn.query_row(
                "SELECT option_value FROM options WHERE option_name = ?1",
                params![name],
                |r| r.get(0),
            )
            .ok()
        };
        if req.post_type == "post"
            && read_option("default_comment_status").as_deref() == Some("closed")
        {
            conn.execute(
                "UPDATE posts SET comment_status = 'closed' WHERE ID = ?1",
                params![id],
            )?;
        }

        if let Some(fm) = req.featured_media {
            if fm > 0 {
                let _ = conn.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_thumbnail_id', ?2)",
                    params![id, fm.to_string()],
                );
            }
        }

        if let Some(ref tpl) = req.template {
            if !tpl.is_empty() {
                let _ = conn.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_page_template', ?2)",
                    params![id, tpl],
                );
            }
        }

        let mut all_taxonomies: std::collections::HashMap<String, Vec<i64>> =
            req.taxonomies.clone().unwrap_or_default();
        if !req.categories.is_empty() && !all_taxonomies.contains_key("category") {
            all_taxonomies.insert("category".to_string(), req.categories.clone());
        }
        if req.post_type == "post"
            && all_taxonomies
                .get("category")
                .is_none_or(|ids| ids.is_empty())
        {
            let default_category = read_option("default_category")
                .and_then(|v| v.trim().parse::<i64>().ok())
                .filter(|id| *id > 0)
                .unwrap_or(1);
            all_taxonomies.insert("category".to_string(), vec![default_category]);
        }
        if !req.tags.is_empty() && !all_taxonomies.contains_key("post_tag") {
            all_taxonomies.insert("post_tag".to_string(), req.tags.clone());
        }

        for (tax_name, term_ids) in all_taxonomies {
            for term_id in term_ids {
                let tt_id: Option<i64> = conn.query_row(
                    "SELECT term_taxonomy_id FROM term_taxonomy WHERE term_id = ?1 AND taxonomy = ?2",
                    params![term_id, &tax_name],
                    |r| r.get(0)
                ).ok();
                if let Some(tid) = tt_id {
                    let _ = conn.execute("INSERT OR IGNORE INTO term_relationships (object_id, term_taxonomy_id) VALUES (?1, ?2)", params![id, tid]);
                }
            }
        }

        // Persist custom / ACF postmeta fields when provided in `req.meta`
        if let Some(serde_json::Value::Object(ref meta_map)) = req.meta {
            for (mk, mv) in meta_map {
                let k = mk.trim();
                if k.is_empty() || k.starts_with("_tax_") || k == "_daypress_layout" || mv.is_null()
                {
                    continue;
                }
                let val_str = match mv {
                    serde_json::Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                let _ = conn.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, ?2, ?3)",
                    params![id, k, val_str],
                );
            }
        }

        // Persist SEO & OpenGraph metadata if provided
        let seo_pairs = [
            ("_daypress_seo_title", req.seo_title.as_deref()),
            ("_daypress_seo_desc", req.seo_description.as_deref()),
            ("_daypress_seo_keyword", req.seo_keyword.as_deref()),
            ("_daypress_seo_canonical", req.seo_canonical.as_deref()),
            ("_daypress_seo_schema_type", req.seo_schema_type.as_deref()),
            ("_daypress_seo_og_image", req.seo_og_image.as_deref()),
        ];
        for (meta_key, val_opt) in seo_pairs {
            if let Some(val) = val_opt.map(str::trim).filter(|s| !s.is_empty()) {
                let _ = conn.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, ?2, ?3)",
                    params![id, meta_key, val],
                );
            }
        }
        if let Some(noindex) = req.seo_noindex {
            if noindex {
                let _ = conn.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_seo_noindex', '1')",
                    params![id],
                );
            }
        }

        // Synchronize AST layout from post content (Gutenberg, Elementor, Divi)
        let _ = super::post_ast::synchronize_post_ast_layout(
            &conn,
            id,
            &req.content,
            req.meta.as_ref(),
        );

        drop(conn);
        Ok(self.get_post_by_id(id)?.unwrap())
    }

    pub fn update_post(
        &self,
        id: i64,
        req: &UpdatePostRequest,
    ) -> Result<Option<DaypressPostResponse>> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let site_offset_secs = super::scheduling::read_site_timezone_offset(&tx, None);
        let now_local = (Utc::now().naive_utc()
            + chrono::Duration::seconds(site_offset_secs as i64))
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

        // Capture old slug & status before updating so we can auto-create a 301 redirect if slug changes on a published post
        let old_slug_and_status: Option<(String, String)> = tx
            .query_row(
                "SELECT post_name, post_status FROM posts WHERE ID = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();

        let revision_candidate =
            req.title.is_some() || req.content.is_some() || req.excerpt.is_some();
        if revision_candidate {
            let _ = Self::snapshot_post_revision_with_conn(&tx, id, &now_local, &now);
        }

        if let Some(title) = &req.title {
            tx.execute(
                "UPDATE posts SET post_title = ?1, post_modified = ?2, post_modified_gmt = ?3 WHERE ID = ?4",
                params![title, now_local, now, id],
            )?;
        }
        if let Some(content) = &req.content {
            tx.execute(
                "UPDATE posts SET post_content = ?1, post_modified = ?2, post_modified_gmt = ?3 WHERE ID = ?4",
                params![content, now_local, now, id],
            )?;
        }
        if let Some(status) = &req.status {
            tx.execute(
                "UPDATE posts SET post_status = ?1, post_modified = ?2, post_modified_gmt = ?3 WHERE ID = ?4",
                params![status, now_local, now, id],
            )?;
            // Keep WordPress trash bookkeeping in sync when status moves into / out of the Trash.
            if let Some((_, old_status)) = old_slug_and_status.as_ref() {
                if status == "trash" && old_status != "trash" {
                    Self::record_trash_meta(&tx, id, old_status);
                } else if status != "trash" && old_status == "trash" {
                    Self::clear_trash_meta(&tx, id);
                }
            }
        }
        // A new date and/or status can (un)schedule the post, like WordPress, respecting site timezone.
        let tz_str: String = tx
            .query_row(
                "SELECT option_value FROM options WHERE option_name = 'timezone_string'",
                [],
                |r| r.get(0),
            )
            .unwrap_or_default();
        let gmt_off_str: Option<String> = tx
            .query_row(
                "SELECT option_value FROM options WHERE option_name = 'gmt_offset'",
                [],
                |r| r.get(0),
            )
            .ok();
        let new_dates = super::scheduling::requested_post_dates_with_tz(
            req.date.as_deref(),
            req.date_gmt.as_deref(),
            &tz_str,
            gmt_off_str.as_deref(),
        );
        super::scheduling::apply_schedule_on_update(
            &tx,
            id,
            req.status.is_some(),
            new_dates.as_ref().map(|(l, u)| (l.as_str(), u.as_str())),
            &now,
        )?;
        if let Some(slug) = &req.slug {
            let clean_slug = if slug.trim().is_empty() {
                req.title
                    .as_deref()
                    .map(generated_slugify::slugify)
                    .unwrap_or_default()
            } else {
                generated_slugify::slugify(slug.trim())
            };
            if !clean_slug.is_empty() {
                let clean_slug = Self::unique_post_slug(&tx, &clean_slug, Some(id))?;
                tx.execute(
                    "UPDATE posts SET post_name = ?1, post_modified = ?2 WHERE ID = ?3",
                    params![clean_slug, now, id],
                )?;

                // Auto-create 301 redirect if a published post's slug changed
                if let Some((old_slug, old_status)) = old_slug_and_status {
                    if !old_slug.is_empty()
                        && old_slug != clean_slug
                        && (old_status == "publish" || req.status.as_deref() == Some("publish"))
                    {
                        let auto_redir: bool = tx
                            .query_row(
                                "SELECT option_value FROM options WHERE option_name = 'daypress_seo_auto_redirect'",
                                [],
                                |r| r.get::<_, String>(0),
                            )
                            .map(|v| v != "0" && v != "false")
                            .unwrap_or(true);
                        if auto_redir {
                            let src = format!("/{}/", old_slug.trim_matches('/'));
                            let dst = format!("/{}/", clean_slug.trim_matches('/'));
                            let _ = tx.execute(
                                "INSERT INTO daypress_redirects (source_path, target_url, status_code, is_active)
                                 VALUES (?1, ?2, 301, 1)
                                 ON CONFLICT(source_path) DO UPDATE SET target_url = excluded.target_url",
                                params![src, dst],
                            );
                        }
                    }
                }
            }
        }
        if let Some(excerpt) = &req.excerpt {
            tx.execute(
                "UPDATE posts SET post_excerpt = ?1, post_modified = ?2 WHERE ID = ?3",
                params![excerpt, now, id],
            )?;
        }
        if let Some(parent) = req.parent {
            tx.execute(
                "UPDATE posts SET post_parent = ?1, post_modified = ?2 WHERE ID = ?3",
                params![parent, now, id],
            )?;
        }
        if let Some(order) = req.menu_order {
            tx.execute(
                "UPDATE posts SET menu_order = ?1, post_modified = ?2 WHERE ID = ?3",
                params![order, now, id],
            )?;
        }
        if let Some(author) = req.author {
            tx.execute(
                "UPDATE posts SET post_author = ?1, post_modified = ?2 WHERE ID = ?3",
                params![author, now, id],
            )?;
        }
        if let Some(ref tpl) = req.template {
            let _ = tx.execute(
                "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_wp_page_template'",
                params![id],
            );
            if !tpl.is_empty() {
                let _ = tx.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_page_template', ?2)",
                    params![id, tpl],
                );
            }
        }
        if let Some(fm) = req.featured_media {
            let _ = tx.execute(
                "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_thumbnail_id'",
                params![id],
            );
            if fm > 0 {
                let _ = tx.execute("INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_thumbnail_id', ?2)", params![id, fm.to_string()]);
            }
        }
        let mut all_taxonomies: std::collections::HashMap<String, Vec<i64>> =
            req.taxonomies.clone().unwrap_or_default();
        if let Some(cats) = &req.categories {
            if !all_taxonomies.contains_key("category") {
                all_taxonomies.insert("category".to_string(), cats.clone());
            }
        }
        if let Some(tags) = &req.tags {
            if !all_taxonomies.contains_key("post_tag") {
                all_taxonomies.insert("post_tag".to_string(), tags.clone());
            }
        }

        for (tax_name, term_ids) in all_taxonomies {
            let _ = tx.execute(
                "DELETE FROM term_relationships WHERE object_id = ?1 AND term_taxonomy_id IN (SELECT term_taxonomy_id FROM term_taxonomy WHERE taxonomy = ?2)",
                params![id, &tax_name],
            );
            for term_id in term_ids {
                let tt_id: Option<i64> = tx.query_row(
                    "SELECT term_taxonomy_id FROM term_taxonomy WHERE term_id = ?1 AND taxonomy = ?2",
                    params![term_id, &tax_name],
                    |r| r.get(0)
                ).ok();
                if let Some(tid) = tt_id {
                    let _ = tx.execute(
                        "INSERT OR IGNORE INTO term_relationships (object_id, term_taxonomy_id) VALUES (?1, ?2)",
                        params![id, tid],
                    );
                }
            }
        }

        // Update SEO & OpenGraph postmeta fields when provided
        let seo_updates = [
            ("_daypress_seo_title", req.seo_title.as_deref()),
            ("_daypress_seo_desc", req.seo_description.as_deref()),
            ("_daypress_seo_keyword", req.seo_keyword.as_deref()),
            ("_daypress_seo_canonical", req.seo_canonical.as_deref()),
            ("_daypress_seo_schema_type", req.seo_schema_type.as_deref()),
            ("_daypress_seo_og_image", req.seo_og_image.as_deref()),
        ];
        for (meta_key, val_opt) in seo_updates {
            if let Some(val) = val_opt {
                let _ = tx.execute(
                    "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = ?2",
                    params![id, meta_key],
                );
                if !val.trim().is_empty() {
                    let _ = tx.execute(
                        "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, ?2, ?3)",
                        params![id, meta_key, val.trim()],
                    );
                }
            }
        }
        if let Some(noindex) = req.seo_noindex {
            let _ = tx.execute(
                "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_seo_noindex'",
                params![id],
            );
            if noindex {
                let _ = tx.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_seo_noindex', '1')",
                    params![id],
                );
            }
        }

        // Persist custom / ACF postmeta fields when provided in `req.meta`
        if let Some(serde_json::Value::Object(ref meta_map)) = req.meta {
            for (mk, mv) in meta_map {
                let k = mk.trim();
                if k.is_empty() || k.starts_with("_tax_") || k == "_daypress_layout" {
                    continue;
                }
                let _ = tx.execute(
                    "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = ?2",
                    params![id, k],
                );
                if !mv.is_null() {
                    let val_str = match mv {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    let _ = tx.execute(
                        "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, ?2, ?3)",
                        params![id, k, val_str],
                    );
                }
            }
        }

        // Keep `layout_json` / `_daypress_layout` synchronized when `post_content` is updated from the Post/Block Editor
        if let Some(ref content) = req.content {
            let _ =
                super::post_ast::synchronize_post_ast_layout(&tx, id, content, req.meta.as_ref());
        }

        tx.commit()?;
        drop(conn);
        self.get_post_by_id(id)
    }

    /// 1-Click Duplicate Post / Page (`Yoast Duplicate Post` parity):
    /// Deep-clones the `posts` row (with `post_status = 'draft'`, `" (Copy)"` suffix,
    /// and unique `-copy` slug), `layout_json`, all `postmeta` rows (SEO, featured image,
    /// custom fields, Elementor/Daypress layout), and `term_relationships` in a single transaction.
    pub fn duplicate_post(&self, source_id: i64) -> Result<Option<DaypressPostResponse>> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let source_row: Option<(
            i64,
            String,
            String,
            String,
            String,
            String,
            String,
            String,
            i64,
            i64,
            Option<String>,
        )> = tx
            .query_row(
                r#"SELECT post_author, post_content, post_title, post_excerpt, comment_status, ping_status, post_name, post_type, COALESCE(post_parent, 0), COALESCE(menu_order, 0), layout_json
                   FROM posts WHERE ID = ?1"#,
                params![source_id],
                |r| {
                    Ok((
                        r.get(0)?,
                        r.get(1)?,
                        r.get(2)?,
                        r.get(3)?,
                        r.get(4)?,
                        r.get(5)?,
                        r.get(6)?,
                        r.get(7)?,
                        r.get(8)?,
                        r.get(9)?,
                        r.get(10)?,
                    ))
                },
            )
            .ok();

        let Some((
            author,
            content,
            title,
            excerpt,
            comment_status,
            ping_status,
            old_slug,
            post_type,
            parent,
            menu_order,
            layout_json,
        )) = source_row
        else {
            return Ok(None);
        };

        let new_title = format!("{} (Copy)", title);
        let base_slug = if old_slug.trim().is_empty() {
            generated_slugify::slugify(&title)
        } else {
            old_slug
        };
        let mut candidate_slug = format!("{}-copy", base_slug);
        let mut suffix = 2;
        while tx
            .query_row(
                "SELECT ID FROM posts WHERE post_name = ?1",
                params![candidate_slug],
                |r| r.get::<_, i64>(0),
            )
            .is_ok()
        {
            candidate_slug = format!("{}-copy-{}", base_slug, suffix);
            suffix += 1;
        }

        tx.execute(
            r#"INSERT INTO posts (post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, comment_status, ping_status, post_name, post_modified, post_modified_gmt, guid, post_type, post_parent, menu_order, layout_json)
               VALUES (?1, ?2, ?2, ?3, ?4, ?5, 'draft', ?6, ?7, ?8, ?2, ?2, '', ?9, ?10, ?11, ?12)"#,
            params![
                author,
                now,
                content,
                new_title,
                excerpt,
                comment_status,
                ping_status,
                candidate_slug,
                post_type,
                parent,
                menu_order,
                layout_json
            ],
        )?;
        let new_id = tx.last_insert_rowid();
        let guid = format!(
            "{}/?p={}",
            Self::get_base_url_with_conn(&tx).trim_end_matches('/'),
            new_id
        );
        let _ = tx.execute(
            "UPDATE posts SET guid = ?1 WHERE ID = ?2",
            params![guid, new_id],
        );

        // Clone all postmeta rows (except Trash bookkeeping of a trashed source)
        tx.execute(
            "INSERT INTO postmeta (post_id, meta_key, meta_value)
             SELECT ?1, meta_key, meta_value FROM postmeta WHERE post_id = ?2
               AND meta_key NOT IN ('_wp_trash_meta_status', '_wp_trash_meta_time')",
            params![new_id, source_id],
        )?;

        // Clone all term_relationships rows
        tx.execute(
            "INSERT OR IGNORE INTO term_relationships (object_id, term_taxonomy_id, term_order)
             SELECT ?1, term_taxonomy_id, term_order FROM term_relationships WHERE object_id = ?2",
            params![new_id, source_id],
        )?;

        tx.commit()?;
        drop(conn);
        self.get_post_by_id(new_id)
    }

    pub fn trash_post(&self, id: i64) -> Result<Option<DaypressPostResponse>> {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        {
            let conn = self.conn.lock().unwrap();
            let prev_status: Option<String> = conn
                .query_row(
                    "SELECT post_status FROM posts WHERE ID = ?1",
                    params![id],
                    |r| r.get(0),
                )
                .ok();
            let Some(prev_status) = prev_status else {
                return Ok(None);
            };
            if prev_status != "trash" {
                conn.execute(
                    "UPDATE posts SET post_status = 'trash', post_modified = ?1 WHERE ID = ?2",
                    params![now, id],
                )?;
                Self::record_trash_meta(&conn, id, &prev_status);
            }
        }
        self.get_post_by_id(id)
    }

    /// WordPress `wp_trash_post()` parity: remember the pre-trash status and trash time
    /// (`_wp_trash_meta_status` / `_wp_trash_meta_time`) so the item can be restored.
    pub(crate) fn record_trash_meta(conn: &Connection, id: i64, prev_status: &str) {
        let _ = conn.execute(
            "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key IN ('_wp_trash_meta_status', '_wp_trash_meta_time')",
            params![id],
        );
        let _ = conn.execute(
            "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_trash_meta_status', ?2)",
            params![id, prev_status],
        );
        let _ = conn.execute(
            "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_trash_meta_time', ?2)",
            params![id, Utc::now().timestamp().to_string()],
        );
    }

    /// WordPress `wp_untrash_post()` parity: drop the trash bookkeeping meta.
    pub(crate) fn clear_trash_meta(conn: &Connection, id: i64) {
        let _ = conn.execute(
            "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key IN ('_wp_trash_meta_status', '_wp_trash_meta_time')",
            params![id],
        );
    }

    pub fn delete_post(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "DELETE FROM postmeta WHERE post_id IN (SELECT ID FROM posts WHERE post_parent = ?1 AND post_type = 'revision')",
            params![id],
        );
        let _ = conn.execute(
            "DELETE FROM posts WHERE post_parent = ?1 AND post_type = 'revision'",
            params![id],
        );
        let _ = conn.execute("DELETE FROM postmeta WHERE post_id = ?1", params![id]);
        let _ = conn.execute(
            "DELETE FROM term_relationships WHERE object_id = ?1",
            params![id],
        );
        let rows = conn.execute("DELETE FROM posts WHERE ID = ?1", params![id])?;
        Ok(rows > 0)
    }
}
