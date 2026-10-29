use super::RenderArgs;
use crate::render::sanitizer::{
    escape_attr, resolve_semantic_tag, sanitize_editor_html, with_extra_classes,
};
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
            let cls = with_extra_classes("dp-heading", extra_cls);
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
            let tag = resolve_semantic_tag(node.styles.html_tag.as_deref(), "div");
            let cls = with_extra_classes("dp-text", extra_cls);
            Some(format!(
                "<{tag} id=\"{}\" class=\"{}\"{}>{}</{tag}>",
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
            let cls = with_extra_classes("dp-image", extra_cls);
            let img_tag = format!(
                "<img id=\"{}\" src=\"{}\" alt=\"{}\"{} class=\"{}\"{} loading=\"lazy\" />",
                escape_attr(&node.id),
                escape_attr(src),
                escape_attr(alt),
                dim_attrs,
                escape_attr(&cls),
                style_attr
            );
            Some(
                if let Some(href) = link.as_ref().filter(|s| !s.is_empty()) {
                    format!(
                        "<a href=\"{}\" class=\"dp-image-link\">{}</a>",
                        escape_attr(href),
                        img_tag
                    )
                } else {
                    img_tag
                },
            )
        }
        NodeKind::Button { text, url, target } => {
            let cls = with_extra_classes("dp-button", extra_cls);
            if node.styles.html_tag.as_deref() == Some("button") && url.is_empty() {
                Some(format!(
                    "<button type=\"button\" class=\"{}\" id=\"{}\"{}>{}</button>",
                    escape_attr(&cls),
                    escape_attr(&node.id),
                    style_attr,
                    sanitize_editor_html(text)
                ))
            } else {
                Some(format!(
                    "<a href=\"{}\" target=\"{}\" class=\"{}\" id=\"{}\"{}>{}</a>",
                    escape_attr(url),
                    escape_attr(target),
                    escape_attr(&cls),
                    escape_attr(&node.id),
                    style_attr,
                    sanitize_editor_html(text)
                ))
            }
        }
        NodeKind::Spacer { height } => Some(format!(
            "<div id=\"{}\" class=\"{}\" style=\"height: {}px; {}\"></div>",
            escape_attr(&node.id),
            escape_attr(&with_extra_classes("dp-spacer", extra_cls)),
            height,
            css
        )),
        NodeKind::Divider { style } => Some(format!(
            "<hr id=\"{}\" class=\"{}\" style=\"border-top-style: {}; {}\" />",
            escape_attr(&node.id),
            escape_attr(&with_extra_classes("dp-divider", extra_cls)),
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
            let cls = with_extra_classes("dp-icon", extra_cls);
            let span_html = format!(
                "<span id=\"{}\" class=\"{}\" data-icon=\"{}\" style=\"{}\"><i class=\"{}\" aria-hidden=\"true\"></i></span>",
                escape_attr(&node.id),
                escape_attr(&cls),
                escape_attr(name),
                combined.trim(),
                escape_attr(name)
            );
            Some(
                if let Some(href) = node.styles.link_url.as_deref().filter(|s| !s.is_empty()) {
                    format!(
                        "<a href=\"{}\" class=\"dp-icon-link\">{}</a>",
                        escape_attr(href),
                        span_html
                    )
                } else {
                    span_html
                },
            )
        }
        _ => None,
    }
}
