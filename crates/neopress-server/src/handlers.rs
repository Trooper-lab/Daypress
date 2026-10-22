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

#[derive(Debug, Deserialize)]
pub struct QueryPostParams {
    pub p: Option<i64>,
    pub page_id: Option<i64>,
    pub s: Option<String>,
    pub q: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ProfileQueryParams {
    pub user_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct LoginQueryParams {
    pub action: Option<String>,
    pub loggedout: Option<String>,
    pub redirect_to: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginForm {
    pub log: Option<String>,
    pub pwd: Option<String>,
    pub redirect_to: Option<String>,
    pub rememberme: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AdminEditQueryParams {
    pub post_type: Option<String>,
    pub id: Option<i64>,
    pub post: Option<i64>,
    pub action: Option<String>,
    pub taxonomy: Option<String>,
}

pub async fn handle_home_or_query(
    State(db): State<Database>,
    Query(params): Query<QueryPostParams>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();

    if let Some(q) = params
        .s
        .as_deref()
        .or(params.q.as_deref())
        .map(str::trim)
        .filter(|s| !s.is_empty())
    {
        let results = db
            .get_posts("any", Some("publish"), Some(q), 50)
            .unwrap_or_default();
        let html = templates::render_search_page(q, &results, &settings, user.as_ref());
        return Html(crate::plugins::inject_plugin_hooks_into_html(
            &db, &settings, None, html,
        ))
        .into_response();
    }

    if let Some(id) = params.p {
        if let Ok(Some(mut post)) = db.get_post_by_id(id) {
            let layout_json = db.get_post_layout(id).unwrap_or(None);
            let hooks = crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&post));
            let expanded = crate::plugins::expand_shortcodes_in_content(
                &post.content.rendered,
                &db,
                &settings,
            );
            post.content.rendered = format!(
                "{}{}{}",
                hooks.content_before_html, expanded, hooks.content_after_html
            );
            let html = templates::render_single_post(
                &post,
                &settings,
                user.as_ref(),
                layout_json.as_deref(),
            );
            return Html(crate::plugins::inject_plugin_hooks_into_html(
                &db,
                &settings,
                Some(&post),
                html,
            ))
            .into_response();
        } else {
            return (
                StatusCode::NOT_FOUND,
                Html(templates::render_not_found_page(&settings, user.as_ref())),
            )
                .into_response();
        }
    }

    if let Some(id) = params.page_id {
        if let Ok(Some(mut page)) = db.get_post_by_id(id) {
            let layout_json = db.get_post_layout(id).unwrap_or(None);
            let hooks = crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&page));
            let expanded = crate::plugins::expand_shortcodes_in_content(
                &page.content.rendered,
                &db,
                &settings,
            );
            page.content.rendered = format!(
                "{}{}{}",
                hooks.content_before_html, expanded, hooks.content_after_html
            );
            let html = templates::render_single_post(
                &page,
                &settings,
                user.as_ref(),
                layout_json.as_deref(),
            );
            return Html(crate::plugins::inject_plugin_hooks_into_html(
                &db,
                &settings,
                Some(&page),
                html,
            ))
            .into_response();
        } else {
            return (
                StatusCode::NOT_FOUND,
                Html(templates::render_not_found_page(&settings, user.as_ref())),
            )
                .into_response();
        }
    }

    if settings.show_on_front == "page" && settings.page_on_front > 0 {
        if let Ok(Some(mut front_page)) = db.get_post_by_id(settings.page_on_front) {
            let layout_json = db.get_post_layout(front_page.id).unwrap_or(None);
            let hooks =
                crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&front_page));
            let expanded = crate::plugins::expand_shortcodes_in_content(
                &front_page.content.rendered,
                &db,
                &settings,
            );
            front_page.content.rendered = format!(
                "{}{}{}",
                hooks.content_before_html, expanded, hooks.content_after_html
            );
            let html = templates::render_single_post(
                &front_page,
                &settings,
                user.as_ref(),
                layout_json.as_deref(),
            );
            return Html(crate::plugins::inject_plugin_hooks_into_html(
                &db,
                &settings,
                Some(&front_page),
                html,
            ))
            .into_response();
        }
    }

    let limit = settings.posts_per_page.clamp(1, 100);
    let posts = db
        .get_posts("post", Some("publish"), None, limit)
        .unwrap_or_default();
    let html = templates::render_home_page(&posts, &settings, user.as_ref());
    Html(crate::plugins::inject_plugin_hooks_into_html(
        &db, &settings, None, html,
    ))
    .into_response()
}

