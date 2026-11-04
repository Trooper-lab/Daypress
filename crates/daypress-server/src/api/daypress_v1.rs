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
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<DaypressLayoutPayload>,
) -> Result<Json<Value>, AppError> {
    if let Some(existing) = db.get_post_by_id(id)? {
        if !user.can_edit_post(existing.author, &existing.status, &existing.post_type) {
            return Err(AppError::Forbidden(
                "Sorry, you are not allowed to edit the layout of this post.".to_string(),
            ));
        }
    }
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
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    if let Some(existing) = db.get_post_by_id(id)? {
        if !user.can_edit_post(existing.author, &existing.status, &existing.post_type) {
            return Err(AppError::Forbidden(
                "Sorry, you are not allowed to delete the layout of this post.".to_string(),
            ));
        }
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
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
) -> Result<Json<crate::hot_cache::HotCacheStats>, AppError> {
    Ok(Json(db.hot_cache_stats()))
}

pub(crate) async fn purge_hot_cache_handler(
    _auth: crate::auth::EditorUser,
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

// --- Native Subsystems: Forms & Webhooks Engine ---

pub(crate) fn interpolate_merge_tags(
    template: &str,
    data: &serde_json::Map<String, Value>,
    form_title: &str,
    entry_id: i64,
    source_url: &str,
) -> String {
    let mut out = template
        .replace("{{form_title}}", form_title)
        .replace("{{entry_id}}", &entry_id.to_string())
        .replace("{{source_url}}", source_url);
    for (k, v) in data {
        let placeholder = format!("{{{{{}}}}}", k);
        let val_str = match v {
            Value::String(s) => s.clone(),
            Value::Null => String::new(),
            other => other.to_string(),
        };
        out = out.replace(&placeholder, &val_str);
    }
    out
}

fn evaluate_condition_match(actual: &str, operator: &str, expected: &str) -> bool {
    let a = actual.trim().to_lowercase();
    let e = expected.trim().to_lowercase();
    match operator.trim() {
        "not_equals" | "!=" => a != e,
        "contains" => a.contains(&e),
        "not_empty" => !a.is_empty(),
        "empty" => a.is_empty(),
        _ => a == e,
    }
}

pub(crate) async fn deliver_form_submission_notifications(
    db: &Database,
    form_rec: &DaypressForm,
    entry_id: i64,
    data_map: &serde_json::Map<String, Value>,
    attachments: &[FormAttachmentMeta],
    source_url: &str,
) -> (String, String, String) {
    let settings: FormSettings = serde_json::from_str(&form_rec.settings_json).unwrap_or_default();

    // 1. Collect target admin recipients (default + matching conditional routes)
    let mut recipients: Vec<String> = Vec::new();
    for part in settings.recipient_email.split(',') {
        let trimmed = part.trim();
        if !trimmed.is_empty() && !recipients.iter().any(|r| r.eq_ignore_ascii_case(trimmed)) {
            recipients.push(trimmed.to_string());
        }
    }
    for route in &settings.conditional_routes {
        let field_val = data_map
            .get(&route.field)
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if evaluate_condition_match(field_val, &route.operator, &route.value) {
            for part in route.recipient_email.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty()
                    && !recipients.iter().any(|r| r.eq_ignore_ascii_case(trimmed))
                {
                    recipients.push(trimmed.to_string());
                }
            }
        }
    }

    let mut email_status = "disabled".to_string();
    let mut email_error = String::new();

    if !recipients.is_empty() {
        let subj_tpl = if settings.email_subject_template.trim().is_empty() {
            "[Daypress Form] New submission: {{form_title}}"
        } else {
            settings.email_subject_template.trim()
        };
        let subject =
            interpolate_merge_tags(subj_tpl, data_map, &form_rec.title, entry_id, source_url);

        let mut summary_rows = Vec::new();
        let mut text_lines = Vec::new();
        for (k, v) in data_map {
            if k.starts_with("_np_") || k.starts_with("_dp_") {
                continue;
            }
            let val_str = v
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| v.to_string());
            summary_rows.push(format!(
                "<tr><td style=\"padding:8px 12px;border-bottom:1px solid #e5e7eb;font-weight:600;color:#111827;\">{}</td><td style=\"padding:8px 12px;border-bottom:1px solid #e5e7eb;color:#374151;\">{}</td></tr>",
                crate::templates::utils::escape_html(k),
                crate::templates::utils::escape_html(&val_str)
            ));
            text_lines.push(format!("{}: {}", k, val_str));
        }
        if !attachments.is_empty() {
            let att_names: Vec<String> = attachments
                .iter()
                .map(|a| format!("{} ({} B)", a.filename, a.size_bytes))
                .collect();
            summary_rows.push(format!(
                "<tr><td style=\"padding:8px 12px;border-bottom:1px solid #e5e7eb;font-weight:600;color:#111827;\">Attachments</td><td style=\"padding:8px 12px;border-bottom:1px solid #e5e7eb;color:#374151;\">{}</td></tr>",
                crate::templates::utils::escape_html(&att_names.join(", "))
            ));
        }

        let body_html = format!(
            "<div style=\"font-family:sans-serif;max-width:600px;\"><h3 style=\"margin-bottom:12px;\">New submission for {} (#{})</h3><table style=\"width:100%;border-collapse:collapse;border:1px solid #e5e7eb;\"><tbody>{}</tbody></table></div>",
            crate::templates::utils::escape_html(&form_rec.title),
            entry_id,
            summary_rows.join("")
        );
        let body_text = format!(
            "New submission for {} (#{}):\n{}",
            form_rec.title,
            entry_id,
            text_lines.join("\n")
        );

        email_status = "sent".to_string();
        for rcpt in &recipients {
            let msg = crate::email::EmailMessage {
                to: rcpt.clone(),
                subject: subject.clone(),
                body_html: body_html.clone(),
                body_text: body_text.clone(),
            };
            if let Err(e) = crate::email::send_transactional_email(db, &msg).await {
                email_status = "failed".to_string();
                email_error = e.to_string();
            }
        }
    }

    // 2. Optional User Auto-Responder
    if settings.auto_responder_enabled {
        let email_field_key = if settings.auto_responder_email_field.trim().is_empty() {
            "email"
        } else {
            settings.auto_responder_email_field.trim()
        };
        let user_email = data_map
            .get(email_field_key)
            .or_else(|| data_map.get("email"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .unwrap_or("");
        if user_email.contains('@') {
            let subj_tpl = if settings.auto_responder_subject.trim().is_empty() {
                "We received your message — {{form_title}}"
            } else {
                settings.auto_responder_subject.trim()
            };
            let body_tpl = if settings.auto_responder_body.trim().is_empty() {
                "Hi,\n\nThank you for contacting us via {{form_title}}. We have received your submission (#{{entry_id}}) and will get back to you shortly."
            } else {
                settings.auto_responder_body.trim()
            };
            let auto_subj =
                interpolate_merge_tags(subj_tpl, data_map, &form_rec.title, entry_id, source_url);
            let auto_text =
                interpolate_merge_tags(body_tpl, data_map, &form_rec.title, entry_id, source_url);
            let auto_html = format!(
                "<div style=\"font-family:sans-serif;line-height:1.6;color:#1f2937;\">{}</div>",
                crate::templates::utils::escape_html(&auto_text).replace('\n', "<br/>")
            );
            let auto_msg = crate::email::EmailMessage {
                to: user_email.to_string(),
                subject: auto_subj,
                body_html: auto_html,
                body_text: auto_text,
            };
            match crate::email::send_transactional_email(db, &auto_msg).await {
                Ok(_) => {
                    if email_status == "disabled" {
                        email_status = "sent".to_string();
                    }
                }
                Err(e) => {
                    email_status = "failed".to_string();
                    if email_error.is_empty() {
                        email_error = format!("Auto-responder failed: {}", e);
                    }
                }
            }
        }
    }

    // 3. Optional Outbound Webhook
    let mut webhook_status = "none".to_string();
    let wh_url = settings.webhook_url.trim();
    if !wh_url.is_empty() {
        webhook_status = "failed".to_string();
        if let Ok(client) = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(6))
            .build()
        {
            let wh_payload = serde_json::json!({
                "event": "form.submitted",
                "form_id": form_rec.id,
                "form_title": form_rec.title,
                "entry_id": entry_id,
                "source_url": source_url,
                "data": data_map,
                "attachments": attachments,
            });
            if let Ok(resp) = client.post(wh_url).json(&wh_payload).send().await {
                if resp.status().is_success() {
                    webhook_status = "sent".to_string();
                }
            }
        }
    }

    let _ = db.update_form_entry_delivery_status(
        entry_id,
        &email_status,
        &email_error,
        &webhook_status,
    );

    (email_status, email_error, webhook_status)
}

pub(crate) fn extract_country_from_headers(headers: &HeaderMap) -> String {
    for key in ["cf-ipcountry", "x-vercel-ip-country", "x-country-code"] {
        if let Some(val) = headers.get(key).and_then(|v| v.to_str().ok()) {
            let clean = val.trim().to_uppercase();
            if clean.len() == 2 && clean != "XX" {
                return clean;
            }
        }
    }
    if let Some(lang) = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(first_tag) = lang.split(',').next() {
            let tag = first_tag.split(';').next().unwrap_or("").trim();
            if let Some((_, region)) = tag.split_once('-') {
                let r = region.trim().to_uppercase();
                if r.len() == 2 && r.chars().all(|c| c.is_ascii_alphabetic()) {
                    return r;
                }
            }
            let primary = tag.to_uppercase();
            if primary.len() == 2 && primary.chars().all(|c| c.is_ascii_alphabetic()) {
                return primary;
            }
        }
    }
    "XX".to_string()
}

