#![allow(unused_imports)]
use super::*;
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

#[derive(Debug, Deserialize, Default)]
pub struct FormsQueryParams {
    pub form_id: Option<i64>,
    pub status: Option<String>,
    pub s: Option<String>,
}

pub async fn handle_admin_forms(
    State(db): State<Database>,
    Query(params): Query<FormsQueryParams>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let forms = db.list_forms().unwrap_or_default();
    let entries = db
        .list_form_entries_filtered(
            params.form_id,
            params.status.as_deref(),
            params.s.as_deref(),
            300,
        )
        .unwrap_or_default();
    Html(templates::render_forms_view_with_filters(
        &settings,
        &forms,
        &entries,
        params.form_id,
        params.status.as_deref().unwrap_or("all"),
        params.s.as_deref().unwrap_or(""),
    ))
}

#[derive(Debug, Deserialize, Default)]
pub struct FormsActionPayload {
    pub action: Option<String>,
    pub id: Option<i64>,
    pub ids: Option<Vec<i64>>,
    pub status: Option<String>,
    pub is_starred: Option<bool>,
    pub title: Option<String>,
    pub fields: Option<Vec<models::FormFieldSchema>>,
    pub settings: Option<models::FormSettings>,
}

pub async fn handle_api_forms(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<FormsActionPayload>,
) -> impl IntoResponse {
    let action = payload.action.as_deref().unwrap_or("save_form");
    match action {
        "delete_form" => {
            if let Some(id) = payload.id {
                let _ = db.delete_form(id);
            }
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        "delete_entry" => {
            if let Some(id) = payload.id {
                let _ = db.delete_form_entry(id);
            }
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        "update_entry_status" => {
            if let Some(id) = payload.id {
                let _ = db.update_form_entry_workflow(
                    id,
                    payload.status.as_deref(),
                    payload.is_starred,
                );
            }
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        "toggle_entry_star" => {
            if let Some(id) = payload.id {
                let next_star = match payload.is_starred {
                    Some(v) => v,
                    None => db
                        .get_form_entry_by_id(id)
                        .ok()
                        .flatten()
                        .map(|e| !e.is_starred)
                        .unwrap_or(true),
                };
                let _ = db.update_form_entry_workflow(id, None, Some(next_star));
                return (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({ "success": true, "is_starred": next_star })),
                )
                    .into_response();
            }
            (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Missing entry id" })),
            )
                .into_response()
        }
        "bulk_entries" => {
            let ids = payload.ids.unwrap_or_default();
            let target_status = payload.status.as_deref().unwrap_or("read");
            for eid in ids {
                if target_status == "delete" {
                    let _ = db.delete_form_entry(eid);
                } else {
                    let _ = db.update_form_entry_workflow(eid, Some(target_status), None);
                }
            }
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        "retry_email" => {
            let Some(entry_id) = payload.id else {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({ "error": "Missing entry id" })),
                )
                    .into_response();
            };
            let Some(entry) = db.get_form_entry_by_id(entry_id).ok().flatten() else {
                return (
                    StatusCode::NOT_FOUND,
                    axum::Json(serde_json::json!({ "error": "Form entry not found" })),
                )
                    .into_response();
            };
            let form_rec =
                db.get_form(entry.form_id)
                    .ok()
                    .flatten()
                    .unwrap_or(models::DaypressForm {
                        id: entry.form_id,
                        title: entry.form_title.clone(),
                        schema_json: "[]".to_string(),
                        settings_json: "{}".to_string(),
                        created_at: entry.created_at.clone(),
                        updated_at: entry.created_at.clone(),
                    });
            let data_map: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(&entry.data_json).unwrap_or_default();
            let (email_status, email_error, webhook_status) =
                crate::api::daypress_v1::deliver_form_submission_notifications(
                    &db,
                    &form_rec,
                    entry_id,
                    &data_map,
                    &entry.attachments,
                    &entry.source_url,
                )
                .await;
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({
                    "success": email_status == "sent",
                    "entry_id": entry_id,
                    "email_status": email_status,
                    "email_error": email_error,
                    "webhook_status": webhook_status
                })),
            )
                .into_response()
        }
        _ => {
            let req = models::CreateFormRequest {
                id: payload.id,
                title: payload.title.unwrap_or_else(|| "Contact Form".to_string()),
                fields: payload.fields.unwrap_or_default(),
                settings: payload.settings.unwrap_or_default(),
            };
            match db.save_form_record(&req) {
                Ok(form) => (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({ "success": true, "form": form })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
    }
}

pub async fn handle_admin_forms_export_csv(
    State(db): State<Database>,
    Query(params): Query<FormsQueryParams>,
) -> impl IntoResponse {
    let entries = db
        .list_form_entries_filtered(
            params.form_id,
            params.status.as_deref(),
            params.s.as_deref(),
            5000,
        )
        .unwrap_or_default();
    let mut csv = String::from(
        "Entry ID,Form ID,Form Title,Status,Starred,Email Status,Webhook Status,Source URL,Created At,IP Hash,Payload JSON\n",
    );
    for e in entries {
        let escaped_json = e.data_json.replace('"', "\"\"");
        let escaped_title = e.form_title.replace('"', "\"\"");
        let escaped_source = e.source_url.replace('"', "\"\"");
        csv.push_str(&format!(
            "{},{},\"{}\",\"{}\",{},\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            e.id,
            e.form_id,
            escaped_title,
            e.status,
            if e.is_starred { "yes" } else { "no" },
            e.email_status,
            e.webhook_status.as_deref().unwrap_or("none"),
            escaped_source,
            e.created_at,
            e.ip_hash,
            escaped_json
        ));
    }
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_static("attachment; filename=\"daypress-form-entries.csv\""),
    );
    (StatusCode::OK, headers, csv)
}

pub async fn handle_admin_forms_export_json(
    State(db): State<Database>,
    Query(params): Query<FormsQueryParams>,
) -> impl IntoResponse {
    let entries = db
        .list_form_entries_filtered(
            params.form_id,
            params.status.as_deref(),
            params.s.as_deref(),
            5000,
        )
        .unwrap_or_default();
    let body = serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "[]".to_string());
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_static("attachment; filename=\"daypress-form-entries.json\""),
    );
    (StatusCode::OK, headers, body)
}

