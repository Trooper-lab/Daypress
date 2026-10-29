use crate::elementor::context::ElementorContext;
use crate::schema::{NodeKind, StyleProperties};
use serde_json::Value;

pub(crate) fn map_dynamic_widget(
    widget_type: &str,
    settings: &Value,
    _styles: &mut StyleProperties,
    ctx: &ElementorContext,
) -> Option<NodeKind> {
    match widget_type {
        "nav-menu" => {
            let menu_slug = settings
                .get("menu")
                .and_then(|v| v.as_str())
                .unwrap_or("primary")
                .to_string();
            let layout = settings
                .get("layout")
                .and_then(|v| v.as_str())
                .unwrap_or("horizontal")
                .to_string();
            let align = settings
                .get("align_items")
                .or_else(|| settings.get("align"))
                .and_then(|v| v.as_str())
                .unwrap_or("left")
                .to_string();
            let mobile_hamburger = settings
                .get("dropdown")
                .and_then(|v| v.as_str())
                .map(|s| s != "none")
                .unwrap_or(true);

            let items = ctx
                .menus
                .get(&menu_slug)
                .cloned()
                .or_else(|| {
                    if menu_slug == "primary" {
                        ctx.menus.values().next().cloned()
                    } else {
                        None
                    }
                })
                .unwrap_or_default();

            Some(NodeKind::NavMenu {
                menu_slug,
                layout,
                align,
                mobile_hamburger,
                items,
            })
        }
        "posts" | "archive-posts" | "loop-grid" | "portfolio" => {
            let post_type = settings
                .get("posts_post_type")
                .and_then(|v| v.as_str())
                .unwrap_or(if widget_type == "portfolio" {
                    "portfolio"
                } else {
                    "post"
                })
                .to_string();
            let posts_per_page = settings
                .get("posts_per_page")
                .and_then(|v| v.as_u64())
                .unwrap_or(6) as u32;
            let columns = settings
                .get("columns")
                .and_then(|v| v.as_u64())
                .unwrap_or(3) as u8;
            let show_thumbnail = settings
                .get("show_image")
                .and_then(|v| v.as_str())
                .map(|s| s != "none")
                .unwrap_or(true);
            let show_excerpt = settings
                .get("show_excerpt")
                .and_then(|v| v.as_str())
                .map(|s| s != "none")
                .unwrap_or(true);
            let show_meta = settings
                .get("show_meta")
                .and_then(|v| v.as_str())
                .map(|s| s != "none")
                .unwrap_or(true);
            let show_pagination = settings
                .get("pagination_type")
                .and_then(|v| v.as_str())
                .map(|s| s != "none")
                .unwrap_or(true);
            let taxonomy = settings
                .get("posts_include_term_ids")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(NodeKind::LoopGrid {
                post_type,
                posts_per_page,
                columns,
                show_thumbnail,
                show_excerpt,
                show_meta,
                show_pagination,
                taxonomy,
            })
        }
        "table-of-contents" => {
            let title = settings
                .get("title")
                .and_then(|v| v.as_str())
                .unwrap_or("Table of Contents")
                .to_string();
            let min_depth = settings
                .get("min_depth")
                .and_then(|v| v.as_u64())
                .unwrap_or(2) as u8;
            let max_depth = settings
                .get("max_depth")
                .and_then(|v| v.as_u64())
                .unwrap_or(4) as u8;
            let collapsible = settings
                .get("minimize_box")
                .and_then(|v| v.as_str())
                .map(|s| s == "yes")
                .unwrap_or(true);

            Some(NodeKind::TableOfContents {
                title,
                min_depth,
                max_depth,
                collapsible,
            })
        }
        "breadcrumbs" => {
            let separator = settings
                .get("separator")
                .and_then(|v| v.as_str())
                .unwrap_or("/")
                .to_string();
            let home_text = settings
                .get("home_title")
                .or_else(|| settings.get("home_text"))
                .and_then(|v| v.as_str())
                .unwrap_or("Home")
                .to_string();

            Some(NodeKind::Breadcrumbs {
                separator,
                home_text,
            })
        }
        "search-form" | "search" => {
            let placeholder = settings
                .get("placeholder")
                .and_then(|v| v.as_str())
                .unwrap_or("Search...")
                .to_string();
            let button_text = settings
                .get("button_text")
                .or_else(|| settings.get("button"))
                .and_then(|v| v.as_str())
                .unwrap_or("Search")
                .to_string();
            let endpoint = settings
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("/")
                .to_string();

            Some(NodeKind::SearchForm {
                placeholder,
                button_text,
                endpoint,
            })
        }
        "login" | "login-form" => {
            let label_username = settings
                .get("user_label")
                .or_else(|| settings.get("label_username"))
                .and_then(|v| v.as_str())
                .unwrap_or("Username or Email Address")
                .to_string();
            let label_password = settings
                .get("password_label")
                .or_else(|| settings.get("label_password"))
                .and_then(|v| v.as_str())
                .unwrap_or("Password")
                .to_string();
            let redirect_url = settings
                .get("redirect_after_login")
                .or_else(|| settings.get("redirect_url"))
                .and_then(|v| v.as_str())
                .unwrap_or("/")
                .to_string();
            let remember_me = settings
                .get("show_remember_me")
                .and_then(|v| v.as_str())
                .map(|s| s == "yes")
                .unwrap_or(true);

            Some(NodeKind::LoginForm {
                label_username,
                label_password,
                redirect_url,
                remember_me,
            })
        }
        "mega-menu" | "mega-menu-container" => {
            let trigger = settings
                .get("trigger")
                .and_then(|v| v.as_str())
                .unwrap_or("hover")
                .to_string();
            let width = settings
                .get("menu_width")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(NodeKind::MegaMenuContainer { trigger, width })
        }
        "popup" | "popup-modal" => {
            let modal_id = settings
                .get("modal_id")
                .or_else(|| settings.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("dp_modal")
                .to_string();
            let trigger_selector = settings
                .get("trigger_selector")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let overlay_bg = settings
                .get("overlay_bg")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let close_button = settings
                .get("close_button")
                .and_then(|v| v.as_str())
                .map(|s| s != "no")
                .unwrap_or(true);

            Some(NodeKind::PopupModal {
                modal_id,
                trigger_selector,
                overlay_bg,
                close_button,
            })
        }
        _ => None,
    }
}
