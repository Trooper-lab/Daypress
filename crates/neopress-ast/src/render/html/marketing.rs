use super::RenderArgs;
use crate::render::forms::render_form_html;
use crate::render::sanitizer::{escape_attr, with_extra_classes};
use crate::schema::NodeKind;

pub(crate) fn render_marketing(args: &RenderArgs) -> Option<String> {
    let RenderArgs {
        node,
        inner_html,
        extra_cls,
        style_attr,
        ..
    } = *args;

    match &node.kind {
        NodeKind::Form {
            form_id,
            title,
            button_text,
            endpoint,
        } => Some(render_form_html(
            &node.id,
            *form_id,
            title.as_deref(),
            button_text.as_deref(),
            endpoint.as_deref(),
            style_attr,
            inner_html,
        )),
        NodeKind::NativeForm {
            title,
            button_text,
            endpoint,
        } => Some(render_form_html(
            &node.id,
            1,
            Some(title.as_str()),
            Some(button_text.as_str()),
            Some(endpoint.as_str()),
            style_attr,
            inner_html,
        )),
        NodeKind::Benchmark {
            label,
            value,
            comparison,
        } => {
            let val_html = if !value.trim().is_empty() {
                format!("<div class=\"np-benchmark-value\">{}</div>\n", value)
            } else {
                String::new()
            };
            let comp_html = if !comparison.trim().is_empty() {
                format!(
                    "<div class=\"np-benchmark-comparison\">{}</div>",
                    comparison
                )
            } else {
                String::new()
            };
            Some(format!(
                "<div id=\"{}\" class=\"np-benchmark\"{}>\n<div class=\"np-benchmark-label\">{}</div>\n{}{}\n</div>",
                escape_attr(&node.id),
                style_attr,
                label,
                val_html,
                comp_html
            ))
        }
        NodeKind::AnimatedHeadline {
            before_text,
            words,
            after_text,
            animation_style,
        } => {
            let words_json = serde_json::to_string(words).unwrap_or_else(|_| "[]".to_string());
            let first_word = words.first().map(|s| s.as_str()).unwrap_or("");
            let cls = with_extra_classes(
                &format!("np-animated-headline np-anim-{}", animation_style),
                extra_cls,
            );
            Some(format!(
                "<h2 id=\"{}\" class=\"{}\" data-words=\"{}\"{}>\n<span class=\"np-anim-before\">{}</span> <span class=\"np-anim-rotating\">{}</span> <span class=\"np-anim-after\">{}</span>\n</h2>",
                escape_attr(&node.id),
                escape_attr(&cls),
                escape_attr(&words_json),
                style_attr,
                escape_attr(before_text),
                escape_attr(first_word),
                escape_attr(after_text)
            ))
        }
        NodeKind::StarRating {
            rating,
            max_rating,
            title,
            star_color,
            star_size,
        } => {
            let mut stars = String::new();
            let rounded = rating.round() as u8;
            for i in 1..=(*max_rating) {
                if i <= rounded {
                    stars.push_str("<i class=\"fa-solid fa-star\"></i>");
                } else {
                    stars.push_str("<i class=\"fa-regular fa-star\"></i>");
                }
            }
            let title_html = title
                .as_ref()
                .map(|t| format!("<span class=\"np-rating-title\">{}</span>", escape_attr(t)))
                .unwrap_or_default();
            let mut inline_styles = String::new();
            if let Some(col) = star_color {
                inline_styles.push_str(&format!("color: {}; ", escape_attr(col)));
            }
            if let Some(sz) = star_size {
                inline_styles.push_str(&format!("font-size: {}px; ", sz));
            }
            let star_style_attr = if inline_styles.is_empty() {
                String::new()
            } else {
                format!(" style=\"{}\"", inline_styles.trim())
            };
            let cls = with_extra_classes("np-star-rating", extra_cls);
            Some(format!(
                "<div id=\"{}\" class=\"{}\"{}>\n{}{}<span class=\"np-rating-stars\"{}>{}</span> <span class=\"np-rating-score\">({}/{})</span>\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                title_html,
                if title.is_some() { " " } else { "" },
                star_style_attr,
                stars,
                rating,
                max_rating
            ))
        }
        NodeKind::Rating {
            rating,
            max_rating,
            title,
        } => {
            let mut stars = String::new();
            let rounded = rating.round() as u8;
            for i in 1..=(*max_rating) {
                if i <= rounded {
                    stars.push_str("<i class=\"fa-solid fa-star\"></i>");
                } else {
                    stars.push_str("<i class=\"fa-regular fa-star\"></i>");
                }
            }
            let title_html = title
                .as_ref()
                .map(|t| format!("<span class=\"np-rating-title\">{}</span>", t))
                .unwrap_or_default();
            Some(format!(
                "<div id=\"{}\" class=\"np-rating\"{}>\n{}{}<span class=\"np-rating-score\">({}/{})</span>\n</div>",
                escape_attr(&node.id),
                style_attr,
                title_html,
                stars,
                rating,
                max_rating
            ))
        }
        NodeKind::ProgressBar {
            title,
            percent,
            display_text,
            track_color,
            bar_color,
            height,
        } => {
            let pct_label = display_text.as_deref().unwrap_or("");
            let mut track_style = String::new();
            if let Some(tc) = track_color {
                track_style.push_str(&format!("background-color: {}; ", escape_attr(tc)));
            }
            if let Some(h) = height {
                track_style.push_str(&format!("height: {}px; ", h));
            }
            let mut fill_style = format!("width: {}%;", percent);
            if let Some(bc) = bar_color {
                fill_style.push_str(&format!(" background-color: {};", escape_attr(bc)));
            }

            let track_style_attr = if track_style.is_empty() {
                String::new()
            } else {
                format!(" style=\"{}\"", track_style.trim())
            };

            let cls = with_extra_classes("np-progressbar", extra_cls);
            Some(format!(
                "<div id=\"{}\" class=\"{}\"{}>\n<div class=\"np-progress-title\"><span>{}</span><span>{}</span></div>\n<div class=\"np-progress-track\"{}>\n<div class=\"np-progress-fill\" style=\"{}\"></div>\n</div>\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                escape_attr(title),
                escape_attr(pct_label),
                track_style_attr,
                fill_style
            ))
        }
        NodeKind::Alert {
            alert_type,
            title,
            description,
            dismissible,
            icon,
        } => {
            let icon_html = icon
                .as_ref()
                .map(|ic| format!("<i class=\"np-alert-icon {}\"></i> ", escape_attr(ic)))
                .unwrap_or_default();
            let dismiss_btn = if *dismissible {
                "<button type=\"button\" class=\"np-alert-close\" aria-label=\"Close\" onclick=\"this.parentElement.style.display='none'\">&times;</button>"
            } else {
                ""
            };
            let cls = with_extra_classes(&format!("np-alert np-alert-{}", alert_type), extra_cls);
            Some(format!(
                "<div id=\"{}\" class=\"{}\" role=\"alert\"{}>\n{}<strong>{}</strong> <span>{}</span>\n{}\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                icon_html,
                title,
                description,
                dismiss_btn
            ))
        }
        NodeKind::Counter {
            number,
            prefix,
            suffix,
            title,
        } => {
            let pfx = prefix.as_deref().unwrap_or("");
            let sfx = suffix.as_deref().unwrap_or("");
            let title_html = title
                .as_ref()
                .map(|t| format!("<div class=\"np-counter-title\">{}</div>", t))
                .unwrap_or_default();
            Some(format!(
                "<div id=\"{}\" class=\"np-counter\"{}>\n<div class=\"np-counter-val\">{}{}{}</div>\n{}\n</div>",
                escape_attr(&node.id),
                style_attr,
                pfx,
                number,
                sfx,
                title_html
            ))
        }
        NodeKind::SocialIcons {
            items,
            skin,
            color_type,
        } => {
            let skin_str = skin.as_deref().unwrap_or("default");
            let color_str = color_type.as_deref().unwrap_or("official");
            let cls = with_extra_classes(
                &format!(
                    "np-social-icons np-social-skin-{} np-social-{}",
                    skin_str, color_str
                ),
                extra_cls,
            );
            let mut icons_html = String::new();
            for item in items {
                icons_html.push_str(&format!(
                    "<a href=\"{}\" class=\"np-social-icon\" target=\"_blank\" rel=\"noopener\"><i class=\"{}\"></i></a>\n",
                    escape_attr(&item.link),
                    escape_attr(&item.icon)
                ));
            }
            Some(format!(
                "<div id=\"{}\" class=\"{}\"{}>\n{}</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                icons_html
            ))
        }
        NodeKind::ShareButtons {
            platforms,
            skin,
            official_colors,
        } => {
            let skin_str = skin.as_deref().unwrap_or("gradient");
            let col_cls = if *official_colors {
                " np-share-official"
            } else {
                " np-share-custom"
            };
            let cls = with_extra_classes(
                &format!("np-share-buttons np-share-{}{}", skin_str, col_cls),
                extra_cls,
            );
            let mut btns_html = String::new();
            for p in platforms {
                let icon_cls = match p.platform.as_str() {
                    "twitter" | "x" => "fa-brands fa-x-twitter",
                    "facebook" => "fa-brands fa-facebook-f",
                    "linkedin" => "fa-brands fa-linkedin-in",
                    "whatsapp" => "fa-brands fa-whatsapp",
                    "pinterest" => "fa-brands fa-pinterest",
                    "reddit" => "fa-brands fa-reddit-alien",
                    "telegram" => "fa-brands fa-telegram",
                    "email" => "fa-solid fa-envelope",
                    _ => "fa-solid fa-share-nodes",
                };
                let label_text = p.label.as_deref().unwrap_or(&p.platform);
                let fallback_url = format!("#share-{}", p.platform);
                let share_url = p.url_template.as_deref().unwrap_or(&fallback_url);
                btns_html.push_str(&format!(
                    "<a href=\"{}\" class=\"np-share-btn np-share-network-{}\" target=\"_blank\" rel=\"noopener\"><i class=\"{}\"></i><span>{}</span></a>\n",
                    escape_attr(share_url),
                    escape_attr(&p.platform),
                    escape_attr(icon_cls),
                    escape_attr(label_text)
                ));
            }
            Some(format!(
                "<div id=\"{}\" class=\"{}\"{}>\n{}</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                btns_html
            ))
        }
        _ => None,
    }
}
