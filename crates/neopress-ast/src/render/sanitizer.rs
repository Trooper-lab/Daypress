use regex::Regex;
use std::sync::OnceLock;

static START_TAG_RE: OnceLock<Regex> = OnceLock::new();
static CONTENTEDITABLE_RE: OnceLock<Regex> = OnceLock::new();
static DRAGGABLE_RE: OnceLock<Regex> = OnceLock::new();
static DATA_ATTRS_RE: OnceLock<Regex> = OnceLock::new();
static EDITOR_CLASS_RE: OnceLock<Regex> = OnceLock::new();
static CLASS_ATTR_RE: OnceLock<Regex> = OnceLock::new();

/// Allowed semantic HTML5 tags for `Container`, `Text`, and layout AST nodes.
pub fn resolve_semantic_tag<'a>(requested: Option<&'a str>, default_tag: &'a str) -> &'a str {
    let Some(raw) = requested.map(str::trim).filter(|s| !s.is_empty()) else {
        return default_tag;
    };
    match raw.to_ascii_lowercase().as_str() {
        "div" => "div",
        "section" => "section",
        "article" => "article",
        "aside" => "aside",
        "nav" => "nav",
        "header" => "header",
        "footer" => "footer",
        "main" => "main",
        "figure" => "figure",
        "figcaption" => "figcaption",
        "blockquote" => "blockquote",
        "dl" => "dl",
        "dt" => "dt",
        "dd" => "dd",
        "ol" => "ol",
        "ul" => "ul",
        "li" => "li",
        "p" => "p",
        "span" => "span",
        "cite" => "cite",
        "code" => "code",
        "pre" => "pre",
        "details" => "details",
        "summary" => "summary",
        "address" => "address",
        "time" => "time",
        "mark" => "mark",
        "small" => "small",
        "strong" => "strong",
        "em" => "em",
        "b" => "b",
        "i" => "i",
        "kbd" => "kbd",
        "sub" => "sub",
        "sup" => "sup",
        "table" => "table",
        "thead" => "thead",
        "tbody" => "tbody",
        "tfoot" => "tfoot",
        "tr" => "tr",
        "th" => "th",
        "td" => "td",
        "caption" => "caption",
        "hgroup" => "hgroup",
        "label" => "label",
        "fieldset" => "fieldset",
        "legend" => "legend",
        "button" => "button",
        _ => default_tag,
    }
}

