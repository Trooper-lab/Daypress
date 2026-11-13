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
    referrer_domain TEXT NOT NULL DEFAULT '',
    channel TEXT NOT NULL DEFAULT 'Direct',
    device_type TEXT NOT NULL DEFAULT 'desktop',
    browser TEXT NOT NULL DEFAULT '',
    os TEXT NOT NULL DEFAULT '',
    country TEXT NOT NULL DEFAULT 'XX',
    utm_source TEXT DEFAULT NULL,
    utm_medium TEXT DEFAULT NULL,
    utm_campaign TEXT DEFAULT NULL,
    duration_secs INTEGER NOT NULL DEFAULT 0,
    scroll_depth INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS daypress_analytics_daily (
    date TEXT NOT NULL,
    page_url TEXT NOT NULL,
    unique_visitors INTEGER NOT NULL DEFAULT 0,
    pageviews INTEGER NOT NULL DEFAULT 0,
    bounces INTEGER NOT NULL DEFAULT 0,
    conversions INTEGER NOT NULL DEFAULT 0,
    total_duration_secs INTEGER NOT NULL DEFAULT 0,
    scroll_sum INTEGER NOT NULL DEFAULT 0,
    engaged_views INTEGER NOT NULL DEFAULT 0,
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
CREATE INDEX IF NOT EXISTS idx_daypress_plugins_active ON daypress_plugins(is_active, slug);

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

/// v0.7 Native SQLite FTS5 Full-Text Search Virtual Table & Automatic Sync Triggers.
pub const FTS5_SEARCH_SCHEMA_SQL: &str = r#"
CREATE VIRTUAL TABLE IF NOT EXISTS posts_fts USING fts5(
    post_title,
    post_excerpt,
    post_content,
    post_name,
    post_type UNINDEXED,
    post_status UNINDEXED,
    tokenize='unicode61 remove_diacritics 2'
);

CREATE TRIGGER IF NOT EXISTS trg_posts_fts_ai AFTER INSERT ON posts
WHEN new.post_type NOT IN ('revision', 'nav_menu_item', 'attachment')
BEGIN
    DELETE FROM posts_fts WHERE rowid = new.ID;
    INSERT INTO posts_fts(rowid, post_title, post_excerpt, post_content, post_name, post_type, post_status)
    VALUES (new.ID, new.post_title, new.post_excerpt, new.post_content, new.post_name, new.post_type, new.post_status);
END;

CREATE TRIGGER IF NOT EXISTS trg_posts_fts_au AFTER UPDATE ON posts
BEGIN
    DELETE FROM posts_fts WHERE rowid = old.ID;
    INSERT INTO posts_fts(rowid, post_title, post_excerpt, post_content, post_name, post_type, post_status)
    SELECT new.ID, new.post_title, new.post_excerpt, new.post_content, new.post_name, new.post_type, new.post_status
    WHERE new.post_type NOT IN ('revision', 'nav_menu_item', 'attachment');
END;

CREATE TRIGGER IF NOT EXISTS trg_posts_fts_ad AFTER DELETE ON posts
BEGIN
    DELETE FROM posts_fts WHERE rowid = old.ID;
END;
"#;

/// v0.8 Multilingual (i18n) & Editorial Workflow Tables (Languages, Translation Groups, Co-Authors, Secret Preview Links).
pub const V08_I18N_AND_EDITORIAL_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS daypress_languages (
    code TEXT PRIMARY KEY,
    locale TEXT NOT NULL,
    name TEXT NOT NULL,
    flag_emoji TEXT NOT NULL DEFAULT '🌐',
    is_default INTEGER NOT NULL DEFAULT 0,
    is_rtl INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0
);

INSERT OR IGNORE INTO daypress_languages (code, locale, name, flag_emoji, is_default, is_rtl, sort_order)
VALUES ('en', 'en_US', 'English', '🇺🇸', 1, 0, 0);

CREATE TABLE IF NOT EXISTS daypress_translations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    object_type TEXT NOT NULL DEFAULT 'post',
    object_id INTEGER NOT NULL,
    lang_code TEXT NOT NULL,
    translation_group_id TEXT NOT NULL,
    UNIQUE(object_type, object_id)
);
CREATE INDEX IF NOT EXISTS idx_translations_group ON daypress_translations(object_type, translation_group_id);
CREATE INDEX IF NOT EXISTS idx_translations_lang ON daypress_translations(object_type, lang_code, object_id);

