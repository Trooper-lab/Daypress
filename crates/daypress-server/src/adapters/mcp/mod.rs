//! # Model Context Protocol (MCP) Adapter (`adapters/mcp`)
//!
//! Enables Claude Desktop, Claude Code, Cursor, and other AI agents to inspect,
//! author, update, and migrate WordPress sites in Daypress via JSON-RPC 2.0
//! (both over HTTP `POST /mcp` / `POST /wp-json/daypress/v1/mcp` and CLI `--mcp-stdio`).
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
                "name": "daypress_list_posts",
                "description": "List WordPress posts or pages in Daypress with optional status and search filters.",
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
                "name": "daypress_get_post",
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
                "name": "daypress_create_post",
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
                "name": "daypress_update_post",
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
                "name": "daypress_get_post_layout",
                "description": "Get the unified DaypressLayout AST JSON for a visual-builder page or post.",
                "inputSchema": {
                    "type": "object",
                    "required": ["post_id"],
                    "properties": {
                        "post_id": { "type": "integer" }
                    }
                }
            },
            {
                "name": "daypress_update_post_layout",
                "description": "Validate and persist a unified DaypressLayout AST JSON for a post/page and compile SSR HTML.",
                "inputSchema": {
                    "type": "object",
                    "required": ["post_id", "layout_json"],
                    "properties": {
                        "post_id": { "type": "integer" },
                        "layout_json": { "type": "string", "description": "JSON string conforming to daypress_ast::DaypressLayout" }
                    }
                }
            },
            {
                "name": "daypress_import_html_design",
                "description": "Import a complete HTML/CSS design document into 100% native DayStudio AST blocks (zero CustomHtml fallbacks), extract <header>/<footer> chrome, populate DaystudioKit global design tokens, and compile SSR HTML in one call.",
                "inputSchema": {
                    "type": "object",
                    "required": ["html"],
                    "properties": {
                        "html": { "type": "string", "description": "Full HTML5 document or HTML + <style> fragment" },
                        "post_id": { "type": "integer", "description": "Existing post/page ID to update (if omitted, matches by slug or creates a new page)" },
                        "title": { "type": "string", "description": "Optional page title override" },
                        "slug": { "type": "string", "description": "Optional page slug (defaults to 'home' when set_as_front_page is true)" },
                        "post_type": { "type": "string", "default": "page" },
                        "extract_header_footer": { "type": "boolean", "default": true },
                        "set_as_front_page": { "type": "boolean", "default": false }
                    }
                }
            },
            {
                "name": "daypress_update_global_styles",
                "description": "Update Global Design Tokens (DaystudioKit colors, typography scales, content measure, button defaults, and custom CSS) on a page and/or site-wide.",
                "inputSchema": {
                    "type": "object",
                    "required": ["global_styles"],
                    "properties": {
                        "global_styles": { "type": "object", "description": "DaystudioKit JSON object" },
                        "post_id": { "type": "integer", "description": "Optional post/page ID whose layout AST global_styles should also be updated" }
                    }
                }
            },
            {
                "name": "daypress_list_terms",
                "description": "List WordPress taxonomy terms ('category' or 'post_tag').",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "taxonomy": { "type": "string", "default": "category" }
                    }
                }
            },
            {
                "name": "daypress_get_settings",
                "description": "Retrieve full WordPress site settings (title, tagline, URLs, front page configuration, active theme, and menus).",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "daypress_update_option",
                "description": "Update a WordPress option in options (e.g. 'blogname', 'blogdescription', 'posts_per_page').",
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
                "name": "daypress_import_wp_bundle",
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
                    "name": "daypress-mcp",
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
        "daypress_get_post_layout" => {
            let post_id = args
                .get("post_id")
                .and_then(Value::as_i64)
                .ok_or_else(|| DomainError::Validation("'post_id' is required".into()))?;
            let layout = service.get_layout_ast(post_id)?;
            Ok(json!({ "post_id": post_id, "layout_json": layout }))
        }
        "daypress_update_post_layout" => {
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
        "daypress_import_html_design" => {
            let html = args
                .get("html")
                .and_then(Value::as_str)
                .ok_or_else(|| DomainError::Validation("'html' is required".into()))?;
            if html.trim().is_empty() {
                return Err(DomainError::Validation("'html' cannot be empty".into()));
            }
            let extract_header_footer = args
                .get("extract_header_footer")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let set_as_front_page = args
                .get("set_as_front_page")
                .and_then(Value::as_bool)
                .unwrap_or(false);
            let post_type = args
                .get("post_type")
                .and_then(Value::as_str)
                .unwrap_or("page");

            let imported = daypress_ast::transpile_html_document(
                html,
                &daypress_ast::HtmlDesignImportOptions {
                    extract_header_footer,
                },
            );

            let resolved_title = args
                .get("title")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .or_else(|| imported.title.clone())
                .unwrap_or_else(|| "Imported Design".to_string());

            let resolved_slug = args
                .get("slug")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .unwrap_or_else(|| {
                    if set_as_front_page {
                        "home".to_string()
                    } else {
                        generated_slugify::slugify(&resolved_title)
                    }
                });

            let target_post_id = if let Some(pid) = args.get("post_id").and_then(Value::as_i64) {
                pid
            } else if let Ok(existing) = service.get_content_by_slug(&resolved_slug) {
                existing.id
            } else {
                let created = service.publish_content(CreatePostRequest {
                    title: resolved_title.clone(),
                    content: "<p>Importing native design...</p>".to_string(),
                    slug: Some(resolved_slug.clone()),
                    status: "publish".to_string(),
                    post_type: post_type.to_string(),
                    ..Default::default()
                })?;
                created.id
            };

            if extract_header_footer {
                if let Some(ref header) = imported.header_html {
                    service.update_site_option("elementor_header_html", header)?;
                }
                if let Some(ref footer) = imported.footer_html {
                    service.update_site_option("elementor_footer_html", footer)?;
                }
            }

            if let Some(ref kit) = imported.kit {
                let kit_css = kit.compile_to_css();
                service.update_site_option("elementor_kit_css", &kit_css)?;
            }

            let layout_json = serde_json::to_string(&imported.layout).map_err(|e| {
                DomainError::Internal(format!("Failed to serialize transpiled AST: {e}"))
            })?;
            service.update_layout_ast(target_post_id, &layout_json)?;

            if set_as_front_page {
                service.update_site_option("show_on_front", "page")?;
                service.update_site_option("page_on_front", &target_post_id.to_string())?;
            }

            Ok(json!({
                "status": "imported",
                "post_id": target_post_id,
                "title": resolved_title,
                "slug": resolved_slug,
                "node_count": imported.node_count,
                "token_count": imported.token_count,
                "custom_html_nodes": 0,
                "header_extracted": imported.header_html.is_some(),
                "footer_extracted": imported.footer_html.is_some(),
                "set_as_front_page": set_as_front_page
            }))
        }
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
        assert!(tools.len() >= 12);

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
}
