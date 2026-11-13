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

// --- Native Subsystems: Cookieless Daily Salted Analytics ---

#[derive(Debug, Deserialize, Default)]
pub struct AnalyticsRangeQuery {
    pub range: Option<String>,
    pub page: Option<String>,
    pub referrer: Option<String>,
    pub channel: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub device: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub country: Option<String>,
    pub goal: Option<String>,
}

impl AnalyticsRangeQuery {
    pub fn to_filter(&self) -> AnalyticsFilter {
        AnalyticsFilter {
            page: self.page.clone(),
            referrer: self.referrer.clone(),
            channel: self.channel.clone(),
            utm_source: self.utm_source.clone(),
            utm_medium: self.utm_medium.clone(),
            utm_campaign: self.utm_campaign.clone(),
            device: self.device.clone(),
            browser: self.browser.clone(),
            os: self.os.clone(),
            country: self.country.clone(),
            goal: self.goal.clone(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveAnalyticsSettingsPayload {
    #[serde(default = "crate::models::default_analytics_retention_days")]
    pub retention_days: i64,
    #[serde(default = "crate::models::default_true_bool")]
    pub exclude_admins: bool,
    #[serde(default = "crate::models::default_true_bool")]
    pub filter_bots: bool,
    #[serde(default = "crate::models::default_true_bool")]
    pub honor_dnt_gpc: bool,
}

pub(crate) fn should_ignore_telemetry_request(
    db: &Database,
    headers: &HeaderMap,
    ua: &str,
) -> Option<&'static str> {
    use crate::adapters::database::analytics::{
        has_dnt_or_gpc, is_bot_or_crawler, is_prefetch_or_prerender,
    };

    if is_prefetch_or_prerender(headers) {
        return Some("prefetch");
    }
    let cfg = db.get_analytics_telemetry_settings().unwrap_or_default();
    if cfg.honor_dnt_gpc && has_dnt_or_gpc(headers) {
        return Some("dnt_gpc");
    }
    if cfg.filter_bots && is_bot_or_crawler(ua) {
        return Some("bot");
    }
    if cfg.exclude_admins {
        if let Some(cookie_hdr) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) {
            if let Some(user) = crate::auth::session_user_from_cookie_header(cookie_hdr, db) {
                if user.is_editor_or_above() {
                    return Some("staff");
                }
            }
        }
    }
    None
}

pub(crate) async fn record_analytics_event_handler(
    State(db): State<Database>,
    request: Request,
) -> Result<Json<Value>, AppError> {
    use crate::adapters::database::analytics::parse_ua_telemetry;

    let (parts, body) = request.into_parts();
    let bytes = to_bytes(body, 64 * 1024)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let payload: AnalyticsEventRequest = serde_json::from_slice(&bytes)
        .map_err(|e| AppError::BadRequest(format!("Invalid analytics payload: {}", e)))?;

    let ua = parts
        .headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    if let Some(reason) = should_ignore_telemetry_request(&db, &parts.headers, ua) {
        return Ok(Json(serde_json::json!({
            "recorded": false,
            "ignored": reason
        })));
    }

    let peer_ip = crate::auth::client_ip(&parts);
    let visitor_hash = db.compute_daily_visitor_hash(&parts.headers, peer_ip.as_deref());
    let country = extract_country_from_headers(&parts.headers);
    let req_host = parts
        .headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok());

    let (browser, os, fallback_device) = parse_ua_telemetry(ua);
    let referrer = payload.referrer.unwrap_or_default();
    let device_type = payload
        .device_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or(fallback_device);
    let event_type = if payload.event_type.trim().is_empty() {
        "pageview"
    } else {
        payload.event_type.trim()
    };

    db.record_analytics_beacon(
        &visitor_hash,
        &payload.page_url,
        &referrer,
        req_host,
        device_type,
        browser,
        os,
        &country,
        payload.utm_source.as_deref(),
        payload.utm_medium.as_deref(),
        payload.utm_campaign.as_deref(),
        event_type,
        payload.goal_name.as_deref(),
        payload.duration_secs,
        payload.scroll_depth,
    )?;

    Ok(Json(serde_json::json!({ "recorded": true })))
}

pub(crate) async fn get_analytics_summary_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Query(query): Query<AnalyticsRangeQuery>,
) -> Result<Json<AnalyticsSummary>, AppError> {
    let range = query.range.as_deref().unwrap_or("30d");
    let summary = db.get_analytics_dashboard_filtered(range, &query.to_filter())?;
    Ok(Json(summary))
}

pub(crate) async fn get_analytics_settings_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
) -> Result<Json<AnalyticsTelemetrySettings>, AppError> {
    let cfg = db.get_analytics_telemetry_settings()?;
    Ok(Json(cfg))
}

pub(crate) async fn save_analytics_settings_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(payload): Json<SaveAnalyticsSettingsPayload>,
) -> Result<Json<Value>, AppError> {
    let (settings, pruned_rows) = db.update_analytics_telemetry_settings(
        payload.retention_days,
        payload.exclude_admins,
        payload.filter_bots,
        payload.honor_dnt_gpc,
    )?;
    Ok(Json(serde_json::json!({
        "saved": true,
        "pruned_rows": pruned_rows,
        "settings": settings,
    })))
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
    let mut archive_filename: Option<String> = None;
    let mut expected_sha256: Option<String> = None;
    let mut expected_signature: Option<String> = None;
    let mut target_version: Option<String> = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(format!("Invalid multipart upload: {e}")))?
    {
        let name = field.name().unwrap_or("").to_string();
        match name.as_str() {
            "archive" | "file" => {
                if let Some(fname) = field.file_name().map(str::to_string) {
                    if target_version.is_none() {
                        // Extract version tag from `daypress-v0.2.0-...` if present
                        if let Some(rest) = fname.strip_prefix("daypress-v") {
                            if let Some(ver) = rest.split('-').next() {
                                target_version = Some(ver.to_string());
                            }
                        }
                    }
                    archive_filename = Some(fname);
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
            "signature" => {
                let txt = field.text().await.unwrap_or_default();
                if !txt.trim().is_empty() {
                    expected_signature = Some(txt.trim().to_string());
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

    let cached_state = crate::updater::load_update_state(&db);
    let fallback_asset_name = archive_filename
        .as_deref()
        .or(cached_state.asset_name.as_deref())
        .unwrap_or("");
    let proof = if let Some(sig) = expected_signature.as_deref() {
        Some(crate::updater::UpdateSignatureProof::ArchiveSignature(sig))
    } else if let Some(sig) = cached_state.asset_signature.as_deref() {
        Some(crate::updater::UpdateSignatureProof::ArchiveSignature(sig))
    } else if let (Some(sums), Some(sig)) = (
        cached_state.sha256sums_raw.as_deref(),
        cached_state.signature.as_deref(),
    ) {
        Some(crate::updater::UpdateSignatureProof::SignedSumsManifest {
            sums_text: sums,
            sums_signature: sig,
            asset_name: fallback_asset_name,
        })
    } else {
        None
    };
    let report = crate::updater::apply_binary_update_to_path(
        &db,
        &exe_path,
        &bytes,
        expected_sha256.as_deref(),
        proof,
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
