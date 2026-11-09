pub mod basic;
pub mod commercial;
pub mod content;
pub mod dynamic;
pub(crate) mod inject;
pub mod interactive;
pub mod layout;
pub mod marketing;

use crate::render::sanitizer::escape_attr;
use crate::schema::{DaypressNode, NodeKind};

pub(crate) struct RenderArgs<'a> {
    pub(crate) node: &'a DaypressNode,
    pub(crate) inner_html: &'a str,
    pub(crate) css: &'a str,
    /// The node's rendered `data-*` attributes (` data-x="y"`), already part of `style_attr`.
    pub(crate) data_attrs: &'a str,
    pub(crate) extra_cls: Option<&'a str>,
    pub(crate) style_attr: &'a str,
    pub(crate) effective_style_attr: &'a str,
    pub(crate) is_synthetic_root: bool,
}

/// Renders `styles.data_attributes`. Only well-formed `data-*` names get through, so a layout can
/// never add other attributes (such as event handlers) this way.
fn data_attributes_html(node: &DaypressNode) -> String {
    let Some(attrs) = node.styles.data_attributes.as_ref() else {
        return String::new();
    };
    attrs
        .iter()
        .filter(|(name, _)| {
            name.strip_prefix("data-").is_some_and(|rest| {
                !rest.is_empty()
                    && rest.chars().all(|c| {
                        c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_' | '.')
                    })
            })
        })
        .map(|(name, value)| format!(" {}=\"{}\"", name, escape_attr(value)))
        .collect()
}

/// Recursively compiles a DaypressNode and its descendants into semantic HTML5.
pub fn render_node_to_html(node: &DaypressNode) -> String {
    let css = node.styles.to_css_style().replace('"', "'");
    let merged_classes = node.styles.merged_classes();
    let extra_cls = merged_classes.as_deref();
    let data_attrs = data_attributes_html(node);
    let style_attr = if css.is_empty() {
        data_attrs.clone()
    } else {
        format!("{} style=\"{}\"", data_attrs, css)
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
        data_attrs.clone()
    } else {
        format!("{} style=\"{}\"", data_attrs, effective_css)
    };

    let args = RenderArgs {
        node,
        inner_html: &inner_html,
        css: &css,
        data_attrs: &data_attrs,
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
