use super::RenderArgs;
use crate::daystudio::normalize_video_embed_url;
use crate::render::sanitizer::{escape_attr, with_extra_classes};
use crate::schema::NodeKind;

fn url_encode_query(input: &str) -> String {
    let mut encoded = String::with_capacity(input.len() * 2);
    for b in input.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push('+'),
            _ => encoded.push_str(&format!("%{:02X}", b)),
        }
    }
    encoded
}

pub(crate) fn render_interactive(args: &RenderArgs) -> Option<String> {
    let RenderArgs {
        node,
        css,
        extra_cls,
        style_attr,
        ..
    } = *args;

    match &node.kind {
        NodeKind::Accordion { items } => {
            let mut acc_html = String::new();
            for item in items {
                acc_html.push_str(&format!(
                    "<details class=\"dp-accordion-item\">\n<summary>{}</summary>\n<div class=\"dp-accordion-body\">{}</div>\n</details>\n",
                    item.title, item.content_html
                ));
            }
            Some(format!(
                "<div id=\"{}\" class=\"dp-accordion\"{}>\n{}\n</div>",
                escape_attr(&node.id),
                style_attr,
                acc_html
            ))
        }
        NodeKind::Video {
            url,
            aspect_ratio,
            autoplay,
            muted,
            loop_video,
        } => {
            let ratio = if aspect_ratio.trim().is_empty() {
                "16/9"
            } else {
                aspect_ratio.as_str()
            };
            let mut embed_url = normalize_video_embed_url(url);
            let mut params = Vec::new();
            if *autoplay {
                params.push("autoplay=1");
            }
            if *muted {
                params.push("mute=1");
            }
            if *loop_video {
                params.push("loop=1");
            }
            if !params.is_empty() && !embed_url.is_empty() {
                let sep = if embed_url.contains('?') { "&" } else { "?" };
                embed_url.push_str(sep);
                embed_url.push_str(&params.join("&"));
            }
            Some(format!(
                "<div id=\"{}\" class=\"dp-video-embed\" style=\"aspect-ratio: {}; {}\">\n<iframe src=\"{}\" frameborder=\"0\" allow=\"autoplay; encrypted-media; picture-in-picture\" allowfullscreen></iframe>\n</div>",
                escape_attr(&node.id),
                escape_attr(ratio),
                css,
                escape_attr(&embed_url)
            ))
        }
        NodeKind::Map { address, zoom } => {
            let z = zoom.unwrap_or(14);
            let encoded = url_encode_query(address);
            Some(format!(
                "<div id=\"{}\" class=\"dp-map\" data-address=\"{}\" data-zoom=\"{}\"{}>\n<iframe src=\"https://maps.google.com/maps?q={}&amp;t=m&amp;z={}&amp;output=embed&amp;iwloc=near\" width=\"100%\" height=\"400\" style=\"border:0;\" allowfullscreen loading=\"lazy\" referrerpolicy=\"no-referrer-when-downgrade\"></iframe>\n</div>",
                escape_attr(&node.id),
                escape_attr(address),
                z,
                style_attr,
                encoded,
                z
            ))
        }
        NodeKind::Testimonial {
            content,
            name,
            job,
            image_url,
        } => {
            let img_html = image_url
                .as_ref()
                .filter(|u| !u.is_empty())
                .map(|u| {
                    format!(
                        "<img src=\"{}\" alt=\"{}\" class=\"dp-testimonial-avatar\" />",
                        escape_attr(u),
                        escape_attr(name)
                    )
                })
                .unwrap_or_default();
            Some(format!(
                "<blockquote id=\"{}\" class=\"dp-testimonial\"{}>\n<p class=\"dp-testimonial-content\">\"{}\"</p>\n<div class=\"dp-testimonial-meta\">\n{}\n<div><cite class=\"dp-testimonial-name\">{}</cite><span class=\"dp-testimonial-job\">{}</span></div>\n</div>\n</blockquote>",
                escape_attr(&node.id),
                style_attr,
                content,
                img_html,
                name,
                job
            ))
        }
        NodeKind::TestimonialCarousel {
            slides,
            skin,
            autoplay,
            loop_slides,
        } => {
            let mut slides_html = String::new();
            for (idx, slide) in slides.iter().enumerate() {
                let avatar_html = slide
                    .avatar_url
                    .as_ref()
                    .map(|img| format!("<img src=\"{}\" alt=\"{}\" class=\"dp-testimonial-avatar\" />", escape_attr(img), escape_attr(&slide.author)))
                    .unwrap_or_default();
                let job_str = match (slide.role.as_deref(), slide.company.as_deref()) {
                    (Some(r), Some(c)) if !r.is_empty() && !c.is_empty() => format!("{}, {}", r, c),
                    (Some(r), _) if !r.is_empty() => r.to_string(),
                    (_, Some(c)) if !c.is_empty() => c.to_string(),
                    _ => String::new(),
                };
                slides_html.push_str(&format!(
                    "<div class=\"dp-carousel-slide\" data-index=\"{}\">\n<blockquote class=\"dp-testimonial\">\n<p class=\"dp-testimonial-content\">\"{}\"</p>\n<div class=\"dp-testimonial-meta\">\n{}\n<div><cite class=\"dp-testimonial-name\">{}</cite><span class=\"dp-testimonial-job\">{}</span></div>\n</div>\n</blockquote>\n</div>\n",
                    idx,
                    escape_attr(&slide.quote),
                    avatar_html,
                    escape_attr(&slide.author),
                    escape_attr(&job_str)
                ));
            }

            let skin_str = skin.as_deref().unwrap_or("bubble");
            let cls = with_extra_classes(
                &format!("dp-testimonial-carousel dp-carousel-skin-{}", skin_str),
                extra_cls,
            );
            Some(format!(
                "<div id=\"{}\" class=\"{}\" data-autoplay=\"{}\" data-loop=\"{}\"{}>\n<div class=\"dp-carousel-track\">\n{}</div>\n<div class=\"dp-carousel-arrows\"><button type=\"button\" class=\"dp-carousel-prev\" aria-label=\"Previous\">&lsaquo;</button><button type=\"button\" class=\"dp-carousel-next\" aria-label=\"Next\">&rsaquo;</button></div>\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                autoplay,
                loop_slides,
                style_attr,
                slides_html
            ))
        }
        NodeKind::Tabs { items } => {
            let mut nav_html = String::new();
            let mut content_html = String::new();
            let tab_onclick = "var p=this.closest('.dp-tabs'),idx=Array.from(this.parentNode.children).indexOf(this);p.querySelectorAll('.dp-tab-btn').forEach(function(b,i){b.classList.toggle('active',i===idx);});p.querySelectorAll('.dp-tab-pane').forEach(function(pane,i){pane.classList.toggle('active',i===idx);pane.style.display=i===idx?'block':'none';});";
            for (idx, item) in items.iter().enumerate() {
                let active = if idx == 0 { " active" } else { "" };
                let display_style = if idx == 0 {
                    " style=\"display: block;\""
                } else {
                    " style=\"display: none;\""
                };
                nav_html.push_str(&format!(
                    "<button type=\"button\" class=\"dp-tab-btn{}\" onclick=\"{}\">{}</button>\n",
                    active, tab_onclick, item.title
                ));
                content_html.push_str(&format!(
                    "<div class=\"dp-tab-pane{}\"{}>{}</div>\n",
                    active, display_style, item.content_html
                ));
            }
            Some(format!(
                "<div id=\"{}\" class=\"dp-tabs\"{}>\n<div class=\"dp-tabs-nav\">{}</div>\n<div class=\"dp-tabs-content\">{}</div>\n</div>",
                escape_attr(&node.id),
                style_attr,
                nav_html,
                content_html
            ))
        }
        NodeKind::Toggle { items } => {
            let mut tog_html = String::new();
            for item in items {
                tog_html.push_str(&format!(
                    "<details class=\"dp-toggle-item\">\n<summary>{}</summary>\n<div class=\"dp-toggle-body\">{}</div>\n</details>\n",
                    item.title, item.content_html
                ));
            }
            Some(format!(
                "<div id=\"{}\" class=\"dp-toggle\"{}>\n{}</div>",
                escape_attr(&node.id),
                style_attr,
                tog_html
            ))
        }
        NodeKind::Audio { url } => Some(format!(
            "<div id=\"{}\" class=\"dp-audio-wrapper\"{}>\n<audio controls src=\"{}\"></audio>\n</div>",
            escape_attr(&node.id),
            style_attr,
            escape_attr(url)
        )),
        _ => None,
    }
}
