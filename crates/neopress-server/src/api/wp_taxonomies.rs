//! WordPress REST API v2 Categories, Tags, Menus, Widgets, Themes, Plugins, Types & Search.

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

pub(crate) async fn list_categories(
    State(db): State<Database>,
) -> Result<(HeaderMap, Json<Vec<WpCategoryResponse>>), AppError> {
    let cats = db.get_terms("category")?;
    let total = cats.len() as i64;
    Ok(wp_list_response(cats, total, total.max(10)))
}

pub(crate) async fn get_category_handler(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<WpCategoryResponse>, AppError> {
    match db.get_term_by_id("category", id)? {
        Some(c) => Ok(Json(c)),
        None => Err(AppError::NotFound(format!("Category {} not found", id))),
    }
}

pub(crate) async fn create_category_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Json(payload): Json<CreateTermRequest>,
) -> Result<(StatusCode, Json<WpCategoryResponse>), AppError> {
    let cat = db.create_category(&payload)?;
    Ok((StatusCode::CREATED, Json(cat)))
}

pub(crate) async fn update_category_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdateTermRequest>,
) -> Result<Json<WpCategoryResponse>, AppError> {
    match db.update_term("category", id, &payload)? {
        Some(c) => Ok(Json(c)),
        None => Err(AppError::NotFound(format!("Category {} not found", id))),
    }
}

pub(crate) async fn delete_category_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    let prev = db.get_term_by_id("category", id)?;
    if db.delete_term("category", id)? {
        Ok(Json(serde_json::json!({
            "deleted": true,
            "previous": prev.unwrap_or_else(|| WpCategoryResponse {
                id,
                count: 0,
                description: String::new(),
                link: String::new(),
                name: String::new(),
                slug: String::new(),
                taxonomy: "category".to_string(),
                parent: 0,
            })
        })))
    } else {
        Err(AppError::NotFound(format!("Category {} not found", id)))
    }
}

pub(crate) async fn list_tags(
    State(db): State<Database>,
) -> Result<(HeaderMap, Json<Vec<WpCategoryResponse>>), AppError> {
    let tags = db.get_terms("post_tag")?;
    let total = tags.len() as i64;
    Ok(wp_list_response(tags, total, total.max(10)))
}

pub(crate) async fn get_tag_handler(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<WpCategoryResponse>, AppError> {
    match db.get_term_by_id("post_tag", id)? {
        Some(t) => Ok(Json(t)),
        None => Err(AppError::NotFound(format!("Tag {} not found", id))),
    }
}

pub(crate) async fn create_tag_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Json(payload): Json<CreateTermRequest>,
) -> Result<(StatusCode, Json<WpCategoryResponse>), AppError> {
    let tag = db.create_tag(&payload)?;
    Ok((StatusCode::CREATED, Json(tag)))
}

pub(crate) async fn update_tag_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdateTermRequest>,
) -> Result<Json<WpCategoryResponse>, AppError> {
    match db.update_term("post_tag", id, &payload)? {
        Some(t) => Ok(Json(t)),
        None => Err(AppError::NotFound(format!("Tag {} not found", id))),
    }
}

pub(crate) async fn delete_tag_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    let prev = db.get_term_by_id("post_tag", id)?;
    if db.delete_term("post_tag", id)? {
        Ok(Json(serde_json::json!({
            "deleted": true,
            "previous": prev.unwrap_or_else(|| WpCategoryResponse {
                id,
                count: 0,
                description: String::new(),
                link: String::new(),
                name: String::new(),
                slug: String::new(),
                taxonomy: "post_tag".to_string(),
                parent: 0,
            })
        })))
    } else {
        Err(AppError::NotFound(format!("Tag {} not found", id)))
    }
}

pub(crate) async fn get_menus_handler(State(db): State<Database>) -> Result<Json<Value>, AppError> {
    let settings = db.get_settings()?;
    Ok(Json(serde_json::json!({
        "name": "Primary Navigation",
        "slug": "primary-navigation",
        "locations": ["primary", "header"],
        "items": settings.nav_menu_items
    })))
}

pub(crate) async fn save_menus_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let items_val = payload
        .get("items")
        .or_else(|| payload.get("nav_menu_items"))
        .cloned()
        .unwrap_or(payload);
    let items: Vec<NavMenuItemDto> = serde_json::from_value(items_val).unwrap_or_default();
    let json_str = serde_json::to_string(&items)
        .map_err(|e| AppError::BadRequest(format!("Invalid menu items: {}", e)))?;
    db.set_option("neopress_nav_menu_items", &json_str)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "items": items
    })))
}

pub(crate) async fn get_widgets_handler(
    State(db): State<Database>,
) -> Result<Json<Value>, AppError> {
    let settings = db.get_settings()?;
    let val: Value = serde_json::from_str(&settings.widgets_json).unwrap_or_else(|_| {
        serde_json::json!({
            "sidebar": ["Search Bar", "Recent Posts"],
            "footer": ["Categories List"]
        })
    });
    Ok(Json(val))
}

pub(crate) async fn save_widgets_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let widgets_val = payload.get("widgets").cloned().unwrap_or(payload);
    let json_str = serde_json::to_string(&widgets_val)
        .map_err(|e| AppError::BadRequest(format!("Invalid widgets payload: {}", e)))?;
    db.set_option("neopress_widgets", &json_str)?;
    Ok(Json(serde_json::json!({
        "success": true,
        "widgets": widgets_val
    })))
}

