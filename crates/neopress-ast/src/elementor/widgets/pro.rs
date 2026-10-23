use super::*;
use crate::elementor::context::ElementorContext;
use crate::schema::{
    NodeKind, PriceFeatureItem, SharePlatformItem, SocialIconItem, StyleProperties,
};
use serde_json::Value;

pub(crate) fn map_pro_widget(
    widget_type: &str,
    settings: &Value,
    _styles: &mut StyleProperties,
    ctx: &ElementorContext,
) -> Option<NodeKind> {
    match widget_type {
        "price-table" | "pricing" => {
            let title = settings
                .get("heading")
                .or_else(|| settings.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("Pro Plan")
                .to_string();
            let currency = settings
                .get("currency_symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("$")
                .to_string();
            let raw_price = settings
                .get("price")
                .and_then(extract_spacing_val)
                .unwrap_or_else(|| "29".to_string());
            let period = settings
                .get("period")
                .and_then(|v| v.as_str())
                .unwrap_or("/ month")
                .to_string();
            let button_text = settings
                .get("button_text")
                .and_then(|v| v.as_str())
                .unwrap_or("Get Started")
                .to_string();
            let button_url =
                resolve_setting_link(settings, "link", ctx).unwrap_or_else(|| "#".to_string());

            let mut features = Vec::new();
            let mut feature_items = Vec::new();
            if let Some(list) = settings.get("features_list").and_then(|v| v.as_array()) {
                for item in list {
                    let txt = item
                        .get("item_text")
                        .or_else(|| item.get("text"))
                        .and_then(|v| v.as_str())
                        .or_else(|| item.as_str())
                        .unwrap_or_default()
                        .to_string();
                    let icon = item
                        .get("selected_item_icon")
                        .and_then(|i| i.get("value"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    features.push(txt.clone());
                    feature_items.push(PriceFeatureItem {
                        text: txt,
                        included: true,
                        icon,
                    });
                }
            }

            let show_ribbon = settings
                .get("show_ribbon")
                .and_then(|v| v.as_str())
                .map(|s| s == "yes")
                .unwrap_or(false);
            let ribbon_title = settings
                .get("ribbon_title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let sub_title = settings
                .get("sub_heading")
                .or_else(|| settings.get("subtitle"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let orig_price = settings
                .get("original_price")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            if show_ribbon || !feature_items.is_empty() || sub_title.is_some() {
                Some(NodeKind::PriceCard {
                    title,
                    subtitle: sub_title,
                    currency: Some(currency),
                    price: raw_price,
                    original_price: orig_price,
                    period: Some(period),
                    badge: ribbon_title,
                    featured: show_ribbon,
                    features: feature_items,
                    button_text,
                    button_url,
                })
            } else {
                let full_price = if raw_price.starts_with('$')
                    || raw_price.starts_with('€')
                    || raw_price.starts_with('£')
                {
                    raw_price
                } else {
                    format!("{}{}", currency, raw_price)
                };
                Some(NodeKind::PriceTable {
                    title,
                    price: full_price,
                    period,
                    features,
                    button_text,
                    button_url,
                })
            }
        }
        "call-to-action" | "cta" => {
            let title = settings
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let description = settings
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let button_text = settings
                .get("button")
                .or_else(|| settings.get("button_text"))
                .and_then(|v| v.as_str())
                .unwrap_or("Learn More")
                .to_string();
            let button_url = resolve_setting_link(settings, "link", ctx)
                .or_else(|| {
                    settings
                        .get("button_url")
                        .and_then(|v| v.as_str())
                        .map(|u| ctx.normalize_url(u))
                })
                .unwrap_or_else(|| "#".to_string());
            let skin = settings
                .get("skin")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let ribbon = settings
                .get("ribbon_title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let image_url = settings
                .get("bg_image")
                .or_else(|| settings.get("graphic_image"))
                .and_then(|img| img.get("url"))
                .and_then(|v| v.as_str())
                .map(|u| ctx.normalize_url(u));
            let secondary_button_text = settings
                .get("secondary_button")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let secondary_button_url = resolve_setting_link(settings, "secondary_link", ctx);

            Some(NodeKind::CallToAction {
                title,
                description,
                button_text,
                button_url,
                skin,
                ribbon,
                image_url,
                secondary_button_text,
                secondary_button_url,
            })
        }
        "flip-box" => {
            let front_icon = settings
                .get("selected_icon_a")
                .and_then(|i| i.get("value"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let front_image = settings
                .get("image_a")
                .and_then(|img| img.get("url"))
                .and_then(|v| v.as_str())
                .map(|u| ctx.normalize_url(u));
            let front_title = settings
                .get("title_text_a")
                .and_then(|v| v.as_str())
                .unwrap_or("Front Title")
                .to_string();
            let front_description = settings
                .get("description_text_a")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let back_title = settings
                .get("title_text_b")
                .and_then(|v| v.as_str())
                .unwrap_or("Back Title")
                .to_string();
            let back_description = settings
                .get("description_text_b")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let button_text = settings
                .get("button_text")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());
            let button_url = resolve_setting_link(settings, "link", ctx);
            let direction = settings
                .get("flip_direction")
                .and_then(|v| v.as_str())
                .unwrap_or("flip_right")
                .to_string();

            Some(NodeKind::FlipBox {
                front_icon,
                front_image,
                front_title,
                front_description,
                back_title,
                back_description,
                button_text,
                button_url,
                direction,
            })
        }
        "countdown" => {
            let target_date = settings
                .get("due_date")
                .and_then(|v| v.as_str())
                .unwrap_or("2026-12-31T23:59:59")
                .to_string();
            let mode = settings
                .get("countdown_type")
                .and_then(|v| v.as_str())
                .unwrap_or("due_date")
                .to_string();
            let show_days = settings
                .get("show_days")
                .and_then(|v| v.as_str())
                .map(|s| s != "no")
                .unwrap_or(true);
            let show_hours = settings
                .get("show_hours")
                .and_then(|v| v.as_str())
                .map(|s| s != "no")
                .unwrap_or(true);
            let show_minutes = settings
                .get("show_minutes")
                .and_then(|v| v.as_str())
                .map(|s| s != "no")
                .unwrap_or(true);
            let show_seconds = settings
                .get("show_seconds")
                .and_then(|v| v.as_str())
                .map(|s| s != "no")
                .unwrap_or(true);
            let expiry_message = settings
                .get("expiry_custom_message")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string());

            Some(NodeKind::Countdown {
                target_date,
                mode,
                show_days,
                show_hours,
                show_minutes,
                show_seconds,
                expiry_message,
            })
        }
        "animated-headline" => {
            let before_text = settings
                .get("before_text")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let words = settings
                .get("rotating_text")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            let after_text = settings
                .get("after_text")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            let animation_style = settings
                .get("animation_type")
                .or_else(|| settings.get("headline_style"))
                .and_then(|v| v.as_str())
                .unwrap_or("typing")
                .to_string();

            Some(NodeKind::AnimatedHeadline {
                before_text,
                words,
                after_text,
                animation_style,
            })
        }
        "share-buttons" => {
            let mut platforms = Vec::new();
            if let Some(list) = settings.get("share_buttons").and_then(|v| v.as_array()) {
                for item in list {
                    let platform = item
                        .get("button")
                        .and_then(|v| v.as_str())
                        .unwrap_or("twitter")
                        .to_string();
                    let label = item
                        .get("text")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());
                    platforms.push(SharePlatformItem {
                        platform,
                        label,
                        url_template: None,
                    });
                }
            }
            let skin = settings
                .get("skin")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let official_colors = settings
                .get("color_type")
                .and_then(|v| v.as_str())
                .map(|s| s == "official")
                .unwrap_or(true);

            Some(NodeKind::ShareButtons {
                platforms,
                skin,
                official_colors,
            })
        }
        "social-icons" => {
            let mut items = Vec::new();
            if let Some(icons) = settings.get("social_icon_list").and_then(|t| t.as_array()) {
                for icon in icons {
                    let icon_name = icon
                        .get("social_icon")
                        .and_then(|i| i.get("value"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("fa-brands fa-facebook")
                        .to_string();
                    let link =
                        resolve_setting_link(icon, "link", ctx).unwrap_or_else(|| "#".to_string());
                    items.push(SocialIconItem {
                        icon: icon_name,
                        link,
                    });
                }
            }
            let skin = settings
                .get("skin")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let color_type = settings
                .get("color_type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(NodeKind::SocialIcons {
                items,
                skin,
                color_type,
            })
        }
        _ => None,
    }
}
