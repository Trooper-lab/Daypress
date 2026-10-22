#![allow(unused_imports)]
use crate::schema::{
    AccordionItem, BackgroundConfig, BorderConfig, IconListItem, NeopressLayout, NeopressNode,
    NodeKind, SocialIconItem, Spacing, StyleProperties, TabItem, ToggleItem, TypographyConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use super::*;

pub(crate) fn extract_global_color(globals: &Map<String, Value>, key: &str) -> Option<String> {
    let val = globals.get(key)?.as_str()?;
    if let Some(idx) = val.find("id=") {
        let id = &val[idx + 3..];
        let clean_id: String = id
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !clean_id.is_empty() {
            return Some(format!("var(--e-global-color-{})", clean_id));
        }
    }
    None
}

pub(crate) fn extract_global_typography(globals: &Map<String, Value>, key: &str) -> Option<String> {
    let val = globals.get(key)?.as_str()?;
    if let Some(idx) = val.find("id=") {
        let id = &val[idx + 3..];
        let clean_id: String = id
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !clean_id.is_empty() {
            return Some(clean_id);
        }
    }
    None
}

pub(crate) fn parse_spacing_obj(obj: &Map<String, Value>) -> Option<Spacing> {
    let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
    let top = obj
        .get("top")
        .and_then(extract_spacing_val)
        .map(|v| format!("{}{}", v, unit));
    let right = obj
        .get("right")
        .and_then(extract_spacing_val)
        .map(|v| format!("{}{}", v, unit));
    let bottom = obj
        .get("bottom")
        .and_then(extract_spacing_val)
        .map(|v| format!("{}{}", v, unit));
    let left = obj
        .get("left")
        .and_then(extract_spacing_val)
        .map(|v| format!("{}{}", v, unit));

    if top.is_some() || right.is_some() || bottom.is_some() || left.is_some() {
        Some(Spacing {
            top,
            right,
            bottom,
            left,
        })
    } else {
        None
    }
}

pub(crate) fn extract_elementor_styles(settings: &Value, ctx: &ElementorContext) -> StyleProperties {
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

    // Background (`background_background` or widget `_background_background`)
    let bg_type = settings
        .get("background_background")
        .or_else(|| settings.get("_background_background"))
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    let bg_color = globals
        .and_then(|g| {
            extract_global_color(g, "background_color")
                .or_else(|| extract_global_color(g, "_background_color"))
                .or_else(|| extract_global_color(g, "button_background_color"))
        })
        .or_else(|| {
            settings
                .get("background_color")
                .or_else(|| settings.get("_background_color"))
                .and_then(|v| v.as_str())
                .filter(|s| {
                    !s.is_empty()
                        && !matches!(
                            *s,
                            "classic" | "gradient" | "video" | "slideshow" | "none"
                        )
                })
                .map(|s| s.to_string())
        });

    if bg_type == "gradient" {
        let color_a = bg_color
            .clone()
            .unwrap_or_else(|| "#0f172a".to_string());
        let color_b = settings
            .get("background_color_b")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .or_else(|| globals.and_then(|g| extract_global_color(g, "background_color_b")))
            .unwrap_or_else(|| "#1e293b".to_string());
        let grad_type = settings
            .get("background_gradient_type")
            .and_then(|v| v.as_str())
            .unwrap_or("linear");
        let angle = settings
            .get("background_gradient_angle")
            .and_then(|a| a.get("size"))
            .and_then(extract_spacing_val)
            .unwrap_or_else(|| "180".to_string());
        let unit = settings
            .get("background_gradient_angle")
            .and_then(|a| a.get("unit"))
            .and_then(|u| u.as_str())
            .unwrap_or("deg");
        let stop_a = settings
            .get("background_color_stop")
            .and_then(|s| s.get("size"))
            .and_then(extract_spacing_val)
            .unwrap_or_else(|| "0".to_string());
        let stop_b = settings
            .get("background_color_b_stop")
            .and_then(|s| s.get("size"))
            .and_then(extract_spacing_val)
            .unwrap_or_else(|| "100".to_string());

        let gradient = if grad_type == "radial" {
            format!(
                "radial-gradient(at center center, {} {}%, {} {}%)",
                color_a, stop_a, color_b, stop_b
            )
        } else {
            format!(
                "linear-gradient({}{}, {} {}%, {} {}%)",
                angle, unit, color_a, stop_a, color_b, stop_b
            )
        };
        styles.background_gradient = Some(gradient);
    }

    let mut bg_img = settings
        .get("background_image")
        .or_else(|| settings.get("_background_image"))
        .and_then(|img| img.get("url"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|u| ctx.normalize_url(u));

    let mut bg_pos = settings
        .get("background_position")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let mut bg_size = settings
        .get("background_size")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    if bg_type == "slideshow" && bg_img.is_none() {
        if let Some(first_url) = settings
            .get("background_slideshow_gallery")
            .and_then(|g| g.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("url"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        {
            bg_img = Some(ctx.normalize_url(first_url));
            if bg_size.is_none() {
                bg_size = Some("cover".to_string());
            }
            if bg_pos.is_none() {
                bg_pos = Some("center center".to_string());
            }
        }
    }

    if bg_color.is_some() || bg_img.is_some() || bg_pos.is_some() || bg_size.is_some() {
        styles.background = Some(BackgroundConfig {
            color: bg_color,
            image_url: bg_img,
            position: bg_pos,
            size: bg_size,
        });
    }

    // Border & Border Radius (`border_*` or widget `_border_*`)
    let border_radius = settings
        .get("border_radius")
        .or_else(|| settings.get("_border_radius"))
        .and_then(|b| {
            if let Some(obj) = b.as_object() {
                let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
                if let Some(sz) = obj.get("size").and_then(extract_spacing_val) {
                    return Some(format!("{}{}", sz, unit));
                }
                let top = obj.get("top").and_then(extract_spacing_val);
                let right = obj.get("right").and_then(extract_spacing_val);
                let bottom = obj.get("bottom").and_then(extract_spacing_val);
                let left = obj.get("left").and_then(extract_spacing_val);
                match (top, right, bottom, left) {
                    (Some(t), Some(r), Some(btm), Some(l)) => {
                        if t == r && r == btm && btm == l {
                            Some(format!("{}{}", t, unit))
                        } else {
                            Some(format!(
                                "{}{} {}{} {}{} {}{}",
                                t, unit, r, unit, btm, unit, l, unit
                            ))
                        }
                    }
                    (Some(t), _, _, _) => Some(format!("{}{}", t, unit)),
                    _ => None,
                }
            } else if let Some(n) = b.as_u64() {
                Some(format!("{}px", n))
            } else {
                b.as_str().filter(|s| !s.is_empty()).map(|s| s.to_string())
            }
        });

    let border_style = settings
        .get("border_border")
        .or_else(|| settings.get("_border_border"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty() && *s != "none")
        .map(|s| s.to_string());

    let border_width = settings
        .get("border_width")
        .or_else(|| settings.get("_border_width"))
        .and_then(|w| w.as_object())
        .and_then(|obj| {
            let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
            if let Some(sz) = obj.get("size").and_then(extract_spacing_val) {
                return Some(format!("{}{}", sz, unit));
            }
            let t = obj.get("top").and_then(extract_spacing_val);
            let r = obj.get("right").and_then(extract_spacing_val);
            let b = obj.get("bottom").and_then(extract_spacing_val);
            let l = obj.get("left").and_then(extract_spacing_val);
            if t.is_some() || r.is_some() || b.is_some() || l.is_some() {
                let ts = t.unwrap_or_else(|| "0".to_string());
                let rs = r.unwrap_or_else(|| "0".to_string());
                let bs = b.unwrap_or_else(|| "0".to_string());
                let ls = l.unwrap_or_else(|| "0".to_string());
                if ts == rs && rs == bs && bs == ls {
                    Some(format!("{}{}", ts, unit))
                } else {
                    Some(format!(
                        "{}{} {}{} {}{} {}{}",
                        ts, unit, rs, unit, bs, unit, ls, unit
                    ))
                }
            } else {
                None
            }
        });

    let border_color = globals
        .and_then(|g| {
            extract_global_color(g, "border_color")
                .or_else(|| extract_global_color(g, "_border_color"))
        })
        .or_else(|| {
            settings
                .get("border_color")
                .or_else(|| settings.get("_border_color"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
        });

    if border_radius.is_some()
        || border_style.is_some()
        || border_width.is_some()
        || border_color.is_some()
    {
        styles.border = Some(BorderConfig {
            radius: border_radius,
            width: border_width,
            color: border_color,
            style: border_style,
        });
    }

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
        append_css_class(&mut styles, "np-hide-desktop");
    }
    if settings
        .get("hide_tablet")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .is_some()
    {
        append_css_class(&mut styles, "np-hide-tablet");
    }
    if settings
        .get("hide_mobile")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .is_some()
    {
        append_css_class(&mut styles, "np-hide-mobile");
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

pub(crate) fn extract_spacing_val(v: &Value) -> Option<String> {
    if let Some(s) = v.as_str() {
        if !s.is_empty() {
            return Some(s.to_string());
        }
    } else if let Some(n) = v.as_i64() {
        return Some(n.to_string());
    } else if let Some(n) = v.as_u64() {
        return Some(n.to_string());
    } else if let Some(n) = v.as_f64() {
        if (n - n.round()).abs() < f64::EPSILON {
            return Some((n.round() as i64).to_string());
        }
        return Some(n.to_string());
    }
    None
}
