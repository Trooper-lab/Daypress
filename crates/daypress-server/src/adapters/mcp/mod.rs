//! # Model Context Protocol (MCP) Adapter (`adapters/mcp`)
//!
//! Enables Claude Desktop, Claude Code, Cursor, and other AI agents to inspect,
//! author, update, and migrate WordPress sites in Daypress via JSON-RPC 2.0
//! (both over HTTP `POST /mcp` / `POST /wp-json/daypress/v1/mcp` and CLI `--mcp-stdio`).
//!
//! Drives the core [`CmsService`] exclusively through the Hexagonal Input Ports
//! ([`ContentUseCase`], [`MigrationUseCase`], [`SettingsUseCase`]).

mod html_import;

use crate::application::CmsService;
use crate::domain::errors::DomainError;
use crate::domain::models::{CreatePostRequest, ImportBundleRequest, UpdatePostRequest};
use crate::ports::input::{ContentUseCase, MigrationUseCase, SettingsUseCase};
use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

#[derive(Debug, Clone, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: Option<String>,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
}

pub mod schemas;
pub use schemas::mcp_tools_manifest;

/// Executes a single MCP JSON-RPC 2.0 request against the Hexagonal [`CmsService`].
pub fn handle_mcp_rpc(service: &CmsService, req: JsonRpcRequest) -> JsonRpcResponse {
    let id = req.id.unwrap_or(Value::Null);

    match req.method.as_str() {
        "initialize" => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "daypress-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            error: None,
        },
        "notifications/initialized" | "notifications/cancelled" | "ping" => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(json!({})),
            error: None,
        },
        "tools/list" => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(mcp_tools_manifest()),
            error: None,
        },
        "resources/list" => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(json!({ "resources": [] })),
            error: None,
        },
        "resources/templates/list" => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(json!({ "resourceTemplates": [] })),
            error: None,
        },
        "prompts/list" => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: Some(json!({ "prompts": [] })),
            error: None,
        },
        "tools/call" => {
            let tool_name = req
                .params
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let args = req.params.get("arguments").cloned().unwrap_or(json!({}));

            match dispatch_tool_call(service, tool_name, args) {
                Ok(payload) => JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id,
                    result: Some(json!({
                        "content": [
                            {
                                "type": "text",
                                "text": serde_json::to_string_pretty(&payload).unwrap_or_default()
                            }
                        ],
                        "isError": false
                    })),
                    error: None,
                },
                Err(err) => JsonRpcResponse {
                    jsonrpc: "2.0".into(),
                    id,
                    result: Some(json!({
                        "content": [
                            {
                                "type": "text",
                                "text": err.to_string()
                            }
                        ],
                        "isError": true
                    })),
                    error: None,
                },
            }
        }
        other => JsonRpcResponse {
            jsonrpc: "2.0".into(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Method not found: {other}"),
            }),
        },
    }
}

/// `daypress_save_template` layouts must be a `DaypressLayout` with a `root` node object.
///
/// Without this check a layout missing `root` (for example a bare node, or the
/// `{"post_id", "layout_json"}` wrapper `daypress_get_post_layout` returns) deserializes to an
/// empty default layout and the template silently renders nothing.
fn validate_template_layout_args(args: &Value) -> Result<(), DomainError> {
    for key in ["layout", "layout_json"] {
        let Some(raw) = args.get(key).filter(|v| !v.is_null()) else {
            continue;
        };
        let parsed;
        let value = match raw {
            Value::String(s) => {
                parsed = serde_json::from_str::<Value>(s).map_err(|e| {
                    DomainError::Validation(format!("'{key}' is not valid JSON: {e}"))
                })?;
                &parsed
            }
            other => other,
        };
        let root = value.get("root").filter(|r| r.is_object()).ok_or_else(|| {
            let hint = if value.get("layout_json").is_some() {
                " (pass the inner `layout_json` value from daypress_get_post_layout, not the whole response)"
            } else {
                ""
            };
            DomainError::Validation(format!(
                "'{key}' must be a DaypressLayout object with a `root` node, e.g. {{\"version\": 1, \"root\": {{\"id\": \"root\", \"type\": \"Root\", \"children\": [...]}}}}{hint}"
            ))
        })?;
        if root
            .get("type")
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            return Err(DomainError::Validation(format!(
                "'{key}.root' must have a `type` (usually \"Root\")"
            )));
        }
        serde_json::from_value::<daypress_ast::DaypressLayout>(value.clone()).map_err(|e| {
            DomainError::Validation(format!("'{key}' is not a valid DaypressLayout: {e}"))
        })?;
    }
    Ok(())
}

