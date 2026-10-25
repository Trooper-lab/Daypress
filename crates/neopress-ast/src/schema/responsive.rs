use serde::{Deserialize, Serialize};

/// Target responsive breakpoint device category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Breakpoint {
    #[default]
    Desktop,
    Tablet,
    Mobile,
}

/// Interactive pseudo-state for styling controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PseudoState {
    #[default]
    Default,
    Hover,
    Focus,
    Active,
}
