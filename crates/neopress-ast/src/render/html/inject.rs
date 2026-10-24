//! Attribute injection onto the root element of raw HTML fragments (`CustomHtml` nodes).
//!
//! NeoStudio applies a `CustomHtml` node's styles directly to the fragment's root element on the
//! canvas. To keep the public SSR output visually identical, the same `id`, classes and inline
//! styles are injected onto that root element instead of wrapping it in an extra `<div>` (a
//! wrapper's background/padding would sit *behind* the root's own theme styles).

use crate::render::sanitizer::escape_attr;
use regex::Regex;
use std::sync::OnceLock;

static ROOT_TAG_RE: OnceLock<Regex> = OnceLock::new();
static TAG_SCAN_RE: OnceLock<Regex> = OnceLock::new();
static ID_ATTR_RE: OnceLock<Regex> = OnceLock::new();
static CLASS_ATTR_RE: OnceLock<Regex> = OnceLock::new();
static STYLE_ATTR_RE: OnceLock<Regex> = OnceLock::new();

const VOID_TAGS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track", "wbr",
];

fn attr_re(cell: &'static OnceLock<Regex>, name: &str) -> &'static Regex {
    cell.get_or_init(|| {
        Regex::new(&format!(
            r#"(?i)(^|\s){}\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s"'>]+))"#,
            name
        ))
        .expect("valid attribute regex")
    })
}

fn attr_value<'a>(caps: &regex::Captures<'a>) -> &'a str {
    caps.get(2)
        .or_else(|| caps.get(3))
        .or_else(|| caps.get(4))
        .map(|m| m.as_str())
        .unwrap_or("")
}

/// Finds the byte index just past the closing tag that matches the root element.
fn find_root_end(html: &str, tag: &str, from: usize) -> Option<usize> {
    let scan = TAG_SCAN_RE.get_or_init(|| {
        Regex::new(r"<(/?)([a-zA-Z][a-zA-Z0-9-]*)[^>]*?(/?)>").expect("valid tag scan regex")
    });
    let mut depth = 1usize;
    for caps in scan.captures_iter(&html[from..]) {
        let name = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        if !name.eq_ignore_ascii_case(tag) {
            continue;
        }
        let is_close = caps.get(1).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        let self_closing = caps.get(3).map(|m| !m.as_str().is_empty()).unwrap_or(false);
        if is_close {
            depth -= 1;
            if depth == 0 {
                return caps.get(0).map(|m| from + m.end());
            }
        } else if !self_closing {
            depth += 1;
        }
    }
    None
}

/// Injects `id`, extra classes and inline CSS into the single root element of `raw`.
///
/// Returns `None` when the fragment has no single root element or the root already carries its
/// own `id` (theme CSS/anchors may depend on it); callers then fall back to a wrapper element.
pub(crate) fn inject_root_attrs(raw: &str, id: &str, extra_cls: Option<&str>, css: &str) -> Option<String> {
    let trimmed = raw.trim();
    let root_re = ROOT_TAG_RE.get_or_init(|| {
        Regex::new(r#"^<([a-zA-Z][a-zA-Z0-9-]*)((?:\s+[^\s"'>/=]+(?:\s*=\s*(?:"[^"]*"|'[^']*'|[^\s"'>]+))?)*)\s*(/?)>"#)
            .expect("valid root tag regex")
    });
    let caps = root_re.captures(trimmed)?;
    let tag = caps.get(1)?.as_str();
    let attrs = caps.get(2).map(|m| m.as_str()).unwrap_or("");
    let open_end = caps.get(0)?.end();
    let self_closing = caps.get(3).map(|m| !m.as_str().is_empty()).unwrap_or(false)
        || VOID_TAGS.iter().any(|v| v.eq_ignore_ascii_case(tag));

    let root_end = if self_closing { open_end } else { find_root_end(trimmed, tag, open_end)? };
    if !trimmed[root_end..].trim().is_empty() {
        return None; // multiple top-level nodes
    }
    if attr_re(&ID_ATTR_RE, "id").is_match(attrs) {
        return None;
    }

    let mut new_attrs = attrs.to_string();

    // Classes: merge into the existing class attribute or append a new one.
    if let Some(extra) = extra_cls.map(str::trim).filter(|s| !s.is_empty()) {
        let class_re = attr_re(&CLASS_ATTR_RE, "class");
        if let Some(c) = class_re.captures(&new_attrs) {
            let mut tokens: Vec<String> = attr_value(&c).split_whitespace().map(String::from).collect();
            for t in extra.split_whitespace() {
                if !tokens.iter().any(|x| x == t) {
                    tokens.push(t.to_string());
                }
            }
            let lead = c.get(1).map(|m| m.as_str()).unwrap_or("");
            let replacement = format!("{}class=\"{}\"", lead, escape_attr(&tokens.join(" ")));
            let range = c.get(0)?.range();
            new_attrs.replace_range(range, &replacement);
        } else {
            new_attrs.push_str(&format!(" class=\"{}\"", escape_attr(extra)));
        }
    }

    // Inline styles: append after the root's own inline styles so builder values win.
    if !css.trim().is_empty() {
        let style_re = attr_re(&STYLE_ATTR_RE, "style");
        if let Some(s) = style_re.captures(&new_attrs) {
            let existing = attr_value(&s).trim().trim_end_matches(';').replace('"', "'");
            let lead = s.get(1).map(|m| m.as_str()).unwrap_or("");
            let merged = if existing.is_empty() {
                css.trim().to_string()
            } else {
                format!("{}; {}", existing, css.trim())
            };
            let range = s.get(0)?.range();
            new_attrs.replace_range(range, &format!("{}style=\"{}\"", lead, merged));
        } else {
            new_attrs.push_str(&format!(" style=\"{}\"", css.trim()));
        }
    }

    let close = if caps.get(3).map(|m| !m.as_str().is_empty()).unwrap_or(false) { " />" } else { ">" };
    Some(format!(
        "<{} id=\"{}\"{}{}{}",
        tag,
        escape_attr(id),
        new_attrs,
        close,
        &trimmed[open_end..]
    ))
}

#[cfg(test)]
mod tests {
    use super::inject_root_attrs;

    #[test]
    fn injects_onto_single_root() {
        let out = inject_root_attrs(
            r#"<section class="section white" style="color:red"><div><section>x</section></div></section>"#,
            "n1",
            Some("hero"),
            "background-color: #000;",
        )
        .unwrap();
        assert!(out.starts_with(r#"<section id="n1" class="section white hero" style="color:red; background-color: #000;">"#), "{}", out);
        assert!(out.ends_with("</section>"));
    }

    #[test]
    fn rejects_multiple_roots_and_existing_ids() {
        assert!(inject_root_attrs("<div>a</div><div>b</div>", "n", None, "color: red;").is_none());
        assert!(inject_root_attrs(r#"<div id="keep">a</div>"#, "n", None, "color: red;").is_none());
        assert!(inject_root_attrs("plain text", "n", None, "color: red;").is_none());
    }

    #[test]
    fn handles_void_root() {
        let out = inject_root_attrs(r#"<img src="a.png">"#, "n", None, "opacity: 0.5;").unwrap();
        assert_eq!(out, r#"<img id="n" src="a.png" style="opacity: 0.5;">"#);
    }
}
