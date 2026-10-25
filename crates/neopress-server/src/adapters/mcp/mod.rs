//! # Model Context Protocol (MCP) Adapter (`adapters/mcp`)
//!
//! Enables Claude Desktop, Claude Code, Cursor, and other AI agents to inspect,
//! author, update, and migrate WordPress sites in Neopress via JSON-RPC 2.0
//! (both over HTTP `POST /mcp` / `POST /wp-json/neopress/v1/mcp` and CLI `--mcp-stdio`).
//!
//! Drives the core [`CmsService`] exclusively through the Hexagonal Input Ports
//! ([`ContentUseCase`], [`MigrationUseCase`], [`SettingsUseCase`]).

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

/// Returns the MCP tool manifest describing all AI-accessible CMS capabilities.
pub fn mcp_tools_manifest() -> Value {
    json!({
        "tools": [
            {
                "name": "neopress_list_posts",
                "description": "List WordPress posts or pages in Neopress with optional status and search filters.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "post_type": { "type": "string", "default": "post", "description": "Post type ('post', 'page', or 'any')" },
                        "status": { "type": "string", "default": "publish", "description": "Post status ('publish', 'draft', 'any')" },
                        "search": { "type": "string", "description": "Optional keyword search query" },
                        "limit": { "type": "integer", "default": 20 }
                    }
                }
            },
            {
                "name": "neopress_get_post",
                "description": "Retrieve a single WordPress post or page by numeric ID or URL slug.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer", "description": "Numeric post ID" },
                        "slug": { "type": "string", "description": "Post URL slug (used if id is omitted)" }
                    }
                }
            },
            {
                "name": "neopress_create_post",
                "description": "Create and publish or draft a new WordPress post or page.",
                "inputSchema": {
                    "type": "object",
                    "required": ["title", "content"],
                    "properties": {
                        "title": { "type": "string" },
                        "content": { "type": "string", "description": "HTML or Gutenberg block markup" },
                        "excerpt": { "type": "string" },
                        "slug": { "type": "string" },
                        "status": { "type": "string", "default": "publish" },
                        "post_type": { "type": "string", "default": "post" }
                    }
                }
            },
            {
                "name": "neopress_update_post",
                "description": "Update an existing WordPress post or page by ID (title, HTML content, excerpt, status, or SEO metadata).",
                "inputSchema": {
                    "type": "object",
                    "required": ["id"],
                    "properties": {
                        "id": { "type": "integer" },
                        "title": { "type": "string" },
                        "content": { "type": "string" },
                        "excerpt": { "type": "string" },
                        "slug": { "type": "string" },
                        "status": { "type": "string" },
                        "seo_title": { "type": "string" },
                        "seo_description": { "type": "string" }
                    }
                }
            },
            {
                "name": "neopress_get_post_layout",
                "description": "Get the unified NeopressLayout AST JSON for a visual-builder page or post.",
                "inputSchema": {
                    "type": "object",
                    "required": ["post_id"],
                    "properties": {
                        "post_id": { "type": "integer" }
                    }
                }
            },
            {
                "name": "neopress_update_post_layout",
                "description": "Validate and persist a unified NeopressLayout AST JSON for a post/page and compile SSR HTML.",
                "inputSchema": {
                    "type": "object",
                    "required": ["post_id", "layout_json"],
                    "properties": {
                        "post_id": { "type": "integer" },
                        "layout_json": { "type": "string", "description": "JSON string conforming to neopress_ast::NeopressLayout" }
                    }
                }
            },
            {
                "name": "neopress_list_terms",
                "description": "List WordPress taxonomy terms ('category' or 'post_tag').",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "taxonomy": { "type": "string", "default": "category" }
                    }
                }
            },
            {
                "name": "neopress_get_settings",
                "description": "Retrieve full WordPress site settings (title, tagline, URLs, front page configuration, active theme, and menus).",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "neopress_update_option",
                "description": "Update a WordPress option in wp_options (e.g. 'blogname', 'blogdescription', 'posts_per_page').",
                "inputSchema": {
                    "type": "object",
                    "required": ["name", "value"],
                    "properties": {
                        "name": { "type": "string" },
                        "value": { "type": "string" }
                    }
                }
            },
            {
                "name": "neopress_import_wp_bundle",
                "description": "Execute a 1-Click WordPress migration bundle import with relational ID remapping and Elementor/Divi/Gutenberg AST transpilation.",
                "inputSchema": {
                    "type": "object",
                    "required": ["bundle"],
                    "properties": {
                        "bundle": { "type": "object", "description": "ImportBundleRequest JSON payload" }
                    }
                }
            }
        ]
    })
}

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
                    "name": "neopress-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            error: None,
        },
        "notifications/initialized" | "ping" => JsonRpcResponse {
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

fn dispatch_tool_call(
    service: &CmsService,
    tool_name: &str,
    args: Value,
) -> Result<Value, DomainError> {
    match tool_name {
        "neopress_list_posts" => {
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
        "neopress_get_post" => {
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
        "neopress_create_post" => {
            let req: CreatePostRequest = serde_json::from_value(args)
                .map_err(|e| DomainError::Validation(format!("Invalid create_post args: {e}")))?;
            let created = service.publish_content(req)?;
            Ok(serde_json::to_value(created).unwrap_or(Value::Null))
        }
        "neopress_update_post" => {
            let id = args
                .get("id")
                .and_then(Value::as_i64)
                .ok_or_else(|| DomainError::Validation("'id' is required".into()))?;
            let req: UpdatePostRequest = serde_json::from_value(args)
                .map_err(|e| DomainError::Validation(format!("Invalid update_post args: {e}")))?;
            let updated = service.edit_content(id, req)?;
            Ok(serde_json::to_value(updated).unwrap_or(Value::Null))
        }
        "neopress_get_post_layout" => {
            let post_id = args
                .get("post_id")
                .and_then(Value::as_i64)
                .ok_or_else(|| DomainError::Validation("'post_id' is required".into()))?;
            let layout = service.get_layout_ast(post_id)?;
            Ok(json!({ "post_id": post_id, "layout_json": layout }))
        }
        "neopress_update_post_layout" => {
            let post_id = args
                .get("post_id")
                .and_then(Value::as_i64)
                .ok_or_else(|| DomainError::Validation("'post_id' is required".into()))?;
            let layout_json = args
                .get("layout_json")
                .and_then(Value::as_str)
                .ok_or_else(|| DomainError::Validation("'layout_json' is required".into()))?;
            service.update_layout_ast(post_id, layout_json)?;
            Ok(json!({ "success": true, "post_id": post_id }))
        }
        "neopress_list_terms" => {
            let taxonomy = args
                .get("taxonomy")
                .and_then(Value::as_str)
                .unwrap_or("category");
            let terms = service.list_taxonomies(taxonomy)?;
            Ok(serde_json::to_value(terms).unwrap_or(Value::Null))
        }
        "neopress_get_settings" => {
            let settings = service.get_site_settings()?;
            Ok(serde_json::to_value(settings).unwrap_or(Value::Null))
        }
        "neopress_update_option" => {
            let name = args
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| DomainError::Validation("'name' is required".into()))?;
            let value = args
                .get("value")
                .and_then(Value::as_str)
                .ok_or_else(|| DomainError::Validation("'value' is required".into()))?;
            service.update_site_option(name, value)?;
            Ok(json!({ "success": true, "option_name": name, "option_value": value }))
        }
        "neopress_import_wp_bundle" => {
            let bundle_val = args.get("bundle").cloned().unwrap_or(args);
            let bundle: ImportBundleRequest = serde_json::from_value(bundle_val)
                .map_err(|e| DomainError::Validation(format!("Invalid WP bundle payload: {e}")))?;
            let res = service.migrate_wordpress_bundle(bundle)?;
            Ok(serde_json::to_value(res).unwrap_or(Value::Null))
        }
        unknown => Err(DomainError::NotFound(format!(
            "Unknown MCP tool: {unknown}"
        ))),
    }
}

/// Axum HTTP handler for `POST /mcp` and `POST /wp-json/neopress/v1/mcp`.
pub async fn handle_http_mcp(
    _auth: crate::auth::AdminUser,
    State(db): State<crate::db::Database>,
    Json(req): Json<JsonRpcRequest>,
) -> Json<JsonRpcResponse> {
    let service = CmsService::from_adapter(db);
    Json(handle_mcp_rpc(&service, req))
}

/// Runs the MCP server over `stdin`/`stdout` (`neopress --mcp-stdio`) for Claude Desktop & Claude Code.
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
            Ok(req) => handle_mcp_rpc(service, req),
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
            "neopress-mcp"
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
        assert!(tools.len() >= 10);

        // 3. tools/call -> neopress_create_post
        let create_resp = handle_mcp_rpc(
            &service,
            JsonRpcRequest {
                jsonrpc: Some("2.0".into()),
                id: Some(json!(3)),
                method: "tools/call".into(),
                params: json!({
                    "name": "neopress_create_post",
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
}
