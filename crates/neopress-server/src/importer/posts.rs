//! Pass 1 Post, Page, CPT & Media Ingestion.

use crate::importer::models::{ParsedWxrItem, PendingAstItem};
use crate::importer::urls::{
    compute_post_permalink, detect_logo_attachment, infer_attachment_mime, remap_post_parents,
    resolve_attachment_guid,
};
use rusqlite::{params, Result, Transaction};
use std::collections::HashMap;

pub struct Pass1Result {
    pub post_count: usize,
    pub page_count: usize,
    pub media_count: usize,
    pub comment_count: usize,
    pub old_to_new_post_id: HashMap<i64, i64>,
    pub post_urls: HashMap<String, String>,
    pub sticky_post_ids: Vec<i64>,
    pub pending_ast_items: Vec<PendingAstItem>,
    pub raw_nav_menu_items: Vec<(i64, String, i64, Option<String>, HashMap<String, String>)>,
    pub detected_logo_url: Option<String>,
    pub detected_kit_raw_settings: Option<String>,
}

pub fn ingest_posts_pass1(
    tx: &Transaction,
    posts: Vec<ParsedWxrItem>,
    wxr_site_title: &str,
    form_id_map: &HashMap<i64, i64>,
    first_migrated_form_id: Option<i64>,
    cf7_forms_map: &HashMap<String, String>,
    now: &str,
    source_site_url: Option<&str>,
) -> Result<Pass1Result> {
    let mut post_count = 0;
    let mut page_count = 0;
    let mut media_count = 0;
    let mut comment_count = 0;

    let mut old_to_new_post_id: HashMap<i64, i64> = HashMap::new();
    let mut post_urls: HashMap<String, String> = HashMap::new();
    let mut pending_parent_remaps: Vec<(i64, i64)> = Vec::new();
    let mut old_to_new_comment_id: HashMap<i64, i64> = HashMap::new();
    let mut pending_comment_parent_remaps: Vec<(i64, i64)> = Vec::new();
    let mut sticky_post_ids: Vec<i64> = Vec::new();
    let mut raw_nav_menu_items: Vec<(i64, String, i64, Option<String>, HashMap<String, String>)> =
        Vec::new();

    let mut detected_logo_url: Option<String> = None;
    let mut detected_kit_raw_settings: Option<String> = None;
    let mut pending_ast_items: Vec<PendingAstItem> = Vec::new();

    let mut detected_front_page_old_id: Option<i64> = None;
    for item in &posts {
        if item.post_type == "nav_menu_item"
            && item.title.trim().eq_ignore_ascii_case("home")
            && item.postmeta.get("_menu_item_object").map(|s| s.as_str()) == Some("page")
        {
            if let Some(oid) = item
                .postmeta
                .get("_menu_item_object_id")
                .and_then(|v| v.trim().parse::<i64>().ok())
            {
                detected_front_page_old_id = Some(oid);
                break;
            }
        }
    }

    for mut item in posts {
        if matches!(item.post_type.as_str(), "wpcf7_contact_form" | "wpforms") {
            continue;
        }
        if !item.content.contains("[et_pb_")
            && !item.content.contains("[dvppl_cf7_styler")
            && cf7_forms_map.is_empty()
        {
            item.content = crate::convergence::rewrite_legacy_form_shortcodes(
                &item.content,
                form_id_map,
                first_migrated_form_id,
            );
        }
        let _ = crate::convergence::converge_post_seo_meta(
            &mut item.postmeta,
            &item.title,
            wxr_site_title,
        );

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
            now
        } else {
            &item.post_date
        };
        let p_type = &item.post_type;
        let p_status = &item.post_status;
        let is_sticky_val: i64 = if item.is_sticky { 1 } else { 0 };

        let raw_guid = if !item.guid.is_empty() {
            item.guid.clone()
        } else if !item.attachment_url.is_empty() {
            item.attachment_url.clone()
        } else {
            String::new()
        };
        let guid = if p_type == "attachment" {
            resolve_attachment_guid(&item.attachment_url, &raw_guid, &item.postmeta)
        } else {
            raw_guid
        };

        let inferred_mime = if p_type == "attachment" {
            infer_attachment_mime(
                &item.post_mime_type,
                &item.attachment_url,
                &guid,
                &slug,
                &item.postmeta,
            )
        } else {
            item.post_mime_type.clone()
        };

        detect_logo_attachment(p_type, &slug, &item.title, &guid, &mut detected_logo_url);

        // Detect Elementor Kit
        let is_kit = p_type == "elementor_kit"
            || item.elementor_library_type.as_deref() == Some("kit")
            || item
                .postmeta
                .get("_elementor_template_type")
                .map(|s| s.as_str())
                == Some("kit")
            || ((p_type == "elementor_snippet" || p_type == "elementor_library")
                && (slug.contains("kit") || item.title.to_lowercase().contains("kit")))
            || item
                .postmeta
                .get("_elementor_page_settings")
                .map(|s| s.contains("system_colors"))
                .unwrap_or(false);

        if is_kit {
            if let Some(kit_data) = item.postmeta.get("_elementor_page_settings") {
                if !kit_data.trim().is_empty() {
                    detected_kit_raw_settings = Some(kit_data.clone());
                }
            }
        }

        let is_leeg_template = item.postmeta.get("_wp_page_template").map(|s| s.as_str())
            == Some("page_templates/leeg.php")
            || item
                .postmeta
                .get("custom_html")
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);

        let mut initial_content = if is_leeg_template {
            if let Some(ch) = item
                .postmeta
                .get("custom_html")
                .filter(|s| !s.trim().is_empty())
            {
                ch.clone()
            } else {
                item.content.clone()
            }
        } else {
            item.content.clone()
        };

        crate::importer::urls::remap_post_content_and_meta(
            &mut initial_content,
            &mut item.postmeta,
            source_site_url,
        );

        let post_id: i64 = match tx.query_row(
            "SELECT ID FROM wp_posts WHERE post_name = ?1 AND post_type = ?2",
            params![slug, p_type],
            |r| r.get(0),
        ) {
            Ok(id) => {
                tx.execute(
                    "UPDATE wp_posts SET post_title = ?1, post_content = ?2, post_excerpt = ?3, post_status = ?4, post_date = ?5, post_mime_type = ?6, post_parent = ?7, menu_order = ?8, is_sticky = ?9 WHERE ID = ?10",
                    params![item.title, initial_content, item.excerpt, p_status, p_date, inferred_mime, item.parent, item.menu_order, is_sticky_val, id],
                )?;
                id
            }
            Err(_) => {
                tx.execute(
                    r#"INSERT INTO wp_posts (post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type, post_mime_type, post_parent, menu_order, is_sticky)
                       VALUES (1, ?1, ?1, ?2, ?3, ?4, ?5, ?6, ?1, ?1, ?7, ?8, ?9, ?10, ?11, ?12)"#,
                    params![p_date, initial_content, item.title, item.excerpt, p_status, slug, guid, p_type, inferred_mime, item.parent, item.menu_order, is_sticky_val],
                )?;
                tx.last_insert_rowid()
            }
        };

        if item.is_sticky {
            sticky_post_ids.push(post_id);
        }

        let site_slug = generated_slugify::slugify(wxr_site_title);
        let is_temp =
            item.title.to_lowercase().contains("temp") || slug.to_lowercase().contains("temp");
        let is_front_page = p_type == "page"
            && p_status == "publish"
            && !is_temp
            && ((item.id.is_some() && item.id == detected_front_page_old_id)
                || slug.eq_ignore_ascii_case("home")
                || slug.eq_ignore_ascii_case("front-page")
                || item
                    .title
                    .trim()
                    .eq_ignore_ascii_case(wxr_site_title.trim())
                || (!site_slug.is_empty() && slug.eq_ignore_ascii_case(&site_slug)));
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

        let permalink = compute_post_permalink(&slug, p_type, is_front_page);
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

        let is_page = p_type == "page";
        let needs_ast = item
            .postmeta
            .get("_elementor_data")
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false)
            || initial_content.contains("[et_pb_")
            || initial_content.contains("[spb_kw_calculator_module")
            || initial_content.contains("[spb_kitwerk_calculator")
            || initial_content.contains("<!-- wp:")
            || (is_page && initial_content.contains("<section"));

        if needs_ast {
            pending_ast_items.push(PendingAstItem {
                post_id,
                old_id: item.id,
                title: item.title.clone(),
                post_type: p_type.clone(),
                post_status: item.post_status.clone(),
                raw_content: initial_content,
                elementor_library_type: item.elementor_library_type.clone(),
                postmeta: item.postmeta.clone(),
            });
        }

        // Insert postmeta
        for (k, v) in &item.postmeta {
            let _ = tx.execute(
                "INSERT INTO wp_postmeta (post_id, meta_key, meta_value) VALUES (?1, ?2, ?3)",
                params![post_id, k, v],
            );
        }

        // Insert term relationships
        let _ = crate::importer::terms::ingest_post_terms(
            tx,
            post_id,
            &item.categories,
            &item.tags,
            &item.custom_terms,
        );

        // Comments
        comment_count += crate::importer::comments::ingest_post_comments(
            tx,
            post_id,
            &item.comments,
            now,
            &mut old_to_new_comment_id,
            &mut pending_comment_parent_remaps,
        )?;

        match p_type.as_str() {
            "page" => page_count += 1,
            "attachment" => media_count += 1,
            "nav_menu_item" | "elementor_library" | "et_header_layout" | "et_footer_layout"
            | "et_template" | "et_pb_layout" | "custom_css" | "wp_global_styles"
            | "wp_navigation" | "spb_service" => {}
            _ => post_count += 1,
        }
    }

    // Remap parent IDs
    remap_post_parents(tx, &pending_parent_remaps, &old_to_new_post_id)?;

    // Remap comment parent IDs
    crate::importer::comments::remap_comment_parents(
        tx,
        &pending_comment_parent_remaps,
        &old_to_new_comment_id,
    )?;

    Ok(Pass1Result {
        post_count,
        page_count,
        media_count,
        comment_count,
        old_to_new_post_id,
        post_urls,
        sticky_post_ids,
        pending_ast_items,
        raw_nav_menu_items,
        detected_logo_url,
        detected_kit_raw_settings,
    })
}
