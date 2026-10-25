use crate::schema::{BackgroundConfig, BorderConfig, Spacing, StyleProperties, TypographyConfig};
use regex::Regex;
use serde_json::Value;

pub(crate) fn extract_gutenberg_styles(attrs: &Value) -> StyleProperties {
    let mut styles = StyleProperties::default();
    let Some(obj) = attrs.as_object() else {
        return styles;
    };

    let style_obj = obj.get("style").and_then(|s| s.as_object());

    // Spacing: Padding & Margin
    if let Some(spacing) = style_obj
        .and_then(|s| s.get("spacing"))
        .and_then(|sp| sp.as_object())
    {
        if let Some(pad) = spacing.get("padding").and_then(|p| p.as_object()) {
            styles.padding = Some(Spacing {
                top: pad
                    .get("top")
                    .and_then(|v| v.as_str())
                    .map(normalize_wp_preset),
                right: pad
                    .get("right")
                    .and_then(|v| v.as_str())
                    .map(normalize_wp_preset),
                bottom: pad
                    .get("bottom")
                    .and_then(|v| v.as_str())
                    .map(normalize_wp_preset),
                left: pad
                    .get("left")
                    .and_then(|v| v.as_str())
                    .map(normalize_wp_preset),
            });
        }
        if let Some(mar) = spacing.get("margin").and_then(|m| m.as_object()) {
            styles.margin = Some(Spacing {
                top: mar
                    .get("top")
                    .and_then(|v| v.as_str())
                    .map(normalize_wp_preset),
                right: mar
                    .get("right")
                    .and_then(|v| v.as_str())
                    .map(normalize_wp_preset),
                bottom: mar
                    .get("bottom")
                    .and_then(|v| v.as_str())
                    .map(normalize_wp_preset),
                left: mar
                    .get("left")
                    .and_then(|v| v.as_str())
                    .map(normalize_wp_preset),
            });
        }
    }

    // Colors
    let color_obj = style_obj
        .and_then(|s| s.get("color"))
        .and_then(|c| c.as_object());

    let bg_color = color_obj
        .and_then(|c| c.get("background"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            obj.get("backgroundColor")
                .and_then(|v| v.as_str())
                .map(|slug| format!("var(--wp--preset--color--{})", slug))
        });

    if let Some(bg) = bg_color {
        styles.background = Some(BackgroundConfig {
            color: Some(bg),
            image_url: None,
            position: None,
            size: None,
        });
    }

    if let Some(grad) = color_obj
        .and_then(|c| c.get("gradient"))
        .and_then(|v| v.as_str())
    {
        styles.background_gradient = Some(grad.to_string());
    }

    let text_color = color_obj
        .and_then(|c| c.get("text"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            obj.get("textColor")
                .and_then(|v| v.as_str())
                .map(|slug| format!("var(--wp--preset--color--{})", slug))
        });

    // Typography
    let typo_obj = style_obj
        .and_then(|s| s.get("typography"))
        .and_then(|t| t.as_object());

    let font_size = typo_obj
        .and_then(|t| t.get("fontSize"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| {
            obj.get("fontSize")
                .and_then(|v| v.as_str())
                .map(|slug| format!("var(--wp--preset--font-size--{})", slug))
        });

    let font_weight = typo_obj
        .and_then(|t| t.get("fontWeight"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let line_height = typo_obj
        .and_then(|t| t.get("lineHeight"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let text_align = obj
        .get("textAlign")
        .or_else(|| obj.get("align"))
        .and_then(|v| v.as_str())
        .filter(|s| matches!(*s, "left" | "center" | "right" | "justify"))
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

    // Border
    if let Some(bdr) = style_obj
        .and_then(|s| s.get("border"))
        .and_then(|b| b.as_object())
    {
        let radius = bdr.get("radius").and_then(|r| {
            r.as_str()
                .map(|s| s.to_string())
                .or_else(|| r.as_u64().map(|n| format!("{}px", n)))
        });
        let width = bdr
            .get("width")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let color = bdr
            .get("color")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let style = bdr
            .get("style")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        if radius.is_some() || width.is_some() || color.is_some() || style.is_some() {
            styles.border = Some(BorderConfig {
                radius,
                width,
                color,
                style,
            });
        }
    }

    styles.sync_flat_and_nested();
    styles
}

pub(crate) fn normalize_wp_preset(val: &str) -> String {
    // Converts "var:preset|spacing|50" -> "var(--wp--preset--spacing--50)"
    if let Some(rest) = val.strip_prefix("var:preset|") {
        let parts: Vec<&str> = rest.split('|').collect();
        if parts.len() == 2 {
            return format!("var(--wp--preset--{}--{})", parts[0], parts[1]);
        }
    }
    val.to_string()
}

pub(crate) fn extract_attr_from_html(html: &str, attr: &str) -> Option<String> {
    let pattern = format!(r#"{}\s*=\s*["']([^"']*)["']"#, regex::escape(attr));
    let re = Regex::new(&pattern).ok()?;
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

pub(crate) fn detect_heading_level(html: &str) -> Option<u8> {
    let re = Regex::new(r"(?i)<h([1-6])\b").ok()?;
    re.captures(html)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u8>().ok())
}

pub(crate) fn strip_outer_heading_tag(html: &str) -> String {
    let re = Regex::new(r"(?is)^\s*<h[1-6][^>]*>(.*?)</h[1-6]>\s*$").unwrap();
    if let Some(caps) = re.captures(html) {
        if let Some(inner) = caps.get(1) {
            return strip_html_tags(inner.as_str());
        }
    }
    strip_html_tags(html)
}

#[allow(dead_code)]
pub(crate) fn extract_tag_inner(html: &str, tag: &str) -> Option<String> {
    let pattern = format!(r"(?is)<{0}[^>]*>(.*?)</{0}>", regex::escape(tag));
    let re = Regex::new(&pattern).ok()?;
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| strip_html_tags(m.as_str()))
}

pub(crate) fn strip_html_tags(html: &str) -> String {
    let re = Regex::new(r"<[^>]+>").unwrap();
    re.replace_all(html, "").trim().to_string()
}
