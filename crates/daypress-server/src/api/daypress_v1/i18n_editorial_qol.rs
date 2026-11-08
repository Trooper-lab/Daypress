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

// --- Native Subsystems: Multilingual (`i18n`), Editorial Workflows & Visual AST Revision Diffs (v0.8) ---

pub(crate) async fn list_languages_v1_handler(
    State(db): State<Database>,
) -> Result<Json<Value>, AppError> {
    let languages = db.list_languages()?;
    let default_lang_code = db.get_default_language_code();
    let prefix_default = db.prefix_default_language_enabled();
    let presets: Vec<Value> = crate::i18n::PRESET_LANGUAGES
        .iter()
        .map(|&(code, name, native_name, flag_emoji, is_rtl)| {
            serde_json::json!({
                "code": code,
                "name": name,
                "native_name": native_name,
                "flag_emoji": flag_emoji,
                "is_rtl": is_rtl,
            })
        })
        .collect();
    Ok(Json(serde_json::json!({
        "languages": languages,
        "default_language": default_lang_code,
        "prefix_default": prefix_default,
        "presets": presets,
    })))
}

#[derive(Debug, Deserialize, Default)]
pub struct ManageLanguagePayload {
    pub action: Option<String>,
    pub code: Option<String>,
    pub locale: Option<String>,
    pub name: Option<String>,
    pub native_name: Option<String>,
    pub flag_emoji: Option<String>,
    pub is_default: Option<bool>,
    pub is_rtl: Option<bool>,
    pub active: Option<bool>,
    pub is_active: Option<bool>,
    pub sort_order: Option<i64>,
    pub prefix_default: Option<bool>,
}

pub(crate) async fn manage_languages_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Json(payload): Json<ManageLanguagePayload>,
) -> Result<Json<Value>, AppError> {
    let action = payload
        .action
        .as_deref()
        .unwrap_or("upsert")
        .trim()
        .to_ascii_lowercase();

    match action.as_str() {
        "set_default" => {
            let code = payload
                .code
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| AppError::BadRequest("Language code is required.".to_string()))?;
            db.set_default_language(code)?;
            let languages = db.list_languages()?;
            Ok(Json(serde_json::json!({
                "status": "updated",
                "default_language": code,
                "languages": languages,
            })))
        }
        "delete" => {
            let code = payload
                .code
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| AppError::BadRequest("Language code is required.".to_string()))?;
            let deleted = db.delete_language(code)?;
            let languages = db.list_languages()?;
            Ok(Json(serde_json::json!({
                "status": if deleted { "deleted" } else { "unchanged" },
                "deleted": deleted,
                "languages": languages,
            })))
        }
        "set_prefix_default" => {
            let enabled = payload.prefix_default.unwrap_or(false);
            db.set_prefix_default_language(enabled)?;
            Ok(Json(serde_json::json!({
                "status": "updated",
                "prefix_default": enabled,
            })))
        }
        _ => {
            let code = payload
                .code
                .as_deref()
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .ok_or_else(|| AppError::BadRequest("Language code is required.".to_string()))?;
            let (preset_locale, preset_name, preset_flag, preset_rtl) =
                crate::i18n::preset_language_info(code);
            let resolved_name = payload
                .name
                .clone()
                .or_else(|| payload.native_name.clone())
                .unwrap_or_else(|| preset_name.to_string());
            let resolved_native_name = payload
                .native_name
                .clone()
                .or_else(|| payload.name.clone())
                .unwrap_or_else(|| preset_name.to_string());
            let lang = DaypressLanguage {
                code: code.to_ascii_lowercase(),
                locale: payload
                    .locale
                    .clone()
                    .unwrap_or_else(|| preset_locale.to_string()),
                name: resolved_name,
                native_name: resolved_native_name,
                flag_emoji: payload
                    .flag_emoji
                    .clone()
                    .unwrap_or_else(|| preset_flag.to_string()),
                is_default: payload.is_default.unwrap_or(false),
                is_rtl: payload.is_rtl.unwrap_or(preset_rtl),
                is_active: payload.is_active.or(payload.active).unwrap_or(true),
                sort_order: payload.sort_order.unwrap_or(10),
            };
            let mut languages = db.upsert_language(&lang)?;
            if lang.is_default {
                db.set_default_language(&lang.code)?;
                languages = db.list_languages().unwrap_or_default();
            }
            Ok(Json(serde_json::json!({
                "status": "saved",
                "language": lang,
                "languages": languages,
            })))
        }
    }
}

pub(crate) async fn delete_language_v1_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    axum::extract::Path(code): axum::extract::Path<String>,
) -> Result<Json<Value>, AppError> {
    let clean = code.trim();
    if clean.is_empty() {
        return Err(AppError::BadRequest(
            "Language code is required.".to_string(),
        ));
    }
    let deleted = db.delete_language(clean)?;
    if !deleted {
        return Err(AppError::BadRequest(
            "Cannot delete the default language or an unknown language.".to_string(),
        ));
    }
    let languages = db.list_languages()?;
    Ok(Json(serde_json::json!({
        "status": "deleted",
        "deleted": true,
        "languages": languages,
    })))
}