pub(crate) async fn submit_form_handler(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    request: Request,
) -> Result<Json<Value>, AppError> {
    let (parts, body) = request.into_parts();
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let ip = parts
        .headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .unwrap_or("127.0.0.1");

    let ua = parts
        .headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let referer_hdr = parts
        .headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(ip.as_bytes());
    let ip_hash = hex::encode(hasher.finalize());

    let form_rec_opt = db.get_form(id)?;
    let (form_settings, form_fields) = if let Some(ref frec) = form_rec_opt {
        let s: FormSettings = serde_json::from_str(&frec.settings_json).unwrap_or_default();
        let f: Vec<FormFieldSchema> = serde_json::from_str(&frec.schema_json).unwrap_or_default();
        (s, f)
    } else {
        (FormSettings::default(), Vec::new())
    };

    // 1. Per-IP Rate Limiter (HTTP 429 on flood)
    let limit_per_min = if form_settings.rate_limit_per_minute == 0 {
        10
    } else {
        form_settings.rate_limit_per_minute
    };
    if !db
        .form_rate_limiter
        .check_and_record(id, &ip_hash, limit_per_min)
    {
        return Err(AppError::TooManyRequests(
            "Too many form submissions from your IP address. Please wait a minute and try again."
                .to_string(),
        ));
    }

    let bytes = to_bytes(body, 25 * 1024 * 1024)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let mut data_map = serde_json::Map::new();
    let mut attachments: Vec<FormAttachmentMeta> = Vec::new();

    if content_type.starts_with("multipart/form-data") {
        let req_with_body = Request::from_parts(parts.clone(), axum::body::Body::from(bytes));
        let mut multipart = Multipart::from_request(req_with_body, &db)
            .await
            .map_err(|e| AppError::BadRequest(format!("Multipart form error: {}", e)))?;

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?
        {
            let field_name = field.name().unwrap_or("field").to_string();
            if let Some(raw_fname) = field.file_name().map(|s| s.to_string()) {
                let mime_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let file_bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                if file_bytes.is_empty() || raw_fname.trim().is_empty() {
                    continue;
                }
                let Some(safe_fname) = crate::media::sanitize_filename(&raw_fname) else {
                    return Err(AppError::BadRequest("Invalid attachment filename.".into()));
                };
                let ext = Path::new(&safe_fname)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                // Block executable/script extensions unconditionally
                if matches!(
                    ext.as_str(),
                    "php"
                        | "phtml"
                        | "exe"
                        | "sh"
                        | "bat"
                        | "cmd"
                        | "js"
                        | "mjs"
                        | "html"
                        | "htm"
                        | "svg"
                ) {
                    return Err(AppError::BadRequest(format!(
                        "File extension .{} is not allowed.",
                        ext
                    )));
                }

                // Check field-level extension allowlist & max MB
                let field_schema = form_fields.iter().find(|f| f.name == field_name);
                let max_mb = field_schema
                    .map(|f| f.max_file_size_mb.max(1))
                    .unwrap_or(10) as usize;
                if file_bytes.len() > max_mb * 1024 * 1024 {
                    return Err(AppError::BadRequest(format!(
                        "File '{}' exceeds the {} MB limit.",
                        safe_fname, max_mb
                    )));
                }
                if let Some(fschema) = field_schema {
                    if !fschema.accept_extensions.trim().is_empty() {
                        let allowed: Vec<String> = fschema
                            .accept_extensions
                            .split(',')
                            .map(|s| s.trim().trim_start_matches('.').to_lowercase())
                            .filter(|s| !s.is_empty())
                            .collect();
                        if !allowed.is_empty() && !allowed.iter().any(|a| a == &ext) {
                            return Err(AppError::BadRequest(format!(
                                "File '{}' must be one of: {}",
                                safe_fname, fschema.accept_extensions
                            )));
                        }
                    }
                }

                let dir = Path::new("uploads")
                    .join("form-submissions")
                    .join(id.to_string());
                let _ = fs::create_dir_all(&dir);
                let unique_name =
                    format!("{}_{}", chrono::Utc::now().timestamp_millis(), safe_fname);
                let stored_path = dir.join(&unique_name);
                fs::write(&stored_path, &file_bytes)
                    .map_err(|e| AppError::Internal(format!("Failed to store upload: {}", e)))?;

                let stored_path_str = stored_path.to_string_lossy().replace('\\', "/");
                attachments.push(FormAttachmentMeta {
                    field_name: field_name.clone(),
                    filename: unique_name.clone(),
                    original_name: safe_fname.clone(),
                    stored_path: stored_path_str,
                    mime_type,
                    size_bytes: file_bytes.len() as u64,
                });
                data_map.insert(field_name, Value::String(unique_name));
            } else {
                let txt = field.text().await.unwrap_or_default();
                if let Some(existing) = data_map.get_mut(&field_name) {
                    let prev = existing.as_str().unwrap_or("").to_string();
                    *existing = Value::String(if prev.is_empty() {
                        txt
                    } else {
                        format!("{}, {}", prev, txt)
                    });
                } else {
                    data_map.insert(field_name, Value::String(txt));
                }
            }
        }
    } else {
        let raw_val: Value = serde_json::from_slice(&bytes)
            .map_err(|e| AppError::BadRequest(format!("Invalid form submission: {}", e)))?;
        let payload_data = match raw_val.get("data") {
            Some(d) if d.is_object() => d.clone(),
            _ => raw_val,
        };
        if let Some(obj) = payload_data.as_object() {
            for (k, v) in obj {
                data_map.insert(k.clone(), v.clone());
            }
        }
    }

    // 2. Optional Turnstile / hCaptcha Verification (HTTP 400 on invalid token)
    let provider = form_settings.captcha_provider.trim().to_lowercase();
    if (provider == "turnstile" || provider == "hcaptcha")
        && !form_settings.captcha_secret_key.trim().is_empty()
    {
        let token = data_map
            .get("cf-turnstile-response")
            .or_else(|| data_map.get("h-captcha-response"))
            .or_else(|| data_map.get("_captcha_token"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        if token.is_empty() {
            return Err(AppError::BadRequest(
                "Captcha verification token is required.".to_string(),
            ));
        }

        let secret = form_settings.captcha_secret_key.trim();
        let is_test_bypass = secret.starts_with("test_") && token == "PASS_TEST_TOKEN";
        if !is_test_bypass {
            let verify_url = if provider == "turnstile" {
                "https://challenges.cloudflare.com/turnstile/v0/siteverify"
            } else {
                "https://api.hcaptcha.com/siteverify"
            };
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let resp = client
                .post(verify_url)
                .header(
                    reqwest::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(format!("secret={}&response={}", secret, token.as_str()))
                .send()
                .await
                .map_err(|_| AppError::BadRequest("Captcha verification request failed.".into()))?;
            let body_json: Value = resp.json().await.unwrap_or_default();
            if body_json.get("success").and_then(|v| v.as_bool()) != Some(true) {
                return Err(AppError::BadRequest(
                    "Captcha verification failed.".to_string(),
                ));
            }
        }
    }

    // 3. Anti-Spam Honeypot (`_np_hp`) & Minimum Submit Timestamp Trap (`_dp_ts`)
    // Quarantined into `status = "spam"` in SQLite so false positives are never lost!
    let mut is_spam = false;
    if let Some(hp) = data_map.get("_np_hp").and_then(|v| v.as_str()) {
        if !hp.trim().is_empty() {
            is_spam = true;
        }
    }
    if !is_spam && form_settings.min_submit_seconds > 0 {
        if let Some(ts_val) = data_map.get("_dp_ts") {
            let rendered_ms = ts_val
                .as_i64()
                .or_else(|| ts_val.as_str().and_then(|s| s.trim().parse::<i64>().ok()))
                .unwrap_or(0);
            if rendered_ms > 0 {
                // Support both seconds and milliseconds timestamps
                let normalized_ms = if rendered_ms < 10_000_000_000 {
                    rendered_ms * 1000
                } else {
                    rendered_ms
                };
                let now_ms = chrono::Utc::now().timestamp_millis();
                let elapsed_ms = now_ms - normalized_ms;
                if elapsed_ms >= 0 && elapsed_ms < (form_settings.min_submit_seconds as i64 * 1000)
                {
                    is_spam = true;
                }
            }
        }
    }

    let source_url = data_map
        .get("_source_url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .unwrap_or(referer_hdr);

    // Clean internal verification tokens before storing in data_json
    let mut clean_map = data_map.clone();
    clean_map.remove("_np_hp");
    clean_map.remove("_dp_ts");
    clean_map.remove("cf-turnstile-response");
    clean_map.remove("h-captcha-response");
    clean_map.remove("_captcha_token");

    let data_str =
        serde_json::to_string(&Value::Object(clean_map.clone())).unwrap_or_else(|_| "{}".into());
    let attachments_str = serde_json::to_string(&attachments).unwrap_or_else(|_| "[]".to_string());

    if is_spam {
        let _ = db.submit_form_entry_full(
            id,
            &data_str,
            &attachments_str,
            "spam",
            "disabled",
            "none",
            &source_url,
            &ip_hash,
            ua,
        )?;
        return Err(AppError::BadRequest(
            "Spam submission detected and quarantined.".to_string(),
        ));
    }

    let has_email_targets = !form_settings.recipient_email.trim().is_empty()
        || !form_settings.conditional_routes.is_empty()
        || form_settings.auto_responder_enabled;
    let initial_email_status = if has_email_targets {
        "queued"
    } else {
        "disabled"
    };
    let initial_webhook_status = if !form_settings.webhook_url.trim().is_empty() {
        "queued"
    } else {
        "none"
    };

    // 4. Fail-Safe SQLite Vault Commit BEFORE SMTP or Webhook dispatch
    let entry_id = db.submit_form_entry_full(
        id,
        &data_str,
        &attachments_str,
        "unread",
        initial_email_status,
        initial_webhook_status,
        &source_url,
        &ip_hash,
        ua,
    )?;

    // 5. Record Cookieless Analytics Conversion (`form_submit`)
    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let mut v_hasher = Sha256::new();
    v_hasher.update(ip.as_bytes());
    v_hasher.update(ua.as_bytes());
    v_hasher.update(today.as_bytes());
    let visitor_hash = hex::encode(v_hasher.finalize());
    let country = extract_country_from_headers(&parts.headers);
    let utm_source = clean_map.get("_utm_source").and_then(|v| v.as_str());
    let utm_medium = clean_map.get("_utm_medium").and_then(|v| v.as_str());
    let utm_campaign = clean_map.get("_utm_campaign").and_then(|v| v.as_str());
    let form_title_for_goal = form_rec_opt
        .as_ref()
        .map(|f| f.title.as_str())
        .unwrap_or("Form");
    let _ = db.record_analytics_event_full(
        &visitor_hash,
        if source_url.is_empty() {
            "/"
        } else {
            &source_url
        },
        "",
        "desktop",
        &country,
        utm_source,
        utm_medium,
        utm_campaign,
        "form_submit",
        Some(&format!("Form: {}", form_title_for_goal)),
    );

    // 6. Dispatch Admin Notifications, User Auto-Responder, and Outbound Webhook
    if let Some(form_rec) = form_rec_opt {
        let (email_status, email_error, webhook_status) = deliver_form_submission_notifications(
            &db,
            &form_rec,
            entry_id,
            &clean_map,
            &attachments,
            &source_url,
        )
        .await;

        return Ok(Json(serde_json::json!({
            "success": true,
            "entry_id": entry_id,
            "status": "unread",
            "email_status": email_status,
            "email_error": email_error,
            "webhook_status": webhook_status,
            "message": form_settings.success_message,
            "redirect_url": form_settings.redirect_url,
            "webhook_url": form_settings.webhook_url
        })));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "entry_id": entry_id,
        "status": "unread",
        "message": "Form submission received successfully."
    })))
}

// --- Native Subsystems: Cookieless Daily Salted Analytics ---

#[derive(Debug, Deserialize, Default)]
pub struct AnalyticsRangeQuery {
    pub range: Option<String>,
}

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
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .unwrap_or("127.0.0.1");

    let ua = parts
        .headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let country = extract_country_from_headers(&parts.headers);

    let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(client_ip.as_bytes());
    hasher.update(ua.as_bytes());
    hasher.update(today.as_bytes());
    let visitor_hash = hex::encode(hasher.finalize());

    let referrer = payload.referrer.unwrap_or_default();
    let device_type = payload.device_type.unwrap_or_else(|| "desktop".to_string());
    let event_type = if payload.event_type.trim().is_empty() {
        "pageview"
    } else {
        payload.event_type.trim()
    };

    db.record_analytics_event_full(
        &visitor_hash,
        &payload.page_url,
        &referrer,
        &device_type,
        &country,
        payload.utm_source.as_deref(),
        payload.utm_medium.as_deref(),
        payload.utm_campaign.as_deref(),
        event_type,
        payload.goal_name.as_deref(),
    )?;

    Ok(Json(serde_json::json!({ "recorded": true })))
}

