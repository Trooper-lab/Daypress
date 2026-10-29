//! CSS containment and scoping for NeoStudio Visual Builder (`templates/admin/builder/scoping.rs`).
//!
//! Site, theme and plugin CSS is wrapped in `@scope (#neostudio-artboard)` so it styles the
//! artboard exactly like the public `<html>`/`<body>` without leaking into the studio chrome:
//! - selectors whose subject is the document root (`:root`, `html`, `body`, `html body` and
//!   `html.<theme-class>` for the theme class public SSR puts on `<html>`) become `:scope`;
//! - width-only `@media` queries keep following the studio window on the desktop artboard and
//!   are repeated as `@container` queries for the tablet/mobile artboards, so those previews hit
//!   the same breakpoints as a real viewport of that width;
//! - `@import`, `@font-face`, `@keyframes` and friends are hoisted in front of the block.

use super::css_tokens::{
    at_keyword, split_chunks, split_first_compound, split_padding, split_top_level, Chunk,
};

/// Container name the artboard registers for breakpoint queries.
pub const ARTBOARD_CONTAINER: &str = "neostudio-artboard";

/// Name-defining at-rules that are global by nature and are kept outside the `@scope` block.
const HOISTED_BLOCKS: [&str; 9] = [
    "font-face",
    "keyframes",
    "-webkit-keyframes",
    "-moz-keyframes",
    "property",
    "page",
    "counter-style",
    "font-feature-values",
    "font-palette-values",
];

/// Conditional group rules whose nested rules are scoped like top-level ones.
const GROUPING_BLOCKS: [&str; 4] = ["media", "supports", "container", "layer"];

/// Rewrites global theme/plugin CSS to be strictly contained within `#neostudio-artboard`.
pub fn scope_theme_css(css: &str) -> String {
    scope_site_css(css, None)
}

/// Like [`scope_theme_css`], additionally mapping `html.<html_class>` (the class public SSR sets
/// on `<html>`, e.g. `elementor-theme-active`) onto the artboard.
pub fn scope_site_css(css: &str, html_class: Option<&str>) -> String {
    let trimmed = css.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let mut hoisted = Hoisted::default();
    let body = rewrite_rules(trimmed, html_class, true, false, &mut hoisted);
    format!(
        "{}{}@scope (#neostudio-artboard) {{\n{body}\n}}",
        hoisted.imports, hoisted.globals
    )
}

/// Global at-rules lifted out of the `@scope` block; `@import` must precede every other rule.
#[derive(Default)]
struct Hoisted {
    imports: String,
    globals: String,
}

/// `desktop`: rules apply to the desktop artboard only (inside a window-width media query).
fn rewrite_rules(
    src: &str,
    html_class: Option<&str>,
    top: bool,
    desktop: bool,
    hoisted: &mut Hoisted,
) -> String {
    let mut out = String::with_capacity(src.len() + 64);
    for chunk in split_chunks(src) {
        match chunk {
            Chunk::Text(t) => out.push_str(t),
            Chunk::Statement(s) => match at_keyword(s).as_deref() {
                Some("charset") if top => {}
                Some("import") | Some("namespace") if top => {
                    hoisted.imports.push_str(s.trim());
                    hoisted.imports.push('\n');
                }
                _ => out.push_str(s),
            },
            Chunk::Block { prelude, body } => match at_keyword(prelude) {
                Some(kw) if top && HOISTED_BLOCKS.contains(&kw.as_str()) => {
                    hoisted.globals.push_str(prelude.trim());
                    hoisted.globals.push('{');
                    hoisted.globals.push_str(body);
                    hoisted.globals.push_str("}\n");
                }
                Some(kw) if GROUPING_BLOCKS.contains(&kw.as_str()) => {
                    let container = if kw == "media" {
                        media_to_container(prelude)
                    } else {
                        None
                    };
                    // Desktop artboard: the studio window is the viewport, as in public SSR
                    // at desktop widths. Tablet/mobile artboards: the artboard width is.
                    let inner_desktop = desktop || container.is_some();
                    out.push_str(prelude);
                    out.push('{');
                    out.push_str(&rewrite_rules(
                        body,
                        html_class,
                        false,
                        inner_desktop,
                        hoisted,
                    ));
                    out.push('}');
                    if let Some(cq) = container {
                        out.push('\n');
                        out.push_str(&cq);
                        out.push('{');
                        out.push_str(&rewrite_rules(body, html_class, false, desktop, hoisted));
                        out.push('}');
                    }
                }
                Some(_) => {
                    out.push_str(prelude);
                    out.push('{');
                    out.push_str(body);
                    out.push('}');
                }
                None => {
                    out.push_str(&rewrite_selector_list(prelude, html_class, desktop));
                    out.push('{');
                    out.push_str(body);
                    out.push('}');
                }
            },
        }
    }
    out
}

