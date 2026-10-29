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
    fn rewrite_upload_urls(text: &str, media_rewrites: &[(String, String)]) -> String {
        if text.is_empty() {
            return String::new();
        }
        let mut out = text.to_string();
        for (old_url, new_url) in media_rewrites {
            if !old_url.is_empty() && old_url != new_url {
                if out.contains(old_url) {
                    out = out.replace(old_url, new_url);
                }
                let old_esc = old_url.replace('/', "\\/");
                let new_esc = new_url.replace('/', "\\/");
                if out.contains(&old_esc) {
                    out = out.replace(&old_esc, &new_esc);
                }
            }
        }
        if out.contains("/wp-content/uploads/") {
            if let Ok(re) = regex::Regex::new(
                r#"(?:https?://[^"'\s\\<>]+)?/wp-content/uploads/(?:\d{4}/\d{2}/)?([a-zA-Z0-9._\-]+)"#,
            ) {
                out = re.replace_all(&out, "/uploads/$1").into_owned();
            }
        }
        if out.contains("\\/wp-content\\/uploads\\/") {
            if let Ok(re_esc) = regex::Regex::new(
                r#"(?:https?:\\/\\/[^"'\s<>]+)?\\/wp-content\\/uploads\\/(?:\d{4}\\/\d{2}\\/)?([a-zA-Z0-9._\-]+)"#,
            ) {
                out = re_esc.replace_all(&out, "\\/uploads\\/$1").into_owned();
            }
        }
        out
    }

    fn normalize_source_menu_url(
        url: &str,
        post_id_map: &std::collections::HashMap<i64, i64>,
    ) -> String {
        let trimmed = url.trim();
        if trimmed.is_empty() {
            return "/".to_string();
        }
        let rel = if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            if let Some(idx) = trimmed
                .find("://")
                .and_then(|i| trimmed[i + 3..].find('/').map(|j| i + 3 + j))
            {
                let path = &trimmed[idx..];
                if path.is_empty() {
                    "/".to_string()
                } else {
                    path.to_string()
                }
            } else {
                "/".to_string()
            }
        } else {
            trimmed.to_string()
        };

        for prefix in ["/?page_id=", "/?p="] {
            if let Some(id_str) = rel.strip_prefix(prefix) {
                if let Ok(old_id) = id_str.trim_end_matches('/').parse::<i64>() {
                    let mapped = post_id_map.get(&old_id).copied().unwrap_or(old_id);
                    return format!("{}{}", prefix, mapped);
                }
            }
        }
        rel
    }

    pub fn import_bundle(&self, req: &ImportBundleRequest) -> Result<ImportBundleResponse> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn.transaction()?;

        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        let mut cat_id_map: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
        let mut tag_id_map: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
        let mut post_id_map: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
        let mut comment_id_map: std::collections::HashMap<i64, i64> =
            std::collections::HashMap::new();

        // When importing a full WordPress site bundle, clean out any prior seed/test posts, comments, and terms
        // so the migrated site starts with a 100% clean database matching the source WordPress site.
        if req.site_settings.is_some() && !req.posts.is_empty() {
            let _ = tx.execute("DELETE FROM term_relationships;", []);
            let _ = tx.execute("DELETE FROM postmeta;", []);
            let _ = tx.execute("DELETE FROM posts;", []);
            let _ = tx.execute("DELETE FROM comments;", []);
            let _ = tx.execute("DELETE FROM term_taxonomy WHERE term_id > 1;", []);
            let _ = tx.execute("DELETE FROM terms WHERE term_id > 1;", []);
            let _ = tx.execute(
                "INSERT INTO options (option_name, option_value) VALUES ('daypress_site_migrated', '1')
                 ON CONFLICT(option_name) DO UPDATE SET option_value = '1'",
                [],
            );
        }

        // 0. Elementor Kit Global Styles
        let mut kit_opt: Option<daypress_ast::ElementorKit> = None;
        if let Some(ref kit_val) = req.elementor_kit {
            if !kit_val.is_null() {
                if let Ok(kit_str) = serde_json::to_string(kit_val) {
                    tx.execute(
                        "INSERT INTO options (option_name, option_value) VALUES ('daypress_elementor_kit', ?1)
                         ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                        params![kit_str],
                    )?;
                }
                kit_opt = serde_json::from_value::<daypress_ast::ElementorKit>(kit_val.clone())
                    .ok()
                    .or_else(|| {
                        kit_val.get("settings").and_then(|s| {
                            serde_json::from_value::<daypress_ast::ElementorKit>(s.clone()).ok()
                        })
                    });
            }
        }

        // Theme mods, compiled CSS, and Header/Footer templates
        if let Some(tm) = &req.theme_mods {
            if let Some(t) = &tm.active_theme {
                for key in ["daypress_imported_theme", "stylesheet", "template"] {
                    tx.execute(
                        "INSERT INTO options (option_name, option_value) VALUES (?1, ?2)
                         ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                        params![key, t],
                    )?;
                }
            }
            let mut final_css = String::new();
            if let Some(css) = &tm.custom_css {
                final_css.push_str(css);
            }
            if let Some(comp) = &tm.compiled_css {
                if !comp.trim().is_empty() {
                    if !final_css.is_empty() {
                        final_css.push_str("\n/* --- COMPILED BUILDER & THEME STYLESHEET --- */\n");
                    }
                    final_css.push_str(comp);
                }
            }
            if !final_css.trim().is_empty() {
                for key in ["daypress_custom_css", "custom_css"] {
                    tx.execute(
                        "INSERT INTO options (option_name, option_value) VALUES (?1, ?2)
                         ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                        params![key, final_css],
                    )?;
                }
            }

            if let Some(hdr) = &tm.header_html {
                if !hdr.trim().is_empty() {
                    let _ = crate::importer::templates::upsert_theme_template(
                        &tx,
                        "Theme Header",
                        "header",
                        hdr,
                        None,
                        100,
                    );
                }
            }
            if let Some(ftr) = &tm.footer_html {
                if !ftr.trim().is_empty() {
                    let _ = crate::importer::templates::upsert_theme_template(
                        &tx,
                        "Theme Footer",
                        "footer",
                        ftr,
                        None,
                        100,
                    );
                }
            }
        }

        // 1. Users
        let mut user_count = 0;
        for u in &req.users {
            let username = u.username.trim();
            if username.is_empty() {
                continue;
            }
            let nicename = if u.nicename.trim().is_empty() {
                generated_slugify::slugify(username)
            } else {
                u.nicename.trim().to_string()
            };
            let display_name = if u.display_name.trim().is_empty() {
                username.to_string()
            } else {
                u.display_name.trim().to_string()
            };
            let registered = if u.registered.trim().is_empty() {
                &now
            } else {
                u.registered.trim()
            };
            let role = if u.role.trim().is_empty() {
                "Subscriber"
            } else {
                u.role.trim()
            };
            // WordPress hashes are not carried over; imported accounts get an unknown random
            // password until an administrator sets one for them.
            let placeholder_pass =
                crate::auth::hash_user_password(&crate::auth::generate_random_password());

            tx.execute(
                r#"INSERT OR IGNORE INTO users (user_login, user_pass, user_nicename, user_email, user_url, user_registered, display_name)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"#,
                params![
                    username,
                    placeholder_pass,
                    nicename,
                    u.email.trim(),
                    u.url.trim(),
                    registered,
                    display_name
                ],
            )?;

            if let Ok(uid) = tx.query_row(
                "SELECT ID FROM users WHERE user_login = ?1",
                params![username],
                |r| r.get::<_, i64>(0),
            ) {
                let _ = tx.execute(
                    "INSERT OR IGNORE INTO usermeta (user_id, meta_key, meta_value) VALUES (?1, 'role', ?2)",
                    params![uid, role],
                );
                user_count += 1;
            }
        }

        // 2. Categories
        let mut cat_count = 0;
        let mut pending_cat_parents: Vec<(i64, i64)> = Vec::new();
        for c in &req.categories {
            let slug = if c.slug.trim().is_empty() {
                generated_slugify::slugify(&c.name)
            } else {
                c.slug.trim().to_string()
            };

            let existing_id: Option<i64> = tx
                .query_row(
                    "SELECT term_id FROM terms WHERE slug = ?1",
                    params![slug],
                    |r| r.get(0),
                )
                .ok();

            let term_id = match existing_id {
                Some(id) => {
                    tx.execute(
                        "UPDATE terms SET name = ?1 WHERE term_id = ?2",
                        params![c.name, id],
                    )?;
                    id
                }
                None => {
                    tx.execute(
                        "INSERT INTO terms (name, slug) VALUES (?1, ?2)",
                        params![c.name, slug],
                    )?;
                    tx.last_insert_rowid()
                }
            };

            if let Some(old_id) = c.id {
                cat_id_map.insert(old_id, term_id);
            }
            if let Some(parent_id) = c.parent {
                if parent_id > 0 {
                    pending_cat_parents.push((term_id, parent_id));
                }
            }

            let existing_tt: Option<i64> = tx.query_row(
                "SELECT term_taxonomy_id FROM term_taxonomy WHERE term_id = ?1 AND taxonomy = 'category'",
                params![term_id],
                |r| r.get(0),
            ).ok();

            match existing_tt {
                Some(tt_id) => {
                    tx.execute(
                        "UPDATE term_taxonomy SET description = ?1 WHERE term_taxonomy_id = ?2",
                        params![c.description, tt_id],
                    )?;
                }
                None => {
                    tx.execute("INSERT INTO term_taxonomy (term_id, taxonomy, description) VALUES (?1, 'category', ?2)", params![term_id, c.description])?;
                }
            }
            cat_count += 1;
        }

        for (new_term_id, old_parent_id) in pending_cat_parents {
            let mapped_parent = cat_id_map
                .get(&old_parent_id)
                .copied()
                .unwrap_or(old_parent_id);
            tx.execute(
                "UPDATE term_taxonomy SET parent = ?1 WHERE term_id = ?2 AND taxonomy = 'category'",
                params![mapped_parent, new_term_id],
            )?;
        }

        // 3. Tags
        let mut tag_count = 0;
        for t in &req.tags {
            let slug = if t.slug.trim().is_empty() {
                generated_slugify::slugify(&t.name)
            } else {
                t.slug.trim().to_string()
            };

            let existing_id: Option<i64> = tx
                .query_row(
                    "SELECT term_id FROM terms WHERE slug = ?1",
                    params![slug],
                    |r| r.get(0),
                )
                .ok();

            let term_id = match existing_id {
                Some(id) => {
                    tx.execute(
                        "UPDATE terms SET name = ?1 WHERE term_id = ?2",
                        params![t.name, id],
                    )?;
                    id
                }
                None => {
                    tx.execute(
                        "INSERT INTO terms (name, slug) VALUES (?1, ?2)",
                        params![t.name, slug],
                    )?;
                    tx.last_insert_rowid()
                }
            };

            if let Some(old_id) = t.id {
                tag_id_map.insert(old_id, term_id);
            }

            let existing_tt: Option<i64> = tx.query_row(
                "SELECT term_taxonomy_id FROM term_taxonomy WHERE term_id = ?1 AND taxonomy = 'post_tag'",
                params![term_id],
                |r| r.get(0),
            ).ok();

            match existing_tt {
                Some(tt_id) => {
                    tx.execute(
                        "UPDATE term_taxonomy SET description = ?1 WHERE term_taxonomy_id = ?2",
                        params![t.description, tt_id],
                    )?;
                }
                None => {
                    tx.execute("INSERT INTO term_taxonomy (term_id, taxonomy, description) VALUES (?1, 'post_tag', ?2)", params![term_id, t.description])?;
                }
            }
            tag_count += 1;
        }

        // 4. Media
        let mut media_count = 0;
        let mut media_rewrites: Vec<(String, String)> = Vec::new();
        for m in &req.media {
            let safe_file_name = crate::media::sanitize_filename(&m.file_name)
                .or_else(|| crate::media::sanitize_filename(&m.source_url))
                .unwrap_or_else(|| format!("media_{}.png", media_count + 1));
            let local_upload_url = format!("/uploads/{}", safe_file_name);

            let final_url = if let Some(b64) = &m.base64_data {
                if !b64.is_empty() {
                    if let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64.trim())
                    {
                        let _ = fs::create_dir_all("uploads");
                        let file_path = Path::new("uploads").join(&safe_file_name);
                        let _ = fs::write(&file_path, &bytes);
                        crate::media::spawn_background_image_processing(file_path);
                        local_upload_url.clone()
                    } else {
                        local_upload_url.clone()
                    }
                } else {
                    local_upload_url.clone()
                }
            } else if m.source_url.contains("/wp-content/uploads/") || !m.file_name.is_empty() {
                local_upload_url.clone()
            } else {
                m.source_url.clone()
            };

            if let Some(ref orig) = m.original_url {
                if !orig.is_empty() {
                    media_rewrites.push((orig.clone(), local_upload_url.clone()));
                }
            }
            if !m.source_url.is_empty() && m.source_url != local_upload_url {
                media_rewrites.push((m.source_url.clone(), local_upload_url.clone()));
            }

            let mime = if m.mime_type.trim().is_empty() {
                crate::wxr::infer_mime_type_from_filename(&safe_file_name).to_string()
            } else {
                m.mime_type.clone()
            };

            let existing_id: Option<i64> = tx.query_row(
                "SELECT ID FROM posts WHERE (post_name = ?1 OR guid = ?2) AND post_type = 'attachment'",
                params![safe_file_name, final_url],
                |r| r.get(0),
            ).ok();

            let media_id = match existing_id {
                Some(id) => {
                    tx.execute(
                        "UPDATE posts SET post_title = ?1, guid = ?2, post_mime_type = ?3 WHERE ID = ?4",
                        params![m.title, final_url, mime, id],
                    )?;
                    id
                }
                None => {
                    tx.execute(
                        r#"INSERT INTO posts (post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type, post_mime_type)
                           VALUES (1, ?1, ?1, '', ?2, '', 'inherit', ?3, ?1, ?1, ?4, 'attachment', ?5)"#,
                        params![now, m.title, safe_file_name, final_url, mime],
                    )?;
                    tx.last_insert_rowid()
                }
            };

            if let Some(old_id) = m.id {
                post_id_map.insert(old_id, media_id);
            }

            let _ = tx.execute(
                "INSERT OR REPLACE INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_attached_file', ?2)",
                params![media_id, safe_file_name],
            );
            if let Some(ref alt) = m.alt_text {
                let _ = tx.execute(
                    "INSERT OR REPLACE INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_attachment_image_alt', ?2)",
                    params![media_id, alt],
                );
            }
            if let Some(size) = m.file_size {
                let _ = tx.execute(
                    "INSERT OR REPLACE INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_filesize', ?2)",
                    params![media_id, size.to_string()],
                );
            }
            media_count += 1;
        }

        // 5. Posts and Pages
        if !req.posts.is_empty() {
            let incoming_slugs: std::collections::HashSet<&str> =
                req.posts.iter().map(|p| p.slug.as_str()).collect();
            for default_slug in ["hello-world", "sample-page"] {
                if !incoming_slugs.contains(default_slug) {
                    let _ = tx.execute(
                        "DELETE FROM posts WHERE post_name = ?1 AND post_content LIKE '%Welcome to WordPress%' OR (post_name = ?1 AND ID <= 2)",
                        params![default_slug],
                    );
                }
            }
            let _ = tx.execute(
                "DELETE FROM comments WHERE comment_author_email = 'wapuu@wordpress.example'",
                [],
            );
        }
        let mut post_count = 0;
        let mut page_count = 0;
        let mut form_count = 0;
        let mut redirect_count = 0;
        let mut converged_plugins: Vec<String> = Vec::new();
        let mut seo_meta_converged = false;
        let mut form_id_map: std::collections::HashMap<i64, i64> = std::collections::HashMap::new();
        let mut first_migrated_form_id: Option<i64> = None;
        let mut pending_post_parents: Vec<(i64, i64)> = Vec::new();

        let source_site_title = req
            .site_settings
            .as_ref()
            .and_then(|s| s.title.as_deref())
            .unwrap_or("Daypress");

        // 4b. Pre-import explicit forms from bundle (`req.forms`) and any CF7/WPForms custom post types in `req.posts`
        for f in &req.forms {
            let (fields, mut settings) = if let Some(ref sj) = f.schema_json {
                let parsed_fields: Vec<FormFieldSchema> = serde_json::from_value(sj.clone())
                    .unwrap_or_else(|_| crate::convergence::default_contact_fields());
                (parsed_fields, FormSettings::default())
            } else {
                let mut meta = std::collections::HashMap::new();
                meta.insert("_form".to_string(), f.raw_form.clone());
                crate::convergence::parse_cf7_or_wpforms_to_schema(
                    "wpcf7_contact_form",
                    &f.title,
                    &f.raw_form,
                    &meta,
                )
                .unwrap_or_else(|| {
                    (
                        crate::convergence::default_contact_fields(),
                        FormSettings::default(),
                    )
                })
            };
            if !f.recipient_email.trim().is_empty() {
                settings.recipient_email = f.recipient_email.trim().to_string();
            }
            let schema_json = serde_json::to_string(&fields).unwrap_or_else(|_| "[]".to_string());
            let settings_json =
                serde_json::to_string(&settings).unwrap_or_else(|_| "{}".to_string());
            let title = if f.title.trim().is_empty() {
                "Migrated Contact Form"
            } else {
                f.title.trim()
            };
            tx.execute(
                "INSERT INTO daypress_forms (title, schema_json, settings_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
                params![title, schema_json, settings_json, now],
            )?;
            let new_fid = tx.last_insert_rowid();
            if first_migrated_form_id.is_none() {
                first_migrated_form_id = Some(new_fid);
            }
            if let Some(old_fid) = f.id {
                form_id_map.insert(old_fid, new_fid);
            }
            form_count += 1;
        }

        // Also scan `req.posts` for `wpcf7_contact_form` or `wpforms` CPTs first so `form_id_map` is ready before content shortcode rewriting
        for p in &req.posts {
            if let Some((fields, settings)) = crate::convergence::parse_cf7_or_wpforms_to_schema(
                &p.post_type,
                &p.title,
                &p.content,
                &p.postmeta,
            ) {
                let schema_json =
                    serde_json::to_string(&fields).unwrap_or_else(|_| "[]".to_string());
                let settings_json =
                    serde_json::to_string(&settings).unwrap_or_else(|_| "{}".to_string());
                let title = if p.title.trim().is_empty() {
                    "Migrated Form"
                } else {
                    p.title.trim()
                };
                tx.execute(
                    "INSERT INTO daypress_forms (title, schema_json, settings_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
                    params![title, schema_json, settings_json, now],
                )?;
                let new_fid = tx.last_insert_rowid();
                if first_migrated_form_id.is_none() {
                    first_migrated_form_id = Some(new_fid);
                }
                if let Some(old_fid) = p.id {
                    form_id_map.insert(old_fid, new_fid);
                }
                form_count += 1;
            }
        }
        if form_count > 0 {
            let _ = crate::convergence::activate_builtin_plugin_tx(&tx, "daypress-forms", None);
            converged_plugins.push(format!(
                "Contact Form 7 / WPForms ({form_count} forms) -> Daypress Forms Studio"
            ));
        }

        for p in &req.posts {
            if matches!(p.post_type.as_str(), "wpcf7_contact_form" | "wpforms") {
                continue;
            }
            let p_date = p.date.as_deref().unwrap_or(&now);
            let p_mod = p.modified.as_deref().unwrap_or(p_date);
            let slug = if p.slug.trim().is_empty() {
                generated_slugify::slugify(&p.title)
            } else {
                p.slug.trim().to_string()
            };

            let url_rewritten = Self::rewrite_upload_urls(&p.content, &media_rewrites);
            let rewritten_content = crate::convergence::rewrite_legacy_form_shortcodes(
                &url_rewritten,
                &form_id_map,
                first_migrated_form_id,
            );
            let mut rewritten_meta: std::collections::HashMap<String, String> =
                std::collections::HashMap::with_capacity(p.postmeta.len());
            for (mk, mv) in &p.postmeta {
                let meta_url_rewritten = Self::rewrite_upload_urls(mv, &media_rewrites);
                rewritten_meta.insert(
                    mk.clone(),
                    crate::convergence::rewrite_legacy_form_shortcodes(
                        &meta_url_rewritten,
                        &form_id_map,
                        first_migrated_form_id,
                    ),
                );
            }
            if crate::convergence::converge_post_seo_meta(
                &mut rewritten_meta,
                &p.title,
                source_site_title,
            ) {
                seo_meta_converged = true;
            }

            let mut final_content = rewritten_content.clone();
            let mut layout_json_str: Option<String> = None;

            if let Some(mut layout) =
                daypress_ast::transpile_content(&rewritten_content, &rewritten_meta)
            {
                if layout.global_styles.is_none() {
                    if let Some(ref kit) = kit_opt {
                        layout.global_styles = serde_json::to_value(kit).ok();
                    } else if let Some(ref kv) = req.elementor_kit {
                        if !kv.is_null() {
                            layout.global_styles = Some(kv.clone());
                        }
                    }
                }
                let rendered_html = daypress_ast::render_layout_to_html(&layout);
                if rewritten_content.trim().is_empty()
                    || rewritten_content.contains("[et_pb_")
                    || rewritten_meta.contains_key("_elementor_data")
                    || !rendered_html.trim().is_empty()
                {
                    final_content = rendered_html;
                }
                if let Ok(lj) = serde_json::to_string(&layout) {
                    layout_json_str = Some(lj);
                }
            }

            let excerpt = if p.excerpt.is_empty() {
                if final_content.chars().count() > 140 {
                    let prefix: String = final_content.chars().take(140).collect();
                    format!("{}...", prefix)
                } else {
                    final_content.clone()
                }
            } else {
                p.excerpt.clone()
            };

            let author = p.author.unwrap_or(1);
            let comment_status = p.comment_status.as_deref().unwrap_or("open");
            let ping_status = p.ping_status.as_deref().unwrap_or("open");
            let parent = p.parent.unwrap_or(0);
            let menu_order = p.menu_order.unwrap_or(0);

            let existing_id: Option<i64> = tx
                .query_row(
                    "SELECT ID FROM posts WHERE post_name = ?1 AND post_type = ?2",
                    params![slug, p.post_type],
                    |r| r.get(0),
                )
                .ok();

            let post_id = match existing_id {
                Some(id) => {
                    tx.execute(
                        "UPDATE posts SET post_author = ?1, post_title = ?2, post_content = ?3, post_excerpt = ?4, post_status = ?5, post_date = ?6, post_modified = ?7, comment_status = ?8, ping_status = ?9, post_parent = ?10, menu_order = ?11, layout_json = ?12 WHERE ID = ?13",
                        params![
                            author,
                            p.title,
                            final_content,
                            excerpt,
                            p.status,
                            p_date,
                            p_mod,
                            comment_status,
                            ping_status,
                            parent,
                            menu_order,
                            layout_json_str,
                            id
                        ],
                    )?;
                    id
                }
                None => {
                    tx.execute(
                        r#"INSERT INTO posts (post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, comment_status, ping_status, post_name, post_modified, post_modified_gmt, guid, post_type, post_parent, menu_order, layout_json)
                           VALUES (?1, ?2, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10, '', ?11, ?12, ?13, ?14)"#,
                        params![
                            author,
                            p_date,
                            final_content,
                            p.title,
                            excerpt,
                            p.status,
                            comment_status,
                            ping_status,
                            slug,
                            p_mod,
                            p.post_type,
                            parent,
                            menu_order,
                            layout_json_str
                        ],
                    )?;
                    let id = tx.last_insert_rowid();
                    let guid = format!("http://localhost:8090/?p={}", id);
                    let _ = tx.execute(
                        "UPDATE posts SET guid = ?1 WHERE ID = ?2",
                        params![guid, id],
                    );
                    id
                }
            };

            if let Some(old_id) = p.id {
                post_id_map.insert(old_id, post_id);
            }
            if parent > 0 {
                pending_post_parents.push((post_id, parent));
            }

            // Persist all postmeta
            for (mk, mv) in &rewritten_meta {
                let _ = tx.execute(
                    "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = ?2",
                    params![post_id, mk],
                );
                let _ = tx.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, ?2, ?3)",
                    params![post_id, mk, mv],
                );
            }

            if let Some(ref lj) = layout_json_str {
                let _ = tx.execute(
                    "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_layout'",
                    params![post_id],
                );
                let _ = tx.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_layout', ?2)",
                    params![post_id, lj],
                );
            }

            if let Some(ref tpl) = p.template {
                if !tpl.is_empty() {
                    let _ = tx.execute(
                        "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_wp_page_template'",
                        params![post_id],
                    );
                    let _ = tx.execute(
                        "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_page_template', ?2)",
                        params![post_id, tpl],
                    );
                }
            }

            // Term relationships: map category and tag terms using cat_id_map and tag_id_map
            for old_cat_id in &p.categories {
                let cat_id = cat_id_map.get(old_cat_id).copied().unwrap_or(*old_cat_id);
                let tt_id: Option<i64> = tx.query_row(
                    "SELECT term_taxonomy_id FROM term_taxonomy WHERE term_id = ?1 AND taxonomy = 'category'",
                    params![cat_id],
                    |r| r.get(0),
                ).ok();

                if let Some(tid) = tt_id {
                    let _ = tx.execute(
                        "INSERT OR IGNORE INTO term_relationships (object_id, term_taxonomy_id) VALUES (?1, ?2)",
                        params![post_id, tid],
                    );
                }
            }

            for old_tag_id in &p.tags {
                let tag_id = tag_id_map.get(old_tag_id).copied().unwrap_or(*old_tag_id);
                let tt_id: Option<i64> = tx.query_row(
                    "SELECT term_taxonomy_id FROM term_taxonomy WHERE term_id = ?1 AND taxonomy = 'post_tag'",
                    params![tag_id],
                    |r| r.get(0),
                ).ok();

                if let Some(tid) = tt_id {
                    let _ = tx.execute(
                        "INSERT OR IGNORE INTO term_relationships (object_id, term_taxonomy_id) VALUES (?1, ?2)",
                        params![post_id, tid],
                    );
                }
            }

            // Featured media (remapped via post_id_map)
            if let Some(old_fm_id) = p.featured_media {
                if old_fm_id > 0 {
                    let fm_id = post_id_map.get(&old_fm_id).copied().unwrap_or(old_fm_id);
                    let _ = tx.execute(
                        "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_thumbnail_id'",
                        params![post_id],
                    );
                    let _ = tx.execute(
                        "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_thumbnail_id', ?2)",
                        params![post_id, fm_id.to_string()],
                    );
                }
            }

            if p.post_type == "page" {
                page_count += 1;
            } else {
                post_count += 1;
            }
        }

        // Second pass for post_parent and _thumbnail_id remapping
        for (new_post_id, old_parent_id) in pending_post_parents {
            let mapped_parent = post_id_map
                .get(&old_parent_id)
                .copied()
                .unwrap_or(old_parent_id);
            tx.execute(
                "UPDATE posts SET post_parent = ?1 WHERE ID = ?2",
                params![mapped_parent, new_post_id],
            )?;
        }

        let thumb_rows: Vec<(i64, String)> = {
            let mut stmt = tx.prepare(
                "SELECT meta_id, COALESCE(meta_value, '') FROM postmeta WHERE meta_key = '_thumbnail_id'",
            )?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            rows.filter_map(|r| r.ok()).collect()
        };
        for (meta_id, old_val) in thumb_rows {
            if let Ok(old_tid) = old_val.trim().parse::<i64>() {
                if let Some(&new_tid) = post_id_map.get(&old_tid) {
                    let _ = tx.execute(
                        "UPDATE postmeta SET meta_value = ?1 WHERE meta_id = ?2",
                        params![new_tid.to_string(), meta_id],
                    );
                }
            }
        }

        // 6. Site Settings (after post_id_map is populated so page_on_front & page_for_posts remap!)
        if let Some(s) = &req.site_settings {
            if let Some(title) = &s.title {
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('blogname', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![title],
                )?;
            }
            if let Some(desc) = &s.description {
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('blogdescription', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![desc],
                )?;
            }
            if let Some(email) = &s.email {
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('admin_email', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![email],
                )?;
            }
            if let Some(df) = &s.date_format {
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('date_format', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![df],
                )?;
            }
            if let Some(tf) = &s.time_format {
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('time_format', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![tf],
                )?;
            }
            if let Some(ppp) = s.posts_per_page {
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('posts_per_page', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![ppp.to_string()],
                )?;
            }
            if let Some(ref sof) = s.show_on_front {
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('show_on_front', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![sof],
                )?;
            }
            if let Some(pof) = s.page_on_front {
                let mapped_pof = if pof > 0 {
                    post_id_map.get(&pof).copied().unwrap_or(pof)
                } else {
                    0
                };
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('page_on_front', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![mapped_pof.to_string()],
                )?;
            }
            if let Some(pfp) = s.page_for_posts {
                let mapped_pfp = if pfp > 0 {
                    post_id_map.get(&pfp).copied().unwrap_or(pfp)
                } else {
                    0
                };
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('page_for_posts', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![mapped_pfp.to_string()],
                )?;
            }
            if let Some(ref perm) = s.permalink_structure {
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('permalink_structure', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![perm],
                )?;
            }
            if let Some(ref tz) = s.timezone_string {
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('timezone_string', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![tz],
                )?;
            }
            if let Some(ref dcs) = s.default_comment_status {
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('default_comment_status', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![dcs],
                )?;
            }
            if let Some(icon_id) = s.site_icon {
                let mapped_icon = if icon_id > 0 {
                    post_id_map.get(&icon_id).copied().unwrap_or(icon_id)
                } else {
                    0
                };
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('site_icon', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![mapped_icon.to_string()],
                )?;
            }
        }

        // 7. Menus
        let mut menu_count = 0;
        if !req.menus.is_empty() {
            menu_count = req.menus.len();
            let primary_menu = req
                .menus
                .iter()
                .find(|m| {
                    m.locations.iter().any(|l| {
                        let lower = l.to_lowercase();
                        lower.contains("primary")
                            || lower.contains("main")
                            || lower.contains("header")
                            || lower.contains("menu-1")
                    }) && !m.items.is_empty()
                })
                .or_else(|| req.menus.iter().find(|m| !m.items.is_empty()))
                .unwrap_or(&req.menus[0]);

            let normalized_items: Vec<NavMenuItemDto> = primary_menu
                .items
                .iter()
                .map(|item| NavMenuItemDto {
                    id: item.id,
                    title: item.title.clone(),
                    url: Self::normalize_source_menu_url(&item.url, &post_id_map),
                    target: item.target.clone(),
                    parent: item.parent,
                    menu_order: item.menu_order,
                })
                .collect();

            if let Ok(json_items) = serde_json::to_string(&normalized_items) {
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('daypress_nav_menu_items', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![json_items],
                )?;
            }
        }

        // 8. Comments
        let mut comment_count = 0;
        let mut pending_comment_parents: Vec<(i64, i64)> = Vec::new();
        for c in &req.comments {
            let mapped_post_id = post_id_map.get(&c.post_id).copied().unwrap_or(c.post_id);
            let c_date = if c.date.trim().is_empty() {
                &now
            } else {
                c.date.trim()
            };
            let approved = match c.approved.trim() {
                "" | "1" | "approve" | "approved" => "1",
                "0" | "hold" | "pending" => "0",
                other => other,
            };

            tx.execute(
                r#"INSERT INTO comments (comment_post_ID, comment_author, comment_author_email, comment_author_url, comment_author_IP, comment_date, comment_date_gmt, comment_content, comment_approved, comment_parent, user_id)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7, ?8, ?9, ?10)"#,
                params![
                    mapped_post_id,
                    c.author,
                    c.author_email,
                    c.author_url,
                    c.author_ip,
                    c_date,
                    c.content,
                    approved,
                    c.parent,
                    c.user_id
                ],
            )?;
            let new_cid = tx.last_insert_rowid();
            if let Some(old_cid) = c.id {
                comment_id_map.insert(old_cid, new_cid);
            }
            if c.parent > 0 {
                pending_comment_parents.push((new_cid, c.parent));
            }
            comment_count += 1;
        }

        for (new_cid, old_cparent) in pending_comment_parents {
            let mapped_cparent = comment_id_map
                .get(&old_cparent)
                .copied()
                .unwrap_or(old_cparent);
            tx.execute(
                "UPDATE comments SET comment_parent = ?1 WHERE comment_ID = ?2",
                params![mapped_cparent, new_cid],
            )?;
        }

        // 9. Widgets
        if let Some(ref w) = req.widgets {
            if !w.is_null() {
                let w_str = if let Some(s) = w.as_str() {
                    s.to_string()
                } else {
                    serde_json::to_string(w).unwrap_or_default()
                };
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES ('daypress_widgets', ?1)
                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                    params![w_str],
                )?;
            }
        }

        // 10. Explicit Redirects from Bundle (Redirection / RankMath / Yoast / Simple 301 Redirects)
        for r in &req.redirects {
            let mut src = r.source_path.trim().to_string();
            let dst = r.target_url.trim().to_string();
            if src.is_empty() || dst.is_empty() {
                continue;
            }
            if !src.starts_with('/') && !src.starts_with("http") {
                src = format!("/{src}");
            }
            let code = if r.status_code == 302 || r.status_code == 307 || r.status_code == 308 {
                r.status_code
            } else {
                301
            };
            let _ = tx.execute(
                r#"INSERT INTO daypress_redirects (source_path, target_url, status_code, hits, is_active, created_at)
                   VALUES (?1, ?2, ?3, 0, 1, ?4)
                   ON CONFLICT(source_path) DO UPDATE SET
                       target_url = excluded.target_url,
                       status_code = excluded.status_code"#,
                params![src, dst, code, now],
            );
            redirect_count += 1;
        }
        if redirect_count > 0 {
            let _ = crate::convergence::activate_builtin_plugin_tx(&tx, "daypress-seo", None);
            converged_plugins.push(format!(
                "Redirection / 301 Rules ({redirect_count} rules) -> Daypress Redirect Manager"
            ));
        }

        // 11. Universal WordPress Plugin Options Convergence
        if !req.plugin_options.is_empty() {
            let mut from_opts =
                crate::convergence::converge_plugin_options(&tx, &req.plugin_options);
            converged_plugins.append(&mut from_opts);
        }
        if seo_meta_converged && !converged_plugins.iter().any(|c| c.contains("SEO Toolkit")) {
            let _ = crate::convergence::activate_builtin_plugin_tx(&tx, "daypress-seo", None);
            converged_plugins.push("Post SEO Metadata -> Daypress SEO Toolkit".to_string());
        }

        tx.commit()?;

        Ok(ImportBundleResponse {
            success: true,
            migrated_posts: post_count,
            migrated_pages: page_count,
            migrated_categories: cat_count,
            migrated_tags: tag_count,
            migrated_media: media_count,
            migrated_menus: menu_count,
            migrated_comments: comment_count,
            migrated_users: user_count,
            migrated_forms: form_count,
            migrated_redirects: redirect_count,
            converged_plugins,
            message: "1-Click Migration & Universal Plugin Convergence committed to Rusqlite WAL storage!"
                .to_string(),
        })
    }

    // --- WXR Streaming Ingestion ---

    pub fn import_wxr<R: std::io::BufRead>(
        &self,
        reader: R,
    ) -> Result<WxrImportResponse, AppError> {
        let mut conn = self.conn.lock().unwrap();
        crate::wxr::import_wxr_stream(reader, &mut conn)
    }

    // --- WXR Export & Privacy Personal Data ---

    pub fn export_wxr(&self) -> Result<String> {
        let settings = self.get_settings()?;
        let posts = self.get_posts("any", Some("any"), None, 500)?;
        let categories = self.get_terms("category")?;
        let tags = self.get_terms("post_tag")?;

        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
        xml.push_str("<rss version=\"2.0\"\n");
        xml.push_str("    xmlns:excerpt=\"http://wordpress.org/export/1.2/excerpt/\"\n");
        xml.push_str("    xmlns:content=\"http://purl.org/rss/1.0/modules/content/\"\n");
        xml.push_str("    xmlns:wfw=\"http://wellformedweb.org/CommentAPI/\"\n");
        xml.push_str("    xmlns:dc=\"http://purl.org/dc/elements/1.1/\"\n");
        xml.push_str("    xmlns:wp=\"http://wordpress.org/export/1.2/\">\n");
        xml.push_str("<channel>\n");
        xml.push_str(&format!("    <title>{}</title>\n", settings.title));
        xml.push_str(&format!("    <link>{}</link>\n", settings.url));
        xml.push_str(&format!(
            "    <description>{}</description>\n",
            settings.description
        ));
        xml.push_str("    <wp:wxr_version>1.2</wp:wxr_version>\n");

        for cat in &categories {
            xml.push_str(&format!("    <wp:category><wp:term_id>{}</wp:term_id><wp:category_nicename>{}</wp:category_nicename><wp:cat_name><![CDATA[{}]]></wp:cat_name></wp:category>\n",
                cat.id, cat.slug, cat.name));
        }

        for tag in &tags {
            xml.push_str(&format!("    <wp:tag><wp:term_id>{}</wp:term_id><wp:tag_slug>{}</wp:tag_slug><wp:tag_name><![CDATA[{}]]></wp:tag_name></wp:tag>\n",
                tag.id, tag.slug, tag.name));
        }

        for p in &posts {
            xml.push_str("    <item>\n");
            xml.push_str(&format!(
                "        <title><![CDATA[{}]]></title>\n",
                p.title.rendered
            ));
            xml.push_str(&format!(
                "        <link>{}/{}/</link>\n",
                settings.url, p.slug
            ));
            xml.push_str("        <dc:creator><![CDATA[admin]]></dc:creator>\n");
            xml.push_str(&format!(
                "        <content:encoded><![CDATA[{}]]></content:encoded>\n",
                p.content.rendered
            ));
            xml.push_str(&format!(
                "        <excerpt:encoded><![CDATA[{}]]></excerpt:encoded>\n",
                p.excerpt.rendered
            ));
            xml.push_str(&format!("        <wp:post_id>{}</wp:post_id>\n", p.id));
            xml.push_str(&format!(
                "        <wp:post_date>{}</wp:post_date>\n",
                p.date
            ));
            xml.push_str(&format!(
                "        <wp:post_name>{}</wp:post_name>\n",
                p.slug
            ));
            xml.push_str(&format!("        <wp:status>{}</wp:status>\n", p.status));
            xml.push_str(&format!(
                "        <wp:post_type>{}</wp:post_type>\n",
                p.post_type
            ));
            xml.push_str("    </item>\n");
        }

        xml.push_str("</channel>\n</rss>\n");
        Ok(xml)
    }
}