pub(crate) async fn get_analytics_summary_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Query(query): Query<AnalyticsRangeQuery>,
) -> Result<Json<AnalyticsSummary>, AppError> {
    let range = query.range.as_deref().unwrap_or("30d");
    let summary = db.get_analytics_dashboard(range)?;
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
    let exe_path = crate::updater::canonical_executable_path()?;

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
    let exe_path = crate::updater::canonical_executable_path()?;
    let restore_file = payload
        .as_ref()
        .and_then(|p| p.restore_backup_filename.as_deref());
    let report = crate::updater::rollback_binary_at_path(&db, &exe_path, restore_file)?;
    Ok(Json(report))
}

// --- Native Subsystems: Static Edge Publishing & Domain Manager ---

pub(crate) async fn get_publish_status_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
) -> Result<Json<crate::publisher::PublishStatusResponse>, AppError> {
    Ok(Json(db.compute_publish_status()))
}

pub(crate) async fn save_publish_settings_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(payload): Json<crate::publisher::PublishSettingsUpdateRequest>,
) -> Result<Json<Value>, AppError> {
    let saved = db.save_publish_settings(&payload)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "settings": saved.redacted()
    })))
}

pub(crate) async fn run_publish_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    payload: Option<Json<crate::publisher::PublishRunRequest>>,
) -> Result<Json<Value>, AppError> {
    let req = payload.map(|Json(p)| p).unwrap_or_default();
    let dep = crate::publisher::execute_publish(&db, &req).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "deployment": dep
    })))
}

