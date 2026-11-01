//! Onboarding, Getting Started AI Agent Wizard, and Login/Logout Handlers.

#![allow(unused_imports)]
use super::*;
use crate::db::Database;
use crate::models;
use crate::templates;
use axum::{
    extract::{Form, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse},
};
use serde::Deserialize;

pub async fn handle_admin_getting_started(State(db): State<Database>) -> Html<String> {
    // Mark the 1-time automatic redirect as seen once the user opens the launchpad,
    // while keeping `/d-admin/getting-started` accessible from the sidebar anytime.
    let _ = db.dismiss_getting_started();
    let settings = db.get_settings().unwrap_or_default();
    let stats = db.get_stats().unwrap_or_default();
    let db_path = db.db_path_display();
    Html(templates::render_getting_started_view(
        &settings, &stats, &db_path,
    ))
}

pub async fn handle_api_getting_started_dismiss(State(db): State<Database>) -> impl IntoResponse {
    let _ = db.dismiss_getting_started();
    (
        StatusCode::OK,
        axum::Json(serde_json::json!({ "success": true })),
    )
}

#[derive(Debug, Default, Deserialize)]
pub struct AgentKeyGeneratePayload {
    pub name: Option<String>,
}

pub async fn handle_api_getting_started_agent_key(
    State(db): State<Database>,
    payload: Option<axum::Json<AgentKeyGeneratePayload>>,
) -> impl IntoResponse {
    let admin = crate::auth::current_admin();
    let user_id = admin.as_ref().map(|a| a.user_id).unwrap_or(1);
    let username = admin
        .as_ref()
        .map(|a| a.username.clone())
        .unwrap_or_else(|| "admin".to_string());
    let key_name = payload
        .and_then(|p| p.0.name)
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "AI Agent (MCP & REST)".to_string());

    match db.create_application_password(user_id, &key_name, Some("daypress-mcp-agent")) {
        Ok(created) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": true,
                "username": username,
                "uuid": created.uuid,
                "name": created.name,
                "password": created.password.unwrap_or_default()
            })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({
                "success": false,
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn handle_login_page(
    State(db): State<Database>,
    Query(params): Query<LoginQueryParams>,
) -> impl IntoResponse {
    if params.action.as_deref() == Some("logout") {
        return handle_logout().await.into_response();
    }
    if db.needs_onboarding() {
        return axum::response::Redirect::to("/setup").into_response();
    }

    let settings = db.get_settings().unwrap_or_default();
    let loggedout =
        params.loggedout.as_deref() == Some("true") || params.loggedout.as_deref() == Some("1");
    let error_msg = params.error.as_deref();
    let redirect_to = params.redirect_to.as_deref();
    if params.action.as_deref() == Some("resetpass") && error_msg.is_none() {
        let notice =
            templates::LoginNotice::Info("Your password has been reset. You can now log in.");
        let html = templates::render_login_view_with_notice(&settings, notice, redirect_to);
        return Html(html).into_response();
    }

    Html(templates::render_login_view(
        &settings,
        error_msg,
        loggedout,
        redirect_to,
    ))
    .into_response()
}

#[derive(Debug, Default, Deserialize)]
pub struct NetworkPermissionProbePayload {
    pub mode: Option<String>,
}

pub async fn handle_setup_network_permission(
    State(db): State<Database>,
    payload: Option<axum::Json<NetworkPermissionProbePayload>>,
) -> impl IntoResponse {
    let mode = payload
        .and_then(|p| p.0.mode)
        .filter(|s| s.trim().eq_ignore_ascii_case("lan"))
        .map(|_| "lan")
        .unwrap_or("localhost");

    let _ = db.set_option("daypress_network_bind_mode", mode);

    let bind_target = if mode == "lan" {
        std::net::SocketAddr::from(([0, 0, 0, 0], 0))
    } else {
        std::net::SocketAddr::from(([127, 0, 0, 1], 0))
    };

    match tokio::net::TcpListener::bind(bind_target).await {
        Ok(listener) => {
            let local = listener
                .local_addr()
                .map(|a| a.to_string())
                .unwrap_or_else(|_| bind_target.to_string());
            drop(listener);
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({
                    "success": true,
                    "mode": mode,
                    "bound_test_addr": local
                })),
            )
                .into_response()
        }
        Err(e) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({
                "success": false,
                "mode": mode,
                "error": e.to_string()
            })),
        )
            .into_response(),
    }
}

pub async fn handle_setup_page(State(db): State<Database>) -> impl IntoResponse {
    if !db.needs_onboarding() {
        return axum::response::Redirect::to("/d-admin/").into_response();
    }
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_onboarding_view(&settings, None, None)).into_response()
}

