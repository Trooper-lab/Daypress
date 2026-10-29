use super::RenderArgs;
use crate::render::forms::{render_login_form_html, render_search_form_html};
use crate::render::sanitizer::{escape_attr, with_extra_classes};
use crate::schema::{NavMenuItem, NodeKind};

pub(crate) fn render_dynamic(args: &RenderArgs) -> Option<String> {
    let RenderArgs {
        node,
        inner_html,
        css,
        extra_cls,
        style_attr,
        ..
    } = *args;

    match &node.kind {
        NodeKind::NavMenu {
            menu_slug,
            layout,
            align,
            mobile_hamburger,
            items,
        } => {
            let hamburger_html = if *mobile_hamburger {
                "<button type=\"button\" class=\"dp-menu-toggle\" aria-label=\"Menu\" onclick=\"this.nextElementSibling.classList.toggle('is-open');\"><span></span><span></span><span></span></button>\n"
            } else {
                ""
            };
            let mut list_html = format!(
                "<ul class=\"dp-nav-menu-list dp-nav-layout-{} dp-nav-align-{}\">\n",
                escape_attr(layout),
                escape_attr(align)
            );
            list_html.push_str(&render_nav_items_recursive(items));
            list_html.push_str("</ul>");

            let cls = with_extra_classes("dp-nav-menu", extra_cls);
            Some(format!(
                "<nav id=\"{}\" class=\"{}\" data-menu=\"{}\"{}>\n{}{}\n</nav>",
                escape_attr(&node.id),
                escape_attr(&cls),
                escape_attr(menu_slug),
                style_attr,
                hamburger_html,
                list_html
            ))
        }
        NodeKind::MegaMenuContainer { trigger, width } => {
            let mut style_extra = String::new();
            if let Some(w) = width {
                style_extra = format!("width: {}; ", escape_attr(w));
            }
            let cls = with_extra_classes("dp-megamenu-container", extra_cls);
            Some(format!(
                "<div id=\"{}\" class=\"{}\" data-trigger=\"{}\" style=\"{}{}\">\n{}\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                escape_attr(trigger),
                style_extra,
                css,
                inner_html
            ))
        }
        NodeKind::PopupModal {
            modal_id,
            trigger_selector,
            overlay_bg,
            close_button,
        } => {
            let close_html = if *close_button {
                "<button type=\"button\" class=\"dp-modal-close\" aria-label=\"Close\" onclick=\"this.closest('.dp-popup-modal').style.display='none';\">&times;</button>\n"
            } else {
                ""
            };
            let ov_style = overlay_bg
                .as_ref()
                .map(|bg| format!(" style=\"background-color: {};\"", escape_attr(bg)))
                .unwrap_or_default();
            let trig_attr = trigger_selector
                .as_ref()
                .map(|t| format!(" data-trigger=\"{}\"", escape_attr(t)))
                .unwrap_or_default();
            let cls = with_extra_classes("dp-popup-modal", extra_cls);
            Some(format!(
                "<div id=\"{}\" class=\"{}\" data-modal-id=\"{}\"{}{} style=\"display:none;\">\n<div class=\"dp-modal-backdrop\"{}></div>\n<div class=\"dp-modal-dialog\"{}>\n{}{}\n</div>\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                escape_attr(modal_id),
                trig_attr,
                style_attr,
                ov_style,
                style_attr,
                close_html,
                inner_html
            ))
        }
        NodeKind::LoopGrid {
            post_type,
            posts_per_page,
            columns,
            show_thumbnail,
            show_excerpt,
            show_meta,
            show_pagination,
            taxonomy,
        } => {
            let tax_attr = taxonomy
                .as_ref()
                .map(|t| format!(" data-taxonomy=\"{}\"", escape_attr(t)))
                .unwrap_or_default();
            let cls =
                with_extra_classes(&format!("dp-loop-grid dp-grid-cols-{}", columns), extra_cls);
            Some(format!(
                "<div id=\"{}\" class=\"{}\" data-post-type=\"{}\" data-posts-per-page=\"{}\" data-show-thumb=\"{}\" data-show-excerpt=\"{}\" data-show-meta=\"{}\" data-show-pagination=\"{}\"{}{}>\n<div class=\"dp-loop-posts\"></div>\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                escape_attr(post_type),
                posts_per_page,
                show_thumbnail,
                show_excerpt,
                show_meta,
                show_pagination,
                tax_attr,
                style_attr
            ))
        }
        NodeKind::TableOfContents {
            title,
            min_depth,
            max_depth,
            collapsible,
        } => {
            let collapse_btn = if *collapsible {
                "<button type=\"button\" class=\"dp-toc-toggle\" aria-label=\"Toggle\" onclick=\"this.closest('.dp-toc').classList.toggle('is-collapsed');\"><i class=\"fa-solid fa-chevron-down\"></i></button>"
            } else {
                ""
            };
            let cls = with_extra_classes("dp-toc", extra_cls);
            Some(format!(
                "<div id=\"{}\" class=\"{}\" data-min-depth=\"{}\" data-max-depth=\"{}\"{}>\n<div class=\"dp-toc-header\"><h4 class=\"dp-toc-title\">{}</h4>{}</div>\n<div class=\"dp-toc-body\"><ul class=\"dp-toc-list\"></ul></div>\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                min_depth,
                max_depth,
                style_attr,
                escape_attr(title),
                collapse_btn
            ))
        }
        NodeKind::Breadcrumbs {
            separator,
            home_text,
        } => {
            let cls = with_extra_classes("dp-breadcrumbs", extra_cls);
            Some(format!(
                "<nav id=\"{}\" class=\"{}\" aria-label=\"Breadcrumbs\"{}>\n<ol class=\"dp-breadcrumb-list\">\n<li class=\"dp-breadcrumb-item\"><a href=\"/\">{}</a></li>\n<li class=\"dp-breadcrumb-separator\">{}</li>\n<li class=\"dp-breadcrumb-item active\" aria-current=\"page\"><span>Current Page</span></li>\n</ol>\n</nav>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                escape_attr(home_text),
                escape_attr(separator)
            ))
        }
        NodeKind::SearchForm {
            placeholder,
            button_text,
            endpoint,
        } => Some(render_search_form_html(
            &node.id,
            placeholder,
            button_text,
            endpoint,
            style_attr,
        )),
        NodeKind::LoginForm {
            label_username,
            label_password,
            redirect_url,
            remember_me,
        } => Some(render_login_form_html(
            &node.id,
            label_username,
            label_password,
            redirect_url,
            *remember_me,
            style_attr,
        )),
        NodeKind::MenuAnchor { anchor_id } => Some(format!(
            "<div id=\"{}\" class=\"dp-menu-anchor\"></div>",
            escape_attr(anchor_id)
        )),
        _ => None,
    }
}

fn render_nav_items_recursive(items: &[NavMenuItem]) -> String {
    let mut out = String::new();
    for item in items {
        let has_children = !item.children.is_empty();
        let li_class = if has_children {
            "menu-item menu-item-has-children dp-nav-item"
        } else {
            "menu-item dp-nav-item"
        };
        let target_attr = if !item.target.is_empty() {
            format!(r#" target="{}""#, escape_attr(&item.target))
        } else {
            String::new()
        };
        out.push_str(&format!(
            r#"<li class="{}"><a href="{}" class="dp-nav-link"{}>{}</a>"#,
            li_class,
            escape_attr(&item.url),
            target_attr,
            escape_attr(&item.title)
        ));
        if has_children {
            out.push_str(r#"<ul class="sub-menu">"#);
            out.push_str(&render_nav_items_recursive(&item.children));
            out.push_str("</ul>");
        }
        out.push_str("</li>\n");
    }
    out
}
