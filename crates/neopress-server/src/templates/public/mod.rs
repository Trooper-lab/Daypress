//! # Public SSR Templates (`templates/public`)
//!
//! Modularized into focused template modules:
//! * [`shell`]: Base HTML document shell, SEO/OpenGraph/JSON-LD head, header, footer, and lightbox.
//! * [`styles`]: Core responsive public theme CSS variables and rules.
//! * [`home`]: Dynamic front page & blog index AST renderer.
//! * [`single`]: Single post/page AST renderer and interactive comments section.
//! * [`archives`]: Category, tag, author, search results, and 404 templates.

pub mod archives;
pub mod home;
pub mod shell;
pub mod single;
mod styles;

pub use archives::{render_archive_page, render_not_found_page, render_search_page};
pub use home::render_home_page;
pub use shell::render_base_layout;
pub use single::render_single_post;
pub use styles::render_public_theme_css;
