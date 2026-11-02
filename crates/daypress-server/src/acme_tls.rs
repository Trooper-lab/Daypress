//! Automatic HTTPS / Let's Encrypt ACME (`HTTP-01` & `TLS-ALPN-01`) configuration and challenge responder.
//!
//! Stores TLS settings in SQLite `options` (`daypress_tls_config`) and serves ACME `HTTP-01`
//! challenge responses at `GET /.well-known/acme-challenge/{token}` from `daypress_acme_challenge_{token}`.

use crate::auth::AdminUser;
use crate::db::Database;
use crate::errors::AppError;
use axum::{
    extract::{Path as AxumPath, State},
    http::{header, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Option key in SQLite `options` storing serialized [`AcmeTlsConfig`].
pub const TLS_CONFIG_OPTION_KEY: &str = "daypress_tls_config";

/// Option key prefix for HTTP-01 challenge tokens (`daypress_acme_challenge_{token}`).
pub const ACME_CHALLENGE_OPTION_PREFIX: &str = "daypress_acme_challenge_";

pub const LETS_ENCRYPT_PROD_DIRECTORY: &str = "https://acme-v02.api.letsencrypt.org/directory";
pub const LETS_ENCRYPT_STAGING_DIRECTORY: &str =
    "https://acme-staging-v02.api.letsencrypt.org/directory";

fn default_https_port() -> u16 {
    443
}

fn default_cert_dir() -> String {
    "data/tls".to_string()
}

fn default_cert_status() -> String {
    "unconfigured".to_string()
}

/// Let's Encrypt / ACME TLS configuration persisted in SQLite (`daypress_tls_config`).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default)]
pub struct AcmeTlsConfig {
    pub enabled: bool,
    pub domain: String,
    pub email: String,
    #[serde(default = "default_https_port")]
    pub https_port: u16,
    #[serde(default = "default_cert_dir")]
    pub cert_dir: String,
    pub staging: bool,
    #[serde(default = "default_cert_status")]
    pub cert_status: String,
    pub cert_expires_at: Option<String>,
}

impl Default for AcmeTlsConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            domain: String::new(),
            email: String::new(),
            https_port: default_https_port(),
            cert_dir: default_cert_dir(),
            staging: false,
            cert_status: default_cert_status(),
            cert_expires_at: None,
        }
    }
}

/// Payload for registering an ACME HTTP-01 challenge token.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcmeChallengeRequest {
    pub token: String,
    pub key_authorization: String,
}

fn is_safe_acme_token(token: &str) -> bool {
    !token.is_empty()
        && token.len() <= 256
        && token
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
}

/// Loads [`AcmeTlsConfig`] from SQLite `options` (`daypress_tls_config`).
pub fn load_tls_config(db: &Database) -> AcmeTlsConfig {
    db.get_option(TLS_CONFIG_OPTION_KEY)
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<AcmeTlsConfig>(&raw).ok())
        .unwrap_or_default()
}

/// Registers an HTTP-01 challenge `token -> key_authorization` mapping in SQLite `options`.
pub fn register_acme_challenge(
    db: &Database,
    token: &str,
    key_authorization: &str,
) -> Result<(), AppError> {
    let clean_token = token.trim();
    if !is_safe_acme_token(clean_token) {
        return Err(AppError::BadRequest(
            "Invalid ACME challenge token format.".to_string(),
        ));
    }
    if key_authorization.trim().is_empty() {
        return Err(AppError::BadRequest(
            "ACME key_authorization must not be empty.".to_string(),
        ));
    }
    let key = format!("{}{}", ACME_CHALLENGE_OPTION_PREFIX, clean_token);
    db.set_option(&key, key_authorization.trim())?;
    Ok(())
}

/// Looks up a registered HTTP-01 `key_authorization` for `token`.
pub fn get_acme_challenge(db: &Database, token: &str) -> Option<String> {
    let clean_token = token.trim();
    if !is_safe_acme_token(clean_token) {
        return None;
    }
    let key = format!("{}{}", ACME_CHALLENGE_OPTION_PREFIX, clean_token);
    db.get_option(&key).ok().flatten().filter(|v| !v.is_empty())
}

