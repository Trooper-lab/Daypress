use crate::gutenberg::parser::GutenbergBlock;
use crate::schema::NodeKind;

pub(crate) fn compile_gutenberg_list(block: &GutenbergBlock) -> String {
    let is_ordered = block
        .attrs
        .get("ordered")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let tag = if is_ordered { "ol" } else { "ul" };

    let mut open_tag = format!("<{}", tag);
    if let Some(start) = block.attrs.get("start").and_then(|v| v.as_u64()) {
        open_tag.push_str(&format!(r#" start="{}""#, start));
    }
    if block
        .attrs
        .get("reversed")
        .and_then(|v| v.as_bool())
        .unwrap_or(false)
    {
        open_tag.push_str(" reversed");
    }
    open_tag.push('>');

    if block.children.is_empty() {
        if !block.inner_html.trim().is_empty() {
            return block.inner_html.trim().to_string();
        }
        return format!("{}</{}>", open_tag, tag);
    }

    let mut items_html = String::new();

    for child in &block.children {
        let child_name = child.name.strip_prefix("core/").unwrap_or(&child.name);
        if child_name == "list-item" {
            let mut item_text = child.inner_html.trim();
            if let Some(stripped) = item_text.strip_prefix("<li>") {
                item_text = stripped;
            }
            if let Some(stripped) = item_text.strip_suffix("</li>") {
                item_text = stripped;
            }
            let item_text = item_text.trim();

            let mut nested_html = String::new();
            for sub in &child.children {
                let sub_name = sub.name.strip_prefix("core/").unwrap_or(&sub.name);
                if sub_name == "list" {
                    nested_html.push('\n');
                    nested_html.push_str(&compile_gutenberg_list(sub));
                }
            }

            items_html.push_str("  <li>");
            items_html.push_str(item_text);
            if !nested_html.is_empty() {
                items_html.push_str(&nested_html);
                items_html.push('\n');
                items_html.push_str("  ");
            }
            items_html.push_str("</li>\n");
        } else if child_name == "list" {
            items_html.push_str("  <li>\n");
            items_html.push_str(&compile_gutenberg_list(child));
            items_html.push_str("\n  </li>\n");
        } else {
            items_html.push_str("  <li>");
            items_html.push_str(child.inner_html.trim());
            items_html.push_str("</li>\n");
        }
    }

    format!("{}\n{}</{}>", open_tag, items_html, tag)
}

pub(crate) fn map_gutenberg_list(block: &GutenbergBlock) -> NodeKind {
    let content_html = compile_gutenberg_list(block);
    NodeKind::Text { content_html }
}
