use crate::schema::{DaypressLayout, DaypressNode, StyleProperties};

/// Google Fonts families offered by the DayStudio typography picker. When a layout uses one of
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
/// visibility from a DaypressLayout.
///
/// Base styles are rendered inline on each element, so every scoped declaration is emitted with
/// `!important` to guarantee it actually overrides the inline base value.
pub fn compile_layout_css(layout: &DaypressLayout) -> String {
    let mut buckets = CssBuckets::default();
    let mut fonts: Vec<&'static str> = Vec::new();
    collect_node_css(&layout.root, &mut buckets, &mut fonts);

    let mut css_out = String::new();

    if !fonts.is_empty() {
        let families: Vec<String> = fonts
            .iter()
            .map(|f| {
                format!(
                    "family={}:wght@300;400;500;600;700;800",
                    f.replace(' ', "+")
                )
            })
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

    for block in buckets.custom.iter() {
        css_out.push_str(block);
        css_out.push('\n');
    }

    // Widest query first, so a narrower layer overrides a wider one on the same element.
    let mut media: Vec<(f32, String, &Vec<String>)> = vec![
        (
            f32::MAX,
            "(min-width: 1025px)".to_string(),
            &buckets.desktop_only,
        ),
        (1024.0, "(max-width: 1024px)".to_string(), &buckets.tablet),
        (
            1024.0,
            "(min-width: 768px) and (max-width: 1024px)".to_string(),
            &buckets.tablet_only,
        ),
        (767.0, "(max-width: 767px)".to_string(), &buckets.mobile),
    ];
    for (width, (px, rules)) in &buckets.design_breakpoints {
        media.push((*px, format!("(max-width: {width})"), rules));
    }
    media.sort_by(|a, b| b.0.total_cmp(&a.0));
    for (_, query, rules) in media {
        push_media(&mut css_out, &query, rules);
    }

    css_out
}

#[derive(Default)]
struct CssBuckets {
    state: Vec<String>,
    /// Node `custom_css` rule blocks, scoped to the node.
    custom: Vec<String>,
    desktop_only: Vec<String>,
    tablet: Vec<String>,
    tablet_only: Vec<String>,
    mobile: Vec<String>,
    /// Layers carrying an imported design's own breakpoint: `width -> (px, rules)`.
    design_breakpoints: std::collections::BTreeMap<String, (f32, Vec<String>)>,
}

impl CssBuckets {
    /// Files a `tablet` / `mobile` layer's rule under its own breakpoint, or `standard` without one.
    fn push_layer(&mut self, layer: &StyleProperties, sel: &str, standard: Bucket) {
        let decls = layer.to_css_style_important();
        if decls.is_empty() {
            return;
        }
        let rule = format!("{} {{ {} }}", sel, decls);
        let custom = layer
            .breakpoint
            .as_deref()
            .and_then(|w| breakpoint_px(w).map(|px| (w.trim().to_string(), px)));
        match (custom, standard) {
            (Some((width, px)), _) => {
                self.design_breakpoints
                    .entry(width)
                    .or_insert_with(|| (px, Vec::new()))
                    .1
                    .push(rule);
            }
            (None, Bucket::Tablet) => self.tablet.push(rule),
            (None, Bucket::Mobile) => self.mobile.push(rule),
        }
    }
}

#[derive(Clone, Copy)]
enum Bucket {
    Tablet,
    Mobile,
}

/// Parses a media-query length (`900px`, `60rem`, `56.25em`) into CSS pixels. Anything else is
/// rejected, so a layout's `breakpoint` value can never inject CSS.
pub(crate) fn breakpoint_px(width: &str) -> Option<f32> {
    let w = width.trim().to_ascii_lowercase();
    let (number, factor) = if let Some(n) = w.strip_suffix("rem") {
        (n, 16.0)
    } else if let Some(n) = w.strip_suffix("em") {
        (n, 16.0)
    } else {
        (w.strip_suffix("px")?, 1.0)
    };
    let n: f32 = number.trim().parse().ok()?;
    (n.is_finite() && n > 0.0).then_some(n * factor)
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
            if let Some(known) = GOOGLE_FONT_FAMILIES
                .iter()
                .find(|g| g.eq_ignore_ascii_case(first))
            {
                if !fonts.contains(known) {
                    fonts.push(known);
                }
            }
        }
    }
}

