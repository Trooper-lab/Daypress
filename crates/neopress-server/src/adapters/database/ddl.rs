//! Core SQLite Database DDL statements for Neopress CMS.

pub const CORE_SCHEMA_V0_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS wp_posts (
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

CREATE INDEX IF NOT EXISTS idx_post_name ON wp_posts(post_name);
CREATE INDEX IF NOT EXISTS idx_type_status_date ON wp_posts(post_type, post_status, post_date, ID);

CREATE TABLE IF NOT EXISTS wp_postmeta (
    meta_id INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id INTEGER NOT NULL DEFAULT 0,
    meta_key TEXT DEFAULT NULL,
    meta_value TEXT DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS wp_options (
    option_id INTEGER PRIMARY KEY AUTOINCREMENT,
    option_name TEXT NOT NULL UNIQUE,
    option_value TEXT NOT NULL,
    autoload TEXT NOT NULL DEFAULT 'yes'
);

CREATE TABLE IF NOT EXISTS wp_users (
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

CREATE TABLE IF NOT EXISTS wp_terms (
    term_id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    slug TEXT NOT NULL UNIQUE,
    term_group INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS wp_term_taxonomy (
    term_taxonomy_id INTEGER PRIMARY KEY AUTOINCREMENT,
    term_id INTEGER NOT NULL,
    taxonomy TEXT NOT NULL DEFAULT 'category',
    description TEXT NOT NULL DEFAULT '',
    parent INTEGER NOT NULL DEFAULT 0,
    count INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS wp_term_relationships (
    object_id INTEGER NOT NULL,
    term_taxonomy_id INTEGER NOT NULL,
    term_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (object_id, term_taxonomy_id)
);

CREATE TABLE IF NOT EXISTS wp_comments (
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

CREATE TABLE IF NOT EXISTS wp_application_passwords (
    uuid TEXT PRIMARY KEY,
    app_id TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    app_name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_used TEXT DEFAULT NULL
);

CREATE TABLE IF NOT EXISTS wp_usermeta (
    umeta_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL DEFAULT 0,
    meta_key TEXT DEFAULT NULL,
    meta_value TEXT DEFAULT NULL
);
CREATE INDEX IF NOT EXISTS idx_user_id ON wp_usermeta(user_id);
PRAGMA user_version = 1;
"#;

pub const SUBSYSTEMS_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS wp_application_passwords (
    uuid TEXT PRIMARY KEY,
    app_id TEXT NOT NULL,
    user_id INTEGER NOT NULL,
    app_name TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    last_used TEXT DEFAULT NULL
);
CREATE TABLE IF NOT EXISTS wp_usermeta (
    umeta_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL DEFAULT 0,
    meta_key TEXT DEFAULT NULL,
    meta_value TEXT DEFAULT NULL
);
CREATE INDEX IF NOT EXISTS idx_user_id ON wp_usermeta(user_id);

-- WordPress comment meta (used for `_wp_trash_meta_status` so Restore / Not Spam
-- return a comment to its previous moderation status, exactly like WP core).
CREATE TABLE IF NOT EXISTS wp_commentmeta (
    meta_id INTEGER PRIMARY KEY AUTOINCREMENT,
    comment_id INTEGER NOT NULL DEFAULT 0,
    meta_key TEXT DEFAULT NULL,
    meta_value TEXT DEFAULT NULL
);
CREATE INDEX IF NOT EXISTS idx_commentmeta_comment_id ON wp_commentmeta(comment_id);

-- Native Subsystems: Transactional Email Queue & Delivery Logs
CREATE TABLE IF NOT EXISTS wp_neopress_email_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipient TEXT NOT NULL,
    subject TEXT NOT NULL,
    transport TEXT NOT NULL DEFAULT 'smtp',
    status TEXT NOT NULL DEFAULT 'delivered',
    latency_ms INTEGER NOT NULL DEFAULT 0,
    error_message TEXT DEFAULT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS wp_neopress_email_queue (
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
CREATE TABLE IF NOT EXISTS wp_neopress_forms (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    title TEXT NOT NULL,
    schema_json TEXT NOT NULL DEFAULT '{}',
    settings_json TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS wp_neopress_form_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    form_id INTEGER NOT NULL,
    data_json TEXT NOT NULL DEFAULT '{}',
    ip_hash TEXT NOT NULL DEFAULT '',
    user_agent TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL
);

-- Native Subsystems: Cookieless Daily Salted Analytics
CREATE TABLE IF NOT EXISTS wp_neopress_analytics_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    visitor_hash TEXT NOT NULL,
    page_url TEXT NOT NULL,
    referrer TEXT NOT NULL DEFAULT '',
    device_type TEXT NOT NULL DEFAULT 'desktop',
    country TEXT NOT NULL DEFAULT 'XX',
    utm_source TEXT DEFAULT NULL,
    utm_medium TEXT DEFAULT NULL,
    utm_campaign TEXT DEFAULT NULL,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS wp_neopress_analytics_daily (
    date TEXT NOT NULL,
    page_url TEXT NOT NULL,
    unique_visitors INTEGER NOT NULL DEFAULT 0,
    pageviews INTEGER NOT NULL DEFAULT 0,
    bounces INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (date, page_url)
);

-- Native Subsystems: SEO Redirects, 404 Monitor & Plugin Ecosystem Registry
CREATE TABLE IF NOT EXISTS wp_neopress_redirects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_path TEXT NOT NULL UNIQUE,
    target_url TEXT NOT NULL,
    status_code INTEGER NOT NULL DEFAULT 301,
    hits INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX IF NOT EXISTS idx_redirects_source ON wp_neopress_redirects(source_path);

CREATE TABLE IF NOT EXISTS wp_neopress_404_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    request_path TEXT NOT NULL UNIQUE,
    referrer TEXT NOT NULL DEFAULT '',
    user_agent TEXT NOT NULL DEFAULT '',
    hits INTEGER NOT NULL DEFAULT 1,
    last_seen_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS wp_neopress_plugins (
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
"#;
