//! Pure-Rust S3 & Cloudflare R2 object storage client (`SigV4`) and continuous SQLite replica engine.
//!
//! Provides zero-dependency (beyond existing workspace crates `reqwest`, `hmac`, `sha2`, `hex`,
//! `chrono`, and `flate2`) AWS Signature Version 4 signing, media library offload to Cloudflare R2
//! or Amazon S3, and online Gzip-compressed `VACUUM INTO` SQLite replica snapshots.

use crate::auth::AdminUser;
use crate::db::Database;
use crate::errors::AppError;
use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use flate2::{write::GzEncoder, Compression};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

type HmacSha256 = Hmac<Sha256>;

/// Option key in the `options` table storing serialized [`S3StorageConfig`].
pub const S3_CONFIG_OPTION_KEY: &str = "daypress_s3_config";

/// Masked placeholder returned to admin UI clients when a secret key is configured.
pub const MASKED_SECRET_PLACEHOLDER: &str = "********";

static REPLICA_TEMP_COUNTER: AtomicU64 = AtomicU64::new(1);

fn default_provider() -> String {
    "r2".to_string()
}

fn default_region() -> String {
    "auto".to_string()
}

fn default_replica_interval_secs() -> u64 {
    300
}

/// Configuration for S3 / Cloudflare R2 media offload and SQLite replica streaming.
/// Persisted in the `options` table under `daypress_s3_config`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct S3StorageConfig {
    pub enabled: bool,
    #[serde(default = "default_provider")]
    pub provider: String,
    pub endpoint: String,
    #[serde(default = "default_region")]
    pub region: String,
    pub bucket: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub public_url_prefix: String,
    pub media_offload_enabled: bool,
    pub sqlite_replica_enabled: bool,
    #[serde(default = "default_replica_interval_secs")]
    pub replica_interval_secs: u64,
    pub last_replica_at: Option<String>,
    pub last_replica_size_bytes: Option<u64>,
}

impl Default for S3StorageConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: default_provider(),
            endpoint: String::new(),
            region: default_region(),
            bucket: String::new(),
            access_key_id: String::new(),
            secret_access_key: String::new(),
            public_url_prefix: String::new(),
            media_offload_enabled: false,
            sqlite_replica_enabled: false,
            replica_interval_secs: default_replica_interval_secs(),
            last_replica_at: None,
            last_replica_size_bytes: None,
        }
    }
}

/// Deterministic output of AWS Signature Version 4 (`SigV4`) request signing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedS3Headers {
    pub host: String,
    pub x_amz_date: String,
    pub x_amz_content_sha256: String,
    pub authorization: String,
    pub canonical_request: String,
    pub string_to_sign: String,
}

fn hmac_sha256_bytes(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts keys of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn extract_host_from_endpoint(endpoint: &str) -> String {
    let trimmed = endpoint.trim();
    let without_scheme = trimmed
        .strip_prefix("https://")
        .or_else(|| trimmed.strip_prefix("http://"))
        .unwrap_or(trimmed);
    without_scheme
        .split('/')
        .next()
        .unwrap_or("localhost")
        .trim()
        .to_string()
}

/// Computes deterministic AWS Signature Version 4 (`SigV4`) headers for `PUT`, `GET`, and `HEAD`
/// S3 / Cloudflare R2 requests.
#[allow(clippy::too_many_arguments)]
pub fn sign_s3_request(
    method: &str,
    endpoint_url: &str,
    canonical_uri: &str,
    canonical_query: &str,
    payload: &[u8],
    region: &str,
    access_key_id: &str,
    secret_access_key: &str,
    amz_date: &str,
) -> SignedS3Headers {
    let method_upper = method.trim().to_ascii_uppercase();
    let host = extract_host_from_endpoint(endpoint_url);
    let uri = if canonical_uri.starts_with('/') {
        canonical_uri.to_string()
    } else {
        format!("/{}", canonical_uri)
    };
    let effective_region = if region.trim().is_empty() {
        "auto"
    } else {
        region.trim()
    };

    let payload_hash = hex::encode(Sha256::digest(payload));
    let date_stamp: String = amz_date.chars().take(8).collect();

    let canonical_headers = format!(
        "host:{}\nx-amz-content-sha256:{}\nx-amz-date:{}\n",
        host, payload_hash, amz_date
    );
    let signed_headers = "host;x-amz-content-sha256;x-amz-date";

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n{}\n{}",
        method_upper, uri, canonical_query, canonical_headers, signed_headers, payload_hash
    );

    let canonical_request_hash = hex::encode(Sha256::digest(canonical_request.as_bytes()));
    let credential_scope = format!("{}/{}/s3/aws4_request", date_stamp, effective_region);

    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date, credential_scope, canonical_request_hash
    );

    let k_secret = format!("AWS4{}", secret_access_key);
    let k_date = hmac_sha256_bytes(k_secret.as_bytes(), date_stamp.as_bytes());
    let k_region = hmac_sha256_bytes(&k_date, effective_region.as_bytes());
    let k_service = hmac_sha256_bytes(&k_region, b"s3");
    let k_signing = hmac_sha256_bytes(&k_service, b"aws4_request");
    let signature = hex::encode(hmac_sha256_bytes(&k_signing, string_to_sign.as_bytes()));

    let authorization = format!(
        "AWS4-HMAC-SHA256 Credential={}/{}, SignedHeaders={}, Signature={}",
        access_key_id.trim(),
        credential_scope,
        signed_headers,
        signature
    );

    SignedS3Headers {
        host,
        x_amz_date: amz_date.to_string(),
        x_amz_content_sha256: payload_hash,
        authorization,
        canonical_request,
        string_to_sign,
    }
}

