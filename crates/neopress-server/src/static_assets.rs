//! Static assets compiled into the binary (`static/`), so `neopress` serves its favicon, icons,
//! starter previews, and WordPress-compatible CSS/fonts from any working directory.
//! Theme files imported at runtime still live on disk under `static/themes/`.

use axum::{
    body::Body,
    extract::Path as AxPath,
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};

macro_rules! asset {
    ($path:literal) => {
        (
            $path,
            include_bytes!(concat!("../static/", $path)) as &'static [u8],
        )
    };
}

static ASSETS: &[(&str, &[u8])] = &[
    asset!("admin-menu.min.css"),
    asset!("common.min.css"),
    asset!("dashboard.min.css"),
    asset!("dashicons.min.css"),
    asset!("edit.min.css"),
    asset!("favicon.svg"),
    asset!("forms.min.css"),
    asset!("list-tables.min.css"),
    asset!("neopress-icon.svg"),
    asset!("neopress-logo.svg"),
    asset!("placeholder.svg"),
    asset!("fonts/dashicons.eot"),
    asset!("fonts/dashicons.svg"),
    asset!("fonts/dashicons.ttf"),
    asset!("fonts/dashicons.woff"),
    asset!("fonts/dashicons.woff2"),
    asset!("starters/agency-header.svg"),
    asset!("starters/clean-footer.svg"),
    asset!("starters/dark-footer.svg"),
    asset!("starters/saas-header.svg"),
];

/// Returns the embedded file at `path` (relative to `static/`), if any.
pub fn embedded_asset(path: &str) -> Option<&'static [u8]> {
    let clean = path.trim_start_matches('/');
    ASSETS
        .iter()
        .find(|(name, _)| *name == clean)
        .map(|(_, bytes)| *bytes)
}

fn content_type(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "css" => "text/css; charset=utf-8",
        "svg" => "image/svg+xml",
        "woff2" => "font/woff2",
        "woff" => "font/woff",
        "ttf" => "font/ttf",
        "eot" => "application/vnd.ms-fontobject",
        _ => "application/octet-stream",
    }
}

fn asset_response(path: &str) -> Response {
    match embedded_asset(path) {
        Some(bytes) => {
            let mut res = Body::from(bytes).into_response();
            let headers = res.headers_mut();
            headers.insert(
                header::CONTENT_TYPE,
                HeaderValue::from_static(content_type(path)),
            );
            headers.insert(
                header::CACHE_CONTROL,
                HeaderValue::from_static("public, max-age=86400"),
            );
            res
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

/// `/static/*`, `/wp-includes/css/*`, and `/wp-admin/css/*`.
pub async fn serve_static(AxPath(path): AxPath<String>) -> Response {
    asset_response(&path)
}

/// `/wp-includes/fonts/*` (where WordPress's `dashicons.min.css` looks for its font files).
pub async fn serve_font(AxPath(path): AxPath<String>) -> Response {
    asset_response(&format!("fonts/{}", path.trim_start_matches('/')))
}
