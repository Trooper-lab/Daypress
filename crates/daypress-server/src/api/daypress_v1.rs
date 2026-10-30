//! Daypress V1 Native REST Extensions (`/wp-json/daypress/v1/*`: 1-Click Import, WXR, Visual Layout AST, Forms, Analytics, Email).

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

pub(crate) fn normalize_import_bundle_payload(raw: Value) -> Result<ImportBundleRequest, AppError> {
    // 0. Unwrap `{ "success": true, "data": { ... } }` or `{ "data": { ... } }` if present
    let mut normalized = raw;
    if let Some(obj) = normalized.as_object() {
        if obj.contains_key("data") {
            if let Some(inner) = obj.get("data") {
                if inner.is_object()
                    && (inner.get("posts").is_some()
                        || inner.get("site_settings").is_some()
                        || inner.get("pages").is_some())
                {
                    normalized = inner.clone();
                }
            }
        }
    }

    // 1. Check if this is a `/d-admin/export-json` backup or `{"options": ..., "posts": ...}` payload
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

        // Normalize `posts` array if items use `DaypressPostResponse` (`{"title": {"rendered": ...}}`) or WP DB columns (`{"post_title": ...}`)
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

pub(crate) async fn import_bundle(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    body: axum::body::Bytes,
) -> Result<Json<ImportBundleResponse>, AppError> {
    let raw_payload: Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::BadRequest(format!("Invalid JSON payload: {}", e)))?;
    let payload = normalize_import_bundle_payload(raw_payload)?;
    let res = db.import_bundle(&payload)?;
    Ok(Json(res))
}

// --- Streaming WXR XML Ingestion Endpoint ---

pub(crate) async fn import_wxr_handler(
    _auth: crate::auth::AdminUser,
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

pub(crate) async fn get_layout_handler(
    crate::auth::OptionalAuthUser(user): crate::auth::OptionalAuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    // Layouts of unpublished posts are only readable by content editors.
    if let Some(post) = db.get_post_by_id(id)? {
        if !crate::auth::can_view_status(&post.status, user.as_ref()) {
            return Err(AppError::NotFound(format!("Post {} not found", id)));
        }
    }
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
            if let Ok(Some(post)) = db.get_post_by_id(id) {
                if let Some(synth) = crate::templates::admin::builder::synthesize_ast_from_post_html(
                    id,
                    &post.content.rendered,
                ) {
                    return Ok(Json(serde_json::json!({
                        "post_id": id,
                        "layout": synth
                    })));
                }
            }
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

pub(crate) async fn save_layout_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<DaypressLayoutPayload>,
) -> Result<Json<Value>, AppError> {
    let layout_str = serde_json::to_string(&payload.layout)
        .map_err(|e| AppError::BadRequest(format!("Invalid layout JSON: {}", e)))?;

    let compiled_ssr = if payload.rendered_html.is_none() {
        serde_json::from_value::<daypress_ast::DaypressLayout>(payload.layout.clone())
            .ok()
            .map(|parsed| daypress_ast::render_layout_to_html(&parsed))
    } else {
        None
    };

    // Persist converged global_styles into options (`daypress_elementor_kit_json` & `daypress_elementor_kit_css`)
    if let Some(gs) = payload.layout.get("global_styles") {
        if gs.is_object() {
            if let Ok(kit) = serde_json::from_value::<daypress_ast::DaystudioKit>(gs.clone()) {
                let kit_css = kit.compile_to_css();
                let _ = db.set_option("daypress_elementor_kit_css", &kit_css);
                if let Ok(kit_json) = serde_json::to_string(&kit) {
                    let _ = db.set_option("daypress_elementor_kit_json", &kit_json);
                }
                if let Some(prim) = kit.system_colors.iter().find(|c| c.id == "primary") {
                    if !prim.color.trim().is_empty() {
                        let _ = db.set_option("daypress_brand_color", prim.color.trim());
                    }
                }
            }
        }
    }

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

pub(crate) async fn delete_layout_handler(
    _auth: crate::auth::EditorUser,
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

pub(crate) async fn duplicate_post_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<DaypressPostResponse>, AppError> {
    match db.duplicate_post(id)? {
        Some(cloned) => Ok(Json(cloned)),
        None => Err(AppError::NotFound(format!("Post {} not found", id))),
    }
}

pub(crate) async fn restore_revision_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(rev_id): AxPath<i64>,
) -> Result<Json<DaypressPostResponse>, AppError> {
    match db.restore_revision_by_id(rev_id)? {
        Some(restored) => Ok(Json(restored)),
        None => Err(AppError::NotFound(format!("Revision {} not found", rev_id))),
    }
}

pub(crate) async fn clean_revisions_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    payload: Option<Json<CleanRevisionsRequest>>,
) -> Result<Json<CleanRevisionsResponse>, AppError> {
    let req = payload.map(|Json(p)| p).unwrap_or_default();
    let res = db.clean_post_revisions(req.keep_per_post, req.older_than_days)?;
    Ok(Json(res))
}

#[allow(dead_code)]
async fn _unused_duplicate_post_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<DaypressPostResponse>, AppError> {
    match db.duplicate_post(id)? {
        Some(cloned) => Ok(Json(cloned)),
        None => Err(AppError::NotFound(format!("Post {} not found", id))),
    }
}

// --- Native Subsystems: Forms & Webhooks Engine ---

pub(crate) async fn submit_form_handler(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    request: Request,
) -> Result<Json<Value>, AppError> {
    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, 10 * 1024 * 1024)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let raw_val: Value = serde_json::from_slice(&bytes)
        .map_err(|e| AppError::BadRequest(format!("Invalid form submission: {}", e)))?;
    let payload_data = match raw_val.get("data") {
        Some(d) if d.is_object() => d.clone(),
        _ => raw_val,
    };
    let payload = FormSubmitRequest {
        data: payload_data,
        antispam_token: None,
    };

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
                        // Visitor input: escape before it goes into the notification HTML.
                        summary_lines.push(format!(
                            "<li><strong>{}</strong>: {}</li>",
                            crate::templates::utils::escape_html(k),
                            crate::templates::utils::escape_html(
                                v.as_str().unwrap_or(&v.to_string())
                            )
                        ));
                    }
                }
            }
            let email_msg = crate::email::EmailMessage {
                to: settings.recipient_email.trim().to_string(),
                subject: format!("[Daypress Form] New submission: {}", form_rec.title),
                body_html: format!(
                    "<h3>New submission for {}</h3><ul>{}</ul>",
                    crate::templates::utils::escape_html(&form_rec.title),
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

pub(crate) async fn record_analytics_event_handler(
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

pub(crate) async fn get_analytics_summary_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
) -> Result<Json<AnalyticsSummary>, AppError> {
    let summary = db.get_analytics_summary()?;
    Ok(Json(summary))
}

// --- Native Subsystems: Transactional Email Handler ---

pub(crate) async fn send_email_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(payload): Json<crate::email::EmailMessage>,
) -> Result<Json<Value>, AppError> {
    crate::email::send_transactional_email(&db, &payload).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "Email dispatched and logged successfully."
    })))
}

