//! Admin Settings, Appearance, Export/Import, and Action API Handlers.

#![allow(unused_imports)]
use crate::auth::OptionalAuthUser;
use crate::db::Database;
use crate::models;
use crate::templates;
use axum::{
    extract::{Form, Path as AxPath, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use super::*;

pub async fn handle_api_update_comment(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    axum::Json(payload): axum::Json<UpdateCommentStatusPayload>,
) -> impl IntoResponse {
    match db.update_comment_status(id, &payload.status) {
        Ok(true) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "status": payload.status })),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "code": "not_found", "message": "Comment not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "code": "error", "message": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn handle_api_delete_comment(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> impl IntoResponse {
    match db.delete_comment(id) {
        Ok(true) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "deleted": true, "previous": { "id": id } })),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "code": "not_found", "message": "Comment not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "code": "error", "message": e.to_string() })),
        )
            .into_response(),
    }
}

// 4. Appearance Submenus
pub async fn handle_admin_customize(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_customize_view(&settings))
}

pub async fn handle_admin_widgets(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_widgets_view(&settings))
}

pub async fn handle_admin_nav_menus(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let pages = db
        .get_posts("page", Some("any"), None, 50)
        .unwrap_or_default();
    Html(templates::render_wp_nav_menus_view(&settings, &pages))
}

pub async fn handle_admin_theme_editor(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_theme_editor_view(&settings))
}

// 5. Plugins Submenus
pub async fn handle_admin_plugin_install(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_plugin_install_view(&settings))
}

pub async fn handle_admin_plugin_editor(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_plugin_editor_view(&settings))
}

#[derive(Debug, Deserialize)]
pub struct ThemeSwitchPayload {
    pub theme: Option<String>,
    pub active_theme: Option<String>,
}

pub async fn handle_api_theme_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<ThemeSwitchPayload>,
) -> impl IntoResponse {
    let theme = payload
        .theme
        .or(payload.active_theme)
        .unwrap_or_else(|| "Neopress Modern".to_string());
    match db.set_option("neopress_active_theme", &theme) {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "theme": theme })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct CustomizerSavePayload {
    pub title: Option<String>,
    pub description: Option<String>,
    pub brand_color: Option<String>,
    pub typography_scale: Option<String>,
    pub custom_css: Option<String>,
}

pub async fn handle_api_customizer_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<CustomizerSavePayload>,
) -> impl IntoResponse {
    if let Some(ref t) = payload.title {
        let _ = db.set_option("blogname", t);
    }
    if let Some(ref d) = payload.description {
        let _ = db.set_option("blogdescription", d);
    }
    if let Some(ref c) = payload.brand_color {
        let _ = db.set_option("neopress_brand_color", c);
    }
    if let Some(ref s) = payload.typography_scale {
        let _ = db.set_option("neopress_typography_scale", s);
    }
    if let Some(ref css) = payload.custom_css {
        let _ = db.set_option("neopress_custom_css", css);
    }
    (
        StatusCode::OK,
        axum::Json(serde_json::json!({ "success": true })),
    )
}

pub async fn handle_api_widgets_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let widgets_val = payload.get("widgets").cloned().unwrap_or(payload);
    let json_str = serde_json::to_string(&widgets_val).unwrap_or_default();
    match db.set_option("neopress_widgets", &json_str) {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "widgets": widgets_val })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn handle_api_menus_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let items_val = payload
        .get("items")
        .or_else(|| payload.get("nav_menu_items"))
        .cloned()
        .unwrap_or(payload);
    let items: Vec<models::NavMenuItemDto> = serde_json::from_value(items_val).unwrap_or_default();
    let json_str = serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string());
    match db.set_option("neopress_nav_menu_items", &json_str) {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "items": items })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct PluginTogglePayload {
    pub plugin: Option<String>,
    pub active: Option<bool>,
    pub active_plugins: Option<Vec<String>>,
}

