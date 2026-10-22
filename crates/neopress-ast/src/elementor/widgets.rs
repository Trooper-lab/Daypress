#![allow(unused_imports)]
use crate::schema::{
    AccordionItem, BackgroundConfig, BorderConfig, IconListItem, NeopressLayout, NeopressNode,
    NodeKind, SocialIconItem, Spacing, StyleProperties, TabItem, ToggleItem, TypographyConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use super::*;

pub(crate) fn map_elementor_widget(
    widget_type: &str,
    settings: &Value,
    styles: &mut StyleProperties,
    children: &mut Vec<NeopressNode>,
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
            })
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
            })
        }
        "social-icons" => {
            let mut items = Vec::new();
            if let Some(icons) = settings.get("social_icon_list").and_then(|t| t.as_array()) {
                for icon in icons {
                    let icon_name = icon
                        .get("social_icon")
                        .and_then(|i| i.get("value"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("fa-brands fa-facebook")
                        .to_string();
                    let link =
                        resolve_setting_link(icon, "link", ctx).unwrap_or_else(|| "#".to_string());
                    items.push(SocialIconItem {
                        icon: icon_name,
                        link,
                    });
                }
            }
            Some(NodeKind::SocialIcons { items })
        }
        "star-rating" | "rating" => {
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
        "audio" => {
            let url = settings
                .get("audio_url")
                .or_else(|| settings.get("url"))
                .and_then(|v| v.as_str())
                .map(|u| ctx.normalize_url(u))
                .unwrap_or_default();
            Some(NodeKind::Audio { url })
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
        "form" => {
            let form_id = settings
                .get("form_id")
                .and_then(|v| {
                    v.as_i64()
                        .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
                })
                .unwrap_or(1);
            let title = settings
                .get("title")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let button_text = settings
                .get("button_text")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let endpoint = settings
                .get("action")
                .or_else(|| settings.get("endpoint"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());

            if let Some(fields) = settings.get("form_fields").and_then(|f| f.as_array()) {
                let redirect_url = resolve_setting_link(settings, "redirect_to", ctx);
                let success_msg = settings
                    .get("success_message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Uw inzending is succesvol verstuurd.");
                let btn_lbl = button_text.as_deref().unwrap_or("Verzenden");
                let custom_fields_html =
                    compile_elementor_form_fields(fields, btn_lbl, success_msg, redirect_url.as_deref());
                children.push(NeopressNode {
                    id: format!("form_fields_{}", form_id),
                    kind: NodeKind::CustomHtml {
                        raw: custom_fields_html,
                    },
                    styles: StyleProperties::default(),
                    children: Vec::new(),
                });
            }

            Some(NodeKind::Form {
                form_id,
                title,
                button_text,
                endpoint,
            })
        }
        "call-to-action" | "cta" => {
            let title = settings
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let description = settings
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let button_text = settings
                .get("button")
                .or_else(|| settings.get("button_text"))
                .and_then(|v| v.as_str())
                .unwrap_or("Learn More")
                .to_string();
            let button_url = resolve_setting_link(settings, "link", ctx)
                .or_else(|| {
                    settings
                        .get("button_url")
                        .and_then(|v| v.as_str())
                        .map(|u| ctx.normalize_url(u))
                })
                .unwrap_or_else(|| "#".to_string());
            Some(NodeKind::CallToAction {
                title,
                description,
                button_text,
                button_url,
            })
        }
        "price-table" => {
            let title = settings
                .get("heading")
                .or_else(|| settings.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("Pro Plan")
                .to_string();
            let currency = settings
                .get("currency_symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("$");
            let raw_price = settings
                .get("price")
                .and_then(extract_spacing_val)
                .unwrap_or_else(|| "29".to_string());
            let price = if raw_price.starts_with('$')
                || raw_price.starts_with('€')
                || raw_price.starts_with('£')
            {
                raw_price
            } else {
                format!("{}{}", currency, raw_price)
            };
            let period = settings
                .get("period")
                .and_then(|v| v.as_str())
                .unwrap_or("/ month")
                .to_string();
            let mut features = Vec::new();
            if let Some(list) = settings.get("features_list").and_then(|v| v.as_array()) {
                for item in list {
                    if let Some(txt) = item
                        .get("item_text")
                        .or_else(|| item.get("text"))
                        .and_then(|v| v.as_str())
                    {
                        features.push(txt.to_string());
                    } else if let Some(s) = item.as_str() {
                        features.push(s.to_string());
                    }
                }
            }
            let button_text = settings
                .get("button_text")
                .and_then(|v| v.as_str())
                .unwrap_or("Get Started")
                .to_string();
            let button_url =
                resolve_setting_link(settings, "link", ctx).unwrap_or_else(|| "#".to_string());
            Some(NodeKind::PriceTable {
                title,
                price,
                period,
                features,
                button_text,
                button_url,
            })
        }
        "price-list" => {
            let mut items = Vec::new();
            if let Some(list) = settings.get("price_list").and_then(|v| v.as_array()) {
                for it in list {
                    let title = it
                        .get("title")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    let price = it
                        .get("price")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
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
        "flip-box" => {
            let title = settings
                .get("title_text_a")
                .or_else(|| settings.get("title_text_b"))
                .and_then(|v| v.as_str())
                .unwrap_or("Feature")
                .to_string();
            let description = settings
                .get("description_text_b")
                .or_else(|| settings.get("description_text_a"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let button_text = settings
                .get("button_text")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let button_url =
                resolve_setting_link(settings, "link", ctx).unwrap_or_else(|| "#".to_string());
            if !button_text.is_empty() {
                Some(NodeKind::CallToAction {
                    title,
                    description,
                    button_text,
                    button_url,
                })
            } else {
                let icon = settings
                    .get("selected_icon")
                    .and_then(|i| i.get("value"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("fa-solid fa-layer-group")
                    .to_string();
                Some(NodeKind::IconBox {
                    icon,
                    title,
                    description,
                    link: if button_url == "#" {
                        None
                    } else {
                        Some(button_url)
                    },
                })
            }
        }
        "slides" => {
            if let Some(slides) = settings.get("slides").and_then(|v| v.as_array()) {
                if let Some(first) = slides.first() {
                    let title = first
                        .get("heading")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let description = first
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let button_text = first
                        .get("button_text")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Learn More")
                        .to_string();
                    let button_url =
                        resolve_setting_link(first, "link", ctx).unwrap_or_else(|| "#".to_string());
                    return Some(NodeKind::CallToAction {
                        title,
                        description,
                        button_text,
                        button_url,
                    });
                }
            }
            Some(NodeKind::CallToAction {
                title: String::new(),
                description: String::new(),
                button_text: "Learn More".to_string(),
                button_url: "#".to_string(),
            })
        }
        "gallery" | "image-gallery" | "basic-gallery" | "image-carousel" => {
            let mut imgs_html = String::new();
            let list = settings
                .get("gallery")
                .or_else(|| settings.get("carousel"))
                .or_else(|| settings.get("wp_gallery"))
                .and_then(|v| v.as_array());
            let gap = settings
                .get("gap")
                .and_then(|g| g.get("size"))
                .and_then(extract_spacing_val)
                .unwrap_or_else(|| "12".to_string());
            if let Some(arr) = list {
                for item in arr {
                    if let Some(raw_url) = item.get("url").and_then(|v| v.as_str()) {
                        let url = ctx.normalize_url(raw_url);
                        let alt = item
                            .get("alt")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Gallery Image");
                        imgs_html.push_str(&format!(
                            "<a href=\"{}\" class=\"np-gallery-item np-lightbox-trigger\"><img src=\"{}\" alt=\"{}\" class=\"np-gallery-img\" loading=\"lazy\" /></a>",
                            escape_html_attr(&url),
                            escape_html_attr(&url),
                            escape_html_attr(alt)
                        ));
                    }
                }
            }
            Some(NodeKind::CustomHtml {
                raw: format!(
                    "<div class=\"np-gallery\" style=\"display:grid;grid-template-columns:repeat(auto-fill,minmax(250px,1fr));gap:{}px;\">{}</div>",
                    gap, imgs_html
                ),
            })
        }
        "posts" | "archive-posts" | "loop-grid" => {
            let count = settings
                .get("posts_per_page")
                .and_then(|v| v.as_u64())
                .unwrap_or(6);
            Some(NodeKind::CustomHtml {
                raw: format!(
                    "<div class=\"np-posts-grid\" data-posts-per-page=\"{}\"></div>",
                    count
                ),
            })
        }
        "nav-menu" => {
            let menu_slug = settings
                .get("menu")
                .and_then(|v| v.as_str())
                .unwrap_or("primary");
            let layout = settings
                .get("layout")
                .and_then(|v| v.as_str())
                .unwrap_or("horizontal");
            let align = settings
                .get("align_items")
                .and_then(|v| v.as_str())
                .unwrap_or("left");

            let items_opt = ctx.menus.get(menu_slug).or_else(|| {
                if menu_slug == "primary" {
                    ctx.menus.values().next()
                } else {
                    None
                }
            });

            let mut list_html = String::new();
            if let Some(items) = items_opt {
                list_html.push_str("<ul class=\"np-nav-menu-list\">");
                for item in items {
                    list_html.push_str(&format!(
                        "<li class=\"np-nav-menu-item\"><a href=\"{}\" class=\"np-nav-menu-link\">{}</a></li>",
                        escape_html_attr(&item.url),
                        escape_html_attr(&item.title)
                    ));
                }
                list_html.push_str("</ul>");
            }

            Some(NodeKind::CustomHtml {
                raw: format!(
                    "<nav class=\"np-nav-menu np-nav-layout-{} np-nav-align-{}\" data-menu=\"{}\">{}</nav>",
                    escape_html_attr(layout),
                    escape_html_attr(align),
                    escape_html_attr(menu_slug),
                    list_html
                ),
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
        "countdown" => {
            let due_date = settings
                .get("due_date")
                .and_then(|v| v.as_str())
                .unwrap_or("00:00:00")
                .to_string();
            let title = settings
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Countdown")
                .to_string();
            Some(NodeKind::Benchmark {
                label: title,
                value: due_date,
                comparison: "Countdown Timer".to_string(),
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
        other => {
            let raw = settings
                .get("raw")
                .or_else(|| settings.get("editor"))
                .and_then(|v| v.as_str())
                .map(|s| ctx.normalize_html(s))
                .unwrap_or_else(|| {
                    format!("<!-- Elementor widget '{}' imported gracefully -->", other)
                });
            Some(NodeKind::CustomHtml { raw })
        }
    }
}

pub(crate) fn compile_elementor_form_fields(
    fields: &[Value],
    button_text: &str,
    success_message: &str,
    redirect_url: Option<&str>,
) -> String {
    let mut html = String::from("<div class=\"np-form-fields np-elementor-form-grid\">\n");
    for (idx, field) in fields.iter().enumerate() {
        let field_type = field
            .get("field_type")
            .and_then(|v| v.as_str())
            .unwrap_or("text");
        if field_type == "recaptcha" || field_type == "recaptcha_v3" || field_type == "honeypot" {
            continue;
        }
        let custom_id = field
            .get("custom_id")
            .or_else(|| field.get("_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("field");
        let name = if custom_id.is_empty() {
            format!("field_{}", idx)
        } else {
            custom_id.to_string()
        };
        let label = field
            .get("field_label")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let placeholder = field
            .get("placeholder")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let required = field
            .get("required")
            .and_then(|v| v.as_str().map(|s| s == "true" || s == "yes").or_else(|| v.as_bool()))
            .unwrap_or(false);
        let req_attr = if required { " required" } else { "" };
        let req_mark = if required {
            " <span class=\"np-form-required\">*</span>"
        } else {
            ""
        };
        let width_pct = field
            .get("width")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .unwrap_or("100");
        let col_class = match width_pct {
            "50" => "np-field-group np-field-w50",
            "33" => "np-field-group np-field-w33",
            "25" => "np-field-group np-field-w25",
            _ => "np-field-group np-field-w100",
        };

        let label_clean = label
            .replace("<strong>", "")
            .replace("</strong>", "")
            .replace("<b>", "")
            .replace("</b>", "");

        html.push_str(&format!("<div class=\"{}\">\n", col_class));
        if !label_clean.is_empty() {
            html.push_str(&format!(
                "<label class=\"np-field-label\">{}{}</label>\n",
                escape_html_attr(&label_clean),
                req_mark
            ));
        }

        match field_type {
            "textarea" => {
                let rows = field
                    .get("rows")
                    .and_then(extract_spacing_val)
                    .unwrap_or_else(|| "4".to_string());
                html.push_str(&format!(
                    "<textarea name=\"{}\" class=\"np-textarea\" rows=\"{}\" placeholder=\"{}\"{}></textarea>\n",
                    escape_html_attr(&name),
                    escape_html_attr(&rows),
                    escape_html_attr(placeholder),
                    req_attr
                ));
            }
            "checkbox" | "radio" => {
                let opts_raw = field
                    .get("field_options")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let inline_cls = field
                    .get("inline_list")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|_| " np-Subgroup-inline")
                    .unwrap_or("");
                html.push_str(&format!(
                    "<div class=\"np-field-subgroup{}\">\n",
                    inline_cls
                ));
                for opt in opts_raw.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
                    html.push_str(&format!(
                        "<label class=\"np-option-label\"><input type=\"{}\" name=\"{}\" value=\"{}\" /> <span>{}</span></label>\n",
                        field_type,
                        escape_html_attr(&name),
                        escape_html_attr(opt),
                        escape_html_attr(opt)
                    ));
                }
                html.push_str("</div>\n");
            }
            "select" => {
                let opts_raw = field
                    .get("field_options")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                html.push_str(&format!(
                    "<select name=\"{}\" class=\"np-input\"{}>\n",
                    escape_html_attr(&name),
                    req_attr
                ));
                for opt in opts_raw.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
                    html.push_str(&format!(
                        "<option value=\"{}\">{}</option>\n",
                        escape_html_attr(opt),
                        escape_html_attr(opt)
                    ));
                }
                html.push_str("</select>\n");
            }
            "upload" => {
                let mult = if field.get("allow_multiple_upload").and_then(|v| v.as_str()) == Some("yes") {
                    " multiple"
                } else {
                    ""
                };
                html.push_str(&format!(
                    "<input type=\"file\" name=\"{}\" class=\"np-input np-file-input\"{}{}/>\n",
                    escape_html_attr(&name),
                    mult,
                    req_attr
                ));
            }
            "date" => {
                html.push_str(&format!(
                    "<input type=\"date\" name=\"{}\" class=\"np-input\" placeholder=\"{}\"{} />\n",
                    escape_html_attr(&name),
                    escape_html_attr(placeholder),
                    req_attr
                ));
            }
            "email" | "tel" | "number" | "url" => {
                html.push_str(&format!(
                    "<input type=\"{}\" name=\"{}\" class=\"np-input\" placeholder=\"{}\"{} />\n",
                    field_type,
                    escape_html_attr(&name),
                    escape_html_attr(placeholder),
                    req_attr
                ));
            }
            _ => {
                html.push_str(&format!(
                    "<input type=\"text\" name=\"{}\" class=\"np-input\" placeholder=\"{}\"{} />\n",
                    escape_html_attr(&name),
                    escape_html_attr(placeholder),
                    req_attr
                ));
            }
        }
        html.push_str("</div>\n");
    }

    let redirect_attr = redirect_url
        .map(|u| format!(" data-redirect-url=\"{}\"", escape_html_attr(u)))
        .unwrap_or_default();
    html.push_str(&format!(
        "<div class=\"np-field-group np-field-w100\"><button type=\"submit\" class=\"np-button np-form-submit\">{}</button></div>\n</div>\n<div class=\"np-form-status\" data-success-msg=\"{}\"{} aria-live=\"polite\"></div>",
        escape_html_attr(button_text),
        escape_html_attr(success_message),
        redirect_attr
    ));
    html
}