pub(crate) async fn rollback_publish_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    let dep = crate::publisher::execute_rollback(&db, id).await?;
    Ok(Json(serde_json::json!({
        "success": true,
        "deployment": dep
    })))
}

pub(crate) async fn download_deployment_zip_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<axum::response::Response, AppError> {
    use axum::response::IntoResponse;
    let (dep, _) = db
        .get_deployment_with_manifest(id)
        .ok_or_else(|| AppError::NotFound(format!("Deployment #{id} not found")))?;
    let path = dep.bundle_path.ok_or_else(|| {
        AppError::NotFound(format!("Deployment #{id} has no saved .zip snapshot"))
    })?;
    let bytes = std::fs::read(&path)
        .map_err(|e| AppError::NotFound(format!("Snapshot archive not found on disk: {e}")))?;

    let filename = format!("daypress-static-deploy-{id}.zip");
    let mut headers = axum::http::HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/zip"),
    );
    if let Ok(disp) = format!("attachment; filename=\"{filename}\"").parse() {
        headers.insert(header::CONTENT_DISPOSITION, disp);
    }
    Ok((axum::http::StatusCode::OK, headers, bytes).into_response())
}

pub(crate) async fn configure_publish_domain_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    payload: Option<Json<crate::publisher::DomainActionRequest>>,
) -> Result<Json<crate::publisher::DomainConfigResult>, AppError> {
    let req = payload.map(|Json(p)| p).unwrap_or_default();
    let mut settings = db.get_publish_settings();
    if let Some(ref prov) = req.provider {
        if !prov.trim().is_empty() {
            settings.provider = prov.trim().to_ascii_lowercase();
            let _ = db.set_option("daypress_publish_provider", &settings.provider);
        }
    }
    if let Some(ref dom) = req.domain {
        let clean = dom
            .trim()
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_end_matches('/');
        if !clean.is_empty() {
            if settings.provider == "cloudflare" {
                let _ = db.set_option("daypress_cf_custom_domain", clean);
                settings.cf_custom_domain = clean.to_string();
            } else if settings.provider == "netlify" {
                let _ = db.set_option("daypress_netlify_custom_domain", clean);
                settings.netlify_custom_domain = clean.to_string();
            }
            if settings.production_url.trim().is_empty() {
                let prod = format!("https://{clean}");
                let _ = db.set_option("daypress_publish_production_url", &prod);
                settings.production_url = prod;
            }
        }
    }
    let res =
        crate::publisher::providers::configure_custom_domain(&settings, req.domain.as_deref())
            .await
            .map_err(AppError::BadRequest)?;
    Ok(Json(res))
}

