//! Core SQLite Database DDL statements for Daypress CMS.

pub const CORE_SCHEMA_V0_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS posts (
    ID INTEGER PRIMARY KEY AUTOINCREMENT,
    post_author INTEGER NOT NULL DEFAULT 1,
    post_date TEXT NOT NULL,
    post_date_gmt TEXT NOT NULL,
    post_content TEXT NOT NULL DEFAULT '',
    post_title TEXT NOT NULL DEFAULT '',
    post_excerpt TEXT NOT NULL DEFAULT '',
    post_status TEXT NOT NULL DEFAULT 'publish',
    comment_status TEXT NOT NULL DEFAULT 'open',
    ping_status TEXT NOT NULL DEFAULT 'open',
    post_password TEXT NOT NULL DEFAULT '',
    post_name TEXT NOT NULL DEFAULT '',
    to_ping TEXT NOT NULL DEFAULT '',
    pinged TEXT NOT NULL DEFAULT '',
    post_modified TEXT NOT NULL,
    post_modified_gmt TEXT NOT NULL,
    post_content_filtered TEXT NOT NULL DEFAULT '',
    post_parent INTEGER NOT NULL DEFAULT 0,
    guid TEXT NOT NULL DEFAULT '',
    menu_order INTEGER NOT NULL DEFAULT 0,
    post_type TEXT NOT NULL DEFAULT 'post',
    post_mime_type TEXT NOT NULL DEFAULT '',
    comment_count INTEGER NOT NULL DEFAULT 0,
    is_sticky INTEGER NOT NULL DEFAULT 0,
    layout_json TEXT DEFAULT NULL
);

CREATE INDEX IF NOT EXISTS idx_post_name ON posts(post_name);
CREATE INDEX IF NOT EXISTS idx_type_status_date ON posts(post_type, post_status, post_date, ID);