#[derive(Debug, Deserialize, Default)]
pub struct LinkTranslationsPayload {
    pub object_type: Option<String>,
    pub group_id: Option<String>,
    pub translation_group_id: Option<String>,
    /// Map of `lang_code -> object_id` (e.g. `{"en": 10, "de": 11}`).
    pub translations: Option<std::collections::HashMap<String, i64>>,
    pub source_id: Option<i64>,
    pub source_post_id: Option<i64>,
    pub source_lang: Option<String>,
    pub target_id: Option<i64>,
    pub target_post_id: Option<i64>,
    pub target_lang: Option<String>,
}

pub(crate) async fn link_translations_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Json(payload): Json<LinkTranslationsPayload>,
) -> Result<Json<Value>, AppError> {
    let obj_type = payload
        .object_type
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("post");
    let explicit_group = payload
        .group_id
        .as_deref()
        .or(payload.translation_group_id.as_deref());

    let src_opt = payload.source_id.or(payload.source_post_id);
    let tgt_opt = payload.target_id.or(payload.target_post_id);
    if let (Some(src_id), Some(tgt_id), Some(tgt_lang)) =
        (src_opt, tgt_opt, payload.target_lang.as_deref())
    {
        let mut map = std::collections::HashMap::new();
        let src_lang = payload
            .source_lang
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(String::from)
            .unwrap_or_else(|| db.get_default_language_code());
        map.insert(src_lang, src_id);
        map.insert(tgt_lang.trim().to_string(), tgt_id);
        let group_id = db.link_translation_map(obj_type, explicit_group, &map)?;
        let updated_map = db.get_translation_group_map(obj_type, &group_id)?;
        return Ok(Json(serde_json::json!({
            "status": "linked",
            "translation_group_id": group_id,
            "translations": updated_map,
        })));
    }

    if let Some(map) = payload.translations {
        if map.is_empty() {
            return Err(AppError::BadRequest(
                "translations map cannot be empty.".to_string(),
            ));
        }
        let group_id = db.link_translation_map(obj_type, explicit_group, &map)?;
        let updated_map = db.get_translation_group_map(obj_type, &group_id)?;
        return Ok(Json(serde_json::json!({
            "status": "linked",
            "translation_group_id": group_id,
            "translations": updated_map,
        })));
    }

    Err(AppError::BadRequest(
        "Provide either `translations` map or `(source_post_id, target_post_id, target_lang)`."
            .to_string(),
    ))
}

#[derive(Debug, Deserialize, Default)]
pub struct CreateTranslationDraftPayload {
    #[serde(default)]
    pub target_lang: String,
    #[serde(default)]
    pub lang_code: String,
}

pub(crate) async fn create_translation_draft_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<CreateTranslationDraftPayload>,
) -> Result<Json<DaypressPostResponse>, AppError> {
    let target = if !payload.target_lang.trim().is_empty() {
        payload.target_lang.trim()
    } else {
        payload.lang_code.trim()
    };
    if target.is_empty() {
        return Err(AppError::BadRequest("target_lang is required.".to_string()));
    }
    let post = db.create_translation_draft(id, target)?;
    Ok(Json(post))
}

#[derive(Debug, Deserialize, Default)]
pub struct ManagePreviewLinkPayload {
    pub action: Option<String>,
    pub expires_in_hours: Option<i64>,
    pub ttl_hours: Option<i64>,
    pub label: Option<String>,
    pub token_id: Option<i64>,
}

pub(crate) async fn list_preview_links_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    let tokens = db.list_preview_tokens(id)?;
    Ok(Json(serde_json::json!({
        "post_id": id,
        "tokens": tokens,
    })))
}

pub(crate) async fn manage_preview_links_v1_handler(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(payload): Json<ManagePreviewLinkPayload>,
) -> Result<Json<Value>, AppError> {
    let action = payload
        .action
        .as_deref()
        .unwrap_or("create")
        .trim()
        .to_ascii_lowercase();
    if action == "revoke" {
        let tid = payload
            .token_id
            .ok_or_else(|| AppError::BadRequest("token_id is required to revoke.".to_string()))?;
        let revoked = db.revoke_preview_token(id, tid)?;
        let tokens = db.list_preview_tokens(id)?;
        return Ok(Json(serde_json::json!({
            "status": "revoked",
            "revoked": revoked,
            "tokens": tokens,
        })));
    }

    let hours = payload
        .ttl_hours
        .or(payload.expires_in_hours)
        .unwrap_or(72)
        .clamp(1, 24 * 365);
    let label = payload.label.as_deref().unwrap_or("Shared Draft Preview");
    let record = db.create_preview_token(id, hours, Some(label), user.user_id)?;
    let tokens = db.list_preview_tokens(id)?;
    Ok(Json(serde_json::json!({
        "status": "created",
        "id": record.id,
        "raw_token": record.raw_token,
        "preview_url": record.preview_url,
        "expires_at": record.expires_at,
        "token": record,
        "tokens": tokens,
    })))
}