pub async fn handle_single(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
    headers: HeaderMap,
    OptionalAuthUser(user): OptionalAuthUser,
) -> impl IntoResponse {
    // 1. Check 301/302 Redirect Manager first
    let req_path = format!("/{}/", slug.trim_matches('/'));
    if let Some((target_url, status_code)) = db.match_redirect_and_increment(&req_path) {
        let mut resp_headers = HeaderMap::new();
        if let Ok(loc) = target_url.parse() {
            resp_headers.insert(header::LOCATION, loc);
        }
        let status = StatusCode::from_u16(status_code).unwrap_or(StatusCode::MOVED_PERMANENTLY);
        return (status, resp_headers, Html(String::new())).into_response();
    }

    let settings = db.get_settings().unwrap_or_default();

    if let Ok(Some(mut post)) = db.get_post_by_slug(&slug) {
        let layout_json = db.get_post_layout(post.id).unwrap_or(None);
        if settings.show_on_front == "page"
            && settings.page_for_posts > 0
            && post.id == settings.page_for_posts
            && layout_json.is_none()
            && post.content.rendered.trim().is_empty()
        {
            let posts = db
                .get_posts(
                    "post",
                    Some("publish"),
                    None,
                    settings.posts_per_page.clamp(1, 100),
                )
                .unwrap_or_default();
            let html = templates::render_archive_page(
                &post.title.rendered,
                "Blog Archive",
                &posts,
                &settings,
                user.as_ref(),
            );
            return (
                StatusCode::OK,
                Html(crate::plugins::inject_plugin_hooks_into_html(
                    &db,
                    &settings,
                    Some(&post),
                    html,
                )),
            )
                .into_response();
        }

        let hooks = crate::plugins::render_active_plugin_hooks(&db, &settings, Some(&post));
        let expanded = crate::plugins::expand_shortcodes_in_content(
            &post.content.rendered,
            &db,
            &settings,
        );
        post.content.rendered = format!(
            "{}{}{}",
            hooks.content_before_html, expanded, hooks.content_after_html
        );

        let html = templates::render_single_post(
            &post,
            &settings,
            user.as_ref(),
            layout_json.as_deref(),
        );
        return (
            StatusCode::OK,
            Html(crate::plugins::inject_plugin_hooks_into_html(
                &db,
                &settings,
                Some(&post),
                html,
            )),
        )
            .into_response();
    }

    // Record 404 hit in 404 Monitor
    let referrer = headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let ua = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    db.record_404_hit(&req_path, referrer, ua);

    (
        StatusCode::NOT_FOUND,
        Html(crate::plugins::inject_plugin_hooks_into_html(
            &db,
            &settings,
            None,
            templates::render_not_found_page(&settings, user.as_ref()),
        )),
    )
        .into_response()
}

pub async fn handle_category_archive(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let (cat_opt, posts) = db.get_posts_by_category_slug(&slug).unwrap_or_default();
    if let Some(cat) = cat_opt {
        (
            StatusCode::OK,
            Html(templates::render_archive_page(
                &format!("Category: {}", cat.name),
                "Category Archive",
                &posts,
                &settings,
                user.as_ref(),
            )),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Html(templates::render_not_found_page(&settings, user.as_ref())),
        )
    }
}

pub async fn handle_tag_archive(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let (tag_opt, posts) = db.get_posts_by_tag_slug(&slug).unwrap_or_default();
    if let Some(tag) = tag_opt {
        (
            StatusCode::OK,
            Html(templates::render_archive_page(
                &format!("Tag: {}", tag.name),
                "Tag Archive",
                &posts,
                &settings,
                user.as_ref(),
            )),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Html(templates::render_not_found_page(&settings, user.as_ref())),
        )
    }
}

pub async fn handle_author_archive(
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let (author_opt, posts) = db.get_posts_by_author_slug(&slug).unwrap_or_default();
    if let Some(author) = author_opt {
        (
            StatusCode::OK,
            Html(templates::render_archive_page(
                &format!("Author: {}", author.display_name),
                "Author Archive",
                &posts,
                &settings,
                user.as_ref(),
            )),
        )
    } else {
        (
            StatusCode::NOT_FOUND,
            Html(templates::render_not_found_page(&settings, user.as_ref())),
        )
    }
}