fn dispatch_tool_call(
    service: &CmsService,
    tool_name: &str,
    args: Value,
) -> Result<Value, DomainError> {
    match tool_name {
        "daypress_list_posts" => {
            let post_type = args
                .get("post_type")
                .and_then(Value::as_str)
                .unwrap_or("post");
            let status = args.get("status").and_then(Value::as_str);
            let search = args.get("search").and_then(Value::as_str);
            let limit = args.get("limit").and_then(Value::as_i64).unwrap_or(20);
            let posts = service.list_content(post_type, status, search, limit)?;
            Ok(serde_json::to_value(posts).unwrap_or(Value::Null))
        }
        "daypress_get_post" => {
            if let Some(id) = args.get("id").and_then(Value::as_i64) {
                let post = service.get_content_by_id(id)?;
                Ok(serde_json::to_value(post).unwrap_or(Value::Null))
            } else if let Some(slug) = args.get("slug").and_then(Value::as_str) {
                let post = service.get_content_by_slug(slug)?;
                Ok(serde_json::to_value(post).unwrap_or(Value::Null))
            } else {
                Err(DomainError::Validation(
                    "Either 'id' or 'slug' must be provided".into(),
                ))
            }
        }
        "daypress_create_post" => {
            let req: CreatePostRequest = serde_json::from_value(args)
                .map_err(|e| DomainError::Validation(format!("Invalid create_post args: {e}")))?;
            let created = service.publish_content(req)?;
            Ok(serde_json::to_value(created).unwrap_or(Value::Null))
        }
        "daypress_update_post" => {
            let id = args
                .get("id")
                .and_then(Value::as_i64)
                .ok_or_else(|| DomainError::Validation("'id' is required".into()))?;
            let req: UpdatePostRequest = serde_json::from_value(args)
                .map_err(|e| DomainError::Validation(format!("Invalid update_post args: {e}")))?;
            let updated = service.edit_content(id, req)?;
            Ok(serde_json::to_value(updated).unwrap_or(Value::Null))
        }
        "daypress_get_post_layout" | "daypress_get_layout_ast" => {
            let post_id = args
                .get("post_id")
                .or_else(|| args.get("id"))
                .and_then(Value::as_i64)
                .ok_or_else(|| DomainError::Validation("'post_id' is required".into()))?;
            let layout = service.get_layout_ast(post_id)?;
            Ok(json!({ "post_id": post_id, "layout_json": layout }))
        }
        "daypress_update_post_layout" | "daypress_save_layout_ast" => {
            let post_id = args
                .get("post_id")
                .or_else(|| args.get("id"))
                .and_then(Value::as_i64)
                .ok_or_else(|| DomainError::Validation("'post_id' is required".into()))?;
            let layout_json = args
                .get("layout_json")
                .and_then(Value::as_str)
                .ok_or_else(|| DomainError::Validation("'layout_json' is required".into()))?;
            service.update_layout_ast(post_id, layout_json)?;
            Ok(json!({ "success": true, "post_id": post_id }))
        }
        "daypress_import_html_design" => html_import::import_html_design(service, &args),
        "daypress_update_global_styles" => {
            let gs_val = args
                .get("global_styles")
                .cloned()
                .ok_or_else(|| DomainError::Validation("'global_styles' is required".into()))?;
            let kit: daypress_ast::DaystudioKit = serde_json::from_value(gs_val).map_err(|e| {
                DomainError::Validation(format!("Invalid DaystudioKit global_styles: {e}"))
            })?;
            let kit_css = kit.compile_to_css();
            service.update_site_option("elementor_kit_css", &kit_css)?;

            let post_id = args.get("post_id").and_then(Value::as_i64);
            if let Some(pid) = post_id {
                let mut layout = match service.get_layout_ast(pid)? {
                    Some(raw) if !raw.trim().is_empty() => {
                        serde_json::from_str::<daypress_ast::DaypressLayout>(&raw)
                            .unwrap_or_default()
                    }
                    _ => daypress_ast::DaypressLayout::default(),
                };
                layout.global_styles = serde_json::to_value(&kit).ok();
                let layout_json = serde_json::to_string(&layout).map_err(|e| {
                    DomainError::Internal(format!("Failed to serialize layout: {e}"))
                })?;
                service.update_layout_ast(pid, &layout_json)?;
            }

            Ok(json!({
                "status": "updated",
                "post_id": post_id,
                "compiled_css_bytes": kit_css.len(),
                "system_colors": kit.system_colors.len(),
                "custom_colors": kit.custom_colors.len(),
                "system_typography": kit.system_typography.len()
            }))
        }
        "daypress_list_terms" => {
            let taxonomy = args
                .get("taxonomy")
                .and_then(Value::as_str)
                .unwrap_or("category");
            let terms = service.list_taxonomies(taxonomy)?;
            Ok(serde_json::to_value(terms).unwrap_or(Value::Null))
        }
        "daypress_get_settings" => {
            let settings = service.get_site_settings()?;
            Ok(serde_json::to_value(settings).unwrap_or(Value::Null))
        }
        "daypress_update_option" => {
            let name = args
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| DomainError::Validation("'name' is required".into()))?;
            let lower_opt = name.trim().to_ascii_lowercase();
            if matches!(
                lower_opt.as_str(),
                "auth_secret"
                    | "daypress_auth_secret"
                    | "daypress_session_secret"
                    | "session_secret"
                    | "jwt_secret"
            ) {
                return Err(DomainError::Validation(format!(
                    "Option '{}' is a protected cryptographic secret and cannot be modified via MCP.",
                    name
                )));
            }
            let value = args
                .get("value")
                .and_then(Value::as_str)
                .ok_or_else(|| DomainError::Validation("'value' is required".into()))?;
            service.update_site_option(name, value)?;
            Ok(json!({ "success": true, "option_name": name, "option_value": value }))
        }
        "daypress_import_wp_bundle" => {
            let bundle_val = args.get("bundle").cloned().unwrap_or(args);
            let bundle: ImportBundleRequest = serde_json::from_value(bundle_val)
                .map_err(|e| DomainError::Validation(format!("Invalid WP bundle payload: {e}")))?;
            let res = service.migrate_wordpress_bundle(bundle)?;
            Ok(serde_json::to_value(res).unwrap_or(Value::Null))
        }
        "daypress_get_publish_status" => {
            let provider = service
                .options_repo()
                .get_option("daypress_publish_provider")?
                .unwrap_or_else(|| "zip".to_string());
            let production_url = service
                .options_repo()
                .get_option("daypress_publish_production_url")?
                .unwrap_or_default();
            let local_export_dir = service
                .options_repo()
                .get_option("daypress_publish_export_dir")?
                .unwrap_or_else(|| "data/static-export".to_string());
            let published_posts = service.list_content("any", Some("publish"), None, 500)?;
            Ok(json!({
                "provider": provider,
                "production_url": production_url,
                "local_export_dir": local_export_dir,
                "published_routes_count": published_posts.len(),
                "publish_portal_url": "/d-admin/publish",
                "publish_api_endpoint": "/wp-json/daypress/v1/publish/run"
            }))
        }
        "daypress_publish_site" => {
            if let Some(p) = args.get("provider").and_then(Value::as_str) {
                service.update_site_option("daypress_publish_provider", p.trim())?;
            }
            if let Some(u) = args.get("production_url").and_then(Value::as_str) {
                service.update_site_option("daypress_publish_production_url", u.trim())?;
            }
            if let Some(d) = args.get("local_export_dir").and_then(Value::as_str) {
                service.update_site_option("daypress_publish_export_dir", d.trim())?;
            }
            let provider = service
                .options_repo()
                .get_option("daypress_publish_provider")?
                .unwrap_or_else(|| "zip".to_string());
            let production_url = service
                .options_repo()
                .get_option("daypress_publish_production_url")?
                .unwrap_or_default();
            let local_export_dir = service
                .options_repo()
                .get_option("daypress_publish_export_dir")?
                .unwrap_or_else(|| "data/static-export".to_string());
            let published_posts = service.list_content("any", Some("publish"), None, 500)?;
            Ok(json!({
                "status": "configured",
                "provider": provider,
                "production_url": production_url,
                "local_export_dir": local_export_dir,
                "published_routes_ready": published_posts.len(),
                "trigger_endpoint": "POST /wp-json/daypress/v1/publish/run",
                "admin_ui": "/d-admin/publish"
            }))
        }
        "daypress_list_forms" => {
            let forms = service.options_repo().list_forms()?;
            let items: Vec<Value> = forms
                .into_iter()
                .map(|(f, entry_count)| {
                    json!({
                        "id": f.id,
                        "title": f.title,
                        "schema_json": f.schema_json,
                        "settings_json": f.settings_json,
                        "entry_count": entry_count,
                        "created_at": f.created_at,
                        "updated_at": f.updated_at
                    })
                })
                .collect();
            Ok(Value::Array(items))
        }
        "daypress_get_form" => {
            let id = args
                .get("id")
                .and_then(Value::as_i64)
                .ok_or_else(|| DomainError::Validation("'id' is required".into()))?;
            let form = service
                .options_repo()
                .get_form_by_id(id)?
                .ok_or_else(|| DomainError::NotFound(format!("Form #{id} not found")))?;
            Ok(serde_json::to_value(form).unwrap_or(Value::Null))
        }
        "daypress_create_form" => {
            let req: crate::domain::models::CreateFormRequest = serde_json::from_value(args)
                .map_err(|e| DomainError::Validation(format!("Invalid create_form args: {e}")))?;
            let id = service.options_repo().save_form_record(&req)?;
            let created = service.options_repo().get_form_by_id(id)?;
            Ok(json!({
                "status": "created",
                "id": id,
                "shortcode": format!("[daypress_form id=\"{id}\"]"),
                "form": created
            }))
        }
        "daypress_update_form" => {
            let id = args
                .get("id")
                .and_then(Value::as_i64)
                .ok_or_else(|| DomainError::Validation("'id' is required".into()))?;
            let existing = service
                .options_repo()
                .get_form_by_id(id)?
                .ok_or_else(|| DomainError::NotFound(format!("Form #{id} not found")))?;
            let title = args
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or(&existing.title)
                .to_string();
            let fields: Vec<crate::domain::models::FormFieldSchema> =
                if let Some(f) = args.get("fields") {
                    serde_json::from_value(f.clone()).unwrap_or_default()
                } else {
                    serde_json::from_str(&existing.schema_json).unwrap_or_default()
                };
            let settings: crate::domain::models::FormSettings =
                if let Some(s) = args.get("settings") {
                    serde_json::from_value(s.clone()).unwrap_or_default()
                } else {
                    serde_json::from_str(&existing.settings_json).unwrap_or_default()
                };
            let saved_id = service.options_repo().save_form_record(
                &crate::domain::models::CreateFormRequest {
                    id: Some(id),
                    title,
                    fields,
                    settings,
                },
            )?;
            Ok(json!({ "status": "updated", "id": saved_id }))
        }
        "daypress_list_form_submissions" => {
            let form_id = args.get("form_id").and_then(Value::as_i64);
            let mut status = args.get("status").and_then(Value::as_str);
            let starred_only = args
                .get("starred_only")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            if starred_only {
                status = Some("starred");
            }
            let search = args.get("search").and_then(Value::as_str);
            let limit = args
                .get("limit")
                .and_then(Value::as_i64)
                .unwrap_or(50)
                .clamp(1, 500);
            let entries = service
                .options_repo()
                .list_form_entries_filtered(form_id, status, search, limit)?;
            Ok(serde_json::to_value(entries).unwrap_or(Value::Null))
        }
        "daypress_get_analytics" => {
            let range = args.get("range").and_then(Value::as_str).unwrap_or("30d");
            let str_arg = |k: &str| -> Option<String> {
                args.get(k)
                    .and_then(Value::as_str)
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(ToString::to_string)
            };
            let filter = crate::models::AnalyticsFilter {
                page: str_arg("page"),
                referrer: str_arg("referrer"),
                channel: str_arg("channel"),
                utm_source: str_arg("utm_source"),
                utm_medium: str_arg("utm_medium"),
                utm_campaign: str_arg("utm_campaign"),
                device: str_arg("device"),
                browser: str_arg("browser"),
                os: str_arg("os"),
                country: str_arg("country"),
                goal: str_arg("goal"),
            };
            let summary = service
                .options_repo()
                .get_analytics_dashboard_filtered(range, &filter)?;
            Ok(serde_json::to_value(summary).unwrap_or(Value::Null))
        }
        "daypress_list_templates" => {
            let template_type = args.get("template_type").and_then(Value::as_str);
            let templates = service.options_repo().list_theme_templates(template_type)?;
            Ok(Value::Array(templates))
        }
        "daypress_save_template" => {
            validate_template_layout_args(&args)?;
            let saved = service
                .options_repo()
                .save_theme_template_from_json(&args)?;
            Ok(saved)
        }
        "daypress_list_media_folders" => {
            if let Some(name) = args
                .get("create_name")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
            {
                let parent_id = args.get("parent_id").and_then(Value::as_i64).unwrap_or(0);
                let _ = service
                    .options_repo()
                    .create_media_folder(name, parent_id)?;
            }
            let folders = service.options_repo().list_media_folders()?;
            Ok(serde_json::to_value(folders).unwrap_or(Value::Null))
        }
        "daypress_scan_unused_media" => {
            let unused = service.options_repo().scan_unused_media()?;
            Ok(json!({
                "unused_count": unused.len(),
                "items": unused
            }))
        }
        "daypress_get_storage_s3_status" => {
            let raw = service
                .options_repo()
                .get_option(crate::storage_s3::S3_CONFIG_OPTION_KEY)?
                .unwrap_or_default();
            let mut cfg: crate::storage_s3::S3StorageConfig =
                serde_json::from_str(&raw).unwrap_or_default();
            if !cfg.secret_access_key.is_empty() {
                cfg.secret_access_key = crate::storage_s3::MASKED_SECRET_PLACEHOLDER.to_string();
            }
            Ok(serde_json::to_value(cfg).unwrap_or(Value::Null))
        }
        "daypress_list_post_types" => {
            let defs = service.options_repo().list_post_type_definitions()?;
            Ok(serde_json::to_value(defs).unwrap_or(Value::Null))
        }
        "daypress_save_post_type" => {
            let def: crate::domain::models::CustomPostTypeDefinition = serde_json::from_value(args)
                .map_err(|e| {
                    DomainError::Validation(format!("Invalid save_post_type args: {e}"))
                })?;
            let saved = service.options_repo().save_post_type_definition(&def)?;
            Ok(serde_json::to_value(saved).unwrap_or(Value::Null))
        }
        "daypress_provision_cpt_preset" => {
            let preset = args
                .get("preset")
                .or_else(|| args.get("preset_key"))
                .and_then(Value::as_str)
                .ok_or_else(|| DomainError::Validation("'preset' is required".into()))?;
            let res = service
                .options_repo()
                .provision_content_type_preset(preset)?;
            Ok(serde_json::to_value(res).unwrap_or(Value::Null))
        }
        "daypress_list_field_groups" => {
            let post_type_filter = args
                .get("post_type")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty());
            let mut groups = service.options_repo().list_field_group_definitions()?;
            if let Some(pt) = post_type_filter {
                groups.retain(|g| {
                    g.location_rules.is_empty()
                        || g.location_rules
                            .iter()
                            .any(|r| r.param == "post_type" && r.value.eq_ignore_ascii_case(pt))
                });
            }
            Ok(serde_json::to_value(groups).unwrap_or(Value::Null))
        }
        "daypress_save_field_group" => {
            let group: crate::domain::models::FieldGroupDefinition = serde_json::from_value(args)
                .map_err(|e| {
                DomainError::Validation(format!("Invalid save_field_group args: {e}"))
            })?;
            let saved = service.options_repo().save_field_group_definition(&group)?;
            Ok(serde_json::to_value(saved).unwrap_or(Value::Null))
        }
        "daypress_search" => {
            let query = args
                .get("query")
                .or_else(|| args.get("q"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let post_type = args
                .get("post_type")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("all"));
            let limit = args
                .get("limit")
                .and_then(Value::as_u64)
                .unwrap_or(12)
                .clamp(1, 50) as usize;
            service
                .options_repo()
                .search_fts_json(query, post_type, limit)
        }
        "daypress_audit_post" => service.options_repo().audit_post_json(&args),
        "daypress_list_languages" => service.options_repo().list_languages_json(),
        "daypress_link_translations" => service.options_repo().link_translations_json(&args),
        "daypress_duplicate_post" => {
            let post_id = args
                .get("post_id")
                .or_else(|| args.get("id"))
                .and_then(Value::as_i64)
                .ok_or_else(|| DomainError::Validation("'post_id' is required".into()))?;
            service.options_repo().duplicate_post_json(post_id)
        }
        "daypress_create_preview_link" => {
            let post_id = args
                .get("post_id")
                .or_else(|| args.get("id"))
                .and_then(Value::as_i64)
                .ok_or_else(|| DomainError::Validation("'post_id' is required".into()))?;
            let hours = args
                .get("expires_in_hours")
                .and_then(Value::as_i64)
                .unwrap_or(48)
                .clamp(1, 24 * 365);
            let label = args
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or("Stakeholder Preview");
            service
                .options_repo()
                .create_preview_link_json(post_id, hours, label)
        }
        "daypress_schedule_revision" => {
            let post_id = args
                .get("post_id")
                .or_else(|| args.get("id"))
                .and_then(Value::as_i64)
                .ok_or_else(|| DomainError::Validation("'post_id' is required".into()))?;
            service
                .options_repo()
                .schedule_revision_json(post_id, &args)
        }
        "daypress_diff_revisions" => {
            let post_id = args
                .get("post_id")
                .or_else(|| args.get("id"))
                .and_then(Value::as_i64)
                .ok_or_else(|| DomainError::Validation("'post_id' is required".into()))?;
            let from_rev = args.get("from_revision_id").and_then(Value::as_i64);
            let to_rev = args.get("to_revision_id").and_then(Value::as_i64);
            service
                .options_repo()
                .diff_revisions_json(post_id, from_rev, to_rev)
        }
        "daypress_list_subscribers" => {
            let status = args.get("status").and_then(Value::as_str);
            let list = args.get("list").and_then(Value::as_str);
            service.options_repo().list_subscribers_json(status, list)
        }
        "daypress_create_newsletter_campaign" => service
            .options_repo()
            .create_newsletter_campaign_json(&args),
        "daypress_manage_shortlinks" => service.options_repo().manage_shortlinks_json(&args),
        "daypress_manage_webhooks" => service.options_repo().manage_webhooks_json(&args),
        "daypress_manage_ab_tests" => service.options_repo().manage_ab_tests_json(&args),
        "daypress_get_web_vitals" => service.options_repo().get_web_vitals_json(),
        "daypress_manage_wasm_extensions" => {
            service.options_repo().manage_wasm_extensions_json(&args)
        }
        "daypress_security_audit" => service.options_repo().security_audit_json(),
        "daypress_run_benchmark" => {
            let iterations = args
                .get("iterations")
                .and_then(Value::as_u64)
                .unwrap_or(25)
                .clamp(5, 500) as usize;
            service.options_repo().run_benchmark_json(iterations)
        }
        unknown => Err(DomainError::NotFound(format!(
            "Unknown MCP tool: {unknown}"
        ))),
    }
}

