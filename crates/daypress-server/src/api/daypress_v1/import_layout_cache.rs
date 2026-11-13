#![allow(unused_imports)]
use super::*;
use crate::auth::{AuthUser, OptionalAuthUser};
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

pub(crate) async fn analyze_import_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    request: Request,
) -> Result<Json<MigrationFidelitySummary>, AppError> {
    let (parts, body) = request.into_parts();
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let mut raw_bytes = to_bytes(body, 200 * 1024 * 1024)
        .await
        .map_err(|e| AppError::BadRequest(format!("Failed to read request body: {}", e)))?;

    if content_type.starts_with("multipart/form-data") {
        let req_with_body = Request::from_parts(parts, axum::body::Body::from(raw_bytes));
        let mut multipart = Multipart::from_request(req_with_body, &db)
            .await
            .map_err(|e| AppError::BadRequest(format!("Multipart parse error: {}", e)))?;
        if let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?
        {
            raw_bytes = field
                .bytes()
                .await
                .map_err(|e| AppError::Internal(e.to_string()))?;
        } else {
            return Err(AppError::BadRequest(
                "No file found in multipart upload".to_string(),
            ));
        }
    }

    let text = String::from_utf8_lossy(&raw_bytes);
    let trimmed = text.trim_start();
    if trimmed.starts_with('<') {
        let summary = crate::wxr::analyze_wxr_xml(trimmed)?;
        return Ok(Json(summary));
    }

    let raw_json: Value = serde_json::from_slice(&raw_bytes).map_err(|e| {
        AppError::BadRequest(format!("Expected JSON bundle or WXR XML payload: {}", e))
    })?;
    if let Some(wxr_str) = raw_json.get("wxr_xml").and_then(|v| v.as_str()) {
        let summary = crate::wxr::analyze_wxr_xml(wxr_str)?;
        return Ok(Json(summary));
    }

    let bundle = normalize_import_bundle_payload(raw_json)?;
    Ok(Json(MigrationFidelitySummary::analyze_bundle(&bundle)))
}

// --- Visual Page Builder Layout API (Hybrid Wasm & JSON Layout Schema) ---

pub(crate) async fn get_layout_handler(
    crate::auth::OptionalAuthUser(user): crate::auth::OptionalAuthUser,
    State(db): State<Database>,
    headers: HeaderMap,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    // Layouts of unpublished posts are hidden (404), and passworded/member-gated layouts
    // require authorized viewer credentials.
    if let Some(mut post) = db.get_post_by_id(id)? {
        if !crate::auth::can_view_post(&post, user.as_ref()) {
            return Err(AppError::NotFound(format!("Post {} not found", id)));
        }
        let (_is_restricted, is_locked) = crate::handlers::public_ssr::apply_content_access_guards(
            &db,
            &mut post,
            user.as_ref(),
            &headers,
        );
        if is_locked {
            return Err(AppError::Forbidden(
                "Authentication or subscription is required to access this protected layout."
                    .to_string(),
            ));
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
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(mut payload): Json<DaypressLayoutPayload>,
) -> Result<Json<Value>, AppError> {
    let existing = db
        .get_post_by_id(id)?
        .ok_or_else(|| AppError::NotFound(format!("Post {} not found", id)))?;
    if !user.can_edit_post(existing.author, &existing.status, &existing.post_type) {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to edit the layout of this post.".to_string(),
        ));
    }
    if !user.is_editor_or_above() {
        sanitize_layout_value_for_non_editors(&mut payload.layout);
    }
    let layout_str = serde_json::to_string(&payload.layout)
        .map_err(|e| AppError::BadRequest(format!("Invalid layout JSON: {}", e)))?;

    let compiled_ssr = if payload.rendered_html.is_none() || !user.is_editor_or_above() {
        serde_json::from_value::<daypress_ast::DaypressLayout>(payload.layout.clone())
            .ok()
            .map(|parsed| daypress_ast::render_layout_to_html(&parsed))
    } else {
        None
    };

    // Persist converged global_styles into options (`daypress_elementor_kit_json` & `daypress_elementor_kit_css`)
    // Only Editors and Administrators may modify site-wide global styling tokens.
    if user.is_editor_or_above() {
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
    }

    let sanitized_rendered = if !user.is_editor_or_above() {
        compiled_ssr.clone().or_else(|| {
            payload
                .rendered_html
                .as_deref()
                .map(daypress_ast::sanitize_untrusted_html)
        })
    } else {
        None
    };
    let effective_rendered = if !user.is_editor_or_above() {
        sanitized_rendered.as_deref()
    } else {
        payload.rendered_html.as_deref().or(compiled_ssr.as_deref())
    };

    db.save_post_layout(id, &layout_str, effective_rendered)?;

    Ok(Json(serde_json::json!({
        "success": true,
        "post_id": id,
        "status": "saved"
    })))
}

