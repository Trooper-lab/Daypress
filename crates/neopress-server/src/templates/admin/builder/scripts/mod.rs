//! # Visual Builder Client Scripts (`templates/admin/builder/scripts`)
//!
//! Modularized into 6 focused, self-contained script modules and concatenated once
//! into a static string via `OnceLock`.

pub mod ast_and_globals;
pub mod canvas_and_drag;
pub mod inspector_content;
pub mod inspector_events;
pub mod inspector_styles;
pub mod navigator_and_modals;

use std::sync::OnceLock;

/// Returns the complete Visual Builder client-side JavaScript bundle.
pub fn get_scripts() -> &'static str {
    static BUNDLE: OnceLock<String> = OnceLock::new();
    BUNDLE.get_or_init(|| {
        [
            ast_and_globals::SCRIPT_AST_AND_GLOBALS,
            canvas_and_drag::SCRIPT_CANVAS_AND_DRAG,
            inspector_content::SCRIPT_INSPECTOR_CONTENT,
            inspector_styles::SCRIPT_INSPECTOR_STYLES,
            inspector_events::SCRIPT_INSPECTOR_EVENTS,
            navigator_and_modals::SCRIPT_NAVIGATOR_AND_MODALS,
        ]
        .concat()
    })
}
