use super::*;
use crate::elementor::context::ElementorContext;
use crate::schema::{DaypressNode, NodeKind, StyleProperties};
use serde_json::Value;

pub(crate) fn map_form_widget(
    settings: &Value,
    _styles: &mut StyleProperties,
    children: &mut Vec<DaypressNode>,
    ctx: &ElementorContext,
) -> Option<NodeKind> {
    let form_id = settings
        .get("form_id")
        .and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
        })
        .unwrap_or(1);
    let title = settings
        .get("title")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let button_text = settings
        .get("button_text")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    let endpoint = settings
        .get("action")
        .or_else(|| settings.get("endpoint"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    if let Some(fields) = settings.get("form_fields").and_then(|f| f.as_array()) {
        let redirect_url = resolve_setting_link(settings, "redirect_to", ctx);
        let success_msg = settings
            .get("success_message")
            .and_then(|v| v.as_str())
            .unwrap_or("Uw inzending is succesvol verstuurd.");
        let btn_lbl = button_text.as_deref().unwrap_or("Verzenden");
        let custom_fields_html =
            compile_elementor_form_fields(fields, btn_lbl, success_msg, redirect_url.as_deref());
        children.push(DaypressNode {
            id: format!("form_fields_{}", form_id),
            kind: NodeKind::CustomHtml {
                raw: custom_fields_html,
            },
            styles: StyleProperties::default(),
            children: Vec::new(),
        });
    }

    Some(NodeKind::Form {
        form_id,
        title,
        button_text,
        endpoint,
    })
}

pub(crate) fn compile_elementor_form_fields(
    fields: &[Value],
    button_text: &str,
    success_message: &str,
    redirect_url: Option<&str>,
) -> String {
    let mut html = String::from("<div class=\"dp-form-fields dp-elementor-form-grid\">\n");
    for (idx, field) in fields.iter().enumerate() {
        let field_type = field
            .get("field_type")
            .and_then(|v| v.as_str())
            .unwrap_or("text");
        if field_type == "recaptcha" || field_type == "recaptcha_v3" || field_type == "honeypot" {
            continue;
        }
        let custom_id = field
            .get("custom_id")
            .or_else(|| field.get("_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("field");
        let name = if custom_id.is_empty() {
            format!("field_{}", idx)
        } else {
            custom_id.to_string()
        };
        let label = field
            .get("field_label")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let placeholder = field
            .get("placeholder")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let required = field
            .get("required")
            .and_then(|v| {
                v.as_str()
                    .map(|s| s == "true" || s == "yes")
                    .or_else(|| v.as_bool())
            })
            .unwrap_or(false);
        let req_attr = if required { " required" } else { "" };
        let req_mark = if required {
            " <span class=\"dp-form-required\">*</span>"
        } else {
            ""
        };
        let width_pct = field
            .get("width")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("100");
        let col_class = match width_pct {
            "50" => "dp-field-group dp-field-w50",
            "33" => "dp-field-group dp-field-w33",
            "25" => "dp-field-group dp-field-w25",
            _ => "dp-field-group dp-field-w100",
        };

        let label_clean = label
            .replace("<strong>", "")
            .replace("</strong>", "")
            .replace("<b>", "")
            .replace("</b>", "");

        html.push_str(&format!("<div class=\"{}\">\n", col_class));
        if !label_clean.is_empty() {
            html.push_str(&format!(
                "<label class=\"dp-field-label\">{}{}</label>\n",
                escape_html_attr(&label_clean),
                req_mark
            ));
        }

        match field_type {
            "textarea" => {
                let rows = field
                    .get("rows")
                    .and_then(extract_spacing_val)
                    .unwrap_or_else(|| "4".to_string());
                html.push_str(&format!(
                    "<textarea name=\"{}\" class=\"dp-textarea\" rows=\"{}\" placeholder=\"{}\"{}></textarea>\n",
                    escape_html_attr(&name),
                    escape_html_attr(&rows),
                    escape_html_attr(placeholder),
                    req_attr
                ));
            }
            "checkbox" | "radio" => {
                let opts_raw = field
                    .get("field_options")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let inline_cls = field
                    .get("inline_list")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|_| " dp-subgroup-inline")
                    .unwrap_or("");
                html.push_str(&format!(
                    "<div class=\"dp-field-subgroup{}\">\n",
                    inline_cls
                ));
                for opt in opts_raw.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
                    html.push_str(&format!(
                        "<label class=\"dp-option-label\"><input type=\"{}\" name=\"{}\" value=\"{}\" /> <span>{}</span></label>\n",
                        field_type,
                        escape_html_attr(&name),
                        escape_html_attr(opt),
                        escape_html_attr(opt)
                    ));
                }
                html.push_str("</div>\n");
            }
            "select" => {
                let opts_raw = field
                    .get("field_options")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                html.push_str(&format!(
                    "<select name=\"{}\" class=\"dp-input\"{}>\n",
                    escape_html_attr(&name),
                    req_attr
                ));
                for opt in opts_raw.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
                    html.push_str(&format!(
                        "<option value=\"{}\">{}</option>\n",
                        escape_html_attr(opt),
                        escape_html_attr(opt)
                    ));
                }
                html.push_str("</select>\n");
            }
            "upload" => {
                let mult =
                    if field.get("allow_multiple_upload").and_then(|v| v.as_str()) == Some("yes") {
                        " multiple"
                    } else {
                        ""
                    };
                html.push_str(&format!(
                    "<input type=\"file\" name=\"{}\" class=\"dp-input dp-file-input\"{}{}/>\n",
                    escape_html_attr(&name),
                    mult,
                    req_attr
                ));
            }
            "date" => {
                html.push_str(&format!(
                    "<input type=\"date\" name=\"{}\" class=\"dp-input\" placeholder=\"{}\"{} />\n",
                    escape_html_attr(&name),
                    escape_html_attr(placeholder),
                    req_attr
                ));
            }
            "email" | "tel" | "number" | "url" => {
                html.push_str(&format!(
                    "<input type=\"{}\" name=\"{}\" class=\"dp-input\" placeholder=\"{}\"{} />\n",
                    field_type,
                    escape_html_attr(&name),
                    escape_html_attr(placeholder),
                    req_attr
                ));
            }
            _ => {
                html.push_str(&format!(
                    "<input type=\"text\" name=\"{}\" class=\"dp-input\" placeholder=\"{}\"{} />\n",
                    escape_html_attr(&name),
                    escape_html_attr(placeholder),
                    req_attr
                ));
            }
        }
        html.push_str("</div>\n");
    }

    let redirect_attr = redirect_url
        .map(|u| format!(" data-redirect-url=\"{}\"", escape_html_attr(u)))
        .unwrap_or_default();
    html.push_str(&format!(
        "<div class=\"dp-field-group dp-field-w100\"><button type=\"submit\" class=\"dp-button dp-form-submit\">{}</button></div>\n</div>\n<div class=\"dp-form-status\" data-success-msg=\"{}\"{} aria-live=\"polite\"></div>",
        escape_html_attr(button_text),
        escape_html_attr(success_message),
        redirect_attr
    ));
    html
}
