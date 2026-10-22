use crate::elementor::normalize_video_embed_url;
use crate::schema::{
    BackgroundConfig, BorderConfig, NeopressLayout, NeopressNode, NodeKind, Spacing,
    StyleProperties, TypographyConfig,
};
use regex::Regex;
use serde_json::Value;

#[derive(Debug, Clone)]
struct GutenbergBlock {
    name: String,
    attrs: Value,
    inner_html: String,
    children: Vec<GutenbergBlock>,
}

#[derive(Debug)]
enum BlockToken {
    Open { name: String, attrs: Value },
    Close { name: String },
    SelfClosing { name: String, attrs: Value },
    Html(String),
}

/// Parses WordPress Gutenberg block markup (`<!-- wp:... -->`) into a canonical NeopressLayout AST.
pub fn transpile_gutenberg_content(content: &str) -> Result<NeopressLayout, String> {
    let tokens = tokenize_gutenberg(content);
    let blocks = parse_gutenberg_tree(tokens);

    let mut id_counter: usize = 0;
    let root_children: Vec<NeopressNode> = blocks
        .iter()
        .filter_map(|b| map_gutenberg_block(b, &mut id_counter))
        .collect();

    Ok(NeopressLayout {
        version: 1,
        root: NeopressNode {
            id: "gutenberg_root".to_string(),
            kind: NodeKind::Section,
            styles: StyleProperties::default(),
            children: root_children,
        },
        global_styles: None,
    })
}

fn tokenize_gutenberg(input: &str) -> Vec<BlockToken> {
    let comment_re =
        Regex::new(r"(?s)<!--\s*(/?)wp:([a-z0-9_\-]+(?:/[a-z0-9_\-]+)?)(?:\s+(\{.*?\}))?\s*(/?)-->")
            .unwrap();

    let mut tokens = Vec::new();
    let mut last_idx = 0;

    for caps in comment_re.captures_iter(input) {
        let full_match = caps.get(0).unwrap();
        let start = full_match.start();
        let end = full_match.end();

        if start > last_idx {
            let html = &input[last_idx..start];
            if !html.trim().is_empty() {
                tokens.push(BlockToken::Html(html.to_string()));
            }
        }

        let is_close = caps.get(1).map(|m| m.as_str() == "/").unwrap_or(false);
        let name = caps.get(2).unwrap().as_str().to_string();
        let attrs = caps
            .get(3)
            .and_then(|m| serde_json::from_str::<Value>(m.as_str()).ok())
            .unwrap_or(Value::Null);
        let is_self_closing = caps.get(4).map(|m| m.as_str() == "/").unwrap_or(false);

        if is_close {
            tokens.push(BlockToken::Close { name });
        } else if is_self_closing {
            tokens.push(BlockToken::SelfClosing { name, attrs });
        } else {
            tokens.push(BlockToken::Open { name, attrs });
        }

        last_idx = end;
    }

    if last_idx < input.len() {
        let html = &input[last_idx..];
        if !html.trim().is_empty() {
            tokens.push(BlockToken::Html(html.to_string()));
        }
    }

    tokens
}

