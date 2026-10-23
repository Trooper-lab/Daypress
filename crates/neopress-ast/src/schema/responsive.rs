use serde::{Deserialize, Serialize};

/// Target responsive breakpoint device category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Breakpoint {
    Desktop,
    Tablet,
    Mobile,
}

impl Default for Breakpoint {
    fn default() -> Self {
        Breakpoint::Desktop
    }
}

/// Interactive pseudo-state for styling controls.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PseudoState {
    Default,
    Hover,
    Focus,
    Active,
}

impl Default for PseudoState {
    fn default() -> Self {
        PseudoState::Default
    }
}
