//! Core WordPress Admin SSR Screen & Authentication Handlers.

#![allow(unused_imports)]
use super::*;
use crate::auth::OptionalAuthUser;
use crate::db::Database;
use crate::models;
use crate::templates;
use axum::{
    extract::{Form, Path as AxPath, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse},
};
use serde::Deserialize;

// Admin Handlers
pub async fn handle_admin_dashboard(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let mut posts: Vec<models::WpPostResponse> = db
        .get_posts("any", Some("publish"), None, 120)
        .unwrap_or_default()
        .into_iter()
        .filter(|p| {
            !matches!(
                p.post_type.as_str(),
                "attachment"
                    | "nav_menu_item"
                    | "revision"
                    | "elementor_library"
                    | "wpcf7_contact_form"
                    | "acf-field"
                    | "acf-field-group"
            )
        })
        .collect();
    if posts.is_empty() {
        posts = db
            .get_posts("post", Some("any"), None, 50)
            .unwrap_or_default();
    }
    let stats = db.get_stats().unwrap_or_default();

    Html(templates::render_wp_dashboard_view(
        &posts, &settings, &stats,
    ))
}

pub async fn handle_admin_posts(
    State(db): State<Database>,
    Query(params): Query<AdminEditQueryParams>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let ptype = params
        .post_type
        .as_deref()
        .map(sanitize_key)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "post".to_string());
    let ptype = ptype.as_str();
    let posts = db
        .get_posts(ptype, Some("any"), None, 200)
        .unwrap_or_default();

    let matrix = if ptype == "page" {
        Some(crate::theme_builder::build_header_footer_matrix(&db))
    } else {
        None
    };

    Html(templates::render_wp_posts_view_with_matrix(
        &posts,
        &settings,
        ptype,
        matrix.as_ref(),
    ))
}

pub async fn handle_admin_post_new(
    State(db): State<Database>,
    Query(params): Query<AdminEditQueryParams>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let target_id = params.id.or(params.post);
    let existing_post = if let Some(id) = target_id {
        db.get_post_by_id(id).unwrap_or(None)
    } else {
        None
    };

    let ptype: String = existing_post
        .as_ref()
        .map(|p| p.post_type.clone())
        .or_else(|| params.post_type.as_deref().map(sanitize_key))
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "post".to_string());

    let pages = db
        .get_posts("page", Some("any"), None, 100)
        .unwrap_or_default();
    let categories = db.get_terms("category").unwrap_or_default();
    let tags = db.get_terms("post_tag").unwrap_or_default();
    let media = db.get_media_attachments().unwrap_or_default();
    let has_visual_layout = existing_post.is_some();
    let taxonomies = db
        .get_taxonomies_for_post_type(&ptype, target_id)
        .unwrap_or_default();

    Html(templates::render_wp_post_new_view(
        &settings,
        &ptype,
        existing_post.as_ref(),
        &pages,
        &categories,
        &tags,
        &media,
        has_visual_layout,
        &taxonomies,
    ))
}

pub async fn handle_admin_builder(
    State(db): State<Database>,
    Query(params): Query<AdminEditQueryParams>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();

    if let Some(tpl_id) = params.template_id {
        let repo = crate::theme_builder::ThemeTemplateRepository::from_database(&db);
        if let Ok(Some(tpl)) = repo.get_by_id(tpl_id) {
            let layout_str =
                serde_json::to_string(&tpl.layout).unwrap_or_else(|_| "{}".to_string());
            let synth_post = super::admin_extra::template_to_builder_post(&tpl);
            return Html(templates::render_wp_builder_view(
                &settings,
                &synth_post,
                Some(&layout_str),
            ));
        }
    }

    let target_id = params.id.or(params.post).or(params.post_id);
    let post = if let Some(id) = target_id {
        db.get_post_by_id(id).unwrap_or(None)
    } else if settings.page_on_front > 0 {
        db.get_post_by_id(settings.page_on_front)
            .unwrap_or(None)
            .or_else(|| {
                db.get_posts("page", Some("publish"), None, 1)
                    .ok()
                    .and_then(|v| v.into_iter().next())
            })
    } else {
        db.get_posts("any", Some("any"), None, 1)
            .ok()
            .and_then(|v| v.into_iter().next())
    };

    let post = post.unwrap_or_else(super::admin_extra::placeholder_builder_post);

    let layout_opt = db.get_post_layout(post.id).unwrap_or(None);
    let partials = super::admin_extra::builder_theme_partials(&db, &settings, &post);

    Html(templates::render_wp_builder_view_with_partials(
        &settings,
        &post,
        layout_opt.as_deref(),
        Some(&partials),
    ))
}

