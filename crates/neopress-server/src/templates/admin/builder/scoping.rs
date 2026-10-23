//! CSS containment and scoping for NeoStudio Visual Builder (`templates/admin/builder/scoping.rs`).

/// Rewrites global theme/plugin CSS to be strictly contained within `#neostudio-artboard`.
///
/// Converts `:root`, `html`, and `body` to `:scope`, and encapsulates all rules
/// inside a modern CSS `@scope (#neostudio-artboard) { ... }` block.
pub fn scope_theme_css(css: &str) -> String {
    let trimmed = css.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    // Replace global root/body references with :scope so styles apply to the artboard root
    let rewritten = trimmed
        .replace(":root", ":scope")
        .replace("html, body", ":scope")
        .replace("html,body", ":scope")
        .replace("body {", ":scope {")
        .replace("body{", ":scope{")
        .replace("html {", ":scope {")
        .replace("html{", ":scope{");

    format!("@scope (#neostudio-artboard) {{\n{}\n}}", rewritten)
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
}
