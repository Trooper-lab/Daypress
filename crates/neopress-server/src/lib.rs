#![recursion_limit = "256"]
// SQLite row tuples and importer passes carry many fields by design.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

// Hexagonal Architecture Core Layers
pub mod adapters;
pub mod application;
pub mod domain;
pub mod ports;

// Canonical Module Exports
pub mod api;
pub mod auth;
pub mod convergence;
pub mod db;
pub mod email;
pub mod errors;
pub mod handlers;
pub mod importer;
pub mod media;
pub mod models;
pub mod password_reset;
pub mod plugins;
pub mod seed;
pub mod static_assets;
pub mod templates;
pub mod theme;
pub mod theme_builder;
pub mod wxr;

pub use adapters::database::SqliteDatabase;
pub use api::api_routes;
pub use application::CmsService;
pub use auth::AuthUser;
pub use convergence::*;
pub use db::Database;
pub use domain::errors::{DomainError, DomainResult};
pub use email::*;
pub use errors::AppError;
pub use handlers::*;
pub use media::*;
pub use models::*;
pub use plugins::*;
pub use ports::*;
pub use seed::*;
pub use theme::*;
pub use theme_builder::*;
pub use wxr::*;

use axum::{
    extract::Path as AxPath,
    response::Redirect,
    routing::{get, post},
    Router,
};
use std::path::Path;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

