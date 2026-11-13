use super::RenderArgs;
use crate::render::sanitizer::{
    escape_attr, resolve_semantic_tag, sanitize_editor_html, with_extra_classes,
};
use crate::schema::NodeKind;
use crate::url_utils::{sanitize_css_value, sanitize_media_url, sanitize_safe_url};

pub(crate) fn render_basic(args: &RenderArgs) -> Option<String> {
    let RenderArgs {
        node,
        css,
        data_attrs,
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
            let safe_src = sanitize_media_url(src);
            let img_tag = format!(
                "<img id=\"{}\" src=\"{}\" alt=\"{}\"{} class=\"{}\"{} loading=\"lazy\" />",
                escape_attr(&node.id),
                escape_attr(&safe_src),
                escape_attr(alt),
                dim_attrs,
                escape_attr(&cls),
                style_attr
            );
            Some(
                if let Some(href) = link.as_ref().filter(|s| !s.is_empty()) {
                    let safe_href = sanitize_safe_url(href);
                    format!(
                        "<a href=\"{}\" class=\"dp-image-link\">{}</a>",
                        escape_attr(&safe_href),
                        img_tag
                    )
                } else {
                    img_tag
                },
            )
        }
        NodeKind::Button { text, url, target } => {
            // `html_tag: "a"` marks a plain text link: no button class, so no theme button look.
            let base_class = if node.styles.html_tag.as_deref() == Some("a") {
                "dp-link"
            } else {
                "dp-button"
            };
            let cls = with_extra_classes(base_class, extra_cls);
            let safe_text = sanitize_editor_html(text);
            if node.styles.html_tag.as_deref() == Some("button") && url.is_empty() {
                // A `<button>` has no link target, so `target` carries its type: "submit" and
                // "reset" act on the enclosing form; anything else is a plain button.
                let button_type = match target.as_str() {
                    "submit" => "submit",
                    "reset" => "reset",
                    _ => "button",
                };
                Some(format!(
                    "<button type=\"{}\" class=\"{}\" id=\"{}\"{}>{}</button>",
                    button_type,
                    escape_attr(&cls),
                    escape_attr(&node.id),
                    style_attr,
                    safe_text
                ))
            } else {
                let safe_url = sanitize_safe_url(url);
                let safe_target = match target.as_str() {
                    "_blank" | "_self" | "_parent" | "_top" | "" => target.as_str(),
                    _ => "_self",
                };
                Some(format!(
                    "<a href=\"{}\" target=\"{}\" class=\"{}\" id=\"{}\"{}>{}</a>",
                    escape_attr(&safe_url),
                    escape_attr(safe_target),
                    escape_attr(&cls),
                    escape_attr(&node.id),
                    style_attr,
                    safe_text
                ))
            }
        }
        NodeKind::Spacer { height } => Some(format!(
            "<div id=\"{}\" class=\"{}\"{} style=\"height: {}px; {}\"></div>",
            escape_attr(&node.id),
            escape_attr(&with_extra_classes("dp-spacer", extra_cls)),
            data_attrs,
            height,
            css
        )),
        NodeKind::Divider { style } => Some(format!(
            "<hr id=\"{}\" class=\"{}\"{} style=\"border-top-style: {}; {}\" />",
            escape_attr(&node.id),
            escape_attr(&with_extra_classes("dp-divider", extra_cls)),
            data_attrs,
            escape_attr(&sanitize_css_value(style)),
            css
        )),
        NodeKind::Icon { name, size, color } => {
            let size_style = size
                .map(|s| format!("font-size: {}px; ", s))
                .unwrap_or_default();
            let color_style = color
                .as_ref()
                .filter(|c| !c.is_empty())
                .map(|c| {
                    format!(
                        "color: {}; ",
                        escape_attr(&sanitize_css_value(c))
                    )
                })
                .unwrap_or_default();
            let combined = format!("{}{}{}", size_style, color_style, css);
            let cls = with_extra_classes("dp-icon", extra_cls);
            let span_html = format!(
                "<span id=\"{}\" class=\"{}\" data-icon=\"{}\"{} style=\"{}\"><i class=\"{}\" aria-hidden=\"true\"></i></span>",
                escape_attr(&node.id),
                escape_attr(&cls),
                escape_attr(name),
                data_attrs,
                combined.trim(),
                escape_attr(name)
            );
            Some(
                if let Some(href) = node.styles.link_url.as_deref().filter(|s| !s.is_empty()) {
                    let safe_href = sanitize_safe_url(href);
                    format!(
                        "<a href=\"{}\" class=\"dp-icon-link\">{}</a>",
                        escape_attr(&safe_href),
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
