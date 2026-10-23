//! # REST API Router & Submodules (`src/api/`)
//!
//! Modularized into 5 focused domain route groups:
//! * [`wp_posts`]: `/wp/v2/posts` & `/wp/v2/pages` CRUD.
//! * [`wp_taxonomies`]: `/wp/v2/categories`, `/tags`, `/menus`, `/widgets`, `/themes`, `/plugins`, `/search`, `/types`, `/statuses`, `/taxonomies`.
//! * [`wp_users_comments`]: `/wp/v2/users`, `/comments`, Application Passwords, Nonces, and JWT tokens.
//! * [`wp_media_settings`]: `/wp/v2/settings`, `/wp/v2/media` multipart uploads, and SQLite health/vacuum.
//! * [`neopress_v1`]: `/neopress/v1/import`, `/import-wxr`, `/layout/{id}`, `/forms/{id}/submit`, `/analytics/*`, and `/email/send`.

pub mod neopress_v1;
pub mod wp_media_settings;
pub mod wp_posts;
pub mod wp_taxonomies;
pub mod wp_users_comments;

use crate::db::Database;
use axum::{
    extract::State,
    http::{header, HeaderMap},
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub(crate) use neopress_v1::*;
pub(crate) use wp_media_settings::*;
pub(crate) use wp_posts::*;
pub(crate) use wp_taxonomies::*;
pub(crate) use wp_users_comments::*;

#[derive(Debug, Deserialize, Default)]
pub struct PostQueryParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
    pub offset: Option<i64>,
    pub slug: Option<String>,
    pub author: Option<i64>,
    pub categories: Option<String>,
    pub tags: Option<String>,
    pub parent: Option<i64>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub orderby: Option<String>,
    pub order: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DeleteQueryParams {
    pub force: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct CommentListQueryParams {
    pub post: Option<i64>,
    pub status: Option<String>,
    pub per_page: Option<i64>,
    pub page: Option<i64>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SearchQueryParams {
    pub search: Option<String>,
    pub per_page: Option<i64>,
    pub page: Option<i64>,
    pub subtype: Option<String>,
}

pub(crate) fn wp_list_response<T: Serialize>(
    items: Vec<T>,
    total: i64,
    per_page: i64,
) -> (HeaderMap, Json<Vec<T>>) {
    let mut headers = HeaderMap::new();
    let per_page_safe = per_page.max(1);
    let total_pages = if total <= 0 {
        0
    } else {
        (total + per_page_safe - 1) / per_page_safe
    };
    if let Ok(v) = total.to_string().parse() {
        headers.insert("X-WP-Total", v);
    }
    if let Ok(v) = total_pages.to_string().parse() {
        headers.insert("X-WP-TotalPages", v);
    }
    headers.insert(
        header::ACCESS_CONTROL_EXPOSE_HEADERS,
        header::HeaderValue::from_static("X-WP-Total, X-WP-TotalPages"),
    );
    (headers, Json(items))
}

pub(crate) fn parse_id_list(raw: Option<&str>) -> Vec<i64> {
    raw.unwrap_or("")
        .split(',')
        .filter_map(|s| s.trim().parse::<i64>().ok())
        .collect()
}

pub fn api_routes() -> Router<Database> {
    Router::new()
        // Core WordPress REST API v2
        .route("/wp/v2/posts", get(list_posts).post(create_post))
        .route(
            "/wp/v2/posts/{id}",
            get(get_post)
                .put(update_post)
                .post(update_post)
                .patch(update_post)
                .delete(delete_post),
        )
        .route("/wp/v2/pages", get(list_pages).post(create_page))
        .route(
            "/wp/v2/pages/{id}",
            get(get_page)
                .put(update_page)
                .post(update_page)
                .patch(update_page)
                .delete(delete_page),
        )
        .route("/wp/v2/media", get(list_media).post(upload_media))
        .route(
            "/wp/v2/media/{id}",
            get(get_media_item)
                .put(update_media_item)
                .post(update_media_item)
                .patch(update_media_item)
                .delete(delete_media),
        )
        .route(
            "/wp/v2/settings",
            get(get_settings)
                .post(update_settings)
                .put(update_settings)
                .patch(update_settings),
        )
        .route("/wp/v2/users", get(list_users).post(create_user))
        .route(
            "/wp/v2/users/me",
            get(get_current_user)
                .post(update_current_user)
                .put(update_current_user)
                .patch(update_current_user),
        )
        .route(
            "/wp/v2/users/{id}",
            get(get_user_by_id)
                .put(update_user_by_id)
                .post(update_user_by_id)
                .patch(update_user_by_id)
                .delete(delete_user),
        )
        .route(
            "/wp/v2/categories",
            get(list_categories).post(create_category_handler),
        )
        .route(
            "/wp/v2/categories/{id}",
            get(get_category_handler)
                .put(update_category_handler)
                .post(update_category_handler)
                .patch(update_category_handler)
                .delete(delete_category_handler),
        )
        .route("/wp/v2/tags", get(list_tags).post(create_tag_handler))
        .route(
            "/wp/v2/tags/{id}",
            get(get_tag_handler)
                .put(update_tag_handler)
                .post(update_tag_handler)
                .patch(update_tag_handler)
                .delete(delete_tag_handler),
        )
        .route(
            "/wp/v2/comments",
            get(list_comments).post(create_comment_handler),
        )
        .route(
            "/wp/v2/comments/{id}",
            get(get_comment)
                .post(update_comment_handler)
                .put(update_comment_handler)
                .patch(update_comment_handler)
                .delete(delete_comment_handler),
        )
        .route("/wp/v2/menus", get(get_menus_handler).post(save_menus_handler))
        .route(
            "/wp/v2/widgets",
            get(get_widgets_handler).post(save_widgets_handler),
        )
        .route("/wp/v2/themes", get(get_themes_handler))
        .route("/wp/v2/plugins", get(get_plugins_handler))
        .route("/wp/v2/search", get(search_content_handler))
        .route("/wp/v2/types", get(get_post_types_handler))
        .route("/wp/v2/statuses", get(get_post_statuses_handler))
        .route("/wp/v2/taxonomies", get(get_taxonomies_handler))
        // Application Passwords API (WordPress 5.6+ standard)
        .route(
            "/wp/v2/users/{id}/application-passwords",
            get(list_app_passwords).post(create_app_password),
        )
        .route(
            "/wp/v2/users/{id}/application-passwords/{uuid}",
            axum::routing::delete(delete_app_password),
        )
        // JWT Authentication Endpoints (standard WordPress headless ecosystem)
        .route(
            "/jwt-auth/v1/token",
            axum::routing::post(generate_jwt_token),
        )
        .route(
            "/jwt-auth/v1/token/validate",
            axum::routing::post(validate_jwt_token),
        )
        // Neopress v1 High-Performance Extensions
        .route("/neopress/v1/auth/nonce", get(get_auth_nonce))
        .route(
            "/neopress/v1/auth/token",
            axum::routing::post(generate_jwt_token),
        )
        .route("/neopress/v1/stats", get(get_db_stats))
        .route("/neopress/v1/vacuum", get(vacuum_db).post(vacuum_db))
        .route("/neopress/v1/integrity-check", get(check_integrity))
        .route("/neopress/v1/import", axum::routing::post(import_bundle))
        .route(
            "/neopress/v1/import-wxr",
            axum::routing::post(import_wxr_handler),
        )
        // Visual Builder Layout API (Hybrid Wasm / JSON schema)
        .route(
            "/neopress/v1/layout/{id}",
            get(get_layout_handler)
                .post(save_layout_handler)
                .delete(delete_layout_handler),
        )
        // Always-On Core Utility: 1-Click Duplicate Post / Page
        .route(
            "/neopress/v1/posts/{id}/duplicate",
            axum::routing::post(duplicate_post_handler),
        )
        // Native Subsystem: Forms & Anti-Spam Submission
        .route(
            "/neopress/v1/forms/{id}/submit",
            axum::routing::post(submit_form_handler),
        )
        // Native Subsystem: Cookieless Analytics Ingestion & Reporting
        .route(
            "/neopress/v1/analytics/event",
            axum::routing::post(record_analytics_event_handler),
        )
        .route(
            "/neopress/v1/analytics/summary",
            get(get_analytics_summary_handler),
        )
        // Native Subsystem: Transactional Email Dispatcher
        .route(
            "/neopress/v1/email/send",
            axum::routing::post(send_email_handler),
        )
        // Theme Template Builder REST API
        .merge(crate::theme_builder::routes::theme_builder_routes())
}

pub async fn wp_json_root(State(db): State<Database>) -> Json<Value> {
    let settings = db.get_settings().unwrap_or_default();

    Json(serde_json::json!({
        "name": settings.title,
        "description": settings.description,
        "url": settings.url,
        "home": settings.url,
        "gmt_offset": "0",
        "timezone_string": settings.timezone,
        "namespaces": [
            "wp/v2",
            "jwt-auth/v1",
            "neopress/v1"
        ],
        "authentication": {
            "application_passwords": {
                "endpoints": {
                    "authorization": "/wp-json/wp/v2/users/me/application-passwords"
                }
            },
            "jwt": {
                "endpoints": {
                    "token": "/wp-json/jwt-auth/v1/token",
                    "validate": "/wp-json/jwt-auth/v1/token/validate"
                }
            },
            "nonce": {
                "endpoint": "/wp-json/neopress/v1/auth/nonce"
            }
        },
        "routes": {
            "/": { "namespace": "", "methods": ["GET"] },
            "/wp/v2/posts": { "namespace": "wp/v2", "methods": ["GET", "POST"] },
            "/wp/v2/posts/(?P<id>[\\d]+)": { "namespace": "wp/v2", "methods": ["GET", "PUT", "POST", "PATCH", "DELETE"] },
            "/wp/v2/pages": { "namespace": "wp/v2", "methods": ["GET", "POST"] },
            "/wp/v2/pages/(?P<id>[\\d]+)": { "namespace": "wp/v2", "methods": ["GET", "PUT", "POST", "PATCH", "DELETE"] },
            "/wp/v2/media": { "namespace": "wp/v2", "methods": ["GET", "POST"] },
            "/wp/v2/media/(?P<id>[\\d]+)": { "namespace": "wp/v2", "methods": ["GET", "PUT", "POST", "PATCH", "DELETE"] },
            "/wp/v2/categories": { "namespace": "wp/v2", "methods": ["GET", "POST"] },
            "/wp/v2/categories/(?P<id>[\\d]+)": { "namespace": "wp/v2", "methods": ["GET", "PUT", "POST", "PATCH", "DELETE"] },
            "/wp/v2/tags": { "namespace": "wp/v2", "methods": ["GET", "POST"] },
            "/wp/v2/tags/(?P<id>[\\d]+)": { "namespace": "wp/v2", "methods": ["GET", "PUT", "POST", "PATCH", "DELETE"] },
            "/wp/v2/comments": { "namespace": "wp/v2", "methods": ["GET", "POST"] },
            "/wp/v2/comments/(?P<id>[\\d]+)": { "namespace": "wp/v2", "methods": ["GET", "PUT", "POST", "PATCH", "DELETE"] },
            "/wp/v2/settings": { "namespace": "wp/v2", "methods": ["GET", "POST", "PUT", "PATCH"] },
            "/wp/v2/users": { "namespace": "wp/v2", "methods": ["GET", "POST"] },
            "/wp/v2/users/me": { "namespace": "wp/v2", "methods": ["GET", "POST", "PUT", "PATCH"] },
            "/wp/v2/users/(?P<id>[\\d]+)": { "namespace": "wp/v2", "methods": ["GET", "PUT", "POST", "PATCH", "DELETE"] },
            "/wp/v2/users/(?P<id>[\\d]+)/application-passwords": { "namespace": "wp/v2", "methods": ["GET", "POST"] },
            "/wp/v2/menus": { "namespace": "wp/v2", "methods": ["GET", "POST"] },
            "/wp/v2/widgets": { "namespace": "wp/v2", "methods": ["GET", "POST"] },
            "/wp/v2/themes": { "namespace": "wp/v2", "methods": ["GET"] },
            "/wp/v2/plugins": { "namespace": "wp/v2", "methods": ["GET"] },
            "/wp/v2/search": { "namespace": "wp/v2", "methods": ["GET"] },
            "/wp/v2/types": { "namespace": "wp/v2", "methods": ["GET"] },
            "/wp/v2/statuses": { "namespace": "wp/v2", "methods": ["GET"] },
            "/wp/v2/taxonomies": { "namespace": "wp/v2", "methods": ["GET"] },
            "/jwt-auth/v1/token": { "namespace": "jwt-auth/v1", "methods": ["POST"] },
            "/jwt-auth/v1/token/validate": { "namespace": "jwt-auth/v1", "methods": ["POST"] },
            "/neopress/v1/auth/nonce": { "namespace": "neopress/v1", "methods": ["GET"] },
            "/neopress/v1/stats": { "namespace": "neopress/v1", "methods": ["GET"] },
            "/neopress/v1/vacuum": { "namespace": "neopress/v1", "methods": ["GET", "POST"] },
            "/neopress/v1/integrity-check": { "namespace": "neopress/v1", "methods": ["GET"] },
            "/neopress/v1/import": { "namespace": "neopress/v1", "methods": ["POST"] },
            "/neopress/v1/import-wxr": { "namespace": "neopress/v1", "methods": ["POST"] }
        },
        "_links": {
            "help": [{ "href": "https://developer.wordpress.org/rest-api/" }]
        }
    }))
}