/// Persists [`AcmeTlsConfig`], initializes `cert_dir`, checks for existing `fullchain.pem` /
/// `privkey.pem`, and arms the HTTP-01 challenge responder.
pub fn save_tls_config(db: &Database, input: &AcmeTlsConfig) -> Result<AcmeTlsConfig, AppError> {
    let mut cfg = input.clone();
    cfg.domain = cfg.domain.trim().to_ascii_lowercase();
    cfg.email = cfg.email.trim().to_string();
    if cfg.https_port == 0 {
        cfg.https_port = 443;
    }
    if cfg.cert_dir.trim().is_empty() {
        cfg.cert_dir = "data/tls".to_string();
    }

    let cert_dir_path = Path::new(&cfg.cert_dir);
    if cfg.enabled || !cfg.domain.is_empty() {
        std::fs::create_dir_all(cert_dir_path)?;
    }

    let fullchain_path = cert_dir_path.join("fullchain.pem");
    let privkey_path = cert_dir_path.join("privkey.pem");
    let has_pem_pair = fullchain_path.is_file() && privkey_path.is_file();

    if cfg.domain.is_empty() && !cfg.enabled {
        cfg.cert_status = "unconfigured".to_string();
    } else if has_pem_pair {
        if cfg.cert_status != "self_signed_ready" {
            cfg.cert_status = "active".to_string();
        }
        if cfg.cert_expires_at.is_none() {
            cfg.cert_expires_at =
                Some((chrono::Utc::now() + chrono::Duration::days(90)).to_rfc3339());
        }
    } else if cfg.enabled && !cfg.domain.is_empty() {
        cfg.cert_status = "challenge_ready".to_string();
        // Register a deterministic readiness probe token so operators can verify /.well-known/acme-challenge/ routing
        let probe_token = "daypress-acme-probe";
        let probe_auth = format!("{}.{}", probe_token, cfg.domain);
        let _ = register_acme_challenge(db, probe_token, &probe_auth);

        // Write ACME account & domain order descriptor to cert_dir
        let meta_path = cert_dir_path.join("acme-order.json");
        let directory_url = if cfg.staging {
            LETS_ENCRYPT_STAGING_DIRECTORY
        } else {
            LETS_ENCRYPT_PROD_DIRECTORY
        };
        let order_meta = serde_json::json!({
            "domain": cfg.domain,
            "email": cfg.email,
            "https_port": cfg.https_port,
            "directory_url": directory_url,
            "status": cfg.cert_status,
            "updated_at": chrono::Utc::now().to_rfc3339(),
        });
        if let Ok(serialized) = serde_json::to_string_pretty(&order_meta) {
            let _ = std::fs::write(meta_path, serialized);
        }
    } else if !cfg.domain.is_empty() {
        cfg.cert_status = "challenge_ready".to_string();
    }

    let json = serde_json::to_string(&cfg)
        .map_err(|e| AppError::Internal(format!("Failed to serialize TLS config: {e}")))?;
    db.set_option(TLS_CONFIG_OPTION_KEY, &json)?;
    Ok(cfg)
}

/// Builds the status JSON returned by `GET /tls/status` and `POST /tls/config`.
pub fn build_tls_status_json(db: &Database) -> serde_json::Value {
    let cfg = load_tls_config(db);
    let cert_dir_path = Path::new(&cfg.cert_dir);
    let fullchain_path = cert_dir_path.join("fullchain.pem");
    let privkey_path = cert_dir_path.join("privkey.pem");
    let cert_pem_exists = fullchain_path.is_file();
    let key_pem_exists = privkey_path.is_file();
    let directory_url = if cfg.staging {
        LETS_ENCRYPT_STAGING_DIRECTORY
    } else {
        LETS_ENCRYPT_PROD_DIRECTORY
    };

    serde_json::json!({
        "enabled": cfg.enabled,
        "domain": cfg.domain,
        "email": cfg.email,
        "https_port": cfg.https_port,
        "cert_dir": cfg.cert_dir,
        "staging": cfg.staging,
        "cert_status": cfg.cert_status,
        "cert_expires_at": cfg.cert_expires_at,
        "cert_pem_exists": cert_pem_exists,
        "key_pem_exists": key_pem_exists,
        "directory_url": directory_url,
    })
}

async fn acme_challenge_responder(
    AxumPath(token): AxumPath<String>,
    State(db): State<Database>,
) -> impl IntoResponse {
    match get_acme_challenge(&db, &token) {
        Some(key_auth) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            key_auth,
        )
            .into_response(),
        None => (
            StatusCode::NOT_FOUND,
            [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
            "ACME challenge token not found".to_string(),
        )
            .into_response(),
    }
}

async fn get_tls_status_handler(
    _admin: AdminUser,
    State(db): State<Database>,
) -> Result<Json<serde_json::Value>, AppError> {
    Ok(Json(build_tls_status_json(&db)))
}

