#![allow(unused_imports)]
use super::*;
use crate::auth::{AuthUser, OptionalAuthUser};
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

// --- Native Subsystems: Custom Post Types, Custom Taxonomies, Structured Field Groups & Collection Grids (v0.6) ---

pub(crate) async fn list_content_types_v1_handler(
    State(db): State<Database>,
) -> Result<Json<Vec<CustomPostTypeDefinition>>, AppError> {
    let defs = db.list_post_type_definitions()?;
    Ok(Json(defs))
}

pub(crate) async fn save_content_type_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(def): Json<CustomPostTypeDefinition>,
) -> Result<Json<CustomPostTypeDefinition>, AppError> {
    if def.slug.trim().is_empty() && def.label_singular.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Custom post type requires a slug or singular label.".to_string(),
        ));
    }
    let saved = db.save_post_type_definition(&def)?;
    Ok(Json(saved))
}

pub(crate) async fn delete_content_type_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
) -> Result<Json<Value>, AppError> {
    let deleted = db.delete_post_type_definition(&slug)?;
    Ok(Json(serde_json::json!({
        "deleted": deleted,
        "slug": slug
    })))
}

pub(crate) async fn provision_content_type_preset_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let preset = payload
        .get("preset")
        .or_else(|| payload.get("preset_key"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();
    if preset.is_empty() {
        return Err(AppError::BadRequest(
            "Missing 'preset' key (e.g. 'case_studies', 'team', 'docs', 'jobs', 'events', 'portfolio', 'testimonials').".to_string(),
        ));
    }
    let (post_type, taxonomies, field_group) = db
        .provision_content_type_preset(preset)
        .map_err(|e| AppError::BadRequest(e.to_string()))?;
    Ok(Json(serde_json::json!({
        "status": "provisioned",
        "preset": preset,
        "post_type": post_type,
        "taxonomies": taxonomies,
        "field_group": field_group
    })))
}

pub(crate) async fn list_custom_taxonomies_v1_handler(
    State(db): State<Database>,
) -> Result<Json<Vec<CustomTaxonomyDefinition>>, AppError> {
    let defs = db.list_custom_taxonomy_definitions()?;
    Ok(Json(defs))
}

pub(crate) async fn save_custom_taxonomy_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(def): Json<CustomTaxonomyDefinition>,
) -> Result<Json<CustomTaxonomyDefinition>, AppError> {
    if def.slug.trim().is_empty() && def.label_singular.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Custom taxonomy requires a slug or singular label.".to_string(),
        ));
    }
    let saved = db.save_custom_taxonomy_definition(&def)?;
    Ok(Json(saved))
}

pub(crate) async fn delete_custom_taxonomy_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    AxPath(slug): AxPath<String>,
) -> Result<Json<Value>, AppError> {
    let deleted = db.delete_custom_taxonomy_definition(&slug)?;
    Ok(Json(serde_json::json!({
        "deleted": deleted,
        "slug": slug
    })))
}

#[derive(Debug, Deserialize, Default)]
pub struct FieldGroupQueryParams {
    pub post_type: Option<String>,
    pub template: Option<String>,
}

pub(crate) async fn list_field_groups_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Query(params): Query<FieldGroupQueryParams>,
) -> Result<Json<Vec<FieldGroupDefinition>>, AppError> {
    if let Some(ref pt) = params.post_type {
        let pt_clean = pt.trim();
        if !pt_clean.is_empty() {
            return Ok(Json(db.list_field_groups(Some(pt_clean))?));
        }
    }
    let groups = db.list_field_groups(None)?;
    Ok(Json(groups))
}

pub(crate) async fn save_field_group_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Json(group): Json<FieldGroupDefinition>,
) -> Result<Json<FieldGroupDefinition>, AppError> {
    if group.title.trim().is_empty() && group.group_key.trim().is_empty() {
        return Err(AppError::BadRequest(
            "Field group requires a title or group_key.".to_string(),
        ));
    }
    let saved = db.save_field_group(&group)?;
    Ok(Json(saved))
}

pub(crate) async fn delete_field_group_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    AxPath(id_or_key): AxPath<String>,
) -> Result<Json<Value>, AppError> {
    let deleted = db.delete_field_group(&id_or_key)?;
    Ok(Json(serde_json::json!({
        "deleted": deleted,
        "id": id_or_key
    })))
}

#[derive(Debug, Deserialize, Default)]
pub struct CollectionQueryV1Params {
    pub post_type: Option<String>,
    pub taxonomy: Option<String>,
    pub term: Option<String>,
    #[serde(alias = "q")]
    pub search: Option<String>,
    pub sort: Option<String>,
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub columns: Option<u8>,
    #[serde(alias = "show_thumbnail")]
    pub show_image: Option<bool>,
    pub show_excerpt: Option<bool>,
    #[serde(alias = "show_meta")]
    pub show_date: Option<bool>,
    #[serde(alias = "skin")]
    pub card_skin: Option<String>,
    pub meta_fields: Option<String>,
}

