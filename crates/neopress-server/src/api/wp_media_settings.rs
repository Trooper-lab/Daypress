//! WordPress REST API v2 Settings, Media Upload/CRUD, and Database Diagnostics.

#![allow(unused_imports)]
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
use super::*;


pub(crate) async fn get_settings(State(db): State<Database>) -> Result<Json<WpSettingsResponse>, AppError> {
    let settings = db.get_settings()?;
    Ok(Json(settings))
}

pub(crate) async fn update_settings(
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

pub(crate) async fn get_db_stats(State(db): State<Database>) -> Result<Json<DbStats>, AppError> {
    let stats = db.get_stats()?;
    Ok(Json(stats))
}

pub(crate) async fn vacuum_db(_auth: AuthUser, State(db): State<Database>) -> Result<Json<Value>, AppError> {
    db.vacuum()?;
    let stats = db.get_stats()?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "SQLite VACUUM completed successfully",
        "new_size_bytes": stats.file_size_bytes
    })))
}

pub(crate) async fn check_integrity(State(db): State<Database>) -> Result<Json<Value>, AppError> {
    let result = db.integrity_check()?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "integrity": result
    })))
}

pub(crate) async fn list_media(
    State(db): State<Database>,
) -> Result<(HeaderMap, Json<Vec<WpMediaResponse>>), AppError> {
    let list = db.get_media_attachments()?;
    let total = list.len() as i64;
    Ok(wp_list_response(list, total, total.max(10)))
}

pub(crate) async fn get_media_item(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<WpMediaResponse>, AppError> {
    match db.get_media_by_id(id)? {
        Some(m) => Ok(Json(m)),
        None => Err(AppError::NotFound(format!("Media {} not found", id))),
    }
}

pub(crate) async fn update_media_item(
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

pub(crate) async fn upload_media(
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

pub(crate) async fn delete_media(
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
