pub mod defaults;
pub mod from_kind;
pub mod from_repr;
pub mod kind;
pub mod normalize;
pub mod repr;
#[cfg(test)]
pub mod tests;

pub use kind::NodeKind;
pub(crate) use normalize::normalize_node_kind_object;

use crate::schema::styles::StyleProperties;
use serde::de::{self, Deserializer};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A node within the Neopress Abstract Syntax Tree.
#[derive(Debug, Clone, PartialEq)]
pub struct NeopressNode {
    pub id: String,
    pub kind: NodeKind,
    pub styles: StyleProperties,
    pub children: Vec<NeopressNode>,
}

impl Serialize for NeopressNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let kind_val = serde_json::to_value(&self.kind).map_err(serde::ser::Error::custom)?;
        let kind_obj = kind_val.as_object();

        let tag_override = self
            .styles
            .html_tag
            .as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty());

        let mut data_val = kind_obj.and_then(|o| o.get("data")).cloned();
        if let Some(tag) = tag_override {
            match &mut data_val {
                Some(Value::Object(m)) => {
                    m.entry("html_tag".to_string())
                        .or_insert_with(|| Value::String(tag.to_string()));
                }
                _ => {
                    let mut m = serde_json::Map::new();
                    m.insert("html_tag".to_string(), Value::String(tag.to_string()));
                    data_val = Some(Value::Object(m));
                }
            }
        }

        let has_data = data_val.is_some();
        let has_children = !self.children.is_empty();

        let mut field_count = 3; // id, type, styles
        if has_data {
            field_count += 1;
        }
        if has_children {
            field_count += 1;
        }

        let mut map = serializer.serialize_map(Some(field_count))?;
        map.serialize_entry("id", &self.id)?;
        if let Some(obj) = kind_obj {
            if let Some(t) = obj.get("type") {
                map.serialize_entry("type", t)?;
            }
        }
        if let Some(ref d) = data_val {
            map.serialize_entry("data", d)?;
        }
        map.serialize_entry("styles", &self.styles)?;
        if has_children {
            map.serialize_entry("children", &self.children)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for NeopressNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = Value::deserialize(deserializer)?;
        let obj = val
            .as_object()
            .ok_or_else(|| de::Error::custom("NeopressNode must be a JSON object"))?;

        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("node")
            .to_string();

        let mut styles: StyleProperties = match obj.get("styles") {
            Some(v) if !v.is_null() => {
                serde_json::from_value(coerce_style_scalars(v.clone())).unwrap_or_default()
            }
            _ => StyleProperties::default(),
        };

        // Also accept `html_tag` or `tag` placed directly on `node` or inside `node.data`
        if styles.html_tag.as_deref().map(str::trim).unwrap_or("").is_empty() {
            let fallback_tag = obj
                .get("html_tag")
                .or_else(|| obj.get("tag"))
                .or_else(|| {
                    obj.get("data")
                        .and_then(|d| d.get("html_tag").or_else(|| d.get("tag")))
                })
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty());
            if let Some(t) = fallback_tag {
                styles.html_tag = Some(t.to_ascii_lowercase());
            }
        }

        styles.sync_flat_and_nested();

        let children: Vec<NeopressNode> = match obj.get("children") {
            Some(v) if !v.is_null() => serde_json::from_value(v.clone()).unwrap_or_default(),
            _ => Vec::new(),
        };

        let normalized_kind_val = normalize_node_kind_object(obj, &styles);
        let kind: NodeKind =
            serde_json::from_value(normalized_kind_val).map_err(de::Error::custom)?;

        Ok(NeopressNode {
            id,
            kind,
            styles,
            children,
        })
    }
}

/// Makes a raw `styles` JSON object safe to deserialize into [`StyleProperties`].
///
/// Every flat style field is an `Option<String>`, so a single numeric value (e.g. `"z_index": 2`
/// or `"opacity": 0.5` written by a builder or importer) would make serde reject the whole object
/// and silently drop *all* styles. Numbers are stringified, and stray booleans/arrays on string
/// fields are discarded. `hide_*` flags stay boolean and `raw_settings` is left untouched.
pub(crate) fn coerce_style_scalars(val: Value) -> Value {
    match val {
        Value::Object(map) => {
            let mut out = serde_json::Map::with_capacity(map.len());
            for (k, v) in map {
                if k == "raw_settings" {
                    out.insert(k, v);
                    continue;
                }
                if k.starts_with("hide_") {
                    let b = match v {
                        Value::Bool(b) => Value::Bool(b),
                        Value::Number(n) => Value::Bool(n.as_f64().unwrap_or(0.0) != 0.0),
                        Value::String(s) => Value::Bool(matches!(s.as_str(), "true" | "1" | "yes")),
                        _ => Value::Null,
                    };
                    out.insert(k, b);
                    continue;
                }
                let coerced = match v {
                    Value::Number(n) => Value::String(n.to_string()),
                    Value::Bool(_) | Value::Array(_) => Value::Null,
                    obj @ Value::Object(_) => coerce_style_scalars(obj),
                    other => other,
                };
                out.insert(k, coerced);
            }
            Value::Object(out)
        }
        other => other,
    }
}