pub async fn handle_admin_media(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let media = db.get_media_attachments().unwrap_or_default();
    Html(templates::render_wp_media_view(&settings, &media))
}

pub async fn handle_admin_profile(
    State(db): State<Database>,
    Query(params): Query<ProfileQueryParams>,
    OptionalAuthUser(auth_user): OptionalAuthUser,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    // Only administrators may open another user's profile via `?user_id=`.
    let is_admin = auth_user.as_ref().is_some_and(|u| u.is_admin());
    let own_id = auth_user.as_ref().map(|u| u.user_id).unwrap_or(1);
    let target_id = if is_admin {
        params.user_id.unwrap_or(own_id)
    } else {
        own_id
    };
    let user = db
        .get_user_detail(target_id)
        .unwrap_or(None)
        .or_else(|| db.get_user_detail(1).unwrap_or(None))
        .unwrap_or_else(|| models::WpUserDetail {
            id: target_id,
            username: "admin".to_string(),
            display_name: "admin".to_string(),
            email: "admin@example.com".to_string(),
            url: "".to_string(),
            registered: "2026-10-21".to_string(),
            role: "Administrator".to_string(),
        });
    Html(templates::render_wp_profile_view(&settings, &user))
}

pub async fn handle_admin_themes(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_themes_view(&settings))
}

pub async fn handle_admin_plugins(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let installed = db.get_all_installed_plugins().unwrap_or_default();
    Html(templates::render_wp_plugins_view(&settings, &installed))
}

pub async fn handle_admin_users(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let users = db.get_all_users().unwrap_or_default();
    Html(templates::render_wp_users_view(&settings, &users))
}

pub async fn handle_admin_user_new(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_user_new_view(&settings))
}

pub async fn handle_admin_user_new_submit(
    State(db): State<Database>,
    Form(req): Form<models::CreateUserRequest>,
) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    match db.create_user(&req) {
        Ok(_) => {
            let mut headers = HeaderMap::new();
            headers.insert(header::LOCATION, "/n-admin/users".parse().unwrap());
            (StatusCode::SEE_OTHER, headers, Html(String::new())).into_response()
        }
        Err(_) => Html(templates::render_wp_user_new_view(&settings)).into_response(),
    }
}

pub async fn handle_login_page(
    State(db): State<Database>,
    Query(params): Query<LoginQueryParams>,
) -> impl IntoResponse {
    if params.action.as_deref() == Some("logout") {
        return handle_logout().await.into_response();
    }

    let settings = db.get_settings().unwrap_or_default();
    let loggedout =
        params.loggedout.as_deref() == Some("true") || params.loggedout.as_deref() == Some("1");
    let error_msg = params.error.as_deref();
    let redirect_to = params.redirect_to.as_deref();
    if params.action.as_deref() == Some("resetpass") && error_msg.is_none() {
        let notice =
            templates::LoginNotice::Info("Your password has been reset. You can now log in.");
        let html = templates::render_wp_login_view_with_notice(&settings, notice, redirect_to);
        return Html(html).into_response();
    }

    Html(templates::render_wp_login_view(
        &settings,
        error_msg,
        loggedout,
        redirect_to,
    ))
    .into_response()
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
        return Html(templates::render_wp_login_view(
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
            Html(templates::render_wp_login_view(
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
                    header::HeaderValue::from_static("/n-admin/"),
                );
            }
            (StatusCode::SEE_OTHER, headers, Html(String::new())).into_response()
        }
        _ => Html(templates::render_wp_login_view(
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
        "neopress_admin=; Path=/; Max-Age=0; SameSite=Lax; HttpOnly"
            .parse()
            .unwrap(),
    );
    headers.insert(header::LOCATION, "/login?loggedout=true".parse().unwrap());
    (StatusCode::SEE_OTHER, headers, Html(String::new()))
}

pub async fn handle_admin_tools(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let stats = db.get_stats().unwrap_or_default();
    Html(templates::render_wp_tools_view(&settings, &stats))
}

pub async fn handle_admin_tags(
    State(db): State<Database>,
    Query(params): Query<AdminEditQueryParams>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let taxonomy = params
        .taxonomy
        .as_deref()
        .map(sanitize_key)
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "category".to_string());
    let terms = db.get_terms(&taxonomy).unwrap_or_default();
    Html(templates::render_wp_tags_view(&settings, &taxonomy, &terms))
}

pub async fn handle_admin_settings(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_settings_view(&settings))
}

pub async fn handle_admin_engine(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let stats = db.get_stats().unwrap_or_default();
    Html(templates::render_wp_engine_view(&settings, &stats))
}
