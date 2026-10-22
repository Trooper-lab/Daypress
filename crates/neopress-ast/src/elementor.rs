use crate::schema::{
    AccordionItem, BackgroundConfig, BorderConfig, IconListItem, NeopressLayout, NeopressNode,
    NodeKind, SocialIconItem, Spacing, StyleProperties, TabItem, ToggleItem, TypographyConfig,
};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// Parses raw Elementor JSON (`_elementor_data`) into a canonical NeopressLayout AST.
pub fn transpile_elementor_json(raw_json: &str) -> Result<NeopressLayout, String> {
    let parsed: Value = serde_json::from_str(raw_json)
        .map_err(|e| format!("Failed to parse Elementor JSON: {}", e))?;

    let root_children = if let Some(arr) = parsed.as_array() {
        arr.iter().filter_map(parse_elementor_node).collect()
    } else if let Some(obj) = parsed.as_object() {
        if let Some(elements) = obj.get("elements").and_then(|e| e.as_array()) {
            elements.iter().filter_map(parse_elementor_node).collect()
        } else {
            vec![parse_elementor_node(&parsed).ok_or("Invalid Elementor node")?]
        }
    } else {
        return Err("Elementor JSON root must be an array or object".to_string());
    };

    Ok(NeopressLayout {
        version: 1,
        root: NeopressNode {
            id: "elem_root".to_string(),
            kind: NodeKind::Section,
            styles: StyleProperties::default(),
            children: root_children,
        },
        global_styles: None,
    })
}

/// Converts YouTube watch/short URLs (`watch?v=ID`, `youtu.be/ID`) and Vimeo URLs into embed URLs.
pub fn normalize_video_embed_url(url: &str) -> String {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // YouTube watch?v=ID
    if (trimmed.contains("youtube.com/watch") || trimmed.contains("youtube-nocookie.com/watch"))
        && trimmed.contains("v=")
    {
        if let Some(pos) = trimmed.find("v=") {
            let after_v = &trimmed[pos + 2..];
            let video_id: String = after_v
                .chars()
                .take_while(|c| *c != '&' && *c != '#' && *c != '?')
                .collect();
            if !video_id.is_empty() {
                return format!("https://www.youtube.com/embed/{}", video_id);
            }
        }
    }

    // YouTube short link youtu.be/ID
    if let Some(pos) = trimmed.find("youtu.be/") {
        let after = &trimmed[pos + "youtu.be/".len()..];
        let video_id: String = after
            .chars()
            .take_while(|c| *c != '?' && *c != '&' && *c != '#' && *c != '/')
            .collect();
        if !video_id.is_empty() {
            return format!("https://www.youtube.com/embed/{}", video_id);
        }
    }

    // YouTube shorts youtube.com/shorts/ID
    if let Some(pos) = trimmed.find("youtube.com/shorts/") {
        let after = &trimmed[pos + "youtube.com/shorts/".len()..];
        let video_id: String = after
            .chars()
            .take_while(|c| *c != '?' && *c != '&' && *c != '#' && *c != '/')
            .collect();
        if !video_id.is_empty() {
            return format!("https://www.youtube.com/embed/{}", video_id);
        }
    }

    // Vimeo standard link vimeo.com/12345678
    if trimmed.contains("vimeo.com/") && !trimmed.contains("player.vimeo.com") {
        if let Some(pos) = trimmed.find("vimeo.com/") {
            let after = &trimmed[pos + "vimeo.com/".len()..];
            let video_id: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            if !video_id.is_empty() {
                return format!("https://player.vimeo.com/video/{}", video_id);
            }
        }
    }

    trimmed.to_string()
}

fn parse_elementor_node(val: &Value) -> Option<NeopressNode> {
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
    let mut styles = extract_elementor_styles(&settings);

    let raw_children = obj
        .get("elements")
        .and_then(|e| e.as_array())
        .cloned()
        .unwrap_or_default();

    let children: Vec<NeopressNode> = raw_children
        .iter()
        .filter_map(parse_elementor_node)
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
            // Elementor 3.6+ Flexbox / Grid Container
            let container_type = settings
                .get("container_type")
                .and_then(|v| v.as_str())
                .unwrap_or("flex");

            if container_type == "grid" {
                styles.display = Some("grid".to_string());
            } else {
                styles.display = Some("flex".to_string());
                if let Some(dir) = settings
                    .get("flex_direction")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                {
                    styles.flex_direction = Some(dir.to_string());
                }
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

            if let Some(w_obj) = settings.get("width").and_then(|v| v.as_object()) {
                let unit = w_obj.get("unit").and_then(|u| u.as_str()).unwrap_or("%");
                if let Some(sz) = w_obj.get("size").and_then(extract_spacing_val) {
                    styles.width = Some(format!("{}{}", sz, unit));
                }
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
            map_elementor_widget(widget_type, &settings, &mut styles)?
        }
        _ => NodeKind::CustomHtml {
            raw: format!("<!-- Unrecognized Elementor type: {} -->", el_type),
        },
    };

    styles.sync_flat_and_nested();

    Some(NeopressNode {
        id,
        kind,
        styles,
        children,
    })
}

