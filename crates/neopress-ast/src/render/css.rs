use crate::schema::{NeopressLayout, NeopressNode};

/// Compiles scoped element styles, hover states, and responsive media queries
/// (`@media (max-width: 1024px)` and `@media (max-width: 767px)`) from a NeopressLayout.
pub fn compile_layout_css(layout: &NeopressLayout) -> String {
    let mut hover_rules = Vec::new();
    let mut desktop_rules = Vec::new();
    let mut tablet_rules = Vec::new();
    let mut mobile_rules = Vec::new();

    collect_node_css(
        &layout.root,
        &mut hover_rules,
        &mut desktop_rules,
        &mut tablet_rules,
        &mut mobile_rules,
    );

    let mut css_out = String::new();

    for rule in &hover_rules {
        css_out.push_str(rule);
        css_out.push('\n');
    }
    for rule in &desktop_rules {
        css_out.push_str(rule);
        css_out.push('\n');
    }

    if !tablet_rules.is_empty() {
        css_out.push_str("@media (max-width: 1024px) {\n");
        for rule in &tablet_rules {
            css_out.push_str("  ");
            css_out.push_str(rule);
            css_out.push('\n');
        }
        css_out.push_str("}\n");
    }

    if !mobile_rules.is_empty() {
        css_out.push_str("@media (max-width: 767px) {\n");
        for rule in &mobile_rules {
            css_out.push_str("  ");
            css_out.push_str(rule);
            css_out.push('\n');
        }
        css_out.push_str("}\n");
    }

    css_out
}

fn collect_node_css(
    node: &NeopressNode,
    hover_rules: &mut Vec<String>,
    desktop_rules: &mut Vec<String>,
    tablet_rules: &mut Vec<String>,
    mobile_rules: &mut Vec<String>,
) {
    let id = &node.id;

    // Hover state
    if let Some(ref h) = node.styles.hover {
        let hover_css = h.to_css_style();
        if !hover_css.is_empty() {
            hover_rules.push(format!("[id=\"{}\"]:hover {{ {} }}", id, hover_css));
        }
    }

    // Desktop visibility
    if node.styles.hide_desktop == Some(true) {
        desktop_rules.push(format!("[id=\"{}\"] {{ display: none !important; }}", id));
    }

    // Tablet state & visibility
    if let Some(ref t) = node.styles.tablet {
        let tab_css = t.to_css_style();
        if !tab_css.is_empty() {
            tablet_rules.push(format!("[id=\"{}\"] {{ {} }}", id, tab_css));
        }
    }
    if node.styles.hide_tablet == Some(true) {
        tablet_rules.push(format!("[id=\"{}\"] {{ display: none !important; }}", id));
    }

    // Mobile state & visibility
    if let Some(ref m) = node.styles.mobile {
        let mob_css = m.to_css_style();
        if !mob_css.is_empty() {
            mobile_rules.push(format!("[id=\"{}\"] {{ {} }}", id, mob_css));
        }
    }
    if node.styles.hide_mobile == Some(true) {
        mobile_rules.push(format!("[id=\"{}\"] {{ display: none !important; }}", id));
    }

    for child in &node.children {
        collect_node_css(child, hover_rules, desktop_rules, tablet_rules, mobile_rules);
    }
}
