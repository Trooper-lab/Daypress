use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use daypress_server::{
    create_app,
    updater::{
        apply_binary_update_to_path, build_update_state_from_release,
        cached_available_update_version, expected_asset_filename, extract_daypress_binary,
        get_update_status, is_newer_version, load_update_history, load_update_settings,
        load_update_state, parse_sha256sums, previous_binary_path, rollback_binary_at_path,
        save_update_state, verify_sha256, GithubAssetDto, GithubReleaseDto, UpdateSettings,
    },
    Database,
};
use flate2::{write::DeflateEncoder, write::GzEncoder, Compression};
use std::io::Write;
use tower::ServiceExt;

const ADMIN_AUTH: &str = "Basic YWRtaW46YWRtaW4=";

/// Helper: constructs a valid POSIX ustar `.tar.gz` archive containing `entry_name` with `payload`.
fn build_test_tar_gz(entry_name: &str, payload: &[u8]) -> Vec<u8> {
    let mut tar = Vec::new();
    let mut header = [0u8; 512];

    let name_bytes = entry_name.as_bytes();
    header[..name_bytes.len()].copy_from_slice(name_bytes);
    // File mode 0000755
    header[100..107].copy_from_slice(b"0000755");
    // File size in octal (11 digits + NUL)
    let size_octal = format!("{:011o}\0", payload.len());
    header[124..136].copy_from_slice(size_octal.as_bytes());
    // Typeflag '0' (regular file)
    header[156] = b'0';
    // ustar magic
    header[257..263].copy_from_slice(b"ustar\0");

    // Checksum field treated as spaces during sum
    header[148..156].fill(b' ');
    let chksum: u32 = header.iter().map(|&b| b as u32).sum();
    let chk_str = format!("{:06o}\0 ", chksum);
    header[148..156].copy_from_slice(chk_str.as_bytes());

    tar.extend_from_slice(&header);
    tar.extend_from_slice(payload);
    let rem = payload.len() % 512;
    if rem != 0 {
        tar.extend(std::iter::repeat_n(0u8, 512 - rem));
    }
    // Two 512-byte zero blocks mark end of archive
    tar.extend(std::iter::repeat_n(0u8, 1024));

    let mut gz = GzEncoder::new(Vec::new(), Compression::default());
    gz.write_all(&tar).unwrap();
    gz.finish().unwrap()
}

/// Helper: constructs a valid PKZIP `.zip` archive (Deflate method 8) containing `entry_name` with `payload`.
fn build_test_zip(entry_name: &str, payload: &[u8]) -> Vec<u8> {
    let mut deflater = DeflateEncoder::new(Vec::new(), Compression::default());
    deflater.write_all(payload).unwrap();
    let compressed = deflater.finish().unwrap();

    let name_bytes = entry_name.as_bytes();
    let mut zip = Vec::new();

    // 1. Local File Header (0x04034b50)
    let local_header_offset = 0u32;
    zip.extend_from_slice(b"PK\x03\x04");
    zip.extend_from_slice(&20u16.to_le_bytes()); // version needed
    zip.extend_from_slice(&0u16.to_le_bytes()); // flags
    zip.extend_from_slice(&8u16.to_le_bytes()); // compression method 8 = Deflate
    zip.extend_from_slice(&0u16.to_le_bytes()); // mod time
    zip.extend_from_slice(&0u16.to_le_bytes()); // mod date
    zip.extend_from_slice(&0u32.to_le_bytes()); // crc32 (not required by extractor, SHA-256 verifies archive)
    zip.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    zip.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    zip.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
    zip.extend_from_slice(&0u16.to_le_bytes()); // extra len
    zip.extend_from_slice(name_bytes);
    zip.extend_from_slice(&compressed);

    // 2. Central Directory Header (0x02014b50)
    let cd_offset = zip.len() as u32;
    zip.extend_from_slice(b"PK\x01\x02");
    zip.extend_from_slice(&20u16.to_le_bytes()); // version made by
    zip.extend_from_slice(&20u16.to_le_bytes()); // version needed
    zip.extend_from_slice(&0u16.to_le_bytes()); // flags
    zip.extend_from_slice(&8u16.to_le_bytes()); // compression = Deflate
    zip.extend_from_slice(&0u16.to_le_bytes()); // time
    zip.extend_from_slice(&0u16.to_le_bytes()); // date
    zip.extend_from_slice(&0u32.to_le_bytes()); // crc32
    zip.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    zip.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    zip.extend_from_slice(&(name_bytes.len() as u16).to_le_bytes());
    zip.extend_from_slice(&0u16.to_le_bytes()); // extra len
    zip.extend_from_slice(&0u16.to_le_bytes()); // comment len
    zip.extend_from_slice(&0u16.to_le_bytes()); // disk number
    zip.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
    zip.extend_from_slice(&0u32.to_le_bytes()); // external attrs
    zip.extend_from_slice(&local_header_offset.to_le_bytes());
    zip.extend_from_slice(name_bytes);

    let cd_size = (zip.len() as u32) - cd_offset;

    // 3. End of Central Directory (0x06054b50)
    zip.extend_from_slice(b"PK\x05\x06");
    zip.extend_from_slice(&0u16.to_le_bytes()); // disk
    zip.extend_from_slice(&0u16.to_le_bytes()); // cd start disk
    zip.extend_from_slice(&1u16.to_le_bytes()); // entries on disk
    zip.extend_from_slice(&1u16.to_le_bytes()); // total entries
    zip.extend_from_slice(&cd_size.to_le_bytes());
    zip.extend_from_slice(&cd_offset.to_le_bytes());
    zip.extend_from_slice(&0u16.to_le_bytes()); // comment len

    zip
}