impl crate::ports::output::MigrationRepository for Database {
    fn import_bundle(&self, req: &ImportBundleRequest) -> DomainResult<ImportBundleResponse> {
        Database::import_bundle(self, req).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn import_wxr_xml(&self, xml_bytes: &[u8]) -> DomainResult<WxrImportResponse> {
        let cursor = std::io::Cursor::new(xml_bytes);
        self.import_wxr(cursor).map_err(DomainError::from)
    }

    fn export_wxr_xml(&self) -> DomainResult<String> {
        self.export_wxr()
            .map_err(|e| DomainError::Storage(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_import_bundle_relational_remapping_and_ast_transpilation() {
        let tmp_dir = std::env::temp_dir();
        let db_path = tmp_dir.join(format!(
            "daypress_test_bundle_{}.sqlite",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(1)
        ));
        let db = Database::new(&db_path).expect("Database::new");

        let mut page_meta = HashMap::new();
        page_meta.insert(
            "_elementor_data".to_string(),
            r#"[{"id":"s1","elType":"section","elements":[{"id":"c1","elType":"column","elements":[{"id":"w1","elType":"widget","widgetType":"heading","settings":{"title":"Migrated Elementor Hero","header_size":"h1"}}]}]}]"#.to_string(),
        );

        let req = ImportBundleRequest {
            site_settings: Some(ImportSiteSettings {
                title: Some("Migrated Corp".to_string()),
                description: Some("Fast Edge CMS".to_string()),
                email: Some("hello@migrated.corp".to_string()),
                date_format: None,
                time_format: None,
                posts_per_page: Some(12),
                show_on_front: Some("page".to_string()),
                page_on_front: Some(500),
                page_for_posts: Some(501),
                permalink_structure: Some("/%postname%/".to_string()),
                timezone_string: Some("UTC".to_string()),
                default_comment_status: Some("open".to_string()),
                site_icon: Some(900),
                site_url: None,
            }),
            categories: vec![ImportTerm {
                id: Some(77),
                name: "Engineering".to_string(),
                slug: "engineering".to_string(),
                description: "Engineering posts".to_string(),
                parent: Some(0),
            }],
            tags: vec![ImportTerm {
                id: Some(88),
                name: "Rust".to_string(),
                slug: "rust".to_string(),
                description: "Rust lang".to_string(),
                parent: Some(0),
            }],
            media: vec![ImportMedia {
                id: Some(900),
                title: "Hero Banner".to_string(),
                file_name: "hero.webp".to_string(),
                mime_type: "image/webp".to_string(),
                source_url: "http://old-wp.local/wp-content/uploads/2026/05/hero.webp".to_string(),
                original_url: None,
                alt_text: Some("Hero Alt".to_string()),
                file_size: Some(4096),
                base64_data: None,
            }],
            posts: vec![
                ImportPost {
                    id: Some(600),
                    title: "Rust vs PHP 😊".to_string(),
                    content: "<p>Check out http://old-wp.local/wp-content/uploads/2026/05/hero.webp in our post! 😊</p>".to_string(),
                    excerpt: String::new(),
                    slug: "rust-vs-php".to_string(),
                    status: "publish".to_string(),
                    post_type: "post".to_string(),
                    date: Some("2026-05-01 10:00:00".to_string()),
                    modified: None,
                    featured_media: Some(900),
                    parent: Some(0),
                    menu_order: Some(0),
                    author: Some(1),
                    comment_status: None,
                    ping_status: None,
                    template: None,
                    categories: vec![77],
                    tags: vec![88],
                    postmeta: HashMap::new(),
                },
                ImportPost {
                    id: Some(500),
                    title: "Landing Home".to_string(),
                    content: "<p>Legacy fallback</p>".to_string(),
                    excerpt: String::new(),
                    slug: "landing-home".to_string(),
                    status: "publish".to_string(),
                    post_type: "page".to_string(),
                    date: Some("2026-05-01 09:00:00".to_string()),
                    modified: None,
                    featured_media: Some(900),
                    parent: Some(0),
                    menu_order: Some(1),
                    author: Some(1),
                    comment_status: None,
                    ping_status: None,
                    template: None,
                    categories: vec![],
                    tags: vec![],
                    postmeta: page_meta,
                },
                ImportPost {
                    id: Some(501),
                    title: "Child Page".to_string(),
                    content: "<p>Subpage content</p>".to_string(),
                    excerpt: String::new(),
                    slug: "child-page".to_string(),
                    status: "publish".to_string(),
                    post_type: "page".to_string(),
                    date: Some("2026-05-01 09:30:00".to_string()),
                    modified: None,
                    featured_media: None,
                    parent: Some(500),
                    menu_order: Some(2),
                    author: Some(1),
                    comment_status: None,
                    ping_status: None,
                    template: None,
                    categories: vec![],
                    tags: vec![],
                    postmeta: HashMap::new(),
                },
            ],
            theme_mods: None,
            menus: vec![ImportMenu {
                id: Some(1),
                name: "Main Menu".to_string(),
                slug: "main-menu".to_string(),
                locations: vec!["primary".to_string()],
                items: vec![NavMenuItemDto {
                    id: 1,
                    title: "Home".to_string(),
                    url: "http://old-wp.local/?page_id=500".to_string(),
                    target: String::new(),
                    parent: 0,
                    menu_order: 1,
                }],
            }],
            comments: vec![ImportComment {
                id: Some(301),
                post_id: 600,
                author: "Alice".to_string(),
                author_email: "alice@example.com".to_string(),
                author_url: String::new(),
                author_ip: "127.0.0.1".to_string(),
                date: "2026-05-01 12:00:00".to_string(),
                content: "Great migration!".to_string(),
                approved: "1".to_string(),
                parent: 0,
                user_id: 0,
            }],
            users: vec![],
            elementor_kit: Some(serde_json::json!({
                "system_colors": [
                    {"_id": "primary", "title": "Primary", "color": "#ff4f00"}
                ]
            })),
            widgets: None,
            plugin_options: HashMap::new(),
            redirects: vec![],
            forms: vec![],
            site_url: None,
        };

        let res = db.import_bundle(&req).expect("import_bundle");
        assert!(res.success);
        assert_eq!(res.migrated_posts, 1);
        assert_eq!(res.migrated_pages, 2);
        assert_eq!(res.migrated_categories, 1);
        assert_eq!(res.migrated_tags, 1);
        assert_eq!(res.migrated_media, 1);

        let landing = db
            .get_post_by_slug("landing-home")
            .unwrap()
            .expect("landing-home exists");
        let child = db
            .get_post_by_slug("child-page")
            .unwrap()
            .expect("child-page exists");
        assert_eq!(child.parent, landing.id);

        let settings = db.get_settings().unwrap();
        assert_eq!(settings.show_on_front, "page");
        assert_eq!(settings.page_on_front, landing.id);
        assert_eq!(settings.page_for_posts, child.id);

        let layout = db.get_post_layout(landing.id).unwrap();
        assert!(layout.is_some());
        assert!(landing.content.rendered.contains("Migrated Elementor Hero"));

        let post = db
            .get_post_by_slug("rust-vs-php")
            .unwrap()
            .expect("rust-vs-php exists");
        assert!(post.category_names.contains(&"Engineering".to_string()));
        assert!(post.tag_names.contains(&"Rust".to_string()));
        assert!(post.featured_image_url.is_some());
        assert_eq!(post.comments.len(), 1);
        assert_eq!(post.comments[0].content.rendered, "Great migration!");

        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_password_hashing_and_verification() {
        let tmp_dir = std::env::temp_dir();
        let db_path = tmp_dir.join(format!(
            "daypress_test_auth_{}.sqlite",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(2)
        ));
        let db = Database::new(&db_path).expect("Database::new");

        // A fresh install never accepts the historical admin/admin default: the seeded admin
        // receives a generated password on first start.
        assert!(db
            .verify_user_credentials("admin", "admin")
            .unwrap()
            .is_none());
        assert!(db
            .verify_user_credentials("admin", "wrong-password")
            .unwrap()
            .is_none());

        // Newly created user with salted SHA-256 hash
        let user = db
            .create_user(&CreateUserRequest {
                username: "editor_jane".to_string(),
                email: "jane@example.com".to_string(),
                password: "CorrectHorseBatteryStaple!".to_string(),
                name: None,
                first_name: None,
                last_name: None,
                display_name: Some("Jane Editor".to_string()),
                url: None,
                role: Some("Editor".to_string()),
            })
            .expect("create_user");
        assert_eq!(user.username, "editor_jane");
        assert!(db
            .verify_user_credentials("editor_jane", "CorrectHorseBatteryStaple!")
            .unwrap()
            .is_some());
        assert!(db
            .verify_user_credentials("editor_jane", "admin")
            .unwrap()
            .is_none());

        let _ = std::fs::remove_file(&db_path);
    }

    #[test]
    fn test_duplicate_post_and_plugin_convergence_and_sdk() {
        let tmp_dir = std::env::temp_dir();
        let db_path = tmp_dir.join(format!(
            "daypress_test_plugins_{}.sqlite",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(3)
        ));
        let db = Database::new(&db_path).expect("Database::new");

        // 1. Create a post with SEO metadata and categories, then duplicate it
        let created = db
            .create_post(&CreatePostRequest {
                title: "Original Pillar Post".to_string(),
                content: "<p>Pillar content</p>".to_string(),
                status: "publish".to_string(),
                slug: Some("original-pillar-post".to_string()),
                post_type: "post".to_string(),
                seo_title: Some("Pillar SEO Title".to_string()),
                seo_description: Some("Pillar SEO Description".to_string()),
                seo_keyword: Some("rust cms".to_string()),
                categories: vec![1],
                ..Default::default()
            })
            .expect("create_post");

        let cloned = db
            .duplicate_post(created.id)
            .expect("duplicate_post")
            .expect("cloned post");
        assert_ne!(cloned.id, created.id);
        assert_eq!(cloned.status, "draft");
        assert!(cloned.title.rendered.contains("(Copy)"));
        assert_eq!(cloned.seo_title, "Pillar SEO Title");
        assert_eq!(cloned.seo_keyword, "rust cms");

        // 2. Slug change on published post automatically creates a 301 redirect
        let _updated = db
            .update_post(
                created.id,
                &UpdatePostRequest {
                    slug: Some("new-pillar-slug".to_string()),
                    ..Default::default()
                },
            )
            .expect("update_post");

        let matched = db.match_redirect_and_increment("/original-pillar-post");
        assert_eq!(matched, Some(("/new-pillar-slug/".to_string(), 301)));

        // 3. Universal Plugin Convergence via import_bundle (Yoast + CF7 + Redirection + Joinchat)
        let mut postmeta = HashMap::new();
        postmeta.insert(
            "_yoast_wpseo_title".to_string(),
            "Converged Yoast Title".to_string(),
        );
        postmeta.insert(
            "_yoast_wpseo_metadesc".to_string(),
            "Converged Yoast Desc".to_string(),
        );
        postmeta.insert(
            "_yoast_wpseo_focuskw".to_string(),
            "daypress migration".to_string(),
        );

        let mut plugin_options = HashMap::new();
        plugin_options.insert(
            "joinchat".to_string(),
            serde_json::json!({
                "telephone": "+15550199999",
                "message_send": "Hello from converged Joinchat!",
                "button_tip": "Chat on WhatsApp"
            }),
        );

        let bundle = ImportBundleRequest {
            posts: vec![ImportPost {
                id: Some(700),
                title: "Contact Us".to_string(),
                slug: "contact-us-converged".to_string(),
                content: "<p>Get in touch:</p>[contact-form-7 id=\"55\" title=\"Main Contact\"]"
                    .to_string(),
                status: "publish".to_string(),
                post_type: "page".to_string(),
                postmeta,
                ..Default::default()
            }],
            forms: vec![ImportForm {
                id: Some(55),
                title: "Main Contact".to_string(),
                source_plugin: "cf7".to_string(),
                raw_form: "[text* your-name] [email* your-email] [textarea your-message]"
                    .to_string(),
                recipient_email: "leads@example.com".to_string(),
                schema_json: None,
            }],
            redirects: vec![ImportRedirect {
                source_path: "/legacy-about".to_string(),
                target_url: "/contact-us-converged/".to_string(),
                status_code: 301,
            }],
            plugin_options,
            ..Default::default()
        };

        let import_res = db.import_bundle(&bundle).expect("import_bundle");
        assert!(import_res.success);
        assert_eq!(import_res.migrated_forms, 1);
        assert!(import_res.migrated_redirects >= 1);

        let migrated_page = db
            .get_post_by_slug("contact-us-converged")
            .unwrap()
            .expect("contact-us-converged");
        assert_eq!(migrated_page.seo_title, "Converged Yoast Title");
        assert_eq!(migrated_page.seo_keyword, "daypress migration");
        assert!(migrated_page
            .content
            .rendered
            .contains("[daypress_form id=\""));

        // Expanding shortcodes renders the native form HTML
        let settings = db.get_settings().unwrap_or_default();
        let expanded = crate::plugins::expand_shortcodes_in_content(
            &migrated_page.content.rendered,
            &db,
            &settings,
        );
        assert!(expanded.contains("<form"));
        assert!(expanded.contains("your-email"));

        // WhatsApp plugin was auto-activated and configured from Joinchat
        let wa_plugin = db
            .get_installed_plugin("whatsapp-chat")
            .unwrap()
            .expect("whatsapp-chat");
        assert!(wa_plugin.active);
        assert_eq!(
            wa_plugin.config.get("phone_number").map(|s| s.as_str()),
            Some("+15550199999")
        );

        // 4. Community Plugin SDK installation & SSR hook rendering
        let sdk_manifest = crate::plugins::starter_plugin_sdk_template();
        db.install_custom_plugin(&sdk_manifest, None)
            .expect("install_custom_plugin");
        let hooks = crate::plugins::render_active_plugin_hooks(&db, &settings, None);
        assert!(hooks.body_top_html.contains("Powered by Daypress Rust CMS"));

        let _ = std::fs::remove_file(&db_path);
    }
}
