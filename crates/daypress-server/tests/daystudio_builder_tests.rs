//! Comprehensive integration and verification tests for DayStudio 2026 Visual Builder UI.

use axum::body::{to_bytes, Body};
use axum::http::{Request, StatusCode};
use base64::Engine;
use daypress_server::adapters::database::Database;
use daypress_server::create_app;
use daypress_server::templates::render_builder_view;
use tower::ServiceExt;

#[test]
fn test_daystudio_markup_contains_2026_cad_tokens_and_shell() {
    let db = Database::new_in_memory().unwrap();
    let settings = db.get_settings().unwrap_or_default();
    let posts = db.get_posts("post", None, None, 1).unwrap();
    let post = &posts[0];

    let sample_layout = serde_json::json!({
        "version": 1,
        "root": {
            "id": "root",
            "type": "Root",
            "children": [
                {
                    "id": "sec_hero",
                    "type": "Section",
                    "styles": { "background_color": "#0D0F12", "padding_top": "64px" },
                    "children": [
                        {
                            "id": "price_1",
                            "type": "PriceCard",
                            "data": { "title": "Enterprise Cloud", "price": "199", "currency": "$" }
                        }
                    ]
                }
            ]
        }
    })
    .to_string();

    let html = render_builder_view(&settings, post, Some(&sample_layout));

    // 1. Verify the theme class and the self-hosted brand typefaces
    assert!(html.contains("class=\"daystudio-theme-active\""));
    assert!(html.contains("/static/fonts/SchibstedGrotesk-Variable.woff2"));
    assert!(html.contains("/static/fonts/IBMPlexMono-Regular.woff2"));
    assert!(!html.contains("fonts.googleapis.com/css2?family=Inter"));

    // 2. Verify the Night edition tokens and signals in CSS
    assert!(html.contains("--day-chassis:         #111214"));
    assert!(html.contains("--day-surface:         #1A1C1F"));
    assert!(html.contains("--day-rule:            #393836"));
    assert!(html.contains("--day-ink:             #EFE9DE"));
    assert!(html.contains("--day-muted:           #A7A297"));
    assert!(html.contains("--day-signal:          #93A1FF"));
    assert!(html.contains("--day-token:           #E6B84A"));

    // 3. Verify 3-Pane Zero-Scroll Dock Shell Elements
    assert!(html.contains("id=\"daystudio-shell\""));
    assert!(html.contains("id=\"daystudio-topbar\""));
    assert!(html.contains("id=\"daystudio-dock\""));
    assert!(html.contains("id=\"daystudio-rail\""));
    assert!(html.contains("id=\"daystudio-drawer\""));
    assert!(html.contains("id=\"daystudio-canvas-viewport\""));
    assert!(html.contains("id=\"daystudio-artboard\""));
    assert!(html.contains("id=\"daystudio-inspector\""));
    assert!(html.contains("id=\"daystudio-bottom-bar\""));

    // 4. Verify Responsive Handles, Selection Chrome & Overlays
    assert!(html.contains("id=\"handle-artboard-left\""));
    assert!(html.contains("id=\"handle-artboard-right\""));
    assert!(html.contains("id=\"daystudio-selection-chrome\""));
    assert!(html.contains("id=\"act-drag-node\""));
    assert!(html.contains("id=\"act-parent-node\""));
    assert!(html.contains("id=\"act-dup-node\""));
    assert!(html.contains("id=\"act-del-node\""));
    assert!(html.contains("id=\"daystudio-box-overlays\""));
    assert!(html.contains("id=\"overlay-margin-top\""));
    assert!(html.contains("id=\"overlay-padding-top\""));

    // 5. Verify Modals and Command Palette
    assert!(html.contains("id=\"modal-finder\""));
    assert!(html.contains("id=\"icon-library-modal\""));
    assert!(html.contains("id=\"settings-modal\""));
    assert!(html.contains("id=\"modal-conditions\""));
    assert!(html.contains("id=\"ast-modal\""));

    // 6. Verify JavaScript Bundles
    assert!(html.contains("window.DayStudioState"));
    assert!(html.contains("window.StudioApi"));
    assert!(html.contains("renderBoxModelWidget"));
    assert!(html.contains("renderProWidget"));
    assert!(html.contains("updateNavigator"));
}

