#![allow(unused_imports)]
use crate::schema::{
    AccordionItem, BackgroundConfig, BorderConfig, IconListItem, NeopressLayout, NeopressNode,
    NodeKind, SocialIconItem, Spacing, StyleProperties, TabItem, ToggleItem, TypographyConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use super::*;

pub(crate) fn parse_elementor_node(
    val: &Value,
    ctx: &ElementorContext,
    depth: usize,
) -> Option<NeopressNode> {
    let obj = val.as_object()?;
    let id = obj
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("elem_node")
        .to_string();

    let el_type = obj
        .get("elType")
        .and_then(|v| v.as_str())
        .unwrap_or("widget");

    let settings = obj.get("settings").cloned().unwrap_or(Value::Null);
    let mut styles = extract_elementor_styles(&settings, ctx);

    let raw_children = obj
        .get("elements")
        .and_then(|e| e.as_array())
        .cloned()
        .unwrap_or_default();

    let mut children: Vec<NeopressNode> = raw_children
        .iter()
        .filter_map(|c| parse_elementor_node(c, ctx, depth + 1))
        .collect();

    let kind = match el_type {
        "section" => {
            let is_inner = obj
                .get("isInner")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if is_inner {
                NodeKind::Container
            } else {
                NodeKind::Section
            }
        }
        "container" => {
            let container_type = settings
                .get("container_type")
                .and_then(|v| v.as_str())
                .unwrap_or("flex");

            if container_type == "grid" {
                styles.display = Some("grid".to_string());
            } else {
                styles.display = Some("flex".to_string());
                let dir = settings
                    .get("flex_direction")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("column");
                styles.flex_direction = Some(dir.to_string());

                if let Some(jc) = settings
                    .get("flex_justify_content")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                {
                    styles.justify_content = Some(jc.to_string());
                }
                if let Some(ai) = settings
                    .get("flex_align_items")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                {
                    styles.align_items = Some(ai.to_string());
                }
                if let Some(fw) = settings
                    .get("flex_wrap")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                {
                    styles.flex_wrap = Some(fw.to_string());
                }
            }

            if let Some(gap_val) = settings.get("flex_gap") {
                if let Some(gap_obj) = gap_val.as_object() {
                    let unit = gap_obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
                    if let Some(sz) = gap_obj.get("size").and_then(extract_spacing_val) {
                        styles.gap = Some(format!("{}{}", sz, unit));
                    } else if let Some(col_gap) =
                        gap_obj.get("column").and_then(extract_spacing_val)
                    {
                        styles.gap = Some(format!("{}{}", col_gap, unit));
                    }
                } else if let Some(s) = extract_spacing_val(gap_val) {
                    styles.gap = Some(format!("{}px", s));
                }
            }

            if let Some(w_obj) = settings.get("width").and_then(|v| v.as_object()) {
                let unit = w_obj.get("unit").and_then(|u| u.as_str()).unwrap_or("%");
                if let Some(sz) = w_obj.get("size").and_then(extract_spacing_val) {
                    styles.width = Some(format!("{}{}", sz, unit));
                }
            }

            // Boxed top-level container support
            let content_width = settings
                .get("content_width")
                .and_then(|v| v.as_str())
                .unwrap_or("boxed");
            if depth == 0 && content_width != "full" {
                append_css_class(&mut styles, "np-container-boxed");
            }

            // Clickable container (`html_tag: "a"` or `__dynamic__.link`)
            if let Some(href) = resolve_setting_link(&settings, "link", ctx) {
                styles.link_url = Some(href);
            }

            NodeKind::Container
        }
        "column" => {
            let col_size = settings
                .get("_column_size")
                .and_then(|v| v.as_f64())
                .map(|p| format!("{}%", p))
                .or_else(|| {
                    settings
                        .get("_inline_size")
                        .and_then(|v| v.as_f64())
                        .map(|p| format!("{}%", p))
                });

            NodeKind::Flex {
                direction: "column".to_string(),
                justify: "flex-start".to_string(),
                align: "stretch".to_string(),
                wrap: false,
                basis: col_size,
            }
        }
        "widget" => {
            let widget_type = obj
                .get("widgetType")
                .and_then(|v| v.as_str())
                .unwrap_or("html");
            map_elementor_widget(widget_type, &settings, &mut styles, &mut children, ctx)?
        }
        _ => NodeKind::CustomHtml {
            raw: format!("<!-- Unrecognized Elementor type: {} -->", el_type),
        },
    };

    styles.sync_flat_and_nested();

    Some(NeopressNode {
        id,
        kind,
        styles,
        children,
    })
}

pub(crate) fn append_css_class(styles: &mut StyleProperties, cls: &str) {
    match styles.css_classes {
        Some(ref mut existing) if !existing.is_empty() => {
            if !existing.split_whitespace().any(|c| c == cls) {
                existing.push(' ');
                existing.push_str(cls);
            }
        }
        _ => styles.css_classes = Some(cls.to_string()),
    }
}

pub(crate) fn escape_html_attr(val: &str) -> String {
    val.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
