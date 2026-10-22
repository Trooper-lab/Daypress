use crate::auth::AuthUser;
use crate::db::Database;
use crate::errors::AppError;
use crate::models::*;
use axum::{
    body::to_bytes,
    extract::{FromRequest, Multipart, Path as AxPath, Query, Request, State},
    http::{header, HeaderMap, StatusCode},
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::Cursor;
use std::path::Path;

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

fn wp_list_response<T: Serialize>(
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

fn parse_id_list(raw: Option<&str>) -> Vec<i64> {
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

async fn list_posts(
    State(db): State<Database>,
    Query(params): Query<PostQueryParams>,
) -> Result<(HeaderMap, Json<Vec<WpPostResponse>>), AppError> {
    let per_page = params.per_page.unwrap_or(10).clamp(1, 500) as usize;
    if let Some(slug) = params.slug.as_deref() {
        if !slug.is_empty()
            && params.status.is_none()
            && params.author.is_none()
            && params.categories.is_none()
            && params.tags.is_none()
        {
            let p = db.get_post_by_slug(slug)?;
            let vec: Vec<WpPostResponse> = p.into_iter().collect();
            let total = vec.len() as i64;
            return Ok(wp_list_response(vec, total, per_page as i64));
        }
    }

    let page = params.page.unwrap_or(1).max(1) as usize;
    let offset = params
        .offset
        .map(|o| o.max(0) as usize)
        .unwrap_or_else(|| (page - 1) * per_page);
    let cats = parse_id_list(params.categories.as_deref());
    let tags = parse_id_list(params.tags.as_deref());

    let (posts, total) = db.query_posts(
        "post",
        params.status.as_deref(),
        params.search.as_deref(),
        params.slug.as_deref(),
        params.author,
        params.parent,
        &cats,
        &tags,
        params.orderby.as_deref(),
        params.order.as_deref(),
        offset,
        per_page,
    )?;
    Ok(wp_list_response(posts, total as i64, per_page as i64))
}

async fn get_post(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<WpPostResponse>, AppError> {
    match db.get_post_by_id(id)? {
        Some(post) => Ok(Json(post)),
        None => Err(AppError::NotFound(format!("Post {} not found", id))),
    }
}

async fn create_post(
    _auth: AuthUser,
    State(db): State<Database>,
    Json(payload): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<WpPostResponse>), AppError> {
    let post = db.create_post(&payload)?;
    Ok((StatusCode::CREATED, Json(post)))
}

async fn update_post(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdatePostRequest>,
) -> Result<Json<WpPostResponse>, AppError> {
    match db.update_post(id, &payload)? {
        Some(post) => Ok(Json(post)),
        None => Err(AppError::NotFound(format!("Post {} not found", id))),
    }
}

async fn delete_post(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Query(params): Query<DeleteQueryParams>,
) -> Result<Json<Value>, AppError> {
    let existing = db
        .get_post_by_id(id)?
        .ok_or_else(|| AppError::NotFound(format!("Post {} not found", id)))?;

    let force = params.force.unwrap_or(false) || existing.status == "trash";
    if force {
        db.delete_post(id)?;
        Ok(Json(serde_json::json!({
            "deleted": true,
            "previous": existing
        })))
    } else {
        let trashed = db
            .trash_post(id)?
            .ok_or_else(|| AppError::NotFound(format!("Post {} not found", id)))?;
        let mut val = serde_json::to_value(&trashed).unwrap_or_default();
        if let Some(obj) = val.as_object_mut() {
            obj.insert("deleted".to_string(), serde_json::json!(true));
            obj.insert("trashed".to_string(), serde_json::json!(true));
            obj.insert(
                "previous".to_string(),
                serde_json::to_value(&existing).unwrap_or_default(),
            );
        }
        Ok(Json(val))
    }
}

async fn list_pages(
    State(db): State<Database>,
    Query(params): Query<PostQueryParams>,
) -> Result<(HeaderMap, Json<Vec<WpPostResponse>>), AppError> {
    let per_page = params.per_page.unwrap_or(10).clamp(1, 500) as usize;
    let page = params.page.unwrap_or(1).max(1) as usize;
    let offset = params
        .offset
        .map(|o| o.max(0) as usize)
        .unwrap_or_else(|| (page - 1) * per_page);
    let cats = parse_id_list(params.categories.as_deref());
    let tags = parse_id_list(params.tags.as_deref());

    let (pages, total) = db.query_posts(
        "page",
        params.status.as_deref(),
        params.search.as_deref(),
        params.slug.as_deref(),
        params.author,
        params.parent,
        &cats,
        &tags,
        params.orderby.as_deref(),
        params.order.as_deref(),
        offset,
        per_page,
    )?;
    Ok(wp_list_response(pages, total as i64, per_page as i64))
}

async fn get_page(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<WpPostResponse>, AppError> {
    match db.get_post_by_id(id)? {
        Some(page) if page.post_type == "page" => Ok(Json(page)),
        _ => Err(AppError::NotFound(format!("Page {} not found", id))),
    }
}

async fn create_page(
    _auth: AuthUser,
    State(db): State<Database>,
    Json(mut payload): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<WpPostResponse>), AppError> {
    payload.post_type = "page".to_string();
    let page = db.create_post(&payload)?;
    Ok((StatusCode::CREATED, Json(page)))
}

async fn update_page(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdatePostRequest>,
) -> Result<Json<WpPostResponse>, AppError> {
    match db.update_post(id, &payload)? {
        Some(page) => Ok(Json(page)),
        None => Err(AppError::NotFound(format!("Page {} not found", id))),
    }
}

async fn delete_page(
    auth: AuthUser,
    state: State<Database>,
    path: AxPath<i64>,
    query: Query<DeleteQueryParams>,
) -> Result<Json<Value>, AppError> {
    delete_post(auth, state, path, query).await
}

async fn get_settings(State(db): State<Database>) -> Result<Json<WpSettingsResponse>, AppError> {
    let settings = db.get_settings()?;
    Ok(Json(settings))
}

async fn update_settings(
    _auth: AuthUser,
    State(db): State<Database>,
    Json(payload): Json<UpdateSettingsRequest>,
) -> Result<Json<WpSettingsResponse>, AppError> {
    if let Some(title) = payload.title {
        db.set_option("blogname", &title)?;
    }
    if let Some(desc) = payload.description {
        db.set_option("blogdescription", &desc)?;
    }
    if let Some(email) = payload.email {
        db.set_option("admin_email", &email)?;
    }
    if let Some(url) = payload.url {
        db.set_option("siteurl", &url)?;
        db.set_option("home", &url)?;
    }
    if let Some(tz) = payload.timezone_string.or(payload.timezone) {
        db.set_option("timezone_string", &tz)?;
    }
    if let Some(df) = payload.date_format {
        db.set_option("date_format", &df)?;
    }
    if let Some(tf) = payload.time_format {
        db.set_option("time_format", &tf)?;
    }
    if let Some(ppp) = payload.posts_per_page {
        db.set_option("posts_per_page", &ppp.to_string())?;
    }
    if let Some(dcs) = payload.default_comment_status {
        db.set_option("default_comment_status", &dcs)?;
    }
    if let Some(sof) = payload.show_on_front {
        db.set_option("show_on_front", &sof)?;
    }
    if let Some(pof) = payload.page_on_front {
        db.set_option("page_on_front", &pof.to_string())?;
    }
    if let Some(pfp) = payload.page_for_posts {
        db.set_option("page_for_posts", &pfp.to_string())?;
    }
    if let Some(ps) = payload.permalink_structure {
        db.set_option("permalink_structure", &ps)?;
    }
    if let Some(theme) = payload.active_theme {
        db.set_option("neopress_active_theme", &theme)?;
    }
    if let Some(color) = payload.brand_color {
        db.set_option("neopress_brand_color", &color)?;
    }
    if let Some(scale) = payload.typography_scale {
        db.set_option("neopress_typography_scale", &scale)?;
    }
    if let Some(css) = payload.custom_css {
        db.set_option("neopress_custom_css", &css)?;
    }
    if let Some(plugins) = payload.active_plugins {
        if let Ok(json) = serde_json::to_string(&plugins) {
            db.set_option("neopress_active_plugins", &json)?;
        }
    }
    if let Some(menus) = payload.nav_menu_items {
        if let Ok(json) = serde_json::to_string(&menus) {
            db.set_option("neopress_nav_menu_items", &json)?;
        }
    }
    if let Some(widgets) = payload.widgets {
        if let Ok(json) = serde_json::to_string(&widgets) {
            db.set_option("neopress_widgets", &json)?;
        }
    }

    let updated = db.get_settings()?;
    Ok(Json(updated))
}

async fn list_users(
    State(db): State<Database>,
) -> Result<(HeaderMap, Json<Vec<WpUserResponse>>), AppError> {
    let base_url = db.get_base_url();
    let users = db.get_all_users()?;
    let list: Vec<WpUserResponse> = users
        .into_iter()
        .map(|u| WpUserResponse {
            id: u.id,
            name: u.display_name,
            url: u.url,
            description: "".to_string(),
            link: format!("{}/author/{}/", base_url.trim_end_matches('/'), u.username),
            slug: u.username,
            avatar_urls: serde_json::json!({
                "24": "https://secure.gravatar.com/avatar/?s=24&d=mm&r=g",
                "48": "https://secure.gravatar.com/avatar/?s=48&d=mm&r=g",
                "96": "https://secure.gravatar.com/avatar/?s=96&d=mm&r=g"
            }),
        })
        .collect();
    let total = list.len() as i64;
    Ok(wp_list_response(list, total, total.max(10)))
}

async fn create_user(
    _auth: AuthUser,
    State(db): State<Database>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<WpUserDetail>), AppError> {
    let user = db.create_user(&payload)?;
    Ok((StatusCode::CREATED, Json(user)))
}

async fn delete_user(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    if db.delete_user(id)? {
        Ok(Json(serde_json::json!({
            "deleted": true,
            "previous": { "id": id }
        })))
    } else {
        Err(AppError::NotFound(format!("User {} not found", id)))
    }
}

async fn list_categories(
    State(db): State<Database>,
) -> Result<(HeaderMap, Json<Vec<WpCategoryResponse>>), AppError> {
    let cats = db.get_terms("category")?;
    let total = cats.len() as i64;
    Ok(wp_list_response(cats, total, total.max(10)))
}

async fn get_category_handler(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<WpCategoryResponse>, AppError> {
    match db.get_term_by_id("category", id)? {
        Some(c) => Ok(Json(c)),
        None => Err(AppError::NotFound(format!("Category {} not found", id))),
    }
}

async fn create_category_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    Json(payload): Json<CreateTermRequest>,
) -> Result<(StatusCode, Json<WpCategoryResponse>), AppError> {
    let cat = db.create_category(&payload)?;
    Ok((StatusCode::CREATED, Json(cat)))
}

async fn update_category_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdateTermRequest>,
) -> Result<Json<WpCategoryResponse>, AppError> {
    match db.update_term("category", id, &payload)? {
        Some(c) => Ok(Json(c)),
        None => Err(AppError::NotFound(format!("Category {} not found", id))),
    }
}

async fn delete_category_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    let prev = db.get_term_by_id("category", id)?;
    if db.delete_term("category", id)? {
        Ok(Json(serde_json::json!({
            "deleted": true,
            "previous": prev.unwrap_or_else(|| WpCategoryResponse {
                id,
                count: 0,
                description: String::new(),
                link: String::new(),
                name: String::new(),
                slug: String::new(),
                taxonomy: "category".to_string(),
                parent: 0,
            })
        })))
    } else {
        Err(AppError::NotFound(format!("Category {} not found", id)))
    }
}

async fn list_tags(
    State(db): State<Database>,
) -> Result<(HeaderMap, Json<Vec<WpCategoryResponse>>), AppError> {
    let tags = db.get_terms("post_tag")?;
    let total = tags.len() as i64;
    Ok(wp_list_response(tags, total, total.max(10)))
}

async fn get_tag_handler(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<WpCategoryResponse>, AppError> {
    match db.get_term_by_id("post_tag", id)? {
        Some(t) => Ok(Json(t)),
        None => Err(AppError::NotFound(format!("Tag {} not found", id))),
    }
}

async fn create_tag_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    Json(payload): Json<CreateTermRequest>,
) -> Result<(StatusCode, Json<WpCategoryResponse>), AppError> {
    let tag = db.create_tag(&payload)?;
    Ok((StatusCode::CREATED, Json(tag)))
}

async fn update_tag_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdateTermRequest>,
) -> Result<Json<WpCategoryResponse>, AppError> {
    match db.update_term("post_tag", id, &payload)? {
        Some(t) => Ok(Json(t)),
        None => Err(AppError::NotFound(format!("Tag {} not found", id))),
    }
}

async fn delete_tag_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    let prev = db.get_term_by_id("post_tag", id)?;
    if db.delete_term("post_tag", id)? {
        Ok(Json(serde_json::json!({
            "deleted": true,
            "previous": prev.unwrap_or_else(|| WpCategoryResponse {
                id,
                count: 0,
                description: String::new(),
                link: String::new(),
                name: String::new(),
                slug: String::new(),
                taxonomy: "post_tag".to_string(),
                parent: 0,
            })
        })))
    } else {
        Err(AppError::NotFound(format!("Tag {} not found", id)))
    }
}

async fn list_comments(
    State(db): State<Database>,
    Query(params): Query<CommentListQueryParams>,
) -> Result<(HeaderMap, Json<Vec<WpCommentAdminItem>>), AppError> {
    let per_page = params.per_page.unwrap_or(100).clamp(1, 500);
    let mut comments = db
        .get_comments(params.status.as_deref(), 500)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if let Some(post_id) = params.post {
        comments.retain(|c| c.post_id == post_id);
    }
    let total = comments.len() as i64;
    let page = params.page.unwrap_or(1).max(1) as usize;
    let offset = (page - 1) * (per_page as usize);
    let paged: Vec<WpCommentAdminItem> = comments
        .into_iter()
        .skip(offset)
        .take(per_page as usize)
        .collect();
    Ok(wp_list_response(paged, total, per_page))
}

async fn create_comment_handler(
    State(db): State<Database>,
    Json(payload): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<WpCommentResponse>), AppError> {
    if payload.content.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Comment content is required".to_string(),
        ));
    }
    let comment = db.create_comment(&payload)?;
    Ok((StatusCode::CREATED, Json(comment)))
}

