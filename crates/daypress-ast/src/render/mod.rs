pub mod css;
pub mod forms;
pub mod html;
pub mod sanitizer;

pub use css::compile_layout_css;
pub use html::render_node_to_html;
pub use sanitizer::{
    clean_empty_class_artifacts, resolve_semantic_tag, sanitize_editor_html, sanitize_ssr_html,
    strip_editor_classes,
};

use crate::kit::DaystudioKit;
use crate::schema::DaypressLayout;

/// Compiles a complete DaypressLayout into pristine semantic HTML5 for public SSR
/// with global styles, scoped layout CSS (hover/tablet/mobile media queries), and a 3-tier
/// zero-leak sanitization pass.
pub fn render_layout_to_html(layout: &DaypressLayout) -> String {
    let mut output = String::new();

    // 1. Compile Global Styles Kit CSS
    if let Some(ref gs) = layout.global_styles {
        if gs.is_object() {
            if let Ok(kit) = serde_json::from_value::<DaystudioKit>(gs.clone()) {
                output.push_str("<style id=\"daypress-kit-css\">\n");
                output.push_str(&kit.compile_to_css());
                output.push_str("</style>\n");
            }
        }
    }

    // 2. Compile Scoped Layout CSS (responsive media queries & hover states)
    let layout_css = compile_layout_css(layout);
    if !layout_css.is_empty() {
        output.push_str("<style id=\"daypress-layout-css\">\n");
        output.push_str(&layout_css);
        output.push_str("</style>\n");
    }

    // 3. Render HTML node tree
    output.push_str(&render_node_to_html(&layout.root));

    // 4. Final 3-tier SSR sanitization firewall (zero DOM leak guarantee)
    sanitize_ssr_html(&output)
}