pub async fn handle_search_page(
    State(db): State<Database>,
    Query(params): Query<QueryPostParams>,
    OptionalAuthUser(user): OptionalAuthUser,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let q = params
        .s
        .as_deref()
        .or(params.q.as_deref())
        .unwrap_or("")
        .trim();
    let posts = if q.is_empty() {
        Vec::new()
    } else {
        db.get_posts("any", Some("publish"), Some(q), 50)
            .unwrap_or_default()
    };
    Html(templates::render_search_page(
        q,
        &posts,
        &settings,
        user.as_ref(),
    ))
}

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

pub async fn handle_api_update_comment(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    axum::Json(payload): axum::Json<UpdateCommentStatusPayload>,
) -> impl IntoResponse {
    match db.update_comment_status(id, &payload.status) {
        Ok(true) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "status": payload.status })),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "code": "not_found", "message": "Comment not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "code": "error", "message": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn handle_api_delete_comment(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> impl IntoResponse {
    match db.delete_comment(id) {
        Ok(true) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "deleted": true, "previous": { "id": id } })),
        )
            .into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "code": "not_found", "message": "Comment not found" })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "code": "error", "message": e.to_string() })),
        )
            .into_response(),
    }
}

// 4. Appearance Submenus
pub async fn handle_admin_customize(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_customize_view(&settings))
}

pub async fn handle_admin_widgets(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_widgets_view(&settings))
}

pub async fn handle_admin_nav_menus(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let pages = db
        .get_posts("page", Some("any"), None, 50)
        .unwrap_or_default();
    Html(templates::render_wp_nav_menus_view(&settings, &pages))
}

pub async fn handle_admin_theme_editor(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_theme_editor_view(&settings))
}

// 5. Plugins Submenus
pub async fn handle_admin_plugin_install(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_plugin_install_view(&settings))
}

pub async fn handle_admin_plugin_editor(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_plugin_editor_view(&settings))
}

#[derive(Debug, Deserialize)]
pub struct ThemeSwitchPayload {
    pub theme: Option<String>,
    pub active_theme: Option<String>,
}

pub async fn handle_api_theme_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<ThemeSwitchPayload>,
) -> impl IntoResponse {
    let theme = payload
        .theme
        .or(payload.active_theme)
        .unwrap_or_else(|| "Neopress Modern".to_string());
    match db.set_option("neopress_active_theme", &theme) {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "theme": theme })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct CustomizerSavePayload {
    pub title: Option<String>,
    pub description: Option<String>,
    pub brand_color: Option<String>,
    pub typography_scale: Option<String>,
    pub custom_css: Option<String>,
}

pub async fn handle_api_customizer_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<CustomizerSavePayload>,
) -> impl IntoResponse {
    if let Some(ref t) = payload.title {
        let _ = db.set_option("blogname", t);
    }
    if let Some(ref d) = payload.description {
        let _ = db.set_option("blogdescription", d);
    }
    if let Some(ref c) = payload.brand_color {
        let _ = db.set_option("neopress_brand_color", c);
    }
    if let Some(ref s) = payload.typography_scale {
        let _ = db.set_option("neopress_typography_scale", s);
    }
    if let Some(ref css) = payload.custom_css {
        let _ = db.set_option("neopress_custom_css", css);
    }
    (
        StatusCode::OK,
        axum::Json(serde_json::json!({ "success": true })),
    )
}

pub async fn handle_api_widgets_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let widgets_val = payload.get("widgets").cloned().unwrap_or(payload);
    let json_str = serde_json::to_string(&widgets_val).unwrap_or_default();
    match db.set_option("neopress_widgets", &json_str) {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "widgets": widgets_val })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn handle_api_menus_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<serde_json::Value>,
) -> impl IntoResponse {
    let items_val = payload
        .get("items")
        .or_else(|| payload.get("nav_menu_items"))
        .cloned()
        .unwrap_or(payload);
    let items: Vec<models::NavMenuItemDto> = serde_json::from_value(items_val).unwrap_or_default();
    let json_str = serde_json::to_string(&items).unwrap_or_else(|_| "[]".to_string());
    match db.set_option("neopress_nav_menu_items", &json_str) {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "items": items })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct PluginTogglePayload {
    pub plugin: Option<String>,
    pub active: Option<bool>,
    pub active_plugins: Option<Vec<String>>,
}

