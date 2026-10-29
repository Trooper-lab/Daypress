use crate::schema::node::{DaypressNode, NodeKind};
use crate::schema::styles::StyleProperties;
use serde::{Deserialize, Serialize};

/// The canonical JSON layout container representation for Daypress.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DaypressLayout {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_root_node")]
    pub root: DaypressNode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global_styles: Option<serde_json::Value>,
}

pub fn default_version() -> u32 {
    1
}

pub fn default_root_node() -> DaypressNode {
    DaypressNode {
        id: "sec_root".to_string(),
        kind: NodeKind::Section,
        styles: StyleProperties::default(),
        children: Vec::new(),
    }
}

impl Default for DaypressLayout {
    fn default() -> Self {
        Self {
            version: 1,
            root: default_root_node(),
            global_styles: None,
        }
    }
}
