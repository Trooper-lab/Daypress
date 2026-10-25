use super::box_model::extract_spacing_val;
use super::globals::extract_global_color;
use crate::elementor::context::ElementorContext;
use crate::schema::{BackgroundConfig, StyleProperties};
use serde_json::{Map, Value};

pub(crate) fn extract_elementor_background(
    settings: &Value,
    globals: Option<&Map<String, Value>>,
    ctx: &ElementorContext,
    styles: &mut StyleProperties,
) {
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
                        && !matches!(*s, "classic" | "gradient" | "video" | "slideshow" | "none")
                })
                .map(|s| s.to_string())
        });

    if bg_type == "gradient" {
        let color_a = bg_color.clone().unwrap_or_else(|| "#0f172a".to_string());
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
}