async fn get_comment(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<WpCommentAdminItem>, AppError> {
    if let Some(c) = db.get_comment_by_id(id)? {
        Ok(Json(c))
    } else {
        Err(AppError::NotFound(format!("Comment {} not found", id)))
    }
}

async fn update_comment_handler(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdateCommentRequest>,
) -> Result<Json<Value>, AppError> {
    match db.update_comment(id, &payload)? {
        Some(updated) => {
            let mut val = serde_json::to_value(&updated).unwrap_or_default();
            if let Some(obj) = val.as_object_mut() {
                obj.insert("success".to_string(), serde_json::json!(true));
            }
            Ok(Json(val))
        }
        None => Err(AppError::NotFound(format!("Comment {} not found", id))),
    }
}

async fn delete_comment_handler(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Query(params): Query<DeleteQueryParams>,
) -> Result<Json<Value>, AppError> {
    let existing = db
        .get_comment_by_id(id)?
        .ok_or_else(|| AppError::NotFound(format!("Comment {} not found", id)))?;
    let force = params.force.unwrap_or(true);
    if force {
        db.delete_comment(id)?;
        Ok(Json(serde_json::json!({
            "deleted": true,
            "previous": existing
        })))
    } else {
        db.update_comment_status(id, "trash")?;
        Ok(Json(serde_json::json!({
            "deleted": true,
            "trashed": true,
            "status": "trash",
            "previous": existing
        })))
    }
}

async fn get_menus_handler(State(db): State<Database>) -> Result<Json<Value>, AppError> {
    let settings = db.get_settings()?;
    Ok(Json(serde_json::json!({
        "name": "Primary Navigation",
        "slug": "primary-navigation",
        "locations": ["primary", "header"],
        "items": settings.nav_menu_items
    })))
}

async fn save_menus_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let items_val = payload
        .get("items")
        .or_else(|| payload.get("nav_menu_items"))
        .cloned()
        .unwrap_or(payload);
    let items: Vec<NavMenuItemDto> = serde_json::from_value(items_val).unwrap_or_default();
    let json_str = serde_json::to_string(&items)
        .map_err(|e| AppError::BadRequest(format!("Invalid menu items: {}", e)))?;
    db.set_option("neopress_nav_menu_items", &json_str)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "items": items
    })))
}

