use crate::schema::node::{NeopressNode, NodeKind};
use crate::schema::styles::StyleProperties;
use serde::{Deserialize, Serialize};

/// The canonical JSON layout container representation for Neopress.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NeopressLayout {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_root_node")]
    pub root: NeopressNode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global_styles: Option<serde_json::Value>,
}

pub fn default_version() -> u32 {
    1
}

pub fn default_root_node() -> NeopressNode {
    NeopressNode {
        id: "sec_root".to_string(),
        kind: NodeKind::Section,
        styles: StyleProperties::default(),
        children: Vec::new(),
    }
}

impl Default for NeopressLayout {
    fn default() -> Self {
        Self {
            version: 1,
            root: default_root_node(),
            global_styles: None,
        }
    }
}