fn is_root_compound(compound: &str, html_class: Option<&str>) -> bool {
    let c = compound.to_ascii_lowercase();
    if matches!(c.as_str(), "html" | ":root" | "html:root") {
        return true;
    }
    html_class
        .map(|cls| c == format!("html.{}", cls.to_ascii_lowercase()))
        .unwrap_or(false)
}

/// Maps a selector whose subject chain starts at the document root onto `:scope`.
fn rewrite_selector(sel: &str, html_class: Option<&str>) -> Option<String> {
    let (first, rest) = split_first_compound(sel);
    if is_root_compound(first, html_class) {
        let after = rest.trim_start();
        let after = after
            .strip_prefix('>')
            .map(str::trim_start)
            .unwrap_or(after);
        let (second, rest2) = split_first_compound(after);
        if second.eq_ignore_ascii_case("body") {
            return Some(format!(":scope{rest2}"));
        }
        return Some(format!(":scope{rest}"));
    }
    if first.eq_ignore_ascii_case("body") {
        return Some(format!(":scope{rest}"));
    }
    None
}

fn rewrite_selector_list(prelude: &str, html_class: Option<&str>, desktop: bool) -> String {
    let (lead, core, trail) = split_padding(prelude);
    let selectors = split_top_level(core, ',');
    let rewritten: Vec<Option<String>> = selectors
        .iter()
        .map(|s| rewrite_selector(s.trim(), html_class))
        .collect();
    if !desktop && rewritten.iter().all(Option::is_none) {
        return prelude.to_string();
    }
    let mut merged: Vec<String> = Vec::with_capacity(selectors.len());
    for (orig, new) in selectors.iter().zip(rewritten) {
        let sel = new.unwrap_or_else(|| orig.trim().to_string());
        let sel = if desktop { desktop_only(&sel) } else { sel };
        if !merged.contains(&sel) {
            merged.push(sel);
        }
    }
    format!("{lead}{}{trail}", merged.join(", "))
}

/// `@media [only] screen|all and (max-width: 767px)` → `@container neostudio-artboard (...)`.
/// Queries with other media types or features (print, hover, prefers-*) give `None`.
fn media_to_container(prelude: &str) -> Option<String> {
    let (_, core, trail) = split_padding(prelude);
    let lower = core.to_ascii_lowercase();
    let query = lower.strip_prefix("@media").map(str::trim)?;
    let mut cond = query;
    for prefix in [
        "only screen and ",
        "screen and ",
        "only all and ",
        "all and ",
    ] {
        if let Some(rest) = cond.strip_prefix(prefix) {
            cond = rest.trim_start();
            break;
        }
    }
    let is_width_feature = |part: &str| {
        let Some(inner) = part
            .trim()
            .strip_prefix('(')
            .and_then(|p| p.strip_suffix(')'))
        else {
            return false;
        };
        let inner = inner.trim_start();
        !inner.contains('(')
            && ["min-width", "max-width", "width"].iter().any(|f| {
                inner
                    .strip_prefix(f)
                    .map(|r| r.trim_start().starts_with([':', '<', '>', '=']))
                    .unwrap_or(false)
            })
    };
    let width_only =
        !cond.is_empty() && !cond.contains(',') && cond.split(" and ").all(is_width_feature);
    width_only.then(|| format!("@container {ARTBOARD_CONTAINER} {cond}{trail}"))
}