pub(crate) async fn verify_publish_domain_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    payload: Option<Json<crate::publisher::DomainActionRequest>>,
) -> Result<Json<crate::publisher::DomainVerifyResult>, AppError> {
    let req = payload.map(|Json(p)| p).unwrap_or_default();
    let settings = db.get_publish_settings();
    let res =
        crate::publisher::providers::verify_domain_status(&settings, req.domain.as_deref()).await;
    Ok(Json(res))
}

// --- Native Subsystems: Custom Post Types, Custom Taxonomies, Structured Field Groups & Collection Grids (v0.6) ---

pub(crate) async fn list_content_types_v1_handler(
    State(db): State<Database>,
) -> Result<Json<Vec<CustomPostTypeDefinition>>, AppError> {
    let defs = db.list_post_type_definitions()?;
    Ok(Json(defs))
}

pub(crate) async fn save_content_type_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(def): Json<CustomPostTypeDefinition>,
) -> Result<Json<CustomPostTypeDefinition>, AppError> {
    if def.slug.trim().is_empty() && def.label_singular.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Custom post type requires a slug or singular label.".to_string(),
        ));
    }
    let saved = db.save_post_type_definition(&def)?;
    Ok(Json(saved))
}

pub(crate) async fn delete_content_type_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
) -> Result<Json<Value>, AppError> {
    let deleted = db.delete_post_type_definition(&slug)?;
    Ok(Json(serde_json::json!({
        "deleted": deleted,
        "slug": slug
    })))
}

