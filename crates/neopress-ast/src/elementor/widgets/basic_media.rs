use crate::elementor::context::{resolve_setting_link, ElementorContext};
use crate::elementor::normalize_video_embed_url;
use crate::elementor::resolve_dynamic_tag;
use crate::schema::{AccordionItem, IconListItem, NodeKind, StyleProperties, TabItem, ToggleItem};
use serde_json::Value;

pub(crate) fn map_basic_media_widget(
    widget_type: &str,
    settings: &Value,
    _styles: &mut StyleProperties,
    ctx: &ElementorContext,
) -> Option<NodeKind> {
    match widget_type {
        "image" | "theme-site-logo" => {
            let src = settings
                .get("__dynamic__")
                .and_then(|d| d.get("image"))
                .and_then(|v| v.as_str())
                .and_then(|tag| resolve_dynamic_tag(tag, ctx))
                .or_else(|| {
                    settings
                        .get("image")
                        .and_then(|img| img.get("url"))
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|u| ctx.normalize_url(u))
                })
                .or_else(|| {
                    if widget_type == "theme-site-logo" {
                        ctx.site_logo_url.clone()
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            let alt = settings
                .get("image")
                .and_then(|img| img.get("alt"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .unwrap_or(if widget_type == "theme-site-logo" {
                    "Site Logo"
                } else {
                    ""
                })
                .to_string();

            let link = resolve_setting_link(settings, "link", ctx).or_else(|| {
                if widget_type == "theme-site-logo" {
                    Some("/".to_string())
                } else {
                    None
                }
            });

            let width = settings
                .get("width")
                .and_then(|w| w.get("size"))
                .and_then(|s| s.as_u64())
                .map(|s| s as u32);

            Some(NodeKind::Image {
                src,
                alt,
                width,
                height: None,
                link,
            })
        }
        "accordion" => {
            let mut items = Vec::new();
            if let Some(tabs) = settings.get("tabs").and_then(|t| t.as_array()) {
                for tab in tabs {
                    let title = tab
                        .get("tab_title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Accordion Item")
                        .to_string();
                    let raw_html = tab
                        .get("tab_content")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    items.push(AccordionItem {
                        title,
                        content_html: ctx.normalize_html(raw_html),
                    });
                }
            }
            Some(NodeKind::Accordion { items })
        }
        "video" => {
            let raw_url = settings
                .get("youtube_url")
                .or_else(|| settings.get("vimeo_url"))
                .or_else(|| settings.get("dailymotion_url"))
                .or_else(|| settings.get("hosted_url").and_then(|h| h.get("url")))
                .or_else(|| settings.get("url"))
                .and_then(|v| v.as_str())
                .unwrap_or_default();
            let url = normalize_video_embed_url(raw_url);
            let aspect_ratio = settings
                .get("aspect_ratio")
                .and_then(|v| v.as_str())
                .map(|s| match s {
                    "169" | "16:9" => "16/9".to_string(),
                    "43" | "4:3" => "4/3".to_string(),
                    "32" | "3:2" => "3/2".to_string(),
                    "219" | "21:9" => "21/9".to_string(),
                    "11" | "1:1" => "1/1".to_string(),
                    "916" | "9:16" => "9/16".to_string(),
                    other => other.replace(':', "/"),
                })
                .unwrap_or_else(|| "16/9".to_string());
            let autoplay = settings
                .get("autoplay")
                .and_then(|v| v.as_str().map(|s| s == "yes").or_else(|| v.as_bool()))
                .unwrap_or(false);
            let muted = settings
                .get("mute")
                .or_else(|| settings.get("muted"))
                .and_then(|v| v.as_str().map(|s| s == "yes").or_else(|| v.as_bool()))
                .unwrap_or(false);
            let loop_video = settings
                .get("loop")
                .and_then(|v| v.as_str().map(|s| s == "yes").or_else(|| v.as_bool()))
                .unwrap_or(false);
            Some(NodeKind::Video {
                url,
                aspect_ratio,
                autoplay,
                muted,
                loop_video,
            })
        }
        "google_maps" => {
            let address = settings
                .get("address")
                .and_then(|v| v.as_str())
                .unwrap_or("London Eye, UK")
                .to_string();
            let zoom = settings
                .get("zoom")
                .and_then(|z| z.get("size"))
                .and_then(|v| v.as_u64())
                .map(|z| z as u8);
            Some(NodeKind::Map { address, zoom })
        }
        "icon-box" => {
            let icon = settings
                .get("selected_icon")
                .and_then(|i| i.get("value"))
                .and_then(|v| v.as_str())
                .or_else(|| settings.get("icon").and_then(|v| v.as_str()))
                .unwrap_or("fa-solid fa-cubes")
                .to_string();
            let title = settings
                .get("title_text")
                .or_else(|| settings.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("Feature Title")
                .to_string();
            let description = settings
                .get("description_text")
                .or_else(|| settings.get("description"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let link = resolve_setting_link(settings, "link", ctx);
            Some(NodeKind::IconBox {
                icon,
                title,
                description,
                link,
            })
        }
        "image-box" => {
            let image_url = settings
                .get("image")
                .and_then(|img| img.get("url"))
                .and_then(|v| v.as_str())
                .map(|u| ctx.normalize_url(u))
                .unwrap_or_default();
            let title = settings
                .get("title_text")
                .or_else(|| settings.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("Image Feature")
                .to_string();
            let description = settings
                .get("description_text")
                .or_else(|| settings.get("description"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let link = resolve_setting_link(settings, "link", ctx);
            Some(NodeKind::ImageBox {
                image_url,
                title,
                description,
                link,
            })
        }
        "icon-list" => {
            let mut items = Vec::new();
            if let Some(icon_list) = settings.get("icon_list").and_then(|t| t.as_array()) {
                for it in icon_list {
                    let text = it
                        .get("text")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let icon = if let Some(sel) = it.get("selected_icon") {
                        sel.get("value")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string()
                    } else if let Some(ic) = it.get("icon").and_then(|v| v.as_str()) {
                        ic.to_string()
                    } else {
                        "fa-solid fa-check".to_string()
                    };
                    let link = resolve_setting_link(it, "link", ctx);
                    items.push(IconListItem { icon, text, link });
                }
            }
            Some(NodeKind::IconList { items })
        }
        "price-list" => {
            let mut items = Vec::new();
            if let Some(list) = settings.get("price_list").and_then(|v| v.as_array()) {
                for it in list {
                    let title = it.get("title").and_then(|v| v.as_str()).unwrap_or_default();
                    let price = it.get("price").and_then(|v| v.as_str()).unwrap_or_default();
                    let text = if price.is_empty() {
                        title.to_string()
                    } else {
                        format!("{} — {}", title, price)
                    };
                    let link = resolve_setting_link(it, "link", ctx);
                    items.push(IconListItem {
                        icon: "fa-solid fa-tag".to_string(),
                        text,
                        link,
                    });
                }
            }
            Some(NodeKind::IconList { items })
        }
        "testimonial" => {
            let content = settings
                .get("testimonial_content")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let name = settings
                .get("testimonial_name")
                .and_then(|v| v.as_str())
                .unwrap_or("John Doe")
                .to_string();
            let job = settings
                .get("testimonial_job")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let image_url = settings
                .get("testimonial_image")
                .and_then(|img| img.get("url"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|u| ctx.normalize_url(u));
            Some(NodeKind::Testimonial {
                content,
                name,
                job,
                image_url,
            })
        }
        "blockquote" => {
            let content = settings
                .get("blockquote_content")
                .or_else(|| settings.get("content"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let name = settings
                .get("author_name")
                .or_else(|| settings.get("author"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            Some(NodeKind::Testimonial {
                content,
                name,
                job: String::new(),
                image_url: None,
            })
        }
        "tabs" => {
            let mut items = Vec::new();
            if let Some(tabs) = settings.get("tabs").and_then(|t| t.as_array()) {
                for tab in tabs {
                    let title = tab
                        .get("tab_title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Tab")
                        .to_string();
                    let content_html = tab
                        .get("tab_content")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    items.push(TabItem {
                        title,
                        content_html: ctx.normalize_html(&content_html),
                    });
                }
            }
            Some(NodeKind::Tabs { items })
        }
        "toggle" => {
            let mut items = Vec::new();
            if let Some(tabs) = settings.get("tabs").and_then(|t| t.as_array()) {
                for tab in tabs {
                    let title = tab
                        .get("tab_title")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Toggle Item")
                        .to_string();
                    let content_html = tab
                        .get("tab_content")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    items.push(ToggleItem {
                        title,
                        content_html: ctx.normalize_html(&content_html),
                    });
                }
            }
            Some(NodeKind::Toggle { items })
        }
        "audio" => {
            let url = settings
                .get("audio_url")
                .or_else(|| settings.get("url"))
                .and_then(|v| v.as_str())
                .map(|u| ctx.normalize_url(u))
                .unwrap_or_default();
            Some(NodeKind::Audio { url })
        }
        _ => None,
    }
}