fn parse_gutenberg_tree(tokens: Vec<BlockToken>) -> Vec<GutenbergBlock> {
    let mut root_blocks: Vec<GutenbergBlock> = Vec::new();
    let mut stack: Vec<GutenbergBlock> = Vec::new();

    for token in tokens {
        match token {
            BlockToken::Open { name, attrs } => {
                stack.push(GutenbergBlock {
                    name,
                    attrs,
                    inner_html: String::new(),
                    children: Vec::new(),
                });
            }
            BlockToken::SelfClosing { name, attrs } => {
                let blk = GutenbergBlock {
                    name,
                    attrs,
                    inner_html: String::new(),
                    children: Vec::new(),
                };
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(blk);
                } else {
                    root_blocks.push(blk);
                }
            }
            BlockToken::Close { name } => {
                if let Some(pos) = stack.iter().rposition(|b| b.name == name) {
                    while stack.len() > pos + 1 {
                        if let Some(unclosed) = stack.pop() {
                            if let Some(parent) = stack.last_mut() {
                                parent.children.push(unclosed);
                            } else {
                                root_blocks.push(unclosed);
                            }
                        }
                    }
                    if let Some(closed) = stack.pop() {
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(closed);
                        } else {
                            root_blocks.push(closed);
                        }
                    }
                }
            }
            BlockToken::Html(html) => {
                if let Some(top) = stack.last_mut() {
                    top.inner_html.push_str(&html);
                } else {
                    root_blocks.push(GutenbergBlock {
                        name: "core/freeform".to_string(),
                        attrs: Value::Null,
                        inner_html: html,
                        children: Vec::new(),
                    });
                }
            }
        }
    }

    while let Some(unclosed) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(unclosed);
        } else {
            root_blocks.push(unclosed);
        }
    }

    root_blocks
}