#[tokio::test]
async fn test_daystudio_sqlite_persistence_and_dual_write() {
    let db = Database::new_in_memory().unwrap();
    let app = create_app(db.clone());

    let admin_auth = format!(
        "Basic {}",
        base64::engine::general_purpose::STANDARD.encode("admin:admin")
    );

    // 1. Save complex layout containing 2026 Pro Blocks
    let payload = serde_json::json!({
        "layout": {
            "version": 1,
            "root": {
                "id": "root",
                "type": "Root",
                "children": [
                    {
                        "id": "sec_pro_1",
                        "type": "Section",
                        "styles": { "padding_top": "80px", "background_color": "#15181E" },
                        "children": [
                            {
                                "id": "price_pro_1",
                                "type": "PriceCard",
                                "data": {
                                    "title": "DayStudio Pro",
                                    "currency": "$",
                                    "price": "99",
                                    "period": "/mo",
                                    "badge": "FEATURED",
                                    "featured": true,
                                    "button_text": "Launch Studio"
                                }
                            },
                            {
                                "id": "countdown_1",
                                "type": "Countdown",
                                "data": {
                                    "target_date": "2026-12-31T23:59:59Z",
                                    "expiry_message": "Offer Concluded"
                                }
                            },
                            {
                                "id": "flipbox_1",
                                "type": "FlipBox",
                                "data": {
                                    "front_title": "Tokio Async Engine",
                                    "back_title": "Zero Latency"
                                }
                            }
                        ]
                    }
                ]
            }
        },
        "layout_json": {
            "version": 1,
            "root": {
                "id": "root",
                "type": "Root",
                "children": []
            }
        }
    });

    let res = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/wp-json/daypress/v1/layout/1")
                .header("Authorization", &admin_auth)
                .header("Content-Type", "application/json")
                .body(Body::from(payload.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(res.status(), StatusCode::OK);

    // 2. Fetch saved layout from SQLite and confirm persistence
    let get_res = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/wp-json/daypress/v1/layout/1")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(get_res.status(), StatusCode::OK);
    let body = to_bytes(get_res.into_body(), usize::MAX).await.unwrap();
    let fetched_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(fetched_json["post_id"], 1);
    let root = &fetched_json["layout"]["root"];
    let first_sec = &root["children"][0];
    assert_eq!(first_sec["type"], "Section");
    let price_card = &first_sec["children"][0];
    assert_eq!(price_card["type"], "PriceCard");
    assert_eq!(price_card["data"]["title"], "DayStudio Pro");
    assert_eq!(price_card["data"]["price"], "99");
}

fn visit_dirs(dir: &std::path::Path, cb: &mut dyn FnMut(&std::path::Path)) {
    if dir.is_dir() {
        for entry in std::fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, cb);
            } else {
                cb(&path);
            }
        }
    }
}

#[test]
fn test_all_builder_rust_files_strictly_under_400_lines() {
    let builder_dir = std::path::Path::new("src/templates/admin/builder");
    let full_path = if builder_dir.exists() {
        builder_dir.to_path_buf()
    } else {
        std::path::Path::new("crates/daypress-server/src/templates/admin/builder").to_path_buf()
    };

    assert!(
        full_path.exists(),
        "Builder directory must exist at {:?}",
        full_path
    );

    let mut visited_count = 0;
    visit_dirs(&full_path, &mut |path| {
        if path.extension().and_then(|s| s.to_str()) == Some("rs") {
            let content = std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
            let line_count = content.lines().count();
            assert!(
                line_count <= 400,
                "File {:?} has {} lines, which violates the strict <= 400 lines rule!",
                path,
                line_count
            );
            visited_count += 1;
        }
    });

    assert!(
        visited_count >= 30,
        "Expected at least 30 modular files, found {}",
        visited_count
    );
}
