use super::RenderArgs;
use crate::render::forms::{render_login_form_html, render_search_form_html};
use crate::render::sanitizer::{escape_attr, with_extra_classes};
use crate::schema::{DaypressNode, NavMenuItem, NodeKind};

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
            let mut custom_vars = String::new();
            for (k, css_var) in [
                ("hamburger_color", "--dp-hamburger-color"),
                ("hamburger_bg", "--dp-hamburger-bg"),
                ("mobile_menu_bg", "--dp-mobile-menu-bg"),
                ("mobile_menu_text", "--dp-mobile-menu-text"),
            ] {
                if let Some(val) = node
                    .styles
                    .raw_settings
                    .as_ref()
                    .and_then(|m| m.get(k))
                    .and_then(|v| v.as_str())
                {
                    if !val.trim().is_empty() {
                        custom_vars.push_str(&format!("{}:{};", css_var, escape_attr(val.trim())));
                    }
                }
            }
            let merged_style = if custom_vars.is_empty() {
                style_attr.to_string()
            } else if css.is_empty() {
                format!(" style=\"{}\"", custom_vars)
            } else {
                format!(" style=\"{}{}\"", css, custom_vars)
            };
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
            list_html.push_str(&render_nav_items_recursive(items, &node.children));
            list_html.push_str("</ul>");

            let cls = with_extra_classes("dp-nav-menu", extra_cls);
            Some(format!(
                "<nav id=\"{}\" class=\"{}\" data-menu=\"{}\"{}>\n{}{}\n</nav>",
                escape_attr(&node.id),
                escape_attr(&cls),
                escape_attr(menu_slug),
                merged_style,
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
            enable_filter_bar,
            enable_sort,
            enable_search,
            default_sort,
            card_skin,
            meta_fields,
        } => {
            let tax_attr = taxonomy
                .as_ref()
                .map(|t| format!(" data-taxonomy=\"{}\"", escape_attr(t)))
                .unwrap_or_default();
            let meta_fields_attr = if meta_fields.is_empty() {
                String::new()
            } else {
                format!(
                    " data-meta-fields=\"{}\"",
                    escape_attr(&meta_fields.join(","))
                )
            };
            let cls = with_extra_classes(
                &format!(
                    "dp-loop-grid dp-grid-cols-{} dp-loop-skin-{}",
                    columns,
                    escape_attr(card_skin)
                ),
                extra_cls,
            );
            Some(format!(
                "<div id=\"{}\" class=\"{}\" data-post-type=\"{}\" data-posts-per-page=\"{}\" data-show-thumb=\"{}\" data-show-excerpt=\"{}\" data-show-meta=\"{}\" data-show-pagination=\"{}\" data-enable-filter=\"{}\" data-enable-sort=\"{}\" data-enable-search=\"{}\" data-default-sort=\"{}\" data-card-skin=\"{}\"{}{}{}>\n<div class=\"dp-loop-posts\"></div>\n</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                escape_attr(post_type),
                posts_per_page,
                show_thumbnail,
                show_excerpt,
                show_meta,
                show_pagination,
                enable_filter_bar,
                enable_sort,
                enable_search,
                escape_attr(default_sort),
                escape_attr(card_skin),
                tax_attr,
                meta_fields_attr,
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
        NodeKind::LanguageSwitcher {
            display_style,
            show_flags,
            show_names,
            show_codes,
            current_lang,
            links,
        } => {
            let default_links = vec![
                crate::schema::LanguageSwitchLink {
                    code: "en".to_string(),
                    name: "English".to_string(),
                    flag: "🇺🇸".to_string(),
                    url: "/".to_string(),
                    active: current_lang.eq_ignore_ascii_case("en") || current_lang.is_empty(),
                },
                crate::schema::LanguageSwitchLink {
                    code: "de".to_string(),
                    name: "Deutsch".to_string(),
                    flag: "🇩🇪".to_string(),
                    url: "/de/".to_string(),
                    active: current_lang.eq_ignore_ascii_case("de"),
                },
                crate::schema::LanguageSwitchLink {
                    code: "nl".to_string(),
                    name: "Nederlands".to_string(),
                    flag: "🇳🇱".to_string(),
                    url: "/nl/".to_string(),
                    active: current_lang.eq_ignore_ascii_case("nl"),
                },
            ];
            let effective_links = if links.is_empty() {
                &default_links
            } else {
                links
            };
            let cls = with_extra_classes(
                &format!(
                    "dp-language-switcher dp-lang-switcher dp-lang-style-{}",
                    escape_attr(display_style)
                ),
                extra_cls,
            );

            if display_style == "dropdown" {
                let mut opts = String::new();
                for item in effective_links {
                    let is_active = item.active || item.code.eq_ignore_ascii_case(current_lang);
                    let mut parts = Vec::new();
                    if *show_flags && !item.flag.is_empty() {
                        parts.push(escape_attr(&item.flag));
                    }
                    if *show_names && !item.name.is_empty() {
                        parts.push(escape_attr(&item.name));
                    }
                    if *show_codes || parts.is_empty() {
                        parts.push(escape_attr(&item.code.to_uppercase()));
                    }
                    let label = parts.join(" ");
                    let sel = if is_active { " selected" } else { "" };
                    opts.push_str(&format!(
                        "<option value=\"{}\" data-lang=\"{}\"{}>{}</option>",
                        escape_attr(&item.url),
                        escape_attr(&item.code),
                        sel,
                        label
                    ));
                }
                Some(format!(
                    "<div id=\"{}\" class=\"{}\" data-current-lang=\"{}\"{}><select class=\"dp-lang-select\" aria-label=\"Select language\" onchange=\"if(this.value)window.location.href=this.value;\">{}</select></div>",
                    escape_attr(&node.id),
                    escape_attr(&cls),
                    escape_attr(current_lang),
                    style_attr,
                    opts
                ))
            } else {
                let mut pills = String::new();
                for item in effective_links {
                    let is_active = item.active || item.code.eq_ignore_ascii_case(current_lang);
                    let mut parts = Vec::new();
                    if *show_flags && !item.flag.is_empty() {
                        parts.push(format!(
                            "<span class=\"dp-lang-flag\" aria-hidden=\"true\">{}</span>",
                            escape_attr(&item.flag)
                        ));
                    }
                    if *show_names && !item.name.is_empty() {
                        parts.push(format!(
                            "<span class=\"dp-lang-name\">{}</span>",
                            escape_attr(&item.name)
                        ));
                    }
                    if *show_codes || parts.is_empty() {
                        parts.push(format!(
                            "<span class=\"dp-lang-code\">{}</span>",
                            escape_attr(&item.code.to_uppercase())
                        ));
                    }
                    let inner = parts.join(" ");
                    let active_cls = if is_active { " is-active" } else { "" };
                    let aria_cur = if is_active {
                        " aria-current=\"true\""
                    } else {
                        ""
                    };
                    pills.push_str(&format!(
                        "<a href=\"{}\" hreflang=\"{}\" lang=\"{}\" class=\"dp-lang-pill{}\"{}>{}</a>",
                        escape_attr(&item.url),
                        escape_attr(&item.code),
                        escape_attr(&item.code),
                        active_cls,
                        aria_cur,
                        inner
                    ));
                }
                Some(format!(
                    "<nav id=\"{}\" class=\"{}\" aria-label=\"Language Switcher\" data-current-lang=\"{}\"{}>{}</nav>",
                    escape_attr(&node.id),
                    escape_attr(&cls),
                    escape_attr(current_lang),
                    style_attr,
                    pills
                ))
            }
        }
        NodeKind::SymbolInstance { symbol_id, .. } => {
            let cls = with_extra_classes("dp-symbol-instance", extra_cls);
            Some(format!(
                "<div id=\"{}\" class=\"{}\" data-symbol-id=\"{}\"{}>{}</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                symbol_id,
                style_attr,
                inner_html
            ))
        }
        _ => None,
    }
}

fn render_nav_items_recursive(items: &[NavMenuItem], nav_children: &[DaypressNode]) -> String {
    let mut out = String::new();
    for item in items {
        let has_children = !item.children.is_empty();
        let inline_mega_node = item
            .mega_menu_node_id
            .as_deref()
            .filter(|id| !id.trim().is_empty())
            .and_then(|id| nav_children.iter().find(|c| c.id == id));

        let resolved_mega_html: Option<String> = if let Some(mega_node) = inline_mega_node {
            let rendered = super::render_node_to_html(mega_node);
            if !rendered.trim().is_empty() {
                Some(rendered)
            } else {
                item.mega_menu_html
                    .as_ref()
                    .filter(|s| !s.trim().is_empty())
                    .cloned()
            }
        } else {
            item.mega_menu_html
                .as_ref()
                .filter(|s| !s.trim().is_empty())
                .cloned()
        };

        let has_mega = resolved_mega_html.is_some()
            || item.mega_menu_template_id.unwrap_or(0) > 0
            || inline_mega_node.is_some();

        let li_class = if has_mega {
            "menu-item menu-item-has-children menu-item-has-megamenu dp-nav-item"
        } else if has_children {
            "menu-item menu-item-has-children dp-nav-item"
        } else {
            "menu-item dp-nav-item"
        };
        let target_attr = if !item.target.is_empty() {
            format!(r#" target="{}""#, escape_attr(&item.target))
        } else {
            String::new()
        };
        let icon_html = item
            .icon
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(|ic| {
                format!(
                    r#"<i class="{} dp-nav-icon" aria-hidden="true"></i> "#,
                    escape_attr(ic)
                )
            })
            .unwrap_or_default();
        let badge_html = item
            .badge
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(|b| format!(r#" <span class="dp-nav-badge">{}</span>"#, escape_attr(b)))
            .unwrap_or_default();
        let desc_html = item
            .description
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(|d| format!(r#"<span class="dp-nav-desc">{}</span>"#, escape_attr(d)))
            .unwrap_or_default();
        let caret_html = if has_mega || has_children {
            r#" <i class="fa-solid fa-chevron-down dp-nav-caret" aria-hidden="true"></i>"#
        } else {
            ""
        };
        out.push_str(&format!(
            r#"<li class="{}"><a href="{}" class="dp-nav-link"{}>{}{}{}{}{}</a>"#,
            li_class,
            escape_attr(&item.url),
            target_attr,
            icon_html,
            escape_attr(&item.title),
            badge_html,
            caret_html,
            desc_html
        ));
        if let Some(ref mega_html) = resolved_mega_html {
            let width_style = item
                .mega_menu_width
                .as_deref()
                .filter(|w| !w.trim().is_empty())
                .map(|w| format!(r#" style="width: {}; max-width: 94vw;""#, escape_attr(w)))
                .unwrap_or_default();
            let node_id_attr = item
                .mega_menu_node_id
                .as_deref()
                .filter(|id| !id.trim().is_empty())
                .map(|id| format!(r#" data-mega-node-id="{}""#, escape_attr(id)))
                .unwrap_or_default();
            out.push_str(&format!(
                r#"<div class="dp-megamenu-dropdown" data-mega-template-id="{}"{}{}>{}</div>"#,
                item.mega_menu_template_id.unwrap_or(0),
                node_id_attr,
                width_style,
                mega_html
            ));
        } else if has_children {
            out.push_str(r#"<ul class="sub-menu">"#);
            out.push_str(&render_nav_items_recursive(&item.children, nav_children));
            out.push_str("</ul>");
        }
        out.push_str("</li>\n");
    }
    out
}