CREATE TABLE IF NOT EXISTS daypress_post_authors (
    post_id INTEGER NOT NULL,
    user_id INTEGER NOT NULL,
    author_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (post_id, user_id)
);
CREATE INDEX IF NOT EXISTS idx_post_authors_user ON daypress_post_authors(user_id, author_order);

CREATE TABLE IF NOT EXISTS daypress_preview_tokens (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    post_id INTEGER NOT NULL,
    token_hash TEXT NOT NULL UNIQUE,
    token_preview TEXT NOT NULL DEFAULT '',
    label TEXT NOT NULL DEFAULT '',
    created_by INTEGER NOT NULL DEFAULT 1,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL,
    revoked_at INTEGER DEFAULT NULL
);
CREATE INDEX IF NOT EXISTS idx_preview_tokens_post ON daypress_preview_tokens(post_id, expires_at);
"#;

/// v0.9 Pillar 1: Built-in Newsletters, Member-Gated Posts & Automated Email Digests (`MailPoet` / `Ghost` parity).
pub const V09_NEWSLETTERS_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS daypress_subscribers (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    email TEXT NOT NULL UNIQUE COLLATE NOCASE,
    name TEXT NOT NULL DEFAULT '',
    status TEXT NOT NULL DEFAULT 'pending',
    lists_json TEXT NOT NULL DEFAULT '["general"]',
    confirm_token TEXT NOT NULL UNIQUE,
    unsubscribe_token TEXT NOT NULL UNIQUE,
    source_url TEXT NOT NULL DEFAULT '',
    ip_hash TEXT NOT NULL DEFAULT '',
    confirmed_at TEXT DEFAULT NULL,
    unsubscribed_at TEXT DEFAULT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_subscribers_status ON daypress_subscribers(status, created_at);

CREATE TABLE IF NOT EXISTS daypress_newsletter_campaigns (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    subject TEXT NOT NULL,
    body_html TEXT NOT NULL,
    target_list TEXT NOT NULL DEFAULT 'general',
    campaign_type TEXT NOT NULL DEFAULT 'broadcast',
    post_id INTEGER DEFAULT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    recipient_count INTEGER NOT NULL DEFAULT 0,
    sent_count INTEGER NOT NULL DEFAULT 0,
    failed_count INTEGER NOT NULL DEFAULT 0,
    sent_at TEXT DEFAULT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_newsletter_campaigns_post ON daypress_newsletter_campaigns(post_id, campaign_type);
"#;

/// v0.9 Pillar 2: Branded Shortlinks (`/go/*`) & HMAC-Signed Outbound Webhooks (`Pretty Links` / `WP Webhooks` parity).
pub const V09_WEBHOOKS_AND_SHORTLINKS_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS daypress_shortlinks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE COLLATE NOCASE,
    target_url TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    status_code INTEGER NOT NULL DEFAULT 302,
    utm_source TEXT NOT NULL DEFAULT '',
    utm_medium TEXT NOT NULL DEFAULT '',
    utm_campaign TEXT NOT NULL DEFAULT '',
    clicks INTEGER NOT NULL DEFAULT 0,
    last_clicked_at TEXT DEFAULT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_shortlinks_slug ON daypress_shortlinks(slug, is_active);

CREATE TABLE IF NOT EXISTS daypress_webhooks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    target_url TEXT NOT NULL,
    preset TEXT NOT NULL DEFAULT 'generic',
    events_json TEXT NOT NULL DEFAULT '["post.published","form.submitted"]',
    secret TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS daypress_webhook_deliveries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    webhook_id INTEGER NOT NULL,
    webhook_name TEXT NOT NULL DEFAULT '',
    event_name TEXT NOT NULL,
    target_url TEXT NOT NULL,
    request_headers_json TEXT NOT NULL DEFAULT '{}',
    payload_json TEXT NOT NULL,
    signature_header TEXT NOT NULL DEFAULT '',
    response_status INTEGER NOT NULL DEFAULT 0,
    response_body TEXT NOT NULL DEFAULT '',
    latency_ms INTEGER NOT NULL DEFAULT 0,
    attempt_count INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL DEFAULT 'delivered',
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_hook ON daypress_webhook_deliveries(webhook_id, created_at);
CREATE INDEX IF NOT EXISTS idx_webhook_deliveries_wh_id_desc ON daypress_webhook_deliveries(webhook_id, id DESC);
"#;

/// v0.9 Pillar 3: Zero-Flicker Server-Side A/B Split Testing & Real-User Core Web Vitals (`LCP` / `INP` / `CLS`).
pub const V09_AB_AND_WEB_VITALS_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS daypress_ab_tests (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    post_id INTEGER NOT NULL,
    target_selector TEXT NOT NULL DEFAULT '',
    control_text TEXT NOT NULL DEFAULT '',
    variant_text TEXT NOT NULL DEFAULT '',
    goal_event TEXT NOT NULL DEFAULT 'conversion',
    traffic_split INTEGER NOT NULL DEFAULT 50,
    status TEXT NOT NULL DEFAULT 'running',
    control_views INTEGER NOT NULL DEFAULT 0,
    control_conversions INTEGER NOT NULL DEFAULT 0,
    variant_views INTEGER NOT NULL DEFAULT 0,
    variant_conversions INTEGER NOT NULL DEFAULT 0,
    winner TEXT DEFAULT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_ab_tests_post_status ON daypress_ab_tests(post_id, status);

CREATE TABLE IF NOT EXISTS daypress_ab_exposures (
    test_id INTEGER NOT NULL,
    visitor_hash TEXT NOT NULL,
    variant TEXT NOT NULL,
    converted INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    PRIMARY KEY (test_id, visitor_hash)
);
CREATE INDEX IF NOT EXISTS idx_ab_exposures_visitor ON daypress_ab_exposures(visitor_hash, converted);

CREATE TABLE IF NOT EXISTS daypress_web_vitals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    page_url TEXT NOT NULL,
    device_type TEXT NOT NULL DEFAULT 'desktop',
    lcp_ms REAL DEFAULT NULL,
    inp_ms REAL DEFAULT NULL,
    cls_score REAL DEFAULT NULL,
    ttfb_ms REAL DEFAULT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_web_vitals_created ON daypress_web_vitals(created_at, page_url);
"#;

/// v0.9 Pillar 4: Sandboxed WebAssembly (`.wasm`) Extension Crates (`daypress_wasm_extensions`, `daypress_wasm_kv`).
pub const V09_WASM_EXTENSIONS_SCHEMA_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS daypress_wasm_extensions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    slug TEXT NOT NULL UNIQUE COLLATE NOCASE,
    name TEXT NOT NULL,
    version TEXT NOT NULL DEFAULT '1.0.0',
    description TEXT NOT NULL DEFAULT '',
    shortcode TEXT NOT NULL DEFAULT '',
    capabilities_json TEXT NOT NULL DEFAULT '{"shortcodes":true}',
    fuel_limit INTEGER NOT NULL DEFAULT 50000,
    max_memory_pages INTEGER NOT NULL DEFAULT 16,
    wasm_sha256 TEXT NOT NULL,
    wasm_bytes BLOB NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1,
    executions INTEGER NOT NULL DEFAULT 0,
    last_error TEXT DEFAULT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_wasm_extensions_active ON daypress_wasm_extensions(is_active, shortcode);

CREATE TABLE IF NOT EXISTS daypress_wasm_kv (
    extension_slug TEXT NOT NULL COLLATE NOCASE,
    kv_key TEXT NOT NULL,
    kv_value TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (extension_slug, kv_key)
);
"#;

/// Canonical SQLite `PRAGMA user_version` for Daypress 1.0 (`v1.0.0-beta.1` and `1.x` series).
///
/// All `1.x` releases guarantee non-breaking additive migrations from `DAYPRESS_SCHEMA_VERSION = 100`.
pub const DAYPRESS_SCHEMA_VERSION: i32 = 100;

/// v1.0 Stable Schema Migration Ledger (`daypress_schema_migrations`) & `PRAGMA user_version = 100`.
pub const V10_SCHEMA_MIGRATIONS_SQL: &str = r#"
CREATE TABLE IF NOT EXISTS daypress_schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    applied_at TEXT NOT NULL DEFAULT (datetime('now'))
);

INSERT OR IGNORE INTO daypress_schema_migrations (version, name) VALUES
    (1,   'v0.1_core_wp_tables_and_ast'),
    (40,  'v0.4_formstudio_analytics_and_redirects'),
    (51,  'v0.5_theme_templates_symbols_and_media_dam'),
    (60,  'v0.6_custom_post_types_and_acf_field_groups'),
    (70,  'v0.7_sqlite_fts5_bm25_search_and_triggers'),
    (80,  'v0.8_multilingual_i18n_and_editorial_preview_tokens'),
    (90,  'v0.9_newsletters_webhooks_ab_vitals_and_wasm'),
    (100, 'v1.0_stable_schema_contract_lock');

PRAGMA user_version = 100;
"#;