pub async fn handle_setup_submit(
    State(db): State<Database>,
    Form(form): Form<crate::db::OnboardingSetupRequest>,
) -> impl IntoResponse {
    if !db.needs_onboarding() {
        return axum::response::Redirect::to("/login").into_response();
    }

    match db.complete_onboarding(&form) {
        Ok(user) => {
            let has_inline_import = !form.import_payload.trim().is_empty();
            if has_inline_import {
                if form.import_format.trim().eq_ignore_ascii_case("json") {
                    if let Ok(bundle) =
                        serde_json::from_str::<models::ImportBundleRequest>(&form.import_payload)
                    {
                        let _ = db.import_bundle(&bundle);
                    }
                } else {
                    let _ = db.import_wxr(std::io::Cursor::new(form.import_payload.as_bytes()));
                }
            }

            let mut headers = HeaderMap::new();
            let secret = db.get_auth_secret().unwrap_or_default();
            let ver = db
                .get_user_detail(user.user_id)
                .ok()
                .flatten()
                .map(|u| u.session_ver)
                .unwrap_or(1);
            let cookie_val =
                crate::auth::session_cookie_header_with_version(user.user_id, &secret, true, ver);
            if let Ok(val) = cookie_val.parse() {
                headers.insert(header::SET_COOKIE, val);
            }
            let target =
                if form.starter_mode.trim().eq_ignore_ascii_case("migrate") && !has_inline_import {
                    "/d-admin/import?welcome=1"
                } else {
                    "/?welcome=1"
                };
            if let Ok(loc) = target.parse() {
                headers.insert(header::LOCATION, loc);
            }
            (StatusCode::SEE_OTHER, headers, Html(String::new())).into_response()
        }
        Err(err) => {
            let settings = db.get_settings().unwrap_or_default();
            let msg = match err {
                crate::AppError::BadRequest(m)
                | crate::AppError::Forbidden(m)
                | crate::AppError::NotFound(m)
                | crate::AppError::Unauthorized(m) => m,
                other => other.to_string(),
            };
            (
                StatusCode::BAD_REQUEST,
                Html(templates::render_onboarding_view(
                    &settings,
                    Some(&form),
                    Some(&msg),
                )),
            )
                .into_response()
        }
    }
}

pub async fn handle_login_submit(
    State(db): State<Database>,
    crate::auth::ClientIp(client): crate::auth::ClientIp,
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let redirect_owned = crate::auth::sanitize_redirect_target(form.redirect_to.as_deref());
    let redirect_to = redirect_owned.as_str();
    let username = form.log.as_deref().unwrap_or("").trim();
    let password = form.pwd.as_deref().unwrap_or("");

    if username.is_empty() || password.is_empty() {
        return Html(templates::render_login_view(
            &settings,
            Some("The username or password field is empty."),
            false,
            Some(redirect_to),
        ))
        .into_response();
    }

    match db.attempt_login(username, password, client.as_deref()) {
        Ok(crate::auth::LoginAttempt::Throttled { retry_after_secs }) => (
            StatusCode::TOO_MANY_REQUESTS,
            Html(templates::render_login_view(
                &settings,
                Some(&format!(
                    "Too many failed login attempts. Please try again in {} minutes.",
                    (retry_after_secs + 59) / 60
                )),
                false,
                Some(redirect_to),
            )),
        )
            .into_response(),
        Ok(crate::auth::LoginAttempt::Success(user)) => {
            let mut headers = HeaderMap::new();
            let secret = db.get_auth_secret().unwrap_or_default();
            let cookie_val = crate::auth::session_cookie_header(
                user.user_id,
                &secret,
                form.rememberme.is_some(),
            );
            if let Ok(val) = cookie_val.parse() {
                headers.insert(header::SET_COOKIE, val);
            }
            if let Ok(loc) = redirect_to.parse() {
                headers.insert(header::LOCATION, loc);
            } else {
                headers.insert(
                    header::LOCATION,
                    header::HeaderValue::from_static("/d-admin/"),
                );
            }
            (StatusCode::SEE_OTHER, headers, Html(String::new())).into_response()
        }
        _ => Html(templates::render_login_view(
            &settings,
            Some("Unknown username or incorrect password."),
            false,
            Some(redirect_to),
        ))
        .into_response(),
    }
}

pub async fn handle_logout() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        "daypress_admin=; Path=/; Max-Age=0; SameSite=Lax; HttpOnly"
            .parse()
            .unwrap(),
    );
    headers.insert(header::LOCATION, "/login?loggedout=true".parse().unwrap());
    (StatusCode::SEE_OTHER, headers, Html(String::new()))
}
