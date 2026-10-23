use super::box_model::extract_spacing_val;
use super::globals::{extract_global_color, extract_global_typography};
use crate::schema::{StyleProperties, TypographyConfig};
use serde_json::{Map, Value};

pub(crate) fn extract_elementor_typography(
    settings: &Value,
    globals: Option<&Map<String, Value>>,
    styles: &mut StyleProperties,
) {
    let global_typo_id = globals.and_then(|g| {
        extract_global_typography(g, "typography_typography")
            .or_else(|| extract_global_typography(g, "menu_typography_typography"))
    });

    let font_family = settings
        .get("typography_font_family")
        .or_else(|| settings.get("menu_typography_font_family"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            global_typo_id
                .as_ref()
                .map(|id| format!("var(--e-global-typography-{}-font-family)", id))
        });
    styles.font_family = font_family;

    let font_size = settings
        .get("typography_font_size")
        .or_else(|| settings.get("menu_typography_font_size"))
        .and_then(|fs| {
            if let Some(obj) = fs.as_object() {
                let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
                obj.get("size")
                    .and_then(extract_spacing_val)
                    .map(|s| format!("{}{}", s, unit))
            } else {
                extract_spacing_val(fs)
            }
        });

    let font_weight = settings
        .get("typography_font_weight")
        .or_else(|| settings.get("menu_typography_font_weight"))
        .and_then(extract_spacing_val)
        .or_else(|| {
            global_typo_id
                .as_ref()
                .map(|id| format!("var(--e-global-typography-{}-font-weight)", id))
        });

    let line_height = settings.get("typography_line_height").and_then(|lh| {
        if let Some(obj) = lh.as_object() {
            let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("em");
            obj.get("size")
                .and_then(extract_spacing_val)
                .map(|s| format!("{}{}", s, unit))
        } else {
            extract_spacing_val(lh)
        }
    });

    styles.text_transform = settings
        .get("typography_text_transform")
        .or_else(|| settings.get("menu_typography_text_transform"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    styles.font_style = settings
        .get("typography_font_style")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    styles.letter_spacing = settings.get("typography_letter_spacing").and_then(|ls| {
        if let Some(obj) = ls.as_object() {
            let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
            obj.get("size")
                .and_then(extract_spacing_val)
                .map(|s| format!("{}{}", s, unit))
        } else {
            extract_spacing_val(ls)
        }
    });

    let text_color = globals
        .and_then(|g| {
            extract_global_color(g, "text_color")
                .or_else(|| extract_global_color(g, "title_color"))
                .or_else(|| extract_global_color(g, "color"))
                .or_else(|| extract_global_color(g, "color_menu_item"))
                .or_else(|| extract_global_color(g, "icon_color"))
                .or_else(|| extract_global_color(g, "button_text_color"))
        })
        .or_else(|| {
            settings
                .get("text_color")
                .or_else(|| settings.get("title_color"))
                .or_else(|| settings.get("color"))
                .or_else(|| settings.get("color_menu_item"))
                .or_else(|| settings.get("icon_color"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty() && *s != "custom" && *s != "default")
                .map(|s| s.to_string())
        });

    let text_align = settings
        .get("align")
        .or_else(|| settings.get("text_align"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    if font_size.is_some()
        || font_weight.is_some()
        || line_height.is_some()
        || text_color.is_some()
        || text_align.is_some()
    {
        styles.typography = Some(TypographyConfig {
            font_size,
            font_weight,
            color: text_color,
            text_align,
            line_height,
        });
    }
}
