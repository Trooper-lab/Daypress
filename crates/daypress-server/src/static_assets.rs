//! Static assets compiled into the binary (`static/`), so `daypress` serves its favicon, logos,
//! desk fonts and starter previews from any working directory.
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
    asset!("favicon.svg"),
    asset!("daypress-icon.svg"),
    asset!("daypress-logo.svg"),
    asset!("daypress-logo-paper.svg"),
    asset!("placeholder.svg"),
    asset!("fonts/Newsreader-Variable.woff2"),
    asset!("fonts/Newsreader-Italic-Variable.woff2"),
    asset!("fonts/SchibstedGrotesk-Variable.woff2"),
    asset!("fonts/IBMPlexMono-Regular.woff2"),
    asset!("fonts/IBMPlexMono-Medium.woff2"),
    asset!("fonts/OFL.txt"),
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
        "svg" => "image/svg+xml",
        "woff2" => "font/woff2",
        "txt" => "text/plain; charset=utf-8",
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

/// `/static/*`.
pub async fn serve_static(AxPath(path): AxPath<String>) -> Response {
    asset_response(&path)
}
