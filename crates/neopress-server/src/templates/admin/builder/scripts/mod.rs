//! NeoStudio 2026 Visual Builder Client Scripts (`scripts/mod.rs`).
//!
//! Subdivided into focused single-responsibility JavaScript modules strictly <= 400 lines,
//! concatenated once into a static string via `OnceLock`.

pub mod api_client;
pub mod canvas_drag;
pub mod canvas_overlays;
pub mod canvas_pro_commercial;
pub mod canvas_pro_dynamic;
pub mod canvas_pro_interactive;
pub mod canvas_pro_widgets;
pub mod canvas_render;
pub mod canvas_selection;
pub mod command_palette;
pub mod core_state;
pub mod css_drawer;
pub mod drawers;
pub mod inspector_box_model;
pub mod inspector_content;
pub mod inspector_controls;
pub mod inspector_core;
pub mod inspector_effects;
pub mod inspector_layout;
pub mod inspector_styles;
pub mod modals;
pub mod navigator;
pub mod palette_defaults;
pub mod style_engine;
pub mod theme_drawer;
pub mod tokens_drawer;
pub mod topbar;
pub mod topbar_switcher;

use std::sync::OnceLock;

/// Returns the complete NeoStudio 2026 Visual Builder client JavaScript bundle.
pub fn get_scripts() -> &'static str {
    static BUNDLE: OnceLock<String> = OnceLock::new();
    BUNDLE.get_or_init(|| {
        [
            style_engine::JS_STYLE_ENGINE,
            core_state::JS_CORE_STATE,
            api_client::JS_API_CLIENT,
            topbar::JS_TOPBAR,
            topbar_switcher::JS_TOPBAR_SWITCHER,
            canvas_render::JS_CANVAS_RENDER,
            canvas_pro_commercial::JS_CANVAS_PRO_COMMERCIAL,
            canvas_pro_interactive::JS_CANVAS_PRO_INTERACTIVE,
            canvas_pro_dynamic::JS_CANVAS_PRO_DYNAMIC,
            canvas_pro_widgets::JS_CANVAS_PRO_WIDGETS,
            canvas_selection::JS_CANVAS_SELECTION,
            canvas_overlays::JS_CANVAS_OVERLAYS,
            canvas_drag::JS_CANVAS_DRAG,
            navigator::JS_NAVIGATOR,
            palette_defaults::JS_PALETTE_DEFAULTS,
            tokens_drawer::JS_TOKENS_DRAWER,
            css_drawer::JS_CSS_DRAWER,
            drawers::JS_DRAWERS,
            theme_drawer::JS_THEME_DRAWER,
            command_palette::JS_COMMAND_PALETTE,
            inspector_core::JS_INSPECTOR_CORE,
            inspector_controls::JS_INSPECTOR_CONTROLS,
            inspector_box_model::JS_INSPECTOR_BOX_MODEL,
            inspector_layout::JS_INSPECTOR_LAYOUT,
            inspector_styles::JS_INSPECTOR_STYLES,
            inspector_effects::JS_INSPECTOR_EFFECTS,
            inspector_content::JS_INSPECTOR_CONTENT,
            modals::JS_MODALS,
        ]
        .concat()
    })
}
