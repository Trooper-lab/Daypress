//! # HTTP SSR & Admin Portal Handlers (`src/handlers/`)
//!
//! Modularized into 4 focused handler groups:
//! * [`public_ssr`]: Front-page, single post/page, category/tag/author archives, search, `sitemap.xml`, and `robots.txt`.
//! * [`admin_pages`]: Dashboard, Posts, Visual Studio Builder, Media, Users, Login/Logout, and Comments screens.
//! * [`admin_actions`]: Customizer, Menus, Widgets, Theme Editor, WXR/JSON Export, GDPR Privacy, and Options persistence.
//! * [`native_modules`]: Cookieless Analytics, SEO & 301 Redirects, Declarative Forms, SQLite Hot Backups, and Plugin SDK.
//! * [`lost_password`]: "Lost your password?" request and reset-link screens.

pub mod admin_actions;
pub mod admin_extra;
pub mod admin_pages;
pub mod archives_ssr;
pub mod cpt_archive_render;
pub mod lost_password;
pub mod native_modules;
pub mod params;
pub mod public_ssr;
pub mod seo_feeds;
pub mod taxonomy_archive_render;

pub use admin_actions::*;
pub use admin_extra::*;
pub use admin_pages::*;
pub use archives_ssr::*;
pub use cpt_archive_render::*;
pub use lost_password::*;
pub use native_modules::*;
pub use params::*;
pub use public_ssr::*;
pub use seo_feeds::*;
pub use taxonomy_archive_render::*;