CREATE TABLE IF NOT EXISTS postmeta (
    meta_id INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id INTEGER NOT NULL DEFAULT 0,
    meta_key TEXT DEFAULT NULL,
    meta_value TEXT DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS options (
    option_id INTEGER PRIMARY KEY AUTOINCREMENT,
    option_name TEXT NOT NULL UNIQUE,
    option_value TEXT NOT NULL,
    autoload TEXT NOT NULL DEFAULT 'yes'
);

CREATE TABLE IF NOT EXISTS users (
    ID INTEGER PRIMARY KEY AUTOINCREMENT,
    user_login TEXT NOT NULL UNIQUE,
    user_pass TEXT NOT NULL,
    user_nicename TEXT NOT NULL DEFAULT '',
    user_email TEXT NOT NULL DEFAULT '',
    user_url TEXT NOT NULL DEFAULT '',
    user_registered TEXT NOT NULL,
    user_activation_key TEXT NOT NULL DEFAULT '',
    user_status INTEGER NOT NULL DEFAULT 0,
    display_name TEXT NOT NULL DEFAULT ''
);

CREATE TABLE IF NOT EXISTS terms (
    term_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    term_group INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS term_taxonomy (
    term_taxonomy_id INTEGER PRIMARY KEY AUTOINCREMENT,
    term_id INTEGER NOT NULL,
    taxonomy TEXT NOT NULL DEFAULT 'category',
    description TEXT NOT NULL DEFAULT '',
    parent INTEGER NOT NULL DEFAULT 0,
    count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS term_relationships (
    object_id INTEGER NOT NULL,
    term_taxonomy_id INTEGER NOT NULL,
    term_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (object_id, term_taxonomy_id)
);

CREATE TABLE IF NOT EXISTS comments (
    comment_ID INTEGER PRIMARY KEY AUTOINCREMENT,
    comment_post_ID INTEGER NOT NULL DEFAULT 0,
    comment_author TEXT NOT NULL,
    comment_author_email TEXT NOT NULL DEFAULT '',
    comment_author_url TEXT NOT NULL DEFAULT '',
    comment_author_IP TEXT NOT NULL DEFAULT '',
    comment_date TEXT NOT NULL,
    comment_date_gmt TEXT NOT NULL,
    comment_content TEXT NOT NULL,
    comment_karma INTEGER NOT NULL DEFAULT 0,
    comment_approved TEXT NOT NULL DEFAULT '1',
    comment_agent TEXT NOT NULL DEFAULT '',
    comment_type TEXT NOT NULL DEFAULT 'comment',
    comment_parent INTEGER NOT NULL DEFAULT 0,
    user_id INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS application_passwords (
    uuid TEXT PRIMARY KEY,
    app_id TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    app_name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_used TEXT DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS usermeta (
    umeta_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL DEFAULT 0,
    meta_key TEXT DEFAULT NULL,
    meta_value TEXT DEFAULT NULL
);
CREATE INDEX IF NOT EXISTS idx_user_id ON usermeta(user_id);
PRAGMA user_version = 1;
"#;

pub const SUBSYSTEMS_SCHEMA_SQL: &str = r#"
-- Lets the scheduled-post promotion (`scheduling.rs`) find due `future` posts cheaply.
CREATE INDEX IF NOT EXISTS idx_post_status_date ON posts(post_status, post_date);

CREATE TABLE IF NOT EXISTS application_passwords (
    uuid TEXT PRIMARY KEY,
    app_id TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    app_name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_used TEXT DEFAULT NULL
);
CREATE TABLE IF NOT EXISTS usermeta (
    umeta_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL DEFAULT 0,
    meta_key TEXT DEFAULT NULL,
    meta_value TEXT DEFAULT NULL
);
CREATE INDEX IF NOT EXISTS idx_user_id ON usermeta(user_id);

-- WordPress comment meta (used for `_wp_trash_meta_status` so Restore / Not Spam
-- return a comment to its previous moderation status, exactly like WP core).
CREATE TABLE IF NOT EXISTS commentmeta (
    meta_id INTEGER PRIMARY KEY AUTOINCREMENT,
    comment_id INTEGER NOT NULL DEFAULT 0,
    meta_key TEXT DEFAULT NULL,
    meta_value TEXT DEFAULT NULL
);
CREATE INDEX IF NOT EXISTS idx_commentmeta_comment_id ON commentmeta(comment_id);

-- Native Subsystems: Transactional Email Queue & Delivery Logs
CREATE TABLE IF NOT EXISTS daypress_email_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipient TEXT NOT NULL,
    subject TEXT NOT NULL,
    transport TEXT NOT NULL DEFAULT 'smtp',
    status TEXT NOT NULL DEFAULT 'delivered',
    latency_ms INTEGER NOT NULL DEFAULT 0,
    error_message TEXT DEFAULT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS daypress_email_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipient TEXT NOT NULL,
    subject TEXT NOT NULL,
    body_html TEXT NOT NULL DEFAULT '',
    body_text TEXT NOT NULL DEFAULT '',
    headers_json TEXT NOT NULL DEFAULT '{}',
    attempts INTEGER NOT NULL DEFAULT 0,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TEXT NOT NULL
);

-- Native Subsystems: Declarative Forms & Submissions
CREATE TABLE IF NOT EXISTS daypress_forms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    schema_json TEXT NOT NULL DEFAULT '{}',
    settings_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS daypress_form_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    form_id INTEGER NOT NULL,
    data_json TEXT NOT NULL DEFAULT '{}',
    attachments_json TEXT NOT NULL DEFAULT '[]',
    ip_hash TEXT NOT NULL DEFAULT '',
    user_agent TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'unread',
    is_starred INTEGER NOT NULL DEFAULT 0,
    email_status TEXT NOT NULL DEFAULT 'disabled',
    email_error TEXT DEFAULT NULL,
    webhook_status TEXT DEFAULT NULL,
    source_url TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);

-- Native Subsystems: Cookieless Daily Salted Analytics
CREATE TABLE IF NOT EXISTS daypress_analytics_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    visitor_hash TEXT NOT NULL,
    event_type TEXT NOT NULL DEFAULT 'pageview',
    goal_name TEXT DEFAULT NULL,
    page_url TEXT NOT NULL,
    referrer TEXT NOT NULL DEFAULT '',
    device_type TEXT NOT NULL DEFAULT 'desktop',
    country TEXT NOT NULL DEFAULT 'XX',
    utm_source TEXT DEFAULT NULL,
    utm_medium TEXT DEFAULT NULL,
    utm_campaign TEXT DEFAULT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS daypress_analytics_daily (
    date TEXT NOT NULL,
    page_url TEXT NOT NULL,
    unique_visitors INTEGER NOT NULL DEFAULT 0,
    pageviews INTEGER NOT NULL DEFAULT 0,
    bounces INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (date, page_url)
);

-- Native Subsystems: SEO Redirects, 404 Monitor & Plugin Ecosystem Registry
CREATE TABLE IF NOT EXISTS daypress_redirects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_path TEXT NOT NULL UNIQUE,
    target_url TEXT NOT NULL,
    status_code INTEGER NOT NULL DEFAULT 301,
    hits INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_redirects_source ON daypress_redirects(source_path);

CREATE TABLE IF NOT EXISTS daypress_404_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_path TEXT NOT NULL UNIQUE,
    referrer TEXT NOT NULL DEFAULT '',
    user_agent TEXT NOT NULL DEFAULT '',
    hits INTEGER NOT NULL DEFAULT 1,
    last_seen_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS daypress_plugins (
    slug TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL DEFAULT '1.0.0',
    description TEXT NOT NULL DEFAULT '',
    category TEXT NOT NULL DEFAULT 'general',
    is_builtin INTEGER NOT NULL DEFAULT 1,
    is_active INTEGER NOT NULL DEFAULT 1,
    manifest_json TEXT NOT NULL DEFAULT '{}',
    config_json TEXT NOT NULL DEFAULT '{}',
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Native Subsystems: Static Edge Publishing & Deployment Ledger
CREATE TABLE IF NOT EXISTS daypress_deployments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    provider TEXT NOT NULL DEFAULT 'zip',
    target_url TEXT NOT NULL DEFAULT '',
    deployment_external_id TEXT DEFAULT NULL,
    status TEXT NOT NULL DEFAULT 'live',
    commit_message TEXT NOT NULL DEFAULT '',
    pages_count INTEGER NOT NULL DEFAULT 0,
    assets_count INTEGER NOT NULL DEFAULT 0,
    total_bytes INTEGER NOT NULL DEFAULT 0,
    manifest_json TEXT NOT NULL DEFAULT '{}',
    bundle_path TEXT DEFAULT NULL,
    error_message TEXT DEFAULT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    duration_ms INTEGER NOT NULL DEFAULT 0
);

-- Native Subsystems: Media Library DAM Virtual Folders
CREATE TABLE IF NOT EXISTS daypress_media_folders (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    parent_id INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_media_folders_parent ON daypress_media_folders(parent_id);

-- Native Subsystems: Content Model Registry (Custom Post Types, Taxonomies & Structured Field Groups)
CREATE TABLE IF NOT EXISTS daypress_post_types (
    slug TEXT PRIMARY KEY,
    label_singular TEXT NOT NULL,
    label_plural TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    icon TEXT NOT NULL DEFAULT 'fa-solid fa-folder-open',
    menu_position INTEGER NOT NULL DEFAULT 25,
    public INTEGER NOT NULL DEFAULT 1,
    has_archive INTEGER NOT NULL DEFAULT 1,
    hierarchical INTEGER NOT NULL DEFAULT 0,
    rewrite_slug TEXT NOT NULL DEFAULT '',
    supports_json TEXT NOT NULL DEFAULT '["title","editor","thumbnail","excerpt","custom-fields"]',
    taxonomies_json TEXT NOT NULL DEFAULT '[]',
    show_in_rest INTEGER NOT NULL DEFAULT 1,
    preset_key TEXT DEFAULT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS daypress_taxonomies (
    slug TEXT PRIMARY KEY,
    label_singular TEXT NOT NULL,
    label_plural TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    hierarchical INTEGER NOT NULL DEFAULT 1,
    public INTEGER NOT NULL DEFAULT 1,
    rewrite_slug TEXT NOT NULL DEFAULT '',
    post_types_json TEXT NOT NULL DEFAULT '[]',
    show_in_rest INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS daypress_field_groups (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    group_key TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    slug TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    location_rules_json TEXT NOT NULL DEFAULT '[]',
    fields_json TEXT NOT NULL DEFAULT '[]',
    position TEXT NOT NULL DEFAULT 'normal',
    menu_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_field_groups_active ON daypress_field_groups(is_active, menu_order);
"#;

/// Single-use "Lost your password?" links. Only the SHA-256 of each token is stored, so a
/// copy of the database cannot be turned into working reset links. Times are Unix seconds.
pub const PASSWORD_RESETS_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS daypress_password_resets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL REFERENCES users(ID) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    used_at INTEGER DEFAULT NULL
);
CREATE INDEX IF NOT EXISTS idx_password_resets_user ON daypress_password_resets(user_id, created_at);
"#;
