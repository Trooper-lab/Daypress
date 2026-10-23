use serde_json::{Map, Value};

pub(crate) fn extract_global_color(globals: &Map<String, Value>, key: &str) -> Option<String> {
    let val = globals.get(key)?.as_str()?;
    if let Some(idx) = val.find("id=") {
        let id = &val[idx + 3..];
        let clean_id: String = id
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !clean_id.is_empty() {
            return Some(format!("var(--e-global-color-{})", clean_id));
        }
    }
    None
}

pub(crate) fn extract_global_typography(globals: &Map<String, Value>, key: &str) -> Option<String> {
    let val = globals.get(key)?.as_str()?;
    if let Some(idx) = val.find("id=") {
        let id = &val[idx + 3..];
        let clean_id: String = id
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
            .collect();
        if !clean_id.is_empty() {
            return Some(clean_id);
        }
    }
    None
}
