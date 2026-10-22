use crate::elementor::{normalize_video_embed_url, NavMenuItem};
use crate::schema::{
    AccordionItem, BackgroundConfig, BorderConfig, NeopressLayout, NeopressNode, NodeKind,
    SocialIconItem, Spacing, StyleProperties, TabItem, ToggleItem, TypographyConfig,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A service item from the `spb_service` custom post type used by the service price calculator.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SpbServiceItem {
    pub id: i64,
    pub title: String,
    pub price: f64,
    #[serde(default)]
    pub unit: String,
    #[serde(default)]
    pub is_extra: bool,
}

/// Site-wide resolution context for Divi dynamic attributes (`@ET-DC@...`),
/// global colors (`gcid-*`), menus (`et_pb_menu`), Contact Form 7 forms (`[contact-form-7]`,
/// `[dvppl_cf7_styler]`), and custom Divi modules (`[spb_kw_calculator_module]`).
#[derive(Debug, Clone, Default)]
pub struct DiviContext {
    pub post_urls: HashMap<String, String>,
    pub menus: HashMap<String, Vec<NavMenuItem>>,
    pub cf7_forms: HashMap<String, String>,
    pub spb_services: Vec<SpbServiceItem>,
    pub source_site_url: Option<String>,
}

impl DiviContext {
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
        if let Some(cf7_json) = meta.get("__np_cf7_forms_json") {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(cf7_json) {
                ctx.cf7_forms = map;
            }
        }
        if let Some(spb_json) = meta.get("__np_spb_services_json") {
            if let Ok(list) = serde_json::from_str::<Vec<SpbServiceItem>>(spb_json) {
                ctx.spb_services = list;
            }
        }
        if let Some(src) = meta.get("__np_source_site_url").filter(|s| !s.is_empty()) {
            ctx.source_site_url = Some(src.trim_end_matches('/').to_string());
        }
        ctx
    }

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
        // Expand any embedded [contact-form-7 ...] shortcodes inside Divi text modules
        if out.contains("[contact-form-7") && !self.cf7_forms.is_empty() {
            out = expand_cf7_shortcodes_in_html(&out, &self.cf7_forms);
        }
        out
    }

    /// Resolves Divi `@ET-DC@<base64>@` dynamic attributes and normalizes site URLs.
    pub fn resolve_divi_attr(&self, raw: &str) -> String {
        let trimmed = raw.trim();
        if let Some(inner) = trimmed
            .strip_prefix("@ET-DC@")
            .and_then(|s| s.strip_suffix('@'))
        {
            if let Some(decoded_bytes) = decode_base64_simple(inner) {
                if let Ok(json_str) = String::from_utf8(decoded_bytes) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&json_str) {
                        let content_type = val
                            .get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default();
                        match content_type {
                            "home_url" => return "/".to_string(),
                            "post_link_url_page"
                            | "post_link_url_post"
                            | "post_link_url"
                            | "custom_meta_link" => {
                                if let Some(pid) = val
                                    .get("settings")
                                    .and_then(|s| s.get("post_id"))
                                    .and_then(|p| {
                                        p.as_str()
                                            .map(|s| s.to_string())
                                            .or_else(|| p.as_i64().map(|n| n.to_string()))
                                    })
                                {
                                    if let Some(url) = self.post_urls.get(&pid) {
                                        return url.clone();
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        self.normalize_url(trimmed)
    }
}

fn decode_base64_simple(input: &str) -> Option<Vec<u8>> {
    let mut out = Vec::with_capacity(input.len() * 3 / 4);
    let mut buf: u32 = 0;
    let mut bits: u8 = 0;
    for b in input.bytes() {
        let val = match b {
            b'A'..=b'Z' => b - b'A',
            b'a'..=b'z' => b - b'a' + 26,
            b'0'..=b'9' => b - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'=' => break,
            b' ' | b'\n' | b'\r' | b'\t' => continue,
            _ => return None,
        };
        buf = (buf << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push(((buf >> bits) & 0xFF) as u8);
        }
    }
    Some(out)
}

fn expand_cf7_shortcodes_in_html(html: &str, cf7_forms: &HashMap<String, String>) -> String {
    let re = Regex::new(r"\[contact-form-7([^\]]*)\]").unwrap();
    re.replace_all(html, |caps: &regex::Captures| {
        let attr_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let attrs = parse_attributes(attr_str);
        if let Some(id_val) = attrs.get("id") {
            if let Some(form_html) = cf7_forms.get(id_val) {
                return form_html.clone();
            }
            let prefix: String = id_val.chars().take(7).collect();
            if let Some(form_html) = cf7_forms.get(&prefix) {
                return form_html.clone();
            }
        }
        if let Some(title_val) = attrs.get("title") {
            if let Some(form_html) = cf7_forms.get(&title_val.to_lowercase()) {
                return form_html.clone();
            }
        }
        if let Some((_, first_form)) = cf7_forms.iter().next() {
            return first_form.clone();
        }
        caps.get(0).unwrap().as_str().to_string()
    })
    .to_string()
}

/// Resolves Divi Global Color IDs (`gcid-*`) into CSS hex/var colors.
pub fn resolve_divi_color(raw: &str) -> String {
    let trimmed = raw.trim();
    match trimmed {
        "gcid-primary-color" => "#e23212".to_string(),
        "gcid-secondary-color" => "#101523".to_string(),
        "gcid-b0c0bd74-c2d9-4a33-8fc8-1f11eab87211" => "#fcfbfc".to_string(),
        "gcid-f5b5c1a3-263d-4635-a170-86276049a28b" => "#101523".to_string(),
        "gcid-heading-color" | "gcid-body-color" => "#101523".to_string(),
        other if other.starts_with("gcid-") => "#e23212".to_string(),
        other => other.to_string(),
    }
}

/// Decodes Divi's URL-encoded `global_colors_info` attribute (e.g. `{%22gcid-primary-color%22:%91%22icon_color%22%93}`)
/// and populates any missing color attributes on the shortcode map.
fn apply_divi_global_colors_info(attrs: &mut HashMap<String, String>) {
    let Some(raw_info) = attrs.get("global_colors_info").cloned() else {
        return;
    };
    if raw_info.is_empty() || raw_info == "{}" {
        return;
    }
    let decoded = raw_info
        .replace("%22", "\"")
        .replace("%91", "[")
        .replace("%93", "]")
        .replace("%5B", "[")
        .replace("%5D", "]")
        .replace("%7B", "{")
        .replace("%7D", "}");
    if let Ok(map) = serde_json::from_str::<HashMap<String, Vec<String>>>(&decoded) {
        for (gcid, props) in map {
            let resolved = resolve_divi_color(&gcid);
            for prop in props {
                attrs.entry(prop).or_insert_with(|| resolved.clone());
            }
        }
    }
}

/// A parsed Divi shortcode element.
#[derive(Debug, Clone)]
pub struct DiviShortcode {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub inner_text: String,
    pub children: Vec<DiviShortcode>,
}

/// Parses raw Divi shortcode markup into a canonical NeopressLayout AST.
pub fn transpile_divi_content(content: &str) -> Result<NeopressLayout, String> {
    transpile_divi_content_with_context(content, &DiviContext::default())
}

/// Parses raw Divi shortcode markup with site-wide context into a canonical NeopressLayout AST.
pub fn transpile_divi_content_with_context(
    content: &str,
    ctx: &DiviContext,
) -> Result<NeopressLayout, String> {
    let cleaned = content
        .replace("<!-- wp:divi/placeholder -->", "")
        .replace("<!-- /wp:divi/placeholder -->", "")
        .replace("<!-- wp:divi/placeholder /-->", "");

    let tokens = tokenize_divi(&cleaned)?;
    let tree = parse_divi_tree(tokens)?;

    let mut id_counter: usize = 0;
    let root_children: Vec<NeopressNode> = tree
        .iter()
        .map(|node| map_divi_node(node, ctx, &mut id_counter))
        .collect();

    Ok(NeopressLayout {
        version: 1,
        root: NeopressNode {
            id: "divi_root".to_string(),
            kind: NodeKind::Section,
            styles: StyleProperties::default(),
            children: root_children,
        },
        global_styles: None,
    })
}

#[derive(Debug, PartialEq)]
enum Token {
    OpenTag {
        name: String,
        attributes: HashMap<String, String>,
    },
    CloseTag {
        name: String,
    },
    Text(String),
}

fn is_divi_leaf_tag(tag: &str) -> bool {
    matches!(
        tag,
        "et_pb_divider"
            | "et_pb_button"
            | "et_pb_image"
            | "et_pb_space"
            | "et_pb_video"
            | "et_pb_map"
            | "et_pb_fullwidth_image"
            | "et_pb_contact_field"
            | "et_pb_icon"
            | "et_pb_menu"
            | "et_pb_fullwidth_menu"
            | "dvppl_cf7_styler"
            | "spb_kw_calculator_module"
            | "spb_kitwerk_calculator"
    )
}

fn is_divi_container_tag(tag: &str) -> bool {
    matches!(
        tag,
        "et_pb_section"
            | "et_pb_fullwidth_section"
            | "et_pb_specialty_section"
            | "et_pb_row"
            | "et_pb_row_inner"
            | "et_pb_column"
            | "et_pb_column_inner"
            | "et_pb_accordion"
            | "et_pb_tabs"
            | "et_pb_counters"
            | "et_pb_pricing_tables"
            | "et_pb_contact_form"
            | "et_pb_social_media_follow"
    )
}

fn tokenize_divi(input: &str) -> Result<Vec<Token>, String> {
    let tag_re = Regex::new(r"\[(/?)((?:et_pb_|spb_|dvppl_)[a-z0-9_]+)([^\]]*)\]").unwrap();
    let mut tokens = Vec::new();
    let mut last_idx = 0;

    for caps in tag_re.captures_iter(input) {
        let full_match = caps.get(0).unwrap();
        let match_start = full_match.start();
        let match_end = full_match.end();

        if match_start > last_idx {
            let text = &input[last_idx..match_start];
            if !text.trim().is_empty() {
                tokens.push(Token::Text(text.to_string()));
            }
        }

        let is_close = caps.get(1).map(|m| m.as_str() == "/").unwrap_or(false);
        let tag_name = caps.get(2).unwrap().as_str().to_string();

        if is_close {
            tokens.push(Token::CloseTag { name: tag_name });
        } else {
            let raw_attrs = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let is_self_closing = raw_attrs.trim_end().ends_with('/');
            let cleaned_attrs = raw_attrs.trim_end().trim_end_matches('/');
            let mut attributes = parse_attributes(cleaned_attrs);
            apply_divi_global_colors_info(&mut attributes);
            tokens.push(Token::OpenTag {
                name: tag_name.clone(),
                attributes,
            });
            if is_self_closing {
                tokens.push(Token::CloseTag { name: tag_name });
            }
        }

        last_idx = match_end;
    }

    if last_idx < input.len() {
        let text = &input[last_idx..];
        if !text.trim().is_empty() {
            tokens.push(Token::Text(text.to_string()));
        }
    }

    Ok(tokens)
}

fn parse_attributes(raw: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    let attr_re = Regex::new(r#"([a-zA-Z0-9_\-]+)=(?:["']([^"']*)["']|([^\s>]+))"#).unwrap();

    for caps in attr_re.captures_iter(raw) {
        let key = caps.get(1).unwrap().as_str().to_string();
        let val = caps
            .get(2)
            .or_else(|| caps.get(3))
            .map(|m| m.as_str())
            .unwrap_or("")
            .to_string();
        attrs.insert(key, val);
    }

    attrs
}

fn pop_top_to_parent(stack: &mut Vec<DiviShortcode>, root_nodes: &mut Vec<DiviShortcode>) {
    if let Some(closed) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(closed);
        } else {
            root_nodes.push(closed);
        }
    }
}

fn parse_divi_tree(tokens: Vec<Token>) -> Result<Vec<DiviShortcode>, String> {
    let mut root_nodes: Vec<DiviShortcode> = Vec::new();
    let mut stack: Vec<DiviShortcode> = Vec::new();

    for token in tokens {
        match token {
            Token::OpenTag { name, attributes } => {
                while let Some(top) = stack.last() {
                    if is_divi_leaf_tag(&top.tag)
                        || (!is_divi_container_tag(&top.tag) && top.tag == name)
                    {
                        pop_top_to_parent(&mut stack, &mut root_nodes);
                    } else {
                        break;
                    }
                }

                let node = DiviShortcode {
                    tag: name,
                    attributes,
                    inner_text: String::new(),
                    children: Vec::new(),
                };
                stack.push(node);
            }
            Token::CloseTag { name } => {
                while let Some(top) = stack.last() {
                    if top.tag != name && is_divi_leaf_tag(&top.tag) {
                        pop_top_to_parent(&mut stack, &mut root_nodes);
                    } else {
                        break;
                    }
                }

                if let Some(pos) = stack.iter().rposition(|n| n.tag == name) {
                    while stack.len() > pos + 1 {
                        pop_top_to_parent(&mut stack, &mut root_nodes);
                    }
                    pop_top_to_parent(&mut stack, &mut root_nodes);
                }
            }
            Token::Text(t) => {
                if let Some(top) = stack.last() {
                    if matches!(
                        top.tag.as_str(),
                        "et_pb_divider"
                            | "et_pb_space"
                            | "et_pb_image"
                            | "et_pb_fullwidth_image"
                            | "et_pb_icon"
                            | "et_pb_menu"
                            | "dvppl_cf7_styler"
                            | "spb_kw_calculator_module"
                            | "spb_kitwerk_calculator"
                    ) {
                        pop_top_to_parent(&mut stack, &mut root_nodes);
                    }
                }
                if let Some(top) = stack.last_mut() {
                    top.inner_text.push_str(&t);
                }
            }
        }
    }

    while !stack.is_empty() {
        pop_top_to_parent(&mut stack, &mut root_nodes);
    }

    Ok(root_nodes)
}

fn map_divi_node(divi: &DiviShortcode, ctx: &DiviContext, counter: &mut usize) -> NeopressNode {
    *counter += 1;
    let short_name = divi.tag.strip_prefix("et_pb_").unwrap_or(&divi.tag);
    let id = divi
        .attributes
        .get("module_id")
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| format!("divi_{}_{}", short_name, *counter));
    let mut styles = extract_divi_styles(&divi.attributes, ctx);

    let mut children: Vec<NeopressNode> = divi
        .children
        .iter()
        .map(|c| map_divi_node(c, ctx, counter))
        .collect();

    let kind = match divi.tag.as_str() {
        "et_pb_section" | "et_pb_fullwidth_section" | "et_pb_specialty_section" => {
            // If specialty section has direct column children, wrap them in a centered flex row container
            let has_direct_columns = !children.is_empty()
                && children
                    .iter()
                    .all(|c| matches!(c.kind, NodeKind::Flex { basis: Some(_), .. } | NodeKind::Column));
            if has_direct_columns {
                let mut row_styles = StyleProperties::default();
                row_styles.width = Some("90%".to_string());
                row_styles.max_width = Some("1200px".to_string());
                row_styles.margin = Some(Spacing {
                    top: Some("0px".to_string()),
                    right: Some("auto".to_string()),
                    bottom: Some("0px".to_string()),
                    left: Some("auto".to_string()),
                });
                row_styles.display = Some("flex".to_string());
                row_styles.flex_direction = Some("row".to_string());
                row_styles.gap = Some("28px".to_string());
                row_styles.align_items = Some("flex-start".to_string());
                row_styles.sync_flat_and_nested();

                let wrapped_cols = std::mem::take(&mut children);
                children.push(NeopressNode {
                    id: format!("{}_specialty_row", id),
                    kind: NodeKind::Container,
                    styles: row_styles,
                    children: wrapped_cols,
                });
            }

            // Inject background video if background_video_mp4 or background_video_webm is set
            let video_mp4 = divi
                .attributes
                .get("background_video_mp4")
                .or_else(|| divi.attributes.get("background_video_webm"))
                .filter(|s| !s.trim().is_empty())
                .map(|u| ctx.normalize_url(u));
            let video_enabled = divi
                .attributes
                .get("background_enable_video_mp4")
                .map(|v| v != "off")
                .unwrap_or(true);

            if let Some(vid_url) = video_mp4.filter(|_| video_enabled) {
                styles.custom_css = Some("position: relative; overflow: hidden;".to_string());
                for child in &mut children {
                    child.styles.custom_css = Some("position: relative; z-index: 2;".to_string());
                    child.styles.sync_flat_and_nested();
                }
                let video_bg_html = format!(
                    "<div class=\"np-section-video-bg\" style=\"position:absolute;inset:0;z-index:0;pointer-events:none;overflow:hidden;\"><video autoplay loop muted playsinline style=\"width:100%;height:100%;object-fit:cover;\"><source src=\"{}\" type=\"video/mp4\" /></video></div>",
                    escape_html(&vid_url)
                );
                children.insert(
                    0,
                    NeopressNode {
                        id: format!("{}_video_bg", id),
                        kind: NodeKind::CustomHtml { raw: video_bg_html },
                        styles: StyleProperties::default(),
                        children: Vec::new(),
                    },
                );
            }

            // Handle Divi Area Popup sections (da_is_popup="on" or module_class contains "popup")
            let is_popup = divi
                .attributes
                .get("da_is_popup")
                .map(|v| v == "on")
                .unwrap_or(false)
                || styles
                    .css_classes
                    .as_deref()
                    .map(|c| c.split_whitespace().any(|w| w == "popup"))
                    .unwrap_or(false);
            if is_popup {
                let mut cls = styles.css_classes.clone().unwrap_or_default();
                if !cls.contains("et_pb_section") {
                    cls = format!("et_pb_section popup {}", cls).trim().to_string();
                }
                styles.css_classes = Some(cls);
                let close_btn_html = "<button type=\"button\" class=\"da-close np-popup-close\" aria-label=\"Sluiten\" onclick=\"var p=this.closest('.popup');if(p)p.classList.remove('is-open');\">&times;</button>".to_string();
                children.insert(
                    0,
                    NeopressNode {
                        id: format!("{}_popup_close", id),
                        kind: NodeKind::CustomHtml {
                            raw: close_btn_html,
                        },
                        styles: StyleProperties::default(),
                        children: Vec::new(),
                    },
                );
            }

            NodeKind::Section
        }
        "et_pb_row" | "et_pb_row_inner" => {
            if divi.tag == "et_pb_row" {
                if styles.width.is_none() {
                    styles.width = Some("90%".to_string());
                }
                if styles.max_width.is_none() {
                    styles.max_width = Some("1200px".to_string());
                }
                let mut mar = styles.margin.clone().unwrap_or_default();
                if mar.left.is_none() {
                    mar.left = Some("auto".to_string());
                }
                if mar.right.is_none() {
                    mar.right = Some("auto".to_string());
                }
                styles.margin = Some(mar);
            }
            NodeKind::Container
        }
        "et_pb_column" | "et_pb_column_inner" => {
            let col_type = divi
                .attributes
                .get("type")
                .map(|s| s.as_str())
                .unwrap_or("4_4");
            let basis = match col_type {
                "4_4" => "100%",
                "1_2" => "50%",
                "1_3" => "33.33%",
                "2_3" => "66.66%",
                "1_4" => "25%",
                "3_4" => "75%",
                "1_5" => "20%",
                "2_5" => "40%",
                "3_5" => "60%",
                "4_5" => "80%",
                "1_6" => "16.66%",
                other => other,
            };
            NodeKind::Flex {
                direction: "column".to_string(),
                justify: "flex-start".to_string(),
                align: "stretch".to_string(),
                wrap: false,
                basis: Some(basis.to_string()),
            }
        }
        "et_pb_text" => {
            let raw_html = if !divi.inner_text.trim().is_empty() {
                divi.inner_text.clone()
            } else {
                divi.attributes.get("content").cloned().unwrap_or_default()
            };
            let content_html = ctx.normalize_html(&raw_html);
            NodeKind::Text { content_html }
        }
        "et_pb_heading" | "et_pb_title" | "et_pb_post_title" => {
            let text = divi
                .attributes
                .get("title")
                .cloned()
                .unwrap_or_else(|| divi.inner_text.clone());
            let level = divi
                .attributes
                .get("title_level")
                .or_else(|| divi.attributes.get("header_level"))
                .and_then(|s| s.strip_prefix('h'))
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(2);
            NodeKind::Heading { text, level }
        }
        "et_pb_image" | "et_pb_fullwidth_image" => {
            let raw_src = divi.attributes.get("src").cloned().unwrap_or_default();
            let src = ctx.resolve_divi_attr(&raw_src);
            let alt = divi
                .attributes
                .get("alt")
                .or_else(|| divi.attributes.get("title_text"))
                .cloned()
                .unwrap_or_default();
            let link = divi
                .attributes
                .get("url")
                .filter(|s| !s.trim().is_empty())
                .map(|u| ctx.resolve_divi_attr(u));
            NodeKind::Image {
                src,
                alt,
                width: None,
                height: None,
                link,
            }
        }
        "et_pb_button" => {
            let text = divi
                .attributes
                .get("button_text")
                .cloned()
                .unwrap_or_else(|| {
                    if !divi.inner_text.trim().is_empty() {
                        divi.inner_text.trim().to_string()
                    } else {
                        "Click Here".to_string()
                    }
                });
            let url = divi
                .attributes
                .get("button_url")
                .filter(|s| !s.trim().is_empty())
                .map(|u| ctx.resolve_divi_attr(u))
                .unwrap_or_else(|| "#".to_string());
            let is_external = divi
                .attributes
                .get("url_new_window")
                .map(|s| s == "on")
                .unwrap_or(false);
            let target = if is_external { "_blank" } else { "_self" }.to_string();

            if let Some(btn_bg) = divi
                .attributes
                .get("button_bg_color")
                .filter(|s| !s.is_empty())
            {
                let resolved_bg = resolve_divi_color(btn_bg);
                styles.background_color = Some(resolved_bg.clone());
                let mut bg = styles.background.clone().unwrap_or_default();
                bg.color = Some(resolved_bg);
                styles.background = Some(bg);
            }
            if let Some(btn_fg) = divi
                .attributes
                .get("button_text_color")
                .filter(|s| !s.is_empty())
            {
                let resolved_fg = resolve_divi_color(btn_fg);
                styles.text_color = Some(resolved_fg.clone());
                let mut typo = styles.typography.clone().unwrap_or_default();
                typo.color = Some(resolved_fg);
                styles.typography = Some(typo);
            }
            if let Some(btn_sz) = divi
                .attributes
                .get("button_text_size")
                .filter(|s| !s.is_empty())
            {
                styles.font_size = Some(btn_sz.clone());
                let mut typo = styles.typography.clone().unwrap_or_default();
                typo.font_size = Some(btn_sz.clone());
                styles.typography = Some(typo);
            }
            if let Some(align) = divi
                .attributes
                .get("button_alignment")
                .filter(|s| !s.is_empty())
            {
                match align.as_str() {
                    "center" => styles.custom_css = Some("align-self: center;".to_string()),
                    "right" => styles.custom_css = Some("align-self: flex-end;".to_string()),
                    "left" => styles.custom_css = Some("align-self: flex-start;".to_string()),
                    _ => {}
                }
            }

            NodeKind::Button { text, url, target }
        }
        "et_pb_blurb" => {
            let title = divi
                .attributes
                .get("title")
                .cloned()
                .unwrap_or_else(|| "Feature".to_string());
            let description = ctx.normalize_html(divi.inner_text.trim());
            let link = divi
                .attributes
                .get("url")
                .filter(|s| !s.trim().is_empty())
                .map(|u| ctx.resolve_divi_attr(u));
            let use_icon = divi
                .attributes
                .get("use_icon")
                .map(|s| s == "on")
                .unwrap_or(false);

            if !use_icon {
                if let Some(img_url) = divi.attributes.get("image").filter(|s| !s.is_empty()) {
                    return NeopressNode {
                        id,
                        kind: NodeKind::ImageBox {
                            image_url: ctx.resolve_divi_attr(img_url),
                            title,
                            description,
                            link,
                        },
                        styles,
                        children: Vec::new(),
                    };
                }
            }

            let icon = divi
                .attributes
                .get("font_icon")
                .map(|ic| map_divi_icon(ic))
                .unwrap_or_else(|| "fa-solid fa-bolt".to_string());

            NodeKind::IconBox {
                icon,
                title,
                description,
                link,
            }
        }
        "et_pb_icon" => {
            let icon = divi
                .attributes
                .get("font_icon")
                .map(|ic| map_divi_icon(ic))
                .unwrap_or_else(|| "fa-solid fa-chevron-down".to_string());
            let size = divi
                .attributes
                .get("icon_width")
                .and_then(|w| {
                    let digits: String = w.chars().take_while(|c| c.is_ascii_digit()).collect();
                    digits.parse::<u32>().ok()
                })
                .or(Some(36));
            let color = divi
                .attributes
                .get("icon_color")
                .filter(|s| !s.is_empty())
                .map(|c| resolve_divi_color(c));
            if styles.text_align.is_none() {
                styles.text_align = Some("center".to_string());
            }
            NodeKind::Icon {
                name: icon,
                size,
                color,
            }
        }
        "et_pb_fullwidth_header" => {
            let title = divi.attributes.get("title").cloned().unwrap_or_default();
            let subhead = divi.attributes.get("subhead").cloned().unwrap_or_default();
            let description = if !divi.inner_text.trim().is_empty() {
                ctx.normalize_html(divi.inner_text.trim())
            } else {
                subhead
            };
            let button_text = divi
                .attributes
                .get("button_text")
                .or_else(|| divi.attributes.get("button_one_text"))
                .cloned()
                .unwrap_or_default();
            let button_url = divi
                .attributes
                .get("button_url")
                .or_else(|| divi.attributes.get("button_one_url"))
                .map(|u| ctx.resolve_divi_attr(u))
                .unwrap_or_else(|| "#".to_string());

            let fullscreen = divi
                .attributes
                .get("header_fullscreen")
                .map(|v| v == "on")
                .unwrap_or(false);
            let scroll_down = divi
                .attributes
                .get("header_scroll_down")
                .map(|v| v == "on")
                .unwrap_or(false);
            let scroll_color = divi
                .attributes
                .get("scroll_down_icon_color")
                .map(|c| resolve_divi_color(c))
                .unwrap_or_else(|| "#ffffff".to_string());
            let overlay_color = divi
                .attributes
                .get("background_overlay_color")
                .filter(|s| !s.is_empty())
                .map(|c| resolve_divi_color(c));
            let video_mp4 = divi
                .attributes
                .get("background_video_mp4")
                .filter(|s| !s.is_empty())
                .map(|u| ctx.normalize_url(u));
            let logo_url = divi
                .attributes
                .get("logo_image_url")
                .filter(|s| !s.is_empty())
                .map(|u| ctx.normalize_url(u));
            let title_color = divi
                .attributes
                .get("title_text_color")
                .map(|c| resolve_divi_color(c))
                .unwrap_or_else(|| "#ffffff".to_string());
            let content_color = divi
                .attributes
                .get("content_text_color")
                .or_else(|| divi.attributes.get("subhead_text_color"))
                .map(|c| resolve_divi_color(c))
                .unwrap_or_else(|| title_color.clone());
            let title_size = divi
                .attributes
                .get("title_font_size")
                .cloned()
                .unwrap_or_else(|| "46px".to_string());

            // If simple header with no video, no logo, and no fullscreen, use CallToAction
            if video_mp4.is_none() && logo_url.is_none() && !fullscreen && !scroll_down {
                NodeKind::CallToAction {
                    title,
                    description,
                    button_text,
                    button_url,
                }
            } else {
                let min_h = if fullscreen { "min-height: 100vh;" } else { "min-height: 480px;" };
                let bg_col = styles
                    .background_color
                    .clone()
                    .unwrap_or_else(|| "#3D2B01".to_string());
                let video_html = video_mp4
                    .map(|v| {
                        format!(
                            "<video autoplay loop muted playsinline style=\"position:absolute;inset:0;width:100%;height:100%;object-fit:cover;z-index:0;pointer-events:none;\"><source src=\"{}\" type=\"video/mp4\" /></video>",
                            escape_html(&v)
                        )
                    })
                    .unwrap_or_default();
                let overlay_html = overlay_color
                    .map(|ov| {
                        format!(
                            "<div class=\"et_pb_fullwidth_header_overlay\" style=\"position:absolute;inset:0;background:{};z-index:1;pointer-events:none;\"></div>",
                            escape_html(&ov)
                        )
                    })
                    .unwrap_or_default();
                let logo_html = logo_url
                    .map(|lg| {
                        format!(
                            "<img src=\"{}\" alt=\"{}\" style=\"max-width:260px;height:auto;margin:0 auto 24px auto;display:block;\" />",
                            escape_html(&lg),
                            escape_html(&title)
                        )
                    })
                    .unwrap_or_default();
                let btn_html = if !button_text.trim().is_empty() {
                    format!(
                        "<div style=\"margin-top:24px;\"><a href=\"{}\" class=\"np-button et_pb_button\">{}</a></div>",
                        escape_html(&button_url),
                        escape_html(&button_text)
                    )
                } else {
                    String::new()
                };
                let scroll_html = if scroll_down {
                    format!(
                        "<a href=\"#\" onclick=\"window.scrollBy({{top:window.innerHeight*0.85,behavior:'smooth'}});return false;\" class=\"et_pb_fullwidth_header_scroll\" style=\"position:absolute;bottom:32px;left:50%;transform:translateX(-50%);z-index:2;color:{};font-size:32px;text-decoration:none;\"><i class=\"fa-solid fa-chevron-down\"></i></a>",
                        escape_html(&scroll_color)
                    )
                } else {
                    String::new()
                };

                let raw_header = format!(
                    "<div class=\"et_pb_module et_pb_fullwidth_header\" style=\"position:relative;overflow:hidden;display:flex;align-items:center;justify-content:center;text-align:center;padding:80px 24px;background-color:{};{}\">{}{}<div class=\"et_pb_fullwidth_header_container\" style=\"position:relative;z-index:2;max-width:900px;margin:0 auto;color:{};\">{}\n<h1 class=\"et_pb_module_header\" style=\"color:{};font-size:{};margin-bottom:20px;line-height:1.15;\">{}</h1>\n<div class=\"et_pb_header_content_wrapper\" style=\"color:{};font-size:18px;line-height:1.6;\">{}</div>{}</div>{}</div>",
                    escape_html(&bg_col),
                    min_h,
                    video_html,
                    overlay_html,
                    escape_html(&content_color),
                    logo_html,
                    escape_html(&title_color),
                    escape_html(&title_size),
                    escape_html(&title),
                    escape_html(&content_color),
                    description,
                    btn_html,
                    scroll_html
                );
                styles = StyleProperties::default();
                NodeKind::CustomHtml { raw: raw_header }
            }
        }
        "et_pb_cta" => {
            let title = divi.attributes.get("title").cloned().unwrap_or_default();
            let subhead = divi.attributes.get("subhead").cloned().unwrap_or_default();
            let description = if !divi.inner_text.trim().is_empty() {
                ctx.normalize_html(divi.inner_text.trim())
            } else {
                subhead
            };
            let button_text = divi
                .attributes
                .get("button_text")
                .or_else(|| divi.attributes.get("button_one_text"))
                .cloned()
                .unwrap_or_default();
            let button_url = divi
                .attributes
                .get("button_url")
                .or_else(|| divi.attributes.get("link_option_url"))
                .or_else(|| divi.attributes.get("button_one_url"))
                .map(|u| ctx.resolve_divi_attr(u))
                .unwrap_or_else(|| "#".to_string());

            NodeKind::CallToAction {
                title,
                description,
                button_text,
                button_url,
            }
        }
        "et_pb_menu" | "et_pb_fullwidth_menu" => {
            let menu_key = divi
                .attributes
                .get("menu_id")
                .filter(|s| !s.is_empty())
                .map(|s| s.as_str())
                .unwrap_or("primary");
            let items = ctx
                .menus
                .get(menu_key)
                .or_else(|| ctx.menus.get("primary"))
                .or_else(|| ctx.menus.values().next())
                .cloned()
                .unwrap_or_default();

            let is_vert = divi
                .attributes
                .get("module_class")
                .map(|c| c.contains("et-vert-menu"))
                .unwrap_or(false);
            let align = divi
                .attributes
                .get("text_orientation")
                .map(|s| s.as_str())
                .unwrap_or("left");
            let justify = match align {
                "right" => "flex-end",
                "center" => "center",
                _ => "flex-start",
            };
            let text_color = divi
                .attributes
                .get("menu_text_color")
                .map(|c| resolve_divi_color(c))
                .unwrap_or_else(|| "#FAD685".to_string());
            let font_size = divi
                .attributes
                .get("menu_font_size")
                .cloned()
                .unwrap_or_else(|| "17px".to_string());
            let line_height = divi
                .attributes
                .get("menu_line_height")
                .cloned()
                .unwrap_or_else(|| "1.5em".to_string());

            let logo_html = divi
                .attributes
                .get("logo")
                .filter(|s| !s.trim().is_empty())
                .map(|lg| {
                    let lg_src = ctx.resolve_divi_attr(lg);
                    let lg_href = divi
                        .attributes
                        .get("logo_url")
                        .map(|u| ctx.resolve_divi_attr(u))
                        .unwrap_or_else(|| "/".to_string());
                    format!(
                        "<div class=\"et_pb_menu__logo-wrap\"><a href=\"{}\"><img src=\"{}\" alt=\"Logo\" style=\"max-height:60px;width:auto;\" /></a></div>",
                        escape_html(&lg_href),
                        escape_html(&lg_src)
                    )
                })
                .unwrap_or_default();

            let flex_dir = if is_vert { "column" } else { "row" };
            let gap = if is_vert { "12px" } else { "28px" };
            let underline = if is_vert {
                "text-decoration:underline;text-decoration-color:#e23212;"
            } else {
                "text-decoration:none;"
            };

            let mut list_html = format!(
                "<ul class=\"np-nav-menu-list et-menu nav\" style=\"display:flex;flex-direction:{};justify-content:{};gap:{};list-style:none;margin:0;padding:0;flex-wrap:wrap;\">",
                flex_dir, justify, gap
            );
            for item in items {
                list_html.push_str(&format!(
                    "<li class=\"np-nav-menu-item menu-item\"><a href=\"{}\" class=\"np-nav-menu-link\" style=\"color:{};font-size:{};line-height:{};{}font-weight:500;\">{}</a></li>",
                    escape_html(&item.url),
                    escape_html(&text_color),
                    escape_html(&font_size),
                    escape_html(&line_height),
                    underline,
                    escape_html(&item.title)
                ));
            }
            list_html.push_str("</ul>");

            let mod_cls = divi
                .attributes
                .get("module_class")
                .cloned()
                .unwrap_or_default();
            let nav_html = format!(
                "<div class=\"et_pb_module et_pb_menu {}\" style=\"display:flex;align-items:center;justify-content:{};gap:24px;\">{}\n<div class=\"et_pb_menu__menu\"><nav class=\"et-menu-nav np-nav-menu\">{}</nav></div></div>",
                escape_html(&mod_cls),
                justify,
                logo_html,
                list_html
            );
            NodeKind::CustomHtml { raw: nav_html }
        }
        "et_pb_divider" => {
            let style = divi
                .attributes
                .get("divider_style")
                .cloned()
                .unwrap_or_else(|| "solid".to_string());
            if let Some(col) = divi.attributes.get("color").filter(|s| !s.is_empty()) {
                let bdr = styles.border.get_or_insert_with(Default::default);
                bdr.color = Some(resolve_divi_color(col));
            }
            NodeKind::Divider { style }
        }
        "et_pb_space" => {
            let height = divi
                .attributes
                .get("height")
                .and_then(|h| {
                    let digits: String = h.chars().take_while(|c| c.is_ascii_digit()).collect();
                    digits.parse::<u32>().ok()
                })
                .unwrap_or(40);
            NodeKind::Spacer { height }
        }
        "et_pb_code" | "et_pb_fullwidth_code" => NodeKind::CustomHtml {
            raw: ctx.normalize_html(&divi.inner_text),
        },
        "et_pb_counters" => NodeKind::Container,
        "et_pb_counter" => {
            let percent = divi
                .attributes
                .get("percent")
                .and_then(|p| p.trim_end_matches('%').parse::<u8>().ok())
                .unwrap_or(100);
            let title = if !divi.inner_text.trim().is_empty() {
                divi.inner_text.trim().to_string()
            } else {
                divi.attributes
                    .get("title")
                    .cloned()
                    .unwrap_or_else(|| "Progress".to_string())
            };
            NodeKind::ProgressBar {
                title,
                percent,
                display_text: Some(format!("{}%", percent)),
            }
        }
        "et_pb_number_counter" | "et_pb_circle_counter" => {
            let number = divi
                .attributes
                .get("number")
                .or_else(|| divi.attributes.get("percent"))
                .and_then(|p| p.trim_end_matches('%').parse::<i64>().ok())
                .unwrap_or(100);
            let percent_sign = divi
                .attributes
                .get("percent_sign")
                .map(|s| s == "on")
                .unwrap_or(true);
            let suffix = if percent_sign {
                Some("%".to_string())
            } else {
                None
            };
            let title = divi.attributes.get("title").cloned().or_else(|| {
                if !divi.inner_text.trim().is_empty() {
                    Some(divi.inner_text.trim().to_string())
                } else {
                    None
                }
            });
            NodeKind::Counter {
                number,
                prefix: None,
                suffix,
                title,
            }
        }
        "et_pb_accordion" => {
            let items: Vec<AccordionItem> = divi
                .children
                .iter()
                .filter(|c| {
                    c.tag == "et_pb_accordion_item"
                        || c.tag == "et_pb_toggle"
                        || !c.inner_text.is_empty()
                })
                .map(|c| AccordionItem {
                    title: c
                        .attributes
                        .get("title")
                        .cloned()
                        .unwrap_or_else(|| "Accordion Item".to_string()),
                    content_html: ctx.normalize_html(c.inner_text.trim()),
                })
                .collect();
            children.clear();
            NodeKind::Accordion { items }
        }
        "et_pb_toggle" => {
            let title = divi
                .attributes
                .get("title")
                .cloned()
                .unwrap_or_else(|| "Toggle".to_string());
            let content_html = ctx.normalize_html(divi.inner_text.trim());
            NodeKind::Toggle {
                items: vec![ToggleItem {
                    title,
                    content_html,
                }],
            }
        }
        "et_pb_tabs" => {
            let items: Vec<TabItem> = divi
                .children
                .iter()
                .filter(|c| c.tag == "et_pb_tab" || !c.inner_text.is_empty())
                .map(|c| TabItem {
                    title: c
                        .attributes
                        .get("title")
                        .cloned()
                        .unwrap_or_else(|| "Tab".to_string()),
                    content_html: ctx.normalize_html(c.inner_text.trim()),
                })
                .collect();
            children.clear();
            NodeKind::Tabs { items }
        }
        "et_pb_testimonial" => {
            let content = ctx.normalize_html(divi.inner_text.trim());
            let name = divi
                .attributes
                .get("author")
                .cloned()
                .unwrap_or_else(|| "Client".to_string());
            let job_title = divi.attributes.get("job_title").cloned().unwrap_or_default();
            let company = divi
                .attributes
                .get("company_name")
                .cloned()
                .unwrap_or_default();
            let job = match (job_title.is_empty(), company.is_empty()) {
                (false, false) => format!("{}, {}", job_title, company),
                (false, true) => job_title,
                (true, false) => company,
                (true, true) => String::new(),
            };
            let image_url = divi
                .attributes
                .get("portrait_url")
                .filter(|s| !s.is_empty())
                .map(|u| ctx.resolve_divi_attr(u));
            NodeKind::Testimonial {
                content,
                name,
                job,
                image_url,
            }
        }
        "et_pb_pricing_tables" => {
            let cols = divi.children.len().clamp(1, 4) as u8;
            NodeKind::Grid {
                columns: cols,
                gap: 24,
            }
        }
        "et_pb_pricing_table" => {
            let raw_title = divi
                .attributes
                .get("title")
                .cloned()
                .unwrap_or_else(|| "Plan".to_string());
            let subtitle = divi
                .attributes
                .get("subtitle")
                .filter(|s| !s.trim().is_empty())
                .cloned();
            let title = if let Some(sub) = subtitle {
                format!("{} ({})", raw_title, sub)
            } else {
                raw_title
            };
            let currency = divi
                .attributes
                .get("currency")
                .cloned()
                .unwrap_or_else(|| "€".to_string());
            let sum = divi
                .attributes
                .get("sum")
                .cloned()
                .unwrap_or_else(|| "29".to_string());
            let price = if sum.starts_with('$') || sum.starts_with('€') || sum.starts_with('£') {
                sum
            } else {
                format!("{}{}", currency, sum)
            };
            let period = divi
                .attributes
                .get("per")
                .filter(|p| !p.trim().is_empty())
                .map(|p| format!("/ {}", p))
                .unwrap_or_default();

            let clean_feature_line = |l: &str| -> String {
                l.trim()
                    .trim_start_matches("<p>")
                    .trim_end_matches("</p>")
                    .trim()
                    .trim_start_matches('+')
                    .trim_start_matches('-')
                    .trim()
                    .to_string()
            };

            let mut features: Vec<String> = divi
                .inner_text
                .lines()
                .map(clean_feature_line)
                .filter(|l| !l.is_empty())
                .collect();
            for child in &divi.children {
                let cf = clean_feature_line(&child.inner_text);
                if !cf.is_empty() {
                    features.push(cf);
                }
            }

            let button_text = divi
                .attributes
                .get("button_text")
                .cloned()
                .unwrap_or_default();
            let button_url = divi
                .attributes
                .get("button_url")
                .map(|u| ctx.resolve_divi_attr(u))
                .unwrap_or_else(|| "#".to_string());

            if divi.attributes.get("featured").map(|v| v == "on").unwrap_or(false) {
                let bdr = styles.border.get_or_insert_with(Default::default);
                bdr.color = Some("#3D2B01".to_string());
                bdr.width = Some("2px".to_string());
                bdr.style = Some("solid".to_string());
            }

            children.clear();
            NodeKind::PriceTable {
                title,
                price,
                period,
                features,
                button_text,
                button_url,
            }
        }
        "et_pb_contact_form" => {
            let title = divi.attributes.get("title").cloned();
            let button_text = divi.attributes.get("submit_button_text").cloned();
            children.clear();
            NodeKind::Form {
                form_id: 1,
                title,
                button_text,
                endpoint: None,
            }
        }
        "dvppl_cf7_styler" => {
            let cf7_id = divi
                .attributes
                .get("cf7")
                .cloned()
                .unwrap_or_else(|| "475".to_string());
            let form_html = ctx
                .cf7_forms
                .get(&cf7_id)
                .cloned()
                .or_else(|| ctx.cf7_forms.values().next().cloned())
                .unwrap_or_else(|| {
                    render_fallback_cf7_form(&cf7_id)
                });
            children.clear();
            NodeKind::CustomHtml {
                raw: format!("<div class=\"dvppl-cf7-styler\">{}</div>", form_html),
            }
        }
        "spb_kw_calculator_module" | "spb_kitwerk_calculator" => {
            children.clear();
            NodeKind::CustomHtml {
                raw: render_spb_calculator_widget(&ctx.spb_services),
            }
        }
        "et_pb_video" => {
            let raw_url = divi
                .attributes
                .get("src")
                .cloned()
                .unwrap_or_default();
            let url = normalize_video_embed_url(&raw_url);
            NodeKind::Video {
                url,
                aspect_ratio: "16/9".to_string(),
                autoplay: false,
                muted: false,
                loop_video: false,
            }
        }
        "et_pb_map" => {
            let address = divi
                .attributes
                .get("address")
                .cloned()
                .unwrap_or_else(|| "New York, NY".to_string());
            let zoom = divi
                .attributes
                .get("zoom_level")
                .and_then(|z| z.parse::<u8>().ok());
            children.clear();
            NodeKind::Map { address, zoom }
        }
        "et_pb_social_media_follow" => {
            let items: Vec<SocialIconItem> = divi
                .children
                .iter()
                .map(|c| {
                    let net = c
                        .attributes
                        .get("social_network")
                        .map(|s| s.as_str())
                        .unwrap_or("facebook");
                    let icon = match net {
                        "twitter" | "x" => "fa-brands fa-x-twitter",
                        "linkedin" => "fa-brands fa-linkedin",
                        "instagram" => "fa-brands fa-instagram",
                        "youtube" => "fa-brands fa-youtube",
                        "github" => "fa-brands fa-github",
                        "whatsapp" => "fa-brands fa-whatsapp",
                        _ => "fa-brands fa-facebook",
                    }
                    .to_string();
                    let link = c
                        .attributes
                        .get("url")
                        .filter(|s| !s.is_empty())
                        .map(|u| ctx.resolve_divi_attr(u))
                        .unwrap_or_else(|| "#".to_string());
                    SocialIconItem { icon, link }
                })
                .collect();
            children.clear();
            NodeKind::SocialIcons { items }
        }
        "et_pb_gallery" => NodeKind::CustomHtml {
            raw: "<div class=\"np-gallery\"></div>".to_string(),
        },
        "et_pb_blog" => NodeKind::CustomHtml {
            raw: "<div class=\"np-posts-grid\"></div>".to_string(),
        },
        _ => NodeKind::ShortcodeFallback {
            tag: divi.tag.clone(),
            attributes: divi.attributes.clone(),
            content: divi.inner_text.clone(),
        },
    };

    styles.sync_flat_and_nested();

    NeopressNode {
        id,
        kind,
        styles,
        children,
    }
}

fn escape_html(val: &str) -> String {
    val.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn map_divi_icon(raw_icon: &str) -> String {
    let trimmed = raw_icon.trim();
    if trimmed.starts_with("fa-") || trimmed.contains("fa-solid") || trimmed.contains("fa-brands") {
        return trimmed.to_string();
    }
    let code = trimmed.split("||").next().unwrap_or(trimmed).trim().to_lowercase();
    match code.as_str() {
        "&#xf5d2;" | "f5d2" => "fa-solid fa-layer-group".to_string(),
        "&#xf0e7;" | "f0e7" => "fa-solid fa-bolt".to_string(),
        "&#xf4fd;" | "f4fd" => "fa-solid fa-user-tie".to_string(),
        "&#xf5a0;" | "f5a0" => "fa-solid fa-map-location-dot".to_string(),
        "&#xf2a0;" | "f2a0" => "fa-solid fa-phone-volume".to_string(),
        "&#x3b;" | "3b" | ";" => "fa-solid fa-chevron-down".to_string(),
        "&#xe093;" => "fa-brands fa-facebook".to_string(),
        "&#xe09a;" => "fa-brands fa-instagram".to_string(),
        _ => "fa-solid fa-bolt".to_string(),
    }
}

fn parse_divi_quad(raw: &str) -> Option<Spacing> {
    let parts: Vec<&str> = raw.split('|').collect();
    let clean_part = |idx: usize| -> Option<String> {
        parts.get(idx).and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty() || matches!(trimmed, "true" | "false" | "on" | "off") {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    };

    let top = clean_part(0);
    let right = clean_part(1);
    let bottom = clean_part(2);
    let left = clean_part(3);

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

fn extract_divi_styles(attrs: &HashMap<String, String>, ctx: &DiviContext) -> StyleProperties {
    let mut styles = StyleProperties::default();

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
            color: bg_color,
            image_url: bg_img,
            position: None,
            size: Some("cover".to_string()),
        });
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
                    let tokens: Vec<&str> = part.trim().split_whitespace().collect();
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

    if let Some(pad) = attrs.get("custom_padding") {
        styles.padding = parse_divi_quad(pad);
    }

    if let Some(mar) = attrs.get("custom_margin") {
        styles.margin = parse_divi_quad(mar);
    }

    let mut border_radius = None;
    if let Some(radii) = attrs.get("border_radii") {
        let parts: Vec<&str> = radii.split('|').collect();
        if parts.len() >= 5 {
            let t = if parts[1].trim().is_empty() { "0px" } else { parts[1].trim() };
            let r = if parts[2].trim().is_empty() { "0px" } else { parts[2].trim() };
            let b = if parts[3].trim().is_empty() { "0px" } else { parts[3].trim() };
            let l = if parts[4].trim().is_empty() { "0px" } else { parts[4].trim() };
            if t != "0px" || r != "0px" || b != "0px" || l != "0px" {
                let rad = if t == r && r == b && b == l {
                    t.to_string()
                } else {
                    format!("{} {} {} {}", t, r, b, l)
                };
                border_radius = Some(rad);
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
        .filter(|s| !s.is_empty())
        .cloned();

    // Parse Divi font string: "Bitter|700|||||||" or "Teko|700|||||||"
    let mut font_weight = None;
    if let Some(font_raw) = attrs
        .get("title_font")
        .or_else(|| attrs.get("header_font"))
        .or_else(|| attrs.get("text_font"))
        .or_else(|| attrs.get("body_font"))
    {
        let parts: Vec<&str> = font_raw.split('|').collect();
        if let Some(fam) = parts.first().map(|s| s.trim()).filter(|s| !s.is_empty()) {
            let resolved_fam = match fam {
                "--et_global_heading_font" => "'Squada One', sans-serif".to_string(),
                "--et_global_body_font" => "'Barlow', sans-serif".to_string(),
                other => format!("'{}', sans-serif", other),
            };
            styles.font_family = Some(resolved_fam);
        }
        if let Some(w) = parts.get(1).map(|s| s.trim()).filter(|s| !s.is_empty()) {
            font_weight = Some(w.to_string());
        }
    }

    if text_align.is_some() || text_color.is_some() || font_size.is_some() || line_height.is_some() || font_weight.is_some() {
        styles.typography = Some(TypographyConfig {
            font_size,
            font_weight,
            color: text_color,
            text_align,
            line_height,
        });
    }

    if let Some(w) = attrs.get("width").filter(|s| !s.is_empty()) {
        styles.width = Some(w.clone());
    }
    if let Some(mw) = attrs.get("max_width").filter(|s| !s.is_empty()) {
        styles.max_width = Some(mw.clone());
    }
    if let Some(mh) = attrs.get("max_height").filter(|s| !s.is_empty()) {
        styles.min_height = None;
        let _ = mh;
    }

    if let Some(shadow_preset) = attrs.get("box_shadow_style").filter(|s| !s.is_empty() && *s != "none") {
        let shadow = match shadow_preset.as_str() {
            "preset1" => "0px 2px 18px 0px rgba(0,0,0,0.3)",
            "preset3" => "0px 12px 18px -6px rgba(0,0,0,0.3)",
            "preset4" => "10px 10px 0px 0px rgba(0,0,0,0.2)",
            "preset6" => "0px 0px 18px 0px rgba(0,0,0,0.3)",
            _ => "0px 4px 16px rgba(0,0,0,0.18)",
        };
        styles.box_shadow = Some(shadow.to_string());
    }

    if let Some(cls) = attrs.get("module_class").filter(|s| !s.trim().is_empty()) {
        styles.css_classes = Some(cls.trim().to_string());
    }

    styles.sync_flat_and_nested();
    styles
}

fn render_fallback_cf7_form(form_id: &str) -> String {
    format!(
        r#"<form class="wpcf7-form np-cf7-form np-form" method="POST" action="/wp-json/neopress/v1/forms/1/submit" onsubmit="event.preventDefault();var f=this,s=f.querySelector('.np-form-status'),d=Object.fromEntries(new FormData(f).entries());fetch(f.action,{{method:'POST',headers:{{'Content-Type':'application/json'}},body:JSON.stringify(d)}}).then(function(r){{if(s)s.textContent=r.ok?'Bedankt voor je bericht. Het is verzonden.':'Er is een fout opgetreden.';if(r.ok)f.reset();}});return false;" data-cf7-id="{}">
<div class="dipe-form-row" style="display:flex;gap:16px;flex-wrap:wrap;margin-bottom:12px;">
  <div class="dipe-form-col" style="flex:1;min-width:200px;"><label style="color:#fff;display:block;margin-bottom:6px;">Wat is je naam?<input type="text" name="your-name" placeholder="Voor- en achternaam" required style="width:100%;padding:12px;border:3px solid #e23212;border-radius:8px;background:#fff;color:#000;" /></label></div>
  <div class="dipe-form-col" style="flex:1;min-width:200px;"><label style="color:#fff;display:block;margin-bottom:6px;">Waar ontvang je e-post?<input type="email" name="your-email" placeholder="Email-adres" required style="width:100%;padding:12px;border:3px solid #e23212;border-radius:8px;background:#fff;color:#000;" /></label></div>
</div>
<div class="dipe-form-row" style="display:flex;gap:16px;flex-wrap:wrap;margin-bottom:12px;">
  <div class="dipe-form-col" style="flex:1;min-width:200px;"><label style="color:#fff;display:block;margin-bottom:6px;">Mogen we je appen?*<input type="tel" name="tel-604" placeholder="Whatsapp nummer" style="width:100%;padding:12px;border:3px solid #e23212;border-radius:8px;background:#fff;color:#000;" /></label></div>
  <div class="dipe-form-col" style="flex:1;min-width:200px;"><label style="color:#fff;display:block;margin-bottom:6px;">Is t ver weg?<input type="text" name="your-plaats" placeholder="Woonplaats" required style="width:100%;padding:12px;border:3px solid #e23212;border-radius:8px;background:#fff;color:#000;" /></label></div>
</div>
<label style="color:#fff;display:block;margin-bottom:12px;">Wat moet er gekit worden?:<textarea name="your-message" rows="4" placeholder="Duidelijke opdrachtomschriving" style="width:100%;padding:12px;border:3px solid #e23212;border-radius:8px;background:#fff;color:#000;"></textarea></label>
<div style="text-align:right;"><button type="submit" class="et_pb_button" style="background:#e23212;color:#fcfbfc;border:2px solid #fff;border-radius:8px;padding:10px 24px;font-size:1.4em;cursor:pointer;">Verzenden</button></div>
<div class="np-form-status" aria-live="polite" style="color:#7CDA24;margin-top:12px;font-weight:600;"></div>
</form>"#,
        escape_html(form_id)
    )
}

/// Renders the complete interactive 2-step Service Price Calculator (`[spb_kw_calculator_module]` & `[spb_kitwerk_calculator]`)
/// populated with the `spb_service` items imported from WXR.
pub fn render_spb_calculator_widget(services: &[SpbServiceItem]) -> String {
    let default_services = vec![
        SpbServiceItem {
            id: 583,
            title: "Keuken/Badkamer/sanitair".to_string(),
            price: 9.0,
            unit: "per meter".to_string(),
            is_extra: false,
        },
        SpbServiceItem {
            id: 588,
            title: "Plinten".to_string(),
            price: 4.5,
            unit: "per meter".to_string(),
            is_extra: false,
        },
        SpbServiceItem {
            id: 589,
            title: "Stucwerk".to_string(),
            price: 5.71,
            unit: "per meter".to_string(),
            is_extra: false,
        },
        SpbServiceItem {
            id: 582,
            title: "Verwijderen van kit (silicone)".to_string(),
            price: 5.0,
            unit: "".to_string(),
            is_extra: false,
        },
        SpbServiceItem {
            id: 591,
            title: "Dilatatie <15mm".to_string(),
            price: 15.0,
            unit: "per meter".to_string(),
            is_extra: false,
        },
        SpbServiceItem {
            id: 592,
            title: "Dilatatie >15 <25mm".to_string(),
            price: 16.09,
            unit: "per meter".to_string(),
            is_extra: false,
        },
    ];

    let active_services = if services.is_empty() {
        &default_services
    } else {
        services
    };

    let mut options_html = String::from("<option value=\"\">Kies een klus</option>\n");
    for svc in active_services.iter().filter(|s| !s.is_extra) {
        options_html.push_str(&format!(
            "<option value=\"{}\" data-price=\"{}\" data-unit=\"{}\">{}</option>\n",
            svc.id,
            svc.price,
            escape_html(&svc.unit),
            escape_html(&svc.title)
        ));
    }

    format!(
        r#"<div class="spb-kw-calculator et_pb_module" data-start-fee="150">
  <div class="spb-kw-success" style="display:none"></div>
  <div class="spb-kw-step spb-kw-step-calculator">
    <h3 class="et_pb_module_header">Kitwerk prijsindicatie</h3>
    <div class="spb-kw-copy spb-kw-copy--step1">
      <p>Bereken snel jouw prijsindicatie voor professioneel kitwerk en stuur deze naar je inbox.</p>
      <ul class="spb-kw-icons">
        <li>🔒 Wij gaan veilig met jouw gegevens om.</li>
        <li>⚡ Supersnel een indicatie in jouw mailbox.</li>
        <li>🛠️ Strak, duurzaam kitwerk door vakmensen.</li>
      </ul>
    </div>
    <div class="spb-kw-top-controls">
      <select class="spb-kw-service-select" style="display:none">
        {options_html}
      </select>
      <div class="spb-kw-step1-services"></div>
    </div>
    <div class="spb-kw-actions">
      <button type="button" class="et_pb_button spb-kw-next">Volgende</button>
    </div>
  </div>
  <div class="spb-kw-step spb-kw-step-lead" style="display:none">
    <form class="spb-kw-lead-form" novalidate>
      <h4 class="spb-kw-lead-title">Ontvang jouw kitwerk prijs per e-mail</h4>
      <div class="spb-kw-copy spb-kw-copy--step2">
        <p>🧾 Overzicht van jouw selectie:</p>
      </div>
      <div class="spb-kw-review" aria-live="polite"></div>
      <p class="spb-kw-info-line">🚗 Reisvergoeding: <span class="spb-kw-travel-rate">€ 0,73</span> per km (niet meegerekend in de prijs)</p>
      <div class="spb-kw-copy spb-kw-copy--step2">
        <p>🔒 We gebruiken je gegevens alleen voor jouw kitwerk-aanvraag.</p>
        <p>✉️ Geen spam. Je ontvangt één bevestiging met jouw prijsindicatie.</p>
      </div>
      <input type="text" name="name" placeholder="Naam" required autocomplete="name" />
      <input type="email" name="email" placeholder="E-mail" required autocomplete="email" />
      <input type="tel" name="phone" placeholder="Telefoon" autocomplete="tel" />
      <input type="text" name="city" placeholder="Woonplaats" autocomplete="address-level2" />
      <label class="spb-kw-consent">
        <input type="checkbox" name="consent" required />
        <span>Ik ga akkoord met de <a href="/privacy-policy" target="_blank" rel="noopener">privacy voorwaarden</a>.</span>
      </label>
      <input type="hidden" name="selected" />
      <input type="hidden" name="total" />
      <div class="spb-kw-lead-actions">
        <button type="button" class="et_pb_button spb-kw-prev">Vorige</button>
        <button type="submit" class="et_pb_button">Verstuur</button>
      </div>
      <div class="spb-kw-message" aria-live="polite"></div>
    </form>
  </div>
</div>
<script>
(function(){{
  function fmtEUR(v){{try{{return new Intl.NumberFormat('nl-NL',{{style:'currency',currency:'EUR'}}).format(v);}}catch(e){{return '€ '+(v||0).toFixed(2);}}}}
  document.querySelectorAll('.spb-kw-calculator').forEach(function(root){{
    if(root.dataset.npBound==='1')return;
    root.dataset.npBound='1';
    var startFee=parseFloat(root.dataset.startFee||'150')||150;
    var srcSelect=root.querySelector('.spb-kw-service-select');
    var step1=root.querySelector('.spb-kw-step1-services');
    if(!srcSelect||!step1)return;
    step1.innerHTML='<div class="spb-kw-step1-list"><div class="spb-kw-rows"></div></div><div class="spb-kw-actions" style="margin-top:10px"><button type="button" class="spb-kw-add-service">Voeg nog een dienst toe</button></div>';
    var rowsEl=step1.querySelector('.spb-kw-rows');
    function addRow(){{
      var r=document.createElement('div');
      r.className='spb-kw-row';
      r.innerHTML='<div class="spb-kw-row-field spb-kw-row-meters-wrap"><label>Meters</label><input type="number" min="0" max="999" step="1" class="spb-kw-row-meters" placeholder="0" inputmode="numeric" /></div><div class="spb-kw-row-field spb-kw-row-service-wrap"><label>Dienst</label><select class="spb-kw-row-service">'+srcSelect.innerHTML+'</select></div><div class="spb-kw-row-actions"><button type="button" class="spb-kw-remove-row" title="Verwijder">×</button></div>';
      rowsEl.appendChild(r);
      var all=rowsEl.querySelectorAll('.spb-kw-row');
      all.forEach(function(el,idx){{var btn=el.querySelector('.spb-kw-remove-row');if(btn)btn.style.display=idx>0?'inline-block':'none';}});
      r.querySelector('.spb-kw-remove-row').addEventListener('click',function(){{r.remove();}});
    }}
    addRow();
    step1.querySelector('.spb-kw-add-service').addEventListener('click',addRow);
    var nextBtn=root.querySelector('.spb-kw-next');
    var prevBtn=root.querySelector('.spb-kw-prev');
    var stepCalc=root.querySelector('.spb-kw-step-calculator');
    var stepLead=root.querySelector('.spb-kw-step-lead');
    if(nextBtn){{
      nextBtn.addEventListener('click',function(e){{
        e.preventDefault();
        stepCalc.style.display='none';
        var review=root.querySelector('.spb-kw-review');
        var tbodyHtml='';
        var baseSum=0;
        var selected=[];
        rowsEl.querySelectorAll('.spb-kw-row').forEach(function(r){{
          var sel=r.querySelector('select');
          var inp=r.querySelector('input[type="number"]');
          var id=parseInt(sel&&sel.value,10)||0;
          var meters=parseFloat(inp&&inp.value)||0;
          var opt=sel&&sel.options[sel.selectedIndex];
          var price=parseFloat(opt&&opt.getAttribute('data-price'))||0;
          var unit=(opt&&opt.getAttribute('data-unit'))||'';
          if(id&&meters>0&&price>0){{
            var line=price*meters;
            baseSum+=line;
            selected.push({{id:id,qty:meters,service:opt.text}});
            tbodyHtml+='<tr><td class="col-item">'+opt.text+(unit?' <small>/ '+unit+'</small>':'')+'</td><td class="col-qty">'+meters+'</td><td class="col-price">'+fmtEUR(price)+'</td><td class="col-total">'+fmtEUR(line)+'</td></tr>';
          }}
        }});
        var total=Math.max(startFee,baseSum);
        if(startFee>0&&total>baseSum){{
          tbodyHtml+='<tr><td class="col-item">Start tarief (minimum)</td><td class="col-qty">—</td><td class="col-price">'+fmtEUR(startFee)+'</td><td class="col-total">'+fmtEUR(startFee)+'</td></tr>';
        }}
        review.innerHTML='<table class="spb-kw-review-table"><thead><tr><th class="col-item">Item</th><th class="col-qty">Aantal</th><th class="col-price">Prijs</th><th class="col-total">Totaal</th></tr></thead><tbody class="spb-kw-rows">'+tbodyHtml+'</tbody><tfoot><tr><th colspan="3" style="text-align:right">Indicatie (incl. btw)</th><th class="spb-kw-grand-total">'+fmtEUR(total)+'</th></tr></tfoot></table>';
        root.querySelector('input[name="selected"]').value=JSON.stringify(selected);
        root.querySelector('input[name="total"]').value=total.toFixed(2);
        stepLead.style.display='block';
      }});
    }}
    if(prevBtn){{
      prevBtn.addEventListener('click',function(e){{
        e.preventDefault();
        stepLead.style.display='none';
        stepCalc.style.display='block';
      }});
    }}
    var form=root.querySelector('.spb-kw-lead-form');
    if(form){{
      form.addEventListener('submit',function(e){{
        e.preventDefault();
        var msgEl=form.querySelector('.spb-kw-message');
        var consent=form.querySelector('input[name="consent"]');
        if(!consent||!consent.checked){{
          if(msgEl)msgEl.textContent='Vink de privacy voorwaarden aan.';
          return;
        }}
        var payload={{
          name:form.querySelector('input[name="name"]').value,
          email:form.querySelector('input[name="email"]').value,
          phone:form.querySelector('input[name="phone"]').value,
          city:form.querySelector('input[name="city"]').value,
          selected:form.querySelector('input[name="selected"]').value,
          total:form.querySelector('input[name="total"]').value
        }};
        fetch('/wp-json/neopress/v1/forms/1/submit',{{
          method:'POST',
          headers:{{'Content-Type':'application/json'}},
          body:JSON.stringify(payload)
        }}).then(function(r){{
          if(r.ok){{
            stepCalc.style.display='none';
            stepLead.style.display='none';
            var succ=root.querySelector('.spb-kw-success');
            if(succ){{succ.textContent='Bedankt! Je ontvangt snel een mail met de prijsindicatie. Neem direct contact op voor vragen.';succ.style.display='block';}}
          }}else if(msgEl){{
            msgEl.textContent='Er ging iets mis.';
          }}
        }});
      }});
    }}
  }});
}})();
</script>"#,
        options_html = options_html
    )
}
