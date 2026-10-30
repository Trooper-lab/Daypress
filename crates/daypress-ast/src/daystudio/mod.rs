//! # Elementor JSON-to-AST Transpiler (`daypress-ast/src/elementor`)
//!
//! Modularized into 4 focused compilation units:
//! * [`context`]: `DaystudioContext`, dynamic tags (`__dynamic__`), video embed normalization, and top-level entrypoints.
//! * [`containers`]: Section, Column, and Flexbox/Grid Container hierarchy transpilation.
//! * [`widgets`]: 25+ Elementor widget mappers (`heading`, `button`, `form`, `nav-menu`, `accordion`, `tabs`, etc.).
//! * [`styles`]: `StyleProperties`, global colors/typography (`__globals__`), spacing, borders, and backgrounds.

pub mod containers;
pub mod context;
pub mod kit;
pub mod styles;
pub mod widgets;

pub(crate) use containers::*;
pub use context::*;
pub(crate) use styles::*;
pub(crate) use widgets::*;