pub(crate) async fn get_scheduled_revision_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    let scheduled = db.get_scheduled_revision_for_post(id)?;
    Ok(Json(serde_json::json!({
        "post_id": id,
        "scheduled_revision": scheduled,
    })))
}

pub(crate) async fn schedule_revision_v1_handler(
    crate::auth::EditorUser(user): crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Json(req): Json<ScheduleRevisionRequest>,
) -> Result<Json<ScheduledRevisionSummary>, AppError> {
    let summary = db.schedule_published_post_revision(id, &req, Some(user.user_id))?;
    Ok(Json(summary))
}

pub(crate) async fn cancel_scheduled_revision_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
) -> Result<Json<Value>, AppError> {
    let cancelled = db.cancel_scheduled_revision(id)?;
    Ok(Json(serde_json::json!({
        "post_id": id,
        "cancelled": cancelled,
    })))
}

#[derive(Debug, Deserialize, Default)]
pub struct DiffRevisionsQueryParams {
    pub rev: Option<i64>,
    pub from: Option<i64>,
    pub to: Option<i64>,
}

pub(crate) async fn diff_post_revisions_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    Query(params): Query<DiffRevisionsQueryParams>,
) -> Result<Json<Value>, AppError> {
    let from_rev_id = match params.rev.or(params.from) {
        Some(fid) => fid,
        None => {
            let revs = db.get_post_revisions(id)?;
            revs.first().map(|r| r.id).unwrap_or(id)
        }
    };
    let report = db.diff_post_revisions(id, from_rev_id, params.to)?;
    let revisions = db.get_post_revisions(id).unwrap_or_default();
    let mut val = serde_json::to_value(&report).unwrap_or_else(|_| serde_json::json!({}));
    if let Some(obj) = val.as_object_mut() {
        obj.insert("post_id".to_string(), serde_json::json!(id));
        obj.insert(
            "from_revision_id".to_string(),
            serde_json::json!(from_rev_id),
        );
        obj.insert("to_revision_id".to_string(), serde_json::json!(params.to));
        obj.insert(
            "diff".to_string(),
            serde_json::to_value(&report).unwrap_or(Value::Null),
        );
        obj.insert(
            "revisions".to_string(),
            serde_json::to_value(&revisions).unwrap_or(Value::Null),
        );
    }
    Ok(Json(val))
}

#[derive(Debug, Deserialize, Default)]
pub struct CherryPickNodePayload {
    pub node_id: String,
}

pub(crate) async fn cherry_pick_revision_node_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    AxPath((id, rev_id)): AxPath<(i64, i64)>,
    Json(payload): Json<CherryPickNodePayload>,
) -> Result<Json<Value>, AppError> {
    if payload.node_id.trim().is_empty() {
        return Err(AppError::BadRequest("node_id is required.".to_string()));
    }
    let updated_post = db.cherry_pick_revision_node(id, rev_id, payload.node_id.trim())?;
    let layout_str = db.get_post_layout(id)?.unwrap_or_default();
    let layout_val: Value = serde_json::from_str(&layout_str).unwrap_or(Value::Null);
    Ok(Json(serde_json::json!({
        "status": "cherry_picked",
        "post_id": id,
        "revision_id": rev_id,
        "node_id": payload.node_id,
        "post": updated_post,
        "layout": layout_val,
    })))
}

// =========================================================================
// v0.8.5 WordPress-Native Admin & Editor Ergonomics Endpoints
// =========================================================================

pub(crate) async fn get_user_preferences_v1_handler(
    OptionalAuthUser(auth): OptionalAuthUser,
    State(db): State<Database>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.as_ref().map(|u| u.user_id).unwrap_or(1);
    let prefs = db.get_user_screen_preferences(user_id)?;
    Ok(Json(serde_json::json!({
        "user_id": user_id,
        "preferences": prefs,
    })))
}

pub(crate) async fn save_user_preferences_v1_handler(
    OptionalAuthUser(auth): OptionalAuthUser,
    State(db): State<Database>,
    Json(payload): Json<Value>,
) -> Result<Json<Value>, AppError> {
    let user_id = auth.as_ref().map(|u| u.user_id).unwrap_or(1);
    let prefs = db.save_user_screen_preferences(user_id, &payload)?;
    Ok(Json(serde_json::json!({
        "status": "saved",
        "user_id": user_id,
        "preferences": prefs,
    })))
}

