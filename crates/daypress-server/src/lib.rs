#![recursion_limit = "256"]
// SQLite row tuples and importer passes carry many fields by design.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

// Hexagonal Architecture Core Layers
pub mod adapters;
pub mod application;
pub mod domain;
pub mod ports;

// Canonical Module Exports
pub mod acme_tls;
pub mod api;
pub mod auth;
pub mod convergence;
pub mod db;
pub mod email;
pub mod errors;
pub mod handlers;
pub mod hot_cache;
pub mod i18n;
pub mod importer;
pub mod legacy;
pub mod media;
pub mod models;
pub mod password_reset;
pub mod plugins;
pub mod publisher;
pub mod seed;
pub mod static_assets;
pub mod storage_s3;
pub mod templates;
pub mod theme;
pub mod theme_builder;
pub mod totp;
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
pub use publisher::*;
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
        .merge(acme_tls::acme_well_known_routes())
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
        .route(
            "/login/2fa",
            get(handle_login_2fa_page).post(handle_login_2fa_submit),
        )
        .route("/setup", get(handle_setup_page).post(handle_setup_submit))
        .route(
            "/setup/network-permission",
            post(handle_setup_network_permission),
        )
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
        .route(
            "/d-admin/login/2fa",
            get(handle_login_2fa_page).post(handle_login_2fa_submit),
        )
        .route("/d-admin/logout", get(handle_logout))
        // 1. Dashboard
        .route(
            "/d-admin",
            get(|| async { Redirect::permanent("/d-admin/") }),
        )
        .route("/d-admin/", get(handle_admin_dashboard))
        .route("/d-admin/index", get(handle_admin_dashboard))
        .route(
            "/d-admin/getting-started",
            get(handle_admin_getting_started),
        )
        .route(
            "/d-admin/api/getting-started/dismiss",
            post(handle_api_getting_started_dismiss),
        )
        .route(
            "/d-admin/api/getting-started/agent-key",
            post(handle_api_getting_started_agent_key),
        )
        .route("/d-admin/update-core", get(handle_admin_updates))
        .route("/d-admin/updates", get(handle_admin_updates))
        // 2. Posts & Pages (both utilize edit and post-new with post_type query parameters)
        .route("/d-admin/edit", get(handle_admin_posts))
        .route("/d-admin/post-new", get(handle_admin_post_new))
        .route("/d-admin/post", get(handle_admin_post_new))
        .route("/d-admin/builder", get(handle_admin_builder))
        .route("/d-admin/studio", get(handle_admin_builder))
        .route("/studio", get(handle_admin_builder))
        .route("/d-admin/edit-tags", get(handle_admin_tags))
        .route("/d-admin/content-types", get(handle_admin_content_types))
        .route("/d-admin/content-model", get(handle_admin_content_types))
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
        .route("/d-admin/theme-layouts", get(handle_admin_theme_layouts))
        .route("/d-admin/theme-builder", get(handle_admin_theme_layouts))
        .route("/d-admin/symbols", get(handle_admin_symbols))
        .route("/d-admin/popups", get(handle_admin_popups))
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
        .route(
            "/d-admin/api/forms/export-json",
            get(handle_admin_forms_export_json),
        )
        .route(
            "/d-admin/forms/attachment/{entry_id}/{filename}",
            get(handle_admin_form_attachment_download),
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
        .route("/d-admin/languages", get(handle_admin_languages))
        .route("/d-admin/i18n", get(handle_admin_languages))
        .route("/d-admin/publish", get(handle_admin_publish))
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
        .route("/og-image/{target}", get(handle_public_og_image))
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
        .route("/preview/{token}", get(handle_secret_preview))
        .route("/preview/{token}/", get(handle_secret_preview))
        .route(
            "/{lang}/{post_type}/{slug}",
            get(handle_localized_cpt_single),
        )
        .route(
            "/{lang}/{post_type}/{slug}/",
            get(handle_localized_cpt_single),
        )
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

#[cfg(test)]
mod v04_tests {
    use super::*;
    use crate::adapters::mcp::{handle_mcp_rpc, JsonRpcRequest};
    use axum::{
        body::Body,
        http::{header, Request, StatusCode},
    };
    use serde_json::{json, Value};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_v04_hot_cache_hit_miss_etag_and_invalidation() {
        let db = Database::new_in_memory().expect("in-memory db");
        let app = create_app(db.clone());

        // 1. First anonymous GET / -> MISS
        let res1 = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res1.status(), StatusCode::OK);
        assert_eq!(
            res1.headers()
                .get("x-daypress-cache")
                .and_then(|v| v.to_str().ok()),
            Some("MISS")
        );
        let etag = res1
            .headers()
            .get(header::ETAG)
            .and_then(|v| v.to_str().ok())
            .expect("ETag header should be present")
            .to_string();

        // 2. Second anonymous GET / with Accept-Encoding: gzip -> HIT + gzip
        let res2 = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(header::ACCEPT_ENCODING, "gzip, deflate")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res2.status(), StatusCode::OK);
        assert_eq!(
            res2.headers()
                .get("x-daypress-cache")
                .and_then(|v| v.to_str().ok()),
            Some("HIT")
        );
        assert_eq!(
            res2.headers()
                .get(header::CONTENT_ENCODING)
                .and_then(|v| v.to_str().ok()),
            Some("gzip")
        );

        // 3. Conditional GET / with If-None-Match -> 304 Not Modified
        let res3 = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(header::IF_NONE_MATCH, &etag)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res3.status(), StatusCode::NOT_MODIFIED);

        // 4. Updating an option invalidates hot cache -> next GET / is MISS
        db.set_option("blogdescription", "Updated Tagline").unwrap();
        let res4 = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(
            res4.headers()
                .get("x-daypress-cache")
                .and_then(|v| v.to_str().ok()),
            Some("MISS")
        );
    }

    #[tokio::test]
    async fn test_v04_formstudio_vault_antispam_rate_limit_and_analytics() {
        let db = Database::new_in_memory().expect("in-memory db");
        let form_id = db
            .save_form_record(&CreateFormRequest {
                id: None,
                title: "Quote Calculator Form".to_string(),
                fields: vec![
                    FormFieldSchema {
                        name: "email".to_string(),
                        label: "Work Email".to_string(),
                        field_type: "email".to_string(),
                        required: true,
                        width: "half".to_string(),
                        step: 1,
                        ..Default::default()
                    },
                    FormFieldSchema {
                        name: "seats".to_string(),
                        label: "Seats".to_string(),
                        field_type: "number".to_string(),
                        step: 1,
                        ..Default::default()
                    },
                    FormFieldSchema {
                        name: "estimate".to_string(),
                        label: "Estimated Price".to_string(),
                        field_type: "calculated".to_string(),
                        calculation_formula: Some("{seats} * 29".to_string()),
                        step: 2,
                        ..Default::default()
                    },
                ],
                settings: FormSettings {
                    step_titles: vec!["Contact".to_string(), "Estimate".to_string()],
                    min_submit_seconds: 3,
                    rate_limit_per_minute: 3,
                    ..Default::default()
                },
            })
            .expect("save form");

        // Verify shortcode SSR renders multi-step wizard and calculated formula
        let settings = db.get_settings().unwrap_or_default();
        let rendered_html = crate::plugins::expand_shortcodes_in_content(
            &format!("<p>[daypress_form id=\"{form_id}\"]</p>"),
            &db,
            &settings,
        );
        assert!(rendered_html.contains("Step 1 of 2: Contact"));
        assert!(rendered_html.contains("data-calc-formula=\"{seats} * 29\""));

        let app = create_app(db.clone());

        // 1. Valid submission -> saved as 'unread' + conversion recorded in analytics
        let valid_body = json!({
            "data": {
                "email": "founder@acme.dev",
                "seats": "10",
                "estimate": "290",
                "_dp_ts": 1000
            }
        });
        let res_ok = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/wp-json/daypress/v1/forms/{form_id}/submit"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-forwarded-for", "203.0.113.10")
                    .body(Body::from(valid_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_ok.status(), StatusCode::OK);

        // 2. Honeypot hit -> quarantined as 'spam' in SQLite Vault
        let spam_body = json!({
            "data": {
                "email": "bot@spam.example",
                "_np_hp": "http://spam-link.example"
            }
        });
        let res_spam = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/wp-json/daypress/v1/forms/{form_id}/submit"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-forwarded-for", "203.0.113.10")
                    .body(Body::from(spam_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_spam.status(), StatusCode::BAD_REQUEST);

        // 3. Timestamp trap hit (submitted 0ms after render when min_submit_seconds = 3) -> quarantined as 'spam'
        let now_ms = chrono::Utc::now().timestamp_millis();
        let fast_body = json!({
            "data": {
                "email": "fastbot@spam.example",
                "_dp_ts": now_ms
            }
        });
        let res_fast = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/wp-json/daypress/v1/forms/{form_id}/submit"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-forwarded-for", "203.0.113.10")
                    .body(Body::from(fast_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_fast.status(), StatusCode::BAD_REQUEST);

        // 4. 4th submission from same IP within 60s exceeds rate_limit_per_minute=3 -> 429 Too Many Requests
        let res_rate = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri(format!("/wp-json/daypress/v1/forms/{form_id}/submit"))
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-forwarded-for", "203.0.113.10")
                    .body(Body::from(valid_body.to_string()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_rate.status(), StatusCode::TOO_MANY_REQUESTS);

        // Verify Vault entries: 1 unread, 2 spam
        let unread = db
            .list_form_entries_filtered(Some(form_id), Some("unread"), None, 50)
            .unwrap();
        let spam = db
            .list_form_entries_filtered(Some(form_id), Some("spam"), None, 50)
            .unwrap();
        assert_eq!(unread.len(), 1);
        assert_eq!(spam.len(), 2);

        // Verify public /uploads/form-submissions/* is blocked with 403 Forbidden
        let res_upload_block = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/uploads/form-submissions/1/secret.pdf")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_upload_block.status(), StatusCode::FORBIDDEN);

        // Verify Cookieless Analytics dashboard recorded the form conversion + pageview with UTM
        let _ = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/wp-json/daypress/v1/analytics/event")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("cf-ipcountry", "NL")
                    .body(Body::from(
                        json!({
                            "page_url": "/pricing",
                            "referrer": "https://news.ycombinator.com/",
                            "device_type": "desktop",
                            "utm_source": "hn",
                            "utm_medium": "social",
                            "utm_campaign": "v04_launch"
                        })
                        .to_string(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        let analytics = db.get_analytics_dashboard("30d").unwrap();
        assert!(analytics.total_pageviews >= 1);
        assert!(analytics.total_conversions >= 1);
        assert!(analytics.utm_sources.iter().any(|u| u.label == "hn"));
        assert!(analytics.countries.iter().any(|c| c.label == "NL"));
    }

    #[tokio::test]
    async fn test_v04_privacy_cookie_banner_and_mcp_tools() {
        let db = Database::new_in_memory().expect("in-memory db");
        let mut priv_settings = db.get_privacy_settings().unwrap();
        priv_settings.cookie_banner_enabled = true;
        priv_settings.cookie_banner_position = "floating-right".to_string();
        priv_settings.cookie_banner_title = "GDPR Consent Test".to_string();
        db.update_privacy_settings(&priv_settings).unwrap();

        let app = create_app(db.clone());
        let res = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let html = String::from_utf8_lossy(&body_bytes);
        assert!(html.contains("id=\"dp-cookie-banner\""));
        assert!(html.contains("dp-cc-floating-right"));
        assert!(html.contains("GDPR Consent Test"));
        assert!(html.contains("Cookie Preferences"));

        // Verify MCP tools for FormStudio & Analytics
        let service = CmsService::from_adapter(db);
        let create_resp = handle_mcp_rpc(
            &service,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(1)),
                method: "tools/call".into(),
                params: json!({
                    "name": "daypress_create_form",
                    "arguments": {
                        "title": "MCP Created Lead Form",
                        "fields": [
                            { "name": "email", "label": "Email", "field_type": "email", "required": true }
                        ]
                    }
                }),
            },
        );
        let res_val = create_resp.result.unwrap();
        assert_eq!(res_val["isError"], false);
        let created_payload: Value =
            serde_json::from_str(res_val["content"][0]["text"].as_str().unwrap()).unwrap();
        let form_id = created_payload["id"].as_i64().unwrap();
        assert!(form_id > 0);

        let analytics_resp = handle_mcp_rpc(
            &service,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(2)),
                method: "tools/call".into(),
                params: json!({
                    "name": "daypress_get_analytics",
                    "arguments": { "range": "30d" }
                }),
            },
        );
        assert_eq!(analytics_resp.result.unwrap()["isError"], false);
    }

    #[tokio::test]
    async fn test_v05_dynamic_templates_symbols_popups_and_media_dam() {
        use crate::theme_builder::models::{
            ConditionType, CreateConditionDto, CreateTemplateRequest, RuleTarget, TemplateSettings,
            TemplateType,
        };
        use crate::theme_builder::repository::ThemeTemplateRepository;
        use daypress_ast::{
            DaypressLayout, DaypressNode, NodeKind, StyleProperties, SymbolOverride,
        };
        use std::collections::HashMap;

        let db = Database::new_in_memory().expect("in-memory db");
        let repo = ThemeTemplateRepository::from_database(&db);

        // 1. Create a SectionSymbol master template
        let symbol_layout = DaypressLayout {
            version: 1,
            global_styles: None,
            root: DaypressNode {
                id: "root".into(),
                kind: NodeKind::Root,
                styles: StyleProperties::default(),
                children: vec![DaypressNode {
                    id: "sym_cta_heading".into(),
                    kind: NodeKind::Heading {
                        text: "Master Symbol Headline".into(),
                        level: 2,
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                }],
            },
        };
        let sym_record = repo
            .create(&CreateTemplateRequest {
                title: "Global Promo Symbol".into(),
                template_type: TemplateType::SectionSymbol,
                layout: Some(symbol_layout),
                layout_json: None,
                status: Some("publish".into()),
                priority: Some(10),
                is_active: Some(true),
                settings: None,
                conditions: None,
            })
            .expect("create symbol");

        // 2. Create a SinglePost template that embeds {{post.title}} AND an overridden SymbolInstance
        let mut overrides = HashMap::new();
        overrides.insert(
            "sym_cta_heading".to_string(),
            SymbolOverride {
                text: Some("Instance Overridden CTA".to_string()),
                ..Default::default()
            },
        );
        let single_layout = DaypressLayout {
            version: 1,
            global_styles: None,
            root: DaypressNode {
                id: "root".into(),
                kind: NodeKind::Root,
                styles: StyleProperties::default(),
                children: vec![
                    DaypressNode {
                        id: "dyn_title".into(),
                        kind: NodeKind::Heading {
                            text: "Article: {{post.title}} by {{author.name}}".into(),
                            level: 1,
                        },
                        styles: StyleProperties::default(),
                        children: vec![],
                    },
                    DaypressNode {
                        id: "sym_inst".into(),
                        kind: NodeKind::SymbolInstance {
                            symbol_id: sym_record.id,
                            overrides,
                        },
                        styles: StyleProperties::default(),
                        children: vec![],
                    },
                ],
            },
        };
        repo.create(&CreateTemplateRequest {
            title: "Custom Single Post Layout".into(),
            template_type: TemplateType::SinglePost,
            layout: Some(single_layout),
            layout_json: None,
            status: Some("publish".into()),
            priority: Some(20),
            is_active: Some(true),
            settings: None,
            conditions: Some(vec![CreateConditionDto {
                id: None,
                condition_type: ConditionType::Include,
                rule_target: RuleTarget::AllPosts,
                target_id: String::new(),
                priority: 20,
            }]),
        })
        .expect("create single_post template");

        // 3. Create a Top Announcement Bar Popup template
        let popup_settings = TemplateSettings {
            popup_variant: Some("announcement_bar_top".to_string()),
            dismiss_days: Some(7),
            ..Default::default()
        };
        let popup_layout = DaypressLayout {
            version: 1,
            global_styles: None,
            root: DaypressNode {
                id: "root".into(),
                kind: NodeKind::Root,
                styles: StyleProperties::default(),
                children: vec![DaypressNode {
                    id: "ann_txt".into(),
                    kind: NodeKind::Text {
                        content_html: "Daypress 0.5 Shipped Announcement!".into(),
                    },
                    styles: StyleProperties::default(),
                    children: vec![],
                }],
            },
        };
        repo.create(&CreateTemplateRequest {
            title: "Sitewide Announcement".into(),
            template_type: TemplateType::Popup,
            layout: Some(popup_layout),
            layout_json: None,
            status: Some("publish".into()),
            priority: Some(10),
            is_active: Some(true),
            settings: Some(popup_settings),
            conditions: None,
        })
        .expect("create popup");

        // 4. Request a single post (/?p=1 is the seeded "Hello world!" post)
        let app = create_app(db.clone());
        let res = app
            .oneshot(Request::builder().uri("/?p=1").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(res.status(), StatusCode::OK);
        let body_bytes = axum::body::to_bytes(res.into_body(), usize::MAX)
            .await
            .unwrap();
        let html = String::from_utf8_lossy(&body_bytes);
        assert!(
            html.contains("Article: Hello world!"),
            "Expected dynamic {{post.title}} binding in SSR output"
        );
        assert!(
            html.contains("Instance Overridden CTA"),
            "Expected SymbolInstance child override in SSR output"
        );
        assert!(
            html.contains("Daypress 0.5 Shipped Announcement!"),
            "Expected Announcement Bar popup in SSR output"
        );

        // 5. Verify Media Library DAM Virtual Folders & Unused Asset Scanner
        let folder = db
            .create_media_folder("Brand Icons", 0)
            .expect("create media folder");
        assert_eq!(folder.name, "Brand Icons");
        let folders = db.list_media_folders().expect("list folders");
        assert_eq!(folders.len(), 1);

        let att = db
            .create_attachment(
                "Unused Icon",
                "unused-test-icon.svg",
                "image/svg+xml",
                128,
                "http://localhost:8090/uploads/unused-test-icon.svg",
            )
            .expect("create attachment");
        let unused = db.scan_unused_media_attachments().expect("scan unused");
        assert!(
            unused.iter().any(|m| m.id == att.id),
            "Unreferenced attachment must be detected by Unused Asset Scanner"
        );
    }
}

#[cfg(test)]
mod v08_tests {
    use super::*;
    use crate::adapters::mcp::{handle_mcp_rpc, JsonRpcRequest};
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use daypress_ast::{DaypressLayout, DaypressNode, NodeKind, StyleProperties};
    use serde_json::{json, Value};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_v08_i18n_multilingual_routing_hreflang_sitemap_and_switcher() {
        let db = Database::new_in_memory().expect("in-memory db");

        // 1. Register German (de) and Dutch (nl) languages alongside default English (en)
        db.upsert_language(&DaypressLanguage {
            code: "de".into(),
            locale: "de_DE".into(),
            name: "Deutsch".into(),
            native_name: "Deutsch".into(),
            flag_emoji: "🇩🇪".into(),
            is_default: false,
            is_rtl: false,
            is_active: true,
            sort_order: 1,
        })
        .expect("upsert de");

        let langs = db.list_languages().expect("list langs");
        assert!(langs.iter().any(|l| l.code == "en" && l.is_default));
        assert!(langs.iter().any(|l| l.code == "de"));

        // 2. Create an English post with a LanguageSwitcher AST block and create its German translation draft
        let layout = DaypressLayout {
            version: 1,
            global_styles: None,
            root: DaypressNode {
                id: "root".into(),
                kind: NodeKind::Root,
                styles: StyleProperties::default(),
                children: vec![
                    DaypressNode {
                        id: "hero_h1".into(),
                        kind: NodeKind::Heading {
                            text: "Welcome to Daypress 0.8".into(),
                            level: 1,
                        },
                        styles: StyleProperties::default(),
                        children: vec![],
                    },
                    DaypressNode {
                        id: "lang_sw".into(),
                        kind: NodeKind::LanguageSwitcher {
                            display_style: "pills".into(),
                            show_flags: true,
                            show_names: true,
                            show_codes: false,
                            current_lang: "en".into(),
                            links: vec![],
                        },
                        styles: StyleProperties::default(),
                        children: vec![],
                    },
                ],
            },
        };
        let en_post = db
            .create_post(&CreatePostRequest {
                title: "Welcome Guide".into(),
                slug: Some("welcome-guide".into()),
                content: "<p>English guide content [daypress_language_switcher]</p>".into(),
                status: "publish".into(),
                post_type: "page".into(),
                lang_code: Some("en".into()),
                ..Default::default()
            })
            .expect("create en page");
        db.save_post_layout(
            en_post.id,
            &serde_json::to_string(&layout).unwrap(),
            Some("<p>English guide content [daypress_language_switcher]</p>"),
        )
        .expect("save en layout");

        let de_draft = db
            .create_translation_draft(en_post.id, "de")
            .expect("create de translation draft");
        assert_eq!(de_draft.lang_code, "de");
        assert_eq!(
            de_draft.translation_group_id,
            format!("trgrp_post_{}", en_post.id)
        );

        // Publish the German translation at slug `willkommen`
        let de_pub = db
            .update_post(
                de_draft.id,
                &UpdatePostRequest {
                    title: Some("Willkommen Leitfaden".into()),
                    slug: Some("willkommen".into()),
                    content: Some("<p>Deutscher Inhalt [daypress_language_switcher]</p>".into()),
                    status: Some("publish".into()),
                    lang_code: Some("de".into()),
                    ..Default::default()
                },
            )
            .expect("update de post")
            .expect("de post exists");
        assert_eq!(de_pub.link, "http://localhost:8090/de/willkommen/");

        // Verify same-slug translations across different languages do not collide (`contact` in en + `contact` in de)
        let en_contact = db
            .create_post(&CreatePostRequest {
                title: "Contact Us".into(),
                slug: Some("contact".into()),
                content: "<p>English contact</p>".into(),
                status: "publish".into(),
                post_type: "page".into(),
                lang_code: Some("en".into()),
                ..Default::default()
            })
            .expect("create en contact");
        assert_eq!(en_contact.slug, "contact");
        let de_contact = db
            .create_post(&CreatePostRequest {
                title: "Kontakt".into(),
                slug: Some("contact".into()),
                content: "<p>Deutscher Kontakt</p>".into(),
                status: "publish".into(),
                post_type: "page".into(),
                lang_code: Some("de".into()),
                translation_group_id: Some(en_contact.translation_group_id.clone()),
                ..Default::default()
            })
            .expect("create de contact with same slug");
        assert_eq!(
            de_contact.slug, "contact",
            "Same-slug translation in another language must not be suffixed with -2"
        );
        assert_eq!(de_contact.link, "http://localhost:8090/de/contact/");

        // 3. Request English page at /welcome-guide/ and verify hreflang tags + LanguageSwitcher SSR
        let app = create_app(db.clone());
        let res_en = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/welcome-guide/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_en.status(), StatusCode::OK);
        let html_en = String::from_utf8_lossy(
            &axum::body::to_bytes(res_en.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .to_string();
        assert!(
            html_en.contains(r#"hreflang="en" href="http://localhost:8090/welcome-guide/""#),
            "SSR head must include English hreflang"
        );
        assert!(
            html_en.contains(r#"hreflang="de" href="http://localhost:8090/de/willkommen/""#),
            "SSR head must include German hreflang"
        );
        assert!(
            html_en.contains(r#"hreflang="x-default""#),
            "SSR head must include x-default hreflang"
        );
        assert!(
            html_en.contains("dp-lang-switcher"),
            "LanguageSwitcher shortcode/widget must render in SSR"
        );

        // 4. Request German page at /de/willkommen/ and verify localized SSR, <html lang="de">, og:locale, and inLanguage JSON-LD
        let res_de = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/de/willkommen/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_de.status(), StatusCode::OK);
        let html_de = String::from_utf8_lossy(
            &axum::body::to_bytes(res_de.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .to_string();
        assert!(html_de.contains("Willkommen Leitfaden"));
        assert!(
            html_de.contains(r#"<html lang="de""#),
            "German SSR must emit <html lang=\"de\">"
        );
        assert!(
            html_de.contains(r#"property="og:locale" content="de_DE""#),
            "German SSR must emit og:locale content=de_DE"
        );
        assert!(
            html_de.contains(r#"property="og:locale:alternate" content="en_US""#),
            "German SSR must emit og:locale:alternate content=en_US"
        );
        assert!(
            html_de.contains(r#""inLanguage": "de""#) || html_de.contains(r#""inLanguage":"de""#),
            "Schema.org JSON-LD must include inLanguage: de"
        );

        // Verify /de/page/1/ paginated localized blog index returns 200 OK
        let res_de_page1 = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/de/page/1/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_de_page1.status(), StatusCode::OK);

        // Verify FTS5 search with ?lang=de filters by language
        let res_search_de = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/wp-json/daypress/v1/search?q=Willkommen&lang=de")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_search_de.status(), StatusCode::OK);
        let search_json: Value = serde_json::from_slice(
            &axum::body::to_bytes(res_search_de.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .unwrap();
        let results = search_json["results"]
            .as_array()
            .expect("search results array");
        assert!(
            results.iter().any(|r| r["lang_code"] == "de"),
            "FTS5 search with ?lang=de must return German post with lang_code='de'"
        );

        // 5. Request /sitemap.xml and verify xhtml:link hreflang alternates
        let res_sm = app
            .oneshot(
                Request::builder()
                    .uri("/sitemap.xml")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_sm.status(), StatusCode::OK);
        let xml = String::from_utf8_lossy(
            &axum::body::to_bytes(res_sm.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .to_string();
        assert!(xml.contains(r#"xmlns:xhtml="http://www.w3.org/1999/xhtml""#));
        assert!(xml.contains(r#"<xhtml:link rel="alternate" hreflang="de" href="http://localhost:8090/de/willkommen/"/>"#));
    }

    #[tokio::test]
    async fn test_v08_editorial_duplication_coauthors_preview_links_and_scheduled_revisions() {
        let db = Database::new_in_memory().expect("in-memory db");

        // 1. Create a second author user ("Alice") for Multi-Author Bylines
        let alice = db
            .create_user(&CreateUserRequest {
                username: "alice".into(),
                name: Some("Alice CoAuthor".into()),
                email: "alice@daypress.dev".into(),
                password: "SecretPass123!".into(),
                role: Some("author".into()),
                ..Default::default()
            })
            .expect("create alice");

        let mut meta_map = serde_json::Map::new();
        meta_map.insert("hero_subtitle".into(), json!("Sub-ms Rust CMS"));

        let post = db
            .create_post(&CreatePostRequest {
                title: "Engineering Deep Dive".into(),
                slug: Some("engineering-deep-dive".into()),
                content: "<p>Original live article body</p>".into(),
                status: "publish".into(),
                post_type: "post".into(),
                coauthors: Some(vec![1, alice.id]),
                lang_code: Some("en".into()),
                meta: Some(Value::Object(meta_map)),
                ..Default::default()
            })
            .expect("create post with coauthors");

        // Verify Multi-Author Byline ("admin & Alice CoAuthor")
        assert_eq!(post.coauthors.len(), 2);
        assert!(
            post.author_name.contains("Alice CoAuthor") && post.author_name.contains("&"),
            "Expected formatted multi-author byline, got: {}",
            post.author_name
        );

        // Verify secondary co-author appears on /author/alice/ archive AND in Schema.org JSON-LD
        let (_, alice_posts) = db
            .get_posts_by_author_slug("alice")
            .expect("alice author archive");
        assert!(
            alice_posts.iter().any(|p| p.id == post.id),
            "Secondary co-author archive must include co-authored post"
        );

        let app = create_app(db.clone());
        let res_live = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/engineering-deep-dive/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        let live_html = String::from_utf8_lossy(
            &axum::body::to_bytes(res_live.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .to_string();
        assert!(
            live_html.contains("Alice CoAuthor") && live_html.contains("/author/alice/"),
            "Schema.org JSON-LD must include co-author Person array with author archive URL"
        );

        // 2. 1-Click Post Duplication preserves meta, co-authors, and language as a (Copy) draft
        let duplicated = db
            .duplicate_post(post.id)
            .expect("duplicate_post")
            .expect("duplicated post");
        assert_eq!(duplicated.status, "draft");
        assert!(duplicated.title.rendered.contains("(Copy)"));
        assert_eq!(duplicated.coauthors.len(), 2);
        assert_eq!(
            duplicated.acf.get("hero_subtitle").and_then(|v| v.as_str()),
            Some("Sub-ms Rust CMS")
        );

        // 3. Shareable Secret Draft Preview Link (no login required + X-Robots-Tag: noindex, nofollow)
        let token_rec = db
            .create_preview_token(duplicated.id, 48, Some("External Reviewer"), 1)
            .expect("create preview token");
        let raw_token = token_rec.raw_token.expect("raw_token returned on creation");

        let res_prev = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/preview/{}", raw_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_prev.status(), StatusCode::OK);
        assert_eq!(
            res_prev
                .headers()
                .get("x-robots-tag")
                .and_then(|v| v.to_str().ok()),
            Some("noindex, nofollow")
        );
        let prev_html = String::from_utf8_lossy(
            &axum::body::to_bytes(res_prev.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .to_string();
        assert!(prev_html.contains("Daypress Secret Draft Preview"));
        assert!(prev_html.contains("Engineering Deep Dive (Copy)"));

        // Revoke token -> subsequent request returns 403 Forbidden
        assert!(db
            .revoke_preview_token(duplicated.id, token_rec.id)
            .expect("revoke token"));
        let res_revoked = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(format!("/preview/{}", raw_token))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_revoked.status(), StatusCode::FORBIDDEN);

        // 4. Scheduled Revision to an Already-Published Post
        let sched_summary = db
            .schedule_published_post_revision(
                post.id,
                &ScheduleRevisionRequest {
                    title: Some("Engineering Deep Dive v2.0 (Updated)".into()),
                    content: Some("<p>Scheduled v2.0 content now live!</p>".into()),
                    excerpt: Some("Updated excerpt".into()),
                    layout: None,
                    date: Some("2099-01-01 12:00:00".into()),
                    date_gmt: Some("2099-01-01 12:00:00".into()),
                },
                Some(1),
            )
            .expect("schedule revision");
        assert_eq!(sched_summary.parent_post_id, post.id);

        // Live post remains untouched before due time
        let still_v1 = db.get_post_by_id(post.id).unwrap().unwrap();
        assert_eq!(still_v1.title.rendered, "Engineering Deep Dive");
        assert_eq!(
            still_v1.scheduled_revision_id,
            Some(sched_summary.revision_post_id)
        );

        // Move scheduled revision due time into the past and trigger lazy promotion
        let due_summary = db
            .schedule_published_post_revision(
                post.id,
                &ScheduleRevisionRequest {
                    title: Some("Engineering Deep Dive v2.0 (Updated)".into()),
                    content: Some("<p>Scheduled v2.0 content now live!</p>".into()),
                    excerpt: Some("Updated excerpt".into()),
                    layout: None,
                    date: Some("2020-01-01 00:00:00".into()),
                    date_gmt: Some("2020-01-01 00:00:00".into()),
                },
                Some(1),
            )
            .expect("reschedule in past");
        assert_eq!(due_summary.revision_post_id, sched_summary.revision_post_id);

        let promoted_count = db.publish_due_scheduled_posts().expect("promote due");
        assert!(promoted_count >= 1);

        let now_v2 = db.get_post_by_id(post.id).unwrap().unwrap();
        assert_eq!(
            now_v2.title.rendered,
            "Engineering Deep Dive v2.0 (Updated)"
        );
        assert!(now_v2
            .content
            .rendered
            .contains("Scheduled v2.0 content now live!"));
        assert_eq!(now_v2.scheduled_revision_id, None);
    }

    #[tokio::test]
    async fn test_v08_ast_revision_diff_cherry_pick_and_mcp_tools() {
        let db = Database::new_in_memory().expect("in-memory db");

        // 1. Create a post with initial AST layout v1, then update to AST layout v2
        let layout_v1 = DaypressLayout {
            version: 1,
            global_styles: None,
            root: DaypressNode {
                id: "root".into(),
                kind: NodeKind::Root,
                styles: StyleProperties::default(),
                children: vec![
                    DaypressNode {
                        id: "hero_heading".into(),
                        kind: NodeKind::Heading {
                            text: "Original V1 Headline".into(),
                            level: 1,
                        },
                        styles: StyleProperties::default(),
                        children: vec![],
                    },
                    DaypressNode {
                        id: "cta_btn".into(),
                        kind: NodeKind::Button {
                            text: "V1 CTA".into(),
                            url: "/v1".into(),
                            target: "_self".into(),
                        },
                        styles: StyleProperties::default(),
                        children: vec![],
                    },
                ],
            },
        };
        let post = db
            .create_post(&CreatePostRequest {
                title: "Diff Test Page".into(),
                slug: Some("diff-test-page".into()),
                content: "<h1>Original V1 Headline</h1>".into(),
                status: "publish".into(),
                post_type: "page".into(),
                ..Default::default()
            })
            .expect("create page");
        db.save_post_layout(
            post.id,
            &serde_json::to_string(&layout_v1).unwrap(),
            Some("<h1>Original V1 Headline</h1>"),
        )
        .expect("save v1 layout");

        let layout_v2 = DaypressLayout {
            version: 1,
            global_styles: None,
            root: DaypressNode {
                id: "root".into(),
                kind: NodeKind::Root,
                styles: StyleProperties::default(),
                children: vec![
                    DaypressNode {
                        id: "hero_heading".into(),
                        kind: NodeKind::Heading {
                            text: "Modified V2 Headline".into(),
                            level: 1,
                        },
                        styles: StyleProperties::default(),
                        children: vec![],
                    },
                    DaypressNode {
                        id: "cta_btn".into(),
                        kind: NodeKind::Button {
                            text: "V2 CTA".into(),
                            url: "/v2".into(),
                            target: "_self".into(),
                        },
                        styles: StyleProperties::default(),
                        children: vec![],
                    },
                ],
            },
        };
        db.save_post_layout(
            post.id,
            &serde_json::to_string(&layout_v2).unwrap(),
            Some("<h1>Modified V2 Headline</h1>"),
        )
        .expect("save v2 layout");

        let revs = db.get_post_revisions(post.id).expect("get revisions");
        assert!(!revs.is_empty());
        // Find the revision containing "Original V1 Headline"
        let v1_rev_id = revs
            .iter()
            .find(|r| {
                db.diff_post_revisions(post.id, r.id, None)
                    .map(|d| d.modified_count > 0)
                    .unwrap_or(false)
            })
            .map(|r| r.id)
            .expect("find v1 revision");

        let diff_report = db
            .diff_post_revisions(post.id, v1_rev_id, None)
            .expect("diff revisions");
        assert!(diff_report.modified_count >= 1);
        assert!(diff_report
            .nodes
            .iter()
            .any(|n| n.node_id == "hero_heading"));

        // 2. Cherry-pick ONLY `hero_heading` from `v1_rev_id`, keeping `cta_btn` at V2
        db.cherry_pick_revision_node(post.id, v1_rev_id, "hero_heading")
            .expect("cherry pick hero_heading");
        let patched_json = db.get_post_layout(post.id).unwrap().unwrap();
        assert!(patched_json.contains("Original V1 Headline"));
        assert!(patched_json.contains("V2 CTA"));

        // 3. Verify v0.8 MCP Tools via JSON-RPC
        let db_arc = std::sync::Arc::new(db.clone());
        let cms = CmsService::new(
            db_arc.clone(),
            db_arc.clone(),
            db_arc.clone(),
            db_arc.clone(),
            db_arc,
        );
        let rpc_langs = handle_mcp_rpc(
            &cms,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(1)),
                method: "tools/call".into(),
                params: json!({
                    "name": "daypress_list_languages",
                    "arguments": {}
                }),
            },
        );
        let langs_res = rpc_langs.result.expect("mcp languages result");
        let langs_text = langs_res["content"][0]["text"].as_str().unwrap_or("");
        assert!(langs_text.contains("\"default_language\""));

        let rpc_dup = handle_mcp_rpc(
            &cms,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(2)),
                method: "tools/call".into(),
                params: json!({
                    "name": "daypress_duplicate_post",
                    "arguments": { "post_id": post.id }
                }),
            },
        );
        let dup_res = rpc_dup.result.expect("mcp duplicate result");
        let dup_text = dup_res["content"][0]["text"].as_str().unwrap_or("");
        assert!(dup_text.contains("(Copy)"));
    }

    #[tokio::test]
    async fn test_v08_polylang_and_wpml_wxr_migration_end_to_end() {
        let db = Database::new_in_memory().expect("in-memory db");

        // Realistic Polylang + WPML WXR export with:
        // - Polylang `language` terms (`en`, `de` with locale de_DE, `ar` with rtl=1)
        // - Polylang `post_translations` serialized group mapping (`en` -> 501, `de` -> 502, `ar` -> 503)
        // - Same-slug pages across locales (`pricing` in EN, `pricing` in DE, `pricing` in AR)
        // - WPML `_wpml_import_translation_group` + `_wpml_language` postmeta on posts 601 & 602
        let wxr_xml = r#"<?xml version="1.0" encoding="UTF-8" ?>
<rss version="2.0"
    xmlns:excerpt="http://wordpress.org/export/1.2/excerpt/"
    xmlns:content="http://purl.org/rss/1.0/modules/content/"
    xmlns:dc="http://purl.org/dc/elements/1.1/"
    xmlns:wp="http://wordpress.org/export/1.2/">
<channel>
    <title>Global Polylang &amp; WPML Corp</title>
    <link>https://example.org</link>
    <description>Multilingual Enterprise Site</description>
    <wp:wxr_version>1.2</wp:wxr_version>
    <wp:term>
        <wp:term_id>10</wp:term_id>
        <wp:term_taxonomy>language</wp:term_taxonomy>
        <wp:term_slug>en</wp:term_slug>
        <wp:term_name><![CDATA[English]]></wp:term_name>
        <wp:term_description><![CDATA[a:3:{s:6:"locale";s:5:"en_US";s:3:"rtl";i:0;s:9:"flag_code";s:2:"us";}]]></wp:term_description>
    </wp:term>
    <wp:term>
        <wp:term_id>11</wp:term_id>
        <wp:term_taxonomy>language</wp:term_taxonomy>
        <wp:term_slug>de</wp:term_slug>
        <wp:term_name><![CDATA[Deutsch]]></wp:term_name>
        <wp:term_description><![CDATA[a:3:{s:6:"locale";s:5:"de_DE";s:3:"rtl";i:0;s:9:"flag_code";s:2:"de";}]]></wp:term_description>
    </wp:term>
    <wp:term>
        <wp:term_id>12</wp:term_id>
        <wp:term_taxonomy>language</wp:term_taxonomy>
        <wp:term_slug>ar</wp:term_slug>
        <wp:term_name><![CDATA[العربية]]></wp:term_name>
        <wp:term_description><![CDATA[a:3:{s:6:"locale";s:2:"ar";s:3:"rtl";i:1;s:9:"flag_code";s:2:"sa";}]]></wp:term_description>
    </wp:term>
    <wp:term>
        <wp:term_id>20</wp:term_id>
        <wp:term_taxonomy>post_translations</wp:term_taxonomy>
        <wp:term_slug>pll_pricing_group</wp:term_slug>
        <wp:term_name><![CDATA[pll_pricing_group]]></wp:term_name>
        <wp:term_description><![CDATA[a:3:{s:2:"en";i:501;s:2:"de";i:502;s:2:"ar";i:503;}]]></wp:term_description>
    </wp:term>
    <item>
        <title>Enterprise Pricing</title>
        <wp:post_id>501</wp:post_id>
        <wp:post_date>2026-05-01 10:00:00</wp:post_date>
        <wp:post_date_gmt>2026-05-01 10:00:00</wp:post_date_gmt>
        <wp:post_name>pricing</wp:post_name>
        <wp:status>publish</wp:status>
        <wp:post_type>page</wp:post_type>
        <category domain="language" nicename="en"><![CDATA[English]]></category>
        <category domain="post_translations" nicename="pll_pricing_group"><![CDATA[pll_pricing_group]]></category>
        <content:encoded><![CDATA[<p>English pricing page [daypress_language_switcher]</p>]]></content:encoded>
    </item>
    <item>
        <title>Unternehmenspreise</title>
        <wp:post_id>502</wp:post_id>
        <wp:post_date>2026-05-01 10:05:00</wp:post_date>
        <wp:post_date_gmt>2026-05-01 10:05:00</wp:post_date_gmt>
        <wp:post_name>pricing</wp:post_name>
        <wp:status>publish</wp:status>
        <wp:post_type>page</wp:post_type>
        <category domain="language" nicename="de"><![CDATA[Deutsch]]></category>
        <category domain="post_translations" nicename="pll_pricing_group"><![CDATA[pll_pricing_group]]></category>
        <content:encoded><![CDATA[<p>Deutsche Preise [daypress_language_switcher]</p>]]></content:encoded>
    </item>
    <item>
        <title>الأسعار للمؤسسات</title>
        <wp:post_id>503</wp:post_id>
        <wp:post_date>2026-05-01 10:10:00</wp:post_date>
        <wp:post_date_gmt>2026-05-01 10:10:00</wp:post_date_gmt>
        <wp:post_name>pricing</wp:post_name>
        <wp:status>publish</wp:status>
        <wp:post_type>page</wp:post_type>
        <category domain="language" nicename="ar"><![CDATA[العربية]]></category>
        <category domain="post_translations" nicename="pll_pricing_group"><![CDATA[pll_pricing_group]]></category>
        <content:encoded><![CDATA[<p>صفحة الأسعار باللغة العربية [daypress_language_switcher]</p>]]></content:encoded>
    </item>
    <item>
        <title>WPML Release Notes</title>
        <wp:post_id>601</wp:post_id>
        <wp:post_date>2026-05-02 10:00:00</wp:post_date>
        <wp:post_date_gmt>2026-05-02 10:00:00</wp:post_date_gmt>
        <wp:post_name>release-notes</wp:post_name>
        <wp:status>publish</wp:status>
        <wp:post_type>post</wp:post_type>
        <content:encoded><![CDATA[<p>English WPML post</p>]]></content:encoded>
        <wp:postmeta>
            <wp:meta_key>_wpml_import_translation_group</wp:meta_key>
            <wp:meta_value><![CDATA[wpml_trid_900]]></wp:meta_value>
        </wp:postmeta>
        <wp:postmeta>
            <wp:meta_key>_wpml_language</wp:meta_key>
            <wp:meta_value><![CDATA[en]]></wp:meta_value>
        </wp:postmeta>
    </item>
    <item>
        <title>WPML Versionshinweise</title>
        <wp:post_id>602</wp:post_id>
        <wp:post_date>2026-05-02 10:05:00</wp:post_date>
        <wp:post_date_gmt>2026-05-02 10:05:00</wp:post_date_gmt>
        <wp:post_name>versionshinweise</wp:post_name>
        <wp:status>publish</wp:status>
        <wp:post_type>post</wp:post_type>
        <content:encoded><![CDATA[<p>Deutscher WPML Beitrag</p>]]></content:encoded>
        <wp:postmeta>
            <wp:meta_key>_wpml_import_translation_group</wp:meta_key>
            <wp:meta_value><![CDATA[wpml_trid_900]]></wp:meta_value>
        </wp:postmeta>
        <wp:postmeta>
            <wp:meta_key>_wpml_language</wp:meta_key>
            <wp:meta_value><![CDATA[de]]></wp:meta_value>
        </wp:postmeta>
    </item>
</channel>
</rss>"#;

        let stats = db
            .import_wxr(std::io::Cursor::new(wxr_xml.as_bytes()))
            .expect("import polylang+wpml wxr");
        assert!(stats.imported_posts + stats.imported_pages >= 5);

        // Verify languages (`en`, `de` with locale de_DE, `ar` with is_rtl = true)
        let langs = db.list_languages().expect("list languages");
        let de_lang = langs
            .iter()
            .find(|l| l.code == "de")
            .expect("de language imported");
        assert_eq!(de_lang.locale, "de_DE");
        let ar_lang = langs
            .iter()
            .find(|l| l.code == "ar")
            .expect("ar language imported");
        assert!(
            ar_lang.is_rtl,
            "Arabic must be marked RTL from Polylang serialized description"
        );

        // Verify same-slug pages (`pricing` in EN, DE, and AR) all preserved slug `pricing` and linked together
        let en_pricing = db
            .get_post_by_lang_and_slug("en", "pricing")
            .expect("query en pricing")
            .expect("en pricing exists");
        let de_pricing = db
            .get_post_by_lang_and_slug("de", "pricing")
            .expect("query de pricing")
            .expect("de pricing exists");
        let ar_pricing = db
            .get_post_by_lang_and_slug("ar", "pricing")
            .expect("query ar pricing")
            .expect("ar pricing exists");
        assert_eq!(en_pricing.title.rendered, "Enterprise Pricing");
        assert_eq!(de_pricing.title.rendered, "Unternehmenspreise");
        assert_eq!(ar_pricing.title.rendered, "الأسعار للمؤسسات");
        assert_eq!(en_pricing.translations.get("de"), Some(&de_pricing.id));
        assert_eq!(en_pricing.translations.get("ar"), Some(&ar_pricing.id));

        // Verify WPML postmeta translation group linked posts 601 & 602
        let wpml_de = db
            .get_post_by_lang_and_slug("de", "versionshinweise")
            .expect("query wpml de")
            .expect("wpml de exists");
        assert!(
            wpml_de.translations.contains_key("en"),
            "WPML _wpml_import_translation_group must link EN and DE posts"
        );

        // Verify SSR on /ar/pricing/ renders <html lang="ar" dir="rtl"> and hreflang cluster
        let app = create_app(db.clone());
        let res_ar = app
            .oneshot(
                Request::builder()
                    .uri("/ar/pricing/")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(res_ar.status(), StatusCode::OK);
        let html_ar = String::from_utf8_lossy(
            &axum::body::to_bytes(res_ar.into_body(), usize::MAX)
                .await
                .unwrap(),
        )
        .to_string();
        assert!(
            html_ar.contains(r#"<html lang="ar" dir="rtl""#),
            "Arabic SSR must emit <html lang=\"ar\" dir=\"rtl\">"
        );
        assert!(
            html_ar.contains(r#"hreflang="de" href="http://localhost:8090/de/pricing/""#),
            "Arabic SSR must include German hreflang alternate"
        );
    }
}
