use super::RenderArgs;
use crate::render::sanitizer::{escape_attr, with_extra_classes};
use crate::schema::NodeKind;

pub(crate) fn render_layout(args: &RenderArgs) -> Option<String> {
    let RenderArgs {
        node,
        inner_html,
        css,
        extra_cls,
        effective_style_attr,
        is_synthetic_root,
        ..
    } = *args;

    match &node.kind {
        NodeKind::Root => Some(inner_html.to_string()),
        NodeKind::Section => {
            if is_synthetic_root && effective_style_attr.is_empty() {
                return Some(inner_html.to_string());
            }
            let cls = with_extra_classes("np-section", extra_cls);
            Some(format!(
                "<section id=\"{}\" class=\"{}\"{}>\n{}\n</section>",
                escape_attr(&node.id),
                escape_attr(&cls),
                effective_style_attr,
                inner_html
            ))
        }
        NodeKind::Container => {
            if is_synthetic_root && effective_style_attr.is_empty() {
                return Some(inner_html.to_string());
            }
            let is_boxed = extra_cls
                .map(|c| c.split_whitespace().any(|w| w == "np-container-boxed"))
                .unwrap_or(false);
            let body_html = if is_boxed {
                format!("<div class=\"np-container-inner\">\n{}\n</div>", inner_html)
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
                    "np-container np-clickable-container np-lightbox-trigger"
                } else {
                    "np-container np-clickable-container"
                };
                let cls = with_extra_classes(base, extra_cls);
                Some(format!(
                    "<a id=\"{}\" href=\"{}\" class=\"{}\"{}>\n{}\n</a>",
                    escape_attr(&node.id),
                    escape_attr(href),
                    escape_attr(&cls),
                    effective_style_attr,
                    body_html
                ))
            } else {
                let cls = with_extra_classes("np-container", extra_cls);
                Some(format!(
                    "<div id=\"{}\" class=\"{}\"{}>\n{}\n</div>",
                    escape_attr(&node.id),
                    escape_attr(&cls),
                    effective_style_attr,
                    body_html
                ))
            }
        }
        NodeKind::Column => {
            let col_style = if css.contains("flex:") || css.contains("flex-basis:") {
                css.to_string()
            } else if css.is_empty() {
                "flex: 1;".to_string()
            } else {
                format!("flex: 1; {}", css)
            };
            Some(format!(
                "<div id=\"{}\" class=\"np-column\" style=\"{}\">\n{}\n</div>",
                escape_attr(&node.id),
                col_style.trim(),
                inner_html
            ))
        }
        NodeKind::Grid { columns, gap } => {
            let grid_style = format!(
                "display: grid; grid-template-columns: repeat({}, minmax(0, 1fr)); gap: {}px; {}",
                columns, gap, css
            );
            Some(format!(
                "<div id=\"{}\" class=\"np-grid\" style=\"{}\">\n{}\n</div>",
                escape_attr(&node.id),
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
            let wrap_str = if *wrap { "wrap" } else { "nowrap" };
            let basis_str = basis
                .as_ref()
                .map(|b| format!("flex: 1 1 {}; min-width: 0; gap: 16px; ", escape_attr(b)))
                .unwrap_or_default();
            let flex_style = format!(
                "display: flex; flex-direction: {}; justify-content: {}; align-items: {}; flex-wrap: {}; {}{}",
                escape_attr(direction),
                escape_attr(justify),
                escape_attr(align),
                wrap_str,
                basis_str,
                css
            );
            Some(format!(
                "<div id=\"{}\" class=\"np-flex\" style=\"{}\">\n{}\n</div>",
                escape_attr(&node.id),
                flex_style.trim(),
                inner_html
            ))
        }
        _ => None,
    }
}