fn collect_node_css(node: &DaypressNode, b: &mut CssBuckets, fonts: &mut Vec<&'static str>) {
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
        b.push_layer(t, &sel, Bucket::Tablet);
    }
    if let Some(ref m) = styles.mobile {
        b.push_layer(m, &sel, Bucket::Mobile);
    }

    // Breakpoint-scoped visibility (each toggle only affects its own device range)
    let hide =
        |bucket: &mut Vec<String>| bucket.push(format!("{} {{ display: none !important; }}", sel));
    if styles.hide_desktop == Some(true) {
        hide(&mut b.desktop_only);
    }
    if styles.hide_tablet == Some(true) {
        hide(&mut b.tablet_only);
    }
    if styles.hide_mobile == Some(true) {
        hide(&mut b.mobile);
    }

    // Node custom CSS written as rules is scoped to the node; plain declarations render inline.
    if let Some(custom) = styles.custom_css.as_deref().filter(|c| c.contains('{')) {
        let scoped = scope_custom_css(&sel, custom);
        if !scoped.trim().is_empty() {
            b.custom.push(scoped);
        }
    }

    for child in &node.children {
        collect_node_css(child, b, fonts);
    }
}

/// Scopes a node's `custom_css` rule blocks to the node.
///
/// Each selector is rewritten relative to the node (`[id="node"]`): the Elementor-style
/// `selector` keyword, `&` and `:scope` stand for the node itself; any other selector targets
/// descendants (`.title` → `[id="node"] .title`). `@media` / `@supports` / `@container` blocks
/// are scoped recursively; other at-rules (`@keyframes`, `@font-face`) are kept as written.
pub fn scope_custom_css(scope: &str, css: &str) -> String {
    let mut out = String::new();
    for (prelude, body) in split_css_blocks(css) {
        let lower = prelude.to_ascii_lowercase();
        if lower.starts_with("@media")
            || lower.starts_with("@supports")
            || lower.starts_with("@container")
        {
            out.push_str(&format!(
                "{} {{\n{}}}\n",
                prelude,
                scope_custom_css(scope, &body)
            ));
        } else if prelude.starts_with('@') {
            out.push_str(&format!("{} {{{}}}\n", prelude, body));
        } else {
            let selectors: Vec<String> = split_top_level_commas(&prelude)
                .into_iter()
                .map(|s| scope_selector(scope, s.trim()))
                .filter(|s| !s.is_empty())
                .collect();
            if !selectors.is_empty() {
                out.push_str(&format!("{} {{ {} }}\n", selectors.join(", "), body.trim()));
            }
        }
    }
    out
}

fn scope_selector(scope: &str, sel: &str) -> String {
    if sel.is_empty() {
        return String::new();
    }
    let with_keyword = replace_selector_keyword(sel, scope);
    if with_keyword != sel {
        return with_keyword;
    }
    if sel.contains('&') {
        return sel.replace('&', scope);
    }
    if let Some(rest) = sel.strip_prefix(":scope") {
        return format!("{}{}", scope, rest);
    }
    format!("{} {}", scope, sel)
}

/// Replaces the whole-word `selector` keyword (Elementor custom CSS) with `scope`.
fn replace_selector_keyword(sel: &str, scope: &str) -> String {
    const KEY: &str = "selector";
    let mut out = String::new();
    let mut rest = sel;
    while let Some(pos) = rest.find(KEY) {
        let head = rest.get(..pos).unwrap_or("");
        let tail = rest.get(pos + KEY.len()..).unwrap_or("");
        let before = head.chars().last();
        let after = tail.chars().next();
        let is_word_char = |c: char| c.is_alphanumeric() || matches!(c, '-' | '_' | '.' | '#');
        let standalone = !before.is_some_and(is_word_char)
            && !after.is_some_and(|c| c.is_alphanumeric() || matches!(c, '-' | '_'));
        out.push_str(head);
        out.push_str(if standalone { scope } else { KEY });
        rest = tail;
    }
    out.push_str(rest);
    out
}

