//! # Public SSR Templates (`templates/public`)
//!
//! Modularized into focused template modules:
//! * [`shell`]: Base HTML document shell, SEO/OpenGraph/JSON-LD head, header, footer, and lightbox.
//! * [`styles`]: Core responsive public theme CSS variables and rules.
//! * [`home`]: Dynamic front page & blog index AST renderer.
//! * [`single`]: Single post/page AST renderer and interactive comments section.
//! * [`archives`]: Category, tag, author, search results, and 404 templates.
//! * [`rss`]: RSS 2.0 feed documents.

pub mod admin_bar;
pub mod archives;
pub mod head_meta;
pub mod home;
pub mod menus;
pub mod pagination;
pub mod rss;
pub mod shell;
pub mod single;
pub(crate) mod styles;

pub use archives::{
    render_archive_page, render_archive_page_paginated, render_not_found_page, render_search_page,
};
pub use home::{render_home_page, render_home_page_paginated};
pub use menus::*;
pub use shell::render_base_layout;
pub use single::render_single_post;
pub use styles::{render_public_css, render_public_theme_css, CSS_WIDGETS};
