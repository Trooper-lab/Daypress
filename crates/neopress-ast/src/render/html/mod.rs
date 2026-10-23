pub mod basic;
pub mod commercial;
pub mod content;
pub mod dynamic;
pub mod interactive;
pub mod layout;
pub mod marketing;

use crate::schema::{NeopressNode, NodeKind};

pub(crate) struct RenderArgs<'a> {
    pub(crate) node: &'a NeopressNode,
    pub(crate) inner_html: &'a str,
    pub(crate) css: &'a str,
    pub(crate) extra_cls: Option<&'a str>,
    pub(crate) style_attr: &'a str,
    pub(crate) effective_style_attr: &'a str,
    pub(crate) is_synthetic_root: bool,
}

/// Recursively compiles a NeopressNode and its descendants into semantic HTML5.
pub fn render_node_to_html(node: &NeopressNode) -> String {
    let css = node.styles.to_css_style().replace('"', "'");
    let extra_cls = node.styles.css_classes.as_deref();
    let style_attr = if css.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", css)
    };

    let inner_html: String = node.children.iter().map(render_node_to_html).collect();

    let is_synthetic_root = matches!(
        node.id.as_str(),
        "elem_root" | "divi_root" | "divi5_root" | "gutenberg_root" | "sec_root" | "root"
    ) || node.id.starts_with("root_");

    let has_multi_columns = node.children.len() > 1
        && node.children.iter().all(|c| {
            matches!(
                c.kind,
                NodeKind::Column | NodeKind::Flex { basis: Some(_), .. }
            )
        });

    let mut effective_css = css.clone();
    if has_multi_columns && !effective_css.contains("display:") {
        effective_css = format!(
            "display: flex; flex-direction: row; gap: 28px; align-items: flex-start; {}",
            effective_css
        )
        .trim()
        .to_string();
    }
    let effective_style_attr = if effective_css.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", effective_css)
    };

    let args = RenderArgs {
        node,
        inner_html: &inner_html,
        css: &css,
        extra_cls,
        style_attr: &style_attr,
        effective_style_attr: &effective_style_attr,
        is_synthetic_root,
    };

    layout::render_layout(&args)
        .or_else(|| basic::render_basic(&args))
        .or_else(|| commercial::render_commercial(&args))
        .or_else(|| marketing::render_marketing(&args))
        .or_else(|| interactive::render_interactive(&args))
        .or_else(|| dynamic::render_dynamic(&args))
        .or_else(|| content::render_content(&args))
        .unwrap_or_default()
}