/// True for class names that only the Elementor, Divi, jQuery UI or NeoStudio editors add.
fn is_editor_class(token: &str) -> bool {
    EDITOR_CLASS_RE
        .get_or_init(|| {
            Regex::new(r#"(?i)^(?:elementor-grid-handle|elementor-element-edit-mode|elementor-editor-active|ui-(?:resizable|draggable|sortable)(?:-[a-z0-9_\-]+)?|active-element|et_pb_element_editing|visual-builder-active|neostudio(?:-[a-z0-9_\-]+)?)$"#).unwrap()
        })
        .is_match(token)
}

/// Applies `rewrite` to every start tag (`<name ...>`). Text, comments and end tags are left
/// alone, so words such as "NeoStudio" or "draggable" in content and links survive.
fn map_start_tags(html: &str, rewrite: impl Fn(&str) -> String) -> String {
    let re = START_TAG_RE
        .get_or_init(|| Regex::new(r#"<[A-Za-z](?:[^>"']|"[^"]*"|'[^']*')*>"#).unwrap());
    re.replace_all(html, |caps: &regex::Captures| rewrite(&caps[0]))
        .into_owned()
}

/// Normalizes the class attributes of one start tag: optionally drops editor-only class names,
/// collapses whitespace, and removes the attribute when no class is left.
fn rewrite_class_attrs(tag: &str, drop_editor_classes: bool) -> String {
    let re = CLASS_ATTR_RE.get_or_init(|| {
        Regex::new(r#"(?i)\s+class\s*=\s*(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#).unwrap()
    });
    re.replace_all(tag, |caps: &regex::Captures| {
        let val = caps
            .get(1)
            .or_else(|| caps.get(2))
            .or_else(|| caps.get(3))
            .map(|m| m.as_str())
            .unwrap_or("");
        let tokens: Vec<&str> = val
            .split_whitespace()
            .filter(|t| !(drop_editor_classes && is_editor_class(t)))
            .collect();
        if tokens.is_empty() {
            String::new()
        } else {
            format!(" class=\"{}\"", tokens.join(" "))
        }
    })
    .into_owned()
}

/// Strips editor-specific class names, either from the class attributes of an HTML fragment
/// or from a plain, space-separated class list.
pub fn strip_editor_classes(raw: &str) -> String {
    if raw.contains('<') {
        map_start_tags(raw, |tag| rewrite_class_attrs(tag, true))
    } else {
        raw.split_whitespace()
            .filter(|t| !is_editor_class(t))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Cleans up empty class attributes (`class=""`, `class="   "`, `class=''`)
/// and normalizes whitespace in remaining classes.
pub fn clean_empty_class_artifacts(html: &str) -> String {
    map_start_tags(html, |tag| rewrite_class_attrs(tag, false))
}

fn sanitize_start_tag(tag: &str) -> String {
    // 1. Case-insensitive contenteditable (with or without value, bare or quoted)
    let ce_re = CONTENTEDITABLE_RE.get_or_init(|| {
        Regex::new(r#"(?i)\s*contenteditable(?:=(?:"[^"]*"|'[^']*'|[^\s>]+))?"#).unwrap()
    });
    // 2. Case-insensitive draggable (with or without value, bare or quoted)
    let drag_re = DRAGGABLE_RE.get_or_init(|| {
        Regex::new(r#"(?i)(?:\s+|^)draggable(?:=(?:"[^"]*"|'[^']*'|[^\s>]+))?"#).unwrap()
    });
    // 3. Case-insensitive editor data attributes (Elementor, Divi/NeoStudio, drag handle, model CID)
    let data_re = DATA_ATTRS_RE.get_or_init(|| {
        Regex::new(r#"(?i)\s*data-(?:elementor[\w\-]*|neostudio[\w\-]*|drag-handle|model-cid)(?:=(?:"[^"]*"|'[^']*'|[^\s>]+))?"#).unwrap()
    });

    let s1 = ce_re.replace_all(tag, "");
    let s2 = drag_re.replace_all(&s1, "");
    let s3 = data_re.replace_all(&s2, "");
    rewrite_class_attrs(&s3, true)
}

/// Cleans editor-only attributes (`contenteditable`, `draggable`, `data-*`) and active classes
/// from every start tag.
pub fn sanitize_editor_html(raw: &str) -> String {
    map_start_tags(raw, sanitize_start_tag)
}

/// Final 3-tier SSR firewall ensuring no editor-only markup reaches public SSR output.
///
/// Tier 1: Class Firewall - Strips all Elementor, Divi, jQuery UI, and NeoStudio editor/active classes.
/// Tier 2: Attribute Firewall - Strips case-insensitive `contenteditable`, `draggable`, and builder data attributes.
/// Tier 3: Artifact Cleanup - Eliminates empty `class=""` artifacts and normalizes attribute whitespace.
///
/// All three tiers only rewrite start tags; text content is never changed.
pub fn sanitize_ssr_html(html: &str) -> String {
    sanitize_editor_html(html)
}

pub(crate) fn escape_attr(val: &str) -> String {
    val.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub(crate) fn with_extra_classes(base_class: &str, extra: Option<&str>) -> String {
    match extra.map(|s| s.trim()).filter(|s| !s.is_empty()) {
        Some(ext) => format!("{} {}", base_class, ext),
        None => base_class.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_and_links_keep_editor_like_words() {
        let html = r#"<p class="np-text active-element">Built with NeoStudio. Make the list draggable or contenteditable.</p><a href="/neostudio-sample/">Sample</a>"#;
        assert_eq!(
            sanitize_ssr_html(html),
            r#"<p class="np-text">Built with NeoStudio. Make the list draggable or contenteditable.</p><a href="/neostudio-sample/">Sample</a>"#
        );
        assert_eq!(
            sanitize_editor_html("A page built with NeoStudio"),
            "A page built with NeoStudio"
        );
    }

    #[test]
    fn attributes_are_stripped_even_with_a_quoted_angle_bracket() {
        let html =
            r#"<div title="a > b" contenteditable="true" class="card neostudio-selected">x</div>"#;
        assert_eq!(
            sanitize_ssr_html(html),
            r#"<div title="a > b" class="card">x</div>"#
        );
    }
}
