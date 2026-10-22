use crate::errors::AppError;
use crate::models::{ImportSiteSettings, NavMenuItemDto, WxrImportResponse};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::io::{BufRead, Cursor};

#[derive(Debug, Default)]
struct ParsedWxrItem {
    id: Option<i64>,
    title: String,
    slug: String,
    content: String,
    excerpt: String,
    post_date: String,
    post_status: String,
    post_type: String,
    post_mime_type: String,
    parent: i64,
    menu_order: i64,
    guid: String,
    attachment_url: String,
    categories: Vec<String>,
    tags: Vec<String>,
    nav_menu_slug: Option<String>,
    elementor_library_type: Option<String>,
    postmeta: HashMap<String, String>,
    comments: Vec<ParsedWxrComment>,
}

#[derive(Debug, Default)]
struct ParsedWxrComment {
    id: Option<i64>,
    author: String,
    email: String,
    url: String,
    date: String,
    content: String,
    approved: String,
    comment_type: String,
    parent: i64,
}

/// Infers MIME type from a filename or URL extension when `post_mime_type` is empty in WXR.
pub fn infer_mime_type_from_filename(path_or_url: &str) -> &'static str {
    let clean = path_or_url
        .split('?')
        .next()
        .unwrap_or(path_or_url)
        .trim()
        .to_lowercase();
    if clean.ends_with(".jpg") || clean.ends_with(".jpeg") {
        "image/jpeg"
    } else if clean.ends_with(".png") {
        "image/png"
    } else if clean.ends_with(".webp") {
        "image/webp"
    } else if clean.ends_with(".svg") {
        "image/svg+xml"
    } else if clean.ends_with(".gif") {
        "image/gif"
    } else if clean.ends_with(".pdf") {
        "application/pdf"
    } else {
        ""
    }
}

fn normalize_menu_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
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
    }
}

#[derive(Debug, Default)]
struct ParsedWxrTerm {
    taxonomy: String,
    name: String,
    slug: String,
    description: String,
    #[allow(dead_code)]
    parent: i64,
}

#[allow(clippy::too_many_arguments)]
fn process_xml_text(
    text: &str,
    tag_stack: &[String],
    site_settings: &mut ImportSiteSettings,
    source_site_url: &mut Option<String>,
    site_language: &mut Option<String>,
    site_favicon_url: &mut Option<String>,
    current_term: &mut Option<ParsedWxrTerm>,
    current_item: &mut Option<ParsedWxrItem>,
    current_comment: &mut Option<ParsedWxrComment>,
    current_meta_key: &mut String,
    current_meta_value: &mut String,
    current_category_domain: &str,
    current_category_nicename: &str,
) {
    if text.is_empty() {
        return;
    }

    let current_tag = tag_stack.last().map(|s| s.as_str()).unwrap_or("");
    let parent_tag = if tag_stack.len() >= 2 {
        tag_stack[tag_stack.len() - 2].as_str()
    } else {
        ""
    };

    // Channel metadata
    if parent_tag == "channel" {
        match current_tag {
            "title" => site_settings.title = Some(text.to_string()),
            "description" => site_settings.description = Some(text.to_string()),
            "wp:base_site_url" | "link" => {
                if source_site_url.is_none() && (text.starts_with("http://") || text.starts_with("https://")) {
                    *source_site_url = Some(text.trim_end_matches('/').to_string());
                }
            }
            "language" => {
                *site_language = Some(text.to_string());
            }
            _ => (),
        }
    } else if parent_tag == "image" && current_tag == "url" {
        *site_favicon_url = Some(text.to_string());
    }

    // Taxonomy parsing
    if let Some(term) = current_term.as_mut() {
        match current_tag {
            "wp:cat_name" | "wp:tag_name" | "wp:term_name" => {
                term.name = text.to_string();
            }
            "wp:category_nicename" | "wp:tag_slug" | "wp:term_slug" => {
                term.slug = text.to_string();
            }
            "wp:category_description" | "wp:term_description" => {
                term.description = text.to_string();
            }
            "wp:term_taxonomy" => {
                term.taxonomy = text.to_string();
            }
            _ => (),
        }
    }

    // Item parsing
    if let Some(item) = current_item.as_mut() {
        if parent_tag == "wp:comment" || tag_stack.iter().any(|s| s == "wp:comment") {
            if let Some(comm) = current_comment.as_mut() {
                match current_tag {
                    "wp:comment_id" => comm.id = text.parse().ok(),
                    "wp:comment_author" => comm.author = text.to_string(),
                    "wp:comment_author_email" => comm.email = text.to_string(),
                    "wp:comment_author_url" => comm.url = text.to_string(),
                    "wp:comment_date" => comm.date = text.to_string(),
                    "wp:comment_content" => comm.content = text.to_string(),
                    "wp:comment_approved" => comm.approved = text.to_string(),
                    "wp:comment_type" => comm.comment_type = text.to_string(),
                    "wp:comment_parent" => comm.parent = text.parse().unwrap_or(0),
                    _ => (),
                }
            }
        } else if parent_tag == "wp:postmeta" || tag_stack.iter().any(|s| s == "wp:postmeta") {
            match current_tag {
                "wp:meta_key" => *current_meta_key = text.to_string(),
                "wp:meta_value" => *current_meta_value = text.to_string(),
                _ => (),
            }
        } else {
            match current_tag {
                "title" => item.title = text.to_string(),
                "wp:post_name" => item.slug = text.to_string(),
                "content:encoded" => item.content = text.to_string(),
                "excerpt:encoded" => item.excerpt = text.to_string(),
                "wp:post_date" | "wp:post_date_gmt" => {
                    if item.post_date.is_empty() {
                        item.post_date = text.to_string();
                    }
                }
                "wp:status" => item.post_status = text.to_string(),
                "wp:post_type" => item.post_type = text.to_string(),
                "wp:post_mime_type" => item.post_mime_type = text.to_string(),
                "wp:post_id" => item.id = text.parse().ok(),
                "wp:attachment_url" => item.attachment_url = text.to_string(),
                "guid" => item.guid = text.to_string(),
                "wp:post_parent" => item.parent = text.parse().unwrap_or(0),
                "wp:menu_order" => item.menu_order = text.parse().unwrap_or(0),
                "category" => {
                    if current_category_domain == "category" || current_category_domain.is_empty() {
                        let slug = if !current_category_nicename.is_empty() {
                            current_category_nicename.to_string()
                        } else {
                            generated_slugify::slugify(text)
                        };
                        item.categories.push(slug);
                    } else if current_category_domain == "post_tag" {
                        let slug = if !current_category_nicename.is_empty() {
                            current_category_nicename.to_string()
                        } else {
                            generated_slugify::slugify(text)
                        };
                        item.tags.push(slug);
                    } else if current_category_domain == "nav_menu" {
                        let slug = if !current_category_nicename.is_empty() {
                            current_category_nicename.to_string()
                        } else {
                            generated_slugify::slugify(text)
                        };
                        item.nav_menu_slug = Some(slug);
                    } else if current_category_domain == "elementor_library_type" {
                        let lib_type = if !current_category_nicename.is_empty() {
                            current_category_nicename.to_string()
                        } else {
                            text.to_string()
                        };
                        item.elementor_library_type = Some(lib_type);
                    }
                }
                _ => (),
            }
        }
    }
}

