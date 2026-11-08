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

// --- v0.9 Pillar 1: Built-in Newsletters, Member-Gated Posts & Automated Digests ---

#[derive(Debug, Deserialize, Default)]
pub struct SubscribersQueryParams {
    pub status: Option<String>,
    pub list: Option<String>,
    pub search: Option<String>,
}

fn resolve_safe_client_ip(client_ip: Option<&str>, headers: &HeaderMap) -> String {
    if let Some(ip) = client_ip.filter(|s| !s.is_empty() && *s != "127.0.0.1" && *s != "::1") {
        return ip.to_string();
    }
    headers
        .get("x-forwarded-for")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .or(client_ip)
        .unwrap_or("127.0.0.1")
        .to_string()
}

pub(crate) async fn subscribe_newsletter_v1_handler(
    crate::auth::OptionalAuthUser(user): crate::auth::OptionalAuthUser,
    crate::auth::ClientIp(socket_ip): crate::auth::ClientIp,
    State(db): State<Database>,
    headers: HeaderMap,
    Json(mut payload): Json<NewsletterSubscribeRequest>,
) -> Result<(HeaderMap, Json<Value>), AppError> {
    if payload.skip_double_opt_in && !user.as_ref().is_some_and(|u| u.can_edit_content()) {
        payload.skip_double_opt_in = false;
    }
    let ip_raw = resolve_safe_client_ip(socket_ip.as_deref(), &headers);
    if !db.form_rate_limiter.check_and_record(-901, &ip_raw, 30) {
        return Err(AppError::TooManyRequests(
            "Too many subscription requests from this IP. Please try again in a minute."
                .to_string(),
        ));
    }
    let rec = db.subscribe_newsletter_contact(&payload, &ip_raw)?;
    let is_unlocked = rec.status == "subscribed";
    let mut resp_headers = HeaderMap::new();
    if is_unlocked {
        if let Some(ref tok) = rec
            .unsubscribe_token
            .as_ref()
            .or(rec.confirm_token.as_ref())
        {
            let cookie_val = format!(
                "daypress_subscriber={}; Path=/; Max-Age=31536000; SameSite=Lax",
                tok
            );
            if let Ok(hv) = cookie_val.parse() {
                resp_headers.insert(header::SET_COOKIE, hv);
            }
        }
    }
    let msg = if is_unlocked {
        "✓ You are subscribed!"
    } else {
        "✓ Almost done! Please check your email inbox and click the confirmation link."
    };
    let mut safe_rec = rec.clone();
    if !user.as_ref().is_some_and(|u| u.can_edit_content()) {
        safe_rec.confirm_token = None;
        safe_rec.unsubscribe_token = None;
    }
    Ok((
        resp_headers,
        Json(serde_json::json!({
            "status": rec.status,
            "unlocked": is_unlocked,
            "message": msg,
            "subscriber": safe_rec,
        })),
    ))
}

#[derive(Debug, Deserialize, Default)]
pub struct UnlockGatePayload {
    pub email: String,
    #[serde(default)]
    pub source_url: String,
}

