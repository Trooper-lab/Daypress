//! Modular WordPress WXR & JSON Importer Subsystem.

pub mod assets;
pub mod ast_pass;
pub mod comments;
pub mod forms;
pub mod menus;
pub mod models;
pub mod posts;
pub mod templates;
pub mod terms;
pub mod urls;
pub mod xml_stream;

#[cfg(test)]
mod tests;

pub use models::{infer_mime_type_from_filename, ParsedWxrComment, ParsedWxrItem, ParsedWxrTerm};
pub use templates::upsert_theme_template;
pub use urls::normalize_menu_url;

use crate::errors::AppError;
use crate::importer::assets::{
    build_divi_global_css, persist_elementor_kit_styles, resolve_and_persist_theme_assets,
};
use crate::importer::ast_pass::execute_ast_pass2;
use crate::importer::forms::ingest_forms_and_spb;
use crate::importer::menus::ingest_nav_menus;
use crate::importer::posts::ingest_posts_pass1;
use crate::importer::templates::{build_universal_theme_footer_html, build_universal_theme_header_html};
use crate::importer::terms::ingest_terms;
use crate::importer::urls::remap_post_thumbnails;
use crate::importer::xml_stream::parse_wxr_stream;
use crate::models::WxrImportResponse;
use rusqlite::{params, Connection};
use std::io::{BufRead, Cursor};

/// Convenience wrapper for importing a WXR XML string directly into SQLite storage.
pub fn import_wxr_xml(xml: &str, conn: &mut Connection) -> Result<WxrImportResponse, AppError> {
    let cleaned: Vec<u8> = xml
        .as_bytes()
        .iter()
        .copied()
        .filter(|&b| b >= 0x20 || b == b'\t' || b == b'\n' || b == b'\r')
        .collect();
    import_wxr_stream(Cursor::new(cleaned), conn)
}