async fn get_widgets_handler(State(db): State<Database>) -> Result<Json<Value>, AppError> {
    let settings = db.get_settings()?;
    let val: Value = serde_json::from_str(&settings.widgets_json).unwrap_or_else(|_| {
        serde_json::json!({
            "sidebar": ["Search Bar", "Recent Posts"],
            "footer": ["Categories List"]
        })
    });
    Ok(Json(val))
}

async fn save_widgets_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let widgets_val = payload.get("widgets").cloned().unwrap_or(payload);
    let json_str = serde_json::to_string(&widgets_val)
        .map_err(|e| AppError::BadRequest(format!("Invalid widgets payload: {}", e)))?;
    db.set_option("neopress_widgets", &json_str)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "widgets": widgets_val
    })))
}

async fn get_themes_handler(State(db): State<Database>) -> Result<Json<Value>, AppError> {
    let settings = db.get_settings()?;
    let active = if settings.active_theme.is_empty() {
        "Neopress Modern".to_string()
    } else {
        settings.active_theme
    };
    Ok(Json(serde_json::json!([
        {
            "stylesheet": "neopress-modern",
            "template": "neopress-modern",
            "name": { "raw": "Neopress Modern", "rendered": "Neopress Modern" },
            "version": "1.0.0",
            "status": if active.eq_ignore_ascii_case("Neopress Modern") { "active" } else { "inactive" }
        },
        {
            "stylesheet": "twenty-twenty-five",
            "template": "twenty-twenty-five",
            "name": { "raw": "Twenty Twenty-Five", "rendered": "Twenty Twenty-Five" },
            "version": "1.0",
            "status": if active.eq_ignore_ascii_case("Twenty Twenty-Five") { "active" } else { "inactive" }
        },
        {
            "stylesheet": "editorial-minimalist",
            "template": "editorial-minimalist",
            "name": { "raw": "Editorial Minimalist", "rendered": "Editorial Minimalist" },
            "version": "2.1",
            "status": if active.eq_ignore_ascii_case("Editorial Minimalist") { "active" } else { "inactive" }
        }
    ])))
}

