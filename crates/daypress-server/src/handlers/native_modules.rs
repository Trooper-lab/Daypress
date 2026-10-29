//! First-Party Modules (Cookieless Analytics, SEO & Redirects, Forms, Hot Backups, Plugin SDK).

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

// =========================================================================
// NATIVE COOKIELESS ANALYTICS DASHBOARD HANDLER
// =========================================================================

pub async fn handle_admin_analytics(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let summary = db
        .get_analytics_summary()
        .unwrap_or(models::AnalyticsSummary {
            total_visitors: 0,
            total_pageviews: 0,
            top_pages: vec![],
        });

    let mut top_pages_rows = String::new();
    for p in &summary.top_pages {
        top_pages_rows.push_str(&format!(
            r#"<tr>
                <td style="padding: 12px; font-family: var(--font-mono); font-size: 13px;"><a href="{url}" target="_blank" style="color: var(--press); text-decoration: none;">{url} ↗</a></td>
                <td style="padding: 12px; font-weight: 700; text-align: right;">{views}</td>
            </tr>"#,
            url = p.page_url,
            views = p.views
        ));
    }

    if summary.top_pages.is_empty() {
        top_pages_rows = r#"<tr><td colspan="2" style="padding: 24px; text-align: center; color: var(--ink-muted);">No visitor events recorded yet. Browse public pages to generate cookieless telemetry.</td></tr>"#.to_string();
    }

    let content = format!(
        r##"
        <div class="page-header" style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 24px;">
            <div>
                <div style="display: flex; align-items: center; gap: 10px;">
                    <h1 class="page-title">Cookieless Analytics</h1>
                    <span class="badge badge-published" style="font-size: 11px;">GDPR / ePrivacy Native</span>
                </div>
                <p class="page-subtitle">Sub-millisecond visitor metrics via daily-rotating salted SHA-256 hashes. Zero cookies, zero banners.</p>
            </div>
            <button class="btn btn-secondary" onclick="location.reload()">Refresh Data ↻</button>
        </div>

        <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; margin-bottom: 30px;">
            <div class="day-card" style="padding: 20px;">
                <div style="font-size: 13px; color: var(--ink-muted); font-weight: 600; text-transform: uppercase;">Unique Daily Visitors</div>
                <div style="font-size: 36px; font-weight: 800; color: var(--ink); margin-top: 8px;">{visitors}</div>
                <div style="font-size: 12px; color: var(--moss); margin-top: 4px;">✓ Cryptographically Salted</div>
            </div>
            <div class="day-card" style="padding: 20px;">
                <div style="font-size: 13px; color: var(--ink-muted); font-weight: 600; text-transform: uppercase;">Total Pageviews</div>
                <div style="font-size: 36px; font-weight: 800; color: var(--press); margin-top: 8px;">{pageviews}</div>
                <div style="font-size: 12px; color: var(--ink-muted); margin-top: 4px;">Direct SQLite Ingest</div>
            </div>
            <div class="day-card" style="padding: 20px;">
                <div style="font-size: 13px; color: var(--ink-muted); font-weight: 600; text-transform: uppercase;">Tracking Footprint</div>
                <div style="font-size: 36px; font-weight: 800; color: var(--moss); margin-top: 8px;">0 Bytes</div>
                <div style="font-size: 12px; color: var(--moss); margin-top: 4px;">0 Cookies • 0 LocalStorage</div>
            </div>
        </div>

        <div class="day-card" style="padding: 24px;">
            <h2 style="font-size: 16px; font-weight: 700; margin-top: 0; margin-bottom: 16px; color: var(--ink);">Top Visited URLs</h2>
            <table style="width: 100%; border-collapse: collapse;">
                <thead>
                    <tr style="border-bottom: 2px solid var(--rule); text-align: left; font-size: 12px; color: var(--ink-muted); text-transform: uppercase;">
                        <th style="padding: 8px 12px;">Page Path</th>
                        <th style="padding: 8px 12px; text-align: right;">Views</th>
                    </tr>
                </thead>
                <tbody>
                    {rows}
                </tbody>
            </table>
        </div>
        "##,
        visitors = summary.total_visitors,
        pageviews = summary.total_pageviews,
        rows = top_pages_rows
    );

    Html(templates::render_wp_admin_shell(
        "analytics",
        "Cookieless Analytics",
        &content,
        &settings,
    ))
}

// =========================================================================
// FIRST-PARTY MODULES & COMMUNITY PLUGIN ECOSYSTEM HANDLERS
// =========================================================================

pub async fn handle_admin_seo(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let seo = db.get_global_seo_settings().unwrap_or_default();
    let redirects = db.list_redirects().unwrap_or_default();
    let logs_404 = db.list_404_logs(100).unwrap_or_default();
    Html(templates::render_wp_seo_view(
        &settings, &seo, &redirects, &logs_404,
    ))
}

pub async fn handle_api_seo_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::GlobalSeoSettings>,
) -> impl IntoResponse {
    match db.save_global_seo_settings(&payload) {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "seo": payload })),
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
pub struct RedirectActionPayload {
    pub action: Option<String>,
    pub id: Option<i64>,
    pub source_path: Option<String>,
    pub target_url: Option<String>,
    pub status_code: Option<i64>,
}

pub async fn handle_api_redirects(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<RedirectActionPayload>,
) -> impl IntoResponse {
    let action = payload.action.as_deref().unwrap_or("create");
    match action {
        "delete" => {
            if let Some(id) = payload.id {
                let _ = db.delete_redirect(id);
            }
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        "clear_404" => {
            let _ = db.clear_404_logs();
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        _ => {
            let req = models::CreateRedirectRequest {
                source_path: payload.source_path.unwrap_or_default(),
                target_url: payload.target_url.unwrap_or_else(|| "/".to_string()),
                status_code: payload.status_code.unwrap_or(301),
            };
            match db.create_redirect(&req) {
                Ok(rule) => (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({ "success": true, "redirect": rule })),
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

#[derive(Debug, Deserialize)]
pub struct FormsQueryParams {
    pub form_id: Option<i64>,
}

pub async fn handle_admin_forms(
    State(db): State<Database>,
    Query(params): Query<FormsQueryParams>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let forms = db.list_forms().unwrap_or_default();
    let entries = db
        .list_form_entries(params.form_id, 200)
        .unwrap_or_default();
    Html(templates::render_wp_forms_view(
        &settings,
        &forms,
        &entries,
        params.form_id,
    ))
}

#[derive(Debug, Deserialize)]
pub struct FormsActionPayload {
    pub action: Option<String>,
    pub id: Option<i64>,
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
        .list_form_entries(params.form_id, 5000)
        .unwrap_or_default();
    let mut csv = String::from("Entry ID,Form ID,Form Title,Created At,IP Hash,Payload JSON\n");
    for e in entries {
        let escaped_json = e.data_json.replace('"', "\"\"");
        let escaped_title = e.form_title.replace('"', "\"\"");
        csv.push_str(&format!(
            "{},{},\"{}\",\"{}\",\"{}\",\"{}\"\n",
            e.id, e.form_id, escaped_title, e.created_at, e.ip_hash, escaped_json
        ));
    }
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "text/csv; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"daypress-form-entries.csv\""
            .parse()
            .unwrap(),
    );
    (StatusCode::OK, headers, csv)
}

pub async fn handle_admin_backups(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let snapshots = db.list_backup_snapshots().unwrap_or_default();
    Html(templates::render_wp_backups_view(&settings, &snapshots))
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
    Html(templates::render_wp_plugin_settings_view(
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
        Err(_) => include_bytes!("../../../../daypress-migrator.zip").to_vec(),
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
