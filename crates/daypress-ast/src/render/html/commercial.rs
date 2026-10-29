use super::RenderArgs;
use crate::render::sanitizer::{escape_attr, with_extra_classes};
use crate::schema::NodeKind;

pub(crate) fn render_commercial(args: &RenderArgs) -> Option<String> {
    let RenderArgs {
        node,
        extra_cls,
        style_attr,
        ..
    } = *args;

    match &node.kind {
        NodeKind::PriceCard {
            title,
            subtitle,
            currency,
            price,
            original_price,
            period,
            badge,
            featured,
            features,
            button_text,
            button_url,
        } => {
            let badge_html = badge
                .as_ref()
                .map(|b| format!("<div class=\"dp-price-badge\">{}</div>", escape_attr(b)))
                .unwrap_or_default();
            let sub_html = subtitle
                .as_ref()
                .map(|s| format!("<p class=\"dp-price-subtitle\">{}</p>", escape_attr(s)))
                .unwrap_or_default();
            let curr = currency.as_deref().unwrap_or("$");
            let orig_html = original_price
                .as_ref()
                .map(|op| format!("<span class=\"dp-price-original\">{}{}</span>", curr, op))
                .unwrap_or_default();
            let per_html = period
                .as_ref()
                .map(|p| {
                    format!(
                        "<span class=\"dp-price-period\">/ {}</span>",
                        escape_attr(p)
                    )
                })
                .unwrap_or_default();

            let mut feats_html = String::new();
            for f in features {
                let inc_class = if f.included { "included" } else { "excluded" };
                let icon_str = if f.included { "fa-check" } else { "fa-xmark" };
                feats_html.push_str(&format!(
                    "<li class=\"dp-price-feature {}\"><i class=\"fa-solid {}\"></i><span>{}</span></li>\n",
                    inc_class, icon_str, escape_attr(&f.text)
                ));
            }

            let featured_cls = if *featured {
                " dp-price-card-featured"
            } else {
                ""
            };
            let cls = with_extra_classes(&format!("dp-price-card{}", featured_cls), extra_cls);

            Some(format!(
                "<div id=\"{}\" class=\"{}\"{}>\n{}\n<div class=\"dp-price-header\"><h3 class=\"dp-price-title\">{}</h3>{}</div>\n<div class=\"dp-price-amount\">{}{}<span class=\"dp-price-val\">{}</span>{}</div>\n<ul class=\"dp-price-features\">\n{}</ul>\n<a href=\"{}\" class=\"dp-button dp-price-btn\">{}</a>\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                badge_html,
                escape_attr(title),
                sub_html,
                orig_html,
                escape_attr(curr),
                escape_attr(price),
                per_html,
                feats_html,
                escape_attr(button_url),
                escape_attr(button_text)
            ))
        }
        NodeKind::PriceTable {
            title,
            price,
            period,
            features,
            button_text,
            button_url,
        } => {
            let feats_html: String = features
                .iter()
                .map(|f| format!("<li class=\"dp-price-feature\">{}</li>\n", f))
                .collect();
            let period_html = if !period.trim().is_empty() {
                format!(" <span class=\"dp-price-period\">{}</span>", period)
            } else {
                String::new()
            };
            let btn_html = if !button_text.trim().is_empty() {
                format!(
                    "\n<a href=\"{}\" class=\"dp-button dp-price-btn\">{}</a>",
                    escape_attr(button_url),
                    button_text
                )
            } else {
                String::new()
            };
            let cls = with_extra_classes("dp-price-table", extra_cls);
            Some(format!(
                "<div id=\"{}\" class=\"{}\"{}>\n<div class=\"dp-price-header\"><h3 class=\"dp-price-title\">{}</h3></div>\n<div class=\"dp-price-amount\"><span class=\"dp-price-val\">{}</span>{}</div>\n<ul class=\"dp-price-features\">\n{}</ul>{}\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                title,
                price,
                period_html,
                feats_html,
                btn_html
            ))
        }
        NodeKind::CallToAction {
            title,
            description,
            button_text,
            button_url,
            skin: _,
            ribbon,
            image_url,
            secondary_button_text,
            secondary_button_url,
        } => {
            let ribbon_html = ribbon
                .as_ref()
                .map(|r| format!("<div class=\"dp-cta-ribbon\">{}</div>\n", escape_attr(r)))
                .unwrap_or_default();
            let img_html = image_url
                .as_ref()
                .map(|img| {
                    format!(
                        "<div class=\"dp-cta-image\"><img src=\"{}\" alt=\"{}\" loading=\"lazy\" /></div>\n",
                        escape_attr(img),
                        escape_attr(title)
                    )
                })
                .unwrap_or_default();
            let title_html = if !title.trim().is_empty() {
                format!("<h2 class=\"dp-cta-title\">{}</h2>\n", title)
            } else {
                String::new()
            };
            let desc_html = if !description.trim().is_empty() {
                format!("<div class=\"dp-cta-desc\">{}</div>\n", description)
            } else {
                String::new()
            };
            let btn_html = if !button_text.trim().is_empty() {
                format!(
                    "<a href=\"{}\" class=\"dp-button dp-cta-btn\">{}</a>",
                    escape_attr(button_url),
                    button_text
                )
            } else {
                String::new()
            };
            let sec_btn_html = match (secondary_button_text, secondary_button_url) {
                (Some(sec_txt), Some(sec_url)) if !sec_txt.trim().is_empty() => {
                    format!(
                        " <a href=\"{}\" class=\"dp-button dp-button-secondary dp-cta-sec-btn\">{}</a>",
                        escape_attr(sec_url),
                        escape_attr(sec_txt)
                    )
                }
                _ => String::new(),
            };

            let cls = with_extra_classes("dp-cta", extra_cls);
            Some(format!(
                "<div id=\"{}\" class=\"{}\"{}>\n{}{}{}{}<div class=\"dp-cta-buttons\">{}{}\n</div>\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                ribbon_html,
                img_html,
                title_html,
                desc_html,
                btn_html,
                sec_btn_html
            ))
        }
        NodeKind::Countdown {
            target_date,
            mode: _,
            show_days,
            show_hours,
            show_minutes,
            show_seconds,
            expiry_message,
        } => {
            let mut items_html = String::new();
            if *show_days {
                items_html.push_str("<div class=\"dp-countdown-item\"><span class=\"dp-countdown-digit\" data-part=\"days\">00</span><span class=\"dp-countdown-label\">Days</span></div>\n");
            }
            if *show_hours {
                items_html.push_str("<div class=\"dp-countdown-item\"><span class=\"dp-countdown-digit\" data-part=\"hours\">00</span><span class=\"dp-countdown-label\">Hours</span></div>\n");
            }
            if *show_minutes {
                items_html.push_str("<div class=\"dp-countdown-item\"><span class=\"dp-countdown-digit\" data-part=\"minutes\">00</span><span class=\"dp-countdown-label\">Minutes</span></div>\n");
            }
            if *show_seconds {
                items_html.push_str("<div class=\"dp-countdown-item\"><span class=\"dp-countdown-digit\" data-part=\"seconds\">00</span><span class=\"dp-countdown-label\">Seconds</span></div>\n");
            }
            let exp_msg = expiry_message.as_deref().unwrap_or("");
            let cls = with_extra_classes("dp-countdown", extra_cls);
            Some(format!(
                "<div id=\"{}\" class=\"{}\" data-target=\"{}\" data-expiry-msg=\"{}\"{}>\n<div class=\"dp-countdown-wrapper\">\n{}</div>\n<div class=\"dp-countdown-expire\" style=\"display:none;\">{}</div>\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                escape_attr(target_date),
                escape_attr(exp_msg),
                style_attr,
                items_html,
                escape_attr(exp_msg)
            ))
        }
        NodeKind::FlipBox {
            front_icon,
            front_image,
            front_title,
            front_description,
            back_title,
            back_description,
            button_text,
            button_url,
            direction,
        } => {
            let icon_html = front_icon
                .as_ref()
                .map(|ic| {
                    format!(
                        "<div class=\"dp-flipbox-icon\"><i class=\"{}\"></i></div>\n",
                        escape_attr(ic)
                    )
                })
                .unwrap_or_default();
            let img_html = front_image
                .as_ref()
                .map(|img| {
                    format!(
                        "<div class=\"dp-flipbox-image\"><img src=\"{}\" alt=\"{}\" /></div>\n",
                        escape_attr(img),
                        escape_attr(front_title)
                    )
                })
                .unwrap_or_default();
            let btn_html = match (button_text, button_url) {
                (Some(txt), Some(url)) if !txt.is_empty() => {
                    format!(
                        "<a href=\"{}\" class=\"dp-button dp-flipbox-btn\">{}</a>\n",
                        escape_attr(url),
                        escape_attr(txt)
                    )
                }
                _ => String::new(),
            };
            let cls =
                with_extra_classes(&format!("dp-flipbox dp-flipbox-{}", direction), extra_cls);
            Some(format!(
                "<div id=\"{}\" class=\"{}\"{}>\n<div class=\"dp-flipbox-front\">\n{}{}<h3 class=\"dp-flipbox-title\">{}</h3>\n<p class=\"dp-flipbox-desc\">{}</p>\n</div>\n<div class=\"dp-flipbox-back\">\n<h3 class=\"dp-flipbox-title\">{}</h3>\n<p class=\"dp-flipbox-desc\">{}</p>\n{}</div>\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                icon_html,
                img_html,
                escape_attr(front_title),
                escape_attr(front_description),
                escape_attr(back_title),
                escape_attr(back_description),
                btn_html
            ))
        }
        _ => None,
    }
}
