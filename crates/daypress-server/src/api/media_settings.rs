//! WordPress REST API v2 Settings, Media Upload/CRUD, and Database Diagnostics.

#![allow(unused_imports)]
use super::*;
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

pub(crate) async fn get_settings(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
) -> Result<Json<DaypressSettingsResponse>, AppError> {
    let settings = db.get_settings()?;
    Ok(Json(settings))
}

pub(crate) async fn update_settings(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(payload): Json<UpdateSettingsRequest>,
) -> Result<Json<DaypressSettingsResponse>, AppError> {
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
        let tz_trimmed = tz.trim();
        let tz_clean = if tz_trimmed.is_empty() {
            "UTC"
        } else {
            tz_trimmed
        };
        db.set_option("timezone_string", tz_clean)?;
        let gmt_offset =
            crate::adapters::database::scheduling::timezone_to_gmt_offset_hours(tz_clean);
        db.set_option("gmt_offset", &gmt_offset.to_string())?;
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
        db.set_option("daypress_active_theme", &theme)?;
    }
    if let Some(color) = payload.brand_color {
        db.set_option("daypress_brand_color", &color)?;
    }
    if let Some(scale) = payload.typography_scale {
        db.set_option("daypress_typography_scale", &scale)?;
    }
    if let Some(css) = payload.custom_css {
        db.set_option("daypress_custom_css", &css)?;
    }
    if let Some(plugins) = payload.active_plugins {
        if let Ok(json) = serde_json::to_string(&plugins) {
            db.set_option("daypress_active_plugins", &json)?;
        }
    }
    if let Some(menus) = payload.nav_menu_items {
        if let Ok(json) = serde_json::to_string(&menus) {
            db.set_option("daypress_nav_menu_items", &json)?;
        }
    }
    if let Some(widgets) = payload.widgets {
        if let Ok(json) = serde_json::to_string(&widgets) {
            db.set_option("daypress_widgets", &json)?;
        }
    }

    let updated = db.get_settings()?;
    Ok(Json(updated))
}

pub(crate) async fn get_db_stats(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
) -> Result<Json<DbStats>, AppError> {
    let stats = db.get_stats()?;
    Ok(Json(stats))
}

pub(crate) async fn vacuum_db(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
) -> Result<Json<Value>, AppError> {
    db.vacuum()?;
    let stats = db.get_stats()?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "message": "SQLite VACUUM completed successfully",
        "new_size_bytes": stats.file_size_bytes
    })))
}

pub(crate) async fn check_integrity(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
) -> Result<Json<Value>, AppError> {
    let result = db.integrity_check()?;
    Ok(Json(serde_json::json!({
        "status": "success",
        "integrity": result
    })))
}

pub(crate) async fn list_media(
    State(db): State<Database>,
) -> Result<(HeaderMap, Json<Vec<DaypressMediaResponse>>), AppError> {
    let list = db.get_media_attachments()?;
    let total = list.len() as i64;
    Ok(rest_list_response(list, total, total.max(10)))
}

pub(crate) async fn get_media_item(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<DaypressMediaResponse>, AppError> {
    match db.get_media_by_id(id)? {
        Some(m) => Ok(Json(m)),
        None => Err(AppError::NotFound(format!("Media {} not found", id))),
    }
}

pub(crate) async fn update_media_item(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdateMediaRequest>,
) -> Result<Json<DaypressMediaResponse>, AppError> {
    if !user.can_upload_files() {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to edit media items.".to_string(),
        ));
    }
    match db.update_media_attachment(id, &payload)? {
        Some(m) => Ok(Json(m)),
        None => Err(AppError::NotFound(format!("Media {} not found", id))),
    }
}

