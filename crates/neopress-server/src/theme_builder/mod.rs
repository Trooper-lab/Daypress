//! # Theme Template Builder Subsystem (`theme_builder/`)
//!
//! Provides SQLite persistence, display condition specificity evaluation,
//! dynamic placeholder AST hydration, REST APIs, starter templates, and
//! standalone preview rendering.

pub mod conditions;
pub mod dto;
pub mod matrix;
pub mod models;
pub mod preview;
pub mod repository;
pub mod repository_ops;
pub mod routes;
pub mod schema;
pub mod ssr_injector;
pub mod starters;

pub use conditions::{evaluate_condition, evaluate_template, resolve_template_for_type};
pub use matrix::{
    assign_page_or_route_templates, build_header_footer_matrix,
    ensure_legacy_headers_footers_synced, format_conditions_summary, resolve_slot_status,
};
pub use models::*;
pub use repository::ThemeTemplateRepository;
pub use routes::theme_builder_routes;
pub use schema::init_theme_builder_schema;
pub use ssr_injector::{HydrationContext, SsrInjector};
pub use starters::{build_starter_template, get_starter_catalog};