pub(crate) async fn provision_content_type_preset_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let preset = payload
        .get("preset")
        .or_else(|| payload.get("preset_key"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    if preset.is_empty() {
        return Err(AppError::BadRequest(
            "Missing 'preset' key (e.g. 'case_studies', 'team', 'docs', 'jobs', 'events', 'portfolio', 'testimonials').".to_string(),
        ));
    }
    let (post_type, taxonomies, field_group) = db
        .provision_content_type_preset(preset)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(Json(serde_json::json!({
        "status": "provisioned",
        "preset": preset,
        "post_type": post_type,
        "taxonomies": taxonomies,
        "field_group": field_group
    })))
}

pub(crate) async fn list_custom_taxonomies_v1_handler(
    State(db): State<Database>,
) -> Result<Json<Vec<CustomTaxonomyDefinition>>, AppError> {
    let defs = db.list_custom_taxonomy_definitions()?;
    Ok(Json(defs))
}

pub(crate) async fn save_custom_taxonomy_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(def): Json<CustomTaxonomyDefinition>,
) -> Result<Json<CustomTaxonomyDefinition>, AppError> {
    if def.slug.trim().is_empty() && def.label_singular.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Custom taxonomy requires a slug or singular label.".to_string(),
        ));
    }
    let saved = db.save_custom_taxonomy_definition(&def)?;
    Ok(Json(saved))
}

