use crate::schema::{
    AccordionItem, BackgroundConfig, BorderConfig, IconListItem, NeopressLayout, NeopressNode,
    NodeKind, SocialIconItem, Spacing, StyleProperties, TabItem, ToggleItem, TypographyConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// Navigation menu item resolved during WXR or bundle transpilation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct NavMenuItem {
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub order: i64,
}

/// Site-wide resolution context for Elementor dynamic tags (`__dynamic__`),
/// Theme Builder widgets (`theme-site-logo`, `nav-menu`), and domain-relative URLs.
#[derive(Debug, Clone, Default)]
pub struct ElementorContext {
    pub post_urls: HashMap<String, String>,
    pub menus: HashMap<String, Vec<NavMenuItem>>,
    pub site_logo_url: Option<String>,
    pub source_site_url: Option<String>,
}

impl ElementorContext {
    pub fn from_meta(meta: &HashMap<String, String>) -> Self {
        let mut ctx = Self::default();
        if let Some(urls_json) = meta.get("__np_post_urls_json") {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(urls_json) {
                ctx.post_urls = map;
            }
        }
        if let Some(menus_json) = meta.get("__np_menus_json") {
            if let Ok(map) = serde_json::from_str::<HashMap<String, Vec<NavMenuItem>>>(menus_json) {
                ctx.menus = map;
            }
        }
        if let Some(logo) = meta.get("__np_site_logo_url").filter(|s| !s.is_empty()) {
            ctx.site_logo_url = Some(logo.clone());
        }
        if let Some(src) = meta.get("__np_source_site_url").filter(|s| !s.is_empty()) {
            ctx.source_site_url = Some(src.trim_end_matches('/').to_string());
        }
        ctx
    }

    /// Rewrites absolute source-site URLs (e.g. `https://example.com/wp-content/uploads/...`)
    /// to root-relative paths (`/wp-content/uploads/...`) so migrated sites are self-contained.
    pub fn normalize_url(&self, raw: &str) -> String {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return String::new();
        }
        if let Some(ref base) = self.source_site_url {
            let https_base = base.replacen("http://", "https://", 1);
            let http_base = base.replacen("https://", "http://", 1);
            for candidate in [&https_base, &http_base] {
                if trimmed == candidate.as_str() {
                    return "/".to_string();
                }
                let with_slash = format!("{}/", candidate);
                if let Some(rest) = trimmed.strip_prefix(&with_slash) {
                    return format!("/{}", rest);
                }
            }
        }
        // Also normalize any `/wp-content/uploads/` URL on the same domain even if source_site_url was not set
        if let Some(idx) = trimmed.find("/wp-content/uploads/") {
            if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                return trimmed[idx..].to_string();
            }
        }
        trimmed.to_string()
    }

    pub fn normalize_html(&self, raw_html: &str) -> String {
        if let Some(ref base) = self.source_site_url {
            let https_base = format!("{}/", base.replacen("http://", "https://", 1));
            let http_base = format!("{}/", base.replacen("https://", "http://", 1));
            return raw_html
                .replace(&https_base, "/")
                .replace(&http_base, "/");
        }
        raw_html.to_string()
    }
}

/// Parses raw Elementor JSON (`_elementor_data`) into a canonical NeopressLayout AST.
pub fn transpile_elementor_json(raw_json: &str) -> Result<NeopressLayout, String> {
    transpile_elementor_json_with_context(raw_json, &ElementorContext::default())
}