fn map_gutenberg_block(block: &GutenbergBlock, counter: &mut usize) -> Option<NeopressNode> {
    *counter += 1;
    let short_name = block
        .name
        .strip_prefix("core/")
        .unwrap_or(&block.name)
        .replace(['/', '-'], "_");
    let id = format!("gb_{}_{}", short_name, *counter);
    let mut styles = extract_gutenberg_styles(&block.attrs);

    let mut children: Vec<NeopressNode> = block
        .children
        .iter()
        .filter_map(|c| map_gutenberg_block(c, counter))
        .collect();

    let norm_name = block.name.strip_prefix("core/").unwrap_or(&block.name);

    let kind = match norm_name {
        "group" => {
            let tag_name = block
                .attrs
                .get("tagName")
                .and_then(|v| v.as_str())
                .unwrap_or("div");
            if tag_name == "section" {
                NodeKind::Section
            } else {
                NodeKind::Container
            }
        }
        "columns" => {
            if styles.gap.is_none() {
                styles.gap = Some("24px".to_string());
            }
            NodeKind::Flex {
                direction: "row".to_string(),
                justify: "flex-start".to_string(),
                align: "stretch".to_string(),
                wrap: false,
                basis: None,
            }
        }
        "column" => {
            let basis = block
                .attrs
                .get("width")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            if basis.is_none() && styles.flex.is_none() {
                styles.flex = Some("1".to_string());
            }
            NodeKind::Flex {
                direction: "column".to_string(),
                justify: "flex-start".to_string(),
                align: "stretch".to_string(),
                wrap: false,
                basis,
            }
        }
        "cover" => {
            let bg_url = block
                .attrs
                .get("url")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .or_else(|| extract_attr_from_html(&block.inner_html, "src"));

            let overlay_color = block
                .attrs
                .get("customOverlayColor")
                .or_else(|| block.attrs.get("overlayColor"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| {
                    if s.starts_with('#') || s.starts_with("rgb") || s.starts_with("var(") {
                        s.to_string()
                    } else {
                        format!("var(--wp--preset--color--{}, #0f172a)", s)
                    }
                })
                .unwrap_or_else(|| "#0f172a".to_string());

            styles.background = Some(BackgroundConfig {
                color: Some(overlay_color),
                image_url: bg_url,
                position: Some("center center".to_string()),
                size: Some("cover".to_string()),
            });
            if styles.padding.is_none() {
                styles.padding = Some(Spacing {
                    top: Some("80px".to_string()),
                    right: Some("24px".to_string()),
                    bottom: Some("80px".to_string()),
                    left: Some("24px".to_string()),
                });
            }
            let typo = styles.typography.get_or_insert_with(Default::default);
            if typo.color.is_none() {
                typo.color = Some("#ffffff".to_string());
            }
            NodeKind::Section
        }
        "media-text" => {
            let media_url = block
                .attrs
                .get("mediaUrl")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| extract_attr_from_html(&block.inner_html, "src"))
                .unwrap_or_default();
            let media_alt = extract_attr_from_html(&block.inner_html, "alt").unwrap_or_default();
            let media_pos = block
                .attrs
                .get("mediaPosition")
                .and_then(|v| v.as_str())
                .unwrap_or("left");

            *counter += 1;
            let img_node = NeopressNode {
                id: format!("gb_media_{}", *counter),
                kind: NodeKind::Image {
                    src: media_url,
                    alt: media_alt,
                    width: None,
                    height: None,
                    link: None,
                },
                styles: StyleProperties::default(),
                children: Vec::new(),
            };

            *counter += 1;
            let content_col = NeopressNode {
                id: format!("gb_media_body_{}", *counter),
                kind: NodeKind::Container,
                styles: StyleProperties::default(),
                children: std::mem::take(&mut children),
            };

            if media_pos == "right" {
                children.push(content_col);
                children.push(img_node);
            } else {
                children.push(img_node);
                children.push(content_col);
            }

            NodeKind::Grid {
                columns: 2,
                gap: 32,
            }
        }
        "buttons" => {
            if styles.gap.is_none() {
                styles.gap = Some("12px".to_string());
            }
            NodeKind::Flex {
                direction: "row".to_string(),
                justify: "flex-start".to_string(),
                align: "center".to_string(),
                wrap: true,
                basis: None,
            }
        }
        "button" => {
            let url = extract_attr_from_html(&block.inner_html, "href")
                .or_else(|| {
                    block
                        .attrs
                        .get("url")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "#".to_string());
            let target = extract_attr_from_html(&block.inner_html, "target")
                .unwrap_or_else(|| "_self".to_string());
            let text = strip_html_tags(&block.inner_html);
            NodeKind::Button {
                text: if text.is_empty() {
                    "Click Here".to_string()
                } else {
                    text
                },
                url,
                target,
            }
        }
        "heading" => {
            let level = block
                .attrs
                .get("level")
                .and_then(|v| v.as_u64())
                .map(|l| l as u8)
                .or_else(|| detect_heading_level(&block.inner_html))
                .unwrap_or(2);
            let text = strip_outer_heading_tag(&block.inner_html);
            NodeKind::Heading { text, level }
        }
        "paragraph" | "freeform" => {
            let content_html = block.inner_html.trim().to_string();
            if content_html.is_empty() {
                return None;
            }
            NodeKind::Text { content_html }
        }
        "list" => {
            let content_html = if !block.children.is_empty() {
                let is_ordered = block
                    .attrs
                    .get("ordered")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                let tag = if is_ordered { "ol" } else { "ul" };
                let items_html: String = block
                    .children
                    .iter()
                    .map(|li| li.inner_html.trim())
                    .collect::<Vec<_>>()
                    .join("\n");
                children.clear();
                format!("<{}>\n{}\n</{}>", tag, items_html, tag)
            } else {
                block.inner_html.trim().to_string()
            };
            NodeKind::Text { content_html }
        }
        "image" => {
            let src = block
                .attrs
                .get("url")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| extract_attr_from_html(&block.inner_html, "src"))
                .unwrap_or_default();
            let alt = block
                .attrs
                .get("alt")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| extract_attr_from_html(&block.inner_html, "alt"))
                .unwrap_or_default();
            let width = block
                .attrs
                .get("width")
                .and_then(|v| v.as_u64())
                .map(|w| w as u32);
            let height = block
                .attrs
                .get("height")
                .and_then(|v| v.as_u64())
                .map(|h| h as u32);
            let link = block
                .attrs
                .get("href")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            NodeKind::Image {
                src,
                alt,
                width,
                height,
                link,
            }
        }
        "spacer" => {
            let height = block
                .attrs
                .get("height")
                .and_then(|v| {
                    v.as_u64().or_else(|| {
                        v.as_str().and_then(|s| {
                            let digits: String =
                                s.chars().take_while(|c| c.is_ascii_digit()).collect();
                            digits.parse::<u64>().ok()
                        })
                    })
                })
                .unwrap_or(40) as u32;
            NodeKind::Spacer { height }
        }
        "separator" => NodeKind::Divider {
            style: "solid".to_string(),
        },
        "quote" | "pullquote" => {
            let cite = extract_tag_inner(&block.inner_html, "cite").unwrap_or_default();
            let content = strip_html_tags(
                &Regex::new(r"(?is)<cite[^>]*>.*?</cite>")
                    .unwrap()
                    .replace_all(&block.inner_html, ""),
            );
            children.clear();
            NodeKind::Testimonial {
                content,
                name: cite,
                job: String::new(),
                image_url: None,
            }
        }
        "video" | "embed" => {
            let raw_url = block
                .attrs
                .get("url")
                .or_else(|| block.attrs.get("src"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| extract_attr_from_html(&block.inner_html, "src"))
                .unwrap_or_else(|| strip_html_tags(&block.inner_html));
            let url = normalize_video_embed_url(&raw_url);
            NodeKind::Video {
                url,
                aspect_ratio: "16/9".to_string(),
                autoplay: false,
                muted: false,
                loop_video: false,
            }
        }
        "code" | "preformatted" | "html" | "table" => NodeKind::CustomHtml {
            raw: block.inner_html.trim().to_string(),
        },
        _ => {
            if !children.is_empty() {
                NodeKind::Container
            } else {
                NodeKind::CustomHtml {
                    raw: block.inner_html.trim().to_string(),
                }
            }
        }
    };

    styles.sync_flat_and_nested();

    Some(NeopressNode {
        id,
        kind,
        styles,
        children,
    })
}