/// Loads [`S3StorageConfig`] from SQLite `options` (`daypress_s3_config`).
pub fn load_s3_config(db: &Database) -> S3StorageConfig {
    db.get_option(S3_CONFIG_OPTION_KEY)
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<S3StorageConfig>(&raw).ok())
        .unwrap_or_default()
}

/// Persists [`S3StorageConfig`] to SQLite `options` (`daypress_s3_config`).
/// If `secret_access_key` equals the masked placeholder (`"********"`) or is empty while a secret
/// already exists in SQLite, the existing secret is preserved.
pub fn save_s3_config(db: &Database, cfg: &S3StorageConfig) -> Result<(), AppError> {
    let existing = load_s3_config(db);
    let mut merged = cfg.clone();

    if (merged.secret_access_key == MASKED_SECRET_PLACEHOLDER
        || merged.secret_access_key.trim().is_empty())
        && !existing.secret_access_key.is_empty()
    {
        merged.secret_access_key = existing.secret_access_key;
    }
    if merged.last_replica_at.is_none() {
        merged.last_replica_at = existing.last_replica_at;
    }
    if merged.last_replica_size_bytes.is_none() {
        merged.last_replica_size_bytes = existing.last_replica_size_bytes;
    }
    if merged.provider.trim().is_empty() {
        merged.provider = "r2".to_string();
    }
    if merged.region.trim().is_empty() {
        merged.region = if merged.provider.eq_ignore_ascii_case("r2") {
            "auto".to_string()
        } else {
            "us-east-1".to_string()
        };
    }
    if merged.replica_interval_secs == 0 {
        merged.replica_interval_secs = 300;
    }

    let json = serde_json::to_string(&merged)
        .map_err(|e| AppError::Internal(format!("Failed to serialize S3 config: {e}")))?;
    db.set_option(S3_CONFIG_OPTION_KEY, &json)?;
    Ok(())
}

/// Uploads an object (`PUT`) to the configured S3 / Cloudflare R2 bucket using `SigV4` signing.
/// Returns the public CDN or endpoint URL of the uploaded object.
pub async fn put_object(
    cfg: &S3StorageConfig,
    key: &str,
    bytes: &[u8],
    content_type: &str,
) -> Result<String, AppError> {
    let endpoint = cfg.endpoint.trim().trim_end_matches('/');
    let bucket = cfg.bucket.trim().trim_matches('/');
    if endpoint.is_empty() || bucket.is_empty() {
        return Err(AppError::BadRequest(
            "S3/R2 endpoint and bucket name are required for object upload.".to_string(),
        ));
    }
    if cfg.access_key_id.trim().is_empty() || cfg.secret_access_key.trim().is_empty() {
        return Err(AppError::BadRequest(
            "S3/R2 access_key_id and secret_access_key are required.".to_string(),
        ));
    }

    let clean_key = key.trim_start_matches('/');
    let canonical_uri = format!("/{}/{}", bucket, clean_key);
    let url = format!("{}{}", endpoint, canonical_uri);
    let amz_date = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

    let signed = sign_s3_request(
        "PUT",
        endpoint,
        &canonical_uri,
        "",
        bytes,
        &cfg.region,
        &cfg.access_key_id,
        &cfg.secret_access_key,
        &amz_date,
    );

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| AppError::Internal(format!("Failed to build HTTP client: {e}")))?;

    let resp = client
        .put(&url)
        .header("Host", &signed.host)
        .header("x-amz-date", &signed.x_amz_date)
        .header("x-amz-content-sha256", &signed.x_amz_content_sha256)
        .header("Authorization", &signed.authorization)
        .header("Content-Type", content_type)
        .body(bytes.to_vec())
        .send()
        .await
        .map_err(|e| AppError::Internal(format!("S3/R2 PUT request failed: {e}")))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let err_body = resp.text().await.unwrap_or_default();
        return Err(AppError::Internal(format!(
            "S3/R2 PUT {} returned HTTP {}: {}",
            clean_key, status, err_body
        )));
    }

    let public_url = if !cfg.public_url_prefix.trim().is_empty() {
        format!(
            "{}/{}",
            cfg.public_url_prefix.trim().trim_end_matches('/'),
            clean_key
        )
    } else {
        url
    };

    Ok(public_url)
}

