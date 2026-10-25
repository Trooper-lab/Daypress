//! WordPress REST API v2 Posts & Pages Endpoints (`/wp-json/wp/v2/posts` & `/pages`).

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

pub(crate) async fn list_posts(
    crate::auth::OptionalAuthUser(user): crate::auth::OptionalAuthUser,
    State(db): State<Database>,
    Query(mut params): Query<PostQueryParams>,
) -> Result<(HeaderMap, Json<Vec<WpPostResponse>>), AppError> {
    restrict_status_for_visitor(&mut params, user.as_ref());
    let per_page = params.per_page.unwrap_or(10).clamp(1, 500) as usize;
    if let Some(slug) = params.slug.as_deref() {
        if !slug.is_empty()
            && params.status.is_none()
            && params.author.is_none()
            && params.categories.is_none()
            && params.tags.is_none()
        {
            let p = db
                .get_post_by_slug(slug)?
                .filter(|p| crate::auth::can_view_status(&p.status, user.as_ref()));
            let vec: Vec<WpPostResponse> = p.into_iter().collect();
            let total = vec.len() as i64;
            return Ok(wp_list_response(vec, total, per_page as i64));
        }
    }

    let page = params.page.unwrap_or(1).max(1) as usize;
    let offset = params
        .offset
        .map(|o| o.max(0) as usize)
        .unwrap_or_else(|| (page - 1) * per_page);
    let cats = parse_id_list(params.categories.as_deref());
    let tags = parse_id_list(params.tags.as_deref());

    let (posts, total) = db.query_posts(
        "post",
        params.status.as_deref(),
        params.search.as_deref(),
        params.slug.as_deref(),
        params.author,
        params.parent,
        &cats,
        &tags,
        params.orderby.as_deref(),
        params.order.as_deref(),
        offset,
        per_page,
    )?;
    Ok(wp_list_response(posts, total as i64, per_page as i64))
}

pub(crate) async fn get_post(
    crate::auth::OptionalAuthUser(user): crate::auth::OptionalAuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<WpPostResponse>, AppError> {
    match db.get_post_by_id(id)? {
        Some(post) if crate::auth::can_view_status(&post.status, user.as_ref()) => Ok(Json(post)),
        _ => Err(AppError::NotFound(format!("Post {} not found", id))),
    }
}

pub(crate) async fn create_post(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Json(payload): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<WpPostResponse>), AppError> {
    let post = db.create_post(&payload)?;
    Ok((StatusCode::CREATED, Json(post)))
}

pub(crate) async fn update_post(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdatePostRequest>,
) -> Result<Json<WpPostResponse>, AppError> {
    match db.update_post(id, &payload)? {
        Some(post) => Ok(Json(post)),
        None => Err(AppError::NotFound(format!("Post {} not found", id))),
    }
}

pub(crate) async fn delete_post(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Query(params): Query<DeleteQueryParams>,
) -> Result<Json<Value>, AppError> {
    let existing = db
        .get_post_by_id(id)?
        .ok_or_else(|| AppError::NotFound(format!("Post {} not found", id)))?;

    let force = params.force.unwrap_or(false) || existing.status == "trash";
    if force {
        db.delete_post(id)?;
        Ok(Json(serde_json::json!({
            "deleted": true,
            "previous": existing
        })))
    } else {
        let trashed = db
            .trash_post(id)?
            .ok_or_else(|| AppError::NotFound(format!("Post {} not found", id)))?;
        let mut val = serde_json::to_value(&trashed).unwrap_or_default();
        if let Some(obj) = val.as_object_mut() {
            obj.insert("deleted".to_string(), serde_json::json!(true));
            obj.insert("trashed".to_string(), serde_json::json!(true));
            obj.insert(
                "previous".to_string(),
                serde_json::to_value(&existing).unwrap_or_default(),
            );
        }
        Ok(Json(val))
    }
}

pub(crate) async fn list_pages(
    crate::auth::OptionalAuthUser(user): crate::auth::OptionalAuthUser,
    State(db): State<Database>,
    Query(mut params): Query<PostQueryParams>,
) -> Result<(HeaderMap, Json<Vec<WpPostResponse>>), AppError> {
    restrict_status_for_visitor(&mut params, user.as_ref());
    let per_page = params.per_page.unwrap_or(10).clamp(1, 500) as usize;
    let page = params.page.unwrap_or(1).max(1) as usize;
    let offset = params
        .offset
        .map(|o| o.max(0) as usize)
        .unwrap_or_else(|| (page - 1) * per_page);
    let cats = parse_id_list(params.categories.as_deref());
    let tags = parse_id_list(params.tags.as_deref());

    let (pages, total) = db.query_posts(
        "page",
        params.status.as_deref(),
        params.search.as_deref(),
        params.slug.as_deref(),
        params.author,
        params.parent,
        &cats,
        &tags,
        params.orderby.as_deref(),
        params.order.as_deref(),
        offset,
        per_page,
    )?;
    Ok(wp_list_response(pages, total as i64, per_page as i64))
}

pub(crate) async fn get_page(
    crate::auth::OptionalAuthUser(user): crate::auth::OptionalAuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<WpPostResponse>, AppError> {
    match db.get_post_by_id(id)? {
        Some(page)
            if page.post_type == "page"
                && crate::auth::can_view_status(&page.status, user.as_ref()) =>
        {
            Ok(Json(page))
        }
        _ => Err(AppError::NotFound(format!("Page {} not found", id))),
    }
}

pub(crate) async fn create_page(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Json(mut payload): Json<CreatePostRequest>,
) -> Result<(StatusCode, Json<WpPostResponse>), AppError> {
    payload.post_type = "page".to_string();
    let page = db.create_post(&payload)?;
    Ok((StatusCode::CREATED, Json(page)))
}

pub(crate) async fn update_page(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdatePostRequest>,
) -> Result<Json<WpPostResponse>, AppError> {
    match db.update_post(id, &payload)? {
        Some(page) => Ok(Json(page)),
        None => Err(AppError::NotFound(format!("Page {} not found", id))),
    }
}

pub(crate) async fn delete_page(
    auth: crate::auth::EditorUser,
    state: State<Database>,
    path: AxPath<i64>,
    query: Query<DeleteQueryParams>,
) -> Result<Json<Value>, AppError> {
    delete_post(auth, state, path, query).await
}

/// Visitors (and subscribers) may only list published content, whatever `status` they ask for.
fn restrict_status_for_visitor(params: &mut PostQueryParams, user: Option<&crate::auth::AuthUser>) {
    if !user.is_some_and(|u| u.can_edit_content()) {
        params.status = Some("publish".to_string());
    }
}