pub async fn handle_api_plugins_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<PluginTogglePayload>,
) -> impl IntoResponse {
    if let Some(ref slug) = payload.plugin {
        let _ = db.toggle_plugin_status(slug, payload.active);
    }
    if let Some(ref list) = payload.active_plugins {
        if let Ok(all) = db.get_all_installed_plugins() {
            for p in all {
                let should_be_active = list
                    .iter()
                    .any(|s| s.eq_ignore_ascii_case(&p.manifest.slug));
                if p.active != should_be_active {
                    let _ = db.toggle_plugin_status(&p.manifest.slug, Some(should_be_active));
                }
            }
        }
    }

    let active_plugins: Vec<String> = db
        .get_all_installed_plugins()
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.active)
        .map(|p| p.manifest.slug)
        .collect();

    let json_str = serde_json::to_string(&active_plugins).unwrap_or_else(|_| "[]".to_string());
    let _ = db.set_option("neopress_active_plugins", &json_str);
    (
        StatusCode::OK,
        axum::Json(serde_json::json!({ "success": true, "active_plugins": active_plugins })),
    )
        .into_response()
}

#[derive(Debug, Deserialize)]
pub struct ThemeFileQuery {
    pub file: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ThemeFileSavePayload {
    pub file: String,
    pub content: String,
}

fn sanitize_theme_file_key(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '.' || *c == '-' || *c == '_')
        .collect();
    if cleaned.is_empty() {
        "style.css".to_string()
    } else {
        cleaned
    }
}

pub async fn handle_api_theme_file_get(
    State(db): State<Database>,
    Query(params): Query<ThemeFileQuery>,
) -> impl IntoResponse {
    let file = sanitize_theme_file_key(params.file.as_deref().unwrap_or("style.css"));
    let key = format!("neopress_file_{}", file);
    let content = db.get_option(&key).ok().flatten().unwrap_or_default();
    (
        StatusCode::OK,
        axum::Json(serde_json::json!({ "file": file, "content": content })),
    )
}

pub async fn handle_api_theme_file_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<ThemeFileSavePayload>,
) -> impl IntoResponse {
    let file = sanitize_theme_file_key(&payload.file);
    let key = format!("neopress_file_{}", file);
    match db.set_option(&key, &payload.content) {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "file": file })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

// 6. Tools Submenus
pub async fn handle_admin_import(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_import_view(&settings))
}

pub async fn handle_admin_export(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_export_view(&settings))
}

pub async fn handle_admin_export_wxr_download(State(db): State<Database>) -> impl IntoResponse {
    let xml = db
        .export_wxr()
        .unwrap_or_else(|e| format!("<!-- Export error: {} -->", e));
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/xml; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"neopress.wxr.xml\"".parse().unwrap(),
    );
    (StatusCode::OK, headers, xml)
}

pub async fn handle_admin_export_json_download(State(db): State<Database>) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let posts = db
        .get_posts("any", Some("any"), None, 500)
        .unwrap_or_default();
    let users = db.get_all_users().unwrap_or_default();
    let comments = db.get_comments(None, 500).unwrap_or_default();
    let backup = serde_json::json!({
        "generator": "Neopress Embedded Rusqlite Backup",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "settings": settings,
        "posts": posts,
        "users": users,
        "comments": comments
    });
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/json; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"neopress-backup.json\""
            .parse()
            .unwrap(),
    );
    (StatusCode::OK, headers, backup.to_string())
}

pub async fn handle_admin_site_health(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let stats = db.get_stats().unwrap_or_default();
    Html(templates::render_wp_site_health_view(&settings, &stats))
}

pub async fn handle_admin_export_personal_data(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_export_personal_data_view(&settings))
}

pub async fn handle_admin_erase_personal_data(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    Html(templates::render_wp_erase_personal_data_view(&settings))
}