fn guess_content_type(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("webp") => "image/webp",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("avif") => "image/avif",
        Some("ico") => "image/x-icon",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mp3") => "audio/mpeg",
        Some("pdf") => "application/pdf",
        Some("json") => "application/json",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        _ => "application/octet-stream",
    }
}

fn collect_upload_files_recursive(
    base: &Path,
    dir: &Path,
    out: &mut Vec<(String, std::path::PathBuf)>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_upload_files_recursive(base, &path, out);
        } else if path.is_file() {
            if let Ok(rel) = path.strip_prefix(base) {
                let rel_slash = rel
                    .components()
                    .map(|c| c.as_os_str().to_string_lossy())
                    .collect::<Vec<_>>()
                    .join("/");
                if !rel_slash.is_empty() {
                    out.push((rel_slash, path));
                }
            }
        }
    }
}

/// Iterates all files in `uploads/`, uploads each to `uploads/<filename>` in the configured S3/R2
/// bucket, and if `public_url_prefix` is configured, updates matching attachment `guid` URLs in SQLite.
pub async fn sync_uploads_to_s3(db: &Database) -> Result<serde_json::Value, AppError> {
    let cfg = load_s3_config(db);
    let uploads_dir = Path::new("uploads");
    let mut files = Vec::new();
    if uploads_dir.exists() {
        collect_upload_files_recursive(uploads_dir, uploads_dir, &mut files);
    }

    let remote_configured = !cfg.endpoint.trim().is_empty()
        && !cfg.bucket.trim().is_empty()
        && !cfg.access_key_id.trim().is_empty()
        && !cfg.secret_access_key.trim().is_empty();

    let mut files_uploaded = 0usize;
    let mut bytes_synced = 0u64;
    let mut attachments_rewritten = 0usize;
    let mut errors = Vec::new();

    for (rel_path, full_path) in &files {
        let Ok(bytes) = std::fs::read(full_path) else {
            continue;
        };
        let s3_key = format!("uploads/{}", rel_path);
        let content_type = guess_content_type(full_path);

        if remote_configured {
            match put_object(&cfg, &s3_key, &bytes, content_type).await {
                Ok(_) => {
                    files_uploaded += 1;
                    bytes_synced += bytes.len() as u64;
                }
                Err(e) => {
                    errors.push(format!("{}: {}", s3_key, e));
                    continue;
                }
            }
        } else {
            bytes_synced += bytes.len() as u64;
        }

        if !cfg.public_url_prefix.trim().is_empty() {
            let new_guid = format!(
                "{}/uploads/{}",
                cfg.public_url_prefix.trim().trim_end_matches('/'),
                rel_path
            );
            let like_pattern = format!("%/uploads/{}", rel_path);
            if let Ok(conn) = db.conn.lock() {
                if let Ok(updated) = conn.execute(
                    "UPDATE posts SET guid = ?1 WHERE post_type = 'attachment' AND (guid LIKE ?2 OR guid = ?3)",
                    rusqlite::params![new_guid, like_pattern, format!("/uploads/{}", rel_path)],
                ) {
                    attachments_rewritten += updated;
                }
            }
        }
    }

    Ok(serde_json::json!({
        "status": "ok",
        "provider": cfg.provider,
        "bucket": cfg.bucket,
        "remote_configured": remote_configured,
        "files_scanned": files.len(),
        "files_uploaded": files_uploaded,
        "bytes_synced": bytes_synced,
        "attachments_rewritten": attachments_rewritten,
        "errors": errors,
    }))
}

