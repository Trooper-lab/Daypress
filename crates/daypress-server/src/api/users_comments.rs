//! WordPress REST API v2 Users, Comments, Application Passwords, Nonces & JWT Auth.

#![allow(unused_imports)]
use super::*;
use crate::auth::AuthUser;
use crate::db::Database;
use crate::errors::AppError;
use crate::models::*;
use axum::{
    body::to_bytes,
    extract::{FromRequest, Multipart, Path as AxPath, Query, Request, State},
    http::{header, HeaderMap, StatusCode},
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::Cursor;
use std::path::Path;

pub(crate) async fn list_users(
    State(db): State<Database>,
) -> Result<(HeaderMap, Json<Vec<DaypressUserResponse>>), AppError> {
    let base_url = db.get_base_url();
    let users = db.get_all_users()?;
    let list: Vec<DaypressUserResponse> = users
        .into_iter()
        .map(|u| {
            let detail = db.get_user_detail(u.id).ok().flatten();
            let av = if !u.avatar_url.trim().is_empty() {
                u.avatar_url.clone()
            } else {
                crate::adapters::database::users::gravatar_url_for_email(&u.email, 96)
            };
            DaypressUserResponse {
                id: u.id,
                name: u.display_name,
                url: u.url,
                description: detail
                    .as_ref()
                    .map(|d| d.description.clone())
                    .unwrap_or_default(),
                link: format!("{}/author/{}/", base_url.trim_end_matches('/'), u.username),
                slug: u.username,
                avatar_urls: serde_json::json!({
                    "24": av,
                    "48": av,
                    "96": av
                }),
                first_name: detail
                    .as_ref()
                    .map(|d| d.first_name.clone())
                    .unwrap_or_default(),
                last_name: detail
                    .as_ref()
                    .map(|d| d.last_name.clone())
                    .unwrap_or_default(),
                nickname: detail
                    .as_ref()
                    .map(|d| d.nickname.clone())
                    .unwrap_or_default(),
                locale: detail
                    .as_ref()
                    .map(|d| d.locale.clone())
                    .unwrap_or_default(),
                roles: vec![u.role],
            }
        })
        .collect();
    let total = list.len() as i64;
    Ok(rest_list_response(list, total, total.max(10)))
}

pub(crate) async fn create_user(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<DaypressUserDetail>), AppError> {
    let user = db.create_user(&payload)?;
    Ok((StatusCode::CREATED, Json(user)))
}

pub(crate) async fn delete_user(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    if db.delete_user(id)? {
        Ok(Json(serde_json::json!({
            "deleted": true,
            "previous": { "id": id }
        })))
    } else {
        Err(AppError::NotFound(format!("User {} not found", id)))
    }
}

pub(crate) async fn list_comments(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    Query(params): Query<CommentListQueryParams>,
) -> Result<(HeaderMap, Json<Vec<DaypressCommentAdminItem>>), AppError> {
    if !user.can_moderate_comments() {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to moderate comments.".to_string(),
        ));
    }
    let per_page = params.per_page.unwrap_or(100).clamp(1, 500);
    let mut comments = db
        .get_comments(params.status.as_deref(), 500)
        .map_err(|e| AppError::Internal(e.to_string()))?;
    if let Some(post_id) = params.post {
        comments.retain(|c| c.post_id == post_id);
    }
    let total = comments.len() as i64;
    let page = params.page.unwrap_or(1).max(1) as usize;
    let offset = (page - 1) * (per_page as usize);
    let paged: Vec<DaypressCommentAdminItem> = comments
        .into_iter()
        .skip(offset)
        .take(per_page as usize)
        .collect();
    Ok(rest_list_response(paged, total, per_page))
}

