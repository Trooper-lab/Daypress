//! Axum REST API route handlers for Theme Builder endpoints.

use super::conditions::resolve_template_for_type;
use super::models::*;
use super::repository::ThemeTemplateRepository;
use crate::auth::AuthUser;
use crate::adapters::database::Database;
use crate::errors::AppError;
use axum::{
    extract::{Path as AxPath, Query, State},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, Default)]
pub struct ListTemplatesQuery {
    pub template_type: Option<String>,
    #[serde(rename = "type")]
    pub type_alias: Option<String>,
    pub status: Option<String>,
    pub active_only: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct DeleteTemplateQuery {
    pub force: Option<bool>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ActiveTemplateQuery {
    pub url: Option<String>,
    pub path: Option<String>,
    pub post_id: Option<i64>,
    pub post_type: Option<String>,
    pub taxonomy: Option<String>,
    pub term_id: Option<i64>,
}

pub fn theme_builder_routes() -> Router<Database> {
    Router::new()
        .route(
            "/neopress/v1/templates",
            get(list_templates_handler).post(create_template_handler),
        )
        .route(
            "/neopress/v1/templates/{id}",
            get(get_template_handler)
                .put(update_template_handler)
                .post(update_template_handler)
                .delete(delete_template_handler),
        )
        .route(
            "/neopress/v1/templates/{id}/conditions",
            get(get_conditions_handler)
                .post(set_conditions_handler)
                .put(set_conditions_handler),
        )
        .route("/neopress/v1/templates/active", get(resolve_active_handler))
        .route(
            "/neopress/v1/templates/starter",
            get(super::starters::list_starters_handler).post(super::starters::install_starter_handler),
        )
        .route(
            "/neopress/v1/templates/preview/{id}",
            get(super::preview::preview_template_handler),
        )
}

pub async fn list_templates_handler(
    State(db): State<Database>,
    Query(query): Query<ListTemplatesQuery>,
) -> Result<Json<Vec<ThemeTemplateRecord>>, AppError> {
    let repo = ThemeTemplateRepository::from_database(&db);
    let t_str = query.template_type.or(query.type_alias);
    let filter = TemplateFilter {
        template_type: t_str.as_deref().and_then(TemplateType::from_str_loose),
        status: query.status,
        is_active: query.active_only,
    };
    let records = repo.list(&filter)?;
    Ok(Json(records))
}

pub async fn create_template_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    Json(payload): Json<CreateTemplateRequest>,
) -> Result<(StatusCode, Json<ThemeTemplateRecord>), AppError> {
    if payload.title.trim().is_empty() {
        return Err(AppError::BadRequest("Template title cannot be empty".to_string()));
    }
    let repo = ThemeTemplateRepository::from_database(&db);
    let record = repo.create(&payload)?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn get_template_handler(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<ThemeTemplateRecord>, AppError> {
    let repo = ThemeTemplateRepository::from_database(&db);
    let record = repo
        .get_by_id(id)?
        .ok_or_else(|| AppError::NotFound(format!("Template {} not found", id)))?;
    Ok(Json(record))
}

pub async fn update_template_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<UpdateTemplateRequest>,
) -> Result<Json<ThemeTemplateRecord>, AppError> {
    let repo = ThemeTemplateRepository::from_database(&db);
    let updated = repo
        .update(id, &payload)?
        .ok_or_else(|| AppError::NotFound(format!("Template {} not found", id)))?;
    Ok(Json(updated))
}

pub async fn delete_template_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Query(query): Query<DeleteTemplateQuery>,
) -> Result<Json<Value>, AppError> {
    let repo = ThemeTemplateRepository::from_database(&db);
    let force = query.force.unwrap_or(false);
    let deleted = repo.delete(id, force)?;
    if !deleted {
        return Err(AppError::NotFound(format!("Template {} not found", id)));
    }
    Ok(Json(serde_json::json!({
        "success": true,
        "id": id,
        "deleted": true,
        "permanent": force
    })))
}

pub async fn get_conditions_handler(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Vec<TemplateCondition>>, AppError> {
    let repo = ThemeTemplateRepository::from_database(&db);
    let _ = repo
        .get_by_id(id)?
        .ok_or_else(|| AppError::NotFound(format!("Template {} not found", id)))?;
    let conditions = repo.get_conditions(id)?;
    Ok(Json(conditions))
}

pub async fn set_conditions_handler(
    _auth: AuthUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<SetConditionsPayload>,
) -> Result<Json<Vec<TemplateCondition>>, AppError> {
    let repo = ThemeTemplateRepository::from_database(&db);
    let _ = repo
        .get_by_id(id)?
        .ok_or_else(|| AppError::NotFound(format!("Template {} not found", id)))?;
    let updated = repo.set_conditions(id, &payload.conditions)?;
    Ok(Json(updated))
}

pub async fn resolve_active_handler(
    State(db): State<Database>,
    Query(query): Query<ActiveTemplateQuery>,
) -> Result<Json<ResolvedTemplatesResponse>, AppError> {
    let context = build_request_context(&db, &query)?;
    let repo = ThemeTemplateRepository::from_database(&db);
    let all_active = repo.list(&TemplateFilter {
        template_type: None,
        status: Some("publish".to_string()),
        is_active: Some(true),
    })?;

    let header = resolve_template_for_type(&all_active, TemplateType::Header, &context);
    let footer = resolve_template_for_type(&all_active, TemplateType::Footer, &context);
    let single = if context.post_type.as_deref() == Some("page") {
        resolve_template_for_type(&all_active, TemplateType::SinglePage, &context)
    } else {
        resolve_template_for_type(&all_active, TemplateType::SinglePost, &context)
    };
    let archive = if context.is_archive {
        resolve_template_for_type(&all_active, TemplateType::Archive, &context)
    } else {
        None
    };
    let error_404 = if context.is_404 {
        resolve_template_for_type(&all_active, TemplateType::Error404, &context)
    } else {
        None
    };

    let popups = all_active
        .into_iter()
        .filter(|t| t.template_type == TemplateType::Popup)
        .collect();

    Ok(Json(ResolvedTemplatesResponse {
        header,
        footer,
        single,
        archive,
        error_404,
        popups,
    }))
}

fn build_request_context(
    db: &Database,
    query: &ActiveTemplateQuery,
) -> Result<RequestContext, AppError> {
    let path = query
        .path
        .clone()
        .or_else(|| query.url.clone())
        .unwrap_or_else(|| "/".to_string());

    let is_front = path == "/" || path.is_empty();

    let mut ctx = RequestContext {
        path: path.clone(),
        post_id: query.post_id,
        post_type: query.post_type.clone(),
        is_front_page: is_front,
        is_archive: false,
        is_404: false,
        is_search: false,
        taxonomy_terms: Vec::new(),
        author_id: None,
    };

    if let (Some(tax), Some(tid)) = (&query.taxonomy, query.term_id) {
        ctx.taxonomy_terms.push(TaxonomyTermRef {
            taxonomy: tax.clone(),
            term_id: tid,
            slug: String::new(),
        });
        ctx.is_archive = true;
    }

    if let Some(pid) = ctx.post_id {
        if ctx.post_type.is_none() {
            if let Ok(Some(post)) = db.get_post_by_id(pid) {
                ctx.post_type = Some(post.post_type);
            }
        }
    }

    Ok(ctx)
}
