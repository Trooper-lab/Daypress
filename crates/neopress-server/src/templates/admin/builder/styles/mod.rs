//! NeoStudio 2026 Visual Builder Stylesheets (`styles/mod.rs`).
//!
//! Subdivided into focused single-responsibility CSS modules and concatenated once via `OnceLock`.

pub mod box_model;
pub mod canvas;
pub mod drawers;
pub mod inspector_base;
pub mod inspector_panels;
pub mod modals;
pub mod navigator;
pub mod overlays;
pub mod shell;
pub mod tokens;
pub mod topbar;
pub mod widgets;

use std::sync::OnceLock;

/// Returns the complete NeoStudio 2026 CAD CSS stylesheet.
pub fn get_styles() -> &'static str {
    static STYLES: OnceLock<String> = OnceLock::new();
    STYLES.get_or_init(|| {
        [
            tokens::CSS_TOKENS,
            shell::CSS_SHELL,
            topbar::CSS_TOPBAR,
            canvas::CSS_CANVAS,
            overlays::CSS_OVERLAYS,
            drawers::CSS_DRAWERS,
            navigator::CSS_NAVIGATOR,
            inspector_base::CSS_INSPECTOR_BASE,
            box_model::CSS_BOX_MODEL,
            inspector_panels::CSS_INSPECTOR_PANELS,
            widgets::CSS_WIDGETS,
            modals::CSS_MODALS,
        ]
        .concat()
    })
}