pub async fn handle_api_plugins_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<PluginTogglePayload>,
) -> impl IntoResponse {
    if let Some(ref slug) = payload.plugin {
        let _ = db.toggle_plugin_status(slug, payload.active);
    }
    if let Some(ref list) = payload.active_plugins {
        if let Ok(all) = db.get_all_installed_plugins() {
            for p in all {
                let should_be_active = list
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(&p.manifest.slug));
                if p.active != should_be_active {
                    let _ = db.toggle_plugin_status(&p.manifest.slug, Some(should_be_active));
                }
            }
        }
    }

    let active_plugins: Vec<String> = db
        .get_all_installed_plugins()
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.active)
        .map(|p| p.manifest.slug)
        .collect();

    let json_str = serde_json::to_string(&active_plugins).unwrap_or_else(|_| "[]".to_string());
    let _ = db.set_option("neopress_active_plugins", &json_str);
    (
        StatusCode::OK,
        axum::Json(serde_json::json!({ "success": true, "active_plugins": active_plugins })),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct ThemeFileQuery {
    pub file: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ThemeFileSavePayload {
    pub file: String,
    pub content: String,
}

fn sanitize_theme_file_key(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect();
    if cleaned.is_empty() {
        "style.css".to_string()
    } else {
        cleaned
    }
}

pub async fn handle_api_theme_file_get(
    State(db): State<Database>,
    Query(params): Query<ThemeFileQuery>,
) -> impl IntoResponse {
    let file = sanitize_theme_file_key(params.file.as_deref().unwrap_or("style.css"));
    let key = format!("neopress_file_{}", file);
    let content = db.get_option(&key).ok().flatten().unwrap_or_default();
    (
        StatusCode::OK,
        axum::Json(serde_json::json!({ "file": file, "content": content })),
    )
}

pub async fn handle_api_theme_file_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<ThemeFileSavePayload>,
) -> impl IntoResponse {
    let file = sanitize_theme_file_key(&payload.file);
    let key = format!("neopress_file_{}", file);
    match db.set_option(&key, &payload.content) {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "file": file })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// 6. Tools Submenus
pub async fn handle_admin_import(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_import_view(&settings))
}

pub async fn handle_admin_export(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_export_view(&settings))
}

pub async fn handle_admin_export_wxr_download(State(db): State<Database>) -> impl IntoResponse {
    let xml = db
        .export_wxr()
        .unwrap_or_else(|e| format!("<!-- Export error: {} -->", e));
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/xml; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"neopress.wxr.xml\"".parse().unwrap(),
    );
    (StatusCode::OK, headers, xml)
}

pub async fn handle_admin_export_json_download(State(db): State<Database>) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let posts = db
        .get_posts("any", Some("any"), None, 500)
        .unwrap_or_default();
    let users = db.get_all_users().unwrap_or_default();
    let comments = db.get_comments(None, 500).unwrap_or_default();
    let backup = serde_json::json!({
        "generator": "Neopress Embedded Rusqlite Backup",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "settings": settings,
        "posts": posts,
        "users": users,
        "comments": comments
    });
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/json; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"neopress-backup.json\""
            .parse()
            .unwrap(),
    );
    (StatusCode::OK, headers, backup.to_string())
}

pub async fn handle_admin_site_health(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let stats = db.get_stats().unwrap_or_default();
    Html(templates::render_wp_site_health_view(&settings, &stats))
}

pub async fn handle_admin_export_personal_data(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_export_personal_data_view(&settings))
}

pub async fn handle_admin_erase_personal_data(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_erase_personal_data_view(&settings))
}

