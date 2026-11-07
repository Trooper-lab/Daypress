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
        post: &mut DaypressPostResponse,
        base_url: &str,
    ) {
        let base_trimmed = base_url.trim_end_matches('/');
        let is_front_page = post.post_type == "page" && {
            let show_on_front: String = conn
                .query_row(
                    "SELECT option_value FROM options WHERE option_name = 'show_on_front' LIMIT 1",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or_else(|_| "posts".to_string());
            let page_on_front: i64 = conn
                .query_row(
                    "SELECT option_value FROM options WHERE option_name = 'page_on_front' LIMIT 1",
                    [],
                    |r| r.get::<_, String>(0),
                )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            show_on_front == "page" && page_on_front > 0 && post.id == page_on_front
        };
        let (lang_code, translation_group_id, translations) =
            Self::get_post_translation_info_with_conn(conn, post.id);
        post.lang_code = lang_code.clone();
        post.translation_group_id = translation_group_id;
        post.translations = translations;
        post.link = Self::format_localized_post_url_with_conn(
            conn,
            base_trimmed,
            post.id,
            &post.post_type,
            &post.slug,
            &lang_code,
            is_front_page,
        );

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
        post.acf = Self::resolve_post_structured_fields_with_conn(
            conn,
            &post.post_type,
            &meta_obj,
            base_url,
        );
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

        let (coauthors, formatted_byline) =
            Self::get_post_coauthors_with_conn(conn, post.id, post.author);
        post.coauthors = coauthors;
        post.author_name = formatted_byline;

        if let Some(sched_rev) = Self::get_scheduled_revision_for_post_with_conn(conn, post.id) {
            post.scheduled_revision_id = Some(sched_rev.revision_post_id);
            post.scheduled_revision_date = Some(sched_rev.scheduled_date);
        } else {
            post.scheduled_revision_id = None;
            post.scheduled_revision_date = None;
        }

        post.template = conn
            .query_row(
                "SELECT meta_value FROM postmeta WHERE post_id = ?1 AND meta_key = '_wp_page_template'",
                params![post.id],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .unwrap_or_default();

        post.password = conn
            .query_row(
                "SELECT COALESCE(post_password, '') FROM posts WHERE ID = ?1",
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
                    Ok(DaypressCommentResponse {
                        id: cid,
                        post: cpost,
                        parent: cparent,
                        author: uid,
                        author_name: cauthor,
                        author_url: curl,
                        date: cdate.replace(' ', "T"),
                        date_gmt: cdate_gmt.replace(' ', "T"),
                        content: DaypressRenderedText {
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

    pub(crate) fn map_post_row(row: &rusqlite::Row) -> rusqlite::Result<DaypressPostResponse> {
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

        Ok(DaypressPostResponse {
            id,
            date: date.replace(' ', "T"),
            date_gmt: date_gmt.replace(' ', "T"),
            guid: DaypressGuid { rendered: guid },
            modified: modified.replace(' ', "T"),
            modified_gmt: modified_gmt.replace(' ', "T"),
            slug: slug.clone(),
            status,
            post_type: ptype,
            link: String::new(),
            title: DaypressRenderedText {
                rendered: title_raw,
                protected: false,
            },
            content: DaypressRenderedText {
                rendered: content_raw,
                protected: false,
            },
            excerpt: DaypressRenderedText {
                rendered: excerpt_raw,
                protected: false,
            },
            author,
            featured_media: 0,
            comment_status,
            ping_status,
            sticky: is_sticky == 1,
            password: String::new(),
            template: String::new(),
            format: "standard".to_string(),
            meta: serde_json::json!({}),
            acf: serde_json::json!({}),
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
            lang_code: "en".to_string(),
            translation_group_id: String::new(),
            translations: std::collections::HashMap::new(),
            coauthors: Vec::new(),
            scheduled_revision_id: None,
            scheduled_revision_date: None,
        })
    }

    pub fn get_posts(
        &self,
        post_type: &str,
        status: Option<&str>,
        search: Option<&str>,
        limit: i64,
    ) -> Result<Vec<DaypressPostResponse>> {
        let conn = self.conn.lock().unwrap();
        let status = status.unwrap_or("publish");
        let base_url = Self::get_base_url_with_conn(&conn);

        let mut posts = Vec::new();

        let select_fields = "SELECT ID, post_date, post_date_gmt, post_modified, post_modified_gmt, post_title, post_content, post_excerpt, post_name, post_status, post_type, guid, post_author, comment_status, ping_status, COALESCE(post_parent, 0), COALESCE(menu_order, 0), COALESCE(is_sticky, 0) FROM posts";

        let is_any_type = post_type == "any" || post_type.is_empty();
        let is_any_status = status == "any";

        let (where_clause, params_vec): (String, Vec<rusqlite::types::Value>) = match (is_any_type, is_any_status, search.filter(|s| !s.is_empty())) {
            (true, true, None) => ("WHERE post_type != 'revision'".to_string(), vec![]),
            (true, false, None) => ("WHERE post_type != 'revision' AND post_status = ?".to_string(), vec![status.to_string().into()]),
            (false, true, None) => ("WHERE post_type = ?".to_string(), vec![post_type.to_string().into()]),
            (false, false, None) => ("WHERE post_type = ? AND post_status = ?".to_string(), vec![post_type.to_string().into(), status.to_string().into()]),
            (true, true, Some(s)) => ("WHERE post_type != 'revision' AND (post_title LIKE ? OR post_content LIKE ?)".to_string(), {
                let p = format!("%{}%", s);
                vec![p.clone().into(), p.into()]
            }),
            (true, false, Some(s)) => ("WHERE post_type != 'revision' AND post_status = ? AND (post_title LIKE ? OR post_content LIKE ?)".to_string(), {
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
    ) -> Result<(Vec<DaypressPostResponse>, usize)> {
        let all = self.get_posts(post_type, status, search, 10000)?;
        let mut filtered: Vec<DaypressPostResponse> = all
            .into_iter()
            .filter(|p| {
                if let Some(s) = slug.filter(|s| !s.is_empty()) {
                    if p.slug != s {
                        return false;
                    }
                }
                if let Some(a) = author {
                    if p.author != a && !p.coauthors.iter().any(|ca| ca.id == a) {
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
    ) -> Result<(Option<DaypressCategoryResponse>, Vec<DaypressPostResponse>)> {
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
    ) -> Result<(Option<DaypressCategoryResponse>, Vec<DaypressPostResponse>)> {
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
    ) -> Result<(Option<DaypressUserDetail>, Vec<DaypressPostResponse>)> {
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

    pub fn get_post_by_id(&self, id: i64) -> Result<Option<DaypressPostResponse>> {
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

    pub fn get_post_by_slug(&self, slug: &str) -> Result<Option<DaypressPostResponse>> {
        let conn = self.conn.lock().unwrap();
        let def_lang = Self::get_default_language_code_with_conn(&conn);
        let mut stmt = conn.prepare(
            r#"SELECT p.ID, p.post_date, p.post_date_gmt, p.post_modified, p.post_modified_gmt, p.post_title, p.post_content, p.post_excerpt, p.post_name, p.post_status, p.post_type, p.guid, p.post_author, p.comment_status, p.ping_status, COALESCE(p.post_parent, 0), COALESCE(p.menu_order, 0), COALESCE(p.is_sticky, 0)
               FROM posts p
               LEFT JOIN daypress_translations tr ON tr.object_type = 'post' AND tr.object_id = p.ID
               WHERE p.post_name = ?1 AND p.post_type NOT IN ('nav_menu_item', 'revision', 'elementor_library')
                 AND p.post_status <> 'trash'
               ORDER BY CASE WHEN LOWER(COALESCE(NULLIF(tr.lang_code, ''), ?2)) = LOWER(?2) THEN 0 ELSE 1 END,
                        CASE WHEN p.post_type IN ('page', 'post') THEN 0 ELSE 1 END,
                        CASE WHEN p.post_status = 'publish' THEN 0 ELSE 1 END,
                        p.ID DESC"#,
        )?;

        let mut rows = stmt.query_map(params![slug, def_lang], Self::map_post_row)?;
        let mut post = match rows.next() {
            Some(res) => res?,
            None => return Ok(None),
        };

        let base_url = Self::get_base_url_with_conn(&conn);
        Self::hydrate_post_relations(&conn, &mut post, &base_url);

        Ok(Some(post))
    }
}

impl crate::ports::output::PostRepository for Database {
    fn list_posts(
        &self,
        post_type: &str,
        status: Option<&str>,
        search: Option<&str>,
        limit: i64,
    ) -> DomainResult<Vec<DaypressPostResponse>> {
        // The MCP stdio server reads through this port without the HTTP middleware.
        let _ = self.publish_due_scheduled_posts();
        self.get_posts(post_type, status, search, limit)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn find_post_by_id(&self, id: i64) -> DomainResult<Option<DaypressPostResponse>> {
        let _ = self.publish_due_scheduled_posts();
        self.get_post_by_id(id)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn find_post_by_slug(&self, slug: &str) -> DomainResult<Option<DaypressPostResponse>> {
        let _ = self.publish_due_scheduled_posts();
        self.get_post_by_slug(slug)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn create_post(&self, req: &CreatePostRequest) -> DomainResult<DaypressPostResponse> {
        Database::create_post(self, req).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn update_post(
        &self,
        id: i64,
        req: &UpdatePostRequest,
    ) -> DomainResult<Option<DaypressPostResponse>> {
        Database::update_post(self, id, req).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn duplicate_post(&self, source_id: i64) -> DomainResult<Option<DaypressPostResponse>> {
        Database::duplicate_post(self, source_id).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn trash_post(&self, id: i64) -> DomainResult<Option<DaypressPostResponse>> {
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