/// Splits CSS into top-level `(prelude, body)` blocks, respecting nested braces and quotes.
fn split_css_blocks(css: &str) -> Vec<(String, String)> {
    let mut blocks = Vec::new();
    let mut prelude = String::new();
    let mut body = String::new();
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut chars = css.chars().peekable();
    while let Some(c) = chars.next() {
        // Comments are dropped.
        if quote.is_none() && c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut prev = ' ';
            for n in chars.by_ref() {
                if prev == '*' && n == '/' {
                    break;
                }
                prev = n;
            }
            continue;
        }
        let target = if depth == 0 { &mut prelude } else { &mut body };
        if let Some(q) = quote {
            target.push(c);
            if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => {
                quote = Some(c);
                target.push(c);
            }
            '{' => {
                if depth > 0 {
                    body.push(c);
                }
                depth += 1;
            }
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let p = prelude.trim().to_string();
                    if !p.is_empty() {
                        blocks.push((p, std::mem::take(&mut body)));
                    }
                    prelude.clear();
                    body.clear();
                } else {
                    body.push(c);
                }
            }
            ';' if depth == 0 => prelude.clear(),
            _ => target.push(c),
        }
    }
    blocks
}

fn split_top_level_commas(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut start = 0usize;
    for (i, c) in s.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            ',' if depth == 0 => {
                parts.push(s.get(start..i).unwrap_or(""));
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(s.get(start..).unwrap_or(""));
    parts
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::{DaypressLayout, DaypressNode, NodeKind, StyleProperties};

    fn layout_with(custom_css: &str) -> DaypressLayout {
        DaypressLayout {
            version: 1,
            root: DaypressNode {
                id: "root".into(),
                kind: NodeKind::Root,
                styles: StyleProperties::default(),
                children: vec![DaypressNode {
                    id: "card".into(),
                    kind: NodeKind::Container,
                    styles: StyleProperties {
                        custom_css: Some(custom_css.into()),
                        ..Default::default()
                    },
                    children: Vec::new(),
                }],
            },
            global_styles: None,
        }
    }

    #[test]
    fn node_custom_css_rules_are_scoped_to_the_node() {
        let layout = layout_with(
            "selector { outline: 1px solid red; }\n.title, & > p { color: blue; }\n:scope:hover { opacity: .9 }\n@media (max-width: 600px) { .title { font-size: 14px; } }",
        );
        let css = compile_layout_css(&layout);
        assert!(
            css.contains(r#"[id="card"] { outline: 1px solid red; }"#),
            "{css}"
        );
        assert!(
            css.contains(r#"[id="card"] .title, [id="card"] > p { color: blue; }"#),
            "{css}"
        );
        assert!(
            css.contains(r#"[id="card"]:hover { opacity: .9 }"#),
            "{css}"
        );
        assert!(
            css.contains("@media (max-width: 600px) {\n[id=\"card\"] .title { font-size: 14px; }"),
            "{css}"
        );

        // Rule blocks are never pasted into the inline style attribute.
        let html = crate::render_node_to_html(&layout.root);
        assert!(!html.contains("style="), "{html}");
    }

    #[test]
    fn node_custom_css_declarations_still_render_inline() {
        let layout = layout_with("backdrop-filter: blur(8px); cursor: pointer;");
        let html = crate::render_node_to_html(&layout.root);
        assert!(html.contains("backdrop-filter: blur(8px);"), "{html}");
        assert!(!compile_layout_css(&layout).contains("[id=\"card\"]"));
    }

    #[test]
    fn selector_keyword_is_only_replaced_as_a_whole_word() {
        assert_eq!(
            scope_custom_css("[id=\"x\"]", ".selector-list, selector .a { b: c }"),
            "[id=\"x\"] .selector-list, [id=\"x\"] .a { b: c }\n"
        );
    }
}
