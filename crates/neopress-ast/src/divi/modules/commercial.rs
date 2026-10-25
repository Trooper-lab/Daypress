use crate::divi::context::DiviContext;
use crate::divi::parser::DiviShortcode;
use crate::divi::widgets::{render_fallback_cf7_form, render_spb_calculator_widget};
use crate::schema::{NeopressNode, NodeKind, StyleProperties};

pub fn map_commercial_module(
    divi: &DiviShortcode,
    ctx: &DiviContext,
    styles: &mut StyleProperties,
    children: &mut Vec<NeopressNode>,
) -> Option<NodeKind> {
    match divi.tag.as_str() {
        "et_pb_pricing_tables" => {
            let cols = divi.children.len().clamp(1, 4) as u8;
            Some(NodeKind::Grid {
                columns: cols,
                gap: 24,
            })
        }
        "et_pb_pricing_table" => {
            let raw_title = divi
                .attributes
                .get("title")
                .cloned()
                .unwrap_or_else(|| "Plan".to_string());
            let subtitle = divi
                .attributes
                .get("subtitle")
                .filter(|s| !s.trim().is_empty())
                .cloned();
            let title = if let Some(sub) = subtitle {
                format!("{} ({})", raw_title, sub)
            } else {
                raw_title
            };
            let currency = divi
                .attributes
                .get("currency")
                .cloned()
                .unwrap_or_else(|| "€".to_string());
            let sum = divi
                .attributes
                .get("sum")
                .cloned()
                .unwrap_or_else(|| "29".to_string());
            let price = if sum.starts_with('$') || sum.starts_with('€') || sum.starts_with('£') {
                sum
            } else {
                format!("{}{}", currency, sum)
            };
            let period = divi
                .attributes
                .get("per")
                .filter(|p| !p.trim().is_empty())
                .map(|p| format!("/ {}", p))
                .unwrap_or_default();

            let clean_feature_line = |l: &str| -> String {
                l.trim()
                    .trim_start_matches("<p>")
                    .trim_end_matches("</p>")
                    .trim()
                    .trim_start_matches('+')
                    .trim_start_matches('-')
                    .trim()
                    .to_string()
            };

            let mut features: Vec<String> = divi
                .inner_text
                .lines()
                .map(clean_feature_line)
                .filter(|l| !l.is_empty())
                .collect();
            for child in &divi.children {
                let cf = clean_feature_line(&child.inner_text);
                if !cf.is_empty() {
                    features.push(cf);
                }
            }

            let button_text = divi
                .attributes
                .get("button_text")
                .cloned()
                .unwrap_or_default();
            let button_url = divi
                .attributes
                .get("button_url")
                .map(|u| ctx.resolve_divi_attr(u))
                .unwrap_or_else(|| "#".to_string());

            if divi
                .attributes
                .get("featured")
                .map(|v| v == "on")
                .unwrap_or(false)
            {
                let bdr = styles.border.get_or_insert_with(Default::default);
                bdr.color = Some("#3D2B01".to_string());
                bdr.width = Some("2px".to_string());
                bdr.style = Some("solid".to_string());
            }

            children.clear();
            Some(NodeKind::PriceTable {
                title,
                price,
                period,
                features,
                button_text,
                button_url,
            })
        }
        "et_pb_countdown_timer" => {
            let target_date = divi
                .attributes
                .get("date_time")
                .cloned()
                .unwrap_or_else(|| "2026-12-31 23:59:59".to_string());
            let show_days = divi
                .attributes
                .get("show_days")
                .map(|v| v != "off")
                .unwrap_or(true);
            let show_hours = divi
                .attributes
                .get("show_hours")
                .map(|v| v != "off")
                .unwrap_or(true);
            let show_minutes = divi
                .attributes
                .get("show_minutes")
                .map(|v| v != "off")
                .unwrap_or(true);
            let show_seconds = divi
                .attributes
                .get("show_seconds")
                .map(|v| v != "off")
                .unwrap_or(true);
            Some(NodeKind::Countdown {
                target_date,
                mode: "due_date".to_string(),
                show_days,
                show_hours,
                show_minutes,
                show_seconds,
                expiry_message: None,
            })
        }
        "et_pb_contact_form" => {
            let title = divi.attributes.get("title").cloned();
            let button_text = divi.attributes.get("submit_button_text").cloned();
            children.clear();
            Some(NodeKind::Form {
                form_id: 1,
                title,
                button_text,
                endpoint: None,
            })
        }
        "et_pb_signup" => {
            let title = divi
                .attributes
                .get("title")
                .cloned()
                .unwrap_or_else(|| "Subscribe".to_string());
            let button_text = divi
                .attributes
                .get("button_text")
                .cloned()
                .unwrap_or_else(|| "Subscribe".to_string());
            let endpoint = divi
                .attributes
                .get("provider")
                .cloned()
                .unwrap_or_else(|| "/api/subscribe".to_string());
            children.clear();
            Some(NodeKind::NativeForm {
                title,
                button_text,
                endpoint,
            })
        }
        "dvppl_cf7_styler" => {
            let cf7_id = divi
                .attributes
                .get("cf7")
                .cloned()
                .unwrap_or_else(|| "475".to_string());
            let form_html = ctx
                .cf7_forms
                .get(&cf7_id)
                .cloned()
                .or_else(|| ctx.cf7_forms.values().next().cloned())
                .unwrap_or_else(|| render_fallback_cf7_form(&cf7_id));
            children.clear();
            Some(NodeKind::CustomHtml {
                raw: format!("<div class=\"dvppl-cf7-styler\">{}</div>", form_html),
            })
        }
        "spb_kw_calculator_module" | "spb_kitwerk_calculator" => {
            children.clear();
            Some(NodeKind::CustomHtml {
                raw: render_spb_calculator_widget(&ctx.spb_services),
            })
        }
        _ => None,
    }
}