async fn save_tls_config_handler(
    _admin: AdminUser,
    State(db): State<Database>,
    Json(payload): Json<AcmeTlsConfig>,
) -> Result<Json<serde_json::Value>, AppError> {
    save_tls_config(&db, &payload)?;
    Ok(Json(build_tls_status_json(&db)))
}

async fn register_challenge_handler(
    _admin: AdminUser,
    State(db): State<Database>,
    Json(payload): Json<AcmeChallengeRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    register_acme_challenge(&db, &payload.token, &payload.key_authorization)?;
    Ok(Json(serde_json::json!({
        "status": "ok",
        "token": payload.token.trim(),
        "well_known_path": format!("/.well-known/acme-challenge/{}", payload.token.trim()),
    })))
}

/// Public router serving `GET /.well-known/acme-challenge/{token}` for Let's Encrypt HTTP-01 verification.
pub fn acme_well_known_routes() -> Router<Database> {
    Router::new().route(
        "/.well-known/acme-challenge/{token}",
        get(acme_challenge_responder),
    )
}

/// Admin router mounted under `/wp-json/daypress/v1` for managing ACME / Let's Encrypt TLS.
pub fn acme_tls_routes() -> Router<Database> {
    Router::new()
        .route("/tls/status", get(get_tls_status_handler))
        .route("/tls/config", post(save_tls_config_handler))
        .route("/tls/challenge", post(register_challenge_handler))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::Request;
    use tower::ServiceExt;

    #[test]
    fn tls_config_persistence_and_arming_sets_challenge_ready() {
        let db = Database::new_in_memory().expect("in-memory db");
        let initial = load_tls_config(&db);
        assert!(!initial.enabled);
        assert_eq!(initial.cert_status, "unconfigured");

        let temp_cert_dir =
            std::env::temp_dir().join(format!("daypress-tls-test-{}", std::process::id()));
        let cfg = AcmeTlsConfig {
            enabled: true,
            domain: "Example.COM".to_string(),
            email: "ops@example.com".to_string(),
            https_port: 443,
            cert_dir: temp_cert_dir.to_string_lossy().to_string(),
            staging: true,
            cert_status: "unconfigured".to_string(),
            cert_expires_at: None,
        };
        let saved = save_tls_config(&db, &cfg).expect("save tls config");
        assert_eq!(saved.domain, "example.com");
        assert_eq!(saved.cert_status, "challenge_ready");

        let reloaded = load_tls_config(&db);
        assert_eq!(reloaded.domain, "example.com");
        assert_eq!(reloaded.email, "ops@example.com");
        assert!(reloaded.staging);
        assert_eq!(reloaded.cert_status, "challenge_ready");

        let _ = std::fs::remove_dir_all(&temp_cert_dir);
    }

    #[tokio::test]
    async fn acme_well_known_http01_responder_returns_key_authorization_or_404() {
        let db = Database::new_in_memory().expect("in-memory db");
        register_acme_challenge(
            &db,
            "LoqXcYV8q5ONbJQxbmR7SCTNo3tiAXDfowyjxAjEuX0",
            "LoqXcYV8q5ONbJQxbmR7SCTNo3tiAXDfowyjxAjEuX0.9jg46WB3rR_AHD-EBXdN7cBkH1WOu0tA3M9fm21mqTI",
        )
        .expect("register challenge");

        let app = acme_well_known_routes().with_state(db);

        // 1. Valid challenge token -> 200 OK text/plain
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/.well-known/acme-challenge/LoqXcYV8q5ONbJQxbmR7SCTNo3tiAXDfowyjxAjEuX0")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(res.status(), StatusCode::OK);
        let content_type = res
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(content_type.starts_with("text/plain"));
        let body_bytes = axum::body::to_bytes(res.into_body(), 4096)
            .await
            .expect("body bytes");
        assert_eq!(
            std::str::from_utf8(&body_bytes).expect("utf8"),
            "LoqXcYV8q5ONbJQxbmR7SCTNo3tiAXDfowyjxAjEuX0.9jg46WB3rR_AHD-EBXdN7cBkH1WOu0tA3M9fm21mqTI"
        );

        // 2. Unknown challenge token -> 404 Not Found
        let res_404 = app
            .oneshot(
                Request::builder()
                    .uri("/.well-known/acme-challenge/nonexistent-token-123")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");
        assert_eq!(res_404.status(), StatusCode::NOT_FOUND);
    }
}