pub(crate) async fn create_comment_handler(
    crate::auth::OptionalAuthUser(auth): crate::auth::OptionalAuthUser,
    State(db): State<Database>,
    Json(mut payload): Json<CreateCommentRequest>,
) -> Result<(StatusCode, Json<DaypressCommentResponse>), AppError> {
    if payload.content.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Comment content is required".to_string(),
        ));
    }
    if !auth.as_ref().is_some_and(|u| u.can_edit_content()) {
        // Visitors may only comment on published posts with comments open; Settings →
        // Discussion decides who may comment and which comments wait for moderation.
        let post = db
            .get_post_by_id(payload.post)?
            .filter(|p| p.status == "publish")
            .ok_or_else(|| AppError::NotFound("Post not found".to_string()))?;
        let closed = || AppError::Forbidden("Comments are closed for this post.".to_string());
        if post.comment_status != "open" {
            return Err(closed());
        }
        let rules = db.get_discussion_settings()?;
        if rules.comment_registration && auth.is_none() {
            return Err(AppError::Forbidden(
                "You must be logged in to post a comment.".to_string(),
            ));
        }
        if rules.close_comments_for_old_posts && rules.close_comments_days_old > 0 {
            let published =
                chrono::NaiveDateTime::parse_from_str(&post.date, "%Y-%m-%dT%H:%M:%S").ok();
            if published.is_some_and(|d| {
                (chrono::Utc::now().naive_utc() - d).num_days() > rules.close_comments_days_old
            }) {
                return Err(closed());
            }
        }
        let author_name = payload.author_name.as_deref().unwrap_or("").trim();
        let author_email = payload.author_email.as_deref().unwrap_or("").trim();
        if rules.require_name_email && (author_name.is_empty() || !author_email.contains('@')) {
            return Err(AppError::BadRequest(
                "Please provide your name and a valid email address.".to_string(),
            ));
        }
        let approve = !rules.comment_moderation
            && (!rules.comment_previously_approved
                || db.has_approved_comment_from(author_email)?);
        payload.status = Some(if approve { "approved" } else { "hold" }.to_string());
    }
    let comment = db.create_comment(&payload)?;
    Ok((StatusCode::CREATED, Json(comment)))
}

pub(crate) async fn get_comment(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<DaypressCommentAdminItem>, AppError> {
    if !user.can_moderate_comments() {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to moderate comments.".to_string(),
        ));
    }
    if let Some(c) = db.get_comment_by_id(id)? {
        Ok(Json(c))
    } else {
        Err(AppError::NotFound(format!("Comment {} not found", id)))
    }
}

pub(crate) async fn update_comment_handler(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdateCommentRequest>,
) -> Result<Json<Value>, AppError> {
    if !user.can_moderate_comments() {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to moderate comments.".to_string(),
        ));
    }
    match db.update_comment(id, &payload)? {
        Some(updated) => {
            let mut val = serde_json::to_value(&updated).unwrap_or_default();
            if let Some(obj) = val.as_object_mut() {
                obj.insert("success".to_string(), serde_json::json!(true));
            }
            Ok(Json(val))
        }
        None => Err(AppError::NotFound(format!("Comment {} not found", id))),
    }
}

pub(crate) async fn delete_comment_handler(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Query(params): Query<DeleteQueryParams>,
) -> Result<Json<Value>, AppError> {
    if !user.can_moderate_comments() {
        return Err(AppError::Forbidden(
            "Sorry, you are not allowed to moderate comments.".to_string(),
        ));
    }
    let existing = db
        .get_comment_by_id(id)?
        .ok_or_else(|| AppError::NotFound(format!("Comment {} not found", id)))?;
    let force = params.force.unwrap_or(true);
    if force {
        db.delete_comment(id)?;
        Ok(Json(serde_json::json!({
            "deleted": true,
            "previous": existing
        })))
    } else {
        db.update_comment_status(id, "trash")?;
        Ok(Json(serde_json::json!({
            "deleted": true,
            "trashed": true,
            "status": "trash",
            "previous": existing
        })))
    }
}