/// Parses raw Elementor JSON (`_elementor_data`) with full site context for dynamic tags and menus.
pub fn transpile_elementor_json_with_context(
    raw_json: &str,
    ctx: &ElementorContext,
) -> Result<NeopressLayout, String> {
    let parsed: Value = serde_json::from_str(raw_json)
        .map_err(|e| format!("Failed to parse Elementor JSON: {}", e))?;

    let root_children = if let Some(arr) = parsed.as_array() {
        arr.iter()
            .filter_map(|v| parse_elementor_node(v, ctx, 0))
            .collect()
    } else if let Some(obj) = parsed.as_object() {
        if let Some(elements) = obj.get("elements").and_then(|e| e.as_array()) {
            elements
                .iter()
                .filter_map(|v| parse_elementor_node(v, ctx, 0))
                .collect()
        } else {
            vec![parse_elementor_node(&parsed, ctx, 0).ok_or("Invalid Elementor node")?]
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

fn percent_decode(input: &str) -> String {
    let mut bytes = Vec::with_capacity(input.len());
    let raw = input.as_bytes();
    let mut i = 0;
    while i < raw.len() {
        if raw[i] == b'%' && i + 2 < raw.len() {
            if let (Some(h), Some(l)) = (
                (raw[i + 1] as char).to_digit(16),
                (raw[i + 2] as char).to_digit(16),
            ) {
                bytes.push(((h << 4) | l) as u8);
                i += 3;
                continue;
            }
        } else if raw[i] == b'+' {
            bytes.push(b' ');
            i += 1;
            continue;
        }
        bytes.push(raw[i]);
        i += 1;
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn extract_tag_attr<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
    let pattern = format!("{}=\"", attr);
    let start = tag.find(&pattern)? + pattern.len();
    let rest = &tag[start..];
    let end = rest.find('"')?;
    Some(&rest[..end])
}

/// Resolves an Elementor `[elementor-tag ...]` shortcode string into a concrete URL or value.
pub fn resolve_dynamic_tag(tag_str: &str, ctx: &ElementorContext) -> Option<String> {
    let name = extract_tag_attr(tag_str, "name")?;
    let settings_enc = extract_tag_attr(tag_str, "settings").unwrap_or("%7B%7D");
    let settings_dec = percent_decode(settings_enc);
    let settings: Value = serde_json::from_str(&settings_dec).unwrap_or(Value::Null);

    match name {
        "internal-url" => {
            let post_id = settings.get("post_id").and_then(extract_spacing_val)?;
            if let Some(url) = ctx.post_urls.get(&post_id) {
                Some(url.clone())
            } else {
                Some(format!("/?p={}", post_id))
            }
        }
        "contact-url" => {
            let link_type = settings
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("tel");
            match link_type {
                "email" => {
                    let mail = settings
                        .get("mail_to")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .trim();
                    if !mail.is_empty() {
                        Some(format!("mailto:{}", mail))
                    } else {
                        None
                    }
                }
                "whatsapp" => {
                    let tel = settings
                        .get("tel_number")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    let digits: String = tel.chars().filter(|c| c.is_ascii_digit()).collect();
                    if !digits.is_empty() {
                        Some(format!("https://wa.me/{}", digits))
                    } else {
                        None
                    }
                }
                _ => {
                    let tel = settings
                        .get("tel_number")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .trim();
                    if !tel.is_empty() {
                        Some(format!("tel:{}", tel))
                    } else {
                        None
                    }
                }
            }
        }
        "site-url" => Some("/".to_string()),
        "site-logo" => ctx.site_logo_url.clone(),
        "lightbox" => {
            let img_url = settings
                .get("image")
                .and_then(|img| img.get("url"))
                .and_then(|v| v.as_str())?;
            Some(ctx.normalize_url(img_url))
        }
        "popup" => {
            let popup_id = settings
                .get("popup")
                .and_then(extract_spacing_val)
                .unwrap_or_else(|| "1".to_string());
            Some(format!("#np-popup-{}", popup_id))
        }
        _ => None,
    }
}

fn resolve_setting_link(settings: &Value, key: &str, ctx: &ElementorContext) -> Option<String> {
    if let Some(dyn_str) = settings
        .get("__dynamic__")
        .and_then(|d| d.get(key))
        .and_then(|v| v.as_str())
    {
        if let Some(resolved) = resolve_dynamic_tag(dyn_str, ctx) {
            return Some(resolved);
        }
    }
    if let Some(val) = settings.get(key) {
        if let Some(u) = val
            .get("url")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
        {
            return Some(ctx.normalize_url(u));
        }
        if let Some(s) = val.as_str().filter(|s| !s.is_empty()) {
            return Some(ctx.normalize_url(s));
        }
    }
    None
}

fn parse_elementor_node(
    val: &Value,
    ctx: &ElementorContext,
    depth: usize,
) -> Option<NeopressNode> {
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
    let mut styles = extract_elementor_styles(&settings, ctx);

    let raw_children = obj
        .get("elements")
        .and_then(|e| e.as_array())
        .cloned()
        .unwrap_or_default();

    let mut children: Vec<NeopressNode> = raw_children
        .iter()
        .filter_map(|c| parse_elementor_node(c, ctx, depth + 1))
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
            let container_type = settings
                .get("container_type")
                .and_then(|v| v.as_str())
                .unwrap_or("flex");

            if container_type == "grid" {
                styles.display = Some("grid".to_string());
            } else {
                styles.display = Some("flex".to_string());
                let dir = settings
                    .get("flex_direction")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("column");
                styles.flex_direction = Some(dir.to_string());

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
                if let Some(fw) = settings
                    .get("flex_wrap")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                {
                    styles.flex_wrap = Some(fw.to_string());
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

            // Boxed top-level container support
            let content_width = settings
                .get("content_width")
                .and_then(|v| v.as_str())
                .unwrap_or("boxed");
            if depth == 0 && content_width != "full" {
                append_css_class(&mut styles, "np-container-boxed");
            }

            // Clickable container (`html_tag: "a"` or `__dynamic__.link`)
            if let Some(href) = resolve_setting_link(&settings, "link", ctx) {
                styles.link_url = Some(href);
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
            map_elementor_widget(widget_type, &settings, &mut styles, &mut children, ctx)?
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

fn append_css_class(styles: &mut StyleProperties, cls: &str) {
    match styles.css_classes {
        Some(ref mut existing) if !existing.is_empty() => {
            if !existing.split_whitespace().any(|c| c == cls) {
                existing.push(' ');
                existing.push_str(cls);
            }
        }
        _ => styles.css_classes = Some(cls.to_string()),
    }
}

fn escape_html_attr(val: &str) -> String {
    val.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn map_elementor_widget(
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

fn compile_elementor_form_fields(
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

        html.push_str(&format!("<div class=\"{}\">\n", col_class));
        if !label.is_empty() {
            html.push_str(&format!(
                "<label class=\"np-field-label\">{}{}</label>\n",
                escape_html_attr(label),
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

fn extract_global_typography(globals: &Map<String, Value>, key: &str) -> Option<String> {
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

fn parse_spacing_obj(obj: &Map<String, Value>) -> Option<Spacing> {
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

fn extract_elementor_styles(settings: &Value, ctx: &ElementorContext) -> StyleProperties {
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
                .filter(|s| !s.is_empty())
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