pub(crate) async fn unlock_newsletter_gate_v1_handler(
    crate::auth::ClientIp(socket_ip): crate::auth::ClientIp,
    State(db): State<Database>,
    headers: HeaderMap,
    Json(payload): Json<UnlockGatePayload>,
) -> Result<(HeaderMap, Json<Value>), AppError> {
    let ip_raw = resolve_safe_client_ip(socket_ip.as_deref(), &headers);
    if !db.form_rate_limiter.check_and_record(-902, &ip_raw, 30) {
        return Err(AppError::TooManyRequests(
            "Too many unlock attempts from this IP. Please try again in a minute.".to_string(),
        ));
    }
    if let Some(existing) = db.unlock_subscriber_by_email(&payload.email)? {
        let mut resp_headers = HeaderMap::new();
        if let Some(ref tok) = existing
            .unsubscribe_token
            .as_ref()
            .or(existing.confirm_token.as_ref())
        {
            let cookie_val = format!(
                "daypress_subscriber={}; Path=/; Max-Age=31536000; SameSite=Lax",
                tok
            );
            if let Ok(hv) = cookie_val.parse() {
                resp_headers.insert(header::SET_COOKIE, hv);
            }
        }
        return Ok((
            resp_headers,
            Json(serde_json::json!({
                "status": "subscribed",
                "unlocked": true,
                "message": "✓ Welcome back! Unlocking article...",
            })),
        ));
    }

    let rec = db.subscribe_newsletter_contact(
        &NewsletterSubscribeRequest {
            email: payload.email,
            name: String::new(),
            lists: vec!["general".to_string()],
            source_url: payload.source_url,
            skip_double_opt_in: false,
        },
        &ip_raw,
    )?;
    let is_unlocked = rec.status == "subscribed";
    let mut resp_headers = HeaderMap::new();
    if is_unlocked {
        if let Some(ref tok) = rec
            .unsubscribe_token
            .as_ref()
            .or(rec.confirm_token.as_ref())
        {
            let cookie_val = format!(
                "daypress_subscriber={}; Path=/; Max-Age=31536000; SameSite=Lax",
                tok
            );
            if let Ok(hv) = cookie_val.parse() {
                resp_headers.insert(header::SET_COOKIE, hv);
            }
        }
    }
    let msg = if is_unlocked {
        "✓ Subscribed! Unlocking article..."
    } else {
        "✓ Check your inbox and click the confirmation link to unlock this article."
    };
    Ok((
        resp_headers,
        Json(serde_json::json!({
            "status": rec.status,
            "unlocked": is_unlocked,
            "message": msg,
        })),
    ))
}

pub(crate) async fn list_newsletter_subscribers_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Query(params): Query<SubscribersQueryParams>,
) -> Result<Json<Value>, AppError> {
    let subs = db.list_subscribers(
        params.status.as_deref(),
        params.list.as_deref(),
        params.search.as_deref(),
        true,
    )?;
    let stats = db.get_newsletter_stats()?;
    Ok(Json(serde_json::json!({
        "subscribers": subs,
        "stats": stats,
    })))
}

#[derive(Debug, Deserialize, Default)]
pub struct UpdateSubscriberStatusPayload {
    pub status: String,
}

pub(crate) async fn update_newsletter_subscriber_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdateSubscriberStatusPayload>,
) -> Result<Json<Value>, AppError> {
    let updated = db.update_subscriber_status(id, &payload.status)?;
    if !updated {
        return Err(AppError::NotFound(format!("Subscriber {} not found", id)));
    }
    Ok(Json(serde_json::json!({
        "id": id,
        "status": payload.status,
        "updated": true,
    })))
}

pub(crate) async fn delete_newsletter_subscriber_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    let deleted = db.delete_subscriber(id)?;
    if !deleted {
        return Err(AppError::NotFound(format!("Subscriber {} not found", id)));
    }
    Ok(Json(serde_json::json!({
        "id": id,
        "deleted": true,
    })))
}

pub(crate) async fn list_newsletter_campaigns_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
) -> Result<Json<Vec<NewsletterCampaignRecord>>, AppError> {
    let campaigns = db.list_newsletter_campaigns()?;
    Ok(Json(campaigns))
}

pub(crate) async fn create_newsletter_campaign_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Json(payload): Json<CreateNewsletterCampaignRequest>,
) -> Result<(StatusCode, Json<NewsletterCampaignRecord>), AppError> {
    let campaign = db.create_and_send_campaign(&payload, "broadcast", None)?;
    Ok((StatusCode::CREATED, Json(campaign)))
}

#[derive(Debug, Deserialize, Default)]
pub struct SaveNewsletterSettingsPayload {
    pub double_opt_in: Option<bool>,
    pub auto_digest: Option<bool>,
    pub email_transport: Option<String>,
    pub resend_api_key: Option<String>,
    pub ses_region: Option<String>,
}

