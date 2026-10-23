use super::RenderArgs;
use crate::render::sanitizer::{escape_attr, sanitize_editor_html, with_extra_classes};
use crate::schema::NodeKind;

pub(crate) fn render_basic(args: &RenderArgs) -> Option<String> {
    let RenderArgs {
        node,
        css,
        extra_cls,
        style_attr,
        ..
    } = *args;

    match &node.kind {
        NodeKind::Heading { text, level } => {
            let lvl = if *level >= 1 && *level <= 6 {
                *level
            } else {
                2
            };
            let cls = with_extra_classes("np-heading", extra_cls);
            Some(format!(
                "<h{} id=\"{}\" class=\"{}\"{}>{}</h{}>",
                lvl,
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                sanitize_editor_html(text),
                lvl
            ))
        }
        NodeKind::Text { content_html } => {
            let cls = with_extra_classes("np-text", extra_cls);
            Some(format!(
                "<div id=\"{}\" class=\"{}\"{}>{}</div>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                sanitize_editor_html(content_html)
            ))
        }
        NodeKind::Image {
            src,
            alt,
            width,
            height,
            link,
        } => {
            let dim_attrs = match (width, height) {
                (Some(w), Some(h)) => format!(" width=\"{}\" height=\"{}\"", w, h),
                (Some(w), None) => format!(" width=\"{}\"", w),
                (None, Some(h)) => format!(" height=\"{}\"", h),
                _ => String::new(),
            };
            let cls = with_extra_classes("np-image", extra_cls);
            let img_tag = format!(
                "<img src=\"{}\" alt=\"{}\"{} class=\"{}\"{} loading=\"lazy\" />",
                escape_attr(src),
                escape_attr(alt),
                dim_attrs,
                escape_attr(&cls),
                style_attr
            );
            Some(if let Some(href) = link.as_ref().filter(|s| !s.is_empty()) {
                format!(
                    "<a href=\"{}\" class=\"np-image-link\">{}</a>",
                    escape_attr(href),
                    img_tag
                )
            } else {
                img_tag
            })
        }
        NodeKind::Button { text, url, target } => {
            let cls = with_extra_classes("np-button", extra_cls);
            Some(format!(
                "<a href=\"{}\" target=\"{}\" class=\"{}\"{}>{}</a>",
                escape_attr(url),
                escape_attr(target),
                escape_attr(&cls),
                style_attr,
                sanitize_editor_html(text)
            ))
        }
        NodeKind::Spacer { height } => Some(format!(
            "<div id=\"{}\" class=\"np-spacer\" style=\"height: {}px;\"></div>",
            escape_attr(&node.id),
            height
        )),
        NodeKind::Divider { style } => Some(format!(
            "<hr id=\"{}\" class=\"np-divider\" style=\"border-top-style: {}; {}\" />",
            escape_attr(&node.id),
            escape_attr(style),
            css
        )),
        NodeKind::Icon { name, size, color } => {
            let size_style = size
                .map(|s| format!("font-size: {}px; ", s))
                .unwrap_or_default();
            let color_style = color
                .as_ref()
                .filter(|c| !c.is_empty())
                .map(|c| format!("color: {}; ", escape_attr(c)))
                .unwrap_or_default();
            let combined = format!("{}{}{}", size_style, color_style, css);
            let cls = with_extra_classes("np-icon", extra_cls);
            let span_html = format!(
                "<span id=\"{}\" class=\"{}\" data-icon=\"{}\" style=\"{}\"><i class=\"{}\" aria-hidden=\"true\"></i></span>",
                escape_attr(&node.id),
                escape_attr(&cls),
                escape_attr(name),
                combined.trim(),
                escape_attr(name)
            );
            Some(if let Some(href) = node.styles.link_url.as_deref().filter(|s| !s.is_empty()) {
                format!(
                    "<a href=\"{}\" class=\"np-icon-link\">{}</a>",
                    escape_attr(href),
                    span_html
                )
            } else {
                span_html
            })
        }
        _ => None,
    }
}
