use crate::elementor::context::{resolve_setting_link, ElementorContext};
use crate::elementor::styles::extract_global_color;
use crate::elementor::widgets::parse_spacing_obj;
use crate::schema::{NodeKind, StyleProperties};
use serde_json::Value;
use std::collections::HashMap;

pub(crate) fn map_basic_text_widget(
    widget_type: &str,
    settings: &Value,
    styles: &mut StyleProperties,
    ctx: &ElementorContext,
) -> Option<NodeKind> {
    let globals = settings.get("__globals__").and_then(|g| g.as_object());

    match widget_type {
        "heading" => {
            let text = settings
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .trim()
                .to_string();

            let level = settings
                .get("header_size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.strip_prefix('h'))
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(2);

            if let Some(color) = globals
                .and_then(|g| extract_global_color(g, "title_color"))
                .or_else(|| {
                    settings
                        .get("title_color")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                })
            {
                let typo = styles.typography.get_or_insert_with(Default::default);
                typo.color = Some(color);
            }

            if let Some(align) = settings
                .get("align")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                let typo = styles.typography.get_or_insert_with(Default::default);
                typo.text_align = Some(align.to_string());
            }

            Some(NodeKind::Heading { text, level })
        }
        "text-editor" => {
            let raw_html = settings
                .get("editor")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let content_html = ctx.normalize_html(raw_html);
            Some(NodeKind::Text { content_html })
        }
        "button" => {
            let text = settings
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("Click Here")
                .to_string();

            let url = resolve_setting_link(settings, "link", ctx).unwrap_or_else(|| "#".to_string());

            let is_external = settings
                .get("link")
                .and_then(|lnk| lnk.get("is_external"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let target = if is_external { "_blank" } else { "_self" }.to_string();

            if let Some(bg_col) = globals
                .and_then(|g| {
                    extract_global_color(g, "background_color")
                        .or_else(|| extract_global_color(g, "button_background_color"))
                })
                .or_else(|| {
                    settings
                        .get("button_background_color")
                        .or_else(|| settings.get("background_color"))
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty() && *s != "classic")
                        .map(|s| s.to_string())
                })
            {
                let bg = styles.background.get_or_insert_with(Default::default);
                bg.color = Some(bg_col);
            }
            if let Some(txt_col) = globals
                .and_then(|g| extract_global_color(g, "button_text_color"))
                .or_else(|| {
                    settings
                        .get("button_text_color")
                        .or_else(|| settings.get("background_color"))
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                })
            {
                let typo = styles.typography.get_or_insert_with(Default::default);
                typo.color = Some(txt_col);
            }

            if let Some(pad) = settings.get("text_padding").and_then(|v| v.as_object()) {
                if let Some(sp) = parse_spacing_obj(pad) {
                    styles.padding = Some(sp);
                }
            }

            Some(NodeKind::Button { text, url, target })
        }
        "spacer" => {
            let height = settings
                .get("space")
                .and_then(|s| s.get("size"))
                .and_then(|v| v.as_u64())
                .unwrap_or(50) as u32;
            Some(NodeKind::Spacer { height })
        }
        "divider" => {
            let style = settings
                .get("style")
                .and_then(|v| v.as_str())
                .unwrap_or("solid")
                .to_string();
            Some(NodeKind::Divider { style })
        }
        "icon" => {
            let name = settings
                .get("selected_icon")
                .and_then(|i| i.get("value"))
                .and_then(|v| v.as_str())
                .or_else(|| settings.get("icon").and_then(|v| v.as_str()))
                .unwrap_or("fa fa-star")
                .to_string();
            let size = settings
                .get("size")
                .and_then(|s| s.get("size"))
                .and_then(|v| v.as_u64())
                .map(|s| s as u32);
            let color = globals
                .and_then(|g| extract_global_color(g, "primary_color"))
                .or_else(|| {
                    settings
                        .get("primary_color")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                });
            if let Some(href) = resolve_setting_link(settings, "link", ctx) {
                styles.link_url = Some(href);
            }
            Some(NodeKind::Icon { name, size, color })
        }
        "counter" => {
            let number = settings
                .get("ending_number")
                .and_then(|v| {
                    v.as_i64()
                        .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
                })
                .unwrap_or(100);
            let prefix = settings
                .get("prefix")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let suffix = settings
                .get("suffix")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let title = settings
                .get("title")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            Some(NodeKind::Counter {
                number,
                prefix,
                suffix,
                title,
            })
        }
        "progress" => {
            let title = settings
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("My Skill")
                .to_string();
            let percent = settings
                .get("percent")
                .and_then(|p| p.get("size"))
                .and_then(|v| v.as_u64())
                .or_else(|| settings.get("percent").and_then(|v| v.as_u64()))
                .unwrap_or(50) as u8;
            let display_text = settings
                .get("display_percentage")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            Some(NodeKind::ProgressBar {
                title,
                percent,
                display_text,
                track_color: None,
                bar_color: None,
                height: None,
            })
        }
        "alert" => {
            let alert_type = settings
                .get("alert_type")
                .and_then(|v| v.as_str())
                .unwrap_or("info")
                .to_string();
            let title = settings
                .get("alert_title")
                .and_then(|v| v.as_str())
                .unwrap_or("Alert")
                .to_string();
            let description = settings
                .get("alert_description")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let dismissible = settings
                .get("show_dismiss")
                .and_then(|v| v.as_str())
                .map(|s| s == "show")
                .unwrap_or(true);
            Some(NodeKind::Alert {
                alert_type,
                title,
                description,
                dismissible,
                icon: None,
            })
        }
        "rating" => {
            let rating = settings
                .get("rating")
                .and_then(|v| v.as_f64())
                .unwrap_or(5.0) as f32;
            let max_rating = settings
                .get("rating_scale")
                .and_then(|v| v.as_u64())
                .unwrap_or(5) as u8;
            let title = settings
                .get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(NodeKind::Rating {
                rating,
                max_rating,
                title,
            })
        }
        "star-rating" => {
            let rating = settings
                .get("rating")
                .and_then(|v| v.as_f64())
                .unwrap_or(5.0) as f32;
            let max_rating = settings
                .get("rating_scale")
                .and_then(|v| v.as_u64())
                .unwrap_or(5) as u8;
            let title = settings
                .get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let star_color = settings
                .get("star_color")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let star_size = settings
                .get("star_size")
                .and_then(|s| s.get("size"))
                .and_then(|v| v.as_u64())
                .map(|v| v as u32);

            Some(NodeKind::StarRating {
                rating,
                max_rating,
                title,
                star_color,
                star_size,
            })
        }
        "menu-anchor" => {
            let anchor_id = settings
                .get("anchor")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            Some(NodeKind::MenuAnchor { anchor_id })
        }
        "html" => {
            let raw = settings
                .get("html")
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            Some(NodeKind::CustomHtml {
                raw: ctx.normalize_html(raw),
            })
        }
        "shortcode" => {
            let sc = settings
                .get("shortcode")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .trim();
            Some(NodeKind::ShortcodeFallback {
                tag: "shortcode".to_string(),
                attributes: HashMap::new(),
                content: sc.to_string(),
            })
        }
        _ => None,
    }
}
