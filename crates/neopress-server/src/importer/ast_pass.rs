use crate::importer::models::PendingAstItem;
use crate::importer::templates::upsert_theme_template;
use rusqlite::{params, Result, Transaction};
use std::collections::HashMap;

pub struct AstPassResult {
    pub header_html: String,
    pub footer_html: String,
    pub popups_html: String,
    pub has_divi_content: bool,
}

pub fn execute_ast_pass2(
    tx: &Transaction,
    pending_ast_items: Vec<PendingAstItem>,
    post_urls: &HashMap<String, String>,
    old_to_new_post_id: &HashMap<i64, i64>,
    menus_by_slug: &HashMap<String, Vec<neopress_ast::NavMenuItem>>,
    cf7_forms_map: &HashMap<String, String>,
    spb_services: &[neopress_ast::SpbServiceItem],
    detected_logo_url: Option<&str>,
    source_site_url: Option<&str>,
    detected_kit_raw_settings: Option<&str>,
    divi_header_old_id: Option<i64>,
    divi_footer_old_id: Option<i64>,
) -> Result<AstPassResult> {
    let post_urls_json = serde_json::to_string(post_urls).unwrap_or_else(|_| "{}".to_string());
    let menus_json = serde_json::to_string(menus_by_slug).unwrap_or_else(|_| "{}".to_string());
    let cf7_forms_json = serde_json::to_string(cf7_forms_map).unwrap_or_else(|_| "{}".to_string());
    let spb_services_json =
        serde_json::to_string(spb_services).unwrap_or_else(|_| "[]".to_string());

    let mut header_html = String::new();
    let mut footer_html = String::new();
    let mut popups_html = String::new();
    let mut has_divi_content = false;

    for mut item in pending_ast_items {
        let post_id = item.post_id;
        let old_id_opt = item.old_id;
        let p_title = &item.title;
        let p_type = &item.post_type;
        let p_status = &item.post_status;
        let mut raw_content = item.raw_content;
        let lib_type_opt = item.elementor_library_type;
        let meta = &mut item.postmeta;

        crate::importer::urls::remap_post_content_and_meta(&mut raw_content, meta, source_site_url);
        if raw_content.contains("[et_pb_")
            || p_type == "et_header_layout"
            || p_type == "et_footer_layout"
        {
            has_divi_content = true;
        }
        meta.insert("__np_post_urls_json".to_string(), post_urls_json.clone());
        meta.insert("__np_menus_json".to_string(), menus_json.clone());
        meta.insert("__np_cf7_forms_json".to_string(), cf7_forms_json.clone());
        meta.insert(
            "__np_spb_services_json".to_string(),
            spb_services_json.clone(),
        );
        if let Some(logo) = detected_logo_url {
            meta.insert("__np_site_logo_url".to_string(), logo.to_string());
        }
        if let Some(src) = source_site_url {
            meta.insert("__np_source_site_url".to_string(), src.to_string());
        }
        if let Some(kit_str) = detected_kit_raw_settings {
            meta.insert("_elementor_kit_settings".to_string(), kit_str.to_string());
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

            if p_type == "et_header_layout" && p_status == "publish" {
                if divi_header_old_id.is_none() || divi_header_old_id == old_id_opt {
                    header_html = neopress_ast::render_node_to_html(&transpiled_layout.root);
                    let _ = upsert_theme_template(
                        tx,
                        "Divi Header",
                        "header",
                        &header_html,
                        Some(&transpiled_layout),
                        100,
                    );
                }
            } else if p_type == "et_footer_layout" && p_status == "publish" {
                if divi_footer_old_id.is_none() || divi_footer_old_id == old_id_opt {
                    footer_html = neopress_ast::render_node_to_html(&transpiled_layout.root);
                    let _ = upsert_theme_template(
                        tx,
                        "Divi Footer",
                        "footer",
                        &footer_html,
                        Some(&transpiled_layout),
                        100,
                    );
                }
            } else {
                let is_elementor_lib = p_type == "elementor_library"
                    || meta.contains_key("_elementor_template_type")
                    || lib_type_opt.is_some();

                if is_elementor_lib && (p_status == "publish" || p_status.is_empty()) {
                    let raw_tpl_type = meta
                        .get("_elementor_template_type")
                        .map(|s| s.as_str())
                        .or(lib_type_opt.as_deref())
                        .unwrap_or("header");
                    let norm_type = crate::importer::templates::normalize_template_type(raw_tpl_type);
                    let body_without_kit_style =
                        neopress_ast::render_node_to_html(&transpiled_layout.root);

                    let raw_conds = meta.get("_elementor_conditions").map(|s| s.as_str());
                    let page_settings = meta.get("_elementor_page_settings").map(|s| s.as_str());

                    let _ = crate::importer::templates::ingest_elementor_library_template(
                        tx,
                        p_title,
                        norm_type,
                        &transpiled_layout,
                        &body_without_kit_style,
                        raw_conds,
                        page_settings,
                        100,
                        old_to_new_post_id,
                    );

                    let has_exclusions = raw_conds.map(|c| c.contains("exclude")).unwrap_or(false);
                    match norm_type {
                        "header" => {
                            if header_html.is_empty() && !has_exclusions {
                                header_html = body_without_kit_style.clone();
                            }
                        }
                        "footer" => {
                            if footer_html.is_empty() && !has_exclusions {
                                footer_html = body_without_kit_style.clone();
                            }
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
    }

    Ok(AstPassResult {
        header_html,
        footer_html,
        popups_html,
        has_divi_content,
    })
}