pub async fn handle_admin_form_attachment_download(
    State(db): State<Database>,
    AxPath((entry_id, filename)): AxPath<(i64, String)>,
) -> impl IntoResponse {
    let Some(safe_name) = crate::media::sanitize_filename(&filename) else {
        return (StatusCode::BAD_REQUEST, "Invalid attachment filename").into_response();
    };
    let Some(entry) = db.get_form_entry_by_id(entry_id).ok().flatten() else {
        return (StatusCode::NOT_FOUND, "Form submission not found").into_response();
    };
    let Some(att) = entry.attachments.iter().find(|a| a.filename == safe_name) else {
        return (StatusCode::NOT_FOUND, "Attachment not found on entry").into_response();
    };
    let path = std::path::Path::new(&att.stored_path);
    let has_traversal = path.components().any(|c| {
        matches!(
            c,
            std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_)
        )
    });
    let in_allowed_dir = path.starts_with("data/form-submissions")
        || path.starts_with("uploads/form-submissions");
    if has_traversal || !in_allowed_dir || !path.exists() {
        return (StatusCode::NOT_FOUND, "Attachment file missing from disk").into_response();
    }
    match std::fs::read(path) {
        Ok(bytes) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::X_CONTENT_TYPE_OPTIONS,
                header::HeaderValue::from_static("nosniff"),
            );
            if let Ok(ct) = att.mime_type.parse() {
                headers.insert(header::CONTENT_TYPE, ct);
            } else {
                headers.insert(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("application/octet-stream"),
                );
            }
            if let Ok(disp) = format!("attachment; filename=\"{}\"", safe_name).parse() {
                headers.insert(header::CONTENT_DISPOSITION, disp);
            }
            (StatusCode::OK, headers, bytes).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read attachment",
        )
            .into_response(),
    }
}

pub async fn handle_admin_backups(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let snapshots = db.list_backup_snapshots().unwrap_or_default();
    Html(templates::render_backups_view(&settings, &snapshots))
}

