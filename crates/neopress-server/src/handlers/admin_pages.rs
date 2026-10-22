//! Core WordPress Admin SSR Screen & Authentication Handlers.

#![allow(unused_imports)]
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
use super::*;

// Admin Handlers
pub async fn handle_admin_dashboard(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let posts = db
        .get_posts("post", Some("any"), None, 50)
        .unwrap_or_default();
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
    let is_page = params.post_type.as_deref() == Some("page");
    let ptype = if is_page { "page" } else { "post" };
    let posts = db
        .get_posts(ptype, Some("any"), None, 50)
        .unwrap_or_default();

    Html(templates::render_wp_posts_view(&posts, &settings, is_page))
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

    let is_page = params.post_type.as_deref() == Some("page")
        || existing_post
            .as_ref()
            .map(|p| p.post_type == "page")
            .unwrap_or(false);
    let ptype = if is_page { "page" } else { "post" };

    let pages = db
        .get_posts("page", Some("any"), None, 100)
        .unwrap_or_default();
    let categories = db.get_terms("category").unwrap_or_default();
    let tags = db.get_terms("post_tag").unwrap_or_default();
    let media = db.get_media_attachments().unwrap_or_default();
    let has_visual_layout = if let Some(ref p) = existing_post {
        db.get_post_layout(p.id).unwrap_or(None).is_some()
    } else {
        false
    };

    Html(templates::render_wp_post_new_view(
        &settings,
        ptype,
        existing_post.as_ref(),
        &pages,
        &categories,
        &tags,
        &media,
        has_visual_layout,
    ))
}

pub async fn handle_admin_builder(
    State(db): State<Database>,
    Query(params): Query<AdminEditQueryParams>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let target_id = params.id.or(params.post);
    let post = if let Some(id) = target_id {
        db.get_post_by_id(id).unwrap_or(None)
    } else {
        db.get_posts("any", Some("any"), None, 1)
            .ok()
            .and_then(|v| v.into_iter().next())
    };

    let post = match post {
        Some(p) => p,
        None => models::WpPostResponse {
            id: 1,
            date: "2026-10-22T00:00:00".to_string(),
            date_gmt: "2026-10-22T00:00:00".to_string(),
            guid: models::WpGuid {
                rendered: "".to_string(),
            },
            modified: "2026-10-22T00:00:00".to_string(),
            modified_gmt: "2026-10-22T00:00:00".to_string(),
            slug: "sample-page".to_string(),
            status: "publish".to_string(),
            post_type: "page".to_string(),
            link: "/sample-page/".to_string(),
            title: models::WpRenderedText {
                rendered: "Sample Page".to_string(),
                protected: false,
            },
            content: models::WpRenderedText {
                rendered: "".to_string(),
                protected: false,
            },
            excerpt: models::WpRenderedText {
                rendered: "".to_string(),
                protected: false,
            },
            author: 1,
            featured_media: 0,
            comment_status: "open".to_string(),
            ping_status: "open".to_string(),
            sticky: false,
            template: "default".to_string(),
            format: "standard".to_string(),
            meta: serde_json::json!({}),
            categories: vec![1],
            tags: vec![],
            parent: 0,
            menu_order: 0,
            author_name: "admin".to_string(),
            category_names: vec![],
            tag_names: vec![],
            featured_image_url: None,
            comments: vec![],
            seo_title: String::new(),
            seo_description: String::new(),
            seo_keyword: String::new(),
            seo_canonical: String::new(),
            seo_noindex: false,
            seo_schema_type: "WebPage".to_string(),
            seo_og_image: String::new(),
        },
    };

    let layout_opt = db.get_post_layout(post.id).unwrap_or(None);

    Html(templates::render_wp_builder_view(
        &settings,
        &post,
        layout_opt.as_deref(),
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
    let target_id = params
        .user_id
        .or_else(|| auth_user.map(|u| u.user_id))
        .unwrap_or(1);
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
            headers.insert(header::LOCATION, "/n-admin/users.php".parse().unwrap());
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
    Form(form): Form<LoginForm>,
) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let redirect_to = form.redirect_to.as_deref().unwrap_or("/n-admin/");
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

    match db.authenticate_credentials(username, password) {
        Ok(Some(user)) => {
            let mut headers = HeaderMap::new();
            let cookie_val = if form.rememberme.is_some() {
                format!(
                    "neopress_admin={}; Path=/; Max-Age=1209600; SameSite=Lax; HttpOnly",
                    user.user_id
                )
            } else {
                format!(
                    "neopress_admin={}; Path=/; SameSite=Lax; HttpOnly",
                    user.user_id
                )
            };
            if let Ok(val) = cookie_val.parse() {
                headers.insert(header::SET_COOKIE, val);
            }
            if let Ok(loc) = redirect_to.parse() {
                headers.insert(header::LOCATION, loc);
            } else {
                headers.insert(header::LOCATION, "/n-admin/".parse().unwrap());
            }
            (StatusCode::SEE_OTHER, headers, Html(String::new())).into_response()
        }
        _ => Html(templates::render_wp_login_view(
            &settings,
            Some("Unknown username or incorrect password. (Default credentials: admin / admin)"),
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
    headers.insert(
        header::LOCATION,
        "/wp-login.php?loggedout=true".parse().unwrap(),
    );
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
    let taxonomy = params.taxonomy.unwrap_or_else(|| "category".to_string());
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

#[derive(Debug, Deserialize)]
pub struct CommentQueryParams {
    pub comment_status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommentStatusPayload {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct ExportPersonalDataQuery {
    pub user: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ErasePersonalDataPayload {
    pub identifier: String,
}

// 1. Dashboard Updates
pub async fn handle_admin_updates(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let stats = db.get_stats().unwrap_or_default();
    Html(templates::render_wp_updates_view(&settings, &stats))
}

// 2. Media Add New
pub async fn handle_admin_media_new(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_media_new_view(&settings))
}

// 3. Comments Moderation Queue
pub async fn handle_admin_comments(
    State(db): State<Database>,
    Query(params): Query<CommentQueryParams>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let comments = db
        .get_comments(params.comment_status.as_deref(), 100)
        .unwrap_or_default();
    let counts = db.get_comment_counts().unwrap_or_default();
    Html(templates::render_wp_comments_view(
        &settings,
        &comments,
        &counts,
        params.comment_status.as_deref(),
    ))
}

