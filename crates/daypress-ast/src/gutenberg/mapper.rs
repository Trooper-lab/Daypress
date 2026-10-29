use super::embeds::map_gutenberg_embed;
use super::gallery::map_gutenberg_gallery;
use super::lists::map_gutenberg_list;
use crate::gutenberg::parser::GutenbergBlock;
use crate::gutenberg::styles::{
    detect_heading_level, extract_attr_from_html, extract_gutenberg_styles, strip_html_tags,
    strip_outer_heading_tag,
};
use crate::schema::{BackgroundConfig, DaypressNode, NodeKind, Spacing, StyleProperties};

pub(crate) fn map_gutenberg_block(
    block: &GutenbergBlock,
    counter: &mut usize,
) -> Option<DaypressNode> {
    *counter += 1;
    let short_name = block
        .name
        .strip_prefix("core/")
        .unwrap_or(&block.name)
        .replace(['/', '-'], "_");
    let id = format!("gb_{}_{}", short_name, *counter);
    let mut styles = extract_gutenberg_styles(&block.attrs);

    let mut children: Vec<DaypressNode> = block
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
        "gallery" => {
            let (grid_kind, updated_children) = map_gutenberg_gallery(block, counter, children);
            children = updated_children;
            grid_kind
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
            let img_node = DaypressNode {
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
            let content_col = DaypressNode {
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
            children.clear();
            map_gutenberg_list(block)
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
            let cite = block
                .attrs
                .get("citation")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            if !children.is_empty() {
                if !cite.is_empty() {
                    *counter += 1;
                    children.push(DaypressNode {
                        id: format!("gb_quote_cite_{}", *counter),
                        kind: NodeKind::Text {
                            content_html: format!("<cite>{}</cite>", cite),
                        },
                        styles: StyleProperties::default(),
                        children: Vec::new(),
                    });
                }
                NodeKind::Container
            } else {
                let inner = block.inner_html.trim();
                let html = if !cite.is_empty() && !inner.contains("<cite") {
                    format!(
                        r#"<blockquote class="wp-block-quote">{}<cite>{}</cite></blockquote>"#,
                        inner, cite
                    )
                } else if !inner.starts_with("<blockquote") {
                    format!(
                        r#"<blockquote class="wp-block-quote">{}</blockquote>"#,
                        inner
                    )
                } else {
                    inner.to_string()
                };
                NodeKind::CustomHtml { raw: html }
            }
        }
        "template-part" => {
            let tag = block
                .attrs
                .get("tagName")
                .and_then(|v| v.as_str())
                .unwrap_or("div");
            if tag == "header" || tag == "footer" || tag == "section" {
                NodeKind::Section
            } else {
                NodeKind::Container
            }
        }
        _ if norm_name.starts_with("embed")
            || norm_name.starts_with("core-embed")
            || norm_name == "video" =>
        {
            map_gutenberg_embed(block)
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

    Some(DaypressNode {
        id,
        kind,
        styles,
        children,
    })
}
