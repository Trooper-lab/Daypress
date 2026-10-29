//! Admin Layout module.

mod markup;
mod navigation;
mod styles_components;
mod styles_forms;
mod styles_shell;

pub use markup::render_wp_admin_shell;
pub use navigation::{format_cpt_title, render_cpt_navigation, NavState};
pub use styles_shell::{DESK_FONT_FACES, DESK_FOUNDATION};
