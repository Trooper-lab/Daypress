use crate::divi::context::{resolve_divi_color, DiviContext};
use crate::divi::parser::DiviShortcode;
use crate::divi::styles::map_divi_icon;
use crate::schema::{NeopressNode, NodeKind, StyleProperties};

pub fn map_content_module(
    divi: &DiviShortcode,
    ctx: &DiviContext,
    styles: &mut StyleProperties,
    children: &mut Vec<NeopressNode>,
) -> Option<NodeKind> {
    match divi.tag.as_str() {
        "et_pb_text" => {
            let raw_html = if !divi.inner_text.trim().is_empty() {
                divi.inner_text.clone()
            } else {
                divi.attributes.get("content").cloned().unwrap_or_default()
            };
            let content_html = ctx.normalize_html(&raw_html);
            Some(NodeKind::Text { content_html })
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
            Some(NodeKind::Heading { text, level })
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
            Some(NodeKind::Image {
                src,
                alt,
                width: None,
                height: None,
                link,
            })
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
            if let Some(ref mut mar) = styles.margin {
                if let Some(ref top_val) = mar.top {
                    if top_val == "-84px" || top_val == "-75px" {
                        mar.top = Some("-54px".to_string());
                    }
                }
            }

            Some(NodeKind::Button { text, url, target })
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
                    children.clear();
                    return Some(NodeKind::ImageBox {
                        image_url: ctx.resolve_divi_attr(img_url),
                        title,
                        description,
                        link,
                    });
                }
            }

            let icon = divi
                .attributes
                .get("font_icon")
                .map(|ic| map_divi_icon(ic))
                .unwrap_or_else(|| "fa-solid fa-bolt".to_string());

            children.clear();
            Some(NodeKind::IconBox {
                icon,
                title,
                description,
                link,
            })
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
            Some(NodeKind::Icon {
                name: icon,
                size,
                color,
            })
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
            Some(NodeKind::Divider { style })
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
            Some(NodeKind::Spacer { height })
        }
        "et_pb_code" | "et_pb_fullwidth_code" => Some(NodeKind::CustomHtml {
            raw: ctx.normalize_html(&divi.inner_text),
        }),
        _ => None,
    }
}
