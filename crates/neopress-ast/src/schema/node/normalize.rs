use crate::schema::styles::StyleProperties;
use serde_json::{Map, Value};

pub(crate) fn normalize_node_kind_object(
    obj: &Map<String, Value>,
    styles: &StyleProperties,
) -> Value {
    let node_type = obj
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("Section");

    let mut out = Map::new();
    out.insert("type".to_string(), Value::String(node_type.to_string()));

    if matches!(node_type, "Root" | "Section" | "Container" | "Column") {
        return Value::Object(out);
    }

    let mut data_map = match obj.get("data") {
        Some(Value::Object(m)) => m.clone(),
        _ => {
            // Support flat Visual Builder payloads where data fields are placed directly on the node object
            let mut m = Map::new();
            for (k, v) in obj {
                if !matches!(k.as_str(), "id" | "type" | "styles" | "children") {
                    m.insert(k.clone(), v.clone());
                }
            }
            m
        }
    };

    if node_type == "Spacer" && !data_map.contains_key("height") {
        if let Some(ref h_str) = styles.height {
            let digits: String = h_str.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(h) = digits.parse::<u64>() {
                data_map.insert("height".to_string(), Value::Number(h.into()));
            }
        }
    }

    if node_type == "Counter" {
        if let Some(Value::String(s)) = data_map.get("number") {
            if s.trim().ends_with('%') && !data_map.contains_key("suffix") {
                data_map.insert("suffix".to_string(), Value::String("%".to_string()));
            }
        }
    }

    out.insert("data".to_string(), Value::Object(data_map));
    Value::Object(out)
}
