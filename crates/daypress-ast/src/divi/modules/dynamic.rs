use crate::daystudio::normalize_video_embed_url;
use crate::divi::context::DiviContext;
use crate::divi::parser::DiviShortcode;
use crate::divi::widgets::escape_html;
use crate::schema::{DaypressNode, NodeKind, SocialIconItem};

pub fn map_dynamic_module(
    divi: &DiviShortcode,
    ctx: &DiviContext,
    children: &mut Vec<DaypressNode>,
) -> Option<NodeKind> {
    match divi.tag.as_str() {
        "et_pb_video" => {
            let raw_url = divi.attributes.get("src").cloned().unwrap_or_default();
            let url = normalize_video_embed_url(&raw_url);
            Some(NodeKind::Video {
                url,
                aspect_ratio: "16/9".to_string(),
                autoplay: false,
                muted: false,
                loop_video: false,
            })
        }
        "et_pb_audio" => {
            let audio_url = divi
                .attributes
                .get("audio")
                .map(|u| ctx.normalize_url(u))
                .unwrap_or_default();
            let title = divi.attributes.get("title").cloned().unwrap_or_default();
            let artist = divi
                .attributes
                .get("artist_name")
                .cloned()
                .unwrap_or_default();
            let raw = format!(
                "<div class=\"et_pb_audio_module dp-audio\"><div class=\"dp-audio-title\"><strong>{}</strong> - {}</div><audio controls style=\"width:100%\"><source src=\"{}\" type=\"audio/mpeg\" /></audio></div>",
                escape_html(&title),
                escape_html(&artist),
                escape_html(&audio_url)
            );
            children.clear();
            Some(NodeKind::CustomHtml { raw })
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
            Some(NodeKind::Map { address, zoom })
        }
        "et_pb_search" => {
            let placeholder = divi
                .attributes
                .get("placeholder")
                .cloned()
                .unwrap_or_else(|| "Search...".to_string());
            let button_text = divi
                .attributes
                .get("button_text")
                .cloned()
                .unwrap_or_else(|| "Search".to_string());
            Some(NodeKind::SearchForm {
                placeholder,
                button_text,
                endpoint: "/".to_string(),
            })
        }
        "et_pb_login" => {
            let redirect_url = divi
                .attributes
                .get("current_page_redirect_url")
                .map(|u| ctx.normalize_url(u))
                .unwrap_or_else(|| "/wp-admin/".to_string());
            Some(NodeKind::LoginForm {
                label_username: "Username or Email".to_string(),
                label_password: "Password".to_string(),
                redirect_url,
                remember_me: true,
            })
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
            Some(NodeKind::SocialIcons {
                items,
                skin: None,
                color_type: None,
            })
        }
        "et_pb_portfolio" | "et_pb_filterable_portfolio" => {
            let posts_per_page = divi
                .attributes
                .get("posts_number")
                .and_then(|n| n.parse::<u32>().ok())
                .unwrap_or(6);
            let show_pagination = divi
                .attributes
                .get("show_pagination")
                .map(|v| v != "off")
                .unwrap_or(true);
            let taxonomy = divi.attributes.get("include_categories").cloned();
            children.clear();
            Some(NodeKind::LoopGrid {
                post_type: "portfolio".to_string(),
                posts_per_page,
                columns: 3,
                show_thumbnail: true,
                show_excerpt: true,
                show_meta: true,
                show_pagination,
                taxonomy,
            })
        }
        "et_pb_blog" => {
            let posts_per_page = divi
                .attributes
                .get("posts_number")
                .and_then(|n| n.parse::<u32>().ok())
                .unwrap_or(6);
            let show_thumbnail = divi
                .attributes
                .get("show_thumbnail")
                .map(|v| v != "off")
                .unwrap_or(true);
            let show_pagination = divi
                .attributes
                .get("show_pagination")
                .map(|v| v != "off")
                .unwrap_or(true);
            let taxonomy = divi.attributes.get("include_categories").cloned();
            children.clear();
            Some(NodeKind::LoopGrid {
                post_type: "post".to_string(),
                posts_per_page,
                columns: 3,
                show_thumbnail,
                show_excerpt: true,
                show_meta: true,
                show_pagination,
                taxonomy,
            })
        }
        "et_pb_gallery" => Some(NodeKind::CustomHtml {
            raw: "<div class=\"dp-gallery\"></div>".to_string(),
        }),
        "et_pb_slider" | "et_pb_fullwidth_slider" | "et_pb_post_slider" => {
            Some(NodeKind::Container)
        }
        "et_pb_slide" => {
            let title = divi.attributes.get("heading").cloned().unwrap_or_default();
            let description = if !divi.inner_text.trim().is_empty() {
                ctx.normalize_html(divi.inner_text.trim())
            } else {
                divi.attributes.get("content").cloned().unwrap_or_default()
            };
            let button_text = divi
                .attributes
                .get("button_text")
                .cloned()
                .unwrap_or_default();
            let button_url = divi
                .attributes
                .get("button_link")
                .or_else(|| divi.attributes.get("button_url"))
                .map(|u| ctx.resolve_divi_attr(u))
                .unwrap_or_else(|| "#".to_string());
            let image_url = divi
                .attributes
                .get("image")
                .filter(|s| !s.is_empty())
                .map(|u| ctx.resolve_divi_attr(u));

            children.clear();
            Some(NodeKind::CallToAction {
                title,
                description,
                button_text,
                button_url,
                skin: None,
                ribbon: None,
                image_url,
                secondary_button_text: None,
                secondary_button_url: None,
            })
        }
        _ => None,
    }
}