async fn get_plugins_handler(State(db): State<Database>) -> Result<Json<Value>, AppError> {
    let settings = db.get_settings()?;
    let is_active = |slug: &str| {
        settings.active_plugins.is_empty()
            || settings
                .active_plugins
                .iter()
                .any(|p| p.eq_ignore_ascii_case(slug))
    };
    Ok(Json(serde_json::json!([
        {
            "plugin": "neopress-wal-optimizer",
            "name": "Neopress WAL Optimizer",
            "version": "1.0.0",
            "status": if is_active("neopress-wal-optimizer") { "active" } else { "inactive" }
        },
        {
            "plugin": "webp-image-engine",
            "name": "WebP Image Engine",
            "version": "2.4.1",
            "status": if is_active("webp-image-engine") { "active" } else { "inactive" }
        },
        {
            "plugin": "wxr-streaming-xml-importer",
            "name": "WXR Streaming XML Importer",
            "version": "1.2.0",
            "status": if is_active("wxr-streaming-xml-importer") { "active" } else { "inactive" }
        }
    ])))
}

async fn search_content_handler(
    State(db): State<Database>,
    Query(params): Query<SearchQueryParams>,
) -> Result<(HeaderMap, Json<Vec<Value>>), AppError> {
    let per_page = params.per_page.unwrap_or(10).clamp(1, 100);
    let page = params.page.unwrap_or(1).max(1) as usize;
    let offset = (page - 1) * (per_page as usize);
    let post_type = params.subtype.as_deref().unwrap_or("any");
    let (posts, total) = db.query_posts(
        post_type,
        Some("publish"),
        params.search.as_deref(),
        None,
        None,
        None,
        &[],
        &[],
        None,
        None,
        offset,
        per_page as usize,
    )?;
    let results: Vec<Value> = posts
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "title": p.title.rendered,
                "url": p.link,
                "type": "post",
                "subtype": p.post_type
            })
        })
        .collect();
    Ok(wp_list_response(results, total as i64, per_page))
}

