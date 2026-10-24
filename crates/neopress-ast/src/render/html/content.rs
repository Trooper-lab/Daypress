use super::RenderArgs;
use crate::render::sanitizer::{escape_attr, sanitize_editor_html, with_extra_classes};
use crate::schema::NodeKind;

pub(crate) fn render_content(args: &RenderArgs) -> Option<String> {
    let RenderArgs {
        node,
        extra_cls,
        style_attr,
        ..
    } = *args;

    match &node.kind {
        NodeKind::IconBox {
            icon,
            title,
            description,
            link,
        } => {
            let align_justify = match node.styles.text_align.as_deref() {
                Some("center") => "center",
                Some("right") => "flex-end",
                _ => "flex-start",
            };
            let desc_html = if description.trim().is_empty() {
                String::new()
            } else {
                format!("\n<div class=\"np-iconbox-desc\">{}</div>", description)
            };
            let inner = format!(
                "<div class=\"np-iconbox-icon\" style=\"justify-content: {};\"><i class=\"{}\"></i></div>\n<h3 class=\"np-iconbox-title\">{}</h3>{}",
                align_justify,
                escape_attr(icon),
                title,
                desc_html
            );
            Some(if let Some(href) = link.as_ref().filter(|s| !s.is_empty()) {
                format!(
                    "<a id=\"{}\" href=\"{}\" class=\"np-iconbox\"{}>\n{}\n</a>",
                    escape_attr(&node.id),
                    escape_attr(href),
                    style_attr,
                    inner
                )
            } else {
                format!(
                    "<div id=\"{}\" class=\"np-iconbox\"{}>\n{}\n</div>",
                    escape_attr(&node.id),
                    style_attr,
                    inner
                )
            })
        }
        NodeKind::ImageBox {
            image_url,
            title,
            description,
            link,
        } => {
            let desc_html = if description.trim().is_empty() {
                String::new()
            } else {
                format!("\n<div class=\"np-imagebox-desc\">{}</div>", description)
            };
            let inner = format!(
                "<div class=\"np-imagebox-img\"><img src=\"{}\" alt=\"{}\" loading=\"lazy\" /></div>\n<h3 class=\"np-imagebox-title\">{}</h3>{}",
                escape_attr(image_url),
                escape_attr(title),
                title,
                desc_html
            );
            Some(if let Some(href) = link.as_ref().filter(|s| !s.is_empty()) {
                format!(
                    "<a id=\"{}\" href=\"{}\" class=\"np-imagebox\"{}>\n{}\n</a>",
                    escape_attr(&node.id),
                    escape_attr(href),
                    style_attr,
                    inner
                )
            } else {
                format!(
                    "<div id=\"{}\" class=\"np-imagebox\"{}>\n{}\n</div>",
                    escape_attr(&node.id),
                    style_attr,
                    inner
                )
            })
        }
        NodeKind::IconList { items } => {
            let mut list_html = String::new();
            for item in items {
                let text_part = if let Some(href) = item.link.as_ref().filter(|s| !s.is_empty()) {
                    format!("<a href=\"{}\">{}</a>", escape_attr(href), item.text)
                } else {
                    item.text.clone()
                };
                if item.icon.trim().is_empty() {
                    list_html.push_str(&format!(
                        "<li class=\"np-iconlist-item\"><span>{}</span></li>\n",
                        text_part
                    ));
                } else {
                    list_html.push_str(&format!(
                        "<li class=\"np-iconlist-item\"><i class=\"{}\"></i><span>{}</span></li>\n",
                        escape_attr(&item.icon),
                        text_part
                    ));
                }
            }
            let cls = with_extra_classes("np-iconlist", extra_cls);
            Some(format!(
                "<ul id=\"{}\" class=\"{}\"{}>\n{}</ul>",
                escape_attr(&node.id),
                escape_attr(&cls),
                style_attr,
                list_html
            ))
        }
        NodeKind::CustomHtml { raw } => {
            let cleaned = sanitize_editor_html(raw);
            let needs_target =
                !style_attr.is_empty() || extra_cls.is_some() || node.styles.has_scoped_rules();
            Some(if !needs_target {
                cleaned
            } else if let Some(injected) =
                super::inject::inject_root_attrs(&cleaned, &node.id, extra_cls, args.css)
            {
                injected
            } else {
                let cls = with_extra_classes("np-custom-widget", extra_cls);
                format!(
                    "<div id=\"{}\" class=\"{}\"{}>{}</div>",
                    escape_attr(&node.id),
                    escape_attr(&cls),
                    style_attr,
                    cleaned
                )
            })
        }
        NodeKind::ShortcodeFallback {
            tag,
            attributes: _,
            content,
        } => Some(format!(
            "<div id=\"{}\" class=\"np-shortcode-fallback\" data-shortcode=\"{}\"{}>\n{}\n</div>",
            escape_attr(&node.id),
            escape_attr(tag),
            style_attr,
            content
        )),
        _ => None,
    }
}