fn extract_gutenberg_styles(attrs: &Value) -> StyleProperties {
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
                top: pad.get("top").and_then(|v| v.as_str()).map(normalize_wp_preset),
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
                top: mar.get("top").and_then(|v| v.as_str()).map(normalize_wp_preset),
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

fn normalize_wp_preset(val: &str) -> String {
    // Converts "var:preset|spacing|50" -> "var(--wp--preset--spacing--50)"
    if let Some(rest) = val.strip_prefix("var:preset|") {
        let parts: Vec<&str> = rest.split('|').collect();
        if parts.len() == 2 {
            return format!("var(--wp--preset--{}--{})", parts[0], parts[1]);
        }
    }
    val.to_string()
}

fn extract_attr_from_html(html: &str, attr: &str) -> Option<String> {
    let pattern = format!(r#"{}\s*=\s*["']([^"']*)["']"#, regex::escape(attr));
    let re = Regex::new(&pattern).ok()?;
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

fn detect_heading_level(html: &str) -> Option<u8> {
    let re = Regex::new(r"(?i)<h([1-6])\b").ok()?;
    re.captures(html)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u8>().ok())
}

fn strip_outer_heading_tag(html: &str) -> String {
    let re = Regex::new(r"(?is)^\s*<h[1-6][^>]*>(.*?)</h[1-6]>\s*$").unwrap();
    if let Some(caps) = re.captures(html) {
        if let Some(inner) = caps.get(1) {
            return strip_html_tags(inner.as_str());
        }
    }
    strip_html_tags(html)
}

fn extract_tag_inner(html: &str, tag: &str) -> Option<String> {
    let pattern = format!(r"(?is)<{0}[^>]*>(.*?)</{0}>", regex::escape(tag));
    let re = Regex::new(&pattern).ok()?;
    re.captures(html)
        .and_then(|c| c.get(1))
        .map(|m| strip_html_tags(m.as_str()))
}

fn strip_html_tags(html: &str) -> String {
    let re = Regex::new(r"<[^>]+>").unwrap();
    re.replace_all(html, "").trim().to_string()
}
