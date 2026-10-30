//! # Admin Post & Page Editor (`templates/admin/editor`)
//!
//! Splits the WordPress Post/Page Editor into SSR markup (`markup.rs`) and client-side editor scripts (`scripts.rs`).

mod markup;
mod scripts;

pub use markup::render_post_new_view;
pub use scripts::render_editor_scripts;
