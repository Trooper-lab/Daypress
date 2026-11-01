//! Core WordPress Admin SSR Screen & Authentication Handlers.

#![allow(unused_imports)]
pub use super::admin_onboarding::*;
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
pub async fn handle_admin_dashboard(State(db): State<Database>) -> impl IntoResponse {
    if db.should_redirect_to_getting_started() {
        return axum::response::Redirect::to("/d-admin/getting-started").into_response();
    }
    let settings = db.get_settings().unwrap_or_default();
    let mut posts: Vec<models::DaypressPostResponse> = db
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

    Html(templates::render_dashboard_view(&posts, &settings, &stats)).into_response()
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

    Html(templates::render_posts_view_with_matrix(
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

    Html(templates::render_post_new_view(
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
            return Html(templates::render_builder_view(
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

    Html(templates::render_builder_view_with_partials(
        &settings,
        &post,
        layout_opt.as_deref(),
        Some(&partials),
    ))
}

pub async fn handle_admin_media(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let media = db.get_media_attachments().unwrap_or_default();
    Html(templates::render_media_view(&settings, &media))
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
        .unwrap_or_else(|| models::DaypressUserDetail {
            id: target_id,
            username: "admin".to_string(),
            display_name: "admin".to_string(),
            email: "admin@example.com".to_string(),
            url: "".to_string(),
            registered: "2026-10-21".to_string(),
            role: "Administrator".to_string(),
            ..Default::default()
        });
    Html(templates::render_profile_view(&settings, &user))
}

pub async fn handle_admin_themes(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_themes_view(&settings))
}

pub async fn handle_admin_plugins(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let installed = db.get_all_installed_plugins().unwrap_or_default();
    Html(templates::render_plugins_view(&settings, &installed))
}

pub async fn handle_admin_users(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let users = db.get_all_users().unwrap_or_default();
    Html(templates::render_users_view(&settings, &users))
}

pub async fn handle_admin_user_new(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_user_new_view(&settings))
}

pub async fn handle_admin_user_new_submit(
    State(db): State<Database>,
    Form(req): Form<models::CreateUserRequest>,
) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    match db.create_user(&req) {
        Ok(_) => {
            let mut headers = HeaderMap::new();
            headers.insert(header::LOCATION, "/d-admin/users".parse().unwrap());
            (StatusCode::SEE_OTHER, headers, Html(String::new())).into_response()
        }
        Err(_) => Html(templates::render_user_new_view(&settings)).into_response(),
    }
}

pub async fn handle_admin_tools(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let stats = db.get_stats().unwrap_or_default();
    Html(templates::render_tools_view(&settings, &stats))
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
    Html(templates::render_tags_view(&settings, &taxonomy, &terms))
}

pub async fn handle_admin_settings(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_settings_view(&settings))
}

pub async fn handle_admin_engine(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let stats = db.get_stats().unwrap_or_default();
    Html(templates::render_engine_view(&settings, &stats))
}

pub async fn handle_admin_publish(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let status = db.compute_publish_status();
    Html(templates::render_publish_view(&settings, &status))
}