/// Axum HTTP handler for `POST /mcp` and `POST /wp-json/daypress/v1/mcp`.
pub async fn handle_http_mcp(
    _auth: crate::auth::AdminUser,
    State(db): State<crate::db::Database>,
    Json(req): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    let service = CmsService::from_adapter(db);
    Json(handle_mcp_rpc(&service, req))
}

/// Runs the MCP server over `stdin`/`stdout` (`daypress --mcp-stdio`) for Claude Desktop & Claude Code.
pub fn run_stdio_mcp_loop(service: &CmsService) -> std::io::Result<()> {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();

    for line_res in stdin.lock().lines() {
        let line = line_res?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let response = match serde_json::from_str::<JsonRpcRequest>(trimmed) {
            Ok(req) => {
                // Per JSON-RPC 2.0 & MCP stdio specification, notifications (no `id` or `notifications/*`)
                // must not emit a response frame on stdout.
                if req.id.is_none() || req.method.starts_with("notifications/") {
                    let _ = handle_mcp_rpc(service, req);
                    continue;
                }
                handle_mcp_rpc(service, req)
            }
            Err(e) => JsonRpcResponse {
                jsonrpc: "2.0".into(),
                id: Value::Null,
                result: None,
                error: Some(JsonRpcError {
                    code: -32700,
                    message: format!("Parse error: {e}"),
                }),
            },
        };
        if let Ok(serialized) = serde_json::to_string(&response) {
            writeln!(stdout, "{serialized}")?;
            stdout.flush()?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    fn count_custom_html(node: &daypress_ast::DaypressNode) -> usize {
        let own = usize::from(matches!(
            node.kind,
            daypress_ast::NodeKind::CustomHtml { .. }
        ));
        own + node.children.iter().map(count_custom_html).sum::<usize>()
    }

    #[test]
    fn test_mcp_jsonrpc_initialize_tools_and_content_workflow() {
        let db = Database::new_in_memory().expect("in-memory db");
        let service = CmsService::from_adapter(db);

        // 1. initialize
        let init_resp = handle_mcp_rpc(
            &service,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(1)),
                method: "initialize".into(),
                params: json!({}),
            },
        );
        assert!(init_resp.error.is_none());
        assert_eq!(
            init_resp.result.unwrap()["serverInfo"]["name"],
            "daypress-mcp"
        );

        // 2. tools/list
        let list_resp = handle_mcp_rpc(
            &service,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(2)),
                method: "tools/list".into(),
                params: json!({}),
            },
        );
        let tools = list_resp.result.unwrap()["tools"]
            .as_array()
            .unwrap()
            .clone();
        assert!(tools.len() >= 14);

        // 3. tools/call -> daypress_create_post
        let create_resp = handle_mcp_rpc(
            &service,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(3)),
                method: "tools/call".into(),
                params: json!({
                    "name": "daypress_create_post",
                    "arguments": {
                        "title": "Authored via Claude MCP",
                        "content": "<p>Updated seamlessly by AI over MCP.</p>",
                        "status": "publish",
                        "post_type": "post"
                    }
                }),
            },
        );
        let result_obj = create_resp.result.unwrap();
        assert_eq!(result_obj["isError"], false);
        let text = result_obj["content"][0]["text"].as_str().unwrap();
        assert!(text.contains("authored-via-claude-mcp"));
    }

    #[test]
    fn test_mcp_v05_templates_symbols_and_media_dam_tools() {
        let db = Database::new_in_memory().expect("in-memory db");
        let service = CmsService::from_adapter(db);

        // 1. Save a Symbol via MCP
        let save_tpl = handle_mcp_rpc(
            &service,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(10)),
                method: "tools/call".into(),
                params: json!({
                    "name": "daypress_save_template",
                    "arguments": {
                        "title": "MCP Synced CTA",
                        "template_type": "symbol",
                        "status": "publish",
                        "is_active": true,
                        "priority": 10
                    }
                }),
            },
        );
        let tpl_res = save_tpl.result.unwrap();
        assert_eq!(tpl_res["isError"], false);
        let tpl_text = tpl_res["content"][0]["text"].as_str().unwrap();
        assert!(tpl_text.contains("MCP Synced CTA"));

        // 2. List templates filtered by 'symbol'
        let list_tpl = handle_mcp_rpc(
            &service,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(11)),
                method: "tools/call".into(),
                params: json!({
                    "name": "daypress_list_templates",
                    "arguments": {
                        "template_type": "symbol"
                    }
                }),
            },
        );
        let list_res = list_tpl.result.unwrap();
        assert_eq!(list_res["isError"], false);
        assert!(list_res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("MCP Synced CTA"));

        // 3. Create & list Media DAM folder via MCP
        let folder_resp = handle_mcp_rpc(
            &service,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(12)),
                method: "tools/call".into(),
                params: json!({
                    "name": "daypress_list_media_folders",
                    "arguments": {
                        "create_name": "Hero Assets"
                    }
                }),
            },
        );
        let folder_res = folder_resp.result.unwrap();
        assert_eq!(folder_res["isError"], false);
        assert!(folder_res["content"][0]["text"]
            .as_str()
            .unwrap()
            .contains("Hero Assets"));

        // 4. Scan unused media & check S3 status via MCP
        let unused_resp = handle_mcp_rpc(
            &service,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(13)),
                method: "tools/call".into(),
                params: json!({
                    "name": "daypress_scan_unused_media",
                    "arguments": {}
                }),
            },
        );
        assert_eq!(unused_resp.result.unwrap()["isError"], false);

        let s3_resp = handle_mcp_rpc(
            &service,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(14)),
                method: "tools/call".into(),
                params: json!({
                    "name": "daypress_get_storage_s3_status",
                    "arguments": {}
                }),
            },
        );
        assert_eq!(s3_resp.result.unwrap()["isError"], false);
    }

    #[test]
    fn test_mcp_import_html_design_end_to_end_broadsheet_and_design_system() {
        let db = Database::new_in_memory().expect("in-memory db");
        let service = CmsService::from_adapter(db);

        // The sample designs are not in the repository. Point DAYPRESS_HTML_SAMPLES_DIR at a
        // folder holding them to run this test; without it, the test is skipped.
        let Ok(samples_dir) = std::env::var("DAYPRESS_HTML_SAMPLES_DIR") else {
            eprintln!("skipped: set DAYPRESS_HTML_SAMPLES_DIR to run the HTML design import test");
            return;
        };
        let samples_dir = std::path::PathBuf::from(samples_dir);
        let paths = [
            samples_dir.join("Daypress.html"),
            samples_dir.join("daypress-design-system.html"),
        ];

        for (idx, path) in paths.iter().enumerate() {
            let html = std::fs::read_to_string(path)
                .unwrap_or_else(|e| panic!("cannot read sample {}: {e}", path.display()));
            let slug = if idx == 0 { "home" } else { "design-system" };
            let resp = handle_mcp_rpc(
                &service,
                JsonRpcRequest {
                    jsonrpc: Some("2.0".into()),
                    id: Some(json!(100 + idx)),
                    method: "tools/call".into(),
                    params: json!({
                        "name": "daypress_import_html_design",
                        "arguments": {
                            "html": html,
                            "slug": slug,
                            "set_as_front_page": idx == 0,
                            "extract_header_footer": true
                        }
                    }),
                },
            );
            let res_obj = resp.result.expect("MCP result");
            assert_eq!(res_obj["isError"], false);
            let payload: Value =
                serde_json::from_str(res_obj["content"][0]["text"].as_str().unwrap()).unwrap();

            assert_eq!(payload["status"], "imported");
            assert_eq!(payload["custom_html_nodes"], 0);
            assert!(payload["node_count"].as_u64().unwrap() > 50);
            assert!(payload["token_count"].as_u64().unwrap() >= 8);
            assert_eq!(payload["header_extracted"], true);
            assert_eq!(payload["footer_extracted"], true);

            let post_id = payload["post_id"].as_i64().unwrap();
            let layout_raw = service
                .get_layout_ast(post_id)
                .unwrap()
                .expect("saved layout AST");
            let layout: daypress_ast::DaypressLayout =
                serde_json::from_str(&layout_raw).expect("valid DaypressLayout");
            assert_eq!(
                count_custom_html(&layout.root),
                0,
                "Imported design {} must have 0 CustomHtml nodes",
                slug
            );
            assert!(layout.global_styles.is_some());

            let post = service.get_content_by_id(post_id).unwrap();
            assert!(!post.content.rendered.is_empty());
            assert!(!post.content.rendered.contains("contenteditable"));
        }
    }

    #[test]
    fn test_mcp_all_47_tools_manifest_and_protocol_compliance() {
        let db = Database::new_in_memory().expect("in-memory db");
        let service = CmsService::from_adapter(db);

        // 1. Verify initialize version matches CARGO_PKG_VERSION (1.0.0-beta.1)
        let init_resp = handle_mcp_rpc(
            &service,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(1)),
                method: "initialize".into(),
                params: json!({}),
            },
        );
        let init_res = init_resp.result.expect("init result");
        assert_eq!(init_res["serverInfo"]["version"], env!("CARGO_PKG_VERSION"));

        // 2. Verify resources/list and prompts/list return clean empty arrays
        for (method, key) in [
            ("resources/list", "resources"),
            ("resources/templates/list", "resourceTemplates"),
            ("prompts/list", "prompts"),
        ] {
            let resp = handle_mcp_rpc(
                &service,
                JsonRpcRequest {
                    jsonrpc: Some("2.0".into()),
                    id: Some(json!(2)),
                    method: method.into(),
                    params: json!({}),
                },
            );
            assert!(resp.error.is_none(), "{method} should not error");
            assert!(resp.result.unwrap()[key].is_array());
        }

        // 3. Verify all 47 tools in manifest are valid and wired in dispatch_tool_call
        let manifest = mcp_tools_manifest();
        let tools = manifest["tools"].as_array().expect("tools array");
        assert_eq!(tools.len(), 47, "Expected exactly 47 MCP tools in manifest");

        for tool in tools {
            let name = tool["name"].as_str().expect("tool name");
            let desc = tool["description"].as_str().expect("tool description");
            assert!(!name.is_empty() && !desc.is_empty());
            assert_eq!(tool["inputSchema"]["type"], "object");

            // Dispatch with empty args: even if a tool requires parameters (Validation error),
            // it must NEVER fail with "Unknown MCP tool".
            let res = dispatch_tool_call(&service, name, json!({}));
            if let Err(err) = res {
                let msg = err.to_string();
                assert!(
                    !msg.contains("Unknown MCP tool"),
                    "Tool {name} is in schemas.rs but missing in dispatch_tool_call: {msg}"
                );
            }
        }

        // 4. Verify daypress_session_secret is blocked from modification via daypress_update_option
        let blocked = dispatch_tool_call(
            &service,
            "daypress_update_option",
            json!({
                "name": "daypress_session_secret",
                "value": "attacker_controlled"
            }),
        );
        assert!(
            blocked.is_err(),
            "daypress_session_secret must be protected"
        );
    }

    fn call_tool(service: &CmsService, name: &str, arguments: Value) -> (bool, String) {
        let resp = handle_mcp_rpc(
            service,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(1)),
                method: "tools/call".into(),
                params: json!({ "name": name, "arguments": arguments }),
            },
        );
        let res = resp.result.expect("MCP result");
        (
            res["isError"].as_bool().unwrap_or(true),
            res["content"][0]["text"].as_str().unwrap_or("").to_string(),
        )
    }

    const AGENCY_DESIGN: &str = r##"<!doctype html><html><head><title>Werkwijze | Studio Noord</title>