pub(crate) async fn delete_custom_taxonomy_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
) -> Result<Json<Value>, AppError> {
    let deleted = db.delete_custom_taxonomy_definition(&slug)?;
    Ok(Json(serde_json::json!({
        "deleted": deleted,
        "slug": slug
    })))
}

#[derive(Debug, Deserialize, Default)]
pub struct FieldGroupQueryParams {
    pub post_type: Option<String>,
    pub template: Option<String>,
}

pub(crate) async fn list_field_groups_v1_handler(
    State(db): State<Database>,
    Query(params): Query<FieldGroupQueryParams>,
) -> Result<Json<Vec<FieldGroupDefinition>>, AppError> {
    if let Some(ref pt) = params.post_type {
        let pt_clean = pt.trim();
        if !pt_clean.is_empty() {
            return Ok(Json(db.list_field_groups(Some(pt_clean))?));
        }
    }
    let groups = db.list_field_groups(None)?;
    Ok(Json(groups))
}

pub(crate) async fn save_field_group_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(group): Json<FieldGroupDefinition>,
) -> Result<Json<FieldGroupDefinition>, AppError> {
    if group.title.trim().is_empty() && group.group_key.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Field group requires a title or group_key.".to_string(),
        ));
    }
    let saved = db.save_field_group(&group)?;
    Ok(Json(saved))
}

