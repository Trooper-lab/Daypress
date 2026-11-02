pub mod basic;
pub mod basic_media;
pub mod basic_text;
pub mod carousel;
pub mod dynamic;
pub mod forms;
pub mod pro;

pub(crate) use basic::map_basic_widget;
pub(crate) use carousel::map_carousel_widget;
pub(crate) use dynamic::map_dynamic_widget;
pub(crate) use forms::map_form_widget;
pub(crate) use pro::map_pro_widget;

pub(crate) use crate::daystudio::containers::escape_html_attr;
pub(crate) use crate::daystudio::context::resolve_setting_link;
pub(crate) use crate::daystudio::styles::extract_spacing_val;

use crate::daystudio::context::DaystudioContext;
use crate::daystudio::styles::parse_spacing_obj;
use crate::schema::{AccordionItem, DaypressNode, NodeKind, StyleProperties, TabItem};
use serde_json::Value;

pub(crate) fn map_daystudio_widget(
    widget_type: &str,
    settings: &Value,
    styles: &mut StyleProperties,
    children: &mut Vec<DaypressNode>,
    ctx: &DaystudioContext,
) -> Option<NodeKind> {
    if widget_type == "form" {
        return map_form_widget(settings, styles, children, ctx);
    }

    if matches!(widget_type, "nested-tabs" | "e-tabs") {
        let mut items = Vec::new();
        if let Some(tabs) = settings.get("tabs").and_then(|t| t.as_array()) {
            for (idx, tab) in tabs.iter().enumerate() {
                let title = tab
                    .get("tab_title")
                    .or_else(|| tab.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Tab")
                    .to_string();
                let raw_content = tab
                    .get("tab_content")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let content_html = if !raw_content.trim().is_empty() {
                    ctx.normalize_html(raw_content)
                } else if let Some(child_node) = children.get(idx) {
                    crate::render_node_to_html(child_node)
                } else {
                    String::new()
                };
                items.push(TabItem {
                    title,
                    content_html,
                });
            }
        } else {
            for (idx, child_node) in children.iter().enumerate() {
                items.push(TabItem {
                    title: format!("Tab {}", idx + 1),
                    content_html: crate::render_node_to_html(child_node),
                });
            }
        }
        children.clear();
        return Some(NodeKind::Tabs { items });
    }

    if matches!(widget_type, "nested-accordion" | "e-accordion") {
        let mut items = Vec::new();
        let tab_list = settings
            .get("tabs")
            .or_else(|| settings.get("items"))
            .and_then(|t| t.as_array());
        if let Some(tabs) = tab_list {
            for (idx, tab) in tabs.iter().enumerate() {
                let title = tab
                    .get("tab_title")
                    .or_else(|| tab.get("item_title"))
                    .or_else(|| tab.get("title"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("Accordion Item")
                    .to_string();
                let raw_content = tab
                    .get("tab_content")
                    .or_else(|| tab.get("item_content"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let content_html = if !raw_content.trim().is_empty() {
                    ctx.normalize_html(raw_content)
                } else if let Some(child_node) = children.get(idx) {
                    crate::render_node_to_html(child_node)
                } else {
                    String::new()
                };
                items.push(AccordionItem {
                    title,
                    content_html,
                });
            }
        } else {
            for (idx, child_node) in children.iter().enumerate() {
                items.push(AccordionItem {
                    title: format!("Item {}", idx + 1),
                    content_html: crate::render_node_to_html(child_node),
                });
            }
        }
        children.clear();
        return Some(NodeKind::Accordion { items });
    }

    if let Some(kind) = map_carousel_widget(widget_type, settings, styles, ctx) {
        return Some(kind);
    }

    if let Some(kind) = map_pro_widget(widget_type, settings, styles, ctx) {
        return Some(kind);
    }

    if let Some(kind) = map_dynamic_widget(widget_type, settings, styles, ctx) {
        return Some(kind);
    }

    if let Some(kind) = map_basic_widget(widget_type, settings, styles, ctx) {
        return Some(kind);
    }

    // Graceful fallback for unrecognized third-party widgets
    let raw = settings
        .get("raw")
        .or_else(|| settings.get("editor"))
        .and_then(|v| v.as_str())
        .map(|s| ctx.normalize_html(s))
        .unwrap_or_else(|| {
            format!(
                "<!-- Elementor widget '{}' imported gracefully -->",
                widget_type
            )
        });
    Some(NodeKind::CustomHtml { raw })
}