fn map_elementor_widget(
    widget_type: &str,
    settings: &Value,
    styles: &mut StyleProperties,
) -> Option<NodeKind> {
    match widget_type {
        "heading" => {
            let text = settings
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let level = settings
                .get("header_size")
                .and_then(|v| v.as_str())
                .and_then(|s| s.strip_prefix('h'))
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(2);

            if let Some(color) = settings
                .get("title_color")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                let typo = styles.typography.get_or_insert_with(Default::default);
                typo.color = Some(color.to_string());
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
            let content_html = settings
                .get("editor")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            Some(NodeKind::Text { content_html })
        }
        "image" => {
            let src = settings
                .get("image")
                .and_then(|img| img.get("url"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let alt = settings
                .get("image")
                .and_then(|img| img.get("alt"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let link = settings
                .get("link")
                .and_then(|lnk| lnk.get("url"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());

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

            let url = settings
                .get("link")
                .and_then(|lnk| lnk.get("url"))
                .and_then(|v| v.as_str())
                .unwrap_or("#")
                .to_string();

            let is_external = settings
                .get("link")
                .and_then(|lnk| lnk.get("is_external"))
                .and_then(|v| v.as_bool())
                .unwrap_or(false);

            let target = if is_external { "_blank" } else { "_self" }.to_string();

            if let Some(bg_col) = settings
                .get("button_background_color")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                let bg = styles.background.get_or_insert_with(Default::default);
                bg.color = Some(bg_col.to_string());
            }
            if let Some(txt_col) = settings
                .get("button_text_color")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                let typo = styles.typography.get_or_insert_with(Default::default);
                typo.color = Some(txt_col.to_string());
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
            let color = settings
                .get("primary_color")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
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
                    let content_html = tab
                        .get("tab_content")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    items.push(AccordionItem {
                        title,
                        content_html,
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
            let link = settings
                .get("link")
                .and_then(|lnk| lnk.get("url"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
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
                .unwrap_or_default()
                .to_string();
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
            let link = settings
                .get("link")
                .and_then(|lnk| lnk.get("url"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
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
                    let icon = it
                        .get("selected_icon")
                        .and_then(|i| i.get("value"))
                        .and_then(|v| v.as_str())
                        .or_else(|| it.get("icon").and_then(|v| v.as_str()))
                        .unwrap_or("fa-solid fa-check")
                        .to_string();
                    let link = it
                        .get("link")
                        .and_then(|lnk| lnk.get("url"))
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
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
                .map(|s| s.to_string());
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
                        content_html,
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
                        content_html,
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
                    let link = icon
                        .get("link")
                        .and_then(|lnk| lnk.get("url"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("#")
                        .to_string();
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
                .unwrap_or_default()
                .to_string();
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
                .unwrap_or_default()
                .to_string();
            Some(NodeKind::CustomHtml { raw })
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
                .get("form_name")
                .or_else(|| settings.get("title"))
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
            let button_url = settings
                .get("link")
                .and_then(|l| l.get("url"))
                .or_else(|| settings.get("button_url"))
                .and_then(|v| v.as_str())
                .unwrap_or("#")
                .to_string();
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
            let button_url = settings
                .get("link")
                .and_then(|l| l.get("url"))
                .or_else(|| settings.get("button_url"))
                .and_then(|v| v.as_str())
                .unwrap_or("#")
                .to_string();
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
                    let link = it
                        .get("link")
                        .and_then(|l| l.get("url"))
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
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
            let button_url = settings
                .get("link")
                .and_then(|l| l.get("url"))
                .and_then(|v| v.as_str())
                .unwrap_or("#")
                .to_string();
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
                    let button_url = first
                        .get("link")
                        .and_then(|l| l.get("url"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("#")
                        .to_string();
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
            if let Some(arr) = list {
                for item in arr {
                    if let Some(url) = item.get("url").and_then(|v| v.as_str()) {
                        let alt = item
                            .get("alt")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Gallery Image");
                        imgs_html.push_str(&format!(
                            "<img src=\"{}\" alt=\"{}\" class=\"np-gallery-img\" loading=\"lazy\" style=\"width:100%;height:auto;border-radius:6px;\" />",
                            url, alt
                        ));
                    }
                }
            }
            Some(NodeKind::CustomHtml {
                raw: format!(
                    "<div class=\"np-gallery\" style=\"display:grid;grid-template-columns:repeat(auto-fill,minmax(200px,1fr));gap:16px;\">{}</div>",
                    imgs_html
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
            let menu = settings
                .get("menu")
                .and_then(|v| v.as_str())
                .unwrap_or("primary");
            Some(NodeKind::CustomHtml {
                raw: format!("<nav class=\"np-nav-menu\" data-menu=\"{}\"></nav>", menu),
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
        // Fallback for custom or unrecognized Elementor addons
        other => {
            let raw = settings
                .get("raw")
                .or_else(|| settings.get("editor"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    format!("<!-- Elementor widget '{}' imported gracefully -->", other)
                });
            Some(NodeKind::CustomHtml { raw })
        }
    }
}

fn extract_global_color(globals: &Map<String, Value>, key: &str) -> Option<String> {
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

fn extract_elementor_styles(settings: &Value) -> StyleProperties {
    let mut styles = StyleProperties::default();
    let globals = settings.get("__globals__").and_then(|g| g.as_object());

    // Spacing: Padding
    if let Some(pad) = settings.get("padding").and_then(|v| v.as_object()) {
        let unit = pad.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
        let top = pad
            .get("top")
            .and_then(extract_spacing_val)
            .map(|v| format!("{}{}", v, unit));
        let right = pad
            .get("right")
            .and_then(extract_spacing_val)
            .map(|v| format!("{}{}", v, unit));
        let bottom = pad
            .get("bottom")
            .and_then(extract_spacing_val)
            .map(|v| format!("{}{}", v, unit));
        let left = pad
            .get("left")
            .and_then(extract_spacing_val)
            .map(|v| format!("{}{}", v, unit));

        if top.is_some() || right.is_some() || bottom.is_some() || left.is_some() {
            styles.padding = Some(Spacing {
                top,
                right,
                bottom,
                left,
            });
        }
    }

    // Spacing: Margin
    if let Some(mar) = settings.get("margin").and_then(|v| v.as_object()) {
        let unit = mar.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
        let top = mar
            .get("top")
            .and_then(extract_spacing_val)
            .map(|v| format!("{}{}", v, unit));
        let right = mar
            .get("right")
            .and_then(extract_spacing_val)
            .map(|v| format!("{}{}", v, unit));
        let bottom = mar
            .get("bottom")
            .and_then(extract_spacing_val)
            .map(|v| format!("{}{}", v, unit));
        let left = mar
            .get("left")
            .and_then(extract_spacing_val)
            .map(|v| format!("{}{}", v, unit));

        if top.is_some() || right.is_some() || bottom.is_some() || left.is_some() {
            styles.margin = Some(Spacing {
                top,
                right,
                bottom,
                left,
            });
        }
    }

    // Typography
    let font_size = settings.get("typography_font_size").and_then(|fs| {
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
        .and_then(extract_spacing_val);

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

    let text_color = settings
        .get("text_color")
        .or_else(|| settings.get("title_color"))
        .or_else(|| settings.get("color"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .or_else(|| {
            globals.and_then(|g| {
                extract_global_color(g, "text_color")
                    .or_else(|| extract_global_color(g, "title_color"))
                    .or_else(|| extract_global_color(g, "color"))
                    .or_else(|| extract_global_color(g, "button_text_color"))
            })
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

    // Background: NEVER use `background_background` ("classic" / "gradient") as a color!
    let bg_type = settings
        .get("background_background")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    let bg_color = settings
        .get("background_color")
        .and_then(|v| v.as_str())
        .filter(|s| {
            !s.is_empty()
                && !matches!(
                    *s,
                    "classic" | "gradient" | "video" | "slideshow" | "none"
                )
        })
        .map(|s| s.to_string())
        .or_else(|| {
            globals.and_then(|g| {
                extract_global_color(g, "background_color")
                    .or_else(|| extract_global_color(g, "button_background_color"))
            })
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

    let bg_img = settings
        .get("background_image")
        .and_then(|img| img.get("url"))
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let bg_pos = settings
        .get("background_position")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    let bg_size = settings
        .get("background_size")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

    if bg_color.is_some() || bg_img.is_some() || bg_pos.is_some() || bg_size.is_some() {
        styles.background = Some(BackgroundConfig {
            color: bg_color,
            image_url: bg_img,
            position: bg_pos,
            size: bg_size,
        });
    }

    // Border & Border Radius
    let border_radius = settings.get("border_radius").and_then(|b| {
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
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty() && *s != "none")
        .map(|s| s.to_string());

    let border_width = settings
        .get("border_width")
        .and_then(|w| w.as_object())
        .and_then(|obj| {
            let unit = obj.get("unit").and_then(|u| u.as_str()).unwrap_or("px");
            obj.get("top")
                .or_else(|| obj.get("size"))
                .and_then(extract_spacing_val)
                .map(|v| format!("{}{}", v, unit))
        });

    let border_color = settings
        .get("border_color")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());

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

fn extract_spacing_val(v: &Value) -> Option<String> {
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