#[derive(Debug, Deserialize, Default)]
pub struct BulkEditPostsPayload {
    #[serde(default)]
    pub post_ids: Vec<i64>,
    pub status: Option<String>,
    pub author: Option<i64>,
    pub comment_status: Option<String>,
    pub ping_status: Option<String>,
    pub sticky: Option<bool>,
    pub template: Option<String>,
    pub categories: Option<Vec<i64>>,
    pub add_tags: Option<String>,
}

pub(crate) async fn bulk_edit_posts_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Json(payload): Json<BulkEditPostsPayload>,
) -> Result<Json<Value>, AppError> {
    if payload.post_ids.is_empty() {
        return Err(AppError::BadRequest("post_ids is required.".to_string()));
    }

    let mut extra_tag_ids = Vec::new();
    if let Some(ref raw_tags) = payload.add_tags {
        for tag_name in raw_tags
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            let term_req = CreateTermRequest {
                name: tag_name.to_string(),
                slug: None,
                description: None,
                parent: None,
            };
            if let Ok(term) = db.create_term("post_tag", &term_req) {
                extra_tag_ids.push(term.id);
            }
        }
    }

    let mut updated_count = 0usize;
    for post_id in &payload.post_ids {
        let merged_tags = if !extra_tag_ids.is_empty() {
            let mut existing = db
                .get_post_by_id(*post_id)
                .ok()
                .flatten()
                .map(|p| p.tags)
                .unwrap_or_default();
            for tid in &extra_tag_ids {
                if !existing.contains(tid) {
                    existing.push(*tid);
                }
            }
            Some(existing)
        } else {
            None
        };

        let update_req = UpdatePostRequest {
            status: payload
                .status
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s != "-1"),
            author: payload.author.filter(|a| *a > 0),
            categories: payload.categories.clone(),
            tags: merged_tags,
            template: payload
                .template
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| s != "-1"),
            sticky: payload.sticky,
            comment_status: payload
                .comment_status
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s != "-1"),
            ping_status: payload
                .ping_status
                .as_ref()
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty() && s != "-1"),
            ..Default::default()
        };

        if db.update_post(*post_id, &update_req).is_ok() {
            updated_count += 1;
        }
    }

    Ok(Json(serde_json::json!({
        "status": "updated",
        "updated_count": updated_count,
        "post_ids": payload.post_ids,
    })))
}

#[derive(Debug, Deserialize, Default)]
pub struct EmptyTrashPayload {
    pub post_type: Option<String>,
}

pub(crate) async fn empty_trash_posts_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Json(payload): Json<EmptyTrashPayload>,
) -> Result<Json<Value>, AppError> {
    let deleted = db.empty_trash_posts(payload.post_type.as_deref())?;
    Ok(Json(serde_json::json!({
        "status": "emptied",
        "deleted_count": deleted,
    })))
}

#[derive(Debug, Deserialize, Default)]
pub struct BulkCommentsPayload {
    #[serde(default)]
    pub comment_ids: Vec<i64>,
    pub action: String,
}

pub(crate) async fn bulk_comments_v1_handler(
    _auth: crate::auth::EditorUser,
    State(db): State<Database>,
    Json(payload): Json<BulkCommentsPayload>,
) -> Result<Json<Value>, AppError> {
    let action = payload.action.trim().to_ascii_lowercase();
    let mut affected = 0usize;
    match action.as_str() {
        "empty_spam" => {
            affected = db.purge_comments_by_status("spam")?;
        }
        "empty_trash" => {
            affected = db.purge_comments_by_status("trash")?;
        }
        "delete" => {
            for cid in &payload.comment_ids {
                if db.delete_comment(*cid).unwrap_or(false) {
                    affected += 1;
                }
            }
        }
        "approve" | "approved" | "1" => {
            for cid in &payload.comment_ids {
                if db.update_comment_status(*cid, "1").unwrap_or(false) {
                    affected += 1;
                }
            }
        }
        "unapprove" | "hold" | "pending" | "0" => {
            for cid in &payload.comment_ids {
                if db.update_comment_status(*cid, "0").unwrap_or(false) {
                    affected += 1;
                }
            }
        }
        "spam" => {
            for cid in &payload.comment_ids {
                if db.update_comment_status(*cid, "spam").unwrap_or(false) {
                    affected += 1;
                }
            }
        }
        "trash" => {
            for cid in &payload.comment_ids {
                if db.update_comment_status(*cid, "trash").unwrap_or(false) {
                    affected += 1;
                }
            }
        }
        other => {
            return Err(AppError::BadRequest(format!(
                "Unsupported bulk comment action: {}",
                other
            )));
        }
    }
    Ok(Json(serde_json::json!({
        "status": "ok",
        "action": action,
        "affected_count": affected,
    })))
}