/// Convenience wrapper for importing a WXR XML string directly into SQLite storage.
pub fn import_wxr_xml(xml: &str, conn: &mut Connection) -> Result<WxrImportResponse, AppError> {
    import_wxr_stream(Cursor::new(xml.as_bytes()), conn)
}

/// Ingests a streaming WordPress XML (WXR) export into SQLite storage.
/// Employs a low-memory event-driven streaming parser capable of processing
/// multi-gigabyte exports with minimal RAM overhead.
pub fn import_wxr_stream<R: BufRead>(
    reader: R,
    conn: &mut Connection,
) -> Result<WxrImportResponse, AppError> {
    let mut xml_reader = Reader::from_reader(reader);
    xml_reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut tag_stack: Vec<String> = Vec::new();

    let mut site_settings = ImportSiteSettings::default();
    let mut source_site_url = Option::<String>::None;
    let mut site_language = Option::<String>::None;
    let mut site_favicon_url = Option::<String>::None;
    let mut current_term = Option::<ParsedWxrTerm>::None;
    let mut current_item = Option::<ParsedWxrItem>::None;
    let mut current_comment = Option::<ParsedWxrComment>::None;
    let mut current_meta_key = String::new();
    let mut current_meta_value = String::new();
    let mut current_category_domain = String::new();
    let mut current_category_nicename = String::new();

    let mut terms_to_insert = Vec::new();
    let mut posts_to_insert = Vec::new();

    let tx = conn.transaction()?;

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag_name == "item" {
                    current_item = Some(ParsedWxrItem {
                        post_status: "publish".to_string(),
                        post_type: "post".to_string(),
                        ..Default::default()
                    });
                } else if tag_name == "wp:category" || tag_name == "wp:tag" || tag_name == "wp:term"
                {
                    let tax = if tag_name == "wp:tag" {
                        "post_tag".to_string()
                    } else {
                        "category".to_string()
                    };
                    current_term = Some(ParsedWxrTerm {
                        taxonomy: tax,
                        ..Default::default()
                    });
                } else if tag_name == "wp:comment" {
                    current_comment = Some(ParsedWxrComment {
                        approved: "1".to_string(),
                        comment_type: "comment".to_string(),
                        ..Default::default()
                    });
                } else if tag_name == "wp:postmeta" {
                    current_meta_key.clear();
                    current_meta_value.clear();
                } else if tag_name == "category" {
                    current_category_domain.clear();
                    current_category_nicename.clear();
                    for attr in e.attributes().flatten() {
                        let k = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let v = String::from_utf8_lossy(&attr.value).to_string();
                        if k == "domain" {
                            current_category_domain = v;
                        } else if k == "nicename" {
                            current_category_nicename = v;
                        }
                    }
                }
                tag_stack.push(tag_name);
            }
            Ok(Event::Text(e)) => {
                let text = match e.unescape() {
                    Ok(cow) => cow.to_string(),
                    Err(_) => String::from_utf8_lossy(e.as_ref()).to_string(),
                };
                process_xml_text(
                    text.trim(),
                    &tag_stack,
                    &mut site_settings,
                    &mut source_site_url,
                    &mut site_language,
                    &mut site_favicon_url,
                    &mut current_term,
                    &mut current_item,
                    &mut current_comment,
                    &mut current_meta_key,
                    &mut current_meta_value,
                    &current_category_domain,
                    &current_category_nicename,
                );
            }
            Ok(Event::CData(e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).to_string();
                process_xml_text(
                    text.trim(),
                    &tag_stack,
                    &mut site_settings,
                    &mut source_site_url,
                    &mut site_language,
                    &mut site_favicon_url,
                    &mut current_term,
                    &mut current_item,
                    &mut current_comment,
                    &mut current_meta_key,
                    &mut current_meta_value,
                    &current_category_domain,
                    &current_category_nicename,
                );
            }
            Ok(Event::End(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag_name == "item" {
                    if let Some(item) = current_item.take() {
                        posts_to_insert.push(item);
                    }
                } else if tag_name == "wp:category" || tag_name == "wp:tag" || tag_name == "wp:term"
                {
                    if let Some(term) = current_term.take() {
                        if !term.name.is_empty() {
                            terms_to_insert.push(term);
                        }
                    }
                } else if tag_name == "wp:comment" {
                    if let (Some(item), Some(comm)) =
                        (current_item.as_mut(), current_comment.take())
                    {
                        item.comments.push(comm);
                    }
                } else if tag_name == "wp:postmeta" {
                    if let Some(item) = current_item.as_mut() {
                        if !current_meta_key.is_empty() {
                            item.postmeta
                                .insert(current_meta_key.clone(), current_meta_value.clone());
                        }
                    }
                }
                tag_stack.pop();
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(AppError::BadRequest(format!(
                    "WXR XML parsing error: {}",
                    e
                )));
            }
            _ => (),
        }
        buf.clear();
    }

    // 1. Commit Site Settings (and clear initial demo seed data if this is the first site migration)
    let wxr_site_title = site_settings
        .title
        .clone()
        .unwrap_or_else(|| "Neopress".to_string());
    if let Some(t) = site_settings.title {
        let is_already_migrated: bool = tx
            .query_row(
                "SELECT 1 FROM wp_options WHERE option_name = 'neopress_site_migrated' AND option_value = '1' LIMIT 1",
                [],
                |_| Ok(true),
            )
            .unwrap_or(false);
        let current_blogname: String = tx
            .query_row(
                "SELECT option_value FROM wp_options WHERE option_name = 'blogname' LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or_default();
        if !is_already_migrated && current_blogname == "Neopress" && t != "Neopress" {
            let _ = tx.execute_batch(
                r#"
                DELETE FROM wp_posts;
                DELETE FROM wp_postmeta;
                DELETE FROM wp_comments;
                DELETE FROM wp_term_relationships;
                "#,
            );
        }
        let _ = tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_site_migrated', '1')
             ON CONFLICT(option_name) DO UPDATE SET option_value = '1';",
            [],
        );
        tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('blogname', ?1)
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![t],
        )?;
    }
    if let Some(d) = site_settings.description {
        tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('blogdescription', ?1)
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![d],
        )?;
    }
    if let Some(ref src_url) = source_site_url {
        tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_source_site_url', ?1)
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![src_url],
        )?;
    }
    if let Some(ref lang) = site_language {
        tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('WPLANG', ?1)
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![lang],
        )?;
    }
    if let Some(ref fav_raw) = site_favicon_url {
        let fav_norm = normalize_menu_url(fav_raw);
        tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_site_favicon', ?1)
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![fav_norm],
        )?;
    }

    // 2. Commit Taxonomies
    let mut cat_count = 0;
    let mut tag_count = 0;
    for term in terms_to_insert {
        let slug = if term.slug.is_empty() {
            generated_slugify::slugify(&term.name)
        } else {
            term.slug
        };

        let term_id: i64 = match tx.query_row(
            "SELECT term_id FROM wp_terms WHERE slug = ?1",
            params![slug],
            |r| r.get(0),
        ) {
            Ok(id) => {
                let _ = tx.execute(
                    "UPDATE wp_terms SET name = ?1 WHERE term_id = ?2",
                    params![term.name, id],
                );
                id
            }
            Err(_) => {
                tx.execute(
                    "INSERT INTO wp_terms (name, slug) VALUES (?1, ?2)",
                    params![term.name, slug],
                )?;
                tx.last_insert_rowid()
            }
        };

        let taxonomy = if term.taxonomy.is_empty() {
            "category"
        } else {
            &term.taxonomy
        };

        let tt_exists: Option<i64> = tx
            .query_row(
                "SELECT term_taxonomy_id FROM wp_term_taxonomy WHERE term_id = ?1 AND taxonomy = ?2",
                params![term_id, taxonomy],
                |r| r.get(0),
            )
            .ok();

        if tt_exists.is_none() {
            let _ = tx.execute(
                "INSERT INTO wp_term_taxonomy (term_id, taxonomy, description) VALUES (?1, ?2, ?3)",
                params![term_id, taxonomy, term.description],
            );
        }

        if taxonomy == "post_tag" {
            tag_count += 1;
        } else {
            cat_count += 1;
        }
    }

    // 3. Commit Posts, Pages, Media & Comments
    let mut post_count = 0;
    let mut page_count = 0;
    let mut media_count = 0;
    let mut comment_count = 0;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut old_to_new_post_id: HashMap<i64, i64> = HashMap::new();
    let mut post_urls: HashMap<String, String> = HashMap::new();
    let mut pending_parent_remaps: Vec<(i64, i64)> = Vec::new();
    let mut old_to_new_comment_id: HashMap<i64, i64> = HashMap::new();
    let mut pending_comment_parent_remaps: Vec<(i64, i64)> = Vec::new();
    let mut raw_nav_menu_items: Vec<(
        i64,
        String,
        i64,
        Option<String>,
        HashMap<String, String>,
    )> = Vec::new();

    // 3a. First Pass for Contact Form 7 / WPForms CPTs and Elementor Forms inside WXR stream
    let mut form_id_map: HashMap<i64, i64> = HashMap::new();
    let mut first_migrated_form_id: Option<i64> = None;
    let mut has_elementor_forms = false;
    for item in &posts_to_insert {
        if item
            .postmeta
            .get("_elementor_data")
            .map(|d| d.contains("\"widgetType\":\"form\""))
            .unwrap_or(false)
        {
            has_elementor_forms = true;
        }
        if let Some((fields, settings)) = crate::convergence::parse_cf7_or_wpforms_to_schema(
            &item.post_type,
            &item.title,
            &item.content,
            &item.postmeta,
        ) {
            let schema_json = serde_json::to_string(&fields).unwrap_or_else(|_| "[]".to_string());
            let settings_json =
                serde_json::to_string(&settings).unwrap_or_else(|_| "{}".to_string());
            let title = if item.title.trim().is_empty() {
                "Migrated WXR Form"
            } else {
                item.title.trim()
            };
            if tx
                .execute(
                    "INSERT INTO wp_neopress_forms (title, schema_json, settings_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
                    params![title, schema_json, settings_json, now],
                )
                .is_ok()
            {
                let new_fid = tx.last_insert_rowid();
                if first_migrated_form_id.is_none() {
                    first_migrated_form_id = Some(new_fid);
                }
                if let Some(old_fid) = item.id {
                    form_id_map.insert(old_fid, new_fid);
                }
            }
        }
    }
    if has_elementor_forms {
        let existing_forms: i64 = tx
            .query_row("SELECT COUNT(*) FROM wp_neopress_forms", [], |r| r.get(0))
            .unwrap_or(0);
        if existing_forms == 0 {
            let _ = tx.execute(
                "INSERT INTO wp_neopress_forms (id, title, schema_json, settings_json, created_at, updated_at) VALUES (1, 'Migrated Elementor Form', '[]', '{}', ?1, ?1)",
                params![now],
            );
        }
    }
    if !form_id_map.is_empty() || has_elementor_forms {
        let _ = crate::convergence::activate_builtin_plugin_tx(&tx, "neopress-forms", None);
    }

    let mut detected_logo_url: Option<String> = None;
    let mut detected_kit_raw_settings: Option<String> = None;
    let mut pending_ast_items: Vec<(
        i64,
        Option<i64>,
        String,
        String,
        String,
        Option<String>,
        HashMap<String, String>,
    )> = Vec::new();

    let mut seo_meta_converged = false;
    for mut item in posts_to_insert {
        if matches!(item.post_type.as_str(), "wpcf7_contact_form" | "wpforms") {
            continue;
        }
        item.content = crate::convergence::rewrite_legacy_form_shortcodes(
            &item.content,
            &form_id_map,
            first_migrated_form_id,
        );
        if crate::convergence::converge_post_seo_meta(
            &mut item.postmeta,
            &item.title,
            &wxr_site_title,
        ) {
            seo_meta_converged = true;
        }
        let slug = if item.slug.is_empty() {
            if item.title.is_empty() {
                format!("item-{}", item.id.unwrap_or(0))
            } else {
                generated_slugify::slugify(&item.title)
            }
        } else {
            item.slug.clone()
        };
        let p_date = if item.post_date.is_empty() {
            &now
        } else {
            &item.post_date
        };
        let p_type = &item.post_type;
        let p_status = &item.post_status;
        let guid = if !item.guid.is_empty() {
            item.guid.clone()
        } else if !item.attachment_url.is_empty() {
            item.attachment_url.clone()
        } else {
            String::new()
        };

        let inferred_mime = if !item.post_mime_type.is_empty() {
            item.post_mime_type.clone()
        } else if p_type == "attachment" {
            let mut m = infer_mime_type_from_filename(&item.attachment_url);
            if m.is_empty() {
                if let Some(af) = item.postmeta.get("_wp_attached_file") {
                    m = infer_mime_type_from_filename(af);
                }
            }
            if m.is_empty() {
                m = infer_mime_type_from_filename(&guid);
            }
            if m.is_empty() {
                m = infer_mime_type_from_filename(&slug);
            }
            m.to_string()
        } else {
            String::new()
        };

        // Detect Site Logo attachment
        if p_type == "attachment" {
            let url_candidate = if !item.attachment_url.is_empty() {
                normalize_menu_url(&item.attachment_url)
            } else if let Some(af) = item.postmeta.get("_wp_attached_file") {
                format!("/wp-content/uploads/{}", af.trim_start_matches('/'))
            } else {
                normalize_menu_url(&guid)
            };
            let slug_lower = slug.to_lowercase();
            let title_lower = item.title.to_lowercase();
            if (slug_lower.contains("logo") || title_lower.contains("logo"))
                && !url_candidate.is_empty()
            {
                let is_better = detected_logo_url
                    .as_ref()
                    .map(|prev| !prev.ends_with(".svg") && url_candidate.ends_with(".svg"))
                    .unwrap_or(true);
                if is_better {
                    detected_logo_url = Some(url_candidate);
                }
            }
        }

        // Detect Elementor Default Kit (`default-kit`)
        let is_kit = p_type == "elementor_library"
            && (slug == "default-kit"
                || item.postmeta.get("_elementor_template_type").map(|s| s.as_str()) == Some("kit")
                || item.elementor_library_type.as_deref() == Some("kit"));
        if is_kit {
            if let Some(raw_kit) = item.postmeta.get("_elementor_page_settings") {
                if !raw_kit.trim().is_empty() {
                    if let Some(kit) = neopress_ast::ElementorKit::from_raw_settings(raw_kit) {
                        let kit_css = kit.compile_to_css();
                        let _ = tx.execute(
                            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_elementor_kit_css', ?1)
                             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
                            params![kit_css],
                        );
                        if let Ok(kit_json) = serde_json::to_string(&kit) {
                            let _ = tx.execute(
                                "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_elementor_kit_json', ?1)
                                 ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
                                params![kit_json],
                            );
                            detected_kit_raw_settings = Some(kit_json);
                        }
                        if let Some(prim) = kit.system_colors.iter().find(|c| c.id == "primary") {
                            if !prim.color.trim().is_empty() {
                                let _ = tx.execute(
                                    "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_brand_color', ?1)
                                     ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
                                    params![prim.color.trim()],
                                );
                            }
                        }
                    }
                }
            }
        }

        // Normalize hardcoded source domain links in standard HTML content
        let mut initial_content = item.content.clone();
        if let Some(ref base) = source_site_url {
            let https_base = format!("{}/", base.replacen("http://", "https://", 1));
            let http_base = format!("{}/", base.replacen("https://", "http://", 1));
            initial_content = initial_content
                .replace(&https_base, "/")
                .replace(&http_base, "/");
        }

        let post_id: i64 = match tx.query_row(
            "SELECT ID FROM wp_posts WHERE post_name = ?1 AND post_type = ?2",
            params![slug, p_type],
            |r| r.get(0),
        ) {
            Ok(id) => {
                tx.execute(
                    "UPDATE wp_posts SET post_title = ?1, post_content = ?2, post_excerpt = ?3, post_status = ?4, post_date = ?5, post_mime_type = ?6, post_parent = ?7, menu_order = ?8 WHERE ID = ?9",
                    params![item.title, initial_content, item.excerpt, p_status, p_date, inferred_mime, item.parent, item.menu_order, id],
                )?;
                id
            }
            Err(_) => {
                tx.execute(
                    r#"INSERT INTO wp_posts (post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type, post_mime_type, post_parent, menu_order)
                       VALUES (1, ?1, ?1, ?2, ?3, ?4, ?5, ?6, ?1, ?1, ?7, ?8, ?9, ?10, ?11)"#,
                    params![p_date, initial_content, item.title, item.excerpt, p_status, slug, guid, p_type, inferred_mime, item.parent, item.menu_order],
                )?;
                tx.last_insert_rowid()
            }
        };

        let is_front_page = p_type == "page"
            && p_status == "publish"
            && (slug.eq_ignore_ascii_case("home") || slug.eq_ignore_ascii_case("front-page"));
        if is_front_page {
            let _ = tx.execute(
                "INSERT INTO wp_options (option_name, option_value) VALUES ('show_on_front', 'page')
                 ON CONFLICT(option_name) DO UPDATE SET option_value = 'page';",
                [],
            );
            let _ = tx.execute(
                "INSERT INTO wp_options (option_name, option_value) VALUES ('page_on_front', ?1)
                 ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
                params![post_id.to_string()],
            );
        }

        let permalink = if is_front_page {
            "/".to_string()
        } else {
            format!("/{}/", slug)
        };
        post_urls.insert(post_id.to_string(), permalink.clone());
        if let Some(old_id) = item.id {
            old_to_new_post_id.insert(old_id, post_id);
            post_urls.insert(old_id.to_string(), permalink);
        }
        if item.parent > 0 {
            pending_parent_remaps.push((post_id, item.parent));
        }
        if p_type == "nav_menu_item" {
            raw_nav_menu_items.push((
                post_id,
                item.title.clone(),
                item.menu_order,
                item.nav_menu_slug.clone(),
                item.postmeta.clone(),
            ));
        }

        // Queue for Pass 2 AST transpilation if it has Elementor, Divi, or Gutenberg content
        let needs_ast = item
            .postmeta
            .get("_elementor_data")
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
            || initial_content.contains("[et_pb_")
            || initial_content.contains("<!-- wp:");
        if needs_ast {
            pending_ast_items.push((
                post_id,
                item.id,
                p_type.clone(),
                p_status.clone(),
                initial_content,
                item.elementor_library_type.clone(),
                item.postmeta.clone(),
            ));
        }

        // Term relationships (categories and tags)
        for cat_slug in &item.categories {
            if let Ok(tid) = tx.query_row(
                "SELECT term_taxonomy_id FROM wp_term_taxonomy tt JOIN wp_terms t ON tt.term_id = t.term_id WHERE t.slug = ?1 AND tt.taxonomy = 'category'",
                params![cat_slug],
                |r| r.get::<_, i64>(0),
            ) {
                let _ = tx.execute("INSERT OR IGNORE INTO wp_term_relationships (object_id, term_taxonomy_id) VALUES (?1, ?2)", params![post_id, tid]);
            }
        }

        for tag_slug in &item.tags {
            if let Ok(tid) = tx.query_row(
                "SELECT term_taxonomy_id FROM wp_term_taxonomy tt JOIN wp_terms t ON tt.term_id = t.term_id WHERE t.slug = ?1 AND tt.taxonomy = 'post_tag'",
                params![tag_slug],
                |r| r.get::<_, i64>(0),
            ) {
                let _ = tx.execute("INSERT OR IGNORE INTO wp_term_relationships (object_id, term_taxonomy_id) VALUES (?1, ?2)", params![post_id, tid]);
            }
        }

        // Postmeta (like _thumbnail_id, _wp_attached_file)
        for (mk, mv) in &item.postmeta {
            let _ = tx.execute(
                "DELETE FROM wp_postmeta WHERE post_id = ?1 AND meta_key = ?2",
                params![post_id, mk],
            );
            let _ = tx.execute(
                "INSERT INTO wp_postmeta (post_id, meta_key, meta_value) VALUES (?1, ?2, ?3)",
                params![post_id, mk, mv],
            );
        }

        // Comments
        for c in &item.comments {
            let c_date = if c.date.is_empty() { &now } else { &c.date };
            let _ = tx.execute(
                r#"INSERT INTO wp_comments (comment_post_ID, comment_author, comment_author_email, comment_author_url, comment_date, comment_date_gmt, comment_content, comment_approved, comment_type, comment_parent)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7, ?8, ?9)"#,
                params![post_id, c.author, c.email, c.url, c_date, c.content, c.approved, c.comment_type, c.parent],
            );
            let new_cid = tx.last_insert_rowid();
            if let Some(old_cid) = c.id {
                old_to_new_comment_id.insert(old_cid, new_cid);
            }
            if c.parent > 0 {
                pending_comment_parent_remaps.push((new_cid, c.parent));
            }
            comment_count += 1;
        }

        match p_type.as_str() {
            "page" => page_count += 1,
            "attachment" => media_count += 1,
            "nav_menu_item" | "elementor_library" => {}
            _ => post_count += 1,
        }
    }

    if let Some(ref logo_url) = detected_logo_url {
        let _ = tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_site_logo', ?1)
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![logo_url],
        );
    }

    // 4. Relational ID Remapping (post_parent, _thumbnail_id, comment_parent, nav_menu_items)
    for (child_new_id, old_parent_id) in pending_parent_remaps {
        if let Some(&new_parent_id) = old_to_new_post_id.get(&old_parent_id) {
            tx.execute(
                "UPDATE wp_posts SET post_parent = ?1 WHERE ID = ?2",
                params![new_parent_id, child_new_id],
            )?;
        }
    }

    let thumb_rows: Vec<(i64, String)> = {
        let mut stmt = tx.prepare(
            "SELECT meta_id, COALESCE(meta_value, '') FROM wp_postmeta WHERE meta_key = '_thumbnail_id'",
        )?;
        let mapped = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
        mapped.filter_map(|r| r.ok()).collect()
    };

    for (meta_id, old_val) in thumb_rows {
        if let Ok(old_thumb_id) = old_val.trim().parse::<i64>() {
            if let Some(&new_thumb_id) = old_to_new_post_id.get(&old_thumb_id) {
                tx.execute(
                    "UPDATE wp_postmeta SET meta_value = ?1 WHERE meta_id = ?2",
                    params![new_thumb_id.to_string(), meta_id],
                )?;
            }
        }
    }

    for (new_cid, old_cparent) in pending_comment_parent_remaps {
        if let Some(&mapped_cparent) = old_to_new_comment_id.get(&old_cparent) {
            let _ = tx.execute(
                "UPDATE wp_comments SET comment_parent = ?1 WHERE comment_ID = ?2",
                params![mapped_cparent, new_cid],
            );
        }
    }

    let mut menus_by_slug: HashMap<String, Vec<neopress_ast::NavMenuItem>> = HashMap::new();

    if !raw_nav_menu_items.is_empty() {
        let mut nav_items: Vec<NavMenuItemDto> = Vec::with_capacity(raw_nav_menu_items.len());
        for (new_item_id, item_title, menu_order, menu_slug_opt, meta) in raw_nav_menu_items {
            let mut resolved_title = item_title;
            let mut resolved_url = meta
                .get("_menu_item_url")
                .map(|u| normalize_menu_url(u))
                .unwrap_or_default();

            if let Some(obj_id_str) = meta.get("_menu_item_object_id") {
                if let Ok(old_obj_id) = obj_id_str.trim().parse::<i64>() {
                    let target_post_id = old_to_new_post_id
                        .get(&old_obj_id)
                        .copied()
                        .unwrap_or(old_obj_id);
                    if let Ok((target_slug, target_title)) = tx.query_row(
                        "SELECT post_name, post_title FROM wp_posts WHERE ID = ?1",
                        params![target_post_id],
                        |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
                    ) {
                        if resolved_title.is_empty() {
                            resolved_title = target_title;
                        }
                        if resolved_url.is_empty() && !target_slug.is_empty() {
                            if let Some(permalink) = post_urls.get(&target_post_id.to_string()) {
                                resolved_url = permalink.clone();
                            } else {
                                resolved_url = format!("/{}/", target_slug);
                            }
                        }
                    }
                }
            }

            if resolved_url.is_empty() {
                resolved_url = "/".to_string();
            }
            if resolved_title.is_empty() {
                resolved_title = "Menu Item".to_string();
            }

            let old_menu_parent: i64 = meta
                .get("_menu_item_menu_item_parent")
                .and_then(|s| s.trim().parse().ok())
                .unwrap_or(0);
            let mapped_menu_parent = if old_menu_parent > 0 {
                old_to_new_post_id
                    .get(&old_menu_parent)
                    .copied()
                    .unwrap_or(old_menu_parent)
            } else {
                0
            };

            let slug_key = menu_slug_opt.unwrap_or_else(|| "primary".to_string());
            menus_by_slug
                .entry(slug_key)
                .or_default()
                .push(neopress_ast::NavMenuItem {
                    title: resolved_title.clone(),
                    url: resolved_url.clone(),
                    order: menu_order,
                });

            nav_items.push(NavMenuItemDto {
                id: new_item_id,
                title: resolved_title,
                url: resolved_url,
                target: meta.get("_menu_item_target").cloned().unwrap_or_default(),
                parent: mapped_menu_parent,
                menu_order,
            });
        }

        for list in menus_by_slug.values_mut() {
            list.sort_by_key(|i| i.order);
        }
        if let Ok(menus_json) = serde_json::to_string(&menus_by_slug) {
            let _ = tx.execute(
                "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_nav_menus_by_slug', ?1)
                 ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
                params![menus_json],
            );
        }

        nav_items.sort_by_key(|i| i.menu_order);
        if let Ok(json_str) = serde_json::to_string(&nav_items) {
            let _ = tx.execute(
                "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_nav_menu_items', ?1)
                 ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
                params![json_str],
            );
        }
    }

    // 5. Pass 2: Transpile all Elementor, Divi, and Gutenberg items with complete site context
    let post_urls_json = serde_json::to_string(&post_urls).unwrap_or_else(|_| "{}".to_string());
    let menus_json = serde_json::to_string(&menus_by_slug).unwrap_or_else(|_| "{}".to_string());

    let mut header_html = String::new();
    let mut footer_html = String::new();
    let mut popups_html = String::new();

    for (post_id, old_id_opt, p_type, p_status, raw_content, lib_type_opt, mut meta) in
        pending_ast_items
    {
        meta.insert("__np_post_urls_json".to_string(), post_urls_json.clone());
        meta.insert("__np_menus_json".to_string(), menus_json.clone());
        if let Some(ref logo) = detected_logo_url {
            meta.insert("__np_site_logo_url".to_string(), logo.clone());
        }
        if let Some(ref src) = source_site_url {
            meta.insert("__np_source_site_url".to_string(), src.clone());
        }
        if let Some(ref kit_str) = detected_kit_raw_settings {
            meta.insert("_elementor_kit_settings".to_string(), kit_str.clone());
        }

        if let Some(transpiled_layout) = neopress_ast::transpile_content(&raw_content, &meta) {
            let rendered_html = neopress_ast::render_layout_to_html(&transpiled_layout);
            let final_content = if !rendered_html.trim().is_empty()
                || raw_content.trim().is_empty()
                || raw_content.contains("[et_pb_")
            {
                rendered_html.clone()
            } else {
                raw_content
            };
            let layout_json_str = serde_json::to_string(&transpiled_layout).ok();

            if let Some(ref lj) = layout_json_str {
                tx.execute(
                    "UPDATE wp_posts SET post_content = ?1, layout_json = ?2 WHERE ID = ?3",
                    params![final_content, lj, post_id],
                )?;
                let _ = tx.execute(
                    "INSERT OR REPLACE INTO wp_postmeta (post_id, meta_key, meta_value) VALUES (?1, '_neopress_layout', ?2)",
                    params![post_id, lj],
                );
            } else {
                tx.execute(
                    "UPDATE wp_posts SET post_content = ?1 WHERE ID = ?2",
                    params![final_content, post_id],
                )?;
            }

            if p_type == "elementor_library" && p_status == "publish" {
                let tpl_type = meta
                    .get("_elementor_template_type")
                    .map(|s| s.as_str())
                    .or(lib_type_opt.as_deref())
                    .unwrap_or("");
                let body_without_kit_style =
                    neopress_ast::render_node_to_html(&transpiled_layout.root);
                match tpl_type {
                    "header" => {
                        header_html = body_without_kit_style;
                    }
                    "footer" => {
                        footer_html = body_without_kit_style;
                    }
                    "popup" => {
                        let popup_key = old_id_opt.unwrap_or(post_id);
                        popups_html.push_str(&format!(
                            "<div id=\"np-popup-{}\" class=\"np-popup-modal\" role=\"dialog\" aria-modal=\"true\"><div class=\"np-popup-backdrop\" onclick=\"this.parentElement.classList.remove('active')\"></div><div class=\"np-popup-dialog\"><button type=\"button\" class=\"np-popup-close\" aria-label=\"Close\" onclick=\"this.closest('.np-popup-modal').classList.remove('active')\">&times;</button>{}</div></div>\n",
                            popup_key, body_without_kit_style
                        ));
                    }
                    _ => {}
                }
            }
        }
    }

    if !header_html.is_empty() {
        let _ = tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_elementor_header_html', ?1)
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![header_html],
        );
    }
    if !footer_html.is_empty() {
        let _ = tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_elementor_footer_html', ?1)
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![footer_html],
        );
    }
    if !popups_html.is_empty() {
        let _ = tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_elementor_popups_html', ?1)
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![popups_html],
        );
    }

    if seo_meta_converged {
        let _ = crate::convergence::activate_builtin_plugin_tx(&tx, "neopress-seo", None);
    }

    tx.commit()?;

    Ok(WxrImportResponse {
        success: true,
        imported_posts: post_count,
        imported_pages: page_count,
        imported_media: media_count,
        imported_categories: cat_count,
        imported_tags: tag_count,
        imported_comments: comment_count,
        message: format!(
            "Successfully ingested WordPress XML Stream: {} posts, {} pages, {} media attachments, {} categories, {} comments.",
            post_count, page_count, media_count, cat_count, comment_count
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;
    use std::io::Cursor;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE wp_posts (
                ID INTEGER PRIMARY KEY AUTOINCREMENT,
                post_author INTEGER NOT NULL DEFAULT 1,
                post_date TEXT NOT NULL,
                post_date_gmt TEXT NOT NULL,
                post_content TEXT NOT NULL DEFAULT '',
                post_title TEXT NOT NULL DEFAULT '',
                post_excerpt TEXT NOT NULL DEFAULT '',
                post_status TEXT NOT NULL DEFAULT 'publish',
                comment_status TEXT NOT NULL DEFAULT 'open',
                ping_status TEXT NOT NULL DEFAULT 'open',
                post_password TEXT NOT NULL DEFAULT '',
                post_name TEXT NOT NULL DEFAULT '',
                to_ping TEXT NOT NULL DEFAULT '',
                pinged TEXT NOT NULL DEFAULT '',
                post_modified TEXT NOT NULL,
                post_modified_gmt TEXT NOT NULL,
                post_content_filtered TEXT NOT NULL DEFAULT '',
                post_parent INTEGER NOT NULL DEFAULT 0,
                guid TEXT NOT NULL DEFAULT '',
                menu_order INTEGER NOT NULL DEFAULT 0,
                post_type TEXT NOT NULL DEFAULT 'post',
                post_mime_type TEXT NOT NULL DEFAULT '',
                comment_count INTEGER NOT NULL DEFAULT 0,
                layout_json TEXT DEFAULT NULL
            );
            CREATE TABLE wp_postmeta (
                meta_id INTEGER PRIMARY KEY AUTOINCREMENT,
                post_id INTEGER NOT NULL DEFAULT 0,
                meta_key TEXT DEFAULT NULL,
                meta_value TEXT DEFAULT NULL
            );
            CREATE TABLE wp_options (
                option_id INTEGER PRIMARY KEY AUTOINCREMENT,
                option_name TEXT NOT NULL UNIQUE,
                option_value TEXT NOT NULL,
                autoload TEXT NOT NULL DEFAULT 'yes'
            );
            CREATE TABLE wp_terms (
                term_id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                slug TEXT NOT NULL UNIQUE,
                term_group INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE wp_term_taxonomy (
                term_taxonomy_id INTEGER PRIMARY KEY AUTOINCREMENT,
                term_id INTEGER NOT NULL,
                taxonomy TEXT NOT NULL DEFAULT 'category',
                description TEXT NOT NULL DEFAULT '',
                parent INTEGER NOT NULL DEFAULT 0,
                count INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE wp_term_relationships (
                object_id INTEGER NOT NULL,
                term_taxonomy_id INTEGER NOT NULL,
                term_order INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY (object_id, term_taxonomy_id)
            );
            CREATE TABLE wp_comments (
                comment_ID INTEGER PRIMARY KEY AUTOINCREMENT,
                comment_post_ID INTEGER NOT NULL DEFAULT 0,
                comment_author TEXT NOT NULL,
                comment_author_email TEXT NOT NULL DEFAULT '',
                comment_author_url TEXT NOT NULL DEFAULT '',
                comment_author_IP TEXT NOT NULL DEFAULT '',
                comment_date TEXT NOT NULL,
                comment_date_gmt TEXT NOT NULL,
                comment_content TEXT NOT NULL,
                comment_karma INTEGER NOT NULL DEFAULT 0,
                comment_approved TEXT NOT NULL DEFAULT '1',
                comment_agent TEXT NOT NULL DEFAULT '',
                comment_type TEXT NOT NULL DEFAULT 'comment',
                comment_parent INTEGER NOT NULL DEFAULT 0,
                user_id INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE wp_neopress_forms (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                title TEXT NOT NULL,
                schema_json TEXT NOT NULL DEFAULT '{}',
                settings_json TEXT NOT NULL DEFAULT '{}',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE wp_neopress_plugins (
                slug TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                version TEXT NOT NULL DEFAULT '1.0.0',
                description TEXT NOT NULL DEFAULT '',
                category TEXT NOT NULL DEFAULT 'general',
                is_builtin INTEGER NOT NULL DEFAULT 1,
                is_active INTEGER NOT NULL DEFAULT 1,
                manifest_json TEXT NOT NULL DEFAULT '{}',
                config_json TEXT NOT NULL DEFAULT '{}',
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            "#,
        )
        .unwrap();
        conn
    }

    #[test]
    fn test_wxr_stream_parsing_and_ingestion() {
        let mut conn = setup_test_db();
        let sample_xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:excerpt="http://wordpress.org/export/1.2/excerpt/"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wfw="http://wellformedweb.org/CommentAPI/"
	xmlns:dc="http://purl.org/dc/elements/1.1/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<title>Exported WP Site</title>
	<description>Testing streaming WXR</description>
	<wp:category>
		<wp:term_id>1</wp:term_id>
		<wp:category_nicename>engineering</wp:category_nicename>
		<wp:cat_name>Engineering</wp:cat_name>
	</wp:category>
	<wp:tag>
		<wp:term_id>2</wp:term_id>
		<wp:tag_slug>rust</wp:tag_slug>
		<wp:tag_name>Rust</wp:tag_name>
	</wp:tag>
	<item>
		<title>Hello Streaming World</title>
		<wp:post_name>hello-streaming-world</wp:post_name>
		<content:encoded><![CDATA[<p>Streaming import works seamlessly!</p>]]></content:encoded>
		<wp:post_date>2026-10-21 14:00:00</wp:post_date>
		<wp:status>publish</wp:status>
		<wp:post_type>post</wp:post_type>
		<category domain="category" nicename="engineering"><![CDATA[Engineering]]></category>
		<category domain="post_tag" nicename="rust"><![CDATA[Rust]]></category>
		<wp:postmeta>
			<wp:meta_key>_test_meta_key</wp:meta_key>
			<wp:meta_value>meta_value_123</wp:meta_value>
		</wp:postmeta>
		<wp:comment>
			<wp:comment_author>Alice</wp:comment_author>
			<wp:comment_content>Awesome WXR stream!</wp:comment_content>
			<wp:comment_date>2026-10-21 14:05:00</wp:comment_date>
		</wp:comment>
	</item>
	<item>
		<title>About This Site</title>
		<wp:post_name>about-this-site</wp:post_name>
		<content:encoded><![CDATA[<p>About page content.</p>]]></content:encoded>
		<wp:status>publish</wp:status>
		<wp:post_type>page</wp:post_type>
	</item>
	<item>
		<title>Diagram PNG</title>
		<wp:post_name>diagram-png</wp:post_name>
		<wp:post_type>attachment</wp:post_type>
		<wp:attachment_url>http://example.com/uploads/diagram.png</wp:attachment_url>
	</item>
</channel>
</rss>"#;

        let res = import_wxr_stream(Cursor::new(sample_xml), &mut conn).unwrap();
        assert_eq!(res.imported_posts, 1);
        assert_eq!(res.imported_pages, 1);
        assert_eq!(res.imported_media, 1);
        assert_eq!(res.imported_categories, 1);
        assert_eq!(res.imported_tags, 1);
        assert_eq!(res.imported_comments, 1);

        let post_title: String = conn
            .query_row(
                "SELECT post_title FROM wp_posts WHERE post_name = 'hello-streaming-world'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(post_title, "Hello Streaming World");

        let attachment_mime: String = conn
            .query_row(
                "SELECT post_mime_type FROM wp_posts WHERE post_name = 'diagram-png'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(attachment_mime, "image/png");

        let blog_title: String = conn
            .query_row(
                "SELECT option_value FROM wp_options WHERE option_name = 'blogname'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(blog_title, "Exported WP Site");
    }

    #[test]
    fn test_wxr_id_remapping_and_ast_transpilation() {
        let mut conn = setup_test_db();
        let sample_xml = r##"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
	xmlns:excerpt="http://wordpress.org/export/1.2/excerpt/"
	xmlns:content="http://purl.org/rss/1.0/modules/content/"
	xmlns:wp="http://wordpress.org/export/1.2/"
>
<channel>
	<title>Relational WXR Test</title>
	<description>Verify post_parent, _thumbnail_id, and AST transpilation</description>
	<item>
		<title>Hero Image</title>
		<wp:post_id>500</wp:post_id>
		<wp:post_name>hero-image</wp:post_name>
		<wp:post_type>attachment</wp:post_type>
		<wp:attachment_url>http://example.com/wp-content/uploads/hero.webp</wp:attachment_url>
	</item>
	<item>
		<title>Services Parent Page</title>
		<wp:post_id>100</wp:post_id>
		<wp:post_name>services</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<content:encoded><![CDATA[[et_pb_section background_color="#0f172a"][et_pb_row][et_pb_column type="4_4"][et_pb_text]<p>Divi Services Overview</p>[/et_pb_text][/et_pb_column][/et_pb_row][/et_pb_section]]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_thumbnail_id</wp:meta_key>
			<wp:meta_value>500</wp:meta_value>
		</wp:postmeta>
	</item>
	<item>
		<title>Rust Migration Child Page</title>
		<wp:post_id>200</wp:post_id>
		<wp:post_name>rust-migration</wp:post_name>
		<wp:post_type>page</wp:post_type>
		<wp:status>publish</wp:status>
		<wp:post_parent>100</wp:post_parent>
		<content:encoded><![CDATA[]]></content:encoded>
		<wp:postmeta>
			<wp:meta_key>_elementor_data</wp:meta_key>
			<wp:meta_value><![CDATA[[{"id":"sec1","elType":"section","settings":{"background_color":"#111827"},"elements":[{"id":"col1","elType":"column","settings":{"_column_size":100},"elements":[{"id":"h1","elType":"widget","widgetType":"heading","settings":{"title":"Sub-Millisecond Rust SSR","header_size":"h1"}}]}]}]]]></wp:meta_value>
		</wp:postmeta>
	</item>
</channel>
</rss>"##;

        let res = import_wxr_xml(sample_xml, &mut conn).unwrap();
        assert_eq!(res.imported_pages, 2);
        assert_eq!(res.imported_media, 1);

        let (media_new_id, media_mime): (i64, String) = conn
            .query_row(
                "SELECT ID, post_mime_type FROM wp_posts WHERE post_name = 'hero-image'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(media_mime, "image/webp");

        let (parent_new_id, parent_content, parent_layout): (i64, String, Option<String>) = conn
            .query_row(
                "SELECT ID, post_content, layout_json FROM wp_posts WHERE post_name = 'services'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert!(parent_layout.is_some());
        assert!(parent_content.contains("Divi Services Overview"));

        // Verify _thumbnail_id was remapped from 500 -> media_new_id
        let thumb_meta: String = conn
            .query_row(
                "SELECT meta_value FROM wp_postmeta WHERE post_id = ?1 AND meta_key = '_thumbnail_id'",
                params![parent_new_id],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(thumb_meta.parse::<i64>().unwrap(), media_new_id);

        // Verify child page post_parent was remapped from 100 -> parent_new_id and Elementor AST transpiled
        let (child_parent_id, child_content, child_layout): (i64, String, Option<String>) = conn
            .query_row(
                "SELECT post_parent, post_content, layout_json FROM wp_posts WHERE post_name = 'rust-migration'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(child_parent_id, parent_new_id);
        assert!(child_layout.is_some());
        assert!(child_content.contains("Sub-Millisecond Rust SSR"));
    }
}
