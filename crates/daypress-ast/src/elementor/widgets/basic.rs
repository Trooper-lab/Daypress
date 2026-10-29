use crate::elementor::context::ElementorContext;
use crate::elementor::widgets::basic_media::map_basic_media_widget;
use crate::elementor::widgets::basic_text::map_basic_text_widget;
use crate::schema::{NodeKind, StyleProperties};
use serde_json::Value;

pub(crate) fn map_basic_widget(
    widget_type: &str,
    settings: &Value,
    styles: &mut StyleProperties,
    ctx: &ElementorContext,
) -> Option<NodeKind> {
    map_basic_text_widget(widget_type, settings, styles, ctx)
        .or_else(|| map_basic_media_widget(widget_type, settings, styles, ctx))
}
