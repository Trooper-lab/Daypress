use regex::Regex;
use std::sync::OnceLock;

static CONTENTEDITABLE_RE: OnceLock<Regex> = OnceLock::new();
static DRAGGABLE_RE: OnceLock<Regex> = OnceLock::new();
static DATA_ATTRS_RE: OnceLock<Regex> = OnceLock::new();
static EDITOR_CLASSES_RE: OnceLock<Regex> = OnceLock::new();
static CLASS_ATTR_RE: OnceLock<Regex> = OnceLock::new();

/// Strips editor-specific class names from class attributes or HTML,
/// cleaning up any empty class attributes or redundant spaces left behind.
pub fn strip_editor_classes(raw: &str) -> String {
    let re = EDITOR_CLASSES_RE.get_or_init(|| {
        Regex::new(r#"(?i)\b(elementor-grid-handle|elementor-element-edit-mode|elementor-editor-active|ui-resizable|ui-draggable|ui-sortable|active-element|et_pb_element_editing|visual-builder-active|neostudio-handle|neostudio(?:-[a-z0-9_\-]+)?)\b"#).unwrap()
    });
    let stripped = re.replace_all(raw, "");

    // If input contains HTML class attributes, clean up empty class artifacts
    if stripped.contains('<')
        && (stripped.contains("class=")
            || stripped.contains("CLASS=")
            || stripped.contains("class ="))
    {
        clean_empty_class_artifacts(&stripped)
    } else if !stripped.contains('<') {
        // Plain class list string: normalize whitespace
        stripped.split_whitespace().collect::<Vec<_>>().join(" ")
    } else {
        stripped.to_string()
    }
}

/// Cleans up empty class attributes (`class=""`, `class="   "`, `class=''`)
/// and normalizes whitespace in remaining classes.
pub fn clean_empty_class_artifacts(html: &str) -> String {
    let class_attr_re = CLASS_ATTR_RE
        .get_or_init(|| Regex::new(r#"(?i)\s*class=(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#).unwrap());

    class_attr_re
        .replace_all(html, |caps: &regex::Captures| {
            let val = caps
                .get(1)
                .or_else(|| caps.get(2))
                .or_else(|| caps.get(3))
                .map(|m| m.as_str())
                .unwrap_or("");
            let tokens: Vec<&str> = val.split_whitespace().collect();
            if tokens.is_empty() {
                "".to_string()
            } else {
                format!(" class=\"{}\"", tokens.join(" "))
            }
        })
        .to_string()
}

/// Cleans editor-only attributes (`contenteditable`, `draggable`, `data-*`) and active classes.
pub fn sanitize_editor_html(raw: &str) -> String {
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

    let s0 = strip_editor_classes(raw);
    let s1 = ce_re.replace_all(&s0, "");
    let s2 = drag_re.replace_all(&s1, "");
    data_re.replace_all(&s2, "").to_string()
}

/// Final 3-tier SSR firewall ensuring strict zero-DOM-leak guarantee for public SSR output.
///
/// Tier 1: Class Firewall - Strips all Elementor, Divi, jQuery UI, and NeoStudio editor/active classes.
/// Tier 2: Attribute Firewall - Strips case-insensitive `contenteditable`, `draggable`, and builder data attributes.
/// Tier 3: Artifact Cleanup - Eliminates empty `class=""` artifacts and normalizes attribute whitespace.
pub fn sanitize_ssr_html(html: &str) -> String {
    let sanitized = sanitize_editor_html(html);
    clean_empty_class_artifacts(&sanitized)
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
