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
    pub(crate) fn hydrate_post_relations(
        conn: &Connection,
        post: &mut WpPostResponse,
        base_url: &str,
    ) {
        let base_trimmed = base_url.trim_end_matches('/');
        post.link = if matches!(
            post.post_type.as_str(),
            "trainingen" | "vacatures" | "cases" | "kandidaten"
        ) {
            format!("{}/{}/{}/", base_trimmed, post.post_type, post.slug)
        } else {
            format!("{}/{}/", base_trimmed, post.slug)
        };

        let featured_media: i64 = conn
            .query_row(
                "SELECT meta_value FROM postmeta WHERE post_id = ?1 AND meta_key = '_thumbnail_id'",
                params![post.id],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);
        post.featured_media = featured_media;

        if featured_media > 0 {
            post.featured_image_url = conn
                .query_row(
                    "SELECT guid FROM posts WHERE ID = ?1",
                    params![featured_media],
                    |r| r.get::<_, String>(0),
                )
                .ok()
                .filter(|s| !s.is_empty());
        } else {
            post.featured_image_url = None;
        }

        let cat_pairs: Vec<(i64, String)> = conn
            .prepare_cached(
                "SELECT tt.term_id, t.name FROM term_relationships r JOIN term_taxonomy tt ON r.term_taxonomy_id = tt.term_taxonomy_id JOIN terms t ON tt.term_id = t.term_id WHERE r.object_id = ?1 AND tt.taxonomy = 'category'",
            )
            .and_then(|mut stmt| {
                let rows = stmt.query_map(params![post.id], |r| Ok((r.get(0)?, r.get(1)?)))?;
                Ok(rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        let custom_tax_pairs: Vec<(String, String)> = conn
            .prepare_cached(
                "SELECT tt.taxonomy, t.name FROM term_relationships r JOIN term_taxonomy tt ON r.term_taxonomy_id = tt.term_taxonomy_id JOIN terms t ON tt.term_id = t.term_id WHERE r.object_id = ?1 AND tt.taxonomy NOT IN ('category', 'post_tag', 'nav_menu')",
            )
            .and_then(|mut stmt| {
                let rows = stmt.query_map(params![post.id], |r| Ok((r.get(0)?, r.get(1)?)))?;
                Ok(rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        if cat_pairs.is_empty() {
            post.categories = vec![1];
            if !custom_tax_pairs.is_empty() {
                post.category_names = custom_tax_pairs.iter().map(|(_, n)| n.clone()).collect();
            } else if post.post_type == "post" {
                let default_cat_name: String = conn
                    .query_row("SELECT name FROM terms WHERE term_id = 1", [], |r| r.get(0))
                    .unwrap_or_else(|_| "Uncategorized".to_string());
                post.category_names = vec![default_cat_name];
            } else {
                post.category_names = vec![];
            }
        } else {
            post.categories = cat_pairs.iter().map(|(id, _)| *id).collect();
            post.category_names = cat_pairs.into_iter().map(|(_, n)| n).collect();
        }

        // Hydrate custom / ACF postmeta fields into `post.meta`
        let meta_rows: Vec<(String, String)> = conn
            .prepare_cached(
                "SELECT meta_key, COALESCE(meta_value, '') FROM postmeta WHERE post_id = ?1 AND meta_key NOT IN ('_elementor_data', '_daypress_layout')",
            )
            .and_then(|mut stmt| {
                let rows = stmt.query_map(params![post.id], |r| Ok((r.get(0)?, r.get(1)?)))?;
                Ok(rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();
        let mut meta_obj = serde_json::Map::new();
        for (k, v) in meta_rows {
            if !k.starts_with("_field_") && !v.starts_with("field_") {
                let parsed_val = if v == "true" {
                    serde_json::Value::Bool(true)
                } else if v == "false" {
                    serde_json::Value::Bool(false)
                } else if let Ok(n) = v.parse::<i64>() {
                    serde_json::Value::Number(n.into())
                } else if (v.starts_with('{') && v.ends_with('}'))
                    || (v.starts_with('[') && v.ends_with(']'))
                {
                    serde_json::from_str::<serde_json::Value>(&v)
                        .unwrap_or_else(|_| serde_json::Value::String(v.clone()))
                } else {
                    serde_json::Value::String(v)
                };
                meta_obj.insert(k, parsed_val);
            }
        }
        for (tax, label) in &custom_tax_pairs {
            meta_obj.insert(
                format!("_tax_{}", tax),
                serde_json::Value::String(label.clone()),
            );
        }
        post.meta = serde_json::Value::Object(meta_obj);

        let tag_pairs: Vec<(i64, String)> = conn
            .prepare_cached(
                "SELECT tt.term_id, t.name FROM term_relationships r JOIN term_taxonomy tt ON r.term_taxonomy_id = tt.term_taxonomy_id JOIN terms t ON tt.term_id = t.term_id WHERE r.object_id = ?1 AND tt.taxonomy = 'post_tag'",
            )
            .and_then(|mut stmt| {
                let rows = stmt.query_map(params![post.id], |r| Ok((r.get(0)?, r.get(1)?)))?;
                Ok(rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        post.tags = tag_pairs.iter().map(|(id, _)| *id).collect();
        post.tag_names = tag_pairs.into_iter().map(|(_, n)| n).collect();

        post.author_name = conn
            .query_row(
                "SELECT display_name FROM users WHERE ID = ?1",
                params![post.author],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "admin".to_string());

        post.template = conn
            .query_row(
                "SELECT meta_value FROM postmeta WHERE post_id = ?1 AND meta_key = '_wp_page_template'",
                params![post.id],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .unwrap_or_default();

        let post_link = post.link.clone();
        post.comments = conn
            .prepare_cached(
                r#"SELECT comment_ID, comment_post_ID, comment_parent, user_id, comment_author, comment_author_url, comment_date, comment_date_gmt, comment_content, comment_approved, comment_type
                   FROM comments
                   WHERE comment_post_ID = ?1 AND (comment_approved = '1' OR comment_approved = 'approve' OR comment_approved = 'approved')
                   ORDER BY comment_ID ASC"#,
            )
            .and_then(|mut stmt| {
                let rows = stmt.query_map(params![post.id], |r| {
                    let cid: i64 = r.get(0)?;
                    let cpost: i64 = r.get(1)?;
                    let cparent: i64 = r.get(2)?;
                    let uid: i64 = r.get(3)?;
                    let cauthor: String = r.get(4)?;
                    let curl: String = r.get(5)?;
                    let cdate: String = r.get(6)?;
                    let cdate_gmt: String = r.get(7)?;
                    let ccontent: String = r.get(8)?;
                    let capproved: String = r.get(9)?;
                    let ctype: String = r.get(10)?;
                    Ok(WpCommentResponse {
                        id: cid,
                        post: cpost,
                        parent: cparent,
                        author: uid,
                        author_name: cauthor,
                        author_url: curl,
                        date: cdate.replace(' ', "T"),
                        date_gmt: cdate_gmt.replace(' ', "T"),
                        content: WpRenderedText {
                            rendered: ccontent,
                            protected: false,
                        },
                        link: format!("{}#comment-{}", post_link, cid),
                        status: if capproved == "1" {
                            "approved".to_string()
                        } else {
                            capproved
                        },
                        comment_type: ctype,
                    })
                })?;
                Ok(rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        // Hydrate SEO & OpenGraph metadata (checking both canonical _daypress_seo_* and legacy Yoast/RankMath keys)
        let seo_rows: Vec<(String, String)> = conn
            .prepare_cached(
                r#"SELECT meta_key, COALESCE(meta_value, '') FROM postmeta
                   WHERE post_id = ?1 AND meta_key IN (
                       '_daypress_seo_title', '_yoast_wpseo_title', 'rank_math_title',
                       '_daypress_seo_desc', '_yoast_wpseo_metadesc', 'rank_math_description',
                       '_daypress_seo_keyword', '_yoast_wpseo_focuskw', 'rank_math_focus_keyword',
                       '_daypress_seo_canonical', '_yoast_wpseo_canonical', 'rank_math_canonical_url',
                       '_daypress_seo_noindex', '_yoast_wpseo_meta-robots-noindex', 'rank_math_robots',
                       '_daypress_seo_schema_type',
                       '_daypress_seo_og_image', '_yoast_wpseo_opengraph-image', 'rank_math_facebook_image'
                   )"#,
            )
            .and_then(|mut stmt| {
                let rows = stmt.query_map(params![post.id], |r| Ok((r.get(0)?, r.get(1)?)))?;
                Ok(rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        let mut seo_map: std::collections::HashMap<String, String> = seo_rows.into_iter().collect();
        let site_title: String = conn
            .query_row(
                "SELECT option_value FROM options WHERE option_name = 'blogname'",
                [],
                |r| r.get(0),
            )
            .unwrap_or_else(|_| "Daypress".to_string());
        crate::convergence::converge_post_seo_meta(&mut seo_map, &post.title.rendered, &site_title);

        post.seo_title = seo_map.remove("_daypress_seo_title").unwrap_or_default();
        post.seo_description = seo_map.remove("_daypress_seo_desc").unwrap_or_default();
        post.seo_keyword = seo_map.remove("_daypress_seo_keyword").unwrap_or_default();
        post.seo_canonical = seo_map
            .remove("_daypress_seo_canonical")
            .unwrap_or_default();
        post.seo_noindex = seo_map
            .get("_daypress_seo_noindex")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        post.seo_schema_type = seo_map
            .remove("_daypress_seo_schema_type")
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                if post.post_type == "page" {
                    "WebPage".to_string()
                } else {
                    "Article".to_string()
                }
            });
        post.seo_og_image = seo_map.remove("_daypress_seo_og_image").unwrap_or_default();
    }

    pub(crate) fn map_post_row(row: &rusqlite::Row) -> rusqlite::Result<WpPostResponse> {
        let id: i64 = row.get(0)?;
        let date: String = row.get(1)?;
        let date_gmt: String = row.get(2)?;
        let modified: String = row.get(3)?;
        let modified_gmt: String = row.get(4)?;
        let title_raw: String = row.get(5)?;
        let content_raw: String = row.get(6)?;
        let excerpt_raw: String = row.get(7)?;
        let slug: String = row.get(8)?;
        let status: String = row.get(9)?;
        let ptype: String = row.get(10)?;
        let guid: String = row.get(11)?;
        let author: i64 = row.get(12)?;
        let comment_status: String = row.get(13)?;
        let ping_status: String = row.get(14)?;
        let parent: i64 = row.get(15)?;
        let menu_order: i64 = row.get(16)?;
        let is_sticky: i64 = row.get(17).unwrap_or(0);

        Ok(WpPostResponse {
            id,
            date: date.replace(' ', "T"),
            date_gmt: date_gmt.replace(' ', "T"),
            guid: WpGuid { rendered: guid },
            modified: modified.replace(' ', "T"),
            modified_gmt: modified_gmt.replace(' ', "T"),
            slug: slug.clone(),
            status,
            post_type: ptype,
            link: String::new(),
            title: WpRenderedText {
                rendered: title_raw,
                protected: false,
            },
            content: WpRenderedText {
                rendered: content_raw,
                protected: false,
            },
            excerpt: WpRenderedText {
                rendered: excerpt_raw,
                protected: false,
            },
            author,
            featured_media: 0,
            comment_status,
            ping_status,
            sticky: is_sticky == 1,
            template: String::new(),
            format: "standard".to_string(),
            meta: serde_json::json!({}),
            categories: vec![1],
            tags: vec![],
            parent,
            menu_order,
            author_name: "admin".to_string(),
            category_names: vec![],
            tag_names: vec![],
            featured_image_url: None,
            comments: vec![],
            seo_title: String::new(),
            seo_description: String::new(),
            seo_keyword: String::new(),
            seo_canonical: String::new(),
            seo_noindex: false,
            seo_schema_type: "Article".to_string(),
            seo_og_image: String::new(),
        })
    }

    pub fn get_posts(
        &self,
        post_type: &str,
        status: Option<&str>,
        search: Option<&str>,
        limit: i64,
    ) -> Result<Vec<WpPostResponse>> {
        let conn = self.conn.lock().unwrap();
        let status = status.unwrap_or("publish");
        let base_url = Self::get_base_url_with_conn(&conn);

        let mut posts = Vec::new();

        let select_fields = "SELECT ID, post_date, post_date_gmt, post_modified, post_modified_gmt, post_title, post_content, post_excerpt, post_name, post_status, post_type, guid, post_author, comment_status, ping_status, COALESCE(post_parent, 0), COALESCE(menu_order, 0), COALESCE(is_sticky, 0) FROM posts";

        let is_any_type = post_type == "any" || post_type.is_empty();
        let is_any_status = status == "any";

        let (where_clause, params_vec): (String, Vec<rusqlite::types::Value>) = match (is_any_type, is_any_status, search.filter(|s| !s.is_empty())) {
            (true, true, None) => ("".to_string(), vec![]),
            (true, false, None) => ("WHERE post_status = ?".to_string(), vec![status.to_string().into()]),
            (false, true, None) => ("WHERE post_type = ?".to_string(), vec![post_type.to_string().into()]),
            (false, false, None) => ("WHERE post_type = ? AND post_status = ?".to_string(), vec![post_type.to_string().into(), status.to_string().into()]),
            (true, true, Some(s)) => ("WHERE (post_title LIKE ? OR post_content LIKE ?)".to_string(), {
                let p = format!("%{}%", s);
                vec![p.clone().into(), p.into()]
            }),
            (true, false, Some(s)) => ("WHERE post_status = ? AND (post_title LIKE ? OR post_content LIKE ?)".to_string(), {
                let p = format!("%{}%", s);
                vec![status.to_string().into(), p.clone().into(), p.into()]
            }),
            (false, true, Some(s)) => ("WHERE post_type = ? AND (post_title LIKE ? OR post_content LIKE ?)".to_string(), {
                let p = format!("%{}%", s);
                vec![post_type.to_string().into(), p.clone().into(), p.into()]
            }),
            (false, false, Some(s)) => ("WHERE post_type = ? AND post_status = ? AND (post_title LIKE ? OR post_content LIKE ?)".to_string(), {
                let p = format!("%{}%", s);
                vec![post_type.to_string().into(), status.to_string().into(), p.clone().into(), p.into()]
            }),
        };

        let sql = if where_clause.is_empty() {
            format!(
                "{} ORDER BY is_sticky DESC, post_date DESC LIMIT ?",
                select_fields
            )
        } else {
            format!(
                "{} {} ORDER BY is_sticky DESC, post_date DESC LIMIT ?",
                select_fields, where_clause
            )
        };

        let mut stmt = conn.prepare(&sql)?;
        let mut final_params = params_vec;
        final_params.push(limit.into());

        let rows = stmt.query_map(
            rusqlite::params_from_iter(final_params.iter()),
            Self::map_post_row,
        )?;
        for r in rows {
            posts.push(r?);
        }

        // Hydrate featured media, categories, tags, author_name, template, and comments
        let mut hydrated = Vec::with_capacity(posts.len());
        for mut post in posts {
            Self::hydrate_post_relations(&conn, &mut post, &base_url);
            hydrated.push(post);
        }

        Ok(hydrated)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn query_posts(
        &self,
        post_type: &str,
        status: Option<&str>,
        search: Option<&str>,
        slug: Option<&str>,
        author: Option<i64>,
        parent: Option<i64>,
        categories: &[i64],
        tags: &[i64],
        orderby: Option<&str>,
        order: Option<&str>,
        offset: usize,
        per_page: usize,
    ) -> Result<(Vec<WpPostResponse>, usize)> {
        let all = self.get_posts(post_type, status, search, 10000)?;
        let mut filtered: Vec<WpPostResponse> = all
            .into_iter()
            .filter(|p| {
                if let Some(s) = slug.filter(|s| !s.is_empty()) {
                    if p.slug != s {
                        return false;
                    }
                }
                if let Some(a) = author {
                    if p.author != a {
                        return false;
                    }
                }
                if let Some(par) = parent {
                    if p.parent != par {
                        return false;
                    }
                }
                if !categories.is_empty() && !categories.iter().any(|c| p.categories.contains(c)) {
                    return false;
                }
                if !tags.is_empty() && !tags.iter().any(|t| p.tags.contains(t)) {
                    return false;
                }
                true
            })
            .collect();

        let is_asc = matches!(order.map(|o| o.to_lowercase()).as_deref(), Some("asc"));
        match orderby.map(|o| o.to_lowercase()).as_deref() {
            Some("title") => {
                filtered.sort_by(|a, b| {
                    let cmp = a
                        .title
                        .rendered
                        .to_lowercase()
                        .cmp(&b.title.rendered.to_lowercase());
                    if is_asc {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            Some("id") => {
                filtered.sort_by(|a, b| {
                    let cmp = a.id.cmp(&b.id);
                    if is_asc {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            Some("slug") => {
                filtered.sort_by(|a, b| {
                    let cmp = a.slug.cmp(&b.slug);
                    if is_asc {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            Some("modified") => {
                filtered.sort_by(|a, b| {
                    let cmp = a.modified.cmp(&b.modified);
                    if is_asc {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            Some("menu_order") => {
                filtered.sort_by(|a, b| {
                    let cmp = a.menu_order.cmp(&b.menu_order);
                    if is_asc {
                        cmp
                    } else {
                        cmp.reverse()
                    }
                });
            }
            _ => {
                if is_asc {
                    filtered.reverse();
                }
            }
        }

        let total = filtered.len();
        let per_page = per_page.max(1);
        let paged = filtered.into_iter().skip(offset).take(per_page).collect();
        Ok((paged, total))
    }

    pub fn get_posts_by_category_slug(
        &self,
        slug: &str,
    ) -> Result<(Option<WpCategoryResponse>, Vec<WpPostResponse>)> {
        let cats = self.get_terms("category")?;
        let cat_opt = cats.into_iter().find(|c| c.slug == slug);
        if let Some(ref cat) = cat_opt {
            let (posts, _) = self.query_posts(
                "post",
                Some("publish"),
                None,
                None,
                None,
                None,
                &[cat.id],
                &[],
                None,
                None,
                0,
                10_000,
            )?;
            Ok((cat_opt, posts))
        } else {
            Ok((None, Vec::new()))
        }
    }

    pub fn get_posts_by_tag_slug(
        &self,
        slug: &str,
    ) -> Result<(Option<WpCategoryResponse>, Vec<WpPostResponse>)> {
        let tags = self.get_terms("post_tag")?;
        let tag_opt = tags.into_iter().find(|t| t.slug == slug);
        if let Some(ref tag) = tag_opt {
            let (posts, _) = self.query_posts(
                "post",
                Some("publish"),
                None,
                None,
                None,
                None,
                &[],
                &[tag.id],
                None,
                None,
                0,
                10_000,
            )?;
            Ok((tag_opt, posts))
        } else {
            Ok((None, Vec::new()))
        }
    }

    pub fn get_posts_by_author_slug(
        &self,
        slug: &str,
    ) -> Result<(Option<WpUserDetail>, Vec<WpPostResponse>)> {
        let users = self.get_all_users()?;
        let matched = users.into_iter().find(|u| {
            u.username.eq_ignore_ascii_case(slug)
                || generated_slugify::slugify(&u.display_name) == slug
        });
        if let Some(u) = matched {
            let detail = self.get_user_detail(u.id)?;
            let (posts, _) = self.query_posts(
                "post",
                Some("publish"),
                None,
                None,
                Some(u.id),
                None,
                &[],
                &[],
                None,
                None,
                0,
                10_000,
            )?;
            Ok((detail, posts))
        } else {
            Ok((None, Vec::new()))
        }
    }

    pub fn get_post_by_id(&self, id: i64) -> Result<Option<WpPostResponse>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT ID, post_date, post_date_gmt, post_modified, post_modified_gmt, post_title, post_content, post_excerpt, post_name, post_status, post_type, guid, post_author, comment_status, ping_status, COALESCE(post_parent, 0), COALESCE(menu_order, 0), COALESCE(is_sticky, 0)
               FROM posts WHERE ID = ?1"#,
        )?;

        let mut rows = stmt.query_map(params![id], Self::map_post_row)?;
        let mut post = match rows.next() {
            Some(res) => res?,
            None => return Ok(None),
        };

        let base_url = Self::get_base_url_with_conn(&conn);
        Self::hydrate_post_relations(&conn, &mut post, &base_url);

        Ok(Some(post))
    }

    pub fn get_post_by_slug(&self, slug: &str) -> Result<Option<WpPostResponse>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT ID, post_date, post_date_gmt, post_modified, post_modified_gmt, post_title, post_content, post_excerpt, post_name, post_status, post_type, guid, post_author, comment_status, ping_status, COALESCE(post_parent, 0), COALESCE(menu_order, 0), COALESCE(is_sticky, 0)
               FROM posts
               WHERE post_name = ?1 AND post_type NOT IN ('nav_menu_item', 'revision', 'elementor_library')
                 AND post_status <> 'trash'
               ORDER BY CASE WHEN post_type IN ('page', 'post') THEN 0 ELSE 1 END,
                        CASE WHEN post_status = 'publish' THEN 0 ELSE 1 END,
                        ID DESC"#,
        )?;

        let mut rows = stmt.query_map(params![slug], Self::map_post_row)?;
        let mut post = match rows.next() {
            Some(res) => res?,
            None => return Ok(None),
        };

        let base_url = Self::get_base_url_with_conn(&conn);
        Self::hydrate_post_relations(&conn, &mut post, &base_url);

        Ok(Some(post))
    }

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

    pub fn create_post(&self, req: &CreatePostRequest) -> Result<WpPostResponse> {
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
        // `publish` dated ahead becomes `future` (scheduled), like WordPress.
        let (post_date, status) = super::scheduling::schedule_for_new_post(
            &req.status,
            &req.post_type,
            req.date.as_deref(),
            req.date_gmt.as_deref(),
            &now,
        );

        conn.execute(
            r#"INSERT INTO posts (post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type, post_parent, menu_order)
               VALUES (?1, ?11, ?11, ?3, ?4, ?5, ?6, ?7, ?2, ?2, '', ?8, ?9, ?10)"#,
            params![author, now, req.content, req.title, excerpt, status, slug, req.post_type, parent, menu_order, post_date],
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

    pub fn update_post(&self, id: i64, req: &UpdatePostRequest) -> Result<Option<WpPostResponse>> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        // Capture old slug & status before updating so we can auto-create a 301 redirect if slug changes on a published post
        let old_slug_and_status: Option<(String, String)> = tx
            .query_row(
                "SELECT post_name, post_status FROM posts WHERE ID = ?1",
                params![id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();

        if let Some(title) = &req.title {
            tx.execute(
                "UPDATE posts SET post_title = ?1, post_modified = ?2 WHERE ID = ?3",
                params![title, now, id],
            )?;
        }
        if let Some(content) = &req.content {
            tx.execute(
                "UPDATE posts SET post_content = ?1, post_modified = ?2 WHERE ID = ?3",
                params![content, now, id],
            )?;
        }
        if let Some(status) = &req.status {
            tx.execute(
                "UPDATE posts SET post_status = ?1, post_modified = ?2 WHERE ID = ?3",
                params![status, now, id],
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
        // A new date and/or status can (un)schedule the post, like WordPress.
        let new_date =
            super::scheduling::requested_post_date(req.date.as_deref(), req.date_gmt.as_deref());
        super::scheduling::apply_schedule_on_update(
            &tx,
            id,
            req.status.is_some(),
            new_date.as_deref(),
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
    pub fn duplicate_post(&self, source_id: i64) -> Result<Option<WpPostResponse>> {
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

    pub fn trash_post(&self, id: i64) -> Result<Option<WpPostResponse>> {
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
        let _ = conn.execute("DELETE FROM postmeta WHERE post_id = ?1", params![id]);
        let _ = conn.execute(
            "DELETE FROM term_relationships WHERE object_id = ?1",
            params![id],
        );
        let rows = conn.execute("DELETE FROM posts WHERE ID = ?1", params![id])?;
        Ok(rows > 0)
    }

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

        if let Some(html) = rendered_html {
            tx.execute(
                "UPDATE posts SET layout_json = ?1, post_content = ?2 WHERE ID = ?3",
                params![layout_json, html, post_id],
            )?;
        } else {
            tx.execute(
                "UPDATE posts SET layout_json = ?1 WHERE ID = ?2",
                params![layout_json, post_id],
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

        tx.commit()?;
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
        Ok(())
    }
}

impl crate::ports::output::PostRepository for Database {
    fn list_posts(
        &self,
        post_type: &str,
        status: Option<&str>,
        search: Option<&str>,
        limit: i64,
    ) -> DomainResult<Vec<WpPostResponse>> {
        // The MCP stdio server reads through this port without the HTTP middleware.
        let _ = self.publish_due_scheduled_posts();
        self.get_posts(post_type, status, search, limit)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn find_post_by_id(&self, id: i64) -> DomainResult<Option<WpPostResponse>> {
        let _ = self.publish_due_scheduled_posts();
        self.get_post_by_id(id)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn find_post_by_slug(&self, slug: &str) -> DomainResult<Option<WpPostResponse>> {
        let _ = self.publish_due_scheduled_posts();
        self.get_post_by_slug(slug)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn create_post(&self, req: &CreatePostRequest) -> DomainResult<WpPostResponse> {
        Database::create_post(self, req).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn update_post(
        &self,
        id: i64,
        req: &UpdatePostRequest,
    ) -> DomainResult<Option<WpPostResponse>> {
        Database::update_post(self, id, req).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn duplicate_post(&self, source_id: i64) -> DomainResult<Option<WpPostResponse>> {
        Database::duplicate_post(self, source_id).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn trash_post(&self, id: i64) -> DomainResult<Option<WpPostResponse>> {
        Database::trash_post(self, id).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn delete_post(&self, id: i64) -> DomainResult<bool> {
        Database::delete_post(self, id).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn get_post_layout(&self, post_id: i64) -> DomainResult<Option<String>> {
        Database::get_post_layout(self, post_id).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn save_post_layout(
        &self,
        post_id: i64,
        layout_json: &str,
        compiled_html: Option<&str>,
    ) -> DomainResult<()> {
        Database::save_post_layout(self, post_id, layout_json, compiled_html)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn delete_post_layout(&self, post_id: i64) -> DomainResult<()> {
        Database::delete_post_layout(self, post_id).map_err(|e| DomainError::Storage(e.to_string()))
    }
}
