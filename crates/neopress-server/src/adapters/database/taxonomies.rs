#![allow(unused_imports)]
use crate::auth::AuthUser;
use crate::domain::errors::{DomainError, DomainResult};
use crate::errors::AppError;
use crate::models::*;
use super::Database;
use base64::Engine;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

impl Database {
    pub fn get_terms(&self, taxonomy: &str) -> Result<Vec<WpCategoryResponse>> {
        let conn = self.conn.lock().unwrap();
        let base_url = Self::get_base_url_with_conn(&conn);
        let route_prefix = if taxonomy == "post_tag" {
            "tag"
        } else {
            "category"
        };
        // `count` mirrors WordPress `_update_post_term_count()`: only published objects are counted.
        let mut stmt = conn.prepare(
            r#"SELECT t.term_id, t.name, t.slug, COALESCE(tt.description, ''), COALESCE(tt.parent, 0),
                      (SELECT count(*) FROM wp_term_relationships r JOIN wp_posts p ON p.ID = r.object_id
                        WHERE r.term_taxonomy_id = tt.term_taxonomy_id AND p.post_status = 'publish') as cnt
               FROM wp_terms t
               JOIN wp_term_taxonomy tt ON t.term_id = tt.term_id
               WHERE tt.taxonomy = ?1
               ORDER BY t.name ASC"#
        )?;

        let rows = stmt.query_map(params![taxonomy], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let slug: String = row.get(2)?;
            let description: String = row.get(3)?;
            let parent: i64 = row.get(4)?;
            let count: i64 = row.get(5)?;

            Ok(WpCategoryResponse {
                id,
                count,
                description,
                link: format!(
                    "{}/{}/{}/",
                    base_url.trim_end_matches('/'),
                    route_prefix,
                    slug
                ),
                name,
                slug,
                taxonomy: taxonomy.to_string(),
                parent,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_term_by_id(&self, taxonomy: &str, id: i64) -> Result<Option<WpCategoryResponse>> {
        let list = self.get_terms(taxonomy)?;
        Ok(list.into_iter().find(|t| t.id == id))
    }

    pub fn create_term(
        &self,
        taxonomy: &str,
        req: &CreateTermRequest,
    ) -> Result<WpCategoryResponse, AppError> {
        let name = req.name.trim();
        if name.is_empty() {
            return Err(AppError::BadRequest("Term name is required".to_string()));
        }
        let slug = match &req.slug {
            Some(s) if !s.trim().is_empty() => generated_slugify::slugify(s),
            _ => generated_slugify::slugify(name),
        };
        let slug = if slug.trim().is_empty() { "term".to_string() } else { slug };
        let desc = req.description.as_deref().unwrap_or("");
        let parent = req.parent.unwrap_or(0).max(0);

        let term_id = {
            let conn = self.conn.lock().unwrap();
            // Re-use (upsert) only a term that already belongs to *this* taxonomy. A slug owned by a
            // different taxonomy (e.g. category `news` vs tag `news`) must NOT be shared, otherwise
            // renaming/deleting the tag would silently mutate the category. `wp_terms.slug` is
            // globally UNIQUE here, so such collisions get a WordPress-style `-2`, `-3` suffix.
            let existing: Option<i64> = conn
                .query_row(
                    "SELECT t.term_id FROM wp_terms t JOIN wp_term_taxonomy tt ON t.term_id = tt.term_id
                     WHERE t.slug = ?1 AND tt.taxonomy = ?2",
                    params![slug, taxonomy],
                    |r| r.get(0),
                )
                .ok();
            let tid = match existing {
                Some(id) => {
                    conn.execute(
                        "UPDATE wp_terms SET name = ?1 WHERE term_id = ?2",
                        params![name, id],
                    )?;
                    id
                }
                None => {
                    let slug_taken = |s: &str| {
                        conn.query_row(
                            "SELECT term_id FROM wp_terms WHERE slug = ?1",
                            params![s],
                            |r| r.get::<_, i64>(0),
                        )
                        .is_ok()
                    };
                    let mut unique_slug = slug.clone();
                    let mut n = 2;
                    while slug_taken(&unique_slug) {
                        unique_slug = format!("{}-{}", slug, n);
                        n += 1;
                    }
                    conn.execute(
                        "INSERT INTO wp_terms (name, slug) VALUES (?1, ?2)",
                        params![name, unique_slug],
                    )?;
                    conn.last_insert_rowid()
                }
            };

            let tt_exists: Option<i64> = conn
                .query_row(
                    "SELECT term_taxonomy_id FROM wp_term_taxonomy WHERE term_id = ?1 AND taxonomy = ?2",
                    params![tid, taxonomy],
                    |r| r.get(0),
                )
                .ok();
            match tt_exists {
                Some(tt_id) => {
                    conn.execute(
                        "UPDATE wp_term_taxonomy SET description = ?1, parent = ?2 WHERE term_taxonomy_id = ?3",
                        params![desc, parent, tt_id],
                    )?;
                }
                None => {
                    conn.execute(
                        "INSERT INTO wp_term_taxonomy (term_id, taxonomy, description, parent) VALUES (?1, ?2, ?3, ?4)",
                        params![tid, taxonomy, desc, parent],
                    )?;
                }
            }
            tid
        };

        self.get_term_by_id(taxonomy, term_id)?
            .ok_or_else(|| AppError::Internal("Failed to load created term".to_string()))
    }

    pub fn create_category(&self, req: &CreateTermRequest) -> Result<WpCategoryResponse, AppError> {
        self.create_term("category", req)
    }

    pub fn create_tag(&self, req: &CreateTermRequest) -> Result<WpCategoryResponse, AppError> {
        self.create_term("post_tag", req)
    }

    pub fn update_term(
        &self,
        taxonomy: &str,
        id: i64,
        req: &UpdateTermRequest,
    ) -> Result<Option<WpCategoryResponse>> {
        {
            let conn = self.conn.lock().unwrap();
            let exists: Option<i64> = conn
                .query_row(
                    "SELECT t.term_id FROM wp_terms t JOIN wp_term_taxonomy tt ON t.term_id = tt.term_id WHERE t.term_id = ?1 AND tt.taxonomy = ?2",
                    params![id, taxonomy],
                    |r| r.get(0),
                )
                .ok();
            if exists.is_none() {
                return Ok(None);
            }

            if let Some(ref name) = req.name {
                conn.execute(
                    "UPDATE wp_terms SET name = ?1 WHERE term_id = ?2",
                    params![name, id],
                )?;
            }
            if let Some(ref slug) = req.slug {
                let s = if slug.trim().is_empty() {
                    req.name
                        .as_deref()
                        .map(generated_slugify::slugify)
                        .unwrap_or_default()
                } else {
                    generated_slugify::slugify(slug)
                };
                if !s.is_empty() {
                    conn.execute(
                        "UPDATE wp_terms SET slug = ?1 WHERE term_id = ?2",
                        params![s, id],
                    )?;
                }
            }
            if let Some(ref desc) = req.description {
                conn.execute(
                    "UPDATE wp_term_taxonomy SET description = ?1 WHERE term_id = ?2 AND taxonomy = ?3",
                    params![desc, id, taxonomy],
                )?;
            }
            if let Some(parent) = req.parent {
                conn.execute(
                    "UPDATE wp_term_taxonomy SET parent = ?1 WHERE term_id = ?2 AND taxonomy = ?3",
                    params![parent, id, taxonomy],
                )?;
            }
        }
        self.get_term_by_id(taxonomy, id)
    }

    pub fn delete_term(&self, taxonomy: &str, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let tt_id: Option<i64> = conn
            .query_row(
                "SELECT term_taxonomy_id FROM wp_term_taxonomy WHERE term_id = ?1 AND taxonomy = ?2",
                params![id, taxonomy],
                |r| r.get(0),
            )
            .ok();
        let Some(tt_id) = tt_id else {
            return Ok(false);
        };
        let _ = conn.execute(
            "DELETE FROM wp_term_relationships WHERE term_taxonomy_id = ?1",
            params![tt_id],
        );
        let rows = conn.execute(
            "DELETE FROM wp_term_taxonomy WHERE term_taxonomy_id = ?1",
            params![tt_id],
        )?;
        let remaining: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM wp_term_taxonomy WHERE term_id = ?1",
                params![id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if remaining == 0 {
            let _ = conn.execute("DELETE FROM wp_terms WHERE term_id = ?1", params![id]);
        }
        Ok(rows > 0)
    }

    pub fn get_taxonomy_term(&self, taxonomy: &str, slug: &str) -> Result<Option<WpCategoryResponse>> {
        let conn = self.conn.lock().unwrap();
        let base_url = Self::get_base_url_with_conn(&conn);
        let route_prefix = if taxonomy == "post_tag" {
            "tag"
        } else if taxonomy == "category" {
            "category"
        } else {
            taxonomy
        };
        let mut stmt = conn.prepare(
            r#"SELECT t.term_id, t.name, t.slug, COALESCE(tt.description, ''), COALESCE(tt.parent, 0),
                      (SELECT count(*) FROM wp_term_relationships r WHERE r.term_taxonomy_id = tt.term_taxonomy_id) as cnt
               FROM wp_terms t
               JOIN wp_term_taxonomy tt ON t.term_id = tt.term_id
               WHERE tt.taxonomy = ?1 AND t.slug = ?2
               LIMIT 1"#,
        )?;

        let mut rows = stmt.query_map(params![taxonomy, slug], |row| {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let slug: String = row.get(2)?;
            let description: String = row.get(3)?;
            let parent: i64 = row.get(4)?;
            let count: i64 = row.get(5)?;

            Ok(WpCategoryResponse {
                id,
                count,
                description,
                link: format!(
                    "{}/{}/{}/",
                    base_url.trim_end_matches('/'),
                    route_prefix,
                    slug
                ),
                name,
                slug,
                taxonomy: taxonomy.to_string(),
                parent,
            })
        })?;

        if let Some(r) = rows.next() {
            Ok(Some(r?))
        } else {
            Ok(None)
        }
    }

    pub fn get_posts_by_taxonomy_term(
        &self,
        taxonomy: &str,
        term_id: i64,
    ) -> Result<Vec<WpPostResponse>> {
        let conn = self.conn.lock().unwrap();
        let base_url = Self::get_base_url_with_conn(&conn);

        let select_fields = "SELECT p.ID, p.post_date, p.post_date_gmt, p.post_modified, p.post_modified_gmt, \
            p.post_title, p.post_content, p.post_excerpt, p.post_name, p.post_status, p.post_type, p.guid, \
            p.post_author, p.comment_status, p.ping_status, COALESCE(p.post_parent, 0), COALESCE(p.menu_order, 0), \
            COALESCE(p.is_sticky, 0) \
            FROM wp_posts p \
            JOIN wp_term_relationships r ON p.ID = r.object_id \
            JOIN wp_term_taxonomy tt ON r.term_taxonomy_id = tt.term_taxonomy_id \
            WHERE tt.taxonomy = ?1 AND tt.term_id = ?2 AND p.post_status = 'publish' \
            ORDER BY p.is_sticky DESC, p.post_date DESC";

        let mut stmt = conn.prepare(select_fields)?;
        let rows = stmt.query_map(params![taxonomy, term_id], Self::map_post_row)?;

        let mut posts = Vec::new();
        for r in rows {
            posts.push(r?);
        }

        let mut hydrated = Vec::with_capacity(posts.len());
        for mut post in posts {
            Self::hydrate_post_relations(&conn, &mut post, &base_url);
            hydrated.push(post);
        }

        Ok(hydrated)
    }
}

impl crate::ports::output::TaxonomyRepository for Database {
    fn list_terms(&self, taxonomy: &str) -> DomainResult<Vec<WpCategoryResponse>> {
        self.get_terms(taxonomy).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn find_term_by_id(&self, taxonomy: &str, id: i64) -> DomainResult<Option<WpCategoryResponse>> {
        self.get_term_by_id(taxonomy, id)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn create_term(&self, taxonomy: &str, req: &CreateTermRequest) -> DomainResult<WpCategoryResponse> {
        Database::create_term(self, taxonomy, req).map_err(DomainError::from)
    }

    fn update_term(
        &self,
        taxonomy: &str,
        id: i64,
        req: &UpdateTermRequest,
    ) -> DomainResult<Option<WpCategoryResponse>> {
        Database::update_term(self, taxonomy, id, req)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn delete_term(&self, taxonomy: &str, id: i64) -> DomainResult<bool> {
        Database::delete_term(self, taxonomy, id).map_err(|e| DomainError::Storage(e.to_string()))
    }
}
