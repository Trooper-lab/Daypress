use super::*;
use crate::elementor::context::ElementorContext;
use crate::schema::{NodeKind, StyleProperties, TestimonialSlide};
use serde_json::Value;

pub(crate) fn map_carousel_widget(
    widget_type: &str,
    settings: &Value,
    _styles: &mut StyleProperties,
    ctx: &ElementorContext,
) -> Option<NodeKind> {
    match widget_type {
        "testimonial-carousel" | "reviews" => {
            let mut slides = Vec::new();
            let raw_slides = settings
                .get("slides")
                .or_else(|| settings.get("reviews"))
                .and_then(|v| v.as_array());

            if let Some(arr) = raw_slides {
                for item in arr {
                    let quote = item
                        .get("content")
                        .or_else(|| item.get("slide_content"))
                        .or_else(|| item.get("text"))
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let author = item
                        .get("name")
                        .or_else(|| item.get("author"))
                        .and_then(|v| v.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let role = item
                        .get("title")
                        .or_else(|| item.get("job"))
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
                    let company = item
                        .get("company")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
                    let avatar_url = item
                        .get("image")
                        .and_then(|img| img.get("url"))
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|u| ctx.normalize_url(u));
                    let rating = item
                        .get("rating")
                        .and_then(|v| v.as_f64().map(|f| f as f32));

                    slides.push(TestimonialSlide {
                        quote,
                        author,
                        role,
                        company,
                        avatar_url,
                        rating,
                    });
                }
            }

            let skin = settings
                .get("skin")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let autoplay = settings
                .get("autoplay")
                .and_then(|v| v.as_str().map(|s| s == "yes").or_else(|| v.as_bool()))
                .unwrap_or(true);
            let loop_slides = settings
                .get("infinite")
                .or_else(|| settings.get("loop"))
                .and_then(|v| v.as_str().map(|s| s == "yes").or_else(|| v.as_bool()))
                .unwrap_or(true);

            Some(NodeKind::TestimonialCarousel {
                slides,
                skin,
                autoplay,
                loop_slides,
            })
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
                        skin: Some("cover".to_string()),
                        ribbon: None,
                        image_url: first
                            .get("background_image")
                            .and_then(|i| i.get("url"))
                            .and_then(|u| u.as_str())
                            .map(|s| ctx.normalize_url(s)),
                        secondary_button_text: None,
                        secondary_button_url: None,
                    });
                }
            }
            Some(NodeKind::CallToAction {
                title: String::new(),
                description: String::new(),
                button_text: "Learn More".to_string(),
                button_url: "#".to_string(),
                skin: None,
                ribbon: None,
                image_url: None,
                secondary_button_text: None,
                secondary_button_url: None,
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
        _ => None,
    }
}