/// Runs `VACUUM INTO` to a temporary file in `backups/`, Gzip-compresses the SQLite snapshot with
/// `flate2::write::GzEncoder`, deletes the temporary raw file, and returns `(filename, gz_bytes)`.
pub fn create_sqlite_replica_snapshot_bytes(db: &Database) -> Result<(String, Vec<u8>), AppError> {
    let backups_dir = Path::new("backups");
    std::fs::create_dir_all(backups_dir)?;

    let now = chrono::Utc::now();
    let ts = now.format("%Y%m%d-%H%M%S").to_string();
    let seq = REPLICA_TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let temp_raw_name = format!(
        "daypress-replica-tmp-{}-{}-{}.sqlite",
        ts,
        std::process::id(),
        seq
    );
    let temp_raw_path = backups_dir.join(&temp_raw_name);
    if temp_raw_path.exists() {
        let _ = std::fs::remove_file(&temp_raw_path);
    }

    {
        let conn = db
            .conn
            .lock()
            .map_err(|_| AppError::Internal("Database lock poisoned".to_string()))?;
        let escaped_path = temp_raw_path.to_string_lossy().replace('\'', "''");
        let sql = format!("VACUUM INTO '{}'", escaped_path);
        conn.execute_batch(&sql)?;
    }

    let raw_bytes = std::fs::read(&temp_raw_path);
    let _ = std::fs::remove_file(&temp_raw_path);
    let raw_bytes = raw_bytes?;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&raw_bytes)?;
    let gz_bytes = encoder.finish()?;

    let gz_filename = format!("daypress-replica-{}.sqlite.gz", ts);
    Ok((gz_filename, gz_bytes))
}

/// Creates a Gzip-compressed `VACUUM INTO` SQLite snapshot, persists the local `.sqlite.gz` replica
/// in `backups/`, uploads `replicas/daypress-latest.sqlite.gz` and `replicas/{filename}` to S3/R2
/// when configured, and updates `last_replica_at` and `last_replica_size_bytes` in `daypress_s3_config`.
pub async fn replicate_sqlite_to_s3(db: &Database) -> Result<serde_json::Value, AppError> {
    let (filename, gz_bytes) = create_sqlite_replica_snapshot_bytes(db)?;
    let size_bytes = gz_bytes.len() as u64;

    let backups_dir = Path::new("backups");
    std::fs::create_dir_all(backups_dir)?;
    let local_latest_path = backups_dir.join("daypress-latest.sqlite.gz");
    let local_versioned_path = backups_dir.join(&filename);
    std::fs::write(&local_latest_path, &gz_bytes)?;
    std::fs::write(&local_versioned_path, &gz_bytes)?;

    let mut cfg = load_s3_config(db);
    let now_rfc3339 = chrono::Utc::now().to_rfc3339();
    cfg.last_replica_at = Some(now_rfc3339.clone());
    cfg.last_replica_size_bytes = Some(size_bytes);
    save_s3_config(db, &cfg)?;

    let remote_configured = !cfg.endpoint.trim().is_empty()
        && !cfg.bucket.trim().is_empty()
        && !cfg.access_key_id.trim().is_empty()
        && !cfg.secret_access_key.trim().is_empty();

    let mut remote_uploaded = false;
    let mut remote_url: Option<String> = None;
    let mut remote_error: Option<String> = None;

    if remote_configured {
        let latest_key = "replicas/daypress-latest.sqlite.gz";
        let versioned_key = format!("replicas/{}", filename);
        match put_object(&cfg, latest_key, &gz_bytes, "application/gzip").await {
            Ok(url) => {
                let _ = put_object(&cfg, &versioned_key, &gz_bytes, "application/gzip").await;
                remote_uploaded = true;
                remote_url = Some(url);
            }
            Err(e) => {
                remote_error = Some(e.to_string());
            }
        }
    }

    Ok(serde_json::json!({
        "status": "ok",
        "filename": filename,
        "local_path": local_versioned_path.to_string_lossy(),
        "local_latest_path": local_latest_path.to_string_lossy(),
        "size_bytes": size_bytes,
        "last_replica_at": now_rfc3339,
        "remote_configured": remote_configured,
        "remote_uploaded": remote_uploaded,
        "remote_url": remote_url,
        "remote_error": remote_error,
    }))
}

fn mask_secret(mut cfg: S3StorageConfig) -> S3StorageConfig {
    if !cfg.secret_access_key.is_empty() {
        cfg.secret_access_key = MASKED_SECRET_PLACEHOLDER.to_string();
    }
    cfg
}

