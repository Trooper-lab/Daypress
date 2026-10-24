#![recursion_limit = "256"]

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
pub mod plugins;
pub mod seed;
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
        .nest_service("/wp-includes/css", ServeDir::new(static_dir))
        .nest_service("/wp-admin/css", ServeDir::new(static_dir))
        .nest_service(
            "/wp-content/themes",
            ServeDir::new(format!("{}/themes", static_dir)),
        )
        .nest_service("/static", ServeDir::new(static_dir))
        .route("/uploads/{*path}", get(media::handle_serve_media))
        .route(
            "/wp-content/uploads/{*path}",
            get(media::handle_serve_media),
        )
        .route(
            "/wp-login.php",
            get(handle_login_page).post(handle_login_submit),
        )
        .route("/login", get(|| async { Redirect::to("/wp-login.php") }))
        .route(
            "/n-admin/login.php",
            get(handle_login_page).post(handle_login_submit),
        )
        .route("/n-admin/logout.php", get(handle_logout))
        // 1. Dashboard
        .route(
            "/n-admin",
            get(|| async { Redirect::permanent("/n-admin/") }),
        )
        .route("/n-admin/", get(handle_admin_dashboard))
        .route("/n-admin/index.php", get(handle_admin_dashboard))
        .route("/n-admin/update-core.php", get(handle_admin_updates))
        // 2. Posts & Pages (both utilize edit.php and post-new.php with post_type query parameters)
        .route("/n-admin/edit.php", get(handle_admin_posts))
        .route("/n-admin/post-new.php", get(handle_admin_post_new))
        .route("/n-admin/post.php", get(handle_admin_post_new))
        .route("/n-admin/builder.php", get(handle_admin_builder))
        .route("/n-admin/builder", get(handle_admin_builder))
        .route("/n-admin/studio", get(handle_admin_builder))
        .route("/studio", get(handle_admin_builder))
        .route("/n-admin/edit-tags.php", get(handle_admin_tags))
        // 3. Media
        .route("/n-admin/upload.php", get(handle_admin_media))
        .route("/n-admin/media-new.php", get(handle_admin_media_new))
        // 4. Comments
        .route("/n-admin/edit-comments.php", get(handle_admin_comments))
        // 5. Appearance
        .route("/n-admin/themes.php", get(handle_admin_themes))
        .route("/n-admin/customize.php", get(handle_admin_customize))
        .route("/n-admin/widgets.php", get(handle_admin_widgets))
        .route("/n-admin/nav-menus.php", get(handle_admin_nav_menus))
        .route("/n-admin/theme-editor.php", get(handle_admin_theme_editor))
        .route("/n-admin/api/theme", post(handle_api_theme_save))
        .route("/n-admin/api/customizer", post(handle_api_customizer_save))
        .route("/n-admin/api/widgets", post(handle_api_widgets_save))
        .route("/n-admin/api/menus", post(handle_api_menus_save))
        .route(
            "/n-admin/api/theme-file",
            get(handle_api_theme_file_get).post(handle_api_theme_file_save),
        )
        // 6. Plugins
        .route("/n-admin/plugins.php", get(handle_admin_plugins))
        .route(
            "/n-admin/plugin-install.php",
            get(handle_admin_plugin_install),
        )
        .route(
            "/n-admin/plugin-editor.php",
            get(handle_admin_plugin_editor),
        )
        .route(
            "/n-admin/plugin-settings.php",
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
        .route("/n-admin/seo.php", get(handle_admin_seo))
        .route("/n-admin/neopress-seo.php", get(handle_admin_seo))
        .route("/n-admin/api/seo", post(handle_api_seo_save))
        .route("/n-admin/api/redirects", post(handle_api_redirects))
        .route("/n-admin/forms.php", get(handle_admin_forms))
        .route("/n-admin/neopress-forms.php", get(handle_admin_forms))
        .route("/n-admin/api/forms", post(handle_api_forms))
        .route(
            "/n-admin/api/forms/export-csv",
            get(handle_admin_forms_export_csv),
        )
        .route("/n-admin/backups.php", get(handle_admin_backups))
        .route("/n-admin/neopress-backups.php", get(handle_admin_backups))
        .route("/n-admin/api/backups", post(handle_api_backups))
        .route(
            "/n-admin/backups/download/{filename}",
            get(handle_admin_backup_download),
        )
        // 7. Users
        .route("/n-admin/users.php", get(handle_admin_users))
        .route(
            "/n-admin/user-new.php",
            get(handle_admin_user_new).post(handle_admin_user_new_submit),
        )
        .route("/n-admin/profile.php", get(handle_admin_profile))
        .route("/n-admin/user-edit.php", get(handle_admin_profile))
        // 8. Tools
        .route("/n-admin/tools.php", get(handle_admin_tools))
        .route("/n-admin/import.php", get(handle_admin_import))
        .route("/n-admin/export.php", get(handle_admin_export))
        .route("/n-admin/export-wxr", get(handle_admin_export_wxr_download))
        .route(
            "/n-admin/export-json",
            get(handle_admin_export_json_download),
        )
        .route("/n-admin/site-health.php", get(handle_admin_site_health))
        .route(
            "/n-admin/export-personal-data.php",
            get(handle_admin_export_personal_data),
        )
        .route(
            "/n-admin/erase-personal-data.php",
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
        .route("/n-admin/options-general.php", get(handle_admin_settings))
        .route(
            "/n-admin/options-writing.php",
            get(handle_admin_options_writing),
        )
        .route(
            "/n-admin/api/options-writing",
            post(handle_api_options_writing_save),
        )
        .route(
            "/n-admin/options-reading.php",
            get(handle_admin_options_reading),
        )
        .route(
            "/n-admin/api/options-reading",
            post(handle_api_options_reading_save),
        )
        .route(
            "/n-admin/options-discussion.php",
            get(handle_admin_options_discussion),
        )
        .route(
            "/n-admin/api/options-discussion",
            post(handle_api_options_discussion_save),
        )
        .route(
            "/n-admin/options-media.php",
            get(handle_admin_options_media),
        )
        .route(
            "/n-admin/api/options-media",
            post(handle_api_options_media_save),
        )
        .route(
            "/n-admin/options-permalink.php",
            get(handle_admin_options_permalink),
        )
        .route(
            "/n-admin/api/options-permalink",
            post(handle_api_options_permalink_save),
        )
        .route(
            "/n-admin/options-privacy.php",
            get(handle_admin_options_privacy),
        )
        .route(
            "/n-admin/api/options-privacy",
            post(handle_api_options_privacy_save),
        )
        // Neopress Extensions & Native Subsystems
        .route("/n-admin/neopress-engine.php", get(handle_admin_engine))
        .route("/n-admin/analytics.php", get(handle_admin_analytics))
        .route("/sitemap.xml", get(handle_sitemap_xml))
        .route("/robots.txt", get(handle_robots_txt))
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
                Redirect::permanent(&format!("/n-admin/{}", path))
            }),
        )
        .route(
            "/status",
            get(|| async { Redirect::to("/n-admin/neopress-engine.php") }),
        )
        .route(
            "/status/",
            get(|| async { Redirect::to("/n-admin/neopress-engine.php") }),
        )
        .route("/admin", get(|| async { Redirect::permanent("/n-admin/") }))
        .route(
            "/admin/",
            get(|| async { Redirect::permanent("/n-admin/") }),
        )
        .route("/", get(handle_home_or_query))
        .route("/search", get(handle_search_page))
        .route("/search/", get(handle_search_page))
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
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .layer(axum::extract::DefaultBodyLimit::max(500 * 1024 * 1024))
        .with_state(database)
}
