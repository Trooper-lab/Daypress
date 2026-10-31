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
pub mod legacy;
pub mod media;
pub mod models;
pub mod password_reset;
pub mod plugins;
pub mod seed;
pub mod static_assets;
pub mod templates;
pub mod theme;
pub mod theme_builder;
pub mod updater;
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
pub use updater::*;
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

    let static_dir = if Path::new("crates/daypress-server/static").exists() {
        "crates/daypress-server/static"
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
        .route("/wp-json", get(api::rest_index_root))
        .route("/wp-json/", get(api::rest_index_root))
        .nest("/wp-json", api::api_routes())
        // Pre-rebrand `/n-admin/*` URLs redirect to `/d-admin/*`.
        .merge(legacy::routes())
        // Bundled assets are compiled into the binary; imported theme files stay on disk.
        .route("/static/{*path}", get(static_assets::serve_static))
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
        .route("/setup", get(handle_setup_page).post(handle_setup_submit))
        .route(
            "/d-admin/install",
            get(handle_setup_page).post(handle_setup_submit),
        )
        .route(
            "/d-admin/setup",
            get(handle_setup_page).post(handle_setup_submit),
        )
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
            "/d-admin/login",
            get(handle_login_page).post(handle_login_submit),
        )
        .route("/d-admin/logout", get(handle_logout))
        // 1. Dashboard
        .route(
            "/d-admin",
            get(|| async { Redirect::permanent("/d-admin/") }),
        )
        .route("/d-admin/", get(handle_admin_dashboard))
        .route("/d-admin/index", get(handle_admin_dashboard))
        .route("/d-admin/update-core", get(handle_admin_updates))
        // 2. Posts & Pages (both utilize edit and post-new with post_type query parameters)
        .route("/d-admin/edit", get(handle_admin_posts))
        .route("/d-admin/post-new", get(handle_admin_post_new))
        .route("/d-admin/post", get(handle_admin_post_new))
        .route("/d-admin/builder", get(handle_admin_builder))
        .route("/d-admin/studio", get(handle_admin_builder))
        .route("/studio", get(handle_admin_builder))
        .route("/d-admin/edit-tags", get(handle_admin_tags))
        // 3. Media
        .route("/d-admin/upload", get(handle_admin_media))
        .route("/d-admin/media-new", get(handle_admin_media_new))
        // 4. Comments
        .route("/d-admin/edit-comments", get(handle_admin_comments))
        // 5. Appearance
        // Theme switching, widgets and the file editors are not part of v0.1: send visitors of
        // those legacy screens to the tools that do the job.
        .route(
            "/d-admin/themes",
            get(|| async { Redirect::to("/d-admin/headers-footers") }),
        )
        .route(
            "/d-admin/headers-footers",
            get(handle_admin_headers_footers),
        )
        .route("/d-admin/theme-builder", get(handle_admin_headers_footers))
        .route("/d-admin/customize", get(handle_admin_customize))
        .route(
            "/d-admin/widgets",
            get(|| async { Redirect::to("/d-admin/headers-footers") }),
        )
        .route("/d-admin/nav-menus", get(handle_admin_nav_menus))
        .route(
            "/d-admin/theme-editor",
            get(|| async { Redirect::to("/d-admin/customize") }),
        )
        .route("/d-admin/api/theme", post(handle_api_theme_save))
        .route("/d-admin/api/customizer", post(handle_api_customizer_save))
        .route("/d-admin/api/widgets", post(handle_api_widgets_save))
        .route("/d-admin/api/menus", post(handle_api_menus_save))
        .route(
            "/d-admin/api/theme-file",
            get(handle_api_theme_file_get).post(handle_api_theme_file_save),
        )
        // 6. Plugins
        .route("/d-admin/plugins", get(handle_admin_plugins))
        .route("/d-admin/plugin-install", get(handle_admin_plugin_install))
        .route(
            "/d-admin/plugin-editor",
            get(|| async { Redirect::to("/d-admin/plugins") }),
        )
        .route(
            "/d-admin/plugin-settings",
            get(handle_admin_plugin_settings),
        )
        .route("/d-admin/api/plugins", post(handle_api_plugins_save))
        .route(
            "/d-admin/api/plugin-settings",
            post(handle_api_plugin_settings_save),
        )
        .route(
            "/d-admin/api/plugin-install",
            post(handle_api_plugin_install_custom),
        )
        .route(
            "/d-admin/api/plugin-sdk-template",
            get(handle_api_plugin_sdk_template),
        )
        .route(
            "/d-admin/api/download-migrator-plugin",
            get(handle_api_download_migrator_plugin),
        )
        // 6b. First-Party Modules (SEO & Redirects, Forms Studio, Hot Backups)
        .route("/d-admin/seo", get(handle_admin_seo))
        .route("/d-admin/daypress-seo", get(handle_admin_seo))
        .route("/d-admin/api/seo", post(handle_api_seo_save))
        .route("/d-admin/api/redirects", post(handle_api_redirects))
        .route("/d-admin/forms", get(handle_admin_forms))
        .route("/d-admin/daypress-forms", get(handle_admin_forms))
        .route("/d-admin/api/forms", post(handle_api_forms))
        .route(
            "/d-admin/api/forms/export-csv",
            get(handle_admin_forms_export_csv),
        )
        .route("/d-admin/backups", get(handle_admin_backups))
        .route("/d-admin/daypress-backups", get(handle_admin_backups))
        .route("/d-admin/api/backups", post(handle_api_backups))
        .route(
            "/d-admin/backups/download/{filename}",
            get(handle_admin_backup_download),
        )
        // 7. Users
        .route("/d-admin/users", get(handle_admin_users))
        .route(
            "/d-admin/user-new",
            get(handle_admin_user_new).post(handle_admin_user_new_submit),
        )
        .route("/d-admin/profile", get(handle_admin_profile))
        .route("/d-admin/user-edit", get(handle_admin_profile))
        // 8. Tools
        .route("/d-admin/tools", get(handle_admin_tools))
        .route("/d-admin/import", get(handle_admin_import))
        .route("/d-admin/export", get(handle_admin_export))
        .route("/d-admin/export-wxr", get(handle_admin_export_wxr_download))
        .route(
            "/d-admin/export-json",
            get(handle_admin_export_json_download),
        )
        .route("/d-admin/site-health", get(handle_admin_site_health))
        .route(
            "/d-admin/export-personal-data",
            get(handle_admin_export_personal_data),
        )
        .route(
            "/d-admin/erase-personal-data",
            get(handle_admin_erase_personal_data),
        )
        .route(
            "/d-admin/api/export-personal-data",
            get(handle_api_export_personal_data),
        )
        .route(
            "/d-admin/api/erase-personal-data",
            post(handle_api_erase_personal_data),
        )
        // 9. Settings
        .route("/d-admin/options-general", get(handle_admin_settings))
        .route(
            "/d-admin/options-writing",
            get(handle_admin_options_writing),
        )
        .route(
            "/d-admin/api/options-writing",
            post(handle_api_options_writing_save),
        )
        .route(
            "/d-admin/options-reading",
            get(handle_admin_options_reading),
        )
        .route(
            "/d-admin/api/options-reading",
            post(handle_api_options_reading_save),
        )
        .route(
            "/d-admin/options-discussion",
            get(handle_admin_options_discussion),
        )
        .route(
            "/d-admin/api/options-discussion",
            post(handle_api_options_discussion_save),
        )
        .route("/d-admin/options-media", get(handle_admin_options_media))
        .route(
            "/d-admin/api/options-media",
            post(handle_api_options_media_save),
        )
        .route(
            "/d-admin/options-permalink",
            get(handle_admin_options_permalink),
        )
        .route(
            "/d-admin/api/options-permalink",
            post(handle_api_options_permalink_save),
        )
        .route(
            "/d-admin/options-privacy",
            get(handle_admin_options_privacy),
        )
        .route(
            "/d-admin/api/options-privacy",
            post(handle_api_options_privacy_save),
        )
        // Daypress Extensions & Native Subsystems
        .route("/d-admin/daypress-engine", get(handle_admin_engine))
        .route("/d-admin/analytics", get(handle_admin_analytics))
        .route("/sitemap.xml", get(handle_sitemap_xml))
        .route("/robots.txt", get(handle_robots_txt))
        // RSS 2.0 feeds (static `feed` segments win over the `/{slug}` catch-alls below).
        .route("/feed", get(handle_rss_feed))
        .route("/feed/rss", get(handle_rss_feed))
        .route("/feed/rss/", get(handle_rss_feed))
        .route("/feed/rss2", get(handle_rss_feed))
        .route("/feed/rss2/", get(handle_rss_feed))
        .route("/feed/atom", get(handle_atom_feed))
        .route("/feed/atom/", get(handle_atom_feed))
        .route("/comments/feed", get(handle_comments_rss_feed))
        .route("/comments/feed/", get(handle_comments_rss_feed))
        .route("/comments/feed/atom", get(handle_comments_atom_feed))
        .route("/comments/feed/atom/", get(handle_comments_atom_feed))
        .route("/category/{slug}/feed/atom", get(handle_category_atom_feed))
        .route(
            "/category/{slug}/feed/atom/",
            get(handle_category_atom_feed),
        )
        .route("/tag/{slug}/feed/atom", get(handle_tag_atom_feed))
        .route("/tag/{slug}/feed/atom/", get(handle_tag_atom_feed))
        .route("/feed/", get(handle_rss_feed))
        .route("/category/{slug}/feed", get(handle_category_rss_feed))
        .route("/category/{slug}/feed/", get(handle_category_rss_feed))
        .route("/tag/{slug}/feed", get(handle_tag_rss_feed))
        .route("/tag/{slug}/feed/", get(handle_tag_rss_feed))
        .route(
            "/wp-admin",
            get(|| async { Redirect::permanent("/d-admin/") }),
        )
        .route(
            "/wp-admin/",
            get(|| async { Redirect::permanent("/d-admin/") }),
        )
        .route(
            "/wp-admin/{*path}",
            get(|AxPath(path): AxPath<String>| async move {
                let clean = path.strip_suffix(".php").unwrap_or(&path);
                Redirect::permanent(&format!("/d-admin/{}", clean))
            }),
        )
        .route(
            "/status",
            get(|| async { Redirect::to("/d-admin/daypress-engine") }),
        )
        .route(
            "/status/",
            get(|| async { Redirect::to("/d-admin/daypress-engine") }),
        )
        .route("/admin", get(|| async { Redirect::permanent("/d-admin/") }))
        .route(
            "/admin/",
            get(|| async { Redirect::permanent("/d-admin/") }),
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
        // Every /d-admin/* screen and API (and /studio) requires a signed login session.
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