async fn get_s3_config_handler(
    _admin: AdminUser,
    State(db): State<Database>,
) -> Result<Json<S3StorageConfig>, AppError> {
    let cfg = load_s3_config(&db);
    Ok(Json(mask_secret(cfg)))
}

async fn save_s3_config_handler(
    _admin: AdminUser,
    State(db): State<Database>,
    Json(payload): Json<S3StorageConfig>,
) -> Result<Json<S3StorageConfig>, AppError> {
    save_s3_config(&db, &payload)?;
    let updated = load_s3_config(&db);
    Ok(Json(mask_secret(updated)))
}

async fn sync_media_handler(
    _admin: AdminUser,
    State(db): State<Database>,
) -> Result<Json<serde_json::Value>, AppError> {
    let report = sync_uploads_to_s3(&db).await?;
    Ok(Json(report))
}

async fn replicate_db_handler(
    _admin: AdminUser,
    State(db): State<Database>,
) -> Result<Json<serde_json::Value>, AppError> {
    let report = replicate_sqlite_to_s3(&db).await?;
    Ok(Json(report))
}

/// Mounts `/storage/s3/*` routes under `/wp-json/daypress/v1`.
pub fn s3_storage_routes() -> Router<Database> {
    Router::new()
        .route(
            "/storage/s3/config",
            get(get_s3_config_handler).post(save_s3_config_handler),
        )
        .route("/storage/s3/sync-media", post(sync_media_handler))
        .route("/storage/s3/replicate-db", post(replicate_db_handler))
}

/// Non-blocking hook triggered after media upload or in-place media replacement:
/// when `daypress_s3_config` has `enabled && media_offload_enabled` and remote credentials are set,
/// spawns a background task to `PUT` `uploads/{file_name}` to S3/R2 and rewrite the attachment
/// `guid` if `public_url_prefix` is configured.
pub fn maybe_offload_media_file(db: &Database, file_name: &str, bytes: &[u8], content_type: &str) {
    let cfg = load_s3_config(db);
    if !cfg.enabled || !cfg.media_offload_enabled {
        return;
    }
    if cfg.endpoint.trim().is_empty()
        || cfg.bucket.trim().is_empty()
        || cfg.access_key_id.trim().is_empty()
        || cfg.secret_access_key.trim().is_empty()
    {
        return;
    }
    if tokio::runtime::Handle::try_current().is_err() {
        return;
    }
    let db_clone = db.clone();
    let rel_name = file_name.trim_start_matches('/').to_string();
    let payload = bytes.to_vec();
    let mime = content_type.to_string();
    tokio::spawn(async move {
        let s3_key = format!("uploads/{}", rel_name);
        if put_object(&cfg, &s3_key, &payload, &mime).await.is_ok()
            && !cfg.public_url_prefix.trim().is_empty()
        {
            let new_guid = format!(
                "{}/uploads/{}",
                cfg.public_url_prefix.trim().trim_end_matches('/'),
                rel_name
            );
            let like_pattern = format!("%/uploads/{}", rel_name);
            let local_url = format!("/uploads/{}", rel_name);
            if let Ok(conn) = db_clone.conn.lock() {
                let _ = conn.execute(
                    "UPDATE posts SET guid = ?1 WHERE post_type = 'attachment' AND (guid LIKE ?2 OR guid = ?3)",
                    rusqlite::params![new_guid, like_pattern, local_url],
                );
            }
        }
    });
}

