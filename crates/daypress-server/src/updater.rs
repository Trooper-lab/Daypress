//! # Self-Updater & Release Management (`updater.rs`)
//!
//! Provides in-app release discovery, 24-hour background update checks persisted in SQLite,
//! SHA-256 verification, `.tar.gz` / `.zip` binary extraction, smoke testing, automatic
//! pre-update SQLite snapshots, atomic executable hot-swapping, 1-click rollback, and
//! container-aware Docker guidance.

use crate::db::Database;
use crate::errors::AppError;
use chrono::Utc;
use flate2::read::{DeflateDecoder, GzDecoder};
use semver::Version;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

pub const OPTION_UPDATE_STATE: &str = "daypress_update_state";
pub const OPTION_UPDATE_SETTINGS: &str = "daypress_update_settings";
pub const OPTION_UPDATE_HISTORY: &str = "daypress_update_history";
pub const DEFAULT_GITHUB_REPO: &str = "Trooper-lab/daypress";
const MAX_EXTRACTED_BINARY_BYTES: u64 = 150 * 1024 * 1024; // 150 MB safety cap

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSettings {
    #[serde(default = "default_true")]
    pub auto_check_enabled: bool,
    #[serde(default = "default_channel")]
    pub channel: String,
    #[serde(default = "default_repo")]
    pub github_repo: String,
}

fn default_true() -> bool {
    true
}

fn default_channel() -> String {
    "stable".to_string()
}

fn default_repo() -> String {
    std::env::var("DAYPRESS_UPDATE_REPO")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_GITHUB_REPO.to_string())
}