async fn get_post_types_handler() -> Json<Value> {
    Json(serde_json::json!({
        "post": {
            "name": "Posts",
            "slug": "post",
            "hierarchical": false,
            "rest_base": "posts"
        },
        "page": {
            "name": "Pages",
            "slug": "page",
            "hierarchical": true,
            "rest_base": "pages"
        },
        "attachment": {
            "name": "Media",
            "slug": "attachment",
            "hierarchical": false,
            "rest_base": "media"
        }
    }))
}

async fn get_post_statuses_handler() -> Json<Value> {
    Json(serde_json::json!({
        "publish": { "name": "Published", "public": true, "slug": "publish" },
        "draft": { "name": "Draft", "public": false, "slug": "draft" },
        "pending": { "name": "Pending", "public": false, "slug": "pending" },
        "private": { "name": "Private", "public": false, "slug": "private" },
        "trash": { "name": "Trash", "public": false, "slug": "trash" }
    }))
}

async fn get_taxonomies_handler() -> Json<Value> {
    Json(serde_json::json!({
        "category": {
            "name": "Categories",
            "slug": "category",
            "types": ["post"],
            "hierarchical": true,
            "rest_base": "categories"
        },
        "post_tag": {
            "name": "Tags",
            "slug": "post_tag",
            "types": ["post"],
            "hierarchical": false,
            "rest_base": "tags"
        }
    }))
}

async fn get_db_stats(State(db): State<Database>) -> Result<Json<DbStats>, AppError> {
    let stats = db.get_stats()?;
    Ok(Json(stats))
}

async fn vacuum_db(_auth: AuthUser, State(db): State<Database>) -> Result<Json<Value>, AppError> {
    db.vacuum()?;
    let stats = db.get_stats()?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "SQLite VACUUM completed successfully",
        "new_size_bytes": stats.file_size_bytes
    })))
}

async fn check_integrity(State(db): State<Database>) -> Result<Json<Value>, AppError> {
    let result = db.integrity_check()?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "integrity": result
    })))
}

async fn list_media(
    State(db): State<Database>,
) -> Result<(HeaderMap, Json<Vec<WpMediaResponse>>), AppError> {
    let list = db.get_media_attachments()?;
    let total = list.len() as i64;
    Ok(wp_list_response(list, total, total.max(10)))
}

async fn get_media_item(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<WpMediaResponse>, AppError> {
    match db.get_media_by_id(id)? {
        Some(m) => Ok(Json(m)),
        None => Err(AppError::NotFound(format!("Media {} not found", id))),
    }
}

async fn update_media_item(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdateMediaRequest>,
) -> Result<Json<WpMediaResponse>, AppError> {
    match db.update_media_attachment(id, &payload)? {
        Some(m) => Ok(Json(m)),
        None => Err(AppError::NotFound(format!("Media {} not found", id))),
    }
}

async fn upload_media(
    _auth: AuthUser,
    State(db): State<Database>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<WpMediaResponse>), AppError> {
    let uploads_dir = Path::new("uploads");
    if !uploads_dir.exists() {
        fs::create_dir_all(uploads_dir)?;
    }

    let mut saved_media: Option<WpMediaResponse> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let file_name_raw = field.file_name().unwrap_or("upload.png").to_string();
        let content_type = field.content_type().unwrap_or("image/png").to_string();

        let clean_name = file_name_raw
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();

        let timestamp = chrono::Utc::now().timestamp();
        let final_file_name = format!("{}_{}", timestamp, clean_name);
        let dest_path = uploads_dir.join(&final_file_name);

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        let file_size = data.len() as u64;

        fs::write(&dest_path, &data)?;

        let url = format!("/uploads/{}", final_file_name);
        let title = clean_name
            .trim_end_matches(".png")
            .trim_end_matches(".jpg")
            .trim_end_matches(".jpeg")
            .trim_end_matches(".webp")
            .trim_end_matches(".svg");

        let media =
            db.create_attachment(title, &final_file_name, &content_type, file_size, &url)?;
        saved_media = Some(media);
    }

    match saved_media {
        Some(m) => Ok((StatusCode::CREATED, Json(m))),
        None => Err(AppError::BadRequest(
            "No file uploaded in form data".to_string(),
        )),
    }
}

async fn delete_media(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    match db.delete_media_attachment(id)? {
        Some(url) => {
            if let Some(file_name) = url.strip_prefix("/uploads/") {
                let path = Path::new("uploads").join(file_name);
                let _ = fs::remove_file(path);
            }
            Ok(Json(serde_json::json!({ "deleted": true, "id": id })))
        }
        None => Err(AppError::NotFound(format!("Media {} not found", id))),
    }
}

async fn get_current_user(
    auth: AuthUser,
    State(db): State<Database>,
) -> Result<Json<WpUserDetail>, AppError> {
    match db.get_user_detail(auth.user_id)? {
        Some(u) => Ok(Json(u)),
        None => Err(AppError::NotFound("User not found".to_string())),
    }
}

