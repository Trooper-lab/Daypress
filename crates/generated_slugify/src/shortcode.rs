use regex::Regex;
use std::collections::HashMap;

/// WordPress `shortcode_parse_atts()` port:
/// Parses shortcode attribute strings (`foo="bar" baz='qux' id=42`) into a key-value map.
pub fn shortcode_parse_atts(text: &str) -> HashMap<String, String> {
    let mut atts = HashMap::new();
    let re = Regex::new(
        r#"([a-zA-Z0-9_-]+)\s*=\s*"([^"]*)"|([a-zA-Z0-9_-]+)\s*=\s*'([^']*)'|([a-zA-Z0-9_-]+)\s*=\s*([^\s"']+)"#,
    )
    .expect("valid shortcode_parse_atts regex");

    for cap in re.captures_iter(text) {
        if let (Some(k), Some(v)) = (cap.get(1), cap.get(2)) {
            atts.insert(k.as_str().to_lowercase(), v.as_str().to_string());
        } else if let (Some(k), Some(v)) = (cap.get(3), cap.get(4)) {
            atts.insert(k.as_str().to_lowercase(), v.as_str().to_string());
        } else if let (Some(k), Some(v)) = (cap.get(5), cap.get(6)) {
            atts.insert(k.as_str().to_lowercase(), v.as_str().to_string());
        }
    }

    atts
}

/// WordPress `shortcode_atts()` port:
/// Combines user shortcode attributes with known default key-value pairs, ignoring unknown keys.
pub fn shortcode_atts(
    defaults: &[(&str, &str)],
    atts: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut out = HashMap::with_capacity(defaults.len());
    for &(key, default_val) in defaults {
        if let Some(val) = atts.get(key).or_else(|| atts.get(&key.to_lowercase())) {
            out.insert(key.to_string(), val.clone());
        } else {
            out.insert(key.to_string(), default_val.to_string());
        }
    }
    out
}
