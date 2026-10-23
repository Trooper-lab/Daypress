use crate::divi::context::{resolve_divi_color, DiviContext};
use crate::divi::parser::DiviShortcode;
use crate::divi::widgets::escape_html;
use crate::schema::{NodeKind, StyleProperties};

pub fn map_headers_module(
    divi: &DiviShortcode,
    ctx: &DiviContext,
    styles: &mut StyleProperties,
) -> Option<NodeKind> {
    match divi.tag.as_str() {
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

            if video_mp4.is_none() && logo_url.is_none() && !fullscreen && !scroll_down {
                Some(NodeKind::CallToAction {
                    title,
                    description,
                    button_text,
                    button_url,
                    skin: None,
                    ribbon: None,
                    image_url: None,
                    secondary_button_text: None,
                    secondary_button_url: None,
                })
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
                *styles = StyleProperties::default();
                Some(NodeKind::CustomHtml { raw: raw_header })
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

            Some(NodeKind::CallToAction {
                title,
                description,
                button_text,
                button_url,
                skin: None,
                ribbon: None,
                image_url: None,
                secondary_button_text: None,
                secondary_button_url: None,
            })
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
            Some(NodeKind::CustomHtml { raw: nav_html })
        }
        _ => None,
    }
}