pub(crate) async fn get_current_user(
    auth: AuthUser,
    State(db): State<Database>,
) -> Result<Json<DaypressUserDetail>, AppError> {
    match db.get_user_detail(auth.user_id)? {
        Some(u) => Ok(Json(u)),
        None => Err(AppError::NotFound("User not found".to_string())),
    }
}

pub(crate) async fn update_current_user(
    auth: AuthUser,
    State(db): State<Database>,
    Json(mut payload): Json<UpdateUserRequest>,
) -> Result<Json<DaypressUserDetail>, AppError> {
    // Users may edit their own profile, but only administrators may change roles.
    if !auth.is_admin() {
        payload.role = None;
    }
    match db.update_user_detail(auth.user_id, &payload)? {
        Some(u) => Ok(Json(u)),
        None => Err(AppError::NotFound("User not found".to_string())),
    }
}

pub(crate) async fn get_user_by_id(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<DaypressUserDetail>, AppError> {
    match db.get_user_detail(id)? {
        Some(u) => Ok(Json(u)),
        None => Err(AppError::NotFound(format!("User {} not found", id))),
    }
}

pub(crate) async fn update_user_by_id(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<DaypressUserDetail>, AppError> {
    match db.update_user_detail(id, &payload)? {
        Some(u) => Ok(Json(u)),
        None => Err(AppError::NotFound(format!("User {} not found", id))),
    }
}

// --- Application Passwords & Session Management Endpoints ---

pub(crate) async fn list_app_passwords(
    auth: AuthUser,
    State(db): State<Database>,
    AxPath(user_id): AxPath<i64>,
) -> Result<Json<Vec<AppPasswordResponse>>, AppError> {
    if auth.user_id != user_id && !auth.is_admin() {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }
    let list = db.list_application_passwords(user_id)?;
    Ok(Json(list))
}

pub(crate) async fn create_app_password(
    auth: AuthUser,
    State(db): State<Database>,
    AxPath(user_id): AxPath<i64>,
    Json(payload): Json<CreateAppPasswordRequest>,
) -> Result<(StatusCode, Json<AppPasswordResponse>), AppError> {
    if auth.user_id != user_id && !auth.is_admin() {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }
    let res = db.create_application_password(user_id, &payload.name, payload.app_id.as_deref())?;
    Ok((StatusCode::CREATED, Json(res)))
}

pub(crate) async fn delete_app_password(
    auth: AuthUser,
    State(db): State<Database>,
    AxPath((user_id, uuid)): AxPath<(i64, String)>,
) -> Result<Json<Value>, AppError> {
    if auth.user_id != user_id && !auth.is_admin() {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }
    if db.delete_application_password(user_id, &uuid)? {
        Ok(Json(serde_json::json!({ "deleted": true, "uuid": uuid })))
    } else {
        Err(AppError::NotFound(format!(
            "Application password {} not found",
            uuid
        )))
    }
}

pub(crate) async fn delete_all_app_passwords(
    auth: AuthUser,
    State(db): State<Database>,
    AxPath(user_id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    if auth.user_id != user_id && !auth.is_admin() {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }
    let count = db.delete_all_application_passwords(user_id)?;
    Ok(Json(serde_json::json!({
        "deleted": true,
        "count": count
    })))
}

pub(crate) async fn destroy_other_sessions_handler(
    auth: AuthUser,
    State(db): State<Database>,
    AxPath(user_id): AxPath<i64>,
) -> Result<(HeaderMap, Json<Value>), AppError> {
    if auth.user_id != user_id && !auth.is_admin() {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }
    let new_ver = db.bump_user_session_version(user_id)?;
    let mut headers = HeaderMap::new();
    if auth.user_id == user_id {
        if let Ok(secret) = db.get_auth_secret() {
            let cookie =
                crate::auth::session_cookie_header_with_version(user_id, &secret, false, new_ver);
            if let Ok(hv) = header::HeaderValue::from_str(&cookie) {
                headers.insert(header::SET_COOKIE, hv);
            }
        }
    }
    Ok((
        headers,
        Json(serde_json::json!({
            "success": true,
            "session_ver": new_ver,
            "message": "You are now logged out everywhere else."
        })),
    ))
}

// --- Nonce & JWT Auth Endpoints ---

pub(crate) async fn get_auth_nonce(
    auth: AuthUser,
    State(db): State<Database>,
) -> Result<Json<NonceResponse>, AppError> {
    let secret = db.get_auth_secret()?;
    let nonce = crate::auth::create_rest_nonce("wp_rest", auth.user_id, &secret);
    Ok(Json(NonceResponse {
        nonce,
        action: "wp_rest".to_string(),
    }))
}

pub(crate) async fn generate_jwt_token(
    crate::auth::ClientIp(client): crate::auth::ClientIp,
    headers: HeaderMap,
    State(db): State<Database>,
    Json(payload): Json<JwtTokenRequest>,
) -> Result<Json<JwtTokenResponse>, AppError> {
    use crate::auth::LoginAttempt;
    let (user, used_app_password) = match db.attempt_login_with_method(
        &payload.username,
        &payload.password,
        client.as_deref(),
    )? {
        (LoginAttempt::Success(u), used_app) => (u, used_app),
        (LoginAttempt::Throttled { retry_after_secs }, _) => {
            return Err(AppError::TooManyRequests(format!(
                "Too many failed login attempts. Try again in {} minutes.",
                (retry_after_secs + 59) / 60
            )))
        }
        (LoginAttempt::Invalid, _) => {
            return Err(AppError::Unauthorized(
                "Invalid username or password".to_string(),
            ))
        }
    };

    if !used_app_password && db.is_user_2fa_enabled(user.user_id).unwrap_or(false) {
        let otp = payload
            .totp_code
            .as_deref()
            .or_else(|| headers.get("x-daypress-otp").and_then(|v| v.to_str().ok()))
            .unwrap_or("");
        if !db.verify_user_2fa_code(user.user_id, otp).unwrap_or(false) {
            return Err(AppError::Unauthorized(
                "Two-factor authentication code required (pass totp_code in JSON or X-Daypress-OTP header).".to_string(),
            ));
        }
    }

    let user_detail = db
        .get_user_detail(user.user_id)?
        .unwrap_or_else(|| DaypressUserDetail {
            id: user.user_id,
            username: user.username.clone(),
            display_name: user.username.clone(),
            ..Default::default()
        });

    let secret = db.get_auth_secret()?;
    let site_url = db.get_base_url();
    let token = crate::auth::create_jwt_token(user.user_id, &user.username, &site_url, &secret, 7);

    Ok(Json(JwtTokenResponse {
        token,
        user_email: user_detail.email,
        user_nicename: user.username,
        user_display_name: user_detail.display_name,
    }))
}

pub(crate) async fn validate_jwt_token(
    State(db): State<Database>,
    request: Request,
) -> Result<Json<JwtValidateResponse>, AppError> {
    let (parts, _) = request.into_parts();
    let _auth = crate::auth::authenticate_request(&parts, &db).await?;
    Ok(Json(JwtValidateResponse {
        code: "jwt_auth_valid_token".to_string(),
        data: serde_json::json!({ "status": 200 }),
    }))
}

// --- Two-Factor Authentication (2FA) Management Endpoints ---

pub(crate) async fn get_user_2fa_status_handler(
    auth: AuthUser,
    State(db): State<Database>,
    AxPath(user_id): AxPath<i64>,
) -> Result<Json<TwoFactorStatusResponse>, AppError> {
    if auth.user_id != user_id && !auth.is_admin() {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }
    let status = db.get_user_2fa_status(user_id)?;
    Ok(Json(status))
}

pub(crate) async fn setup_user_2fa_handler(
    auth: AuthUser,
    State(db): State<Database>,
    AxPath(user_id): AxPath<i64>,
) -> Result<Json<TwoFactorSetupResponse>, AppError> {
    if auth.user_id != user_id && !auth.is_admin() {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }
    let setup = db.begin_user_2fa_setup(user_id)?;
    Ok(Json(setup))
}

pub(crate) async fn enable_user_2fa_handler(
    auth: AuthUser,
    State(db): State<Database>,
    AxPath(user_id): AxPath<i64>,
    Json(payload): Json<TwoFactorVerifyRequest>,
) -> Result<Json<TwoFactorEnableResponse>, AppError> {
    if auth.user_id != user_id && !auth.is_admin() {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }
    let res = db.enable_user_2fa(user_id, &payload.code)?;
    Ok(Json(res))
}

pub(crate) async fn regenerate_user_2fa_recovery_codes_handler(
    auth: AuthUser,
    State(db): State<Database>,
    AxPath(user_id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    if auth.user_id != user_id && !auth.is_admin() {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }
    let codes = db.regenerate_user_2fa_recovery_codes(user_id)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "recovery_codes": codes
    })))
}

