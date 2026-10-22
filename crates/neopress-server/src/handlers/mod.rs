//! # HTTP SSR & Admin Portal Handlers (`src/handlers/`)
//!
//! Modularized into 4 focused handler groups:
//! * [`public_ssr`]: Front-page, single post/page, category/tag/author archives, search, `sitemap.xml`, and `robots.txt`.
//! * [`admin_pages`]: Dashboard, Posts, Visual Studio Builder, Media, Users, Login/Logout, and Comments screens.
//! * [`admin_actions`]: Customizer, Menus, Widgets, Theme Editor, WXR/JSON Export, GDPR Privacy, and Options persistence.
//! * [`native_modules`]: Cookieless Analytics, SEO & 301 Redirects, Declarative Forms, SQLite Hot Backups, and Plugin SDK.

pub mod admin_actions;
pub mod admin_pages;
pub mod native_modules;
pub mod public_ssr;

pub use admin_actions::*;
pub use admin_pages::*;
pub use native_modules::*;
pub use public_ssr::*;
