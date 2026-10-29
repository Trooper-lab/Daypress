use crate::divi::context::DiviContext;
use crate::divi::parser::DiviShortcode;
use crate::schema::{AccordionItem, DaypressNode, NodeKind, TabItem, ToggleItem};

pub fn map_interactive_module(
    divi: &DiviShortcode,
    ctx: &DiviContext,
    children: &mut Vec<DaypressNode>,
) -> Option<NodeKind> {
    match divi.tag.as_str() {
        "et_pb_counters" | "et_pb_bar_counters" => Some(NodeKind::Container),
        "et_pb_counter" => {
            let percent = divi
                .attributes
                .get("percent")
                .and_then(|p| p.trim_end_matches('%').parse::<u8>().ok())
                .unwrap_or(100);
            let title = if !divi.inner_text.trim().is_empty() {
                divi.inner_text.trim().to_string()
            } else {
                divi.attributes
                    .get("title")
                    .cloned()
                    .unwrap_or_else(|| "Progress".to_string())
            };
            Some(NodeKind::ProgressBar {
                title,
                percent,
                display_text: Some(format!("{}%", percent)),
                track_color: None,
                bar_color: None,
                height: None,
            })
        }
        "et_pb_number_counter" | "et_pb_circle_counter" => {
            let number = divi
                .attributes
                .get("number")
                .or_else(|| divi.attributes.get("percent"))
                .and_then(|p| p.trim_end_matches('%').parse::<i64>().ok())
                .unwrap_or(100);
            let percent_sign = divi
                .attributes
                .get("percent_sign")
                .map(|s| s == "on")
                .unwrap_or(true);
            let suffix = if percent_sign {
                Some("%".to_string())
            } else {
                None
            };
            let title = divi.attributes.get("title").cloned().or_else(|| {
                if !divi.inner_text.trim().is_empty() {
                    Some(divi.inner_text.trim().to_string())
                } else {
                    None
                }
            });
            Some(NodeKind::Counter {
                number,
                prefix: None,
                suffix,
                title,
            })
        }
        "et_pb_accordion" => {
            let items: Vec<AccordionItem> = divi
                .children
                .iter()
                .filter(|c| {
                    c.tag == "et_pb_accordion_item"
                        || c.tag == "et_pb_toggle"
                        || !c.inner_text.is_empty()
                })
                .map(|c| AccordionItem {
                    title: c
                        .attributes
                        .get("title")
                        .cloned()
                        .unwrap_or_else(|| "Accordion Item".to_string()),
                    content_html: ctx.normalize_html(c.inner_text.trim()),
                })
                .collect();
            children.clear();
            Some(NodeKind::Accordion { items })
        }
        "et_pb_toggle" => {
            let title = divi
                .attributes
                .get("title")
                .cloned()
                .unwrap_or_else(|| "Toggle".to_string());
            let content_html = ctx.normalize_html(divi.inner_text.trim());
            Some(NodeKind::Toggle {
                items: vec![ToggleItem {
                    title,
                    content_html,
                }],
            })
        }
        "et_pb_tabs" => {
            let items: Vec<TabItem> = divi
                .children
                .iter()
                .filter(|c| c.tag == "et_pb_tab" || !c.inner_text.is_empty())
                .map(|c| TabItem {
                    title: c
                        .attributes
                        .get("title")
                        .cloned()
                        .unwrap_or_else(|| "Tab".to_string()),
                    content_html: ctx.normalize_html(c.inner_text.trim()),
                })
                .collect();
            children.clear();
            Some(NodeKind::Tabs { items })
        }
        "et_pb_testimonial" => {
            let content = ctx.normalize_html(divi.inner_text.trim());
            let name = divi
                .attributes
                .get("author")
                .cloned()
                .unwrap_or_else(|| "Client".to_string());
            let job_title = divi
                .attributes
                .get("job_title")
                .cloned()
                .unwrap_or_default();
            let company = divi
                .attributes
                .get("company_name")
                .cloned()
                .unwrap_or_default();
            let job = match (job_title.is_empty(), company.is_empty()) {
                (false, false) => format!("{}, {}", job_title, company),
                (false, true) => job_title,
                (true, false) => company,
                (true, true) => String::new(),
            };
            let image_url = divi
                .attributes
                .get("portrait_url")
                .filter(|s| !s.is_empty())
                .map(|u| ctx.resolve_divi_attr(u));
            Some(NodeKind::Testimonial {
                content,
                name,
                job,
                image_url,
            })
        }
        _ => None,
    }
}
