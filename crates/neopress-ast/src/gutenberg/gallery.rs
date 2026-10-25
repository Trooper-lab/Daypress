use crate::gutenberg::parser::GutenbergBlock;
use crate::gutenberg::styles::extract_attr_from_html;
use crate::schema::{NeopressNode, NodeKind, StyleProperties};
use regex::Regex;

pub(crate) fn map_gutenberg_gallery(
    block: &GutenbergBlock,
    counter: &mut usize,
    mut children: Vec<NeopressNode>,
) -> (NodeKind, Vec<NeopressNode>) {
    let mut columns = block
        .attrs
        .get("columns")
        .and_then(|v| v.as_u64())
        .map(|c| c as u8)
        .unwrap_or(3);

    let link_to_media = block
        .attrs
        .get("linkTo")
        .and_then(|v| v.as_str())
        .map(|s| s == "media")
        .unwrap_or(false);

    let gap = block
        .attrs
        .get("style")
        .and_then(|s| s.get("spacing"))
        .and_then(|sp| sp.get("blockGap"))
        .and_then(|g| g.as_str())
        .and_then(|s| {
            let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
            digits.parse::<u32>().ok()
        })
        .unwrap_or(16);

    // If no AST children were parsed from block.children, extract <img> tags from legacy inner_html
    if children.is_empty() && !block.inner_html.is_empty() {
        let img_tag_re =
            Regex::new(r"(?is)<figure[^>]*>.*?<img(?P<attrs>[^>]+)>.*?</figure>").unwrap();
        let fallback_img_re = Regex::new(r"(?is)<img(?P<attrs>[^>]+)>").unwrap();

        let mut matched_attrs = Vec::new();
        for caps in img_tag_re.captures_iter(&block.inner_html) {
            if let Some(m) = caps.name("attrs") {
                matched_attrs.push(m.as_str().to_string());
            }
        }
        if matched_attrs.is_empty() {
            for caps in fallback_img_re.captures_iter(&block.inner_html) {
                if let Some(m) = caps.name("attrs") {
                    matched_attrs.push(m.as_str().to_string());
                }
            }
        }

        for raw_attrs in matched_attrs {
            let src = extract_attr_from_html(&raw_attrs, "src").unwrap_or_default();
            if src.is_empty() {
                continue;
            }
            let alt = extract_attr_from_html(&raw_attrs, "alt").unwrap_or_default();
            let width =
                extract_attr_from_html(&raw_attrs, "width").and_then(|w| w.parse::<u32>().ok());
            let height =
                extract_attr_from_html(&raw_attrs, "height").and_then(|h| h.parse::<u32>().ok());
            let link = if link_to_media {
                Some(src.clone())
            } else {
                None
            };

            *counter += 1;
            children.push(NeopressNode {
                id: format!("gb_gallery_img_{}", *counter),
                kind: NodeKind::Image {
                    src,
                    alt,
                    width,
                    height,
                    link,
                },
                styles: StyleProperties::default(),
                children: Vec::new(),
            });
        }
    } else if link_to_media {
        for child in &mut children {
            if let NodeKind::Image { src, link, .. } = &mut child.kind {
                if link.is_none() {
                    *link = Some(src.clone());
                }
            }
        }
    }

    if !children.is_empty() {
        columns = columns.min(children.len() as u8).max(1);
    }

    (NodeKind::Grid { columns, gap }, children)
}