pub(crate) async fn disable_user_2fa_handler(
    auth: AuthUser,
    State(db): State<Database>,
    AxPath(user_id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    if auth.user_id != user_id && !auth.is_admin() {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }
    let was_enabled = db.disable_user_2fa(user_id)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "disabled": true,
        "was_enabled": was_enabled
    })))
}

pub(crate) async fn upload_user_avatar_handler(
    auth: AuthUser,
    State(db): State<Database>,
    AxPath(user_id): AxPath<i64>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<Value>), AppError> {
    if auth.user_id != user_id && !auth.is_admin() {
        return Err(AppError::Forbidden("Permission denied".to_string()));
    }

    let uploads_dir = Path::new("uploads");
    if !uploads_dir.exists() {
        fs::create_dir_all(uploads_dir)?;
    }

    if let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?
    {
        let file_name_raw = field.file_name().unwrap_or("avatar.png").to_string();
        let content_type = field.content_type().unwrap_or("image/png").to_string();

        let clean_name = file_name_raw
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect::<String>();

        let lower_name = clean_name.to_ascii_lowercase();
        let is_image_ext = lower_name.ends_with(".png")
            || lower_name.ends_with(".jpg")
            || lower_name.ends_with(".jpeg")
            || lower_name.ends_with(".webp")
            || lower_name.ends_with(".gif")
            || lower_name.ends_with(".svg")
            || lower_name.ends_with(".avif");
        if !content_type.starts_with("image/") && !is_image_ext {
            return Err(AppError::BadRequest(
                "Avatar must be an image file (PNG, JPG, WebP, GIF, SVG, or AVIF).".to_string(),
            ));
        }

        let data = field
            .bytes()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;
        if data.is_empty() {
            return Err(AppError::BadRequest("Uploaded file is empty".to_string()));
        }
        let file_size = data.len() as u64;

        let timestamp = chrono::Utc::now().timestamp();
        let final_file_name = format!("{}_avatar_{}_{}", timestamp, user_id, clean_name);
        let dest_path = uploads_dir.join(&final_file_name);
        fs::write(&dest_path, &data)?;

        let url = format!("/uploads/{}", final_file_name);
        let title = format!("Avatar #{}", user_id);
        let media =
            db.create_attachment(&title, &final_file_name, &content_type, file_size, &url)?;
        let req = UpdateUserRequest {
            avatar_url: Some(url.clone()),
            ..Default::default()
        };
        let updated_user = db.update_user_detail(user_id, &req)?;

        return Ok((
            StatusCode::CREATED,
            Json(serde_json::json!({
                "success": true,
                "url": url,
                "media": media,
                "user": updated_user
            })),
        ));
    }

    Err(AppError::BadRequest(
        "No image file uploaded in form data".to_string(),
    ))
}