pub(crate) async fn save_newsletter_settings_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Json(payload): Json<SaveNewsletterSettingsPayload>,
) -> Result<Json<Value>, AppError> {
    if let Some(doi) = payload.double_opt_in {
        db.set_option(
            "daypress_newsletter_double_opt_in",
            if doi { "1" } else { "0" },
        )?;
    }
    if let Some(ad) = payload.auto_digest {
        db.set_option(
            "daypress_newsletter_auto_digest",
            if ad { "1" } else { "0" },
        )?;
    }
    if let Some(ref tr) = payload.email_transport {
        db.set_option("daypress_email_transport", tr.trim())?;
    }
    if let Some(ref key) = payload.resend_api_key {
        if !key.trim().is_empty() {
            db.set_option("daypress_resend_api_key", key.trim())?;
        }
    }
    if let Some(ref reg) = payload.ses_region {
        if !reg.trim().is_empty() {
            db.set_option("daypress_ses_region", reg.trim())?;
        }
    }
    Ok(Json(serde_json::json!({
        "saved": true,
        "double_opt_in": db.is_newsletter_double_opt_in_enabled(),
        "auto_digest": db.is_newsletter_auto_digest_enabled(),
    })))
}

// --- Native Subsystems: Branded Shortlinks (`/go/*`) & HMAC-Signed Outbound Webhooks (v0.9) ---

pub(crate) async fn list_shortlinks_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
) -> Result<Json<Vec<ShortlinkRecord>>, AppError> {
    let links = db.list_shortlinks()?;
    Ok(Json(links))
}

pub(crate) async fn save_shortlink_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Json(payload): Json<CreateShortlinkRequest>,
) -> Result<(StatusCode, Json<ShortlinkRecord>), AppError> {
    let rec = db.save_shortlink(&payload)?;
    Ok((StatusCode::CREATED, Json(rec)))
}

pub(crate) async fn delete_shortlink_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    let deleted = db.delete_shortlink(id)?;
    Ok(Json(serde_json::json!({ "deleted": deleted, "id": id })))
}

pub(crate) async fn list_webhooks_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
) -> Result<Json<Vec<WebhookEndpointRecord>>, AppError> {
    let hooks = db.list_webhook_endpoints()?;
    Ok(Json(hooks))
}

pub(crate) async fn save_webhook_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(payload): Json<CreateWebhookEndpointRequest>,
) -> Result<(StatusCode, Json<WebhookEndpointRecord>), AppError> {
    let rec = db.save_webhook_endpoint(&payload)?;
    Ok((StatusCode::CREATED, Json(rec)))
}

pub(crate) async fn delete_webhook_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    let deleted = db.delete_webhook_endpoint(id)?;
    Ok(Json(serde_json::json!({ "deleted": deleted, "id": id })))
}

#[derive(Debug, Deserialize, Default)]
pub struct WebhookDeliveriesQueryParams {
    pub webhook_id: Option<i64>,
    pub limit: Option<usize>,
}

pub(crate) async fn list_webhook_deliveries_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Query(params): Query<WebhookDeliveriesQueryParams>,
) -> Result<Json<Vec<WebhookDeliveryRecord>>, AppError> {
    let deliveries = db.list_webhook_deliveries(params.webhook_id, params.limit.unwrap_or(50))?;
    Ok(Json(deliveries))
}

pub(crate) async fn replay_webhook_delivery_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<WebhookDeliveryRecord>, AppError> {
    let rec = db.replay_webhook_delivery(id).await?;
    Ok(Json(rec))
}

pub(crate) async fn test_webhook_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(payload): Json<Value>,
) -> Result<Json<Vec<WebhookDeliveryRecord>>, AppError> {
    let event_type = payload
        .get("event_type")
        .and_then(|v| v.as_str())
        .unwrap_or("post.published");
    let deliveries = db.dispatch_outbound_webhook_event(
        event_type,
        serde_json::json!({
            "test": true,
            "message": "Daypress test webhook dispatch",
            "title": "Sample Daypress Post",
        }),
    )?;
    Ok(Json(deliveries))
}

// --- Native Subsystems: Zero-Flicker Server-Side A/B Split Testing & Core Web Vitals (`LCP` / `INP` / `CLS`) (v0.9) ---