/// Limits a selector to the desktop artboard without changing its specificity.
fn desktop_only(sel: &str) -> String {
    match sel.strip_prefix(":scope") {
        Some(rest) => format!(":scope:where(.artboard-desktop){rest}"),
        None => format!(":where(:scope.artboard-desktop) {sel}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_theme_css_wraps_in_at_scope() {
        let input = "body { background: #fff; } .btn { color: red; }";
        let output = scope_theme_css(input);
        assert!(output.contains("@scope (#neostudio-artboard) {"));
        assert!(output.contains(":scope { background: #fff; }"));
        assert!(output.contains(".btn { color: red; }"));
    }

    #[test]
    fn test_scope_theme_css_empty() {
        assert_eq!(scope_theme_css("   "), "");
    }

    #[test]
    fn test_class_names_containing_body_are_not_corrupted() {
        let out = scope_theme_css(".np-card-body { padding: 24px; } tbody { color: red; }");
        assert!(out.contains(".np-card-body { padding: 24px; }"), "{out}");
        assert!(out.contains("tbody { color: red; }"), "{out}");
        assert!(!out.contains("-:scope"), "{out}");
    }

    #[test]
    fn test_root_chains_and_theme_class_map_to_scope() {
        let css = "html, body { display: flex; }\nhtml.elementor-theme-active body { color: #111; }\nhtml.elementor-theme-active .np-heading { color: #222; }\nhtml > body .x { margin: 0; }";
        let out = scope_site_css(css, Some("elementor-theme-active"));
        assert!(out.contains(":scope { display: flex; }"), "{out}");
        assert!(out.contains(":scope { color: #111; }"), "{out}");
        assert!(out.contains(":scope .np-heading { color: #222; }"), "{out}");
        assert!(out.contains(":scope .x { margin: 0; }"), "{out}");
        // Without the matching theme class the rule stays relative (and never matches)
        let dark = scope_site_css(css, Some("dark"));
        assert!(
            dark.contains("html.elementor-theme-active .np-heading"),
            "{dark}"
        );
    }

    #[test]
    fn test_width_media_queries_follow_window_on_desktop_and_artboard_otherwise() {
        let css = "@media (max-width: 900px) { .np-grid { grid-template-columns: 1fr !important; } body { font-size: 14px; } }\n@media print { .x { display: none; } }\n@media (prefers-reduced-motion: reduce) { .y { transition: none; } }\n@media screen and (min-width: 768px) and (max-width: 1024px) { .z { color: red; } }";
        let out = scope_theme_css(css);
        // Desktop artboard: window media query, limited to the desktop artboard
        assert!(
            out.contains("@media (max-width: 900px) { :where(:scope.artboard-desktop) .np-grid {"),
            "{out}"
        );
        assert!(
            out.contains(":scope:where(.artboard-desktop) { font-size: 14px; }"),
            "{out}"
        );
        // Tablet/mobile artboards: the same rules against the artboard width
        assert!(
            out.contains("@container neostudio-artboard (max-width: 900px) { .np-grid {"),
            "{out}"
        );
        assert!(out.contains(":scope { font-size: 14px; }"), "{out}");
        assert!(
            out.contains("@media print { .x { display: none; } }"),
            "{out}"
        );
        assert!(
            out.contains("@media (prefers-reduced-motion: reduce) {"),
            "{out}"
        );
        assert!(
            out.contains(
                "@container neostudio-artboard (min-width: 768px) and (max-width: 1024px) {"
            ),
            "{out}"
        );
    }

    #[test]
    fn test_imports_and_font_faces_are_hoisted_before_scope() {
        // Concatenated sources: a later sheet's @import still has to end up first
        let css = "@charset \"utf-8\";\n@font-face { font-family: X; src: url(x.woff2); }\n:root { --a: 1; }\n@import url('https://fonts.googleapis.com/css2?family=Roboto');\n@keyframes spin { to { transform: rotate(1turn); } }";
        let out = scope_theme_css(css);
        assert!(
            out.starts_with("@import url('https://fonts.googleapis.com/css2?family=Roboto');\n"),
            "{out}"
        );
        let scope_at = out.find("@scope (#neostudio-artboard)").unwrap_or(0);
        assert!(
            out.find("@font-face").unwrap_or(usize::MAX) < scope_at,
            "{out}"
        );
        assert!(
            out.find("@keyframes spin").unwrap_or(usize::MAX) < scope_at,
            "{out}"
        );
        assert!(!out.contains("@charset"), "{out}");
        assert!(out.contains(":scope { --a: 1; }"), "{out}");
    }

    #[test]
    fn test_comments_strings_and_utf8_survive() {
        let css = "/* Kop — header */\nbody { content: \"a{b}c\"; } .ü::after { content: '}'; }";
        let out = scope_theme_css(css);
        assert!(
            out.contains("/* Kop — header */\n:scope { content: \"a{b}c\"; }"),
            "{out}"
        );
        assert!(out.contains(".ü::after { content: '}'; }"), "{out}");
    }
}