pub async fn handle_api_export_personal_data(
    State(db): State<Database>,
    Query(params): Query<ExportPersonalDataQuery>,
) -> impl IntoResponse {
    let identifier = params.user.unwrap_or_default();
    match db.export_personal_data(&identifier) {
        Ok(val) => (StatusCode::OK, axum::Json(val)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn handle_api_erase_personal_data(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<ErasePersonalDataPayload>,
) -> impl IntoResponse {
    match db.erase_personal_data(&payload.identifier) {
        Ok(count) => (StatusCode::OK, axum::Json(serde_json::json!({
            "success": true,
            "items_removed": count,
            "message": format!("Personal data anonymized and scrubbed across {} comments in SQLite", count)
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// 7. Settings Submenus
pub async fn handle_admin_options_writing(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let writing = db
        .get_writing_settings()
        .unwrap_or_else(|_| models::WritingSettings {
            default_category: 1,
            default_post_format: "standard".to_string(),
            use_smilies: true,
            default_pingback_flag: true,
            mailserver_url: "mail.example.com".to_string(),
            mailserver_login: "login@example.com".to_string(),
            mailserver_pass: "".to_string(),
            mailserver_port: 110,
        });
    let categories = db.get_terms("category").unwrap_or_default();
    Html(templates::render_wp_settings_writing_view(
        &settings,
        &writing,
        &categories,
    ))
}

pub async fn handle_api_options_writing_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::WritingSettings>,
) -> impl IntoResponse {
    match db.update_writing_settings(&payload) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn handle_admin_options_reading(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let reading = db
        .get_reading_settings()
        .unwrap_or_else(|_| models::ReadingSettings {
            show_on_front: "posts".to_string(),
            page_on_front: 0,
            page_for_posts: 0,
            posts_per_page: 10,
            posts_per_rss: 10,
            blog_public: true,
        });
    let pages = db
        .get_posts("page", Some("any"), None, 50)
        .unwrap_or_default();
    Html(templates::render_wp_settings_reading_view(
        &settings, &reading, &pages,
    ))
}

pub async fn handle_api_options_reading_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::ReadingSettings>,
) -> impl IntoResponse {
    match db.update_reading_settings(&payload) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn handle_admin_options_discussion(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let discussion = db
        .get_discussion_settings()
        .unwrap_or_else(|_| models::DiscussionSettings {
            default_ping_status: "open".to_string(),
            default_comment_status: "open".to_string(),
            require_name_email: true,
            comment_registration: false,
            close_comments_for_old_posts: false,
            close_comments_days_old: 14,
            thread_comments: true,
            thread_comments_depth: 5,
            comment_moderation: false,
            comment_previously_approved: true,
            show_avatars: true,
        });
    Html(templates::render_wp_settings_discussion_view(
        &settings,
        &discussion,
    ))
}

pub async fn handle_api_options_discussion_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::DiscussionSettings>,
) -> impl IntoResponse {
    match db.update_discussion_settings(&payload) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn handle_admin_options_media(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let media = db.get_media_settings().unwrap_or(models::MediaSettings {
        thumbnail_size_w: 150,
        thumbnail_size_h: 150,
        thumbnail_crop: true,
        medium_size_w: 300,
        medium_size_h: 300,
        large_size_w: 1024,
        large_size_h: 1024,
        uploads_use_yearmonth_folders: true,
    });
    Html(templates::render_wp_settings_media_view(&settings, &media))
}

pub async fn handle_api_options_media_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::MediaSettings>,
) -> impl IntoResponse {
    match db.update_media_settings(&payload) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn handle_admin_options_permalink(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let permalinks = db
        .get_permalink_settings()
        .unwrap_or_else(|_| models::PermalinkSettings {
            permalink_structure: "/%postname%/".to_string(),
            category_base: "".to_string(),
            tag_base: "".to_string(),
        });
    Html(templates::render_wp_settings_permalink_view(
        &settings,
        &permalinks,
    ))
}

pub async fn handle_api_options_permalink_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::PermalinkSettings>,
) -> impl IntoResponse {
    match db.update_permalink_settings(&payload) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn handle_admin_options_privacy(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let privacy = db
        .get_privacy_settings()
        .unwrap_or(models::PrivacySettings {
            wp_page_for_privacy_policy: 0,
        });
    let pages = db
        .get_posts("page", Some("any"), None, 50)
        .unwrap_or_default();
    Html(templates::render_wp_settings_privacy_view(
        &settings, &privacy, &pages,
    ))
}

pub async fn handle_api_options_privacy_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::PrivacySettings>,
) -> impl IntoResponse {
    match db.update_privacy_settings(&payload) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
