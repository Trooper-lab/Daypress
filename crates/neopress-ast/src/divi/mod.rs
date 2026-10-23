pub mod context;
pub mod divi5;
pub mod modules;
pub mod parser;
pub mod styles;
pub mod widgets;

pub use context::*;
pub use divi5::*;
pub use modules::*;
pub use parser::*;
pub use widgets::*;

use crate::schema::{NeopressLayout, NeopressNode, NodeKind, StyleProperties};

/// Parses raw Divi shortcode markup into a canonical NeopressLayout AST.
pub fn transpile_divi_content(content: &str) -> Result<NeopressLayout, String> {
    transpile_divi_content_with_context(content, &DiviContext::default())
}

/// Parses raw Divi shortcode or Divi 5 block markup with site-wide context into a canonical NeopressLayout AST.
pub fn transpile_divi_content_with_context(
    content: &str,
    ctx: &DiviContext,
) -> Result<NeopressLayout, String> {
    // If Divi 5 Gutenberg block comments format is detected
    if content.contains("<!-- wp:divi/") && !content.contains("<!-- wp:divi/placeholder") {
        return transpile_divi5_content_with_context(content, ctx);
    }

    let cleaned = content
        .replace("<!-- wp:divi/placeholder -->", "")
        .replace("<!-- /wp:divi/placeholder -->", "")
        .replace("<!-- wp:divi/placeholder /-->", "");

    let tokens = tokenize_divi(&cleaned)?;
    let tree = parse_divi_tree(tokens)?;

    let mut id_counter: usize = 0;
    let root_children: Vec<NeopressNode> = tree
        .iter()
        .map(|node| map_divi_node(node, ctx, &mut id_counter))
        .collect();

    Ok(NeopressLayout {
        version: 1,
        root: NeopressNode {
            id: "divi_root".to_string(),
            kind: NodeKind::Section,
            styles: StyleProperties::default(),
            children: root_children,
        },
        global_styles: None,
    })
}