#[test]
fn test_semver_and_sha256sums_parsing() {
    assert!(is_newer_version("0.1.1", "0.1.2"));
    assert!(is_newer_version("v0.1.1", "v0.2.0"));
    assert!(!is_newer_version("0.1.1", "0.1.1"));
    assert!(!is_newer_version("0.2.0", "0.1.9"));

    let sums = "\
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  daypress-v0.2.0-x86_64-unknown-linux-gnu.tar.gz
aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa *daypress-v0.2.0-x86_64-pc-windows-msvc.zip
";
    assert_eq!(
        parse_sha256sums(sums, "daypress-v0.2.0-x86_64-unknown-linux-gnu.tar.gz").as_deref(),
        Some("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855")
    );
    assert_eq!(
        parse_sha256sums(sums, "daypress-v0.2.0-x86_64-pc-windows-msvc.zip").as_deref(),
        Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );
    assert!(parse_sha256sums(sums, "nonexistent.tar.gz").is_none());

    assert_eq!(
        expected_asset_filename("0.2.0", "x86_64-unknown-linux-gnu"),
        "daypress-v0.2.0-x86_64-unknown-linux-gnu.tar.gz"
    );
    assert_eq!(
        expected_asset_filename("v0.2.0", "x86_64-pc-windows-msvc"),
        "daypress-v0.2.0-x86_64-pc-windows-msvc.zip"
    );
}

#[test]
fn test_archive_extractors_tar_gz_and_zip() {
    let fake_elf = b"\x7fELF-daypress-binary-payload-v0.2.0";

    // 1. Test .tar.gz extraction
    let tar_gz = build_test_tar_gz(
        "daypress-v0.2.0-x86_64-unknown-linux-gnu/daypress",
        fake_elf,
    );
    let extracted_tar = extract_daypress_binary(&tar_gz).expect("Should extract from .tar.gz");
    assert_eq!(extracted_tar, fake_elf);

    // 2. Test .zip extraction
    let fake_pe = b"MZ-daypress-windows-binary-payload-v0.2.0";
    let zip_bytes = build_test_zip(
        "daypress-v0.2.0-x86_64-pc-windows-msvc/daypress.exe",
        fake_pe,
    );
    let extracted_zip = extract_daypress_binary(&zip_bytes).expect("Should extract from .zip");
    assert_eq!(extracted_zip, fake_pe);

    // 3. Test SHA-256 verification success & mismatch rejection
    let digest = verify_sha256(&zip_bytes, None).unwrap();
    assert_eq!(verify_sha256(&zip_bytes, Some(&digest)).unwrap(), digest);
    assert!(verify_sha256(
        &zip_bytes,
        Some("0000000000000000000000000000000000000000000000000000000000000000")
    )
    .is_err());
}

