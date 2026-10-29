pub mod commercial;
pub mod content;
pub mod dynamic;
pub mod headers;
pub mod interactive;
pub mod layout;

use crate::divi::context::DiviContext;
use crate::divi::parser::DiviShortcode;
use crate::divi::styles::extract_divi_styles;
use crate::schema::{DaypressNode, NodeKind};

pub fn map_divi_node(divi: &DiviShortcode, ctx: &DiviContext, counter: &mut usize) -> DaypressNode {
    *counter += 1;
    let short_name = divi.tag.strip_prefix("et_pb_").unwrap_or(&divi.tag);
    let id = divi
        .attributes
        .get("module_id")
        .filter(|s| !s.trim().is_empty())
        .cloned()
        .unwrap_or_else(|| format!("divi_{}_{}", short_name, *counter));
    let mut styles = extract_divi_styles(&divi.attributes, ctx);

    let mut children: Vec<DaypressNode> = divi
        .children
        .iter()
        .map(|c| map_divi_node(c, ctx, counter))
        .collect();

    let kind = layout::map_layout_module(divi, ctx, &mut styles, &mut children, id.as_str())
        .or_else(|| content::map_content_module(divi, ctx, &mut styles, &mut children))
        .or_else(|| headers::map_headers_module(divi, ctx, &mut styles))
        .or_else(|| interactive::map_interactive_module(divi, ctx, &mut children))
        .or_else(|| commercial::map_commercial_module(divi, ctx, &mut styles, &mut children))
        .or_else(|| dynamic::map_dynamic_module(divi, ctx, &mut children))
        .unwrap_or_else(|| NodeKind::ShortcodeFallback {
            tag: divi.tag.clone(),
            attributes: divi.attributes.clone(),
            content: divi.inner_text.clone(),
        });

    styles.sync_flat_and_nested();

    DaypressNode {
        id,
        kind,
        styles,
        children,
    }
}
