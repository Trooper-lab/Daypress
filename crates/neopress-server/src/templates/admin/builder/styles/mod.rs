//! # Visual Builder Stylesheets (`templates/admin/builder/styles`)
//!
//! Modularized into focused CSS segments concatenated once via `OnceLock`.

pub mod canvas_and_widgets;
pub mod modals_and_globals;
pub mod panel_and_controls;

use std::sync::OnceLock;

/// Returns the complete Visual Builder CSS stylesheet.
pub fn get_styles() -> &'static str {
    static STYLES: OnceLock<String> = OnceLock::new();
    STYLES.get_or_init(|| {
        [
            panel_and_controls::STYLE_PANEL_AND_CONTROLS,
            canvas_and_widgets::STYLE_CANVAS_AND_WIDGETS,
            modals_and_globals::STYLE_MODALS_AND_GLOBALS,
        ]
        .concat()
    })
}