#[test]
fn test_atomic_binary_swap_backup_and_rollback() {
    std::env::set_var("DAYPRESS_SKIP_BINARY_SMOKE_TEST", "1");
    std::env::set_var("DAYPRESS_DISABLE_RESTART_ON_UPDATE", "1");

    let db = Database::new_in_memory().expect("in-memory DB");
    let temp_dir = std::env::temp_dir().join(format!("daypress_upd_test_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&temp_dir);
    std::fs::create_dir_all(&temp_dir).unwrap();

    let exe_path = temp_dir.join("daypress.exe");
    let v1_bytes = b"MZ-daypress-v0.1.1-original-binary";
    let v2_bytes = b"MZ-daypress-v0.2.0-updated-binary";
    std::fs::write(&exe_path, v1_bytes).unwrap();

    let zip_v2 = build_test_zip(
        "daypress-v0.2.0-x86_64-pc-windows-msvc/daypress.exe",
        v2_bytes,
    );
    let expected_sha = verify_sha256(&zip_v2, None).unwrap();

    // Apply update to v0.2.0
    let report = apply_binary_update_to_path(
        &db,
        &exe_path,
        &zip_v2,
        Some(&expected_sha),
        "0.2.0",
        "update",
    )
    .expect("Binary update should succeed");

    assert!(report.success);
    assert_eq!(report.to_version, "0.2.0");
    assert!(!report.backup_filename.is_empty());
    assert_eq!(std::fs::read(&exe_path).unwrap(), v2_bytes);

    let prev_path = previous_binary_path(&exe_path);
    assert!(prev_path.exists());
    assert_eq!(std::fs::read(&prev_path).unwrap(), v1_bytes);

    let history = load_update_history(&db);
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].action, "update");
    assert_eq!(history[0].to_version, "0.2.0");

    // Now test 1-click Rollback
    let rb_report = rollback_binary_at_path(&db, &exe_path, None).expect("Rollback should succeed");
    assert!(rb_report.success);
    assert_eq!(rb_report.action, "rollback");
    assert_eq!(std::fs::read(&exe_path).unwrap(), v1_bytes);
    assert!(!prev_path.exists());

    let history_after = load_update_history(&db);
    assert_eq!(history_after.len(), 2);
    assert_eq!(history_after[0].action, "rollback");

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[tokio::test]
async fn test_update_api_routes_and_admin_updates_screen() {
    let db = Database::new_in_memory().expect("in-memory DB");

    // Simulate a discovered GitHub Release v9.9.0 persisted in SQLite
    let release = GithubReleaseDto {
        tag_name: "v9.9.0".to_string(),
        html_url: "https://github.com/Trooper-lab/daypress/releases/tag/v9.9.0".to_string(),
        body: Some(
            "### Highlights\n- Instant in-app self-updater\n- Automatic SQLite pre-update snapshot"
                .to_string(),
        ),
        prerelease: false,
        draft: false,
        published_at: Some("2026-10-30T16:00:00Z".to_string()),
        assets: vec![GithubAssetDto {
            name: expected_asset_filename(
                "v9.9.0",
                &daypress_server::updater::current_target_triple(),
            ),
            browser_download_url:
                "https://github.com/Trooper-lab/daypress/releases/download/v9.9.0/asset".to_string(),
            size: 12_345_678,
        }],
    };
    let sums = format!(
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb  {}\n",
        expected_asset_filename("v9.9.0", &daypress_server::updater::current_target_triple())
    );
    let state = build_update_state_from_release(
        &release,
        &daypress_server::updater::current_target_triple(),
        Some(&sums),
    );
    save_update_state(&db, &state).unwrap();

    assert_eq!(
        cached_available_update_version(&db).as_deref(),
        Some("9.9.0")
    );
    let status = get_update_status(&db);
    assert!(status.update_available);
    assert_eq!(
        status.state.sha256_digest.as_deref(),
        Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
    );

    let app = create_app(db.clone());

    // 1. GET /wp-json/daypress/v1/update/status
    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/update/status")
                .header("Authorization", ADMIN_AUTH)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res.status(), StatusCode::OK);
    let body_bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
    assert_eq!(json["update_available"], true);
    assert_eq!(json["state"]["latest_version"], "9.9.0");

    // 2. POST /wp-json/daypress/v1/update/settings
    let new_settings = UpdateSettings {
        auto_check_enabled: false,
        channel: "prerelease".to_string(),
        github_repo: "Trooper-lab/daypress".to_string(),
    };
    let res_settings = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/update/settings")
                .header("Authorization", ADMIN_AUTH)
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_settings).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_settings.status(), StatusCode::OK);
    let loaded_settings = load_update_settings(&db);
    assert!(!loaded_settings.auto_check_enabled);
    assert_eq!(loaded_settings.channel, "prerelease");

    // 3. GET /d-admin/update-core renders the new release card, sidebar badge, and manual upload form
    let res_html = app
        .oneshot(
            Request::builder()
                .uri("/d-admin/update-core")
                .header("Authorization", ADMIN_AUTH)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(res_html.status(), StatusCode::OK);
    let html_bytes = to_bytes(res_html.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8_lossy(&html_bytes);
    assert!(html.contains("Daypress v9.9.0 is ready"));
    assert!(html.contains("Instant in-app self-updater"));
    assert!(html.contains("Manual Release Archive Upload"));
    assert!(html.contains("Daypress v9.9.0 available"));
    assert!(load_update_state(&db).latest_version.as_deref() == Some("9.9.0"));
}
