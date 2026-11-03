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