/// Spawns a non-blocking background Tokio task that checks `daypress_s3_config` every 60 seconds
/// and streams a Gzip-compressed `VACUUM INTO` SQLite snapshot to `backups/` and S3/R2 whenever
/// `enabled && sqlite_replica_enabled` and `replica_interval_secs` has elapsed.
pub fn spawn_background_sqlite_replicator(db: Database) {
    if tokio::runtime::Handle::try_current().is_err() {
        return;
    }
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(60));
        // Skip immediate tick at t=0 so startup is instant
        interval.tick().await;
        loop {
            interval.tick().await;
            let cfg = load_s3_config(&db);
            if !cfg.enabled || !cfg.sqlite_replica_enabled {
                continue;
            }
            let min_interval = cfg.replica_interval_secs.max(60) as i64;
            let should_run = match cfg.last_replica_at.as_deref() {
                Some(ts) => match chrono::DateTime::parse_from_rfc3339(ts) {
                    Ok(last_dt) => {
                        let elapsed = chrono::Utc::now()
                            .signed_duration_since(last_dt.with_timezone(&chrono::Utc))
                            .num_seconds();
                        elapsed >= min_interval
                    }
                    Err(_) => true,
                },
                None => true,
            };
            if should_run {
                let _ = replicate_sqlite_to_s3(&db).await;
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::read::GzDecoder;
    use std::io::Read;

    #[test]
    fn sigv4_canonical_request_and_authorization_header_are_deterministic() {
        let signed = sign_s3_request(
            "PUT",
            "https://demo-account.r2.cloudflarestorage.com",
            "/daypress-bucket/replicas/daypress-latest.sqlite.gz",
            "",
            b"hello-daypress-r2",
            "auto",
            "AKIAIOSFODNN7EXAMPLE",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            "20261102T202820Z",
        );

        assert_eq!(signed.host, "demo-account.r2.cloudflarestorage.com");
        assert_eq!(signed.x_amz_date, "20261102T202820Z");
        assert_eq!(
            signed.x_amz_content_sha256,
            "826106b314a991db2d436576c5cba61495f072142bcc54595b1236cc3ea1a415"
        );
        assert!(
            signed
                .canonical_request
                .starts_with("PUT\n/daypress-bucket/replicas/daypress-latest.sqlite.gz\n\nhost:demo-account.r2.cloudflarestorage.com\n"),
            "unexpected canonical request: {}",
            signed.canonical_request
        );
        assert!(
            signed.authorization.starts_with(
                "AWS4-HMAC-SHA256 Credential=AKIAIOSFODNN7EXAMPLE/20261102/auto/s3/aws4_request, SignedHeaders=host;x-amz-content-sha256;x-amz-date, Signature="
            ),
            "unexpected Authorization header: {}",
            signed.authorization
        );
        // Verify deterministic repeat call produces identical signature
        let signed_again = sign_s3_request(
            "PUT",
            "https://demo-account.r2.cloudflarestorage.com",
            "/daypress-bucket/replicas/daypress-latest.sqlite.gz",
            "",
            b"hello-daypress-r2",
            "auto",
            "AKIAIOSFODNN7EXAMPLE",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            "20261102T202820Z",
        );
        assert_eq!(signed, signed_again);
    }

    #[test]
    fn create_sqlite_replica_snapshot_bytes_produces_valid_gzip_sqlite_image() {
        let db = Database::new_in_memory().expect("in-memory db");
        db.set_option("blogname", "Replica Verification Site")
            .expect("set blogname");

        let (filename, gz_bytes) =
            create_sqlite_replica_snapshot_bytes(&db).expect("create replica snapshot");
        assert!(filename.starts_with("daypress-replica-"));
        assert!(filename.ends_with(".sqlite.gz"));
        assert!(!gz_bytes.is_empty(), "compressed replica must not be empty");

        // Decompress and verify SQLite3 magic header
        let mut decoder = GzDecoder::new(&gz_bytes[..]);
        let mut raw_sqlite = Vec::new();
        decoder
            .read_to_end(&mut raw_sqlite)
            .expect("decompress gzip replica");
        assert!(
            raw_sqlite.starts_with(b"SQLite format 3\0"),
            "decompressed bytes must start with SQLite format 3 header"
        );
    }

    #[test]
    fn s3_config_roundtrip_preserves_masked_secret_key() {
        let db = Database::new_in_memory().expect("in-memory db");
        let cfg = S3StorageConfig {
            enabled: true,
            provider: "r2".to_string(),
            endpoint: "https://acct.r2.cloudflarestorage.com".to_string(),
            region: "auto".to_string(),
            bucket: "daypress-media".to_string(),
            access_key_id: "AKIA123".to_string(),
            secret_access_key: "super-secret-r2-key".to_string(),
            public_url_prefix: "https://cdn.example.com".to_string(),
            media_offload_enabled: true,
            sqlite_replica_enabled: true,
            replica_interval_secs: 120,
            last_replica_at: None,
            last_replica_size_bytes: None,
        };
        save_s3_config(&db, &cfg).expect("save initial s3 config");

        // Simulate UI sending back masked secret
        let mut ui_update = cfg.clone();
        ui_update.bucket = "daypress-media-v2".to_string();
        ui_update.secret_access_key = MASKED_SECRET_PLACEHOLDER.to_string();
        save_s3_config(&db, &ui_update).expect("save masked update");

        let loaded = load_s3_config(&db);
        assert_eq!(loaded.bucket, "daypress-media-v2");
        assert_eq!(loaded.secret_access_key, "super-secret-r2-key");
    }
}
