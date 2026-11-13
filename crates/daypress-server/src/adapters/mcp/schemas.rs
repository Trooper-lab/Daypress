//! MCP Tool Manifest Schemas (dapters/mcp/schemas.rs).

use serde_json::{json, Value};

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
                "description": "Import a complete HTML/CSS design document in one call: the page becomes 100% native DayStudio AST blocks (zero CustomHtml fallbacks) with the design's stylesheet in its DaystudioKit; the body-level <header> (everything before <main>) and <footer> (everything after it, e.g. a mobile call bar) become active, site-wide 'header'/'footer' theme templates titled '{site_name} header'/'{site_name} footer'; every <form> becomes a FormStudio form titled '{site_name}: {submit label}' (fields from labels, names, required, placeholders and options; checkbox groups -> checkbox_group, file inputs -> file) rendered with the design's own field markup, so submissions land in the Submissions Vault. Re-importing updates the same page, templates and forms instead of duplicating them. Scripts are not imported (tabs, carousels, script menus need native equivalents). IMPORTANT: External <img src=\"https://...\"> URLs and inline data:image/... URIs are kept verbatim in the AST and NOT auto-uploaded; after importing, always upload all page and header/footer images into the Media Library (/uploads/...) via daypress_import_wp_bundle (bundle.media), rewrite the page and template AST URLs to /uploads/<file_name>, and verify with daypress_audit_post.",
                "inputSchema": {
                    "type": "object",
                    "required": ["html"],
                    "properties": {
                        "html": { "type": "string", "description": "Full HTML5 document or HTML + <style> fragment" },
                        "post_id": { "type": "integer", "description": "Existing post/page ID to update (if omitted, matches by slug or creates a new page)" },
                        "title": { "type": "string", "description": "Optional page title override" },
                        "slug": { "type": "string", "description": "Optional page slug (defaults to 'home' when set_as_front_page is true)" },
                        "site_name": { "type": "string", "description": "Name used for the header/footer template and form titles (defaults to the last segment of the design's <title>, e.g. 'Acme')" },
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
                "description": "Execute a 1-Click WordPress migration bundle import with relational ID remapping and Elementor/Divi/Gutenberg AST transpilation, OR incrementally upload media files into the Media Library (/uploads/...) with automatic WebP/AVIF & dimension metadata by passing bundle: {\"media\": [{\"file_name\", \"mime_type\", \"title\", \"alt_text\", \"base64_data\"}]} with site_settings omitted.",
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
                "description": "Retrieve the Plausible-style Cookieless Analytics dashboard summary (pageviews, unique visitors, live visitors, engaged time, scroll depth, bounce rate, period-over-period deltas, time-series, top/entry/exit pages, channels, referrers, UTMs, devices, browsers, OS, countries, conversions, outbound clicks, and file downloads).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "range": { "type": "string", "default": "30d", "description": "Date range ('today', '24h', '7d', '30d', '90d', '12m', or 'all')" },
                        "page": { "type": "string", "description": "Optional page path filter (e.g. '/pricing')" },
                        "channel": { "type": "string", "description": "Optional acquisition channel filter ('Direct', 'Organic Search', 'Social', 'Referral', 'Email', 'Paid')" },
                        "referrer": { "type": "string", "description": "Optional normalized referrer domain filter (e.g. 'google.com')" },
                        "country": { "type": "string", "description": "Optional 2-letter ISO country filter (e.g. 'NL', 'US')" },
                        "device": { "type": "string", "description": "Optional device filter ('desktop', 'mobile', 'tablet')" },
                        "browser": { "type": "string", "description": "Optional browser family filter ('Chrome', 'Safari', 'Firefox', 'Edge')" },
                        "os": { "type": "string", "description": "Optional OS filter ('macOS', 'Windows', 'iOS', 'Android', 'Linux')" },
                        "goal": { "type": "string", "description": "Optional goal/conversion filter to inspect converting visitors" }
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
                "description": "Run the real-time WCAG 2.1 AA/AAA Contrast, Media Library Image Check (flags external hotlinks, inline data:image/ URIs, and unregistered /uploads/ assets), Broken-Link (SQLite-verified), Heading Hierarchy, and SEO Linter on a post/page by post_id or draft payload.",
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
            },
            {
                "name": "daypress_list_subscribers",
                "description": "List newsletter subscribers and summary statistics from `daypress_subscribers` (`status`, `list`).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "status": { "type": "string", "description": "'subscribed', 'pending', 'unsubscribed', or 'all'" },
                        "list": { "type": "string", "description": "Filter by mailing list name (e.g. 'general')" }
                    }
                }
            },
            {
                "name": "daypress_create_newsletter_campaign",
                "description": "Create and optionally send a newsletter email broadcast to confirmed subscribers in `target_list`.",
                "inputSchema": {
                    "type": "object",
                    "required": ["subject", "body_html"],
                    "properties": {
                        "subject": { "type": "string" },
                        "body_html": { "type": "string" },
                        "target_list": { "type": "string", "default": "general" },
                        "send_now": { "type": "boolean", "default": true }
                    }
                }
            },
            {
                "name": "daypress_manage_shortlinks",
                "description": "List, create, or delete branded `/go/{slug}` shortlinks with UTM tagging and affiliate `nofollow`/`sponsored` headers.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "enum": ["list", "create", "delete"], "default": "list" },
                        "id": { "type": "integer" },
                        "slug": { "type": "string" },
                        "target_url": { "type": "string" },
                        "title": { "type": "string" },
                        "status_code": { "type": "integer", "enum": [301, 302, 307], "default": 307 },
                        "nofollow": { "type": "boolean", "default": true },
                        "sponsored": { "type": "boolean", "default": false },
                        "utm_source": { "type": "string" },
                        "utm_medium": { "type": "string" },
                        "utm_campaign": { "type": "string" }
                    }
                }
            },
            {
                "name": "daypress_manage_webhooks",
                "description": "List, create, delete, or replay HMAC-SHA256 signed outbound webhooks (`post.published`, `form.submitted`, `subscriber.confirmed`).",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "enum": ["list", "create", "delete", "replay"], "default": "list" },
                        "id": { "type": "integer" },
                        "delivery_id": { "type": "integer" },
                        "name": { "type": "string" },
                        "target_url": { "type": "string" },
                        "preset": { "type": "string", "enum": ["generic", "slack", "discord", "hubspot", "zapier"], "default": "generic" },
                        "events": { "type": "array", "items": { "type": "string" } }
                    }
                }
            },
            {
                "name": "daypress_manage_ab_tests",
                "description": "List, create, delete, or promote the winner of a zero-flicker server-side A/B split experiment.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "enum": ["list", "create", "promote", "delete"], "default": "list" },
                        "id": { "type": "integer" },
                        "name": { "type": "string" },
                        "post_id": { "type": "integer" },
                        "control_text": { "type": "string" },
                        "variant_text": { "type": "string" },
                        "goal_event": { "type": "string", "default": "conversion" },
                        "traffic_split": { "type": "integer", "default": 50 },
                        "winner": { "type": "string", "enum": ["A", "B"] }
                    }
                }
            },
            {
                "name": "daypress_get_web_vitals",
                "description": "Read the real-user Core Web Vitals (`LCP`, `INP`, `CLS`, `TTFB`) p75 summary.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "daypress_manage_wasm_extensions",
                "description": "List, install/compile, execute, or delete sandboxed WebAssembly (`.wasm`) extensions with fuel metering and capability permissions.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "action": { "type": "string", "enum": ["list", "install", "execute", "delete"], "default": "list" },
                        "slug": { "type": "string" },
                        "name": { "type": "string" },
                        "shortcode": { "type": "string" },
                        "wasm_base64": { "type": "string" },
                        "template_source": { "type": "string" },
                        "fuel_limit": { "type": "integer", "default": 50000 },
                        "max_memory_pages": { "type": "integer", "default": 16 },
                        "capabilities": { "type": "object" },
                        "attrs": { "type": "object" }
                    }
                }
            },
            {
                "name": "daypress_security_audit",
                "description": "Run the Daypress 1.0 Security & Production Readiness Audit, SQLite Schema v100 Contract check, Wasm Host ABI v1 check, and 70%+ WordPress Migration Corpus Sign-Off.",
                "inputSchema": {
                    "type": "object",
                    "properties": {}
                }
            },
            {
                "name": "daypress_run_benchmark",
                "description": "Run the reproducible Daypress 1.0 Performance, Latency (p50/p95/p99 µs) & Memory Benchmark Suite across SSR Hot Cache, Cold AST Compile, SQLite FTS5 BM25 Search, REST API v2, and Wasm Sandbox.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "iterations": {
                            "type": "integer",
                            "default": 25,
                            "description": "Number of timed iterations per workload (5..=500)."
                        }
                    }
                }
            }
        ]
    })
}
