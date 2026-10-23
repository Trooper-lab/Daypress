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

        let has_data = kind_obj.and_then(|o| o.get("data")).is_some();
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
            if let Some(d) = obj.get("data") {
                map.serialize_entry("data", d)?;
            }
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
            Some(v) if !v.is_null() => serde_json::from_value(v.clone()).unwrap_or_default(),
            _ => StyleProperties::default(),
        };
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