pub(crate) async fn delete_field_group_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    AxPath(id_or_key): AxPath<String>,
) -> Result<Json<Value>, AppError> {
    let deleted = db.delete_field_group(&id_or_key)?;
    Ok(Json(serde_json::json!({
        "deleted": deleted,
        "id": id_or_key
    })))
}

#[derive(Debug, Deserialize, Default)]
pub struct CollectionQueryV1Params {
    pub post_type: Option<String>,
    pub taxonomy: Option<String>,
    pub term: Option<String>,
    #[serde(alias = "q")]
    pub search: Option<String>,
    pub sort: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub columns: Option<u8>,
    #[serde(alias = "show_thumbnail")]
    pub show_image: Option<bool>,
    pub show_excerpt: Option<bool>,
    #[serde(alias = "show_meta")]
    pub show_date: Option<bool>,
    #[serde(alias = "skin")]
    pub card_skin: Option<String>,
    pub meta_fields: Option<String>,
}

pub(crate) async fn collection_query_v1_handler(
    State(db): State<Database>,
    Query(params): Query<CollectionQueryV1Params>,
) -> Result<Json<Value>, AppError> {
    let post_type = params
        .post_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("post");
    let taxonomy = params
        .taxonomy
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let term = params
        .term
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("all"));
    let search = params
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let sort = params
        .sort
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("date_desc");
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(6).clamp(1, 100);
    let columns = params.columns.unwrap_or(3).clamp(1, 4);
    let show_image = params.show_image.unwrap_or(true);
    let show_excerpt = params.show_excerpt.unwrap_or(true);
    let show_date = params.show_date.unwrap_or(true);
    let card_skin = params.card_skin.as_deref().unwrap_or("editorial");
    let meta_fields: Vec<String> = params
        .meta_fields
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let base_url = db
        .get_settings()
        .map(|s| s.url)
        .unwrap_or_else(|_| "http://localhost:8090".to_string());
    let (posts, total, filter_taxonomy, term_counts) = db.query_collection_posts(
        post_type,
        taxonomy,
        term,
        Some(sort),
        search,
        page,
        per_page,
        &base_url,
    )?;
    let total_pages = if total == 0 {
        1
    } else {
        total.div_ceil(per_page.max(1))
    };
    let cards_html =
        crate::theme_builder::ssr_injector::SsrInjector::render_loop_grid_html_with_skin(
            &posts,
            columns,
            show_image,
            show_excerpt,
            show_date,
            false,
            None,
            card_skin,
            &meta_fields,
        );

    Ok(Json(serde_json::json!({
        "posts": posts,
        "total": total,
        "page": page,
        "per_page": per_page,
        "total_pages": total_pages,
        "taxonomy": filter_taxonomy,
        "term_counts": term_counts,
        "cards_html": cards_html
    })))
}

// --- Native Subsystems: SQLite FTS5 Instant Search, Auto-OG Social Cards & Live SEO/WCAG Linter (v0.7) ---

#[derive(Debug, Deserialize, Default)]
pub struct FtsSearchQueryParams {
    #[serde(alias = "search")]
    pub q: Option<String>,
    pub post_type: Option<String>,
    #[serde(alias = "per_page")]
    pub limit: Option<usize>,
}

pub(crate) async fn fts_search_v1_handler(
    State(db): State<Database>,
    Query(params): Query<FtsSearchQueryParams>,
) -> Result<Json<crate::adapters::database::search_fts::FtsSearchResponse>, AppError> {
    let q = params.q.as_deref().unwrap_or("").trim();
    let post_type = params
        .post_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("all"));
    let limit = params.limit.unwrap_or(12).clamp(1, 50);
    let resp = db.search_fts(q, post_type, limit)?;
    Ok(Json(resp))
}

pub(crate) async fn fts_reindex_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
) -> Result<Json<Value>, AppError> {
    let indexed = db.rebuild_fts_index()?;
    Ok(Json(serde_json::json!({
        "status": "reindexed",
        "engine": "sqlite-fts5-bm25",
        "indexed_count": indexed
    })))
}

pub(crate) async fn seo_audit_v1_handler(
    State(db): State<Database>,
    Json(payload): Json<crate::adapters::database::search_fts::AuditRequestPayload>,
) -> Result<Json<crate::adapters::database::search_fts::AuditReportResponse>, AppError> {
    let report = db.audit_post_wcag_links_seo(&payload);
    Ok(Json(report))
}