pub(crate) async fn upload_media(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<DaypressMediaResponse>), AppError> {
    if !user.can_upload_files() {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to upload files.".to_string(),
        ));
    }
    let uploads_dir = Path::new("uploads");
    if !uploads_dir.exists() {
        fs::create_dir_all(uploads_dir)?;
    }

    let mut saved_media: Option<DaypressMediaResponse> = None;
    let mut target_folder_id: Option<i64> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        if field.name() == Some("folder_id") && field.file_name().is_none() {
            if let Ok(txt) = field.text().await {
                if let Ok(fid) = txt.trim().parse::<i64>() {
                    if fid > 0 {
                        target_folder_id = Some(fid);
                    }
                }
            }
            continue;
        }

        let file_name_raw = field.file_name().unwrap_or("upload.png").to_string();
        let content_type_raw = field.content_type().unwrap_or("image/png").to_string();

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

        let lower_name = clean_name.to_ascii_lowercase();
        let lower_ct = content_type_raw.trim().to_ascii_lowercase();
        let is_dangerous_ext = [
            ".html", ".htm", ".xhtml", ".shtml", ".js", ".mjs", ".cjs", ".php", ".phtml", ".phar",
            ".sh", ".bat", ".cmd", ".exe", ".dll", ".msi", ".ps1", ".vbs", ".jsp", ".asp", ".aspx",
        ]
        .iter()
        .any(|ext| lower_name.ends_with(ext));
        let is_dangerous_mime = matches!(
            lower_ct.as_str(),
            "text/html"
                | "application/xhtml+xml"
                | "application/javascript"
                | "text/javascript"
                | "application/x-javascript"
                | "application/x-httpd-php"
                | "application/x-sh"
                | "application/x-msdownload"
        );
        if is_dangerous_ext || is_dangerous_mime {
            return Err(AppError::BadRequest(format!(
                "Uploading executable or HTML/script files ('{}') is not permitted for security reasons.",
                clean_name
            )));
        }

        let is_svg =
            lower_name.ends_with(".svg") || content_type_raw.eq_ignore_ascii_case("image/svg+xml");
        let content_type = if is_svg {
            "image/svg+xml".to_string()
        } else {
            content_type_raw
        };

        let timestamp = chrono::Utc::now().timestamp();
        let final_file_name = format!("{}_{}", timestamp, clean_name);
        let dest_path = uploads_dir.join(&final_file_name);

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let write_bytes: Vec<u8> = if is_svg {
            crate::media::sanitize_svg_bytes(&data)?
        } else {
            data.to_vec()
        };
        let file_size = write_bytes.len() as u64;

        fs::write(&dest_path, &write_bytes)?;

        let url = format!("/uploads/{}", final_file_name);
        let title = clean_name
            .trim_end_matches(".png")
            .trim_end_matches(".jpg")
            .trim_end_matches(".jpeg")
            .trim_end_matches(".webp")
            .trim_end_matches(".avif")
            .trim_end_matches(".svg");

        let media =
            db.create_attachment(title, &final_file_name, &content_type, file_size, &url)?;
        crate::storage_s3::maybe_offload_media_file(
            &db,
            &final_file_name,
            &write_bytes,
            &content_type,
        );
        saved_media = Some(media);
    }

    if let (Some(ref m), Some(fid)) = (&saved_media, target_folder_id) {
        let req = UpdateMediaRequest {
            title: None,
            alt_text: None,
            caption: None,
            description: None,
            folder_id: Some(fid),
            tags: None,
            focal_x: None,
            focal_y: None,
        };
        if let Ok(Some(updated)) = db.update_media_attachment(m.id, &req) {
            saved_media = Some(updated);
        }
    }

    match saved_media {
        Some(m) => Ok((StatusCode::CREATED, Json(m))),
        None => Err(AppError::BadRequest(
            "No file uploaded in form data".to_string(),
        )),
    }
}

pub(crate) async fn delete_media(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    if !user.can_upload_files() {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to delete media items.".to_string(),
        ));
    }
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

#[derive(Debug, Deserialize)]
pub struct CreateMediaFolderRequest {
    pub name: String,
    #[serde(default)]
    pub parent_id: i64,
}

#[derive(Debug, Deserialize)]
pub struct ReplaceMediaJsonRequest {
    pub base64_data: String,
    #[serde(default)]
    pub mime_type: Option<String>,
}

pub(crate) async fn list_media_folders_handler(
    State(db): State<Database>,
) -> Result<Json<Vec<MediaFolderDto>>, AppError> {
    let folders = db.list_media_folders()?;
    Ok(Json(folders))
}

pub(crate) async fn create_media_folder_handler(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    Json(payload): Json<CreateMediaFolderRequest>,
) -> Result<(StatusCode, Json<MediaFolderDto>), AppError> {
    if !user.can_upload_files() {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to manage media folders.".to_string(),
        ));
    }
    let clean_name = payload.name.trim();
    if clean_name.is_empty() {
        return Err(AppError::BadRequest(
            "Folder name cannot be empty".to_string(),
        ));
    }
    let folder = db.create_media_folder(clean_name, payload.parent_id)?;
    Ok((StatusCode::CREATED, Json(folder)))
}