pub(crate) async fn collection_query_v1_handler(
    crate::auth::OptionalAuthUser(user): crate::auth::OptionalAuthUser,
    State(db): State<Database>,
    headers: axum::http::HeaderMap,
    Query(params): Query<CollectionQueryV1Params>,
) -> Result<Json<Value>, AppError> {
    let post_type = params
        .post_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("post");
    let taxonomy = params
        .taxonomy
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let term = params
        .term
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("all"));
    let search = params
        .search
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let sort = params
        .sort
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("date_desc");
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(6).clamp(1, 100);
    let columns = params.columns.unwrap_or(3).clamp(1, 4);
    let show_image = params.show_image.unwrap_or(true);
    let show_excerpt = params.show_excerpt.unwrap_or(true);
    let show_date = params.show_date.unwrap_or(true);
    let card_skin = params.card_skin.as_deref().unwrap_or("editorial");
    let meta_fields: Vec<String> = params
        .meta_fields
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let base_url = db
        .get_settings()
        .map(|s| s.url)
        .unwrap_or_else(|_| "http://localhost:8090".to_string());
    let (mut posts, total, filter_taxonomy, term_counts) = db.query_collection_posts(
        post_type,
        taxonomy,
        term,
        Some(sort),
        search,
        page,
        per_page,
        &base_url,
    )?;
    for p in &mut posts {
        crate::handlers::public_ssr::redact_post_for_api(&db, p, user.as_ref(), &headers);
    }
    let total_pages = if total == 0 {
        1
    } else {
        total.div_ceil(per_page.max(1))
    };
    let cards_html =
        crate::theme_builder::ssr_injector::SsrInjector::render_loop_grid_html_with_skin(
            &posts,
            columns,
            show_image,
            show_excerpt,
            show_date,
            false,
            None,
            card_skin,
            &meta_fields,
        );

    Ok(Json(serde_json::json!({
        "posts": posts,
        "total": total,
        "page": page,
        "per_page": per_page,
        "total_pages": total_pages,
        "taxonomy": filter_taxonomy,
        "term_counts": term_counts,
        "cards_html": cards_html
    })))
}

// --- Native Subsystems: SQLite FTS5 Instant Search, Auto-OG Social Cards & Live SEO/WCAG Linter (v0.7) ---

#[derive(Debug, Deserialize, Default)]
pub struct FtsSearchQueryParams {
    #[serde(alias = "search")]
    pub q: Option<String>,
    pub post_type: Option<String>,
    #[serde(alias = "lang_code")]
    pub lang: Option<String>,
    #[serde(alias = "per_page")]
    pub limit: Option<usize>,
}

pub(crate) async fn fts_search_v1_handler(
    State(db): State<Database>,
    Query(params): Query<FtsSearchQueryParams>,
) -> Result<Json<crate::adapters::database::search_fts::FtsSearchResponse>, AppError> {
    let q = params.q.as_deref().unwrap_or("").trim();
    let post_type = params
        .post_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("all"));
    let limit = params.limit.unwrap_or(12).clamp(1, 50);
    let mut resp = db.search_fts(q, post_type, limit)?;
    if let Some(lang_filter) = params
        .lang
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("all"))
    {
        resp.results
            .retain(|r| r.lang_code.eq_ignore_ascii_case(lang_filter));
        resp.total = resp.results.len();
    }
    Ok(Json(resp))
}

pub(crate) async fn fts_reindex_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
) -> Result<Json<Value>, AppError> {
    let indexed = db.rebuild_fts_index()?;
    Ok(Json(serde_json::json!({
        "status": "reindexed",
        "engine": "sqlite-fts5-bm25",
        "indexed_count": indexed
    })))
}

pub(crate) async fn seo_audit_v1_handler(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    Json(payload): Json<crate::adapters::database::search_fts::AuditRequestPayload>,
) -> Result<Json<crate::adapters::database::search_fts::AuditReportResponse>, AppError> {
    if let Some(pid) = payload.post_id.filter(|&id| id > 0) {
        let Some(existing) = db.get_post_by_id(pid)? else {
            return Err(AppError::NotFound(format!("Post {} not found", pid)));
        };
        if !user.can_edit_post(existing.author, &existing.status, &existing.post_type) {
            return Err(AppError::Forbidden(
                "Sorry, you are not allowed to audit this post.".to_string(),
            ));
        }
    }
    let report = db.audit_post_wcag_links_seo(&payload);
    Ok(Json(report))
}