impl Default for UpdateSettings {
    fn default() -> Self {
        Self {
            auto_check_enabled: true,
            channel: default_channel(),
            github_repo: default_repo(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateState {
    pub last_checked_at: Option<String>,
    pub latest_version: Option<String>,
    pub release_tag: Option<String>,
    pub release_url: Option<String>,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
    pub prerelease: bool,
    pub asset_name: Option<String>,
    pub asset_download_url: Option<String>,
    pub asset_size_bytes: Option<u64>,
    pub sha256_digest: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHistoryEntry {
    pub timestamp: String,
    pub action: String, // "update" | "upload" | "rollback"
    pub from_version: String,
    pub to_version: String,
    pub backup_filename: Option<String>,
    pub sha256_digest: String,
    pub status: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEnvironment {
    pub current_version: String,
    pub target_triple: String,
    pub hosting_mode: String, // "native" | "docker" | "readonly"
    pub is_container: bool,
    pub can_self_update: bool,
    pub executable_path: String,
    pub rollback_available: bool,
    pub rollback_version_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatusResponse {
    pub environment: UpdateEnvironment,
    pub settings: UpdateSettings,
    pub state: UpdateState,
    pub update_available: bool,
    pub history: Vec<UpdateHistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateApplyReport {
    pub success: bool,
    pub action: String,
    pub from_version: String,
    pub to_version: String,
    pub backup_filename: String,
    pub sha256_digest: String,
    pub restart_scheduled: bool,
    pub message: String,
}

/// Returns the current compiled Daypress version (e.g., `"0.1.1"`).
pub fn current_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Returns the canonical release target triple matching `.github/workflows/release.yml`.
pub fn current_target_triple() -> String {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    match (arch, os) {
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu".to_string(),
        ("aarch64", "macos") => "aarch64-apple-darwin".to_string(),
        ("x86_64", "macos") => "x86_64-apple-darwin".to_string(),
        ("x86_64", "windows") => "x86_64-pc-windows-msvc".to_string(),
        _ => format!("{arch}-{os}"),
    }
}

/// Expected release archive filename for a given version tag and target triple.
pub fn expected_asset_filename(tag: &str, target_triple: &str) -> String {
    let clean_tag = if tag.starts_with('v') {
        tag.to_string()
    } else {
        format!("v{tag}")
    };
    if target_triple.contains("windows") {
        format!("daypress-{clean_tag}-{target_triple}.zip")
    } else {
        format!("daypress-{clean_tag}-{target_triple}.tar.gz")
    }
}

/// Compares two semantic version strings (stripping leading `v` if present).
/// Returns `true` strictly when `candidate > current`.
pub fn is_newer_version(current: &str, candidate: &str) -> bool {
    let cur_clean = current.trim().trim_start_matches('v');
    let cand_clean = candidate.trim().trim_start_matches('v');
    match (Version::parse(cur_clean), Version::parse(cand_clean)) {
        (Ok(cur), Ok(cand)) => cand > cur,
        _ => false,
    }
}

/// Detects whether Daypress is running inside a container or on a read-only binary path.
pub fn inspect_environment(db: &Database) -> UpdateEnvironment {
    let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("daypress"));
    inspect_environment_for_path(db, &exe_path)
}

pub fn inspect_environment_for_path(db: &Database, exe_path: &Path) -> UpdateEnvironment {
    let is_container = Path::new("/.dockerenv").exists()
        || Path::new("/run/.containerenv").exists()
        || std::env::var("DAYPRESS_CONTAINER")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

    let writable = is_executable_dir_writable(exe_path);
    let can_self_update = !is_container && writable;

    let hosting_mode = if is_container {
        "docker".to_string()
    } else if writable {
        "native".to_string()
    } else {
        "readonly".to_string()
    };

    let prev_path = previous_binary_path(exe_path);
    let rollback_available = prev_path
        .metadata()
        .map(|m| m.is_file() && m.len() > 0)
        .unwrap_or(false);

    let history = load_update_history(db);
    let rollback_version_hint = if rollback_available {
        history.first().map(|h| h.from_version.clone())
    } else {
        None
    };

    UpdateEnvironment {
        current_version: current_version().to_string(),
        target_triple: current_target_triple(),
        hosting_mode,
        is_container,
        can_self_update,
        executable_path: exe_path.display().to_string(),
        rollback_available,
        rollback_version_hint,
    }
}

fn is_executable_dir_writable(exe_path: &Path) -> bool {
    let Some(parent) = exe_path.parent() else {
        return false;
    };
    let parent_dir = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let probe = parent_dir.join(format!(".daypress-write-probe-{}", std::process::id()));
    match fs::write(&probe, b"ok") {
        Ok(()) => {
            let _ = fs::remove_file(&probe);
            true
        }
        Err(_) => false,
    }
}

pub fn staged_binary_path(exe_path: &Path) -> PathBuf {
    let mut p = exe_path.as_os_str().to_os_string();
    p.push(".staged");
    PathBuf::from(p)
}

pub fn previous_binary_path(exe_path: &Path) -> PathBuf {
    let mut p = exe_path.as_os_str().to_os_string();
    p.push(".prev");
    PathBuf::from(p)
}

pub fn load_update_settings(db: &Database) -> UpdateSettings {
    db.get_option(OPTION_UPDATE_SETTINGS)
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<UpdateSettings>(&raw).ok())
        .unwrap_or_default()
}

pub fn save_update_settings(db: &Database, settings: &UpdateSettings) -> Result<(), AppError> {
    let json = serde_json::to_string(settings)
        .map_err(|e| AppError::Internal(format!("Failed to serialize update settings: {e}")))?;
    db.set_option(OPTION_UPDATE_SETTINGS, &json)?;
    Ok(())
}

pub fn load_update_state(db: &Database) -> UpdateState {
    db.get_option(OPTION_UPDATE_STATE)
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<UpdateState>(&raw).ok())
        .unwrap_or_default()
}

pub fn save_update_state(db: &Database, state: &UpdateState) -> Result<(), AppError> {
    let json = serde_json::to_string(state)
        .map_err(|e| AppError::Internal(format!("Failed to serialize update state: {e}")))?;
    db.set_option(OPTION_UPDATE_STATE, &json)?;
    Ok(())
}

pub fn load_update_history(db: &Database) -> Vec<UpdateHistoryEntry> {
    db.get_option(OPTION_UPDATE_HISTORY)
        .ok()
        .flatten()
        .and_then(|raw| serde_json::from_str::<Vec<UpdateHistoryEntry>>(&raw).ok())
        .unwrap_or_default()
}

pub fn append_update_history(db: &Database, entry: UpdateHistoryEntry) -> Result<(), AppError> {
    let mut history = load_update_history(db);
    history.insert(0, entry);
    history.truncate(25);
    let json = serde_json::to_string(&history)
        .map_err(|e| AppError::Internal(format!("Failed to serialize update history: {e}")))?;
    db.set_option(OPTION_UPDATE_HISTORY, &json)?;
    Ok(())
}

/// Returns `Some(latest_version)` if the cached SQLite update state indicates an update is available.
pub fn cached_available_update_version(db: &Database) -> Option<String> {
    let state = load_update_state(db);
    let latest = state.latest_version.as_deref()?;
    if is_newer_version(current_version(), latest) {
        Some(latest.trim_start_matches('v').to_string())
    } else {
        None
    }
}

/// Builds the full status payload for `/d-admin/update-core` and `GET /wp-json/daypress/v1/update/status`.
pub fn get_update_status(db: &Database) -> UpdateStatusResponse {
    let environment = inspect_environment(db);
    let settings = load_update_settings(db);
    let state = load_update_state(db);
    let update_available = state
        .latest_version
        .as_deref()
        .map(|latest| is_newer_version(&environment.current_version, latest))
        .unwrap_or(false);
    let history = load_update_history(db);

    UpdateStatusResponse {
        environment,
        settings,
        state,
        update_available,
        history,
    }
}

#[derive(Debug, Deserialize)]
pub struct GithubReleaseDto {
    pub tag_name: String,
    pub html_url: String,
    #[serde(default)]
    pub body: Option<String>,
    #[serde(default)]
    pub prerelease: bool,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub published_at: Option<String>,
    #[serde(default)]
    pub assets: Vec<GithubAssetDto>,
}

#[derive(Debug, Deserialize)]
pub struct GithubAssetDto {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

/// Parses a `SHA256SUMS` file and extracts the 64-character hex digest for `asset_name`.
pub fn parse_sha256sums(sums_text: &str, asset_name: &str) -> Option<String> {
    for line in sums_text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let hash = parts.next()?;
        let file_part = parts.next()?;
        let clean_file = file_part.trim_start_matches('*').trim_start_matches("./");
        let file_basename = Path::new(clean_file)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(clean_file);
        if file_basename == asset_name
            && hash.len() == 64
            && hash.chars().all(|c| c.is_ascii_hexdigit())
        {
            return Some(hash.to_ascii_lowercase());
        }
    }
    None
}

/// Converts a GitHub release JSON response + optional `SHA256SUMS` text into an `UpdateState`.
pub fn build_update_state_from_release(
    release: &GithubReleaseDto,
    target_triple: &str,
    sums_text: Option<&str>,
) -> UpdateState {
    let expected_name = expected_asset_filename(&release.tag_name, target_triple);
    let matched_asset = release
        .assets
        .iter()
        .find(|a| a.name == expected_name)
        .or_else(|| {
            release
                .assets
                .iter()
                .find(|a| a.name.contains(target_triple))
        });

    let sha256_digest = matched_asset
        .and_then(|asset| sums_text.and_then(|txt| parse_sha256sums(txt, &asset.name)));

    let clean_version = release.tag_name.trim().trim_start_matches('v').to_string();
    let notes_safe: String = release
        .body
        .as_deref()
        .unwrap_or("")
        .chars()
        .take(8000)
        .collect();

    UpdateState {
        last_checked_at: Some(Utc::now().to_rfc3339()),
        latest_version: Some(clean_version),
        release_tag: Some(release.tag_name.clone()),
        release_url: Some(release.html_url.clone()),
        release_notes: Some(notes_safe),
        published_at: release.published_at.clone(),
        prerelease: release.prerelease,
        asset_name: matched_asset.map(|a| a.name.clone()),
        asset_download_url: matched_asset.map(|a| a.browser_download_url.clone()),
        asset_size_bytes: matched_asset.map(|a| a.size),
        sha256_digest,
        last_error: None,
    }
}

/// Queries GitHub Releases API for the latest release matching the configured channel,
/// fetches `SHA256SUMS` if attached, and persists the resulting `UpdateState` in SQLite.
pub async fn check_for_updates_now(db: &Database) -> Result<UpdateState, AppError> {
    let settings = load_update_settings(db);
    let repo = if settings.github_repo.trim().is_empty() {
        DEFAULT_GITHUB_REPO.to_string()
    } else {
        settings.github_repo.trim().to_string()
    };

    let client = reqwest::Client::builder()
        .user_agent(format!("Daypress-Updater/{}", current_version()))
        .timeout(Duration::from_secs(20))
        .build()
        .map_err(|e| AppError::Internal(format!("HTTP client initialization failed: {e}")))?;

    let api_base = std::env::var("DAYPRESS_GITHUB_API_BASE")
        .unwrap_or_else(|_| "https://api.github.com".to_string());

    let release: GithubReleaseDto = if settings.channel == "prerelease" {
        let url = format!(
            "{}/repos/{repo}/releases?per_page=10",
            api_base.trim_end_matches('/')
        );
        let resp = client.get(&url).send().await.map_err(|e| {
            record_check_error(db, &format!("Could not reach GitHub Releases: {e}"));
            AppError::BadRequest(format!("Could not reach GitHub Releases: {e}"))
        })?;
        if !resp.status().is_success() {
            let msg = format!("GitHub API returned HTTP {}", resp.status());
            record_check_error(db, &msg);
            return Err(AppError::BadRequest(msg));
        }
        let releases: Vec<GithubReleaseDto> = resp.json().await.map_err(|e| {
            AppError::Internal(format!("Invalid GitHub Releases JSON payload: {e}"))
        })?;
        releases
            .into_iter()
            .find(|r| !r.draft)
            .ok_or_else(|| AppError::NotFound("No published releases found on GitHub.".into()))?
    } else {
        let url = format!(
            "{}/repos/{repo}/releases/latest",
            api_base.trim_end_matches('/')
        );
        let resp = client.get(&url).send().await.map_err(|e| {
            record_check_error(db, &format!("Could not reach GitHub Releases: {e}"));
            AppError::BadRequest(format!("Could not reach GitHub Releases: {e}"))
        })?;
        if resp.status().as_u16() == 404 {
            // Fall back to /releases list if no non-prerelease latest tag exists yet
            let list_url = format!(
                "{}/repos/{repo}/releases?per_page=5",
                api_base.trim_end_matches('/')
            );
            let list_resp = client.get(&list_url).send().await.map_err(|e| {
                AppError::BadRequest(format!("Could not reach GitHub Releases: {e}"))
            })?;
            let releases: Vec<GithubReleaseDto> = list_resp.json().await.unwrap_or_default();
            if let Some(first) = releases.into_iter().find(|r| !r.draft) {
                first
            } else {
                let state = UpdateState {
                    last_checked_at: Some(Utc::now().to_rfc3339()),
                    latest_version: Some(current_version().to_string()),
                    release_tag: Some(format!("v{}", current_version())),
                    release_url: Some(format!("https://github.com/{repo}/releases")),
                    release_notes: Some("No newer release published yet.".to_string()),
                    ..Default::default()
                };
                save_update_state(db, &state)?;
                return Ok(state);
            }
        } else if !resp.status().is_success() {
            let msg = format!("GitHub API returned HTTP {}", resp.status());
            record_check_error(db, &msg);
            return Err(AppError::BadRequest(msg));
        } else {
            resp.json().await.map_err(|e| {
                AppError::Internal(format!("Invalid GitHub Release JSON payload: {e}"))
            })?
        }
    };

    // Fetch SHA256SUMS asset if present
    let sums_asset = release
        .assets
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case("SHA256SUMS"));
    let sums_text = if let Some(sums) = sums_asset {
        match client.get(&sums.browser_download_url).send().await {
            Ok(r) if r.status().is_success() => r.text().await.ok(),
            _ => None,
        }
    } else {
        None
    };

    let target = current_target_triple();
    let state = build_update_state_from_release(&release, &target, sums_text.as_deref());
    save_update_state(db, &state)?;
    Ok(state)
}

fn record_check_error(db: &Database, message: &str) {
    let mut state = load_update_state(db);
    state.last_checked_at = Some(Utc::now().to_rfc3339());
    state.last_error = Some(message.chars().take(500).collect());
    let _ = save_update_state(db, &state);
}

/// Spawns a non-blocking 24-hour background loop that checks GitHub Releases when
/// `auto_check_enabled` is true and the cached check is older than 24 hours.
pub fn spawn_background_update_checker(db: Database) {
    if std::env::var("DAYPRESS_DISABLE_UPDATE_CHECKER")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return;
    }

    tokio::spawn(async move {
        // Initial delay so server startup is completely unaffected
        tokio::time::sleep(Duration::from_secs(15)).await;
        loop {
            let settings = load_update_settings(&db);
            if settings.auto_check_enabled && should_run_scheduled_check(&db) {
                let _ = check_for_updates_now(&db).await;
            }
            tokio::time::sleep(Duration::from_secs(3600)).await;
        }
    });
}

pub fn should_run_scheduled_check(db: &Database) -> bool {
    let state = load_update_state(db);
    let Some(last_str) = state.last_checked_at.as_deref() else {
        return true;
    };
    let Ok(last_dt) = chrono::DateTime::parse_from_rfc3339(last_str) else {
        return true;
    };
    let elapsed = Utc::now().signed_duration_since(last_dt.with_timezone(&Utc));
    elapsed.num_hours() >= 24
}

/// Computes the lower-case SHA-256 hex digest of `bytes` and optionally verifies it against `expected_hex`.
pub fn verify_sha256(bytes: &[u8], expected_hex: Option<&str>) -> Result<String, AppError> {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let actual_hex = hex::encode(hasher.finalize());

    if let Some(exp) = expected_hex.map(str::trim).filter(|s| !s.is_empty()) {
        if !actual_hex.eq_ignore_ascii_case(exp) {
            return Err(AppError::BadRequest(format!(
                "SHA-256 checksum mismatch: expected {exp}, got {actual_hex}"
            )));
        }
    }
    Ok(actual_hex)
}

/// Extracts the `daypress` or `daypress.exe` binary from `.tar.gz`, `.zip`, or raw executable bytes.
pub fn extract_daypress_binary(archive_bytes: &[u8]) -> Result<Vec<u8>, AppError> {
    if archive_bytes.len() < 4 {
        return Err(AppError::BadRequest(
            "Uploaded update payload is empty or truncated.".into(),
        ));
    }

    // 1. Gzip-compressed POSIX tar archive (.tar.gz)
    if archive_bytes.starts_with(&[0x1f, 0x8b]) {
        return extract_binary_from_tar_gz(archive_bytes);
    }

    // 2. PKZIP archive (.zip)
    if archive_bytes.starts_with(b"PK\x03\x04") {
        return extract_binary_from_zip(archive_bytes);
    }

    // 3. Raw native executable (ELF, Mach-O, or Windows PE)
    let is_elf = archive_bytes.starts_with(b"\x7fELF");
    let is_macho = archive_bytes.starts_with(&[0xcf, 0xfa, 0xed, 0xfe])
        || archive_bytes.starts_with(&[0xfe, 0xed, 0xfa, 0xcf]);
    let is_pe = archive_bytes.starts_with(b"MZ");
    if is_elf || is_macho || is_pe {
        return Ok(archive_bytes.to_vec());
    }

    Err(AppError::BadRequest(
        "Unsupported archive format. Expected a .tar.gz release archive, .zip release archive, or native daypress executable.".into(),
    ))
}

fn is_daypress_binary_entry_name(entry_path: &str) -> bool {
    let normalized = entry_path.replace('\\', "/");
    let Some(base) = normalized.rsplit('/').next() else {
        return false;
    };
    base == "daypress" || base.eq_ignore_ascii_case("daypress.exe")
}

/// Extracts `daypress` / `daypress.exe` from a `.tar.gz` stream using `flate2` and POSIX ustar headers.
pub fn extract_binary_from_tar_gz(bytes: &[u8]) -> Result<Vec<u8>, AppError> {
    let gz = GzDecoder::new(bytes);
    let mut limited = gz.take(MAX_EXTRACTED_BINARY_BYTES + (16 * 1024 * 1024));
    let mut tar_bytes = Vec::new();
    limited.read_to_end(&mut tar_bytes).map_err(|e| {
        AppError::BadRequest(format!("Invalid gzip stream in .tar.gz archive: {e}"))
    })?;

    let mut offset = 0usize;
    while offset + 512 <= tar_bytes.len() {
        let header = &tar_bytes[offset..offset + 512];
        offset += 512;

        // End-of-archive is marked by two 512-byte zero blocks
        if header.iter().all(|&b| b == 0) {
            break;
        }

        let name_raw = &header[0..100];
        let name_len = name_raw.iter().position(|&b| b == 0).unwrap_or(100);
        let name = String::from_utf8_lossy(&name_raw[..name_len]);

        let size_raw = &header[124..136];
        let size_str = String::from_utf8_lossy(size_raw);
        let size_clean = size_str.trim_matches(|c: char| c == '\0' || c.is_ascii_whitespace());
        let entry_size = if size_clean.is_empty() {
            0usize
        } else {
            usize::from_str_radix(size_clean, 8).map_err(|_| {
                AppError::BadRequest("Corrupted octal size field in .tar.gz header.".into())
            })?
        };

        let typeflag = header[156];
        let is_regular_file = typeflag == b'0' || typeflag == 0;

        if offset + entry_size > tar_bytes.len() {
            return Err(AppError::BadRequest(
                "Truncated entry inside .tar.gz release archive.".into(),
            ));
        }

        if is_regular_file && is_daypress_binary_entry_name(&name) && entry_size > 0 {
            return Ok(tar_bytes[offset..offset + entry_size].to_vec());
        }

        let padded = (entry_size + 511) & !511;
        offset = offset.saturating_add(padded);
    }

    Err(AppError::BadRequest(
        "No `daypress` binary found inside the .tar.gz archive.".into(),
    ))
}

/// Extracts `daypress` / `daypress.exe` from a PKZIP `.zip` archive using Central Directory metadata.
pub fn extract_binary_from_zip(bytes: &[u8]) -> Result<Vec<u8>, AppError> {
    // Locate End of Central Directory (EOCD: signature 0x06054b50) in the last 65 KB
    let search_start = bytes.len().saturating_sub(65_557);
    let mut eocd_pos = None;
    for i in (search_start..=bytes.len().saturating_sub(22)).rev() {
        if &bytes[i..i + 4] == b"PK\x05\x06" {
            eocd_pos = Some(i);
            break;
        }
    }
    let eocd = eocd_pos.ok_or_else(|| {
        AppError::BadRequest("Invalid .zip archive (missing EOCD record).".into())
    })?;

    let total_entries = u16::from_le_bytes([bytes[eocd + 10], bytes[eocd + 11]]) as usize;
    let cd_offset = u32::from_le_bytes([
        bytes[eocd + 16],
        bytes[eocd + 17],
        bytes[eocd + 18],
        bytes[eocd + 19],
    ]) as usize;

    let mut pos = cd_offset;
    for _ in 0..total_entries {
        if pos + 46 > bytes.len() || &bytes[pos..pos + 4] != b"PK\x01\x02" {
            break;
        }
        let compression = u16::from_le_bytes([bytes[pos + 10], bytes[pos + 11]]);
        let comp_size = u32::from_le_bytes([
            bytes[pos + 20],
            bytes[pos + 21],
            bytes[pos + 22],
            bytes[pos + 23],
        ]) as usize;
        let uncomp_size = u32::from_le_bytes([
            bytes[pos + 24],
            bytes[pos + 25],
            bytes[pos + 26],
            bytes[pos + 27],
        ]) as u64;
        let name_len = u16::from_le_bytes([bytes[pos + 28], bytes[pos + 29]]) as usize;
        let extra_len = u16::from_le_bytes([bytes[pos + 30], bytes[pos + 31]]) as usize;
        let comment_len = u16::from_le_bytes([bytes[pos + 32], bytes[pos + 33]]) as usize;
        let local_header_offset = u32::from_le_bytes([
            bytes[pos + 42],
            bytes[pos + 43],
            bytes[pos + 44],
            bytes[pos + 45],
        ]) as usize;

        if pos + 46 + name_len > bytes.len() {
            break;
        }
        let name = String::from_utf8_lossy(&bytes[pos + 46..pos + 46 + name_len]);
        pos += 46 + name_len + extra_len + comment_len;

        if !name.ends_with('/') && is_daypress_binary_entry_name(&name) && comp_size > 0 {
            if uncomp_size > MAX_EXTRACTED_BINARY_BYTES {
                return Err(AppError::BadRequest(
                    "Extracted binary exceeds 150 MB safety limit.".into(),
                ));
            }
            if local_header_offset + 30 > bytes.len()
                || &bytes[local_header_offset..local_header_offset + 4] != b"PK\x03\x04"
            {
                return Err(AppError::BadRequest(
                    "Corrupted local file header in .zip archive.".into(),
                ));
            }
            let l_name_len = u16::from_le_bytes([
                bytes[local_header_offset + 26],
                bytes[local_header_offset + 27],
            ]) as usize;
            let l_extra_len = u16::from_le_bytes([
                bytes[local_header_offset + 28],
                bytes[local_header_offset + 29],
            ]) as usize;
            let data_start = local_header_offset + 30 + l_name_len + l_extra_len;
            let data_end = data_start.saturating_add(comp_size);
            if data_end > bytes.len() {
                return Err(AppError::BadRequest(
                    "Truncated compressed data in .zip archive.".into(),
                ));
            }
            let compressed_slice = &bytes[data_start..data_end];
            return match compression {
                0 => Ok(compressed_slice.to_vec()),
                8 => {
                    let decoder = DeflateDecoder::new(compressed_slice);
                    let mut limited = decoder.take(MAX_EXTRACTED_BINARY_BYTES);
                    let mut out = Vec::with_capacity(uncomp_size as usize);
                    limited.read_to_end(&mut out).map_err(|e| {
                        AppError::BadRequest(format!("Deflate decompression failed: {e}"))
                    })?;
                    Ok(out)
                }
                other => Err(AppError::BadRequest(format!(
                    "Unsupported ZIP compression method ({other})."
                ))),
            };
        }
    }

    Err(AppError::BadRequest(
        "No `daypress` or `daypress.exe` binary found inside the .zip archive.".into(),
    ))
}

/// Verifies, smoke-tests, backs up SQLite, and atomically swaps `exe_path` with the binary inside `archive_bytes`.
pub fn apply_binary_update_to_path(
    db: &Database,
    exe_path: &Path,
    archive_bytes: &[u8],
    expected_sha256: Option<&str>,
    target_version: &str,
    action_kind: &str,
) -> Result<UpdateApplyReport, AppError> {
    let env = inspect_environment_for_path(db, exe_path);
    if !env.can_self_update {
        return Err(AppError::BadRequest(format!(
            "In-place binary replacement is not supported in `{}` mode. Create a pre-update SQLite backup and update your container image.",
            env.hosting_mode
        )));
    }

    // 1. Verify SHA-256 digest of the archive
    let computed_sha256 = verify_sha256(archive_bytes, expected_sha256)?;

    // 2. Extract executable payload
    let binary_bytes = extract_daypress_binary(archive_bytes)?;

    // 3. Write to `.staged` and set executable permissions
    let staged_path = staged_binary_path(exe_path);
    fs::write(&staged_path, &binary_bytes).map_err(|e| {
        AppError::Internal(format!(
            "Failed to write staged binary to {}: {e}",
            staged_path.display()
        ))
    })?;
    set_executable_permissions(&staged_path)?;

    // 4. Smoke-test staged binary (`daypress.staged --help`) unless explicitly skipped in cross-arch unit tests
    if let Err(smoke_err) = run_binary_smoke_test(&staged_path) {
        let _ = fs::remove_file(&staged_path);
        return Err(smoke_err);
    }

    // 5. Create automatic pre-update SQLite backup snapshot
    let from_ver = current_version().to_string();
    let clean_to_ver = target_version.trim().trim_start_matches('v').to_string();
    let backup_label = format!("pre-update-v{from_ver}-to-v{clean_to_ver}");
    let snapshot = db.create_sqlite_snapshot(Some(&backup_label))?;

    // 6. Atomic swap: current -> .prev, .staged -> current
    let prev_path = previous_binary_path(exe_path);
    if prev_path.exists() {
        let _ = fs::remove_file(&prev_path);
    }
    if exe_path.exists() {
        fs::rename(exe_path, &prev_path).map_err(|e| {
            let _ = fs::remove_file(&staged_path);
            AppError::Internal(format!(
                "Could not move current binary to {}: {e}",
                prev_path.display()
            ))
        })?;
    }
    if let Err(e) = fs::rename(&staged_path, exe_path) {
        // Immediately roll back the rename so the server binary is never missing
        let _ = fs::rename(&prev_path, exe_path);
        return Err(AppError::Internal(format!(
            "Failed to activate staged binary {}: {e}",
            exe_path.display()
        )));
    }

    // 7. Record update history in SQLite
    let history_entry = UpdateHistoryEntry {
        timestamp: Utc::now().to_rfc3339(),
        action: action_kind.to_string(),
        from_version: from_ver.clone(),
        to_version: clean_to_ver.clone(),
        backup_filename: Some(snapshot.filename.clone()),
        sha256_digest: computed_sha256.clone(),
        status: "completed".to_string(),
        details: format!(
            "Updated Daypress from v{from_ver} to v{clean_to_ver} (backup: {})",
            snapshot.filename
        ),
    };
    append_update_history(db, history_entry)?;

    let restart_scheduled = schedule_process_restart(exe_path);

    Ok(UpdateApplyReport {
        success: true,
        action: action_kind.to_string(),
        from_version: from_ver,
        to_version: clean_to_ver.clone(),
        backup_filename: snapshot.filename,
        sha256_digest: computed_sha256,
        restart_scheduled,
        message: format!(
            "Daypress v{clean_to_ver} verified, staged, and activated. Pre-update SQLite backup created."
        ),
    })
}

/// Rolls back `exe_path` to `{exe_path}.prev` and optionally restores a pre-update SQLite snapshot.
pub fn rollback_binary_at_path(
    db: &Database,
    exe_path: &Path,
    restore_backup_filename: Option<&str>,
) -> Result<UpdateApplyReport, AppError> {
    let prev_path = previous_binary_path(exe_path);
    if !prev_path.exists() {
        return Err(AppError::NotFound(
            "No previous binary (`daypress.prev`) found on disk to roll back to.".into(),
        ));
    }

    let prev_bytes = fs::read(&prev_path)
        .map_err(|e| AppError::Internal(format!("Could not read previous binary: {e}")))?;
    let sha256_digest = verify_sha256(&prev_bytes, None)?;

    run_binary_smoke_test(&prev_path)?;

    let from_ver = current_version().to_string();
    let history = load_update_history(db);
    let target_ver = history
        .first()
        .map(|h| h.from_version.clone())
        .unwrap_or_else(|| "previous".to_string());

    let safety_snap = db.create_sqlite_snapshot(Some(&format!("pre-rollback-v{from_ver}")))?;

    if let Some(backup_file) = restore_backup_filename
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        db.restore_sqlite_snapshot(backup_file)?;
    }

    // Swap exe_path <-> prev_path via temporary path
    let rollback_tmp = staged_binary_path(exe_path);
    let _ = fs::remove_file(&rollback_tmp);
    fs::copy(&prev_path, &rollback_tmp)
        .map_err(|e| AppError::Internal(format!("Failed to stage rollback binary: {e}")))?;
    set_executable_permissions(&rollback_tmp)?;

    let failed_backup = exe_path.with_extension("rolledback");
    let _ = fs::remove_file(&failed_backup);
    if exe_path.exists() {
        fs::rename(exe_path, &failed_backup).map_err(|e| {
            let _ = fs::remove_file(&rollback_tmp);
            AppError::Internal(format!(
                "Failed to move current binary during rollback: {e}"
            ))
        })?;
    }
    if let Err(e) = fs::rename(&rollback_tmp, exe_path) {
        let _ = fs::rename(&failed_backup, exe_path);
        return Err(AppError::Internal(format!(
            "Failed to restore previous binary: {e}"
        )));
    }
    let _ = fs::remove_file(&prev_path);
    let _ = fs::remove_file(&failed_backup);

    let entry = UpdateHistoryEntry {
        timestamp: Utc::now().to_rfc3339(),
        action: "rollback".to_string(),
        from_version: from_ver.clone(),
        to_version: target_ver.clone(),
        backup_filename: Some(safety_snap.filename.clone()),
        sha256_digest: sha256_digest.clone(),
        status: "completed".to_string(),
        details: format!("Rolled back binary from v{from_ver} to v{target_ver}"),
    };
    append_update_history(db, entry)?;

    let restart_scheduled = schedule_process_restart(exe_path);

    Ok(UpdateApplyReport {
        success: true,
        action: "rollback".to_string(),
        from_version: from_ver,
        to_version: target_ver.clone(),
        backup_filename: safety_snap.filename,
        sha256_digest,
        restart_scheduled,
        message: format!("Rolled back to Daypress v{target_ver}."),
    })
}

/// Downloads the latest release asset from `UpdateState`, verifies SHA-256, backs up SQLite, and swaps the binary.
pub async fn download_and_apply_github_update(
    db: &Database,
) -> Result<UpdateApplyReport, AppError> {
    let mut state = load_update_state(db);
    if state.asset_download_url.is_none() {
        state = check_for_updates_now(db).await?;
    }

    let download_url = state.asset_download_url.as_deref().ok_or_else(|| {
        AppError::NotFound(format!(
            "No prebuilt binary asset found for target `{}` on the latest release.",
            current_target_triple()
        ))
    })?;
    let target_version = state
        .latest_version
        .clone()
        .unwrap_or_else(|| current_version().to_string());

    let client = reqwest::Client::builder()
        .user_agent(format!("Daypress-Updater/{}", current_version()))
        .timeout(Duration::from_secs(120))
        .build()
        .map_err(|e| AppError::Internal(format!("HTTP client initialization failed: {e}")))?;

    let resp =
        client.get(download_url).send().await.map_err(|e| {
            AppError::BadRequest(format!("Failed to download release archive: {e}"))
        })?;
    if !resp.status().is_success() {
        return Err(AppError::BadRequest(format!(
            "Release asset download returned HTTP {}",
            resp.status()
        )));
    }
    let archive_bytes = resp
        .bytes()
        .await
        .map_err(|e| AppError::BadRequest(format!("Interrupted release archive download: {e}")))?;

    let exe_path = std::env::current_exe().map_err(|e| {
        AppError::Internal(format!("Could not resolve current executable path: {e}"))
    })?;

    apply_binary_update_to_path(
        db,
        &exe_path,
        &archive_bytes,
        state.sha256_digest.as_deref(),
        &target_version,
        "update",
    )
}

#[cfg(unix)]
fn set_executable_permissions(path: &Path) -> Result<(), AppError> {
    use std::os::unix::fs::PermissionsExt;
    let perms = fs::Permissions::from_mode(0o755);
    fs::set_permissions(path, perms).map_err(|e| {
        AppError::Internal(format!(
            "Failed to set executable permissions on {}: {e}",
            path.display()
        ))
    })
}

#[cfg(not(unix))]
fn set_executable_permissions(_path: &Path) -> Result<(), AppError> {
    Ok(())
}

fn run_binary_smoke_test(binary_path: &Path) -> Result<(), AppError> {
    if std::env::var("DAYPRESS_SKIP_BINARY_SMOKE_TEST")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return Ok(());
    }

    let output = Command::new(binary_path)
        .arg("--help")
        .output()
        .map_err(|e| {
            AppError::BadRequest(format!(
                "Staged binary failed smoke test execution (`--help`): {e}"
            ))
        })?;

    if !output.status.success() {
        return Err(AppError::BadRequest(format!(
            "Staged binary exited with non-zero status ({}) during `--help` smoke test.",
            output.status
        )));
    }
    Ok(())
}

fn schedule_process_restart(exe_path: &Path) -> bool {
    let Ok(current_exe) = std::env::current_exe() else {
        return false;
    };
    // Never restart the process if we updated a test fixture path rather than the actual running binary
    if current_exe != exe_path {
        return false;
    }
    if std::env::var("DAYPRESS_DISABLE_RESTART_ON_UPDATE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return false;
    }

    let exe_owned = exe_path.to_path_buf();
    let args: Vec<String> = std::env::args().skip(1).collect();

    tokio::spawn(async move {
        // Give Axum 750ms to flush the HTTP 200 OK JSON response to the browser
        tokio::time::sleep(Duration::from_millis(750)).await;
        let _ = Command::new(&exe_owned).args(&args).spawn();
        std::process::exit(0);
    });

    true
}
