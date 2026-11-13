use super::RenderArgs;
use crate::render::sanitizer::{escape_attr, resolve_semantic_tag, with_extra_classes};
use crate::schema::NodeKind;
use crate::url_utils::{sanitize_css_value, sanitize_safe_url};

pub(crate) fn render_layout(args: &RenderArgs) -> Option<String> {
    let RenderArgs {
        node,
        inner_html,
        css,
        data_attrs,
        extra_cls,
        effective_style_attr,
        is_synthetic_root,
        ..
    } = *args;

    match &node.kind {
        NodeKind::Root => Some(inner_html.to_string()),
        NodeKind::Section => {
            if is_synthetic_root && effective_style_attr.is_empty() && extra_cls.is_none() {
                return Some(inner_html.to_string());
            }
            let tag = resolve_semantic_tag(node.styles.html_tag.as_deref(), "section");
            let cls = with_extra_classes("dp-section", extra_cls);
            Some(format!(
                "<{tag} id=\"{}\" class=\"{}\"{}>\n{}\n</{tag}>",
                escape_attr(&node.id),
                escape_attr(&cls),
                effective_style_attr,
                inner_html
            ))
        }
        NodeKind::Container => {
            if is_synthetic_root
                && effective_style_attr.is_empty()
                && extra_cls.is_none()
                && node.styles.html_tag.is_none()
            {
                return Some(inner_html.to_string());
            }
            let is_boxed = extra_cls
                .map(|c| c.split_whitespace().any(|w| w == "dp-container-boxed"))
                .unwrap_or(false);
            let body_html = if is_boxed {
                format!("<div class=\"dp-container-inner\">\n{}\n</div>", inner_html)
            } else {
                inner_html.to_string()
            };
            if let Some(href) = node.styles.link_url.as_deref().filter(|s| !s.is_empty()) {
                let lower = href.to_ascii_lowercase();
                let is_img = lower.ends_with(".jpg")
                    || lower.ends_with(".jpeg")
                    || lower.ends_with(".png")
                    || lower.ends_with(".webp")
                    || lower.ends_with(".svg");
                let base = if is_img {
                    "dp-container dp-clickable-container dp-lightbox-trigger"
                } else {
                    "dp-container dp-clickable-container"
                };
                let cls = with_extra_classes(base, extra_cls);
                let safe_href = sanitize_safe_url(href);
                Some(format!(
                    "<a id=\"{}\" href=\"{}\" class=\"{}\"{}>\n{}\n</a>",
                    escape_attr(&node.id),
                    escape_attr(&safe_href),
                    escape_attr(&cls),
                    effective_style_attr,
                    body_html
                ))
            } else {
                let tag = resolve_semantic_tag(node.styles.html_tag.as_deref(), "div");
                let cls = with_extra_classes("dp-container", extra_cls);
                Some(format!(
                    "<{tag} id=\"{}\" class=\"{}\"{}>\n{}\n</{tag}>",
                    escape_attr(&node.id),
                    escape_attr(&cls),
                    effective_style_attr,
                    body_html
                ))
            }
        }
        NodeKind::Column => {
            let tag = resolve_semantic_tag(node.styles.html_tag.as_deref(), "div");
            let col_style = if css.contains("flex:") || css.contains("flex-basis:") {
                css.to_string()
            } else if css.is_empty() {
                "flex: 1;".to_string()
            } else {
                format!("flex: 1; {}", css)
            };
            Some(format!(
                "<{tag} id=\"{}\" class=\"{}\"{} style=\"{}\">\n{}\n</{tag}>",
                escape_attr(&node.id),
                escape_attr(&with_extra_classes("dp-column", extra_cls)),
                data_attrs,
                col_style.trim(),
                inner_html
            ))
        }
        NodeKind::Grid { columns, gap } => {
            let tag = resolve_semantic_tag(node.styles.html_tag.as_deref(), "div");
            let cols_decl = if *columns > 0 && !css.contains("grid-template-columns:") {
                format!(
                    "grid-template-columns: repeat({}, minmax(0, 1fr)); ",
                    columns
                )
            } else {
                String::new()
            };
            let gap_decl = if !css.contains("gap:") {
                format!("gap: {}px; ", gap)
            } else {
                String::new()
            };
            let grid_style = format!("display: grid; {}{}{}", cols_decl, gap_decl, css);
            Some(format!(
                "<{tag} id=\"{}\" class=\"{}\"{} style=\"{}\">\n{}\n</{tag}>",
                escape_attr(&node.id),
                escape_attr(&with_extra_classes("dp-grid", extra_cls)),
                data_attrs,
                grid_style.trim(),
                inner_html
            ))
        }
        NodeKind::Flex {
            direction,
            justify,
            align,
            wrap,
            basis,
        } => {
            let tag = resolve_semantic_tag(node.styles.html_tag.as_deref(), "div");
            let wrap_str = if *wrap { "wrap" } else { "nowrap" };
            let basis_str = basis
                .as_ref()
                .map(|b| {
                    format!(
                        "flex: 1 1 {}; min-width: 0; gap: 16px; ",
                        escape_attr(&sanitize_css_value(b))
                    )
                })
                .unwrap_or_default();
            let flex_style = format!(
                "display: flex; flex-direction: {}; justify-content: {}; align-items: {}; flex-wrap: {}; {}{}",
                escape_attr(&sanitize_css_value(direction)),
                escape_attr(&sanitize_css_value(justify)),
                escape_attr(&sanitize_css_value(align)),
                wrap_str,
                basis_str,
                css
            );
            Some(format!(
                "<{tag} id=\"{}\" class=\"{}\"{} style=\"{}\">\n{}\n</{tag}>",
                escape_attr(&node.id),
                escape_attr(&with_extra_classes("dp-flex", extra_cls)),
                data_attrs,
                flex_style.trim(),
                inner_html
            ))
        }
        _ => None,
    }
}