async fn update_current_user(
    auth: AuthUser,
    State(db): State<Database>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<WpUserDetail>, AppError> {
    match db.update_user_detail(auth.user_id, &payload)? {
        Some(u) => Ok(Json(u)),
        None => Err(AppError::NotFound("User not found".to_string())),
    }
}

async fn get_user_by_id(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<WpUserDetail>, AppError> {
    match db.get_user_detail(id)? {
        Some(u) => Ok(Json(u)),
        None => Err(AppError::NotFound(format!("User {} not found", id))),
    }
}

async fn update_user_by_id(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<WpUserDetail>, AppError> {
    match db.update_user_detail(id, &payload)? {
        Some(u) => Ok(Json(u)),
        None => Err(AppError::NotFound(format!("User {} not found", id))),
    }
}

fn normalize_import_bundle_payload(raw: Value) -> Result<ImportBundleRequest, AppError> {
    // 1. Check if this is a `/n-admin/export-json` backup or `{"options": ..., "posts": ...}` payload
    let mut normalized = raw;
    if let Some(obj) = normalized.as_object_mut() {
        // Normalize `options` -> `site_settings`
        if !obj.contains_key("site_settings") {
            if let Some(opts) = obj.get("options").and_then(|v| v.as_object()) {
                let mut ss = serde_json::Map::new();
                if let Some(t) = opts.get("blogname") {
                    ss.insert("title".to_string(), t.clone());
                }
                if let Some(d) = opts.get("blogdescription") {
                    ss.insert("description".to_string(), d.clone());
                }
                if let Some(u) = opts.get("siteurl").or_else(|| opts.get("home")) {
                    ss.insert("url".to_string(), u.clone());
                }
                if let Some(e) = opts.get("admin_email") {
                    ss.insert("email".to_string(), e.clone());
                }
                if !ss.is_empty() {
                    obj.insert("site_settings".to_string(), Value::Object(ss));
                }
            } else if let Some(settings_val) = obj.get("settings") {
                obj.insert("site_settings".to_string(), settings_val.clone());
            }
        }

        // Normalize `posts` array if items use `WpPostResponse` (`{"title": {"rendered": ...}}`) or WP DB columns (`{"post_title": ...}`)
        if let Some(posts_arr) = obj.get_mut("posts").and_then(|v| v.as_array_mut()) {
            for p in posts_arr.iter_mut() {
                if let Some(p_obj) = p.as_object_mut() {
                    if let Some(pt) = p_obj.get("post_title").and_then(|v| v.as_str()) {
                        let s = pt.to_string();
                        p_obj.entry("title".to_string()).or_insert(Value::String(s));
                    } else if let Some(t_rendered) = p_obj
                        .get("title")
                        .and_then(|v| v.get("rendered"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                    {
                        p_obj.insert("title".to_string(), Value::String(t_rendered));
                    }

                    if let Some(pc) = p_obj.get("post_content").and_then(|v| v.as_str()) {
                        let s = pc.to_string();
                        p_obj
                            .entry("content".to_string())
                            .or_insert(Value::String(s));
                    } else if let Some(c_rendered) = p_obj
                        .get("content")
                        .and_then(|v| v.get("rendered"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                    {
                        p_obj.insert("content".to_string(), Value::String(c_rendered));
                    }

                    if let Some(pe) = p_obj.get("post_excerpt").and_then(|v| v.as_str()) {
                        let s = pe.to_string();
                        p_obj
                            .entry("excerpt".to_string())
                            .or_insert(Value::String(s));
                    } else if let Some(e_rendered) = p_obj
                        .get("excerpt")
                        .and_then(|v| v.get("rendered"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                    {
                        p_obj.insert("excerpt".to_string(), Value::String(e_rendered));
                    }

                    if let Some(pn) = p_obj
                        .get("post_name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                    {
                        p_obj.entry("slug".to_string()).or_insert(Value::String(pn));
                    }
                    if let Some(ps) = p_obj
                        .get("post_status")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                    {
                        p_obj
                            .entry("status".to_string())
                            .or_insert(Value::String(ps));
                    }
                }
            }
        }
    }

    serde_json::from_value(normalized)
        .map_err(|e| AppError::BadRequest(format!("Invalid import bundle payload: {}", e)))
}

async fn import_bundle(
    _auth: AuthUser,
    State(db): State<Database>,
    Json(raw_payload): Json<Value>,
) -> Result<Json<ImportBundleResponse>, AppError> {
    let payload = normalize_import_bundle_payload(raw_payload)?;
    let res = db.import_bundle(&payload)?;
    Ok(Json(res))
}

// --- Application Passwords Endpoints ---

async fn list_app_passwords(
    auth: AuthUser,
    State(db): State<Database>,
    AxPath(user_id): AxPath<i64>,
) -> Result<Json<Vec<AppPasswordResponse>>, AppError> {
    if auth.user_id != user_id && auth.user_id != 1 {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }
    let list = db.list_application_passwords(user_id)?;
    Ok(Json(list))
}

async fn create_app_password(
    auth: AuthUser,
    State(db): State<Database>,
    AxPath(user_id): AxPath<i64>,
    Json(payload): Json<CreateAppPasswordRequest>,
) -> Result<(StatusCode, Json<AppPasswordResponse>), AppError> {
    if auth.user_id != user_id && auth.user_id != 1 {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }
    let res = db.create_application_password(user_id, &payload.name, payload.app_id.as_deref())?;
    Ok((StatusCode::CREATED, Json(res)))
}

async fn delete_app_password(
    auth: AuthUser,
    State(db): State<Database>,
    AxPath((user_id, uuid)): AxPath<(i64, String)>,
) -> Result<Json<Value>, AppError> {
    if auth.user_id != user_id && auth.user_id != 1 {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }
    if db.delete_application_password(user_id, &uuid)? {
        Ok(Json(serde_json::json!({ "deleted": true, "uuid": uuid })))
    } else {
        Err(AppError::NotFound(format!(
            "Application password {} not found",
            uuid
        )))
    }
}

// --- Nonce & JWT Auth Endpoints ---

async fn get_auth_nonce(State(db): State<Database>) -> Result<Json<NonceResponse>, AppError> {
    let secret = db.get_auth_secret()?;
    let nonce = crate::auth::create_wp_nonce("wp_rest", 1, &secret);
    Ok(Json(NonceResponse {
        nonce,
        action: "wp_rest".to_string(),
    }))
}

async fn generate_jwt_token(
    State(db): State<Database>,
    Json(payload): Json<JwtTokenRequest>,
) -> Result<Json<JwtTokenResponse>, AppError> {
    let user = match db.authenticate_credentials(&payload.username, &payload.password)? {
        Some(u) => u,
        None => {
            return Err(AppError::Unauthorized(
                "Invalid username or password".to_string(),
            ))
        }
    };

    let user_detail = db
        .get_user_detail(user.user_id)?
        .unwrap_or_else(|| WpUserDetail {
            id: user.user_id,
            username: user.username.clone(),
            display_name: user.username.clone(),
            email: "admin@example.com".to_string(),
            url: "".to_string(),
            registered: "".to_string(),
            role: "administrator".to_string(),
        });

    let secret = db.get_auth_secret()?;
    let site_url = db.get_base_url();
    let token = crate::auth::create_jwt_token(user.user_id, &user.username, &site_url, &secret, 7);

    Ok(Json(JwtTokenResponse {
        token,
        user_email: user_detail.email,
        user_nicename: user.username,
        user_display_name: user_detail.display_name,
    }))
}

async fn validate_jwt_token(
    State(db): State<Database>,
    request: Request,
) -> Result<Json<JwtValidateResponse>, AppError> {
    let (parts, _) = request.into_parts();
    let _auth = crate::auth::authenticate_request(&parts, &db).await?;
    Ok(Json(JwtValidateResponse {
        code: "jwt_auth_valid_token".to_string(),
        data: serde_json::json!({ "status": 200 }),
    }))
}

// --- Streaming WXR XML Ingestion Endpoint ---

async fn import_wxr_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    request: Request,
) -> Result<Json<WxrImportResponse>, AppError> {
    let (parts, body) = request.into_parts();
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let bytes = to_bytes(body, 500 * 1024 * 1024)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read request body: {}", e)))?;

    if content_type.starts_with("multipart/form-data") {
        let req_with_body = Request::from_parts(parts, axum::body::Body::from(bytes));
        let mut multipart = Multipart::from_request(req_with_body, &db)
            .await
            .map_err(|e| AppError::BadRequest(format!("Multipart parse error: {}", e)))?;

        if let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?
        {
            let field_bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let cursor = Cursor::new(field_bytes);
            let res = db.import_wxr(cursor)?;
            return Ok(Json(res));
        }

        Err(AppError::BadRequest(
            "No XML file found in multipart upload".to_string(),
        ))
    } else {
        let cursor = Cursor::new(bytes);
        let res = db.import_wxr(cursor)?;
        Ok(Json(res))
    }
}

// --- Visual Page Builder Layout API (Hybrid Wasm & JSON Layout Schema) ---

async fn get_layout_handler(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    let layout_opt = db.get_post_layout(id)?;
    match layout_opt {
        Some(layout_str) => {
            let val: Value = serde_json::from_str(&layout_str)
                .unwrap_or_else(|_| serde_json::json!({ "raw": layout_str }));
            Ok(Json(serde_json::json!({
                "post_id": id,
                "layout": val
            })))
        }
        None => {
            // Return clean default layout specification
            Ok(Json(serde_json::json!({
                "post_id": id,
                "layout": {
                    "version": 1,
                    "root": {
                        "id": format!("sec_{}", id),
                        "type": "Section",
                        "styles": {},
                        "children": []
                    }
                }
            })))
        }
    }
}

async fn save_layout_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<NeopressLayoutPayload>,
) -> Result<Json<Value>, AppError> {
    let layout_str = serde_json::to_string(&payload.layout)
        .map_err(|e| AppError::BadRequest(format!("Invalid layout JSON: {}", e)))?;

    let compiled_ssr = if payload.rendered_html.is_none() {
        serde_json::from_value::<neopress_ast::NeopressLayout>(payload.layout.clone())
            .ok()
            .map(|parsed| neopress_ast::render_layout_to_html(&parsed))
    } else {
        None
    };

    db.save_post_layout(
        id,
        &layout_str,
        payload.rendered_html.as_deref().or(compiled_ssr.as_deref()),
    )?;

    Ok(Json(serde_json::json!({
        "success": true,
        "post_id": id,
        "status": "saved"
    })))
}

async fn delete_layout_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    db.delete_post_layout(id)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "post_id": id,
        "status": "deleted",
        "message": "Visual layout removed. Standard text editor unlocked."
    })))
}

async fn duplicate_post_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<WpPostResponse>, AppError> {
    match db.duplicate_post(id)? {
        Some(cloned) => Ok(Json(cloned)),
        None => Err(AppError::NotFound(format!("Post {} not found", id))),
    }
}

// --- Native Subsystems: Forms & Webhooks Engine ---

async fn submit_form_handler(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    request: Request,
) -> Result<Json<Value>, AppError> {
    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, 10 * 1024 * 1024)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let payload: FormSubmitRequest = serde_json::from_slice(&bytes)
        .map_err(|e| AppError::BadRequest(format!("Invalid form submission: {}", e)))?;

    // Anti-spam honeypot verification
    if let Some(obj) = payload.data.as_object() {
        if let Some(honeypot) = obj.get("_np_hp") {
            if let Some(s) = honeypot.as_str() {
                if !s.is_empty() {
                    return Err(AppError::BadRequest("Spam detected".to_string()));
                }
            }
        }
    }

    let ip = parts
        .headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("127.0.0.1");

    let ua = parts
        .headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(ip.as_bytes());
    let ip_hash = hex::encode(hasher.finalize());

    let data_str =
        serde_json::to_string(&payload.data).map_err(|e| AppError::BadRequest(e.to_string()))?;

    let entry_id = db.submit_form_entry(id, &data_str, &ip_hash, ua)?;

    // Trigger notification email if configured on the form
    if let Ok(Some(form_rec)) = db.get_form(id) {
        let settings: FormSettings =
            serde_json::from_str(&form_rec.settings_json).unwrap_or_default();
        if !settings.recipient_email.trim().is_empty() {
            let mut summary_lines = Vec::new();
            if let Some(obj) = payload.data.as_object() {
                for (k, v) in obj {
                    if k != "_np_hp" {
                        summary_lines.push(format!(
                            "<li><strong>{}</strong>: {}</li>",
                            k,
                            v.as_str().unwrap_or(&v.to_string())
                        ));
                    }
                }
            }
            let email_msg = crate::email::EmailMessage {
                to: settings.recipient_email.trim().to_string(),
                subject: format!("[Neopress Form] New submission: {}", form_rec.title),
                body_html: format!(
                    "<h3>New submission for {}</h3><ul>{}</ul>",
                    form_rec.title,
                    summary_lines.join("")
                ),
                body_text: format!("New submission for {}: {}", form_rec.title, data_str),
            };
            let _ = crate::email::send_transactional_email(&db, &email_msg).await;
        }
        return Ok(Json(serde_json::json!({
            "success": true,
            "entry_id": entry_id,
            "message": settings.success_message,
            "webhook_url": settings.webhook_url
        })));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "entry_id": entry_id,
        "message": "Form submission received successfully."
    })))
}