#[derive(Debug, Deserialize, Default)]
pub struct AbTestsQueryParams {
    pub post_id: Option<i64>,
}

pub(crate) async fn list_ab_tests_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Query(params): Query<AbTestsQueryParams>,
) -> Result<Json<Vec<AbTestRecord>>, AppError> {
    let tests = db.list_ab_tests(params.post_id)?;
    Ok(Json(tests))
}

pub(crate) async fn save_ab_test_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Json(payload): Json<CreateAbTestRequest>,
) -> Result<(StatusCode, Json<AbTestRecord>), AppError> {
    let rec = db.save_ab_test(&payload)?;
    Ok((StatusCode::CREATED, Json(rec)))
}

pub(crate) async fn delete_ab_test_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    let deleted = db.delete_ab_test(id)?;
    Ok(Json(serde_json::json!({ "deleted": deleted, "id": id })))
}

#[derive(Debug, Deserialize, Default)]
pub struct PromoteAbWinnerPayload {
    pub winner: Option<String>,
}

pub(crate) async fn promote_ab_test_winner_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    payload: Option<Json<PromoteAbWinnerPayload>>,
) -> Result<Json<AbTestRecord>, AppError> {
    let winner = payload
        .and_then(|Json(p)| p.winner)
        .unwrap_or_else(|| "B".to_string());
    let rec = db.promote_ab_test_winner(id, &winner)?;
    Ok(Json(rec))
}

pub(crate) async fn record_web_vitals_v1_handler(
    crate::auth::ClientIp(socket_ip): crate::auth::ClientIp,
    State(db): State<Database>,
    headers: HeaderMap,
    Json(payload): Json<WebVitalsBeaconRequest>,
) -> Result<Json<Value>, AppError> {
    let ip_raw = resolve_safe_client_ip(socket_ip.as_deref(), &headers);
    if !db.form_rate_limiter.check_and_record(-903, &ip_raw, 120) {
        return Err(AppError::TooManyRequests(
            "Too many Web Vitals beacons from your IP address.".to_string(),
        ));
    }
    db.record_web_vitals_beacon(&payload)?;
    Ok(Json(serde_json::json!({ "recorded": true })))
}

pub(crate) async fn get_web_vitals_summary_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
) -> Result<Json<CoreWebVitalsSummary>, AppError> {
    let summary = db.get_core_web_vitals_summary()?;
    Ok(Json(summary))
}

// --- Native Subsystems: Sandboxed WebAssembly (`.wasm`) Extension Crates (v0.9) ---

pub(crate) async fn list_wasm_extensions_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
) -> Result<Json<Vec<WasmExtensionRecord>>, AppError> {
    let exts = db.list_wasm_extensions()?;
    Ok(Json(exts))
}

pub(crate) async fn install_wasm_extension_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(payload): Json<InstallWasmExtensionRequest>,
) -> Result<(StatusCode, Json<WasmExtensionRecord>), AppError> {
    let rec = db.install_or_update_wasm_extension(&payload)?;
    Ok((StatusCode::CREATED, Json(rec)))
}

pub(crate) async fn delete_wasm_extension_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
) -> Result<Json<Value>, AppError> {
    let deleted = db.delete_wasm_extension(&slug)?;
    Ok(Json(
        serde_json::json!({ "deleted": deleted, "slug": slug }),
    ))
}

#[derive(Debug, Deserialize, Default)]
pub struct ExecuteWasmExtensionPayload {
    pub export_name: Option<String>,
    #[serde(default)]
    pub attrs: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub content: String,
}

pub(crate) async fn execute_wasm_extension_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
    Json(payload): Json<ExecuteWasmExtensionPayload>,
) -> Result<Json<daypress_wasm::WasmExecutionOutput>, AppError> {
    let export_name = payload.export_name.as_deref().unwrap_or("render_shortcode");
    let out = db.execute_wasm_extension(&slug, export_name, &payload.attrs, &payload.content)?;
    Ok(Json(out))
}
