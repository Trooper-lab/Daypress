use super::fonts::parse_divi_font_composite;
use super::quad::parse_divi_quad;
use super::responsive::apply_divi_responsive_styles;
use crate::divi::context::{resolve_divi_color, DiviContext};
use crate::schema::{BackgroundConfig, BorderConfig, StyleProperties, TypographyConfig};
use std::collections::HashMap;

pub(crate) fn extract_divi_styles(
    attrs: &HashMap<String, String>,
    ctx: &DiviContext,
) -> StyleProperties {
    let mut styles = StyleProperties::default();

    // 1. Background color & image
    let bg_enabled = attrs
        .get("background_enable_color")
        .map(|v| v != "off")
        .unwrap_or(true);
    let bg_color = if bg_enabled {
        attrs
            .get("background_color")
            .filter(|s| !s.is_empty())
            .map(|c| resolve_divi_color(c))
    } else {
        None
    };

    let img_enabled = attrs
        .get("background_enable_image")
        .map(|v| v != "off")
        .unwrap_or(true);
    let bg_img = if img_enabled {
        attrs
            .get("background_image")
            .filter(|s| !s.is_empty())
            .map(|u| ctx.normalize_url(u))
    } else {
        None
    };

    if bg_color.is_some() || bg_img.is_some() {
        styles.background = Some(BackgroundConfig {
            color: bg_color.clone(),
            image_url: bg_img,
            position: None,
            size: Some("cover".to_string()),
        });
        styles.background_color = bg_color;
    }

    // Handle Divi background gradients (use_background_color_gradient="on")
    if attrs
        .get("use_background_color_gradient")
        .map(|v| v == "on")
        .unwrap_or(false)
    {
        if let Some(stops_raw) = attrs
            .get("background_color_gradient_stops")
            .filter(|s| !s.is_empty())
        {
            let stops: Vec<String> = stops_raw
                .split('|')
                .map(|part| {
                    let tokens: Vec<&str> = part.split_whitespace().collect();
                    if let Some(first) = tokens.first() {
                        let col = resolve_divi_color(first);
                        if tokens.len() > 1 {
                            format!("{} {}", col, tokens[1..].join(" "))
                        } else {
                            col
                        }
                    } else {
                        part.to_string()
                    }
                })
                .collect();
            let grad = format!("linear-gradient(180deg, {})", stops.join(", "));
            if let Some(ref mut bg) = styles.background {
                if let Some(ref img) = bg.image_url {
                    styles.background_gradient = Some(format!("{}, url('{}')", grad, img));
                    bg.image_url = None;
                } else {
                    styles.background_gradient = Some(grad);
                }
            } else {
                styles.background_gradient = Some(grad);
            }
        }
    }

    // 2. Padding and Margin (Desktop)
    if let Some(pad) = attrs.get("custom_padding") {
        styles.padding = parse_divi_quad(pad);
    }
    if let Some(mar) = attrs.get("custom_margin") {
        styles.margin = parse_divi_quad(mar);
    }

    // 3. Borders & Radius
    let mut border_radius = None;
    if let Some(radii) = attrs.get("border_radii") {
        let parts: Vec<&str> = radii.split('|').collect();
        if parts.len() >= 5 {
            let t = if parts[1].trim().is_empty() {
                "0px"
            } else {
                parts[1].trim()
            };
            let r = if parts[2].trim().is_empty() {
                "0px"
            } else {
                parts[2].trim()
            };
            let b = if parts[3].trim().is_empty() {
                "0px"
            } else {
                parts[3].trim()
            };
            let l = if parts[4].trim().is_empty() {
                "0px"
            } else {
                parts[4].trim()
            };
            if t != "0px" || r != "0px" || b != "0px" || l != "0px" {
                let rad = if t == r && r == b && b == l {
                    t.to_string()
                } else {
                    format!("{} {} {} {}", t, r, b, l)
                };
                border_radius = Some(rad.clone());
                styles.border_radius = Some(rad);
            }
        }
    }

    let border_width = attrs
        .get("border_width_all")
        .or_else(|| attrs.get("border_width_bottom"))
        .filter(|s| !s.is_empty() && *s != "0px")
        .cloned();
    let border_color = attrs
        .get("border_color_all")
        .or_else(|| attrs.get("border_color_bottom"))
        .filter(|s| !s.is_empty())
        .map(|c| resolve_divi_color(c));

    if border_radius.is_some() || border_width.is_some() || border_color.is_some() {
        styles.border = Some(BorderConfig {
            radius: border_radius,
            width: border_width,
            color: border_color,
            style: Some("solid".to_string()),
        });
    }

    // 4. Typography
    let text_align = attrs
        .get("text_orientation")
        .or_else(|| attrs.get("title_text_align"))
        .or_else(|| attrs.get("header_text_align"))
        .or_else(|| attrs.get("module_alignment"))
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s == "justified" {
                "justify".to_string()
            } else {
                s.clone()
            }
        });

    let mut text_color = attrs
        .get("title_text_color")
        .or_else(|| attrs.get("header_text_color"))
        .or_else(|| attrs.get("text_text_color"))
        .or_else(|| attrs.get("body_text_color"))
        .or_else(|| attrs.get("text_color"))
        .filter(|s| !s.is_empty())
        .map(|c| resolve_divi_color(c));

    if text_color.is_none()
        && attrs
            .get("background_layout")
            .map(|s| s == "dark")
            .unwrap_or(false)
    {
        text_color = Some("#ffffff".to_string());
    }

    let font_size = attrs
        .get("title_font_size")
        .or_else(|| attrs.get("header_font_size"))
        .or_else(|| attrs.get("text_font_size"))
        .or_else(|| attrs.get("body_font_size"))
        .filter(|s| !s.is_empty())
        .cloned();

    let line_height = attrs
        .get("title_line_height")
        .or_else(|| attrs.get("text_line_height"))
        .or_else(|| attrs.get("header_line_height"))
        .filter(|s| !s.is_empty())
        .cloned();

    let letter_spacing = attrs
        .get("title_letter_spacing")
        .or_else(|| attrs.get("header_letter_spacing"))
        .or_else(|| attrs.get("text_letter_spacing"))
        .filter(|s| !s.is_empty())
        .cloned();

    if let Some(ls) = letter_spacing {
        styles.letter_spacing = Some(ls);
    }

    // Parse Divi font composite string: "Bitter|700|||||||"
    if let Some(font_raw) = attrs
        .get("title_font")
        .or_else(|| attrs.get("header_font"))
        .or_else(|| attrs.get("text_font"))
        .or_else(|| attrs.get("body_font"))
    {
        parse_divi_font_composite(font_raw, &mut styles);
    }

    if text_align.is_some()
        || text_color.is_some()
        || font_size.is_some()
        || line_height.is_some()
        || styles.font_weight.is_some()
    {
        styles.typography = Some(TypographyConfig {
            font_size: font_size.clone(),
            font_weight: styles.font_weight.clone(),
            color: text_color.clone(),
            text_align: text_align.clone(),
            line_height: line_height.clone(),
        });
        styles.text_align = text_align;
        styles.text_color = text_color;
        styles.font_size = font_size;
        styles.line_height = line_height;
    }

    // 5. Sizing
    if let Some(w) = attrs.get("width").filter(|s| !s.is_empty()) {
        styles.width = Some(w.clone());
    }
    if let Some(mw) = attrs.get("max_width").filter(|s| !s.is_empty()) {
        styles.max_width = Some(mw.clone());
    }

    // 6. Box shadow
    if let Some(shadow_preset) = attrs
        .get("box_shadow_style")
        .filter(|s| !s.is_empty() && *s != "none")
    {
        let shadow = match shadow_preset.as_str() {
            "preset1" => "0px 2px 18px 0px rgba(0,0,0,0.3)",
            "preset3" => "0px 12px 18px -6px rgba(0,0,0,0.3)",
            "preset4" => "10px 10px 0px 0px rgba(0,0,0,0.2)",
            "preset6" => "0px 0px 18px 0px rgba(0,0,0,0.3)",
            _ => "0px 4px 16px rgba(0,0,0,0.18)",
        };
        styles.box_shadow = Some(shadow.to_string());
    }

    // 7. Custom CSS & Classes
    if let Some(cls) = attrs.get("module_class").filter(|s| !s.trim().is_empty()) {
        styles.css_classes = Some(cls.trim().to_string());
    }

    // 8. Responsive Quads & Typography (Tablet & Mobile/Phone) & Disabled On
    apply_divi_responsive_styles(&mut styles, attrs);

    styles.sync_flat_and_nested();
    styles
}