fn sanitize_layout_value_for_non_editors(val: &mut serde_json::Value) {
    match val {
        serde_json::Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                if let serde_json::Value::String(s) = v {
                    let kl = k.to_ascii_lowercase();
                    if matches!(
                        kl.as_str(),
                        "raw"
                            | "content_html"
                            | "text"
                            | "description"
                            | "title"
                            | "content"
                            | "html"
                            | "quote"
                            | "button_text"
                            | "secondary_button_text"
                            | "label"
                            | "value"
                            | "comparison"
                            | "period"
                            | "prefix"
                            | "suffix"
                    ) {
                        *s = daypress_ast::sanitize_untrusted_html(s);
                    } else if matches!(
                        kl.as_str(),
                        "custom_css"
                            | "color"
                            | "background"
                            | "background_color"
                            | "width"
                            | "height"
                            | "border"
                            | "star_color"
                            | "track_color"
                            | "bar_color"
                            | "overlay_bg"
                            | "aspect_ratio"
                    ) {
                        *s = daypress_ast::sanitize_css_value(s);
                    } else if matches!(
                        kl.as_str(),
                        "url"
                            | "src"
                            | "href"
                            | "link"
                            | "link_url"
                            | "button_url"
                            | "image_url"
                            | "avatar_url"
                            | "secondary_button_url"
                            | "endpoint"
                            | "redirect_url"
                    ) {
                        *s = daypress_ast::sanitize_safe_url(s);
                    }
                } else {
                    sanitize_layout_value_for_non_editors(v);
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr.iter_mut() {
                sanitize_layout_value_for_non_editors(item);
            }
        }
        _ => {}
    }
}

pub(crate) async fn delete_layout_handler(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    let existing = db
        .get_post_by_id(id)?
        .ok_or_else(|| AppError::NotFound(format!("Post {} not found", id)))?;
    if !user.can_edit_post(existing.author, &existing.status, &existing.post_type) {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to delete the layout of this post.".to_string(),
        ));
    }
    db.delete_post_layout(id)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "post_id": id,
        "status": "deleted",
        "message": "Visual layout removed. Standard text editor unlocked."
    })))
}

pub(crate) async fn duplicate_post_handler(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<DaypressPostResponse>, AppError> {
    let existing = db
        .get_post_by_id(id)?
        .ok_or_else(|| AppError::NotFound(format!("Post {} not found", id)))?;
    if existing.post_type.eq_ignore_ascii_case("page") && !user.can_edit_pages() {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to duplicate pages.".to_string(),
        ));
    }
    if !user.is_editor_or_above() && existing.author != user.user_id {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to duplicate another user's post.".to_string(),
        ));
    }
    match db.duplicate_post(id)? {
        Some(cloned) => Ok(Json(cloned)),
        None => Err(AppError::NotFound(format!("Post {} not found", id))),
    }
}

pub(crate) async fn restore_revision_v1_handler(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(rev_id): AxPath<i64>,
) -> Result<Json<DaypressPostResponse>, AppError> {
    if !user.is_editor_or_above() {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to restore revisions directly.".to_string(),
        ));
    }
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

// --- Native Subsystems: SSR Hot Cache & Edge Purge ---

#[derive(Debug, Deserialize, Default)]
pub struct PurgeCachePayload {
    pub cf_zone_id: Option<String>,
    pub cf_api_token: Option<String>,
}

pub(crate) async fn get_hot_cache_stats_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
) -> Result<Json<crate::hot_cache::HotCacheStats>, AppError> {
    Ok(Json(db.hot_cache_stats()))
}

pub(crate) async fn purge_hot_cache_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    payload: Option<Json<PurgeCachePayload>>,
) -> Result<Json<Value>, AppError> {
    if let Some(Json(cfg)) = payload {
        if let Some(zone) = cfg.cf_zone_id {
            let _ = db.set_option("daypress_cf_zone_id", zone.trim());
        }
        if let Some(token) = cfg.cf_api_token {
            if !token.trim().is_empty() {
                let _ = db.set_option("daypress_cf_api_token", token.trim());
            }
        }
    }
    db.invalidate_hot_cache("manual_purge");
    Ok(Json(serde_json::json!({
        "success": true,
        "message": "In-memory SSR hot cache purged.",
        "stats": db.hot_cache_stats()
    })))
}
