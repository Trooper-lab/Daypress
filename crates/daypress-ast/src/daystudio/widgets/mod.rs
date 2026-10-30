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
use crate::schema::{DaypressNode, NodeKind, StyleProperties};
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
