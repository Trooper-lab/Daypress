use crate::divi::context::DiviContext;
use crate::divi::parser::DiviShortcode;
use crate::divi::widgets::escape_html;
use crate::schema::{NeopressNode, NodeKind, Spacing, StyleProperties};

pub fn map_layout_module(
    divi: &DiviShortcode,
    ctx: &DiviContext,
    styles: &mut StyleProperties,
    children: &mut Vec<NeopressNode>,
    id: &str,
) -> Option<NodeKind> {
    match divi.tag.as_str() {
        "et_pb_section" | "et_pb_fullwidth_section" | "et_pb_specialty_section" => {
            // If specialty section has direct column children, wrap them in a centered flex row container
            let has_direct_columns = !children.is_empty()
                && children.iter().all(|c| {
                    matches!(
                        c.kind,
                        NodeKind::Flex { basis: Some(_), .. } | NodeKind::Column
                    )
                });
            if has_direct_columns {
                let mut row_styles = StyleProperties {
                    width: Some("90%".to_string()),
                    max_width: Some("1200px".to_string()),
                    margin: Some(Spacing {
                        top: Some("0px".to_string()),
                        right: Some("auto".to_string()),
                        bottom: Some("0px".to_string()),
                        left: Some("auto".to_string()),
                    }),
                    display: Some("flex".to_string()),
                    flex_direction: Some("row".to_string()),
                    gap: Some("28px".to_string()),
                    align_items: Some("flex-start".to_string()),
                    ..Default::default()
                };
                row_styles.sync_flat_and_nested();

                let wrapped_cols = std::mem::take(children);
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
                for child in children.iter_mut() {
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

            Some(NodeKind::Section)
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
            let has_col_children = !children.is_empty()
                && children.iter().all(|c| {
                    matches!(
                        c.kind,
                        NodeKind::Column | NodeKind::Flex { basis: Some(_), .. }
                    )
                });
            if has_col_children {
                styles.display = Some("flex".to_string());
                styles.flex_direction = Some("row".to_string());
                if styles.gap.is_none() {
                    styles.gap = Some("28px".to_string());
                }
                if styles.align_items.is_none() {
                    let make_equal = divi
                        .attributes
                        .get("make_equal")
                        .map(|v| v == "on")
                        .unwrap_or(false);
                    styles.align_items = Some(if make_equal {
                        "stretch".to_string()
                    } else {
                        "flex-start".to_string()
                    });
                }
            }
            Some(NodeKind::Container)
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
            Some(NodeKind::Flex {
                direction: "column".to_string(),
                justify: "flex-start".to_string(),
                align: "stretch".to_string(),
                wrap: false,
                basis: Some(basis.to_string()),
            })
        }
        _ => None,
    }
}
