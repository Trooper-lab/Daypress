//! Forms processing and CF7 / WPForms template compiler.

use crate::importer::models::ParsedWxrItem;
use rusqlite::{params, Result, Transaction};
use std::collections::HashMap;

/// Compiles a Contact Form 7 `_form` template into clean HTML wired to `/wp-json/daypress/v1/forms/{form_id}/submit`.
pub fn compile_cf7_form_template(raw_tpl: &str, form_id: i64) -> String {
    let template_part = raw_tpl
        .lines()
        .take_while(|line| {
            let t = line.trim();
            !t.starts_with("1\n") && t != "1" && !t.starts_with("[_site_")
        })
        .collect::<Vec<_>>()
        .join("\n")
        .replace("[dipe_row]", "<div class=\"form-row\">")
        .replace("[/dipe_row]", "</div>")
        .replace("[dipe_one_half]", "<div class=\"form-column\">")
        .replace("[/dipe_one_half]", "</div>")
        .replace("[dipe_one_third]", "<div class=\"form-column\">")
        .replace("[/dipe_one_third]", "</div>");

    let tag_re = regex::Regex::new(
        r#"\[(text\*?|email\*?|tel\*?|textarea\*?|number\*?|date\*?|select\*?|submit)\s+([^\]]+)\]"#,
    )
    .expect("valid cf7 regex");

    let rendered_fields = tag_re.replace_all(&template_part, |caps: &regex::Captures| {
        let kind = caps.get(1).map(|m| m.as_str()).unwrap_or("text");
        let rest = caps.get(2).map(|m| m.as_str()).unwrap_or("").trim();
        let required = kind.ends_with('*');
        let base_kind = kind.trim_end_matches('*');
        let req_attr = if required { " required" } else { "" };

        if base_kind == "submit" {
            let label = rest.trim_matches('"').trim();
            let btn_text = if label.is_empty() { "Submit" } else { label };
            return format!(
                r#"<button type="submit" class="wpcf7-form-control wpcf7-submit dp-btn-submit">{}</button>"#,
                btn_text
            );
        }

        let mut parts = rest.splitn(2, char::is_whitespace);
        let field_name = parts.next().unwrap_or("field").trim();
        let remainder = parts.next().unwrap_or("").trim();
        let placeholder = if let Some(start) = remainder.find('"') {
            let after = &remainder[start + 1..];
            if let Some(end) = after.find('"') {
                &after[..end]
            } else {
                ""
            }
        } else {
            ""
        };
        let ph_attr = if !placeholder.is_empty() {
            format!(r#" placeholder="{}""#, placeholder)
        } else {
            String::new()
        };

        match base_kind {
            "textarea" => format!(
                r#"<span class="wpcf7-form-control-wrap"><textarea name="{name}" rows="5" class="wpcf7-form-control wpcf7-textarea"{ph}{req}></textarea></span>"#,
                name = field_name,
                ph = ph_attr,
                req = req_attr
            ),
            "email" => format!(
                r#"<span class="wpcf7-form-control-wrap"><input type="email" name="{name}" class="wpcf7-form-control wpcf7-text wpcf7-email"{ph}{req} /></span>"#,
                name = field_name,
                ph = ph_attr,
                req = req_attr
            ),
            "tel" => format!(
                r#"<span class="wpcf7-form-control-wrap"><input type="tel" name="{name}" class="wpcf7-form-control wpcf7-text wpcf7-tel"{ph}{req} /></span>"#,
                name = field_name,
                ph = ph_attr,
                req = req_attr
            ),
            _ => format!(
                r#"<span class="wpcf7-form-control-wrap"><input type="text" name="{name}" class="wpcf7-form-control wpcf7-text"{ph}{req} /></span>"#,
                name = field_name,
                ph = ph_attr,
                req = req_attr
            ),
        }
    });

    let body_html = if rendered_fields.contains("<div") {
        rendered_fields.into_owned()
    } else {
        rendered_fields
            .split("\n\n")
            .map(|b| b.trim())
            .filter(|b| !b.is_empty())
            .map(|b| {
                if b.starts_with("<p") || b.starts_with("<div") {
                    b.to_string()
                } else {
                    format!("<p>{}</p>", b.replace('\n', "<br />"))
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };

    format!(
        r#"<div class="wpcf7"><form class="wpcf7-form dp-cf7-form" onsubmit="event.preventDefault(); var f=this; var fd=new FormData(f); var obj={{}}; fd.forEach(function(v,k){{obj[k]=v;}}); fetch('/wp-json/daypress/v1/forms/{form_id}/submit', {{method:'POST', headers:{{'Content-Type':'application/json'}}, body:JSON.stringify(obj)}}).then(function(){{ var m=f.parentElement.querySelector('.wpcf7-response-output'); if(m){{m.style.display='block'; m.textContent='Thank you for your message. It has been sent.';}} f.reset(); }});">{fields}</form><div class="wpcf7-response-output" style="display:none; margin-top:12px; padding:10px 14px; border:2px solid #46b450; border-radius:4px; font-size:14px;"></div></div>"#,
        form_id = form_id,
        fields = body_html
    )
}

/// Ingests CF7 and WPForms from parsed items into `daypress_forms`.
pub fn ingest_forms_and_spb(
    tx: &Transaction,
    posts: &[ParsedWxrItem],
    now: &str,
) -> Result<(
    HashMap<i64, i64>,
    HashMap<String, String>,
    Option<i64>,
    Vec<daypress_ast::SpbServiceItem>,
    Option<i64>,
    Option<i64>,
    String,
    std::collections::BTreeSet<String>,
)> {
    let mut form_id_map: HashMap<i64, i64> = HashMap::new();
    let mut cf7_forms_map: HashMap<String, String> = HashMap::new();
    let mut spb_services: Vec<daypress_ast::SpbServiceItem> = Vec::new();
    let mut divi_header_old_id: Option<i64> = None;
    let mut divi_footer_old_id: Option<i64> = None;
    let mut accumulated_custom_css = String::new();
    let mut enqueued_google_fonts: std::collections::BTreeSet<String> =
        std::collections::BTreeSet::new();
    let mut first_migrated_form_id: Option<i64> = None;
    let mut has_daystudio_forms = false;
    let mut has_spb_calculator = false;

    for item in posts {
        if item
            .postmeta
            .get("_elementor_data")
            .map(|d| d.contains("\"widgetType\":\"form\""))
            .unwrap_or(false)
        {
            has_daystudio_forms = true;
        }
        if item.content.contains("[spb_kw_calculator_module")
            || item.content.contains("[spb_kitwerk_calculator")
        {
            has_spb_calculator = true;
        }
        if item.post_type == "spb_service" && item.post_status == "publish" {
            has_spb_calculator = true;
            let price = item
                .postmeta
                .get("_spb_kw_price")
                .and_then(|s| s.trim().parse::<f64>().ok())
                .unwrap_or(0.0);
            let unit = item
                .postmeta
                .get("_spb_kw_unit")
                .cloned()
                .unwrap_or_default();
            let is_extra = item
                .postmeta
                .get("_spb_kw_is_extra")
                .map(|v| v == "1" || v == "true")
                .unwrap_or(false);
            spb_services.push(daypress_ast::SpbServiceItem {
                id: item.id.unwrap_or(0),
                title: item.title.clone(),
                price,
                unit,
                is_extra,
            });
        }
        if item.post_type == "custom_css"
            && item.post_status == "publish"
            && !item.content.trim().is_empty()
        {
            accumulated_custom_css.push_str(&item.content);
            accumulated_custom_css.push('\n');
        }
        if let Some(fonts_raw) = item.postmeta.get("et_enqueued_post_fonts") {
            for fam in crate::importer::assets::extract_divi_enqueued_fonts(fonts_raw) {
                enqueued_google_fonts.insert(fam);
            }
        }
        if item.post_type == "et_template"
            && item.post_status == "publish"
            && !item
                .postmeta
                .contains_key("_et_theme_builder_marked_as_unused")
        {
            if item
                .postmeta
                .get("_et_header_layout_enabled")
                .map(|v| v == "1")
                .unwrap_or(true)
            {
                if let Some(hid) = item
                    .postmeta
                    .get("_et_header_layout_id")
                    .and_then(|s| s.trim().parse::<i64>().ok())
                    .filter(|&id| id > 0)
                {
                    divi_header_old_id = Some(hid);
                }
            }
            if item
                .postmeta
                .get("_et_footer_layout_enabled")
                .map(|v| v == "1")
                .unwrap_or(true)
            {
                if let Some(fid) = item
                    .postmeta
                    .get("_et_footer_layout_id")
                    .and_then(|s| s.trim().parse::<i64>().ok())
                    .filter(|&id| id > 0)
                {
                    divi_footer_old_id = Some(fid);
                }
            }
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
                    "INSERT INTO daypress_forms (title, schema_json, settings_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)",
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
                if item.post_type == "wpcf7_contact_form" {
                    let raw_tpl = item
                        .postmeta
                        .get("_form")
                        .filter(|s| !s.trim().is_empty())
                        .cloned()
                        .unwrap_or_else(|| {
                            item.content
                                .split("\n1\n")
                                .next()
                                .unwrap_or(&item.content)
                                .to_string()
                        });
                    let compiled_html = compile_cf7_form_template(&raw_tpl, new_fid);
                    cf7_forms_map.insert(new_fid.to_string(), compiled_html.clone());
                    if let Some(old_fid) = item.id {
                        cf7_forms_map.insert(old_fid.to_string(), compiled_html.clone());
                    }
                    if let Some(hash) = item.postmeta.get("_hash").filter(|h| !h.is_empty()) {
                        cf7_forms_map.insert(hash.clone(), compiled_html.clone());
                        let prefix: String = hash.chars().take(7).collect();
                        cf7_forms_map.insert(prefix, compiled_html.clone());
                    }
                    if !item.title.trim().is_empty() {
                        cf7_forms_map.insert(item.title.trim().to_lowercase(), compiled_html);
                    }
                }
            }
        }
    }

    if has_daystudio_forms || has_spb_calculator {
        let existing_forms: i64 = tx
            .query_row("SELECT COUNT(*) FROM daypress_forms", [], |r| r.get(0))
            .unwrap_or(0);
        if existing_forms == 0 {
            let _ = tx.execute(
                "INSERT INTO daypress_forms (id, title, schema_json, settings_json, created_at, updated_at) VALUES (1, 'Migrated Lead Form', '[]', '{}', ?1, ?1)",
                params![now],
            );
        }
    }
    if !form_id_map.is_empty() || has_daystudio_forms || has_spb_calculator {
        let _ = crate::convergence::activate_builtin_plugin_tx(tx, "daypress-forms", None);
    }

    Ok((
        form_id_map,
        cf7_forms_map,
        first_migrated_form_id,
        spb_services,
        divi_header_old_id,
        divi_footer_old_id,
        accumulated_custom_css,
        enqueued_google_fonts,
    ))
}