pub(crate) async fn get_themes_handler(
    State(db): State<Database>,
) -> Result<Json<Value>, AppError> {
    let settings = db.get_settings()?;
    let active = if settings.active_theme.is_empty() {
        "Neopress Modern".to_string()
    } else {
        settings.active_theme
    };
    Ok(Json(serde_json::json!([
        {
            "stylesheet": "neopress-modern",
            "template": "neopress-modern",
            "name": { "raw": "Neopress Modern", "rendered": "Neopress Modern" },
            "version": "1.0.0",
            "status": if active.eq_ignore_ascii_case("Neopress Modern") { "active" } else { "inactive" }
        },
        {
            "stylesheet": "twenty-twenty-five",
            "template": "twenty-twenty-five",
            "name": { "raw": "Twenty Twenty-Five", "rendered": "Twenty Twenty-Five" },
            "version": "1.0",
            "status": if active.eq_ignore_ascii_case("Twenty Twenty-Five") { "active" } else { "inactive" }
        },
        {
            "stylesheet": "editorial-minimalist",
            "template": "editorial-minimalist",
            "name": { "raw": "Editorial Minimalist", "rendered": "Editorial Minimalist" },
            "version": "2.1",
            "status": if active.eq_ignore_ascii_case("Editorial Minimalist") { "active" } else { "inactive" }
        }
    ])))
}

pub(crate) async fn get_plugins_handler(
    State(db): State<Database>,
) -> Result<Json<Value>, AppError> {
    let settings = db.get_settings()?;
    let is_active = |slug: &str| {
        settings.active_plugins.is_empty()
            || settings
                .active_plugins
                .iter()
                .any(|p| p.eq_ignore_ascii_case(slug))
    };
    Ok(Json(serde_json::json!([
        {
            "plugin": "neopress-wal-optimizer",
            "name": "Neopress WAL Optimizer",
            "version": "1.0.0",
            "status": if is_active("neopress-wal-optimizer") { "active" } else { "inactive" }
        },
        {
            "plugin": "webp-image-engine",
            "name": "WebP Image Engine",
            "version": "2.4.1",
            "status": if is_active("webp-image-engine") { "active" } else { "inactive" }
        },
        {
            "plugin": "wxr-streaming-xml-importer",
            "name": "WXR Streaming XML Importer",
            "version": "1.2.0",
            "status": if is_active("wxr-streaming-xml-importer") { "active" } else { "inactive" }
        }
    ])))
}

pub(crate) async fn search_content_handler(
    State(db): State<Database>,
    Query(params): Query<SearchQueryParams>,
) -> Result<(HeaderMap, Json<Vec<Value>>), AppError> {
    let per_page = params.per_page.unwrap_or(10).clamp(1, 100);
    let page = params.page.unwrap_or(1).max(1) as usize;
    let offset = (page - 1) * (per_page as usize);
    let post_type = params.subtype.as_deref().unwrap_or("any");
    let (posts, total) = db.query_posts(
        post_type,
        Some("publish"),
        params.search.as_deref(),
        None,
        None,
        None,
        &[],
        &[],
        None,
        None,
        offset,
        per_page as usize,
    )?;
    let results: Vec<Value> = posts
        .into_iter()
        .map(|p| {
            serde_json::json!({
                "id": p.id,
                "title": p.title.rendered,
                "url": p.link,
                "type": "post",
                "subtype": p.post_type
            })
        })
        .collect();
    Ok(wp_list_response(results, total as i64, per_page))
}

pub(crate) async fn get_post_types_handler() -> Json<Value> {
    Json(serde_json::json!({
        "post": {
            "name": "Posts",
            "slug": "post",
            "hierarchical": false,
            "rest_base": "posts"
        },
        "page": {
            "name": "Pages",
            "slug": "page",
            "hierarchical": true,
            "rest_base": "pages"
        },
        "attachment": {
            "name": "Media",
            "slug": "attachment",
            "hierarchical": false,
            "rest_base": "media"
        }
    }))
}

pub(crate) async fn get_post_statuses_handler() -> Json<Value> {
    Json(serde_json::json!({
        "publish": { "name": "Published", "public": true, "slug": "publish" },
        "future": { "name": "Scheduled", "public": false, "slug": "future" },
        "draft": { "name": "Draft", "public": false, "slug": "draft" },
        "pending": { "name": "Pending", "public": false, "slug": "pending" },
        "private": { "name": "Private", "public": false, "slug": "private" },
        "trash": { "name": "Trash", "public": false, "slug": "trash" }
    }))
}

pub(crate) async fn get_taxonomies_handler(State(db): State<Database>) -> Json<Value> {
    let mut map = serde_json::Map::new();
    map.insert(
        "category".to_string(),
        serde_json::json!({
            "name": "Categories",
            "slug": "category",
            "types": ["post"],
            "hierarchical": true,
            "rest_base": "categories"
        }),
    );
    map.insert(
        "post_tag".to_string(),
        serde_json::json!({
            "name": "Tags",
            "slug": "post_tag",
            "types": ["post"],
            "hierarchical": false,
            "rest_base": "tags"
        }),
    );

    if let Ok(conn) = db.conn.lock() {
        if let Ok(mut stmt) = conn.prepare("SELECT DISTINCT taxonomy FROM wp_term_taxonomy WHERE taxonomy NOT IN ('nav_menu', 'link_category', 'post_format', 'category', 'post_tag')") {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                for slug in rows.flatten() {
                    let name = crate::adapters::database::taxonomies_discovery::format_taxonomy_display_name(&slug);
                    let hierarchical = !(slug.ends_with("_tag") || slug.ends_with("_tags"));
                    map.insert(
                        slug.clone(),
                        serde_json::json!({
                            "name": name,
                            "slug": slug,
                            "types": ["post"],
                            "hierarchical": hierarchical,
                            "rest_base": slug
                        }),
                    );
                }
            }
        }
    }

    Json(Value::Object(map))
}
