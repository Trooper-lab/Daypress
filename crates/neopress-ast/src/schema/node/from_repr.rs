use super::kind::NodeKind;
use super::repr::NodeKindRepr;

impl From<NodeKindRepr> for NodeKind {
    fn from(r: NodeKindRepr) -> Self {
        match r {
            NodeKindRepr::Root => NodeKind::Root,
            NodeKindRepr::Section => NodeKind::Section,
            NodeKindRepr::Container => NodeKind::Container,
            NodeKindRepr::Column => NodeKind::Column,
            NodeKindRepr::Grid { columns, gap } => NodeKind::Grid { columns, gap },
            NodeKindRepr::Flex { direction, justify, align, wrap, basis } => NodeKind::Flex { direction, justify, align, wrap, basis },
            NodeKindRepr::Heading { text, level } => NodeKind::Heading { text, level },
            NodeKindRepr::Text { content_html } => NodeKind::Text { content_html },
            NodeKindRepr::Image { src, alt, width, height, link } => NodeKind::Image { src, alt, width, height, link },
            NodeKindRepr::Button { text, url, target } => NodeKind::Button { text, url, target },
            NodeKindRepr::Spacer { height } => NodeKind::Spacer { height },
            NodeKindRepr::Divider { style } => NodeKind::Divider { style },
            NodeKindRepr::Icon { name, size, color } => NodeKind::Icon { name, size, color },
            NodeKindRepr::Form { form_id, title, button_text, endpoint } => NodeKind::Form { form_id, title, button_text, endpoint },
            NodeKindRepr::NativeForm { title, button_text, endpoint } => NodeKind::NativeForm { title, button_text, endpoint },
            NodeKindRepr::PriceCard {
                title, subtitle, currency, price, original_price, period, badge, featured, features, button_text, button_url,
            } => NodeKind::PriceCard {
                title, subtitle, currency, price, original_price, period, badge, featured, features, button_text, button_url,
            },
            NodeKindRepr::PriceTable { title, price, period, features, button_text, button_url } => {
                NodeKind::PriceTable { title, price, period, features, button_text, button_url }
            }
            NodeKindRepr::CallToAction {
                title, description, button_text, button_url, skin, ribbon, image_url, secondary_button_text, secondary_button_url,
            } => NodeKind::CallToAction {
                title, description, button_text, button_url, skin, ribbon, image_url, secondary_button_text, secondary_button_url,
            },
            NodeKindRepr::Testimonial { content, name, job, image_url } => NodeKind::Testimonial { content, name, job, image_url },
            NodeKindRepr::TestimonialCarousel { slides, skin, autoplay, loop_slides } => {
                NodeKind::TestimonialCarousel { slides, skin, autoplay, loop_slides }
            }
            NodeKindRepr::Countdown { target_date, mode, show_days, show_hours, show_minutes, show_seconds, expiry_message } => {
                NodeKind::Countdown { target_date, mode, show_days, show_hours, show_minutes, show_seconds, expiry_message }
            }
            NodeKindRepr::FlipBox {
                front_icon, front_image, front_title, front_description, back_title, back_description, button_text, button_url, direction,
            } => NodeKind::FlipBox {
                front_icon, front_image, front_title, front_description, back_title, back_description, button_text, button_url, direction,
            },
            NodeKindRepr::AnimatedHeadline { before_text, words, after_text, animation_style } => {
                NodeKind::AnimatedHeadline { before_text, words, after_text, animation_style }
            }
            NodeKindRepr::Rating { rating, max_rating, title } => NodeKind::Rating { rating, max_rating, title },
            NodeKindRepr::StarRating { rating, max_rating, title, star_color, star_size } => {
                NodeKind::StarRating { rating, max_rating, title, star_color, star_size }
            }
            NodeKindRepr::ProgressBar { title, percent, display_text, track_color, bar_color, height } => {
                NodeKind::ProgressBar { title, percent, display_text, track_color, bar_color, height }
            }
            NodeKindRepr::Alert { alert_type, title, description, dismissible, icon } => {
                NodeKind::Alert { alert_type, title, description, dismissible, icon }
            }
            NodeKindRepr::Counter { number, prefix, suffix, title } => NodeKind::Counter { number, prefix, suffix, title },
            NodeKindRepr::Benchmark { label, value, comparison } => NodeKind::Benchmark { label, value, comparison },
            NodeKindRepr::SocialIcons { items, skin, color_type } => NodeKind::SocialIcons { items, skin, color_type },
            NodeKindRepr::ShareButtons { platforms, skin, official_colors } => NodeKind::ShareButtons { platforms, skin, official_colors },
            NodeKindRepr::NavMenu { menu_slug, layout, align, mobile_hamburger, items } => {
                NodeKind::NavMenu { menu_slug, layout, align, mobile_hamburger, items }
            }
            NodeKindRepr::MegaMenuContainer { trigger, width } => NodeKind::MegaMenuContainer { trigger, width },
            NodeKindRepr::PopupModal { modal_id, trigger_selector, overlay_bg, close_button } => {
                NodeKind::PopupModal { modal_id, trigger_selector, overlay_bg, close_button }
            }
            NodeKindRepr::LoopGrid {
                post_type, posts_per_page, columns, show_thumbnail, show_excerpt, show_meta, show_pagination, taxonomy,
            } => NodeKind::LoopGrid {
                post_type, posts_per_page, columns, show_thumbnail, show_excerpt, show_meta, show_pagination, taxonomy,
            },
            NodeKindRepr::TableOfContents { title, min_depth, max_depth, collapsible } => {
                NodeKind::TableOfContents { title, min_depth, max_depth, collapsible }
            }
            NodeKindRepr::Breadcrumbs { separator, home_text } => NodeKind::Breadcrumbs { separator, home_text },
            NodeKindRepr::SearchForm { placeholder, button_text, endpoint } => NodeKind::SearchForm { placeholder, button_text, endpoint },
            NodeKindRepr::LoginForm { label_username, label_password, redirect_url, remember_me } => {
                NodeKind::LoginForm { label_username, label_password, redirect_url, remember_me }
            }
            NodeKindRepr::Accordion { items } => NodeKind::Accordion { items },
            NodeKindRepr::Tabs { items } => NodeKind::Tabs { items },
            NodeKindRepr::Toggle { items } => NodeKind::Toggle { items },
            NodeKindRepr::Video { url, aspect_ratio, autoplay, muted, loop_video } => {
                NodeKind::Video { url, aspect_ratio, autoplay, muted, loop_video }
            }
            NodeKindRepr::Map { address, zoom } => NodeKind::Map { address, zoom },
            NodeKindRepr::IconBox { icon, title, description, link } => NodeKind::IconBox { icon, title, description, link },
            NodeKindRepr::ImageBox { image_url, title, description, link } => NodeKind::ImageBox { image_url, title, description, link },
            NodeKindRepr::IconList { items } => NodeKind::IconList { items },
            NodeKindRepr::Audio { url } => NodeKind::Audio { url },
            NodeKindRepr::MenuAnchor { anchor_id } => NodeKind::MenuAnchor { anchor_id },
            NodeKindRepr::CustomHtml { raw } => NodeKind::CustomHtml { raw },
            NodeKindRepr::ShortcodeFallback { tag, attributes, content } => NodeKind::ShortcodeFallback { tag, attributes, content },
        }
    }
}