pub fn create_app(database: Database) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let static_dir = if Path::new("crates/neopress-server/static").exists() {
        "crates/neopress-server/static"
    } else {
        "static"
    };

    let _uploads_dir = if Path::new("uploads").exists() {
        "uploads"
    } else {
        let _ = std::fs::create_dir_all("uploads");
        "uploads"
    };

    Router::new()
        .route("/mcp", post(adapters::mcp::handle_http_mcp))
        .route(
            "/wp-json/neopress/v1/mcp",
            post(adapters::mcp::handle_http_mcp),
        )
        .route("/wp-json", get(api::wp_json_root))
        .route("/wp-json/", get(api::wp_json_root))
        .nest("/wp-json", api::api_routes())
        // Bundled assets are compiled into the binary; imported theme files stay on disk.
        .route("/static/{*path}", get(static_assets::serve_static))
        .route("/wp-includes/css/{*path}", get(static_assets::serve_static))
        .route("/wp-admin/css/{*path}", get(static_assets::serve_static))
        .route("/wp-includes/fonts/{*path}", get(static_assets::serve_font))
        .nest_service(
            "/wp-content/themes",
            ServeDir::new(format!("{}/themes", static_dir)),
        )
        .route("/uploads/{*path}", get(media::handle_serve_media))
        .route(
            "/wp-content/uploads/{*path}",
            get(media::handle_serve_media),
        )
        .route("/login", get(handle_login_page).post(handle_login_submit))
        .route(
            "/login/lost-password",
            get(handle_lost_password_page).post(handle_lost_password_submit),
        )
        .route(
            "/login/reset-password",
            get(handle_reset_password_page).post(handle_reset_password_submit),
        )
        .route("/logout", get(handle_logout))
        .route(
            "/n-admin/login",
            get(handle_login_page).post(handle_login_submit),
        )
        .route("/n-admin/logout", get(handle_logout))
        // 1. Dashboard
        .route(
            "/n-admin",
            get(|| async { Redirect::permanent("/n-admin/") }),
        )
        .route("/n-admin/", get(handle_admin_dashboard))
        .route("/n-admin/index", get(handle_admin_dashboard))
        .route("/n-admin/update-core", get(handle_admin_updates))
        // 2. Posts & Pages (both utilize edit and post-new with post_type query parameters)
        .route("/n-admin/edit", get(handle_admin_posts))
        .route("/n-admin/post-new", get(handle_admin_post_new))
        .route("/n-admin/post", get(handle_admin_post_new))
        .route("/n-admin/builder", get(handle_admin_builder))
        .route("/n-admin/studio", get(handle_admin_builder))
        .route("/studio", get(handle_admin_builder))
        .route("/n-admin/edit-tags", get(handle_admin_tags))
        // 3. Media
        .route("/n-admin/upload", get(handle_admin_media))
        .route("/n-admin/media-new", get(handle_admin_media_new))
        // 4. Comments
        .route("/n-admin/edit-comments", get(handle_admin_comments))
        // 5. Appearance
        // Theme switching, widgets and the file editors are not part of v0.1: send visitors of
        // those legacy screens to the tools that do the job.
        .route(
            "/n-admin/themes",
            get(|| async { Redirect::to("/n-admin/headers-footers") }),
        )
        .route(
            "/n-admin/headers-footers",
            get(handle_admin_headers_footers),
        )
        .route("/n-admin/theme-builder", get(handle_admin_headers_footers))
        .route("/n-admin/customize", get(handle_admin_customize))
        .route(
            "/n-admin/widgets",
            get(|| async { Redirect::to("/n-admin/headers-footers") }),
        )
        .route("/n-admin/nav-menus", get(handle_admin_nav_menus))
        .route(
            "/n-admin/theme-editor",
            get(|| async { Redirect::to("/n-admin/customize") }),
        )
        .route("/n-admin/api/theme", post(handle_api_theme_save))
        .route("/n-admin/api/customizer", post(handle_api_customizer_save))
        .route("/n-admin/api/widgets", post(handle_api_widgets_save))
        .route("/n-admin/api/menus", post(handle_api_menus_save))
        .route(
            "/n-admin/api/theme-file",
            get(handle_api_theme_file_get).post(handle_api_theme_file_save),
        )
        // 6. Plugins
        .route("/n-admin/plugins", get(handle_admin_plugins))
        .route("/n-admin/plugin-install", get(handle_admin_plugin_install))
        .route(
            "/n-admin/plugin-editor",
            get(|| async { Redirect::to("/n-admin/plugins") }),
        )
        .route(
            "/n-admin/plugin-settings",
            get(handle_admin_plugin_settings),
        )
        .route("/n-admin/api/plugins", post(handle_api_plugins_save))
        .route(
            "/n-admin/api/plugin-settings",
            post(handle_api_plugin_settings_save),
        )
        .route(
            "/n-admin/api/plugin-install",
            post(handle_api_plugin_install_custom),
        )
        .route(
            "/n-admin/api/plugin-sdk-template",
            get(handle_api_plugin_sdk_template),
        )
        .route(
            "/n-admin/api/download-migrator-plugin",
            get(handle_api_download_migrator_plugin),
        )
        // 6b. First-Party Modules (SEO & Redirects, Forms Studio, Hot Backups)
        .route("/n-admin/seo", get(handle_admin_seo))
        .route("/n-admin/neopress-seo", get(handle_admin_seo))
        .route("/n-admin/api/seo", post(handle_api_seo_save))
        .route("/n-admin/api/redirects", post(handle_api_redirects))
        .route("/n-admin/forms", get(handle_admin_forms))
        .route("/n-admin/neopress-forms", get(handle_admin_forms))
        .route("/n-admin/api/forms", post(handle_api_forms))
        .route(
            "/n-admin/api/forms/export-csv",
            get(handle_admin_forms_export_csv),
        )
        .route("/n-admin/backups", get(handle_admin_backups))
        .route("/n-admin/neopress-backups", get(handle_admin_backups))
        .route("/n-admin/api/backups", post(handle_api_backups))
        .route(
            "/n-admin/backups/download/{filename}",
            get(handle_admin_backup_download),
        )
        // 7. Users
        .route("/n-admin/users", get(handle_admin_users))
        .route(
            "/n-admin/user-new",
            get(handle_admin_user_new).post(handle_admin_user_new_submit),
        )
        .route("/n-admin/profile", get(handle_admin_profile))
        .route("/n-admin/user-edit", get(handle_admin_profile))
        // 8. Tools
        .route("/n-admin/tools", get(handle_admin_tools))
        .route("/n-admin/import", get(handle_admin_import))
        .route("/n-admin/export", get(handle_admin_export))
        .route("/n-admin/export-wxr", get(handle_admin_export_wxr_download))
        .route(
            "/n-admin/export-json",
            get(handle_admin_export_json_download),
        )
        .route("/n-admin/site-health", get(handle_admin_site_health))
        .route(
            "/n-admin/export-personal-data",
            get(handle_admin_export_personal_data),
        )
        .route(
            "/n-admin/erase-personal-data",
            get(handle_admin_erase_personal_data),
        )
        .route(
            "/n-admin/api/export-personal-data",
            get(handle_api_export_personal_data),
        )
        .route(
            "/n-admin/api/erase-personal-data",
            post(handle_api_erase_personal_data),
        )
        // 9. Settings
        .route("/n-admin/options-general", get(handle_admin_settings))
        .route(
            "/n-admin/options-writing",
            get(handle_admin_options_writing),
        )
        .route(
            "/n-admin/api/options-writing",
            post(handle_api_options_writing_save),
        )
        .route(
            "/n-admin/options-reading",
            get(handle_admin_options_reading),
        )
        .route(
            "/n-admin/api/options-reading",
            post(handle_api_options_reading_save),
        )
        .route(
            "/n-admin/options-discussion",
            get(handle_admin_options_discussion),
        )
        .route(
            "/n-admin/api/options-discussion",
            post(handle_api_options_discussion_save),
        )
        .route("/n-admin/options-media", get(handle_admin_options_media))
        .route(
            "/n-admin/api/options-media",
            post(handle_api_options_media_save),
        )
        .route(
            "/n-admin/options-permalink",
            get(handle_admin_options_permalink),
        )
        .route(
            "/n-admin/api/options-permalink",
            post(handle_api_options_permalink_save),
        )
        .route(
            "/n-admin/options-privacy",
            get(handle_admin_options_privacy),
        )
        .route(
            "/n-admin/api/options-privacy",
            post(handle_api_options_privacy_save),
        )
        // Neopress Extensions & Native Subsystems
        .route("/n-admin/neopress-engine", get(handle_admin_engine))
        .route("/n-admin/analytics", get(handle_admin_analytics))
        .route("/sitemap.xml", get(handle_sitemap_xml))
        .route("/robots.txt", get(handle_robots_txt))
        // RSS 2.0 feeds (static `feed` segments win over the `/{slug}` catch-alls below).
        .route("/feed", get(handle_rss_feed))
        .route("/feed/", get(handle_rss_feed))
        .route("/category/{slug}/feed", get(handle_category_rss_feed))
        .route("/category/{slug}/feed/", get(handle_category_rss_feed))
        .route("/tag/{slug}/feed", get(handle_tag_rss_feed))
        .route("/tag/{slug}/feed/", get(handle_tag_rss_feed))
        .route(
            "/wp-admin",
            get(|| async { Redirect::permanent("/n-admin/") }),
        )
        .route(
            "/wp-admin/",
            get(|| async { Redirect::permanent("/n-admin/") }),
        )
        .route(
            "/wp-admin/{*path}",
            get(|AxPath(path): AxPath<String>| async move {
                let clean = path.strip_suffix(".php").unwrap_or(&path);
                Redirect::permanent(&format!("/n-admin/{}", clean))
            }),
        )
        .route(
            "/status",
            get(|| async { Redirect::to("/n-admin/neopress-engine") }),
        )
        .route(
            "/status/",
            get(|| async { Redirect::to("/n-admin/neopress-engine") }),
        )
        .route("/admin", get(|| async { Redirect::permanent("/n-admin/") }))
        .route(
            "/admin/",
            get(|| async { Redirect::permanent("/n-admin/") }),
        )
        .route("/", get(handle_home_or_query))
        .route("/search", get(handle_search_page))
        .route("/search/", get(handle_search_page))
        .route("/page/{page}", get(handle_home_paged))
        .route("/page/{page}/", get(handle_home_paged))
        .route(
            "/category/{slug}/page/{page}",
            get(handle_category_archive_paged),
        )
        .route(
            "/category/{slug}/page/{page}/",
            get(handle_category_archive_paged),
        )
        .route("/tag/{slug}/page/{page}", get(handle_tag_archive_paged))
        .route("/tag/{slug}/page/{page}/", get(handle_tag_archive_paged))
        .route(
            "/author/{slug}/page/{page}",
            get(handle_author_archive_paged),
        )
        .route(
            "/author/{slug}/page/{page}/",
            get(handle_author_archive_paged),
        )
        .route("/{slug}/page/{page}", get(handle_single_paged))
        .route("/{slug}/page/{page}/", get(handle_single_paged))
        .route("/category/{slug}", get(handle_category_archive))
        .route("/category/{slug}/", get(handle_category_archive))
        .route("/tag/{slug}", get(handle_tag_archive))
        .route("/tag/{slug}/", get(handle_tag_archive))
        .route("/author/{slug}", get(handle_author_archive))
        .route("/author/{slug}/", get(handle_author_archive))
        .route("/{post_type}/{slug}", get(handle_cpt_single))
        .route("/{post_type}/{slug}/", get(handle_cpt_single))
        .route("/{slug}", get(handle_single))
        .route("/{slug}/", get(handle_single))
        // Every /n-admin/* screen and API (and /studio) requires a signed login session.
        .layer(axum::middleware::from_fn_with_state(
            database.clone(),
            auth::admin_session_gate,
        ))
        // Scheduled posts whose date has passed are published before any request reads posts.
        .layer(axum::middleware::from_fn_with_state(
            database.clone(),
            handlers::publish_due_posts_layer,
        ))
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(axum::extract::DefaultBodyLimit::max(500 * 1024 * 1024))
        .with_state(database)
}