pub(crate) async fn delete_media_folder_handler(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    if !user.can_upload_files() {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to manage media folders.".to_string(),
        ));
    }
    if db.delete_media_folder(id)? {
        Ok(Json(serde_json::json!({ "deleted": true, "id": id })))
    } else {
        Err(AppError::NotFound(format!("Media folder {} not found", id)))
    }
}

pub(crate) async fn scan_unused_media_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
) -> Result<Json<Vec<DaypressMediaResponse>>, AppError> {
    let unused = db.scan_unused_media_attachments()?;
    Ok(Json(unused))
}

pub(crate) async fn replace_media_handler(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    req: Request,
) -> Result<Json<DaypressMediaResponse>, AppError> {
    if !user.can_upload_files() {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to replace media files.".to_string(),
        ));
    }

    let content_type_hdr = req
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_ascii_lowercase();

    let (raw_bytes, new_mime) = if content_type_hdr.starts_with("multipart/form-data") {
        let mut multipart = Multipart::from_request(req, &db)
            .await
            .map_err(|e| AppError::BadRequest(format!("Invalid multipart payload: {}", e)))?;

        let mut file_bytes: Option<Vec<u8>> = None;
        let mut detected_mime: Option<String> = None;

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?
        {
            let fname = field.file_name().unwrap_or("").to_ascii_lowercase();
            let fmime = field.content_type().map(|s| s.to_string());
            let bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
            if !bytes.is_empty() {
                if fname.ends_with(".svg") {
                    detected_mime = Some("image/svg+xml".to_string());
                } else {
                    detected_mime = fmime;
                }
                file_bytes = Some(bytes.to_vec());
                break;
            }
        }

        let bytes = file_bytes.ok_or_else(|| {
            AppError::BadRequest("No replacement file provided in form data".to_string())
        })?;
        (bytes, detected_mime)
    } else {
        let body_bytes = to_bytes(req.into_body(), 64 * 1024 * 1024)
            .await
            .map_err(|e| AppError::BadRequest(format!("Failed to read request body: {}", e)))?;
        let payload: ReplaceMediaJsonRequest = serde_json::from_slice(&body_bytes)
            .map_err(|e| AppError::BadRequest(format!("Invalid JSON payload: {}", e)))?;

        let b64_clean = payload
            .base64_data
            .trim()
            .split_once(";base64,")
            .map(|(_, data)| data)
            .unwrap_or_else(|| payload.base64_data.trim());

        use base64::Engine as _;
        let decoded = base64::engine::general_purpose::STANDARD
            .decode(b64_clean)
            .map_err(|e| AppError::BadRequest(format!("Invalid base64_data: {}", e)))?;
        if decoded.is_empty() {
            return Err(AppError::BadRequest(
                "Replacement file data cannot be empty".to_string(),
            ));
        }
        (decoded, payload.mime_type)
    };

    // Check if target attachment or replacement payload is SVG and validate upfront so bad SVGs return 400 BadRequest
    let existing = db
        .get_media_by_id(id)?
        .ok_or_else(|| AppError::NotFound(format!("Media {} not found", id)))?;
    let is_svg = existing.mime_type.eq_ignore_ascii_case("image/svg+xml")
        || existing.source_url.to_ascii_lowercase().ends_with(".svg")
        || new_mime
            .as_deref()
            .map(|m| m.eq_ignore_ascii_case("image/svg+xml"))
            .unwrap_or(false);
    let sanitized_bytes = if is_svg {
        crate::media::sanitize_svg_bytes(&raw_bytes)?
    } else {
        raw_bytes
    };

    match db.replace_media_attachment(id, &sanitized_bytes, new_mime.as_deref())? {
        Some(updated) => {
            if let Some(rel_name) = updated.source_url.strip_prefix("/uploads/") {
                crate::storage_s3::maybe_offload_media_file(
                    &db,
                    rel_name,
                    &sanitized_bytes,
                    &updated.mime_type,
                );
            }
            Ok(Json(updated))
        }
        None => Err(AppError::NotFound(format!("Media {} not found", id))),
    }
}

/// Sub-router for Media DAM endpoints mounted under `/wp-json/daypress/v1`.
pub fn media_dam_routes() -> Router<Database> {
    Router::new()
        .route(
            "/media/folders",
            get(list_media_folders_handler).post(create_media_folder_handler),
        )
        .route(
            "/media/folders/{id}",
            axum::routing::delete(delete_media_folder_handler),
        )
        .route("/media/unused", get(scan_unused_media_handler))
        .route(
            "/media/{id}/replace",
            axum::routing::post(replace_media_handler),
        )
}