pub async fn handle_api_export_personal_data(
    State(db): State<Database>,
    Query(params): Query<ExportPersonalDataQuery>,
) -> impl IntoResponse {
    let identifier = params.user.unwrap_or_default();
    match db.export_personal_data(&identifier) {
        Ok(val) => (StatusCode::OK, axum::Json(val)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

pub async fn handle_api_erase_personal_data(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<ErasePersonalDataPayload>,
) -> impl IntoResponse {
    match db.erase_personal_data(&payload.identifier) {
        Ok(count) => (StatusCode::OK, axum::Json(serde_json::json!({
            "success": true,
            "items_removed": count,
            "message": format!("Personal data anonymized and scrubbed across {} comments in SQLite", count)
        }))).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, axum::Json(serde_json::json!({ "error": e.to_string() }))).into_response(),
    }
}

// 7. Settings Submenus
pub async fn handle_admin_options_writing(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let writing = db
        .get_writing_settings()
        .unwrap_or_else(|_| models::WritingSettings {
            default_category: 1,
            default_post_format: "standard".to_string(),
            use_smilies: true,
            default_pingback_flag: true,
            mailserver_url: "mail.example.com".to_string(),
            mailserver_login: "login@example.com".to_string(),
            mailserver_pass: "".to_string(),
            mailserver_port: 110,
        });
    let categories = db.get_terms("category").unwrap_or_default();
    Html(templates::render_wp_settings_writing_view(
        &settings,
        &writing,
        &categories,
    ))
}

pub async fn handle_api_options_writing_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::WritingSettings>,
) -> impl IntoResponse {
    match db.update_writing_settings(&payload) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn handle_admin_options_reading(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let reading = db
        .get_reading_settings()
        .unwrap_or_else(|_| models::ReadingSettings {
            show_on_front: "posts".to_string(),
            page_on_front: 0,
            page_for_posts: 0,
            posts_per_page: 10,
            posts_per_rss: 10,
            blog_public: true,
        });
    let pages = db
        .get_posts("page", Some("any"), None, 50)
        .unwrap_or_default();
    Html(templates::render_wp_settings_reading_view(
        &settings, &reading, &pages,
    ))
}

pub async fn handle_api_options_reading_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::ReadingSettings>,
) -> impl IntoResponse {
    match db.update_reading_settings(&payload) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn handle_admin_options_discussion(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let discussion = db
        .get_discussion_settings()
        .unwrap_or_else(|_| models::DiscussionSettings {
            default_ping_status: "open".to_string(),
            default_comment_status: "open".to_string(),
            require_name_email: true,
            comment_registration: false,
            close_comments_for_old_posts: false,
            close_comments_days_old: 14,
            thread_comments: true,
            thread_comments_depth: 5,
            comment_moderation: false,
            comment_previously_approved: true,
            show_avatars: true,
        });
    Html(templates::render_wp_settings_discussion_view(
        &settings,
        &discussion,
    ))
}

pub async fn handle_api_options_discussion_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::DiscussionSettings>,
) -> impl IntoResponse {
    match db.update_discussion_settings(&payload) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn handle_admin_options_media(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let media = db.get_media_settings().unwrap_or(models::MediaSettings {
        thumbnail_size_w: 150,
        thumbnail_size_h: 150,
        thumbnail_crop: true,
        medium_size_w: 300,
        medium_size_h: 300,
        large_size_w: 1024,
        large_size_h: 1024,
        uploads_use_yearmonth_folders: true,
    });
    Html(templates::render_wp_settings_media_view(&settings, &media))
}

pub async fn handle_api_options_media_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::MediaSettings>,
) -> impl IntoResponse {
    match db.update_media_settings(&payload) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn handle_admin_options_permalink(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let permalinks = db
        .get_permalink_settings()
        .unwrap_or_else(|_| models::PermalinkSettings {
            permalink_structure: "/%postname%/".to_string(),
            category_base: "".to_string(),
            tag_base: "".to_string(),
        });
    Html(templates::render_wp_settings_permalink_view(
        &settings,
        &permalinks,
    ))
}

pub async fn handle_api_options_permalink_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::PermalinkSettings>,
) -> impl IntoResponse {
    match db.update_permalink_settings(&payload) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn handle_admin_options_privacy(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let privacy = db
        .get_privacy_settings()
        .unwrap_or(models::PrivacySettings {
            wp_page_for_privacy_policy: 0,
        });
    let pages = db
        .get_posts("page", Some("any"), None, 50)
        .unwrap_or_default();
    Html(templates::render_wp_settings_privacy_view(
        &settings, &privacy, &pages,
    ))
}

pub async fn handle_api_options_privacy_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::PrivacySettings>,
) -> impl IntoResponse {
    match db.update_privacy_settings(&payload) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// =========================================================================
// NATIVE SEO & STREAMING SITEMAP / ROBOTS HANDLERS
// =========================================================================

pub async fn handle_sitemap_xml(State(db): State<Database>) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let base_url = settings.url.trim_end_matches('/');
    let posts = db
        .get_posts("any", Some("publish"), None, 50000)
        .unwrap_or_default();

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str("<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n");

    // Home Page
    xml.push_str(&format!(
        "  <url><loc>{}/</loc><changefreq>daily</changefreq><priority>1.0</priority></url>\n",
        base_url
    ));

    for post in posts {
        if post.seo_noindex {
            continue;
        }
        if post.post_type == "post" || post.post_type == "page" {
            let lastmod = if !post.modified.is_empty() {
                &post.modified
            } else {
                &post.date
            };
            let priority = if post.post_type == "page" {
                "0.8"
            } else {
                "0.6"
            };
            xml.push_str(&format!(
                "  <url><loc>{}/{}/</loc><lastmod>{}</lastmod><priority>{}</priority></url>\n",
                base_url, post.slug, lastmod, priority
            ));
        }
    }

    xml.push_str("</urlset>");

    (
        [(header::CONTENT_TYPE, "application/xml; charset=utf-8")],
        xml,
    )
}

pub async fn handle_robots_txt(State(db): State<Database>) -> impl IntoResponse {
    let settings = db.get_settings().unwrap_or_default();
    let seo_global = db.get_global_seo_settings().unwrap_or_default();
    let base_url = settings.url.trim_end_matches('/');
    let robots = if !seo_global.custom_robots_txt.trim().is_empty() {
        seo_global.custom_robots_txt
    } else {
        format!(
            "User-agent: *\nAllow: /\nDisallow: /n-admin/\nDisallow: /wp-json/\n\nSitemap: {}/sitemap.xml\n",
            base_url
        )
    };
    (
        [(header::CONTENT_TYPE, "text/plain; charset=utf-8")],
        robots,
    )
}

// =========================================================================
// NATIVE COOKIELESS ANALYTICS DASHBOARD HANDLER
// =========================================================================

pub async fn handle_admin_analytics(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let summary = db
        .get_analytics_summary()
        .unwrap_or(models::AnalyticsSummary {
            total_visitors: 0,
            total_pageviews: 0,
            top_pages: vec![],
        });

    let mut top_pages_rows = String::new();
    for p in &summary.top_pages {
        top_pages_rows.push_str(&format!(
            r#"<tr>
                <td style="padding: 12px; font-family: var(--font-mono); font-size: 13px;"><a href="{url}" target="_blank" style="color: #2563eb; text-decoration: none;">{url} ↗</a></td>
                <td style="padding: 12px; font-weight: 700; text-align: right;">{views}</td>
            </tr>"#,
            url = p.page_url,
            views = p.views
        ));
    }

    if summary.top_pages.is_empty() {
        top_pages_rows = r#"<tr><td colspan="2" style="padding: 24px; text-align: center; color: #94a3b8;">No visitor events recorded yet. Browse public pages to generate cookieless telemetry.</td></tr>"#.to_string();
    }

    let content = format!(
        r##"
        <div class="page-header" style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 24px;">
            <div>
                <div style="display: flex; align-items: center; gap: 10px;">
                    <h1 class="page-title">Cookieless Analytics</h1>
                    <span class="badge badge-published" style="font-size: 11px;">GDPR / ePrivacy Native</span>
                </div>
                <p class="page-subtitle">Sub-millisecond visitor metrics via daily-rotating salted SHA-256 hashes. Zero cookies, zero banners.</p>
            </div>
            <button class="btn btn-secondary" onclick="location.reload()">Refresh Data ↻</button>
        </div>

        <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 20px; margin-bottom: 30px;">
            <div class="neo-card" style="padding: 20px;">
                <div style="font-size: 13px; color: #64748b; font-weight: 600; text-transform: uppercase;">Unique Daily Visitors</div>
                <div style="font-size: 36px; font-weight: 800; color: #0f172a; margin-top: 8px;">{visitors}</div>
                <div style="font-size: 12px; color: #10b981; margin-top: 4px;">✓ Cryptographically Salted</div>
            </div>
            <div class="neo-card" style="padding: 20px;">
                <div style="font-size: 13px; color: #64748b; font-weight: 600; text-transform: uppercase;">Total Pageviews</div>
                <div style="font-size: 36px; font-weight: 800; color: #2563eb; margin-top: 8px;">{pageviews}</div>
                <div style="font-size: 12px; color: #64748b; margin-top: 4px;">Direct SQLite Ingest</div>
            </div>
            <div class="neo-card" style="padding: 20px;">
                <div style="font-size: 13px; color: #64748b; font-weight: 600; text-transform: uppercase;">Tracking Footprint</div>
                <div style="font-size: 36px; font-weight: 800; color: #059669; margin-top: 8px;">0 Bytes</div>
                <div style="font-size: 12px; color: #10b981; margin-top: 4px;">0 Cookies • 0 LocalStorage</div>
            </div>
        </div>

        <div class="neo-card" style="padding: 24px;">
            <h2 style="font-size: 16px; font-weight: 700; margin-top: 0; margin-bottom: 16px; color: #0f172a;">Top Visited URLs</h2>
            <table class="wp-list-table" style="width: 100%; border-collapse: collapse;">
                <thead>
                    <tr style="border-bottom: 2px solid #e2e8f0; text-align: left; font-size: 12px; color: #64748b; text-transform: uppercase;">
                        <th style="padding: 8px 12px;">Page Path</th>
                        <th style="padding: 8px 12px; text-align: right;">Views</th>
                    </tr>
                </thead>
                <tbody>
                    {rows}
                </tbody>
            </table>
        </div>
        "##,
        visitors = summary.total_visitors,
        pageviews = summary.total_pageviews,
        rows = top_pages_rows
    );

    Html(templates::render_wp_admin_shell(
        "analytics",
        "Cookieless Analytics",
        &content,
        &settings,
    ))
}

// =========================================================================
// FIRST-PARTY MODULES & COMMUNITY PLUGIN ECOSYSTEM HANDLERS
// =========================================================================

pub async fn handle_admin_seo(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let seo = db.get_global_seo_settings().unwrap_or_default();
    let redirects = db.list_redirects().unwrap_or_default();
    let logs_404 = db.list_404_logs(100).unwrap_or_default();
    Html(templates::render_wp_seo_view(
        &settings, &seo, &redirects, &logs_404,
    ))
}

pub async fn handle_api_seo_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::GlobalSeoSettings>,
) -> impl IntoResponse {
    match db.save_global_seo_settings(&payload) {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "seo": payload })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct RedirectActionPayload {
    pub action: Option<String>,
    pub id: Option<i64>,
    pub source_path: Option<String>,
    pub target_url: Option<String>,
    pub status_code: Option<i64>,
}

