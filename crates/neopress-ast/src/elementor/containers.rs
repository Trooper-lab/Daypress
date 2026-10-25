#![allow(unused_imports)]
use super::*;
use crate::schema::{
    AccordionItem, BackgroundConfig, BorderConfig, IconListItem, NeopressLayout, NeopressNode,
    NodeKind, SocialIconItem, Spacing, StyleProperties, TabItem, ToggleItem, TypographyConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

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
                parse_elementor_grid_container(&settings, &mut styles, ctx);
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

    // Grid item placement
    if let Some(gc) = settings
        .get("_grid_column")
        .or_else(|| settings.get("grid_column"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        styles.grid_column = Some(gc.to_string());
    } else if let Some(span) = settings.get("_grid_column_span").and_then(|v| v.as_u64()) {
        styles.grid_column = Some(format!("span {}", span));
    }
    if let Some(gr) = settings
        .get("_grid_row")
        .or_else(|| settings.get("grid_row"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        styles.grid_row = Some(gr.to_string());
    } else if let Some(span) = settings.get("_grid_row_span").and_then(|v| v.as_u64()) {
        styles.grid_row = Some(format!("span {}", span));
    }

    styles.sync_flat_and_nested();

    Some(NeopressNode {
        id,
        kind,
        styles,
        children,
    })
}

pub(crate) fn parse_grid_track_def(settings: &Value, base_key: &str) -> Option<String> {
    let custom_key = format!("{}_custom", base_key);
    if let Some(custom) = settings
        .get(&custom_key)
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
    {
        return Some(custom.trim().to_string());
    }

    if let Some(val) = settings.get(base_key) {
        if let Some(s) = val.as_str().filter(|s| !s.trim().is_empty()) {
            if s.chars().all(|c| c.is_ascii_digit()) {
                if let Ok(n) = s.parse::<u32>() {
                    return Some(format!("repeat({}, 1fr)", n));
                }
            }
            return Some(s.trim().to_string());
        }
        if let Some(n) = val.as_u64() {
            return Some(format!("repeat({}, 1fr)", n));
        }
        if let Some(obj) = val.as_object() {
            let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("fr");
            if unit == "custom" {
                if let Some(custom) = obj
                    .get("custom")
                    .or_else(|| obj.get("size"))
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.trim().is_empty())
                {
                    return Some(custom.trim().to_string());
                }
            }
            if let Some(size_val) = obj.get("size") {
                if let Some(sz_num) = size_val.as_u64() {
                    if unit == "fr" {
                        return Some(format!("repeat({}, 1fr)", sz_num));
                    } else {
                        return Some(format!("repeat({}, 1{})", sz_num, unit));
                    }
                } else if let Some(sz_str) = size_val.as_str() {
                    if sz_str.chars().all(|c| c.is_ascii_digit()) {
                        return Some(format!("repeat({}, 1fr)", sz_str));
                    }
                    return Some(sz_str.to_string());
                }
            }
        }
    }
    None
}

pub(crate) fn parse_grid_gap_def(settings: &Value, base_key: &str) -> Option<String> {
    if let Some(val) = settings.get(base_key) {
        if let Some(obj) = val.as_object() {
            let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
            let row = obj.get("row").and_then(extract_spacing_val);
            let col = obj.get("column").and_then(extract_spacing_val);
            match (row, col) {
                (Some(r), Some(c)) if r == c => Some(format!("{}{}", r, unit)),
                (Some(r), Some(c)) => Some(format!("{}{} {}{}", r, unit, c, unit)),
                (Some(r), None) => Some(format!("{}{}", r, unit)),
                (None, Some(c)) => Some(format!("{}{}", c, unit)),
                (None, None) => None,
            }
        } else {
            extract_spacing_val(val).map(|s| format!("{}px", s))
        }
    } else {
        None
    }
}

pub(crate) fn parse_elementor_grid_container(
    settings: &Value,
    styles: &mut StyleProperties,
    _ctx: &ElementorContext,
) {
    styles.display = Some("grid".to_string());

    // Columns & Rows
    if let Some(cols) = parse_grid_track_def(settings, "grid_columns_grid")
        .or_else(|| parse_grid_track_def(settings, "grid_columns"))
    {
        styles.grid_template_columns = Some(cols);
    }
    if let Some(rows) = parse_grid_track_def(settings, "grid_rows_grid")
        .or_else(|| parse_grid_track_def(settings, "grid_rows"))
    {
        styles.grid_template_rows = Some(rows);
    }

    // Gaps
    if let Some(gap) = parse_grid_gap_def(settings, "grid_gaps")
        .or_else(|| parse_grid_gap_def(settings, "grid_gap"))
    {
        styles.grid_gap = Some(gap);
    }

    // Auto Flow
    if let Some(flow) = settings
        .get("grid_auto_flow")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
    {
        styles.grid_auto_flow = Some(flow.trim().to_string());
    }

    // Alignment
    if let Some(ji) = settings
        .get("grid_justify_items")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
    {
        styles.justify_items = Some(ji.trim().to_string());
    }
    if let Some(ai) = settings
        .get("grid_align_items")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
    {
        styles.align_items = Some(ai.trim().to_string());
    }

    // Responsive Tablet
    let tab_cols = parse_grid_track_def(settings, "grid_columns_grid_tablet")
        .or_else(|| parse_grid_track_def(settings, "grid_columns_tablet"));
    let tab_rows = parse_grid_track_def(settings, "grid_rows_grid_tablet")
        .or_else(|| parse_grid_track_def(settings, "grid_rows_tablet"));
    let tab_gap = parse_grid_gap_def(settings, "grid_gaps_tablet")
        .or_else(|| parse_grid_gap_def(settings, "grid_gap_tablet"));

    if tab_cols.is_some() || tab_rows.is_some() || tab_gap.is_some() {
        let tab = styles
            .tablet
            .get_or_insert_with(|| Box::new(StyleProperties::default()));
        if let Some(c) = tab_cols {
            tab.grid_template_columns = Some(c);
        }
        if let Some(r) = tab_rows {
            tab.grid_template_rows = Some(r);
        }
        if let Some(g) = tab_gap {
            tab.grid_gap = Some(g);
        }
    }

    // Responsive Mobile
    let mob_cols = parse_grid_track_def(settings, "grid_columns_grid_mobile")
        .or_else(|| parse_grid_track_def(settings, "grid_columns_mobile"));
    let mob_rows = parse_grid_track_def(settings, "grid_rows_grid_mobile")
        .or_else(|| parse_grid_track_def(settings, "grid_rows_mobile"));
    let mob_gap = parse_grid_gap_def(settings, "grid_gaps_mobile")
        .or_else(|| parse_grid_gap_def(settings, "grid_gap_mobile"));

    if mob_cols.is_some() || mob_rows.is_some() || mob_gap.is_some() {
        let mob = styles
            .mobile
            .get_or_insert_with(|| Box::new(StyleProperties::default()));
        if let Some(c) = mob_cols {
            mob.grid_template_columns = Some(c);
        }
        if let Some(r) = mob_rows {
            mob.grid_template_rows = Some(r);
        }
        if let Some(g) = mob_gap {
            mob.grid_gap = Some(g);
        }
    }
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
