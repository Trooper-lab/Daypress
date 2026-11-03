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
        let mut kit_opt: Option<daypress_ast::DaystudioKit> = None;
        if let Some(ref kit_val) = req.daystudio_kit {
            if !kit_val.is_null() {
                if let Ok(kit_str) = serde_json::to_string(kit_val) {
                    tx.execute(
                        "INSERT INTO options (option_name, option_value) VALUES ('daypress_elementor_kit', ?1)
                         ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
                        params![kit_str],
                    )?;
                }
                kit_opt = serde_json::from_value::<daypress_ast::DaystudioKit>(kit_val.clone())
                    .ok()
                    .or_else(|| {
                        kit_val.get("settings").and_then(|s| {
                            serde_json::from_value::<daypress_ast::DaystudioKit>(s.clone()).ok()
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
        let mut fidelity_items: Vec<PostFidelityItem> = Vec::new();
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
            if matches!(
                p.post_type.as_str(),
                "wpcf7_contact_form"
                    | "wpforms"
                    | "acf-field-group"
                    | "acf-field"
                    | "acf-post-type"
                    | "acf-taxonomy"
            ) {
                continue;
            }
            let bundle_tz = req
                .site_settings
                .as_ref()
                .and_then(|s| s.timezone_string.as_deref())
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("UTC");
            let bundle_gmt_off: Option<&str> = None;
            let (p_date_local, p_date_gmt) = super::scheduling::normalize_post_date_with_tz(
                p.date.as_deref().unwrap_or(&now),
                bundle_tz,
                bundle_gmt_off,
            )
            .unwrap_or_else(|| {
                let d = p.date.as_deref().unwrap_or(&now);
                (d.to_string(), d.to_string())
            });
            let p_date = p_date_local.as_str();
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
            let transpiled_opt =
                daypress_ast::transpile_content(&rewritten_content, &rewritten_meta);

            let report = daypress_ast::analyze_post_fidelity(
                &rewritten_content,
                &rewritten_meta,
                transpiled_opt.as_ref(),
            );
            fidelity_items.push(PostFidelityItem {
                title: p.title.clone(),
                post_type: p.post_type.clone(),
                builder_detected: report.builder_detected,
                total_nodes: report.total_nodes,
                native_nodes: report.native_nodes,
                fallback_html_nodes: report.fallback_html_nodes,
                unresolved_shortcodes: report.unresolved_shortcodes,
                fidelity_score_pct: report.fidelity_score_pct,
            });

            if let Some(mut layout) = transpiled_opt {
                if layout.global_styles.is_none() {
                    if let Some(ref kit) = kit_opt {
                        layout.global_styles = serde_json::to_value(kit).ok();
                    } else if let Some(ref kv) = req.daystudio_kit {
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
                        "UPDATE posts SET post_author = ?1, post_title = ?2, post_content = ?3, post_excerpt = ?4, post_status = ?5, post_date = ?6, post_date_gmt = ?14, post_modified = ?7, comment_status = ?8, ping_status = ?9, post_parent = ?10, menu_order = ?11, layout_json = ?12 WHERE ID = ?13",
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
                            layout_json_str.as_deref(),

                            id,
                            p_date_gmt.as_str()
                        ],
                    )?;
                    id
                }
                None => {
                    tx.execute(
                        r#"INSERT INTO posts (post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, comment_status, ping_status, post_name, post_modified, post_modified_gmt, guid, post_type, post_parent, menu_order, layout_json)
                           VALUES (?1, ?2, ?15, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10, '', ?11, ?12, ?13, ?14)"#,
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
                            layout_json_str.as_deref(),
                            p_date_gmt.as_str()
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
                    icon: item.icon.clone(),
                    badge: item.badge.clone(),
                    description: item.description.clone(),
                    mega_menu_template_id: item.mega_menu_template_id,
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

        // 11. Universal WordPress Plugin Options Convergence + ACF / CPT UI Schema Ingestion
        if let Ok(mut acf_converged) =
            crate::importer::acf_cpt_ingest::ingest_acf_and_cpt_from_bundle(
                &tx,
                &req.posts,
                &req.plugin_options,
            )
        {
            converged_plugins.append(&mut acf_converged);
        }
        let _ = crate::importer::acf_cpt_ingest::finalize_acf_and_cpt_migration(&tx, &post_id_map);

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
            fidelity: Some(MigrationFidelitySummary::from_items(fidelity_items)),
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