#[derive(Debug, Deserialize)]
pub struct BackupActionPayload {
    pub action: String,
    pub label: Option<String>,
    pub filename: Option<String>,
}

pub async fn handle_api_backups(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<BackupActionPayload>,
) -> impl IntoResponse {
    match payload.action.as_str() {
        "create" => match db.create_sqlite_snapshot(payload.label.as_deref()) {
            Ok(snap) => (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true, "snapshot": snap })),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        "delete" => {
            let fname = payload.filename.unwrap_or_default();
            match db.delete_backup_snapshot(&fname) {
                Ok(deleted) => (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({ "success": deleted })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        "restore" => {
            let fname = payload.filename.unwrap_or_default();
            match db.restore_sqlite_snapshot(&fname) {
                Ok(_) => (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({
                        "success": true,
                        "message": format!("Snapshot {} restored into live SQLite database.", fname)
                    })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        _ => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Unknown backup action" })),
        )
            .into_response(),
    }
}

pub async fn handle_admin_backup_download(AxPath(filename): AxPath<String>) -> impl IntoResponse {
    let safe_name: String = filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .collect();
    let path = std::path::Path::new("backups").join(&safe_name);
    if !path.exists() {
        return (StatusCode::NOT_FOUND, "Snapshot not found").into_response();
    }
    match std::fs::read(&path) {
        Ok(bytes) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                "application/octet-stream".parse().unwrap(),
            );
            if let Ok(disp) = format!("attachment; filename=\"{}\"", safe_name).parse() {
                headers.insert(header::CONTENT_DISPOSITION, disp);
            }
            (StatusCode::OK, headers, bytes).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read snapshot").into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct PluginSettingsQuery {
    pub plugin: Option<String>,
}

pub async fn handle_admin_plugin_settings(
    State(db): State<Database>,
    Query(params): Query<PluginSettingsQuery>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let slug = params.plugin.unwrap_or_else(|| "whatsapp-chat".to_string());
    let plugin = db.get_installed_plugin(&slug).ok().flatten();
    Html(templates::render_plugin_settings_view(
        &settings,
        plugin.as_ref(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct SavePluginConfigPayload {
    pub slug: String,
    pub config: std::collections::HashMap<String, String>,
}

pub async fn handle_api_plugin_settings_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<SavePluginConfigPayload>,
) -> impl IntoResponse {
    match db.save_plugin_config(&payload.slug, &payload.config) {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true })),
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
pub struct CustomPluginActionPayload {
    pub action: Option<String>,
    pub slug: Option<String>,
    pub manifest: Option<models::PluginManifest>,
}

pub async fn handle_api_plugin_install_custom(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<CustomPluginActionPayload>,
) -> impl IntoResponse {
    if payload.action.as_deref() == Some("uninstall") {
        let slug = payload.slug.unwrap_or_default();
        match db.uninstall_custom_plugin(&slug) {
            Ok(removed) => {
                return (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({ "success": removed })),
                )
                    .into_response()
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response()
            }
        }
    }

    if let Some(manifest) = payload.manifest {
        match db.install_custom_plugin(&manifest, None) {
            Ok(_) => (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true, "slug": manifest.slug })),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Missing plugin manifest" })),
        )
            .into_response()
    }
}

pub async fn handle_api_plugin_sdk_template() -> impl IntoResponse {
    let tpl = crate::plugins::starter_plugin_sdk_template();
    let json = serde_json::to_string_pretty(&tpl).unwrap_or_else(|_| "{}".to_string());
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/json; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"daypress-plugin-starter.json\""
            .parse()
            .unwrap(),
    );
    (StatusCode::OK, headers, json)
}

pub async fn handle_api_download_migrator_plugin() -> impl IntoResponse {
    let zip_bytes = match std::fs::read("daypress-migrator.zip") {
        Ok(b) => b,
        Err(_) => include_bytes!("../../../../../daypress-migrator.zip").to_vec(),
    };
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/zip".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"daypress-migrator.zip\""
            .parse()
            .unwrap(),
    );
    (StatusCode::OK, headers, zip_bytes)
}