// --- Native Subsystems: Cookieless Daily Salted Analytics ---

async fn record_analytics_event_handler(
    State(db): State<Database>,
    request: Request,
) -> Result<Json<Value>, AppError> {
    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, 64 * 1024)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let payload: AnalyticsEventRequest = serde_json::from_slice(&bytes)
        .map_err(|e| AppError::BadRequest(format!("Invalid analytics payload: {}", e)))?;

    let client_ip = parts
        .headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("127.0.0.1");

    let ua = parts
        .headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let country = parts
        .headers
        .get("cf-ipcountry")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("XX");

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(client_ip.as_bytes());
    hasher.update(ua.as_bytes());
    hasher.update(today.as_bytes());
    let visitor_hash = hex::encode(hasher.finalize());

    let referrer = payload.referrer.unwrap_or_default();
    let device_type = payload.device_type.unwrap_or_else(|| "desktop".to_string());

    db.record_analytics_event(
        &visitor_hash,
        &payload.page_url,
        &referrer,
        &device_type,
        country,
        payload.utm_source.as_deref(),
        payload.utm_medium.as_deref(),
        payload.utm_campaign.as_deref(),
    )?;

    Ok(Json(serde_json::json!({ "recorded": true })))
}

async fn get_analytics_summary_handler(
    _auth: AuthUser,
    State(db): State<Database>,
) -> Result<Json<AnalyticsSummary>, AppError> {
    let summary = db.get_analytics_summary()?;
    Ok(Json(summary))
}

// --- Native Subsystems: Transactional Email Handler ---

async fn send_email_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    Json(payload): Json<crate::email::EmailMessage>,
) -> Result<Json<Value>, AppError> {
    crate::email::send_transactional_email(&db, &payload).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Email dispatched and logged successfully."
    })))
}
