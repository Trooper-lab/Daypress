use crate::schema::{NeopressLayout, NeopressNode, StyleProperties};

/// Google Fonts families offered by the NeoStudio typography picker. When a layout uses one of
/// them, the public stylesheet imports it so the live site matches the editor canvas.
pub const GOOGLE_FONT_FAMILIES: &[&str] = &[
    "Inter",
    "Roboto",
    "Open Sans",
    "Lato",
    "Montserrat",
    "Poppins",
    "Raleway",
    "Nunito",
    "DM Sans",
    "Plus Jakarta Sans",
    "Space Grotesk",
    "Work Sans",
    "Manrope",
    "Outfit",
    "Oswald",
    "Playfair Display",
    "Merriweather",
    "Lora",
    "DM Serif Display",
    "Libre Baskerville",
    "JetBrains Mono",
    "Fira Code",
];

/// Compiles scoped element styles, pseudo-states (`:hover`, `:focus`, `:active`), responsive
/// overrides (`@media (max-width: 1024px)` / `@media (max-width: 767px)`) and breakpoint-scoped
/// visibility from a NeopressLayout.
///
/// Base styles are rendered inline on each element, so every scoped declaration is emitted with
/// `!important` to guarantee it actually overrides the inline base value.
pub fn compile_layout_css(layout: &NeopressLayout) -> String {
    let mut buckets = CssBuckets::default();
    let mut fonts: Vec<&'static str> = Vec::new();
    collect_node_css(&layout.root, &mut buckets, &mut fonts);

    let mut css_out = String::new();

    if !fonts.is_empty() {
        let families: Vec<String> = fonts
            .iter()
            .map(|f| format!("family={}:wght@300;400;500;600;700;800", f.replace(' ', "+")))
            .collect();
        css_out.push_str(&format!(
            "@import url('https://fonts.googleapis.com/css2?{}&display=swap');\n",
            families.join("&")
        ));
    }

    for rule in buckets.state.iter() {
        css_out.push_str(rule);
        css_out.push('\n');
    }

    push_media(&mut css_out, "(min-width: 1025px)", &buckets.desktop_only);
    push_media(&mut css_out, "(max-width: 1024px)", &buckets.tablet);
    push_media(&mut css_out, "(min-width: 768px) and (max-width: 1024px)", &buckets.tablet_only);
    push_media(&mut css_out, "(max-width: 767px)", &buckets.mobile);

    css_out
}

#[derive(Default)]
struct CssBuckets {
    state: Vec<String>,
    desktop_only: Vec<String>,
    tablet: Vec<String>,
    tablet_only: Vec<String>,
    mobile: Vec<String>,
}

fn push_media(out: &mut String, query: &str, rules: &[String]) {
    if rules.is_empty() {
        return;
    }
    out.push_str(&format!("@media {} {{\n", query));
    for rule in rules {
        out.push_str("  ");
        out.push_str(rule);
        out.push('\n');
    }
    out.push_str("}\n");
}

fn selector_for(id: &str) -> String {
    format!("[id=\"{}\"]", id.replace('\\', "\\\\").replace('"', "\\\""))
}

fn collect_fonts(styles: &StyleProperties, fonts: &mut Vec<&'static str>) {
    let layers = [
        Some(styles),
        styles.tablet.as_deref(),
        styles.mobile.as_deref(),
        styles.hover.as_deref(),
        styles.focus.as_deref(),
        styles.active.as_deref(),
    ];
    for layer in layers.into_iter().flatten() {
        if let Some(ff) = layer.font_family.as_deref() {
            let first = ff
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches(|c| c == '\'' || c == '"')
                .trim();
            if let Some(known) = GOOGLE_FONT_FAMILIES.iter().find(|g| g.eq_ignore_ascii_case(first)) {
                if !fonts.contains(known) {
                    fonts.push(known);
                }
            }
        }
    }
}

fn collect_node_css(node: &NeopressNode, b: &mut CssBuckets, fonts: &mut Vec<&'static str>) {
    let sel = selector_for(&node.id);
    let styles = &node.styles;
    collect_fonts(styles, fonts);

    // Pseudo-states
    for (layer, pseudo) in [
        (styles.hover.as_deref(), ":hover"),
        (styles.focus.as_deref(), ":focus"),
        (styles.active.as_deref(), ":active"),
    ] {
        if let Some(s) = layer {
            let decls = s.to_css_style_important();
            if !decls.is_empty() {
                b.state.push(format!("{}{} {{ {} }}", sel, pseudo, decls));
            }
        }
    }

    // Responsive overrides (cascade: tablet applies to tablet + mobile, mobile only to mobile)
    if let Some(ref t) = styles.tablet {
        let decls = t.to_css_style_important();
        if !decls.is_empty() {
            b.tablet.push(format!("{} {{ {} }}", sel, decls));
        }
    }
    if let Some(ref m) = styles.mobile {
        let decls = m.to_css_style_important();
        if !decls.is_empty() {
            b.mobile.push(format!("{} {{ {} }}", sel, decls));
        }
    }

    // Breakpoint-scoped visibility (each toggle only affects its own device range)
    let hide = |bucket: &mut Vec<String>| bucket.push(format!("{} {{ display: none !important; }}", sel));
    if styles.hide_desktop == Some(true) {
        hide(&mut b.desktop_only);
    }
    if styles.hide_tablet == Some(true) {
        hide(&mut b.tablet_only);
    }
    if styles.hide_mobile == Some(true) {
        hide(&mut b.mobile);
    }

    for child in &node.children {
        collect_node_css(child, b, fonts);
    }
}
