//! Shared test harness helpers for in-process multi-site stress tests.

use axum::body::to_bytes;
use axum::response::Response;
use neopress_server::Database;
use std::fs;
use std::path::{Path, PathBuf};

/// Creates an isolated in-memory SQLite database for testing.
pub fn setup_test_db() -> Database {
    Database::new_in_memory().expect("Failed to initialize test SQLite DB")
}

/// Asynchronously extracts the response body into a UTF-8 String.
pub async fn body_to_string(res: Response) -> String {
    let bytes = to_bytes(res.into_body(), usize::MAX)
        .await
        .expect("Failed to read response body");
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Resolves a fixture path, checking workspace-root and crate-relative locations.
pub fn resolve_fixture_path(relative_subpath: &str) -> Option<PathBuf> {
    let candidates = [
        format!("fixtures/stress_test/{}", relative_subpath),
        format!("../../fixtures/stress_test/{}", relative_subpath),
        format!("../fixtures/stress_test/{}", relative_subpath),
        relative_subpath.to_string(),
        format!("../../{}", relative_subpath),
        format!("../{}", relative_subpath),
    ];
    for c in &candidates {
        let p = Path::new(c);
        if p.exists() {
            return Some(p.to_path_buf());
        }
    }
    None
}

/// Loads fixture content from disk if available, or returns the embedded fallback.
pub fn load_fixture_or_fallback(relative_subpath: &str, fallback: &str) -> String {
    if let Some(path) = resolve_fixture_path(relative_subpath) {
        if let Ok(content) = fs::read_to_string(path) {
            return content;
        }
    }
    fallback.to_string()
}

/// Asserts zero raw unparsed block comment leaks and zero editor chrome leaks in SSR HTML.
pub fn assert_zero_comment_leaks(html: &str, context: &str) {
    assert!(
        !html.contains("<!-- wp:"),
        "Leak detected in {}: Found unparsed '<!-- wp:' block comment",
        context
    );
    assert!(
        !html.contains("<!-- /wp:"),
        "Leak detected in {}: Found unparsed '<!-- /wp:' block comment",
        context
    );
    assert!(
        !html.contains("contenteditable=\"true\""),
        "Leak detected in {}: Found editor-only attribute 'contenteditable=\"true\"'",
        context
    );
    assert!(
        !html.contains(".elementor-grid-handle"),
        "Leak detected in {}: Found editor-only selector '.elementor-grid-handle'",
        context
    );
    assert!(
        !html.contains(".neostudio-handle"),
        "Leak detected in {}: Found editor-only selector '.neostudio-handle'",
        context
    );
}
