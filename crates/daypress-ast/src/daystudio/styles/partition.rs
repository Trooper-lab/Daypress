use super::background::extract_daystudio_background;
use super::box_model::{extract_border_config, extract_spacing_val, parse_spacing_obj};
use super::typography::extract_daystudio_typography;
use crate::daystudio::containers::append_css_class;
use crate::daystudio::context::DaystudioContext;
use crate::schema::StyleProperties;
use serde_json::{Map, Value};
use std::collections::HashMap;

pub(crate) fn partition_daystudio_settings(settings: &Value) -> (Value, Value, Value, Value) {
    let mut desktop = Map::new();
    let mut tablet = Map::new();
    let mut mobile = Map::new();
    let mut hover = Map::new();

    let Some(obj) = settings.as_object() else {
        return (Value::Null, Value::Null, Value::Null, Value::Null);
    };

    for (k, v) in obj {
        if let Some(base_key) = k.strip_suffix("_mobile") {
            mobile.insert(base_key.to_string(), v.clone());
        } else if let Some(base_key) = k.strip_suffix("_tablet") {
            tablet.insert(base_key.to_string(), v.clone());
        } else if let Some(base_key) = k.strip_suffix("_hover") {
            hover.insert(base_key.to_string(), v.clone());
        } else if let Some(base_key) = k.strip_prefix("hover_") {
            hover.insert(base_key.to_string(), v.clone());
        } else {
            desktop.insert(k.clone(), v.clone());
        }
    }

    (
        Value::Object(desktop),
        Value::Object(tablet),
        Value::Object(mobile),
        Value::Object(hover),
    )
}

pub(crate) fn extract_daystudio_styles(
    settings: &Value,
    ctx: &DaystudioContext,
) -> StyleProperties {
    let (desktop, tablet, mobile, hover) = partition_daystudio_settings(settings);
    let mut styles = extract_daystudio_single_state_styles(&desktop, ctx);

    if let Some(t_obj) = tablet.as_object() {
        if !t_obj.is_empty() {
            let t_styles = extract_daystudio_single_state_styles(&tablet, ctx);
            styles.tablet = Some(Box::new(t_styles));
        }
    }

    if let Some(m_obj) = mobile.as_object() {
        if !m_obj.is_empty() {
            let m_styles = extract_daystudio_single_state_styles(&mobile, ctx);
            styles.mobile = Some(Box::new(m_styles));
        }
    }

    if let Some(h_obj) = hover.as_object() {
        if !h_obj.is_empty() {
            let h_styles = extract_daystudio_single_state_styles(&hover, ctx);
            styles.hover = Some(Box::new(h_styles));
        }
    }

    // Lossless raw_settings fallback:
    if let Some(obj) = settings.as_object() {
        let mut raw_map = HashMap::new();
        for (k, v) in obj {
            if k.starts_with("ea_")
                || k.starts_with("dss_")
                || k.starts_with("acf_")
                || k.starts_with("custom_")
                || k.starts_with("_elementor_")
                || k.contains("tooltip")
                || k.contains("animation")
            {
                raw_map.insert(k.clone(), v.clone());
            }
        }
        if !raw_map.is_empty() {
            styles.raw_settings = Some(raw_map);
        }
    }

    styles.sync_flat_and_nested();
    styles
}

pub(crate) fn extract_daystudio_single_state_styles(
    settings: &Value,
    ctx: &DaystudioContext,
) -> StyleProperties {
    let mut styles = StyleProperties::default();
    let globals = settings.get("__globals__").and_then(|g| g.as_object());

    // Spacing: Padding (`padding` or widget `_padding`)
    if let Some(pad) = settings
        .get("padding")
        .or_else(|| settings.get("_padding"))
        .and_then(|v| v.as_object())
    {
        styles.padding = parse_spacing_obj(pad);
    }

    // Spacing: Margin (`margin` or widget `_margin`)
    if let Some(mar) = settings
        .get("margin")
        .or_else(|| settings.get("_margin"))
        .and_then(|v| v.as_object())
    {
        styles.margin = parse_spacing_obj(mar);
    }

    // Typography
    extract_daystudio_typography(settings, globals, &mut styles);

    // Background
    extract_daystudio_background(settings, globals, ctx, &mut styles);

    // Border
    styles.border = extract_border_config(settings, globals);

    // Min-height (`min_height`)
    if let Some(mh_obj) = settings.get("min_height").and_then(|v| v.as_object()) {
        let unit = mh_obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
        if let Some(sz) = mh_obj.get("size").and_then(extract_spacing_val) {
            styles.min_height = Some(format!("{}{}", sz, unit));
        }
    }

    // Widget custom width (`_element_width` / `_element_custom_width`)
    if let Some(ew) = settings.get("_element_width").and_then(|v| v.as_str()) {
        match ew {
            "auto" => {
                styles.width = Some("auto".to_string());
            }
            "initial" => {
                if let Some(cw_obj) = settings
                    .get("_element_custom_width")
                    .and_then(|v| v.as_object())
                {
                    let unit = cw_obj.get("unit").and_then(|u| u.as_str()).unwrap_or("%");
                    if let Some(sz) = cw_obj.get("size").and_then(extract_spacing_val) {
                        styles.width = Some(format!("{}{}", sz, unit));
                    }
                }
            }
            "inherit" | "full" => {
                styles.width = Some("100%".to_string());
            }
            _ => {}
        }
    }

    // Responsive visibility classes
    if settings
        .get("hide_desktop")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .is_some()
    {
        append_css_class(&mut styles, "dp-hide-desktop");
    }
    if settings
        .get("hide_tablet")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .is_some()
    {
        append_css_class(&mut styles, "dp-hide-tablet");
    }
    if settings
        .get("hide_mobile")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .is_some()
    {
        append_css_class(&mut styles, "dp-hide-mobile");
    }

    if let Some(custom_css) = settings
        .get("custom_css")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
    {
        styles.custom_css = Some(custom_css.to_string());
    }

    styles.sync_flat_and_nested();
    styles
}
