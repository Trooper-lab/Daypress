use crate::daystudio::context::DaystudioContext;
use crate::daystudio::widgets::basic_media::map_basic_media_widget;
use crate::daystudio::widgets::basic_text::map_basic_text_widget;
use crate::schema::{NodeKind, StyleProperties};
use serde_json::Value;

pub(crate) fn map_basic_widget(
    widget_type: &str,
    settings: &Value,
    styles: &mut StyleProperties,
    ctx: &DaystudioContext,
) -> Option<NodeKind> {
    map_basic_text_widget(widget_type, settings, styles, ctx)
        .or_else(|| map_basic_media_widget(widget_type, settings, styles, ctx))
}