// --- Native Subsystems: Self-Updater & Release Lifecycle ---

pub(crate) async fn get_update_status_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
) -> Result<Json<crate::updater::UpdateStatusResponse>, AppError> {
    Ok(Json(crate::updater::get_update_status(&db)))
}

pub(crate) async fn check_update_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
) -> Result<Json<crate::updater::UpdateStatusResponse>, AppError> {
    crate::updater::check_for_updates_now(&db).await?;
    Ok(Json(crate::updater::get_update_status(&db)))
}

pub(crate) async fn save_update_settings_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(payload): Json<crate::updater::UpdateSettings>,
) -> Result<Json<crate::updater::UpdateStatusResponse>, AppError> {
    let mut cleaned = payload;
    if cleaned.channel != "prerelease" {
        cleaned.channel = "stable".to_string();
    }
    if cleaned.github_repo.trim().is_empty() {
        cleaned.github_repo = crate::updater::DEFAULT_GITHUB_REPO.to_string();
    }
    crate::updater::save_update_settings(&db, &cleaned)?;
    Ok(Json(crate::updater::get_update_status(&db)))
}

pub(crate) async fn apply_github_update_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
) -> Result<Json<crate::updater::UpdateApplyReport>, AppError> {
    let report = crate::updater::download_and_apply_github_update(&db).await?;
    Ok(Json(report))
}

pub(crate) async fn upload_update_archive_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    mut multipart: Multipart,
) -> Result<Json<crate::updater::UpdateApplyReport>, AppError> {
    let mut archive_bytes: Option<Vec<u8>> = None;
    let mut expected_sha256: Option<String> = None;
    let mut target_version: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Invalid multipart upload: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "archive" | "file" => {
                if target_version.is_none() {
                    if let Some(fname) = field.file_name() {
                        // Extract version tag from `daypress-v0.2.0-...` if present
                        if let Some(rest) = fname.strip_prefix("daypress-v") {
                            if let Some(ver) = rest.split('-').next() {
                                target_version = Some(ver.to_string());
                            }
                        }
                    }
                }
                let data = field.bytes().await.map_err(|e| {
                    AppError::BadRequest(format!("Failed to read archive field: {e}"))
                })?;
                archive_bytes = Some(data.to_vec());
            }
            "sha256" => {
                let txt = field.text().await.unwrap_or_default();
                if !txt.trim().is_empty() {
                    expected_sha256 = Some(txt.trim().to_string());
                }
            }
            "version" => {
                let txt = field.text().await.unwrap_or_default();
                if !txt.trim().is_empty() {
                    target_version = Some(txt.trim().to_string());
                }
            }
            _ => {}
        }
    }

    let bytes = archive_bytes.ok_or_else(|| {
        AppError::BadRequest("Missing `archive` file field in multipart upload.".into())
    })?;
    let ver = target_version.unwrap_or_else(|| "custom".to_string());
    let exe_path = std::env::current_exe().map_err(|e| {
        AppError::Internal(format!("Could not determine current executable path: {e}"))
    })?;

    let report = crate::updater::apply_binary_update_to_path(
        &db,
        &exe_path,
        &bytes,
        expected_sha256.as_deref(),
        &ver,
        "upload",
    )?;
    Ok(Json(report))
}

#[derive(Debug, Deserialize, Default)]
pub struct RollbackUpdatePayload {
    pub restore_backup_filename: Option<String>,
}

pub(crate) async fn rollback_update_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    payload: Option<Json<RollbackUpdatePayload>>,
) -> Result<Json<crate::updater::UpdateApplyReport>, AppError> {
    let exe_path = std::env::current_exe().map_err(|e| {
        AppError::Internal(format!("Could not determine current executable path: {e}"))
    })?;
    let restore_file = payload
        .as_ref()
        .and_then(|p| p.restore_backup_filename.as_deref());
    let report = crate::updater::rollback_binary_at_path(&db, &exe_path, restore_file)?;
    Ok(Json(report))
}
