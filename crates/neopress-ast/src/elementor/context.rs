#![allow(unused_imports)]
use crate::schema::{
    AccordionItem, BackgroundConfig, BorderConfig, IconListItem, NeopressLayout, NeopressNode,
    NodeKind, SocialIconItem, Spacing, StyleProperties, TabItem, ToggleItem, TypographyConfig,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;
use super::*;

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
        let mut out = if let Some(ref base) = self.source_site_url {
            let https_base = format!("{}/", base.replacen("http://", "https://", 1));
            let http_base = format!("{}/", base.replacen("https://", "http://", 1));
            raw_html
                .replace(&https_base, "/")
                .replace(&http_base, "/")
        } else {
            raw_html.to_string()
        };
        if out.contains("src=https://") {
            out = out.replace("src=https://", "src=\"https://");
        }
        if out.contains("src=http://") {
            out = out.replace("src=http://", "src=\"http://");
        }
        out
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

pub(crate) fn percent_decode(input: &str) -> String {
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

pub(crate) fn extract_tag_attr<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
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

pub(crate) fn resolve_setting_link(settings: &Value, key: &str, ctx: &ElementorContext) -> Option<String> {
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
