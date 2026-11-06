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
                "description": "Get the unified DaypressLayout AST for a visual-builder page or post. Returns {\"post_id\", \"layout_json\"} where `layout_json` is the layout as a JSON string (parse it; pass the parsed object as `layout` to daypress_save_template).",
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
                "description": "Import a complete HTML/CSS design document in one call: the page becomes 100% native DayStudio AST blocks (zero CustomHtml fallbacks) with the design's stylesheet in its DaystudioKit; the body-level <header> (everything before <main>) and <footer> (everything after it, e.g. a mobile call bar) become active, site-wide 'header'/'footer' theme templates titled '{site_name} header'/'{site_name} footer'; every <form> becomes a FormStudio form titled '{site_name}: {submit label}' (fields from labels, names, required, placeholders and options; checkbox groups -> checkbox_group, file inputs -> file) rendered with the design's own field markup, so submissions land in the Submissions Vault. Re-importing updates the same page, templates and forms instead of duplicating them. Scripts are not imported (tabs, carousels, script menus need native equivalents).",
                "inputSchema": {
                    "type": "object",
                    "required": ["html"],
                    "properties": {
                        "html": { "type": "string", "description": "Full HTML5 document or HTML + <style> fragment" },
                        "post_id": { "type": "integer", "description": "Existing post/page ID to update (if omitted, matches by slug or creates a new page)" },
                        "title": { "type": "string", "description": "Optional page title override" },
                        "slug": { "type": "string", "description": "Optional page slug (defaults to 'home' when set_as_front_page is true)" },
                        "site_name": { "type": "string", "description": "Name used for the header/footer template and form titles (defaults to the last segment of the design's <title>, e.g. 'Grezzo')" },
                        "post_type": { "type": "string", "default": "page" },
                        "extract_header_footer": { "type": "boolean", "default": true, "description": "Save the design's header and footer as site-wide theme templates (false keeps them in the page)" },
                        "extract_forms": { "type": "boolean", "default": true, "description": "Turn <form> elements into FormStudio forms + Form blocks" },
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
            },
            {
                "name": "daypress_get_publish_status",
                "description": "Retrieve static edge publishing configuration (Cloudflare Pages, Netlify, or Local/ZIP), published page count, and unpublished change queue.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "daypress_publish_site",
                "description": "Configure or trigger a static edge deployment of the local Daypress site to Cloudflare Pages, Netlify, or a local static export directory.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "provider": { "type": "string", "description": "Optional target override ('local', 'cloudflare', or 'netlify')" },
                        "production_url": { "type": "string", "description": "Optional canonical production URL (e.g. 'https://example.com')" },
                        "local_export_dir": { "type": "string", "description": "Optional local export folder path when provider is 'local'" }
                    }
                }
            },
            {
                "name": "daypress_list_forms",
                "description": "List all FormStudio forms in Daypress with their field schemas, multi-step settings, and submission counts.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "daypress_get_form",
                "description": "Retrieve a single FormStudio form by numeric ID.",
                "inputSchema": {
                    "type": "object",
                    "required": ["id"],
                    "properties": {
                        "id": { "type": "integer", "description": "Numeric form ID" }
                    }
                }
            },
            {
                "name": "daypress_create_form",
                "description": "Create a new FormStudio form with a JSON array of fields (`FormFieldSchema`) and optional `FormSettings` JSON.",
                "inputSchema": {
                    "type": "object",
                    "required": ["title", "fields"],
                    "properties": {
                        "title": { "type": "string" },
                        "fields": { "type": "array", "description": "Array of FormFieldSchema objects" },
                        "settings": { "type": "object", "description": "Optional FormSettings object" }
                    }
                }
            },
            {
                "name": "daypress_update_form",
                "description": "Update an existing FormStudio form's title, fields array, or settings object by ID.",
                "inputSchema": {
                    "type": "object",
                    "required": ["id"],
                    "properties": {
                        "id": { "type": "integer" },
                        "title": { "type": "string" },
                        "fields": { "type": "array" },
                        "settings": { "type": "object" }
                    }
                }
            },
            {
                "name": "daypress_list_form_submissions",
                "description": "Query the FormStudio Submissions Vault with optional form_id, status ('unread', 'read', 'spam', 'trash', 'all'), search, and starred filters.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "form_id": { "type": "integer" },
                        "status": { "type": "string", "default": "all" },
                        "search": { "type": "string" },
                        "starred_only": { "type": "boolean", "default": false },
                        "limit": { "type": "integer", "default": 50 }
                    }
                }
            },
            {
                "name": "daypress_get_analytics",
                "description": "Retrieve the Plausible-style Cookieless Analytics dashboard summary (pageviews, unique visitors, bounce rate, time-series, top pages, referrers, UTM campaigns, devices, countries, and conversions).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "range": { "type": "string", "default": "30d", "description": "Date range ('today', '7d', '30d', '90d', '12m', or 'all')" }
                    }
                }
            },
            {
                "name": "daypress_list_templates",
                "description": "List Dynamic Theme Templates, Synced Components (Symbols), and Site-Wide Popups/Announcement Bars ('header', 'footer', 'single_post', 'single_page', 'archive', 'error_404', 'symbol', 'popup') with their display conditions.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "template_type": { "type": "string", "description": "Optional filter ('header', 'footer', 'single_post', 'single_page', 'archive', 'error_404', 'symbol', 'popup', or 'all')" }
                    }
                }
            },
            {
                "name": "daypress_save_template",
                "description": "Create or update a Dynamic Theme Template, Synced Component ('symbol'), or Site-Wide Popup ('popup') with DaypressLayout AST, settings, and display conditions.",
                "inputSchema": {
                    "type": "object",
                    "required": ["title", "template_type"],
                    "properties": {
                        "id": { "type": "integer", "description": "Optional template ID to update an existing template" },
                        "title": { "type": "string" },
                        "template_type": { "type": "string", "description": "Template type ('header', 'footer', 'single_post', 'single_page', 'archive', 'error_404', 'symbol', 'popup')" },
                        "status": { "type": "string", "default": "publish" },
                        "priority": { "type": "integer", "default": 10 },
                        "is_active": { "type": "boolean", "default": true },
                        "layout": { "type": "object", "description": "DaypressLayout AST JSON object with a `root` node ({\"version\": 1, \"root\": {\"id\": \"root\", \"type\": \"Root\", \"children\": [...]}}); a layout without `root` is rejected" },
                        "settings": { "type": "object", "description": "Optional TemplateSettings (e.g. popup_variant, trigger, dismiss_days)" },
                        "conditions": { "type": "array", "description": "Optional array of CreateConditionDto rules" }
                    }
                }
            },
            {
                "name": "daypress_list_media_folders",
                "description": "List all virtual Media Library DAM folders with item counts, or create a new virtual folder when 'create_name' is provided.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "create_name": { "type": "string", "description": "Optional folder name to create before returning the folder list" },
                        "parent_id": { "type": "integer", "default": 0, "description": "Optional parent folder ID when creating a folder" }
                    }
                }
            },
            {
                "name": "daypress_scan_unused_media",
                "description": "Scan all media attachments across posts, daypress_templates, _thumbnail_id, and site options, returning unreferenced media assets.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "daypress_get_storage_s3_status",
                "description": "Retrieve S3 / Cloudflare R2 object storage offload configuration and continuous SQLite replica status.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "daypress_list_post_types",
                "description": "List all registered Custom Post Types (CPTs) with their labels, rewrite slugs, taxonomies, and item counts.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "daypress_save_post_type",
                "description": "Create or update a Custom Post Type definition in SQLite (`daypress_post_types`).",
                "inputSchema": {
                    "type": "object",
                    "required": ["slug", "singular_label", "plural_label"],
                    "properties": {
                        "slug": { "type": "string", "description": "Machine key (e.g. 'case_study')" },
                        "singular_label": { "type": "string" },
                        "plural_label": { "type": "string" },
                        "description": { "type": "string" },
                        "menu_icon": { "type": "string", "default": "dashicons-portfolio" },
                        "public": { "type": "boolean", "default": true },
                        "has_archive": { "type": "boolean", "default": true },
                        "hierarchical": { "type": "boolean", "default": false },
                        "show_in_rest": { "type": "boolean", "default": true },
                        "rest_base": { "type": "string" },
                        "rewrite_slug": { "type": "string" },
                        "supports": { "type": "array", "items": { "type": "string" } },
                        "taxonomies": { "type": "array", "items": { "type": "string" } }
                    }
                }
            },
            {
                "name": "daypress_provision_cpt_preset",
                "description": "Provision a complete 1-click Custom Post Type blueprint ('case_studies', 'team', 'docs', 'jobs', 'events', 'portfolio', or 'testimonials') including linked taxonomies, ACF structured field group, and single/archive theme templates.",
                "inputSchema": {
                    "type": "object",
                    "required": ["preset"],
                    "properties": {
                        "preset": { "type": "string", "description": "Preset key ('case_studies', 'team', 'docs', 'jobs', 'events', 'portfolio', 'testimonials')" }
                    }
                }
            },
            {
                "name": "daypress_list_field_groups",
                "description": "List all Structured Field Groups (ACF parity) with their field schemas and location rules.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "post_type": { "type": "string", "description": "Optional post_type filter" }
                    }
                }
            },
            {
                "name": "daypress_save_field_group",
                "description": "Create or update a Structured Field Group (`daypress_field_groups`) with typed fields ('text', 'textarea', 'wysiwyg', 'number', 'date', 'url', 'true_false', 'select', 'checkbox', 'media', 'post_object', 'repeater') and location rules.",
                "inputSchema": {
                    "type": "object",
                    "required": ["title", "fields"],
                    "properties": {
                        "id": { "type": "integer" },
                        "group_key": { "type": "string" },
                        "title": { "type": "string" },
                        "description": { "type": "string" },
                        "position": { "type": "string", "default": "normal" },
                        "menu_order": { "type": "integer", "default": 0 },
                        "is_active": { "type": "boolean", "default": true },
                        "location_rules": { "type": "array" },
                        "fields": { "type": "array" }
                    }
                }
            },
            {
                "name": "daypress_search",
                "description": "Search published posts, pages, and Custom Post Types using the native SQLite FTS5 BM25 full-text index with typo tolerance and highlighted snippets.",
                "inputSchema": {
                    "type": "object",
                    "required": ["query"],
                    "properties": {
                        "query": { "type": "string", "description": "Full-text search query (supports prefix and typo-tolerant matching)" },
                        "post_type": { "type": "string", "description": "Optional post_type filter (e.g. 'post', 'page', 'feature', or 'all')" },
                        "limit": { "type": "integer", "default": 12 }
                    }
                }
            },
            {
                "name": "daypress_audit_post",
                "description": "Run the real-time WCAG 2.1 AA/AAA Contrast, Broken-Link (SQLite-verified), Heading Hierarchy, and SEO Linter on a post/page by post_id or draft payload.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "post_id": { "type": "integer", "description": "Optional post/page ID to audit and persist _daypress_audit_score" },
                        "title": { "type": "string" },
                        "html": { "type": "string" },
                        "layout_json": { "type": "string" },
                        "theme": { "type": "string", "default": "light" }
                    }
                }
            },
            {
                "name": "daypress_list_languages",
                "description": "List all configured site languages, the default language, and whether the default language uses a URL prefix.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "daypress_link_translations",
                "description": "Link localized posts/pages into a shared translation group (`daypress_translations`) so `<link rel=\"alternate\" hreflang>` and the LanguageSwitcher widget resolve across locales.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "source_post_id": { "type": "integer" },
                        "target_post_id": { "type": "integer" },
                        "target_lang": { "type": "string", "description": "ISO language code (e.g. 'de', 'nl', 'fr', 'es')" },
                        "translations": { "type": "object", "description": "Map of lang_code -> post_id (e.g. {\"en\": 1, \"de\": 2})" },
                        "upsert_language_code": { "type": "string", "description": "Optional language code to auto-enable in daypress_languages" }
                    }
                }
            },
            {
                "name": "daypress_duplicate_post",
                "description": "1-Click clone of any post, page, or Custom Post Type item into a new Draft, preserving its Daystudio layout_json, structured fields (ACF/postmeta), taxonomies, co-authors, and language.",
                "inputSchema": {
                    "type": "object",
                    "required": ["post_id"],
                    "properties": {
                        "post_id": { "type": "integer", "description": "ID of the source post or page to duplicate" }
                    }
                }
            },
            {
                "name": "daypress_create_preview_link",
                "description": "Generate a time-bound, revocable secret draft preview link (`?preview_token=...` and `/preview/{token}`) so external stakeholders can preview an unpublished draft without logging in.",
                "inputSchema": {
                    "type": "object",
                    "required": ["post_id"],
                    "properties": {
                        "post_id": { "type": "integer" },
                        "expires_in_hours": { "type": "integer", "default": 48 },
                        "label": { "type": "string", "default": "Stakeholder Preview" }
                    }
                }
            },
            {
                "name": "daypress_schedule_revision",
                "description": "Schedule a future revision update to an already-published post or page without unpublishing the live URL. When `scheduled_date_gmt` arrives, Daypress snapshots the live post and atomically swaps in the scheduled revision.",
                "inputSchema": {
                    "type": "object",
                    "required": ["post_id", "scheduled_date_gmt"],
                    "properties": {
                        "post_id": { "type": "integer" },
                        "scheduled_date_gmt": { "type": "string", "description": "ISO-8601 / SQLite UTC timestamp (e.g. '2026-12-01 09:00:00')" },
                        "title": { "type": "string" },
                        "content": { "type": "string" },
                        "excerpt": { "type": "string" },
                        "layout_json": { "type": "string" }
                    }
                }
            },
            {
                "name": "daypress_diff_revisions",
                "description": "Compute a structural AST & content diff between a post's revision and its current live state (or between two revisions), returning added, modified, and removed AST nodes and property-level diffs.",
                "inputSchema": {
                    "type": "object",
                    "required": ["post_id"],
                    "properties": {
                        "post_id": { "type": "integer" },
                        "from_revision_id": { "type": "integer", "description": "Baseline revision ID (defaults to latest revision)" },
                        "to_revision_id": { "type": "integer", "description": "Target revision ID (defaults to live post state when omitted)" }
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
            .map_or(true, str::is_empty)
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
            let summary = service.options_repo().get_analytics_dashboard(range)?;
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