<style>
:root { --ink: #111; }
.site-header { position: absolute; inset: 0 0 auto 0; }
.nav { display: flex; gap: 24px; }
@media (max-width: 60rem) { .nav { display: none; } .nav-open .nav { display: block; } }
.field { display: flex; flex-direction: column; }
</style></head><body>
<header class="site-header"><a class="brand" href="/">Studio Noord</a><nav class="nav" id="menu"><a href="/">Home</a></nav></header>
<main>
<section class="hero"><h1>Werkwijze</h1></section>
<section class="contact"><form action="https://formspree.io/f/x" method="POST">
<input type="hidden" name="_subject" value="Nieuwe aanvraag">
<div class="field"><label for="n">Naam</label><input id="n" name="naam" required></div>
<fieldset><legend>Interesse</legend><label><input type="checkbox" name="interesse[]" value="Keuken"> Keuken</label><label><input type="checkbox" name="interesse[]" value="Deur"> Deur</label></fieldset>
<div class="field"><label for="f">Bijlage</label><input type="file" id="f" name="bijlage" accept=".pdf"></div>
<button type="submit">Verstuur</button>
<div class="ok" role="status" hidden>Bedankt!</div>
</form></section>
</main>
<footer class="site-footer"><p>Studio Noord</p></footer>
</body></html>"##;

    fn form_ids_in(node: &daypress_ast::DaypressNode, out: &mut Vec<i64>) {
        if let daypress_ast::NodeKind::Form { form_id, .. } = node.kind {
            out.push(form_id);
        }
        for c in &node.children {
            form_ids_in(c, out);
        }
    }

    #[test]
    fn import_html_design_creates_templates_and_forms_and_updates_them_on_reimport() {
        let db = Database::new_in_memory().expect("in-memory db");
        let service = CmsService::from_adapter(db);

        let (err, text) = call_tool(
            &service,
            "daypress_import_html_design",
            json!({ "html": AGENCY_DESIGN, "set_as_front_page": true }),
        );
        assert!(!err, "{text}");
        let first: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(first["site_name"], "Studio Noord");
        assert_eq!(first["header_template"]["title"], "Studio Noord header");
        assert_eq!(first["header_template"]["status"], "created");
        assert_eq!(first["footer_template"]["title"], "Studio Noord footer");
        assert_eq!(first["forms"][0]["title"], "Studio Noord: Verstuur");
        assert_eq!(first["forms"][0]["status"], "created");
        let form_id = first["forms"][0]["id"].as_i64().unwrap();
        assert!(form_id > 0);

        // Templates: active, shown on the entire site, native blocks (no raw HTML).
        let templates = service.options_repo().list_theme_templates(None).unwrap();
        assert_eq!(templates.len(), 2);
        for t in &templates {
            assert_eq!(t["is_active"], true);
            assert_eq!(t["conditions"][0]["rule_target"], "entire_site");
            let layout: daypress_ast::DaypressLayout =
                serde_json::from_value(t["layout"].clone()).unwrap();
            assert_eq!(count_custom_html(&layout.root), 0);
        }
        let header = templates
            .iter()
            .find(|t| t["template_type"] == "header")
            .unwrap();
        let header_html = header["rendered_html"].as_str().unwrap();
        assert!(
            header_html.contains("site-header dp-design"),
            "{header_html}"
        );
        assert!(
            header_html.contains("daypress-kit-css"),
            "the header carries the design CSS"
        );
        // Nothing goes to the never-rendered legacy chrome options any more.
        assert!(service
            .get_site_option("elementor_header_html")
            .unwrap()
            .is_none());

        // FormStudio form built from the markup.
        let form = service
            .options_repo()
            .get_form_by_id(form_id)
            .unwrap()
            .unwrap();
        let fields: Vec<Value> = serde_json::from_str(&form.schema_json).unwrap();
        let kinds: Vec<(String, String)> = fields
            .iter()
            .map(|f| {
                (
                    f["name"].as_str().unwrap().into(),
                    f["field_type"].as_str().unwrap().into(),
                )
            })
            .collect();
        assert_eq!(
            kinds,
            vec![
                ("naam".to_string(), "text".to_string()),
                ("interesse".to_string(), "checkbox_group".to_string()),
                ("bijlage".to_string(), "file".to_string()),
            ]
        );
        assert_eq!(fields[0]["required"], true);
        assert_eq!(fields[1]["options"], json!(["Keuken", "Deur"]));
        assert_eq!(fields[2]["accept_extensions"], "pdf");
        let settings: Value = serde_json::from_str(&form.settings_json).unwrap();
        assert_eq!(settings["submit_label"], "Verstuur");
        assert_eq!(settings["success_message"], "Bedankt!");
        assert_eq!(settings["email_subject_template"], "Nieuwe aanvraag");

        // The page's Form node points at the saved form.
        let post_id = first["post_id"].as_i64().unwrap();
        let layout: daypress_ast::DaypressLayout =
            serde_json::from_str(&service.get_layout_ast(post_id).unwrap().unwrap()).unwrap();
        let mut ids = Vec::new();
        form_ids_in(&layout.root, &mut ids);
        assert_eq!(ids, vec![form_id]);
        let rendered = daypress_ast::render_node_to_html(&layout.root);
        assert!(rendered.contains(&format!("/wp-json/daypress/v1/forms/{form_id}/submit")));
        assert!(!rendered.contains("site-header"));

        let forms_after_first = service.options_repo().list_forms().unwrap().len();

        // Someone sets a recipient; a re-import updates in place and keeps it.
        let mut s: crate::domain::models::FormSettings =
            serde_json::from_str(&form.settings_json).unwrap();
        s.recipient_email = "info@example.test".into();
        service
            .options_repo()
            .save_form_record(&crate::domain::models::CreateFormRequest {
                id: Some(form_id),
                title: form.title.clone(),
                fields: serde_json::from_str(&form.schema_json).unwrap(),
                settings: s,
            })
            .unwrap();

        let (err, text) = call_tool(
            &service,
            "daypress_import_html_design",
            json!({ "html": AGENCY_DESIGN, "set_as_front_page": true }),
        );
        assert!(!err, "{text}");
        let second: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(second["post_id"], first["post_id"]);
        assert_eq!(second["header_template"]["status"], "updated");
        assert_eq!(
            second["header_template"]["id"],
            first["header_template"]["id"]
        );
        assert_eq!(second["footer_template"]["status"], "updated");
        assert_eq!(second["forms"][0]["status"], "updated");
        assert_eq!(second["forms"][0]["id"].as_i64(), Some(form_id));
        assert_eq!(
            service
                .options_repo()
                .list_theme_templates(None)
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            service.options_repo().list_forms().unwrap().len(),
            forms_after_first
        );
        let kept = service
            .options_repo()
            .get_form_by_id(form_id)
            .unwrap()
            .unwrap();
        assert!(kept.settings_json.contains("info@example.test"));
    }

    #[test]
    fn save_template_rejects_a_layout_without_root() {
        let db = Database::new_in_memory().expect("in-memory db");
        let service = CmsService::from_adapter(db);

        // A bare node instead of a layout.
        let (err, text) = call_tool(
            &service,
            "daypress_save_template",
            json!({ "title": "Broken", "template_type": "header",
                    "layout": { "id": "x", "type": "Section", "children": [] } }),
        );
        assert!(err);
        assert!(text.contains("`root`"), "{text}");

        // The whole daypress_get_post_layout response instead of its `layout_json`.
        let (err, text) = call_tool(
            &service,
            "daypress_save_template",
            json!({ "title": "Broken", "template_type": "header",
                    "layout": { "post_id": 4, "layout_json": "{}" } }),
        );
        assert!(err);
        assert!(text.contains("layout_json"), "{text}");

        // `layout_json` given as a string is checked too.
        let (err, _) = call_tool(
            &service,
            "daypress_save_template",
            json!({ "title": "Broken", "template_type": "footer", "layout_json": "{\"version\":1}" }),
        );
        assert!(err);
        assert!(service
            .options_repo()
            .list_theme_templates(None)
            .unwrap()
            .is_empty());

        let (err, text) = call_tool(
            &service,
            "daypress_save_template",
            json!({ "title": "Good", "template_type": "footer",
                    "layout": { "version": 1, "root": { "id": "root", "type": "Root", "children": [
                        { "id": "t", "type": "Text", "data": { "content_html": "Hi" }, "styles": {} } ] } } }),
        );
        assert!(!err, "{text}");
    }
}