pub async fn handle_api_redirects(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<RedirectActionPayload>,
) -> impl IntoResponse {
    let action = payload.action.as_deref().unwrap_or("create");
    match action {
        "delete" => {
            if let Some(id) = payload.id {
                let _ = db.delete_redirect(id);
            }
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        "clear_404" => {
            let _ = db.clear_404_logs();
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        _ => {
            let req = models::CreateRedirectRequest {
                source_path: payload.source_path.unwrap_or_default(),
                target_url: payload.target_url.unwrap_or_else(|| "/".to_string()),
                status_code: payload.status_code.unwrap_or(301),
            };
            match db.create_redirect(&req) {
                Ok(rule) => (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({ "success": true, "redirect": rule })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct FormsQueryParams {
    pub form_id: Option<i64>,
}

pub async fn handle_admin_forms(
    State(db): State<Database>,
    Query(params): Query<FormsQueryParams>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let forms = db.list_forms().unwrap_or_default();
    let entries = db.list_form_entries(params.form_id, 200).unwrap_or_default();
    Html(templates::render_wp_forms_view(
        &settings,
        &forms,
        &entries,
        params.form_id,
    ))
}

#[derive(Debug, Deserialize)]
pub struct FormsActionPayload {
    pub action: Option<String>,
    pub id: Option<i64>,
    pub title: Option<String>,
    pub fields: Option<Vec<models::FormFieldSchema>>,
    pub settings: Option<models::FormSettings>,
}

pub async fn handle_api_forms(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<FormsActionPayload>,
) -> impl IntoResponse {
    let action = payload.action.as_deref().unwrap_or("save_form");
    match action {
        "delete_form" => {
            if let Some(id) = payload.id {
                let _ = db.delete_form(id);
            }
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        "delete_entry" => {
            if let Some(id) = payload.id {
                let _ = db.delete_form_entry(id);
            }
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        _ => {
            let req = models::CreateFormRequest {
                id: payload.id,
                title: payload
                    .title
                    .unwrap_or_else(|| "Contact Form".to_string()),
                fields: payload.fields.unwrap_or_default(),
                settings: payload.settings.unwrap_or_default(),
            };
            match db.save_form_record(&req) {
                Ok(form) => (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({ "success": true, "form": form })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
    }
}

pub async fn handle_admin_forms_export_csv(
    State(db): State<Database>,
    Query(params): Query<FormsQueryParams>,
) -> impl IntoResponse {
    let entries = db.list_form_entries(params.form_id, 5000).unwrap_or_default();
    let mut csv = String::from("Entry ID,Form ID,Form Title,Created At,IP Hash,Payload JSON\n");
    for e in entries {
        let escaped_json = e.data_json.replace('"', "\"\"");
        let escaped_title = e.form_title.replace('"', "\"\"");
        csv.push_str(&format!(
            "{},{},\"{}\",\"{}\",\"{}\",\"{}\"\n",
            e.id, e.form_id, escaped_title, e.created_at, e.ip_hash, escaped_json
        ));
    }
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "text/csv; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"neopress-form-entries.csv\""
            .parse()
            .unwrap(),
    );
    (StatusCode::OK, headers, csv)
}

pub async fn handle_admin_backups(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let snapshots = db.list_backup_snapshots().unwrap_or_default();
    Html(templates::render_wp_backups_view(&settings, &snapshots))
}

#[derive(Debug, Deserialize)]
pub struct BackupActionPayload {
    pub action: String,
    pub label: Option<String>,
    pub filename: Option<String>,
}

pub async fn handle_api_backups(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<BackupActionPayload>,
) -> impl IntoResponse {
    match payload.action.as_str() {
        "create" => match db.create_sqlite_snapshot(payload.label.as_deref()) {
            Ok(snap) => (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true, "snapshot": snap })),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        "delete" => {
            let fname = payload.filename.unwrap_or_default();
            match db.delete_backup_snapshot(&fname) {
                Ok(deleted) => (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({ "success": deleted })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        "restore" => {
            let fname = payload.filename.unwrap_or_default();
            match db.restore_sqlite_snapshot(&fname) {
                Ok(_) => (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({
                        "success": true,
                        "message": format!("Snapshot {} restored into live SQLite database.", fname)
                    })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        _ => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Unknown backup action" })),
        )
            .into_response(),
    }
}

pub async fn handle_admin_backup_download(AxPath(filename): AxPath<String>) -> impl IntoResponse {
    let safe_name: String = filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .collect();
    let path = std::path::Path::new("backups").join(&safe_name);
    if !path.exists() {
        return (StatusCode::NOT_FOUND, "Snapshot not found").into_response();
    }
    match std::fs::read(&path) {
        Ok(bytes) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                "application/octet-stream".parse().unwrap(),
            );
            if let Ok(disp) = format!("attachment; filename=\"{}\"", safe_name).parse() {
                headers.insert(header::CONTENT_DISPOSITION, disp);
            }
            (StatusCode::OK, headers, bytes).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read snapshot",
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct PluginSettingsQuery {
    pub plugin: Option<String>,
}

pub async fn handle_admin_plugin_settings(
    State(db): State<Database>,
    Query(params): Query<PluginSettingsQuery>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let slug = params
        .plugin
        .unwrap_or_else(|| "whatsapp-chat".to_string());
    let plugin = db.get_installed_plugin(&slug).ok().flatten();
    Html(templates::render_wp_plugin_settings_view(
        &settings,
        plugin.as_ref(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct SavePluginConfigPayload {
    pub slug: String,
    pub config: std::collections::HashMap<String, String>,
}

pub async fn handle_api_plugin_settings_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<SavePluginConfigPayload>,
) -> impl IntoResponse {
    match db.save_plugin_config(&payload.slug, &payload.config) {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct CustomPluginActionPayload {
    pub action: Option<String>,
    pub slug: Option<String>,
    pub manifest: Option<models::PluginManifest>,
}

pub async fn handle_api_plugin_install_custom(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<CustomPluginActionPayload>,
) -> impl IntoResponse {
    if payload.action.as_deref() == Some("uninstall") {
        let slug = payload.slug.unwrap_or_default();
        match db.uninstall_custom_plugin(&slug) {
            Ok(removed) => {
                return (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({ "success": removed })),
                )
                    .into_response()
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response()
            }
        }
    }

    if let Some(manifest) = payload.manifest {
        match db.install_custom_plugin(&manifest, None) {
            Ok(_) => (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true, "slug": manifest.slug })),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Missing plugin manifest" })),
        )
            .into_response()
    }
}

pub async fn handle_api_plugin_sdk_template() -> impl IntoResponse {
    let tpl = crate::plugins::starter_plugin_sdk_template();
    let json = serde_json::to_string_pretty(&tpl).unwrap_or_else(|_| "{}".to_string());
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/json; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"neopress-plugin-starter.json\""
            .parse()
            .unwrap(),
    );
    (StatusCode::OK, headers, json)
}