/// Ingests a streaming WordPress XML (WXR) export into SQLite storage.
pub fn import_wxr_stream<R: BufRead>(
    reader: R,
    conn: &mut Connection,
) -> Result<WxrImportResponse, AppError> {
    let (
        site_settings,
        source_site_url,
        site_language,
        site_favicon_url,
        terms_to_insert,
        posts_to_insert,
    ) = parse_wxr_stream(reader)?;

    let tx = conn.transaction()?;

    let wxr_site_title = site_settings
        .title
        .clone()
        .unwrap_or_else(|| "Neopress".to_string());

    if let Some(ref t) = site_settings.title {
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
                DELETE FROM wp_terms;
                DELETE FROM wp_term_taxonomy;
                DELETE FROM wp_term_relationships;
                DELETE FROM wp_neopress_templates;
                DELETE FROM wp_neopress_template_conditions;
                "#,
            );
        }
        let _ = tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_site_migrated', '1') \
             ON CONFLICT(option_name) DO UPDATE SET option_value = '1';",
            [],
        );
        tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('blogname', ?1) \
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![t],
        )?;
    }

    if let Some(ref d) = site_settings.description {
        tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('blogdescription', ?1) \
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![d],
        )?;
    }
    if let Some(ref src_url) = source_site_url {
        tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_source_site_url', ?1) \
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![src_url],
        )?;
    }
    if let Some(ref lang) = site_language {
        tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('WPLANG', ?1) \
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![lang],
        )?;
    }
    if let Some(ref fav_raw) = site_favicon_url {
        let fav_norm = normalize_menu_url(fav_raw);
        tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_site_favicon', ?1) \
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![fav_norm],
        )?;
    }

    // 1. Commit Taxonomies
    let (cat_count, tag_count) = ingest_terms(&tx, terms_to_insert)?;

    // 2. Forms & SPB Services Pass
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let (
        form_id_map,
        cf7_forms_map,
        first_migrated_form_id,
        spb_services,
        divi_header_old_id,
        divi_footer_old_id,
        accumulated_custom_css,
        enqueued_google_fonts,
    ) = ingest_forms_and_spb(&tx, &posts_to_insert, &now)?;

    // 3. Pass 1 Posts Ingestion
    let pass1 = ingest_posts_pass1(
        &tx,
        posts_to_insert,
        &wxr_site_title,
        &form_id_map,
        first_migrated_form_id,
        &cf7_forms_map,
        &now,
        source_site_url.as_deref(),
    )?;

    if let Some(ref kit_raw) = pass1.detected_kit_raw_settings {
        let _ = persist_elementor_kit_styles(&tx, kit_raw);
    }

    if let Some(ref logo_url) = pass1.detected_logo_url {
        let _ = tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_site_logo', ?1) \
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![logo_url],
        );
    }

    // Persist sticky posts array to wp_options
    let sticky_json = serde_json::to_string(&pass1.sticky_post_ids).unwrap_or_else(|_| "[]".to_string());
    let _ = tx.execute(
        "INSERT INTO wp_options (option_name, option_value) VALUES ('sticky_posts', ?1) \
         ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
        params![sticky_json],
    );

    // Remap thumbnail IDs
    remap_post_thumbnails(&tx, &pass1.old_to_new_post_id)?;

    // 4. Ingest Nav Menus
    let menus_by_slug = ingest_nav_menus(
        &tx,
        pass1.raw_nav_menu_items,
        &pass1.old_to_new_post_id,
        &pass1.post_urls,
        source_site_url.as_deref(),
    )?;

    // 5. Pass 2 AST Transpilation
    let ast_res = execute_ast_pass2(
        &tx,
        pass1.pending_ast_items,
        &pass1.post_urls,
        &pass1.old_to_new_post_id,
        &menus_by_slug,
        &cf7_forms_map,
        &spb_services,
        pass1.detected_logo_url.as_deref(),
        source_site_url.as_deref(),
        pass1.detected_kit_raw_settings.as_deref(),
        divi_header_old_id,
        divi_footer_old_id,
    )?;

    let mut header_html = ast_res.header_html;
    let mut footer_html = ast_res.footer_html;

    if header_html.is_empty() && !menus_by_slug.is_empty() {
        header_html = build_universal_theme_header_html(
            &menus_by_slug,
            &wxr_site_title,
            pass1.detected_logo_url.as_deref(),
        );
        let _ = upsert_theme_template(&tx, "Theme Header", "header", &header_html, None, 100);
    }
    if footer_html.is_empty() && !menus_by_slug.is_empty() {
        footer_html = build_universal_theme_footer_html(
            &menus_by_slug,
            &wxr_site_title,
            pass1.detected_logo_url.as_deref(),
        );
        let _ = upsert_theme_template(&tx, "Theme Footer", "footer", &footer_html, None, 100);
    }

    // 6. Global CSS & Assets Manifest
    if ast_res.has_divi_content || !accumulated_custom_css.trim().is_empty() || !spb_services.is_empty() {
        let combined_css = build_divi_global_css(
            &enqueued_google_fonts,
            &accumulated_custom_css,
            !spb_services.is_empty(),
            ast_res.has_divi_content,
        );
        if !combined_css.trim().is_empty() {
            let _ = tx.execute(
                "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_custom_css', ?1) \
                 ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
                params![combined_css],
            );
        }
    }

    resolve_and_persist_theme_assets(
        &tx,
        &wxr_site_title,
        site_favicon_url,
        pass1.detected_logo_url,
        &accumulated_custom_css,
    )?;

    if !header_html.is_empty() {
        let _ = tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_elementor_header_html', ?1) \
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![header_html],
        );
    }
    if !footer_html.is_empty() {
        let _ = tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_elementor_footer_html', ?1) \
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![footer_html],
        );
    }
    if !ast_res.popups_html.is_empty() {
        let _ = tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES ('neopress_elementor_popups_html', ?1) \
             ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![ast_res.popups_html],
        );
    }

    tx.commit()?;

    Ok(WxrImportResponse {
        success: true,
        imported_posts: pass1.post_count,
        imported_pages: pass1.page_count,
        imported_media: pass1.media_count,
        imported_categories: cat_count,
        imported_tags: tag_count,
        imported_comments: pass1.comment_count,
        message: format!(
            "Successfully imported {} posts, {} pages, {} media items, {} categories, and {} comments.",
            pass1.post_count, pass1.page_count, pass1.media_count, cat_count, pass1.comment_count
        ),
    })
}
