use crate::db::Database;
use crate::models::{
    DaypressPostResponse, DaypressSettingsResponse, PluginHooksManifest, PluginManifest,
    PluginSettingField,
};
use std::collections::HashMap;

/// Returns the catalog of built-in Daypress first-party plugins.
pub fn builtin_plugins_catalog() -> Vec<PluginManifest> {
    vec![
        PluginManifest {
            slug: "daypress-seo".to_string(),
            name: "Search, Schema & Redirection Studio".to_string(),
            version: "2.5.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Per-post SEO title, meta description & focus-keyphrase analyzer, Schema.org JSON-LD knowledge graph, OpenGraph social cards, 301/302 Redirect Manager, and 404 Monitor.".to_string(),
            settings_url: Some("/d-admin/seo".to_string()),
            is_builtin: true,
            // Global SEO settings live in `options` and are edited on /d-admin/seo
            // (see `Database::save_global_seo_settings`), so no duplicate plugin-config schema here.
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "daypress-forms".to_string(),
            name: "FormStudio & Submissions Vault".to_string(),
            version: "3.0.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Multi-step wizard forms, multi-column layouts, conditional field visibility, calculated fields, visitor file uploads, honeypot/Turnstile anti-spam, and a fail-safe SQLite Submissions Vault with 1-click email retry.".to_string(),
            settings_url: Some("/d-admin/forms".to_string()),
            is_builtin: true,
            // Per-form settings (notifications, honeypot, webhooks) are stored on each form
            // in `daypress_forms.settings_json` and edited on /d-admin/forms.
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "daypress-cookie-consent".to_string(),
            name: "Privacy & Consent Banner".to_string(),
            version: "1.0.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Zero-bloat (<2 KB) GDPR/CCPA & ePrivacy consent banner with granular script category gating (Necessary, Analytics, Marketing) configured in Settings → Privacy.".to_string(),
            settings_url: Some("/d-admin/options-privacy".to_string()),
            is_builtin: true,
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "daypress-backups".to_string(),
            name: "Hot Snapshots & Point-in-Time Backups".to_string(),
            version: "1.8.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Zero-downtime SQLite VACUUM INTO hot snapshots, 1-click point-in-time restore, and full JSON/WXR portable archive downloads.".to_string(),
            settings_url: Some("/d-admin/backups".to_string()),
            is_builtin: true,
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "whatsapp-chat".to_string(),
            name: "Direct Chat & WhatsApp Launcher".to_string(),
            version: "1.5.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Zero-JS floating click-to-chat launcher with customizable phone number, pre-filled greeting, CTA tooltip badge, and corner placement.".to_string(),
            settings_url: Some("/d-admin/plugin-settings?plugin=whatsapp-chat".to_string()),
            is_builtin: true,
            settings_schema: vec![
                PluginSettingField {
                    key: "phone_number".to_string(),
                    label: "WhatsApp Phone Number (with country code)".to_string(),
                    field_type: "text".to_string(),
                    default_value: "+15550192834".to_string(),
                    description: "Digits and country code, e.g. +15550192834 or 31612345678.".to_string(),
                    options: vec![],
                },
                PluginSettingField {
                    key: "welcome_message".to_string(),
                    label: "Pre-filled Greeting Message".to_string(),
                    field_type: "text".to_string(),
                    default_value: "Hi! I have a question about {{page_title}}".to_string(),
                    description: "Supports {{page_title}} and {{site_name}} placeholders.".to_string(),
                    options: vec![],
                },
                PluginSettingField {
                    key: "tooltip_text".to_string(),
                    label: "Tooltip Call-to-Action Text".to_string(),
                    field_type: "text".to_string(),
                    default_value: "Chat with us on WhatsApp".to_string(),
                    description: "Displayed next to the floating WhatsApp icon.".to_string(),
                    options: vec![],
                },
                PluginSettingField {
                    key: "position".to_string(),
                    label: "Screen Position".to_string(),
                    field_type: "select".to_string(),
                    default_value: "bottom-right".to_string(),
                    description: "Corner placement on public pages.".to_string(),
                    options: vec!["bottom-right".to_string(), "bottom-left".to_string()],
                },
                PluginSettingField {
                    key: "accent_color".to_string(),
                    label: "Button Accent Color".to_string(),
                    field_type: "color".to_string(),
                    default_value: "#25D366".to_string(),
                    description: "Brand color for the floating button.".to_string(),
                    options: vec![],
                },
            ],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "code-snippets".to_string(),
            name: "Header, Footer & Script Injector".to_string(),
            version: "1.4.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Safely inject custom HTML, CSS, tracking pixels (GTM, Meta Pixel, Plausible), or verification tags into <head>, <body>, or <footer>.".to_string(),
            settings_url: Some("/d-admin/plugin-settings?plugin=code-snippets".to_string()),
            is_builtin: true,
            settings_schema: vec![
                PluginSettingField {
                    key: "head_html".to_string(),
                    label: "Header Scripts & Meta (injected before </head>)".to_string(),
                    field_type: "textarea".to_string(),
                    default_value: "".to_string(),
                    description: "Ideal for Google Tag Manager, custom CSS, or verification meta tags.".to_string(),
                    options: vec![],
                },
                PluginSettingField {
                    key: "body_top_html".to_string(),
                    label: "Body Top Snippet (injected right after <body>)".to_string(),
                    field_type: "textarea".to_string(),
                    default_value: "".to_string(),
                    description: "Ideal for announcement banners or GTM noscript tags.".to_string(),
                    options: vec![],
                },
                PluginSettingField {
                    key: "footer_html".to_string(),
                    label: "Footer Scripts (injected before </body>)".to_string(),
                    field_type: "textarea".to_string(),
                    default_value: "".to_string(),
                    description: "Ideal for deferred analytics scripts or chat embeds.".to_string(),
                    options: vec![],
                },
            ],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "daypress-wal-optimizer".to_string(),
            name: "SQLite WAL Checkpoint & Compaction".to_string(),
            version: "1.0.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Automated Write-Ahead-Log maintenance: runs PRAGMA wal_checkpoint(TRUNCATE) every 5 minutes to keep the -wal journal compact and reads sub-millisecond.".to_string(),
            settings_url: Some("/d-admin/daypress-engine".to_string()),
            is_builtin: true,
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "webp-image-engine".to_string(),
            name: "Responsive WebP & Media Pipeline".to_string(),
            version: "2.4.1".to_string(),
            author: "Daypress Core".to_string(),
            description: "On-the-fly dynamic image dimension scaling, focal-point auto-crop matrix, and lossless/lossy WebP transcoding.".to_string(),
            settings_url: Some("/d-admin/options-media".to_string()),
            is_builtin: true,
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "wxr-streaming-xml-importer".to_string(),
            name: "1-Click Migration & WXR Importer".to_string(),
            version: "2.0.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "High-throughput WordPress XML & 1-Click JSON importer that automatically converges Elementor/Divi/Gutenberg layouts, SEO meta, forms, redirects, and active modules.".to_string(),
            settings_url: Some("/d-admin/import".to_string()),
            is_builtin: true,
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "daypress-newsletters".to_string(),
            name: "Dispatches, Newsletters & Member Gate".to_string(),
            version: "1.0.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Native SQLite subscriber lists with Double Opt-In, 1-click RFC-8058 unsubscribe, [daypress_newsletter] signup widget, subscriber-gated posts, and automated new-post email digests.".to_string(),
            settings_url: Some("/d-admin/newsletters".to_string()),
            is_builtin: true,
            settings_schema: vec![
                PluginSettingField {
                    key: "double_opt_in".to_string(),
                    label: "Require Double Opt-In Confirmation Email".to_string(),
                    field_type: "select".to_string(),
                    default_value: "true".to_string(),
                    description: "Send a verification link before activating new subscribers.".to_string(),
                    options: vec!["true".to_string(), "false".to_string()],
                },
                PluginSettingField {
                    key: "auto_digest_on_publish".to_string(),
                    label: "Automatically Email Subscribers When a New Post is Published".to_string(),
                    field_type: "select".to_string(),
                    default_value: "false".to_string(),
                    description: "Automatically dispatches a post digest campaign on publish.".to_string(),
                    options: vec!["false".to_string(), "true".to_string()],
                },
            ],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "daypress-webhooks".to_string(),
            name: "Branded Shortlinks (/go/*) & Signed Webhooks".to_string(),
            version: "1.0.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Branded /go/* affiliate & campaign shortlinks with click/UTM analytics, plus HMAC-SHA256 signed outbound webhooks for Slack, Discord, HubSpot, and Zapier with 1-click replay.".to_string(),
            settings_url: Some("/d-admin/webhooks".to_string()),
            is_builtin: true,
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "daypress-ab-testing".to_string(),
            name: "Zero-Flicker A/B Experiments & Web Vitals".to_string(),
            version: "1.0.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Server-side zero-flicker A/B headline/CTA experiments with two-proportion Z-test statistical confidence, 1-click winner promotion, and real-user Core Web Vitals (LCP / INP / CLS p75).".to_string(),
            settings_url: Some("/d-admin/ab-testing".to_string()),
            is_builtin: true,
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "daypress-wasm-runtime".to_string(),
            name: "WebAssembly (.wasm) Extension Sandbox".to_string(),
            version: "1.0.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Run third-party shortcodes, content filters, and custom logic in a memory-isolated, fuel-metered WebAssembly (.wasm) sandbox with explicit capability permissions.".to_string(),
            settings_url: Some("/d-admin/wasm".to_string()),
            is_builtin: true,
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
    ]
}

/// All built-in modules can be activated or deactivated individually by the site owner
/// (or automatically activated when importing a WordPress site).
pub const CORE_ALWAYS_ON_PLUGINS: &[&str] = &[];

pub fn is_core_always_on(slug: &str) -> bool {
    CORE_ALWAYS_ON_PLUGINS.contains(&slug)
}

/// Returns the compiled catalog manifest for a built-in slug (if any).
pub fn builtin_manifest(slug: &str) -> Option<PluginManifest> {
    builtin_plugins_catalog()
        .into_iter()
        .find(|m| m.slug == slug)
}

/// Editable "files" of a declarative community plugin in the Plugin File Editor.
/// Each hook file maps to one field of [`PluginHooksManifest`].
pub const PLUGIN_MANIFEST_FILE: &str = "daypress-plugin.json";
pub const PLUGIN_HOOK_FILES: &[&str] = &[
    "hooks/head_html.html",
    "hooks/body_top_html.html",
    "hooks/content_before_html.html",
    "hooks/content_after_html.html",
    "hooks/footer_html.html",
    "hooks/shortcode_html.html",
];

/// Returns a mutable reference to the hook template backing an editor hook file.
pub fn hook_field_mut<'a>(
    hooks: &'a mut PluginHooksManifest,
    file: &str,
) -> Option<&'a mut String> {
    match file {
        "hooks/head_html.html" => Some(&mut hooks.head_html),
        "hooks/body_top_html.html" => Some(&mut hooks.body_top_html),
        "hooks/content_before_html.html" => Some(&mut hooks.content_before_html),
        "hooks/content_after_html.html" => Some(&mut hooks.content_after_html),
        "hooks/footer_html.html" => Some(&mut hooks.footer_html),
        "hooks/shortcode_html.html" => Some(&mut hooks.shortcode_html),
        _ => None,
    }
}

/// Returns the hook template backing an editor hook file.
pub fn hook_field<'a>(hooks: &'a PluginHooksManifest, file: &str) -> Option<&'a str> {
    match file {
        "hooks/head_html.html" => Some(&hooks.head_html),
        "hooks/body_top_html.html" => Some(&hooks.body_top_html),
        "hooks/content_before_html.html" => Some(&hooks.content_before_html),
        "hooks/content_after_html.html" => Some(&hooks.content_after_html),
        "hooks/footer_html.html" => Some(&hooks.footer_html),
        "hooks/shortcode_html.html" => Some(&hooks.shortcode_html),
        _ => None,
    }
}

fn is_valid_plugin_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.chars().count() <= 64
        && slug
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

fn is_valid_setting_key(key: &str) -> bool {
    !key.is_empty()
        && key.chars().count() <= 64
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Validates and normalizes a community plugin manifest before it is persisted.
/// Returns non-fatal warnings (e.g. declared-but-unsupported hooks) or a fatal error message.
pub fn validate_community_manifest(manifest: &mut PluginManifest) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();
    manifest.slug = manifest.slug.trim().to_string();
    manifest.name = manifest.name.trim().to_string();
    if !is_valid_plugin_slug(&manifest.slug) {
        return Err(
            "\"slug\" must be 1-64 chars of lowercase letters, digits, '-' or '_'".to_string(),
        );
    }
    if manifest.name.is_empty() {
        return Err("\"name\" is required".to_string());
    }
    if builtin_manifest(&manifest.slug).is_some() {
        return Err(format!(
            "\"{}\" is a reserved built-in plugin slug; choose a different slug",
            manifest.slug
        ));
    }
    // Community manifests can never elevate themselves to built-in status.
    manifest.is_builtin = false;
    if let Some(url) = manifest.settings_url.as_ref() {
        let u = url.trim();
        if u.is_empty() || !u.starts_with('/') || u.starts_with("//") {
            if !u.is_empty() {
                warnings.push(format!(
                    "settings_url \"{}\" ignored (only site-relative paths starting with '/' are allowed)",
                    u
                ));
            }
            manifest.settings_url = None;
        }
    }
    let mut seen = std::collections::HashSet::new();
    for field in &mut manifest.settings_schema {
        field.key = field.key.trim().to_string();
        if !is_valid_setting_key(&field.key) {
            return Err(format!(
                "settings_schema key \"{}\" is invalid (use letters, digits, '-' or '_')",
                field.key
            ));
        }
        if !seen.insert(field.key.clone()) {
            return Err(format!("duplicate settings_schema key \"{}\"", field.key));
        }
        if field.label.trim().is_empty() {
            field.label = field.key.clone();
        }
        if field.field_type == "select"
            && !field.options.is_empty()
            && !field.options.contains(&field.default_value)
        {
            field.default_value = field.options[0].clone();
        }
    }
    let sc = manifest.hooks.shortcode.trim().to_string();
    if !sc.is_empty() && !is_valid_setting_key(&sc) {
        return Err(format!(
            "hooks.shortcode \"{}\" is invalid (use letters, digits, '-' or '_', without brackets)",
            sc
        ));
    }
    manifest.hooks.shortcode = sc;
    if !manifest.hooks.shortcode.is_empty() && manifest.hooks.shortcode_html.trim().is_empty() {
        warnings.push(format!(
            "shortcode [{}] declared without hooks.shortcode_html; it will not render",
            manifest.hooks.shortcode
        ));
    }
    if !manifest.hooks.on_post_publish_webhook.trim().is_empty()
        || !manifest.hooks.on_form_submit_webhook.trim().is_empty()
    {
        warnings.push(
            "on_post_publish_webhook / on_form_submit_webhook are not supported yet and are ignored"
                .to_string(),
        );
    }
    Ok(warnings)
}

/// Effective plugin config used for SSR: schema defaults overlaid with saved values.
/// Values of non-`textarea` fields are HTML-escaped (they are typically interpolated into
/// attributes such as `style="background: {{bg_color}}"`); textarea fields are raw HTML.
fn effective_render_config(
    manifest: &PluginManifest,
    config: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut out: HashMap<String, String> = HashMap::new();
    for field in &manifest.settings_schema {
        let raw = config
            .get(&field.key)
            .cloned()
            .unwrap_or_else(|| field.default_value.clone());
        let val = if matches!(field.field_type.as_str(), "textarea" | "html" | "code") {
            raw
        } else {
            crate::templates::utils::escape_html(&raw)
        };
        out.insert(field.key.clone(), val);
    }
    for (k, v) in config {
        out.entry(k.clone())
            .or_insert_with(|| crate::templates::utils::escape_html(v));
    }
    out
}

#[derive(Default, Debug, Clone)]
pub struct RenderedPluginHooks {
    pub head_html: String,
    pub body_top_html: String,
    pub content_before_html: String,
    pub content_after_html: String,
    pub footer_html: String,
}

fn interpolate_template(
    raw: &str,
    config: &HashMap<String, String>,
    settings: &DaypressSettingsResponse,
    current_post: Option<&DaypressPostResponse>,
) -> String {
    if raw.is_empty() {
        return String::new();
    }
    let mut out = raw.to_string();
    let page_title = current_post
        .map(|p| p.title.rendered.as_str())
        .unwrap_or(settings.title.as_str());
    let page_slug = current_post.map(|p| p.slug.as_str()).unwrap_or("");
    let page_url = if page_slug.is_empty() {
        settings.url.clone()
    } else {
        format!("{}/{}/", settings.url.trim_end_matches('/'), page_slug)
    };

    out = out.replace("{{site_name}}", &settings.title);
    out = out.replace("{{site_url}}", &settings.url);
    out = out.replace("{{page_title}}", page_title);
    out = out.replace("{{page_url}}", &page_url);

    for (k, v) in config {
        let placeholder = format!("{{{{{}}}}}", k);
        out = out.replace(&placeholder, v);
    }
    out
}

/// Evaluates all active built-in and community plugins to produce SSR HTML hook injections.
pub fn render_active_plugin_hooks(
    db: &Database,
    settings: &DaypressSettingsResponse,
    current_post: Option<&DaypressPostResponse>,
) -> RenderedPluginHooks {
    let mut hooks = RenderedPluginHooks::default();
    if settings.active_plugins.is_empty() && !settings.title.is_empty() {
        return hooks;
    }
    let installed = db.get_active_installed_plugins().unwrap_or_default();

    for record in installed {
        let slug = record.manifest.slug.as_str();
        let cfg = &record.config;

        if slug == "whatsapp-chat" {
            let phone_raw = cfg
                .get("phone_number")
                .map(|s| s.as_str())
                .unwrap_or("+15550192834");
            let clean_phone: String = phone_raw.chars().filter(|c| c.is_ascii_digit()).collect();
            if !clean_phone.is_empty() {
                let welcome = interpolate_template(
                    cfg.get("welcome_message")
                        .map(|s| s.as_str())
                        .unwrap_or("Hi! I have a question."),
                    cfg,
                    settings,
                    current_post,
                );
                let encoded_msg: String = welcome
                    .bytes()
                    .map(|b| match b {
                        b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => {
                            (b as char).to_string()
                        }
                        b' ' => "%20".to_string(),
                        _ => format!("%{:02X}", b),
                    })
                    .collect();
                let tooltip = cfg
                    .get("tooltip_text")
                    .map(|s| s.as_str())
                    .unwrap_or("Chat with us on WhatsApp");
                let pos = cfg
                    .get("position")
                    .map(|s| s.as_str())
                    .unwrap_or("bottom-right");
                let color = cfg
                    .get("accent_color")
                    .map(|s| s.as_str())
                    .filter(|s| !s.is_empty())
                    .unwrap_or("#25D366");
                let pos_css = if pos == "bottom-left" {
                    "left: 24px;"
                } else {
                    "right: 24px;"
                };

                hooks.footer_html.push_str(&format!(
                    r##"
<!-- Daypress Native WhatsApp Chat Widget -->
<div id="dp-whatsapp-widget" style="position: fixed; bottom: 24px; {pos_css} z-index: 9998; display: flex; align-items: center; gap: 10px; font-family: var(--font-sans, sans-serif);">
    <a href="https://wa.me/{phone}?text={msg}" target="_blank" rel="noopener noreferrer" aria-label="{tooltip}"
       style="display: inline-flex; align-items: center; gap: 10px; background: {color}; color: #ffffff; padding: 12px 18px; border-radius: 9999px; font-weight: 700; font-size: 14px; text-decoration: none; box-shadow: 0 10px 25px -5px rgba(37, 211, 102, 0.5); transition: transform 0.2s ease;">
        <svg width="22" height="22" viewBox="0 0 24 24" fill="currentColor"><path d="M12.031 6.172c-3.181 0-5.767 2.586-5.768 5.766-.001 1.298.38 2.27 1.019 3.287l-.582 2.128 2.182-.573c.978.58 1.911.928 3.145.929 3.178 0 5.767-2.587 5.768-5.766.001-3.187-2.575-5.77-5.764-5.771zm3.392 8.244c-.144.405-.837.774-1.17.824-.299.045-.677.063-1.092-.069-.252-.08-.575-.187-.988-.365-1.739-.751-2.874-2.502-2.961-2.617-.087-.116-.708-.94-.708-1.793s.448-1.273.607-1.446c.159-.173.346-.217.462-.217l.332.006c.106.005.249-.04.39.298.144.347.491 1.2.534 1.287.043.087.072.188.014.304-.058.116-.087.188-.173.289l-.26.304c-.087.086-.177.18-.076.354.101.174.449.741.964 1.201.662.591 1.221.774 1.394.86s.274.072.376-.043c.101-.116.433-.506.549-.68.116-.173.231-.145.39-.087s1.011.477 1.184.564.289.13.332.202c.045.072.045.419-.1.824zm-3.423-14.416c-6.627 0-12 5.373-12 12 0 2.625.846 5.059 2.284 7.033l-1.494 5.462 5.601-1.469c1.908 1.132 4.132 1.787 6.517 1.787 6.627 0 12-5.373 12-12 0-6.627-5.373-12-11.908-12.813z"/></svg>
        <span>{tooltip}</span>
    </a>
</div>
"##,
                    pos_css = pos_css,
                    phone = clean_phone,
                    msg = encoded_msg,
                    tooltip = crate::templates::utils::escape_html(tooltip),
                    color = crate::templates::utils::escape_html(color),
                ));
            }
        } else if slug == "code-snippets" {
            if let Some(h) = cfg.get("head_html") {
                if !h.trim().is_empty() {
                    hooks.head_html.push('\n');
                    hooks.head_html.push_str(h);
                }
            }
            if let Some(b) = cfg.get("body_top_html") {
                if !b.trim().is_empty() {
                    hooks.body_top_html.push('\n');
                    hooks.body_top_html.push_str(b);
                }
            }
            if let Some(f) = cfg.get("footer_html") {
                if !f.trim().is_empty() {
                    hooks.footer_html.push('\n');
                    hooks.footer_html.push_str(f);
                }
            }
        }

        // Community & Declarative Plugin Hook Templates
        let hm = &record.manifest.hooks;
        let render_cfg = effective_render_config(&record.manifest, cfg);
        let cfg = &render_cfg;
        if !hm.head_html.is_empty() {
            hooks.head_html.push_str(&interpolate_template(
                &hm.head_html,
                cfg,
                settings,
                current_post,
            ));
        }
        if !hm.body_top_html.is_empty() {
            hooks.body_top_html.push_str(&interpolate_template(
                &hm.body_top_html,
                cfg,
                settings,
                current_post,
            ));
        }
        if !hm.content_before_html.is_empty() {
            hooks.content_before_html.push_str(&interpolate_template(
                &hm.content_before_html,
                cfg,
                settings,
                current_post,
            ));
        }
        if !hm.content_after_html.is_empty() {
            hooks.content_after_html.push_str(&interpolate_template(
                &hm.content_after_html,
                cfg,
                settings,
                current_post,
            ));
        }
        if !hm.footer_html.is_empty() {
            hooks.footer_html.push_str(&interpolate_template(
                &hm.footer_html,
                cfg,
                settings,
                current_post,
            ));
        }
    }

    hooks
}

/// Renders any `[daypress_form id="X"]` shortcodes or community plugin shortcodes inside post content.
pub fn expand_shortcodes_in_content(
    content: &str,
    db: &Database,
    settings: &DaypressSettingsResponse,
) -> String {
    expand_shortcodes_in_content_for_post(content, db, settings, None)
}

pub fn expand_shortcodes_in_content_for_post(
    content: &str,
    db: &Database,
    settings: &DaypressSettingsResponse,
    current_post: Option<&DaypressPostResponse>,
) -> String {
    let mut out = content.to_string();
    // Form shortcodes are provided by the built-in "daypress-forms" plugin; like WordPress
    // with Contact Form 7 deactivated, the raw shortcode is left untouched when it is inactive.
    let forms_active = out.contains('[')
        && (db.is_plugin_active("daypress-forms")
            || !db.list_forms().unwrap_or_default().is_empty());

    // 1. Expand [daypress_form id="X"] (or a bare [daypress_form] -> the first form)
    if forms_active && out.contains("[daypress_form") {
        static FORM_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        let re = FORM_RE.get_or_init(|| {
            regex::Regex::new(r#"\[daypress_form(?:\s+id=["']?(\d+)["']?)?[^\]]*\]"#).unwrap()
        });
        let default_form_id = db
            .list_forms()
            .ok()
            .and_then(|v| v.first().map(|(f, _)| f.id))
            .unwrap_or(1);
        out = re
            .replace_all(&out, |caps: &regex::Captures| {
                let form_id: i64 = caps
                    .get(1)
                    .and_then(|m| m.as_str().parse().ok())
                    .unwrap_or(default_form_id);
                render_public_form_html(db, form_id)
            })
            .into_owned();
    }

    // 2. Also expand any un-converged [contact-form-7 ...] or [wpforms ...] or [ninja_forms ...] shortcodes on the fly
    if forms_active
        && (out.contains("[contact-form-7")
            || out.contains("[wpforms")
            || out.contains("[ninja_forms"))
    {
        static LEGACY_FORM_RE: std::sync::OnceLock<regex::Regex> = std::sync::OnceLock::new();
        let re = LEGACY_FORM_RE.get_or_init(|| {
            regex::Regex::new(r#"\[(?:contact-form-7|wpforms|ninja_forms)[^\]]*\]"#).unwrap()
        });
        if re.is_match(&out) {
            let first_form_id = db
                .list_forms()
                .ok()
                .and_then(|v| v.first().map(|(f, _)| f.id))
                .unwrap_or(1);
            out = re
                .replace_all(&out, |_caps: &regex::Captures| {
                    render_public_form_html(db, first_form_id)
                })
                .into_owned();
        }
    }

    // 3. Built-in TOC & Reading-Time shortcodes ([daypress_toc], [toc], [daypress_reading_time])
    if out.contains("[daypress_reading_time") || out.contains("[reading_time") {
        let (words, mins) =
            crate::adapters::database::search_fts::estimate_reading_time_minutes(&out);
        let badge = format!(
            r#"<span class="dp-reading-time" style="font-family:var(--font-mono, monospace);font-size:12px;color:var(--ink-muted, var(--text-muted));"><i class="fa-regular fa-clock" style="margin-right:4px;"></i>{} min read ({} words)</span>"#,
            mins, words
        );
        out = out
            .replace("[daypress_reading_time]", &badge)
            .replace("[reading_time]", &badge);
    }
    if out.contains("[daypress_toc") || out.contains("[toc]") {
        let (anchored, toc_nav, _) =
            crate::templates::public::single::inject_heading_anchors_and_build_toc(
                &out,
                "Table of Contents",
                2,
                4,
                true,
            );
        out = anchored
            .replace("[daypress_toc]", &toc_nav)
            .replace("[toc]", &toc_nav);
    }

    // 3b. Built-in Language Switcher shortcode ([daypress_language_switcher] / [language_switcher] / [polylang_switcher])
    if out.contains("[daypress_language_switcher")
        || out.contains("[language_switcher")
        || out.contains("[polylang_switcher")
    {
        let default_lang = db.get_default_language_code();
        let active_lang = current_post
            .map(|p| p.lang_code.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or(&default_lang)
            .to_string();
        let links = db.build_language_switcher_links(current_post.map(|p| p.id), &active_lang);
        if let Ok(re) = regex::Regex::new(
            r#"\[(?:daypress_language_switcher|language_switcher|polylang_switcher)([^\]]*)\]"#,
        ) {
            out = re
                .replace_all(&out, |caps: &regex::Captures| {
                    let attrs = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    let style = if attrs.contains("dropdown") {
                        "dropdown"
                    } else {
                        "pills"
                    };
                    let show_flags = !attrs.contains("flags=\"false\"")
                        && !attrs.contains("show_flags=\"false\"");
                    let show_names = !attrs.contains("names=\"false\"")
                        && !attrs.contains("show_names=\"false\"");
                    let show_codes =
                        attrs.contains("codes=\"true\"") || attrs.contains("show_codes=\"true\"");
                    let switcher_node = daypress_ast::DaypressNode {
                        id: "sc-lang-switcher".to_string(),
                        kind: daypress_ast::NodeKind::LanguageSwitcher {
                            display_style: style.to_string(),
                            show_flags,
                            show_names,
                            show_codes,
                            current_lang: active_lang.clone(),
                            links: links.clone(),
                        },
                        styles: daypress_ast::StyleProperties::default(),
                        children: vec![],
                    };
                    daypress_ast::render_layout_to_html(&daypress_ast::DaypressLayout {
                        version: 1,
                        global_styles: None,
                        root: daypress_ast::DaypressNode {
                            id: "root".to_string(),
                            kind: daypress_ast::NodeKind::Root,
                            styles: daypress_ast::StyleProperties::default(),
                            children: vec![switcher_node],
                        },
                    })
                })
                .into_owned();
        }
    }

    // 3c. Built-in Newsletter Signup shortcode ([daypress_newsletter], [newsletter_signup], [mailpoet_form])
    if out.contains("[daypress_newsletter")
        || out.contains("[newsletter_signup")
        || out.contains("[mailpoet_form")
    {
        if let Ok(re) = regex::Regex::new(
            r#"\[(?:daypress_newsletter|newsletter_signup|mailpoet_form)([^\]]*)\]"#,
        ) {
            out = re
                .replace_all(&out, |caps: &regex::Captures| {
                    let attrs = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                    let extract_attr = |key: &str, default_val: &str| -> String {
                        let pat = format!(r#"{}\s*=\s*"([^"]*)""#, key);
                        regex::Regex::new(&pat)
                            .ok()
                            .and_then(|r| r.captures(attrs))
                            .and_then(|c| c.get(1).map(|m| m.as_str().to_string()))
                            .filter(|s| !s.trim().is_empty())
                            .unwrap_or_else(|| default_val.to_string())
                    };
                    let title = extract_attr("title", "Subscribe to our newsletter");
                    let subtitle = extract_attr(
                        "subtitle",
                        "Get new posts and editorial dispatches delivered straight to your inbox. No spam, unsubscribe anytime.",
                    );
                    let list_name = extract_attr("list", "general");
                    let btn_label = extract_attr("button", "Subscribe");
                    render_newsletter_widget_html(&title, &subtitle, &list_name, &btn_label)
                })
                .into_owned();
        }
    }

    // 4. Community plugin custom shortcodes
    if out.contains('[') && (!settings.active_plugins.is_empty() || settings.title.is_empty()) {
        if let Ok(plugins) = db.get_active_installed_plugins() {
            for p in plugins {
                let sc = p.manifest.hooks.shortcode.trim();
                if !sc.is_empty() && !p.manifest.hooks.shortcode_html.is_empty() {
                    let tag = format!("[{}]", sc);
                    if out.contains(&tag) {
                        let render_cfg = effective_render_config(&p.manifest, &p.config);
                        let rendered = interpolate_template(
                            &p.manifest.hooks.shortcode_html,
                            &render_cfg,
                            settings,
                            None,
                        );
                        out = out.replace(&tag, &rendered);
                    }
                }
            }
        }
        // 5. Sandboxed WebAssembly (`.wasm`) extension shortcodes (`v0.9`)
        if settings
            .active_plugins
            .iter()
            .any(|p| p == "daypress-wasm-runtime")
            || (settings.title.is_empty() && db.is_plugin_active("daypress-wasm-runtime"))
        {
            out = db.expand_active_wasm_shortcodes(&out);
        }
    }

    out
}

/// Renders an interactive HTML5 form from `daypress_forms` wired to `/wp-json/daypress/v1/forms/{id}/submit`.
pub fn render_public_form_html(db: &Database, form_id: i64) -> String {
    let form_opt = db.get_form(form_id).ok().flatten();
    let (id, title, fields, form_settings) = match form_opt {
        Some(f) => {
            let fields: Vec<crate::models::FormFieldSchema> =
                serde_json::from_str(&f.schema_json).unwrap_or_default();
            let s: crate::models::FormSettings =
                serde_json::from_str(&f.settings_json).unwrap_or_default();
            (f.id, f.title, fields, s)
        }
        None => (
            form_id,
            "Contact Us".to_string(),
            vec![
                crate::models::FormFieldSchema {
                    name: "your_name".to_string(),
                    label: "Your Name".to_string(),
                    field_type: "text".to_string(),
                    required: true,
                    placeholder: "Jane Doe".to_string(),
                    ..Default::default()
                },
                crate::models::FormFieldSchema {
                    name: "your_email".to_string(),
                    label: "Your Email".to_string(),
                    field_type: "email".to_string(),
                    required: true,
                    placeholder: "jane@example.com".to_string(),
                    ..Default::default()
                },
                crate::models::FormFieldSchema {
                    name: "your_message".to_string(),
                    label: "Message".to_string(),
                    field_type: "textarea".to_string(),
                    required: true,
                    placeholder: "How can we help you?".to_string(),
                    ..Default::default()
                },
            ],
            crate::models::FormSettings::default(),
        ),
    };

    let max_step = fields.iter().map(|f| f.step.max(1)).max().unwrap_or(1);
    let is_multistep = max_step > 1;

    let mut steps_html = String::new();
    for step_idx in 1..=max_step {
        let step_display = if step_idx == 1 {
            "display: flex;"
        } else {
            "display: none;"
        };
        let mut step_fields_html = String::new();

        for field in fields.iter().filter(|f| f.step.max(1) == step_idx) {
            let req_attr = if field.required {
                "required data-dp-required=\"1\""
            } else {
                ""
            };
            let req_star = if field.required {
                r#" <span style="color: #ef4444;">*</span>"#
            } else {
                ""
            };
            let fname = crate::templates::utils::escape_html(&field.name);
            let flabel = crate::templates::utils::escape_html(&field.label);
            let fph = crate::templates::utils::escape_html(&field.placeholder);
            let fdef = crate::templates::utils::escape_html(&field.default_value);
            let fhelp = if field.help_text.trim().is_empty() {
                String::new()
            } else {
                format!(
                    r#"<div style="font-size: 12px; color: var(--text-muted, #94a3b8); margin-top: 4px;">{}</div>"#,
                    crate::templates::utils::escape_html(&field.help_text)
                )
            };

            let width_css = match field.width.as_str() {
                "half" | "1/2" => "flex: 1 1 calc(50% - 12px); min-width: 220px;",
                "third" | "1/3" => "flex: 1 1 calc(33.333% - 12px); min-width: 180px;",
                "two_thirds" | "2/3" => "flex: 1 1 calc(66.666% - 12px); min-width: 240px;",
                _ => "flex: 1 1 100%; width: 100%;",
            };

            let cond_attrs = if let Some(ref c) = field.condition {
                if !c.field.trim().is_empty() {
                    format!(
                        r#" data-cond-field="{}" data-cond-op="{}" data-cond-val="{}""#,
                        crate::templates::utils::escape_html(&c.field),
                        crate::templates::utils::escape_html(&c.operator),
                        crate::templates::utils::escape_html(&c.value)
                    )
                } else {
                    String::new()
                }
            } else {
                String::new()
            };

            let base_input_style = "width: 100%; padding: 10px 14px; border-radius: 8px; border: 1px solid var(--border, #334155); background: var(--bg-base, #090d16); color: var(--text-main, #f8fafc); font-family: inherit; font-size: 14px;";

            if field.field_type == "hidden" {
                step_fields_html.push_str(&format!(
                    r#"<input type="hidden" name="{fname}" value="{fdef}" />"#
                ));
                continue;
            }

            let input_el = match field.field_type.as_str() {
                "textarea" => format!(
                    r#"<textarea name="{fname}" rows="4" placeholder="{fph}" {req_attr} style="{base_input_style}">{fdef}</textarea>"#
                ),
                "select" => {
                    let mut opts = String::new();
                    if !fph.is_empty() {
                        opts.push_str(&format!(r#"<option value="">{fph}</option>"#));
                    }
                    for o in &field.options {
                        let o_esc = crate::templates::utils::escape_html(o);
                        let sel = if o == &field.default_value {
                            " selected"
                        } else {
                            ""
                        };
                        opts.push_str(&format!(r#"<option value="{o_esc}"{sel}>{o_esc}</option>"#));
                    }
                    format!(
                        r#"<select name="{fname}" {req_attr} style="{base_input_style}">{opts}</select>"#
                    )
                }
                "radio" => {
                    let mut radios = String::new();
                    for (idx, o) in field.options.iter().enumerate() {
                        let o_esc = crate::templates::utils::escape_html(o);
                        let r_req = if field.required && idx == 0 {
                            req_attr
                        } else {
                            ""
                        };
                        radios.push_str(&format!(
                            r#"<label style="display: inline-flex; align-items: center; gap: 8px; margin-right: 16px; cursor: pointer; font-size: 14px;"><input type="radio" name="{fname}" value="{o_esc}" {r_req} /> <span>{o_esc}</span></label>"#
                        ));
                    }
                    format!(
                        r#"<div style="display: flex; flex-wrap: wrap; gap: 8px; padding-top: 4px;">{radios}</div>"#
                    )
                }
                "checkbox_group" => {
                    let mut boxes = String::new();
                    for o in &field.options {
                        let o_esc = crate::templates::utils::escape_html(o);
                        boxes.push_str(&format!(
                            r#"<label style="display: inline-flex; align-items: center; gap: 8px; margin-right: 16px; cursor: pointer; font-size: 14px;"><input type="checkbox" name="{fname}" value="{o_esc}" /> <span>{o_esc}</span></label>"#
                        ));
                    }
                    format!(
                        r#"<div style="display: flex; flex-wrap: wrap; gap: 8px; padding-top: 4px;">{boxes}</div>"#
                    )
                }
                "checkbox" | "consent" => format!(
                    r#"<label style="display: flex; align-items: flex-start; gap: 8px; cursor: pointer; font-size: 14px;"><input type="checkbox" name="{fname}" value="yes" {req_attr} style="margin-top: 3px;" /> <span>{flabel}{req_star}</span></label>"#
                ),
                "file" => {
                    let accept_attr = if field.accept_extensions.trim().is_empty() {
                        String::new()
                    } else {
                        format!(
                            r#" accept="{}""#,
                            crate::templates::utils::escape_html(&field.accept_extensions)
                        )
                    };
                    format!(
                        r#"<input type="file" name="{fname}"{accept_attr} {req_attr} style="{base_input_style}" />"#
                    )
                }
                "calculated" => {
                    let formula_esc = crate::templates::utils::escape_html(
                        field.calculation_formula.as_deref().unwrap_or(""),
                    );
                    format!(
                        r#"<input type="text" name="{fname}" value="{fdef}" readonly data-calc-formula="{formula_esc}" style="{base_input_style} font-weight: 700; opacity: 0.92;" />"#
                    )
                }
                other => {
                    let itype = match other {
                        "email" | "tel" | "url" | "number" | "date" | "time" => other,
                        _ => "text",
                    };
                    format!(
                        r#"<input type="{itype}" name="{fname}" value="{fdef}" placeholder="{fph}" {req_attr} style="{base_input_style}" />"#
                    )
                }
            };

            if matches!(field.field_type.as_str(), "checkbox" | "consent") {
                step_fields_html.push_str(&format!(
                    r#"<div class="dp-form-field-wrap" style="{width_css} margin-bottom: 16px;"{cond_attrs}>{input_el}{fhelp}</div>"#
                ));
            } else {
                step_fields_html.push_str(&format!(
                    r#"<div class="dp-form-field-wrap" style="{width_css} margin-bottom: 16px;"{cond_attrs}>
                        <label style="display: block; font-size: 13px; font-weight: 600; margin-bottom: 6px; color: var(--text-main, #f8fafc);">{flabel}{req_star}</label>
                        {input_el}
                        {fhelp}
                    </div>"#
                ));
            }
        }

        let nav_buttons = if is_multistep {
            let prev_btn = if step_idx > 1 {
                format!(
                    r#"<button type="button" class="dp-button dp-step-prev" onclick="window.dpFormStep({id}, {prev})" style="cursor: pointer; border: 1px solid var(--border, #334155); padding: 10px 18px; border-radius: 8px; background: transparent; color: var(--text-main, #f8fafc); font-weight: 600; font-size: 14px;">&larr; Previous</button>"#,
                    id = id,
                    prev = step_idx - 1
                )
            } else {
                String::new()
            };
            let next_or_submit = if step_idx < max_step {
                format!(
                    r#"<button type="button" class="dp-button dp-step-next" onclick="window.dpFormStep({id}, {next})" style="cursor: pointer; border: none; padding: 11px 22px; border-radius: 8px; background: var(--accent, #2563eb); color: #ffffff; font-weight: 700; font-size: 14px;">Next Step &rarr;</button>"#,
                    id = id,
                    next = step_idx + 1
                )
            } else {
                format!(
                    r#"<button type="submit" class="dp-button" style="cursor: pointer; border: none; padding: 11px 24px; border-radius: 8px; background: var(--accent, #2563eb); color: #ffffff; font-weight: 700; font-size: 14px;">{}</button>"#,
                    crate::templates::utils::escape_html(&form_settings.submit_label)
                )
            };
            format!(
                r#"<div style="width: 100%; display: flex; justify-content: space-between; align-items: center; gap: 12px; margin-top: 8px;">{prev_btn}{next_or_submit}</div>"#
            )
        } else {
            String::new()
        };

        steps_html.push_str(&format!(
            r#"<div class="dp-form-step" data-step="{step_idx}" style="{step_display} flex-wrap: wrap; gap: 0 16px;">{step_fields_html}{nav_buttons}</div>"#
        ));
    }

    let step_header_html = if is_multistep {
        let first_step_title = form_settings
            .step_titles
            .first()
            .map(|s| s.as_str())
            .unwrap_or("Step 1");
        let step_titles_json =
            serde_json::to_string(&form_settings.step_titles).unwrap_or_else(|_| "[]".into());
        let pct = (100.0 / max_step as f64).round() as i32;
        format!(
            r#"<div id="dp-step-header-{id}" data-max-step="{max_step}" data-step-titles='{titles_json}' style="margin-bottom: 18px;">
                <div style="display: flex; justify-content: space-between; font-size: 12.5px; font-weight: 600; color: var(--text-muted, #94a3b8); margin-bottom: 6px;">
                    <span id="dp-step-label-{id}">Step 1 of {max_step}: {first_title}</span>
                    <span id="dp-step-pct-{id}">{pct}%</span>
                </div>
                <div style="width: 100%; height: 6px; background: rgba(148,163,184,0.2); border-radius: 999px; overflow: hidden;">
                    <div id="dp-step-bar-{id}" style="width: {pct}%; height: 100%; background: var(--accent, #2563eb); transition: width 0.25s ease;"></div>
                </div>
            </div>"#,
            id = id,
            max_step = max_step,
            titles_json = crate::templates::utils::escape_html(&step_titles_json),
            first_title = crate::templates::utils::escape_html(first_step_title),
            pct = pct
        )
    } else {
        String::new()
    };

    let single_submit_btn = if !is_multistep {
        format!(
            r#"<button type="submit" class="dp-button" style="cursor: pointer; border: none; padding: 11px 24px; border-radius: 8px; background: var(--accent, #2563eb); color: #ffffff; font-weight: 700; font-size: 14px;">{}</button>"#,
            crate::templates::utils::escape_html(&form_settings.submit_label)
        )
    } else {
        String::new()
    };

    let captcha_html = match form_settings.captcha_provider.as_str() {
        "turnstile" if !form_settings.captcha_site_key.trim().is_empty() => format!(
            r#"<div class="cf-turnstile" data-sitekey="{}" style="margin-bottom: 14px;"></div><script src="https://challenges.cloudflare.com/turnstile/v0/api.js" async defer></script>"#,
            crate::templates::utils::escape_html(form_settings.captcha_site_key.trim())
        ),
        "hcaptcha" if !form_settings.captcha_site_key.trim().is_empty() => format!(
            r#"<div class="h-captcha" data-sitekey="{}" style="margin-bottom: 14px;"></div><script src="https://js.hcaptcha.com/1/api.js" async defer></script>"#,
            crate::templates::utils::escape_html(form_settings.captcha_site_key.trim())
        ),
        _ => String::new(),
    };

    let success_msg = crate::templates::utils::escape_html(&form_settings.success_message);
    let redirect_url = crate::templates::utils::escape_html(&form_settings.redirect_url);
    let title_esc = crate::templates::utils::escape_html(&title);

    format!(
        r##"<div class="dp-native-form-card" style="background: var(--bg-card, #111827); border: 1px solid var(--border, #1e293b); border-radius: 12px; padding: 28px; margin: 24px 0;">
            <h3 style="font-size: 18px; font-weight: 700; margin: 0 0 18px 0; color: var(--text-main, #f8fafc);">{title_esc}</h3>
            {step_header_html}
            <form id="dp-form-{id}" data-form-id="{id}" onsubmit="return submitDaypressForm(event, {id})" oninput="window.dpEvaluateFormLogic && window.dpEvaluateFormLogic(this)" onchange="window.dpEvaluateFormLogic && window.dpEvaluateFormLogic(this)">
                <div style="display:none;" aria-hidden="true">
                    <input type="text" name="_np_hp" value="" tabindex="-1" autocomplete="off" />
                    <input type="hidden" name="_dp_ts" value="" />
                    <input type="hidden" name="_source_url" value="" />
                    <input type="hidden" name="_utm_source" value="" />
                    <input type="hidden" name="_utm_medium" value="" />
                    <input type="hidden" name="_utm_campaign" value="" />
                    <input type="hidden" name="_referrer" value="" />
                </div>
                {steps_html}
                {captcha_html}
                {single_submit_btn}
                <div id="dp-form-status-{id}" style="margin-top: 12px; font-size: 13.5px; font-weight: 600; display: none;" data-success="{success_msg}" data-redirect="{redirect_url}"></div>
            </form>
            <script>
            (function() {{
                var f = document.getElementById('dp-form-{id}');
                if (f) {{
                    var ts = f.querySelector('input[name="_dp_ts"]');
                    if (ts) ts.value = String(Date.now());
                    var src = f.querySelector('input[name="_source_url"]');
                    if (src) src.value = window.location.pathname + window.location.search;
                    var ref = f.querySelector('input[name="_referrer"]');
                    if (ref) ref.value = document.referrer || '';
                    try {{
                        var sp = new URLSearchParams(window.location.search);
                        ['utm_source', 'utm_medium', 'utm_campaign'].forEach(function(k) {{
                            var el = f.querySelector('input[name="_' + k + '"]');
                            if (el && sp.get(k)) el.value = sp.get(k);
                        }});
                    }} catch (_) {{}}
                }}
                if (!window.dpEvaluateFormLogic) {{
                    window.dpEvaluateFormLogic = function(form) {{
                        var fd = new FormData(form);
                        form.querySelectorAll('[data-cond-field]').forEach(function(wrap) {{
                            var cf = wrap.getAttribute('data-cond-field');
                            var op = wrap.getAttribute('data-cond-op') || 'equals';
                            var cv = (wrap.getAttribute('data-cond-val') || '').toLowerCase();
                            var actual = String(fd.get(cf) || '').toLowerCase();
                            var show = op === 'not_equals' ? actual !== cv : (op === 'contains' ? actual.indexOf(cv) !== -1 : actual === cv);
                            wrap.style.display = show ? '' : 'none';
                            wrap.querySelectorAll('[data-dp-required="1"]').forEach(function(inp) {{
                                if (show) inp.setAttribute('required', 'required');
                                else inp.removeAttribute('required');
                            }});
                        }});
                        form.querySelectorAll('input[data-calc-formula]').forEach(function(calcInp) {{
                            var expr = calcInp.getAttribute('data-calc-formula') || '';
                            expr = expr.replace(/\{{([^}}]+)\}}/g, function(_, key) {{
                                var v = parseFloat(fd.get(key.trim()) || '0');
                                return isNaN(v) ? '0' : String(v);
                            }});
                            if (/^[0-9+\-*/().\s]+$/.test(expr) && expr.trim() !== '') {{
                                try {{
                                    var val = Function('"use strict";return (' + expr + ')')();
                                    if (typeof val === 'number' && isFinite(val)) {{
                                        calcInp.value = Number.isInteger(val) ? String(val) : val.toFixed(2);
                                    }}
                                }} catch (_) {{}}
                            }}
                        }});
                    }};
                }}
                if (!window.dpFormStep) {{
                    window.dpFormStep = function(formId, targetStep) {{
                        var form = document.getElementById('dp-form-' + formId);
                        if (!form) return;
                        var steps = form.querySelectorAll('.dp-form-step');
                        var currentStepEl = null;
                        steps.forEach(function(s) {{ if (s.style.display !== 'none') currentStepEl = s; }});
                        if (currentStepEl && Number(currentStepEl.getAttribute('data-step')) < targetStep) {{
                            var inputs = currentStepEl.querySelectorAll('input, select, textarea');
                            for (var i = 0; i < inputs.length; i++) {{
                                if (!inputs[i].checkValidity()) {{
                                    inputs[i].reportValidity();
                                    return;
                                }}
                            }}
                        }}
                        steps.forEach(function(s) {{
                            s.style.display = Number(s.getAttribute('data-step')) === targetStep ? 'flex' : 'none';
                        }});
                        var hdr = document.getElementById('dp-step-header-' + formId);
                        if (hdr) {{
                            var maxS = Number(hdr.getAttribute('data-max-step') || 1);
                            var titles = [];
                            try {{ titles = JSON.parse(hdr.getAttribute('data-step-titles') || '[]'); }} catch (_) {{}}
                            var tLabel = titles[targetStep - 1] || ('Step ' + targetStep);
                            var pct = Math.round((targetStep / maxS) * 100);
                            var lblEl = document.getElementById('dp-step-label-' + formId);
                            var pctEl = document.getElementById('dp-step-pct-' + formId);
                            var barEl = document.getElementById('dp-step-bar-' + formId);
                            if (lblEl) lblEl.textContent = 'Step ' + targetStep + ' of ' + maxS + ': ' + tLabel;
                            if (pctEl) pctEl.textContent = pct + '%';
                            if (barEl) barEl.style.width = pct + '%';
                        }}
                    }};
                }}
                if (!window.submitDaypressForm) {{
                    window.submitDaypressForm = async function(e, formId) {{
                        e.preventDefault();
                        var form = e.target;
                        var statusEl = document.getElementById('dp-form-status-' + formId);
                        var formData = new FormData(form);
                        var hasFile = !!form.querySelector('input[type="file"]');
                        var fetchOpts = hasFile
                            ? {{ method: 'POST', body: formData }}
                            : (function() {{
                                var data = {{}};
                                formData.forEach(function(v, k) {{
                                    if (data[k] !== undefined) data[k] = data[k] + ', ' + v;
                                    else data[k] = v;
                                }});
                                return {{
                                    method: 'POST',
                                    headers: {{ 'Content-Type': 'application/json' }},
                                    body: JSON.stringify({{ data: data }})
                                }};
                            }})();
                        try {{
                            var res = await fetch('/wp-json/daypress/v1/forms/' + formId + '/submit', fetchOpts);
                            var payload = await res.json().catch(function() {{ return {{}}; }});
                            if (res.ok) {{
                                form.reset();
                                var ts = form.querySelector('input[name="_dp_ts"]');
                                if (ts) ts.value = String(Date.now());
                                if (statusEl) {{
                                    statusEl.style.display = 'block';
                                    statusEl.style.color = '#10b981';
                                    statusEl.textContent = '✓ ' + (payload.message || statusEl.getAttribute('data-success') || 'Submitted successfully!');
                                }}
                                var redir = payload.redirect_url || (statusEl && statusEl.getAttribute('data-redirect'));
                                if (redir) {{
                                    setTimeout(function() {{ window.location.href = redir; }}, 350);
                                }}
                            }} else {{
                                if (statusEl) {{
                                    statusEl.style.display = 'block';
                                    statusEl.style.color = '#ef4444';
                                    statusEl.textContent = payload.message || 'Submission failed. Please check your inputs.';
                                }}
                            }}
                        }} catch (err) {{
                            if (statusEl) {{
                                statusEl.style.display = 'block';
                                statusEl.style.color = '#ef4444';
                                statusEl.textContent = 'Network error submitting form.';
                            }}
                        }}
                        return false;
                    }};
                }}
                if (f && window.dpEvaluateFormLogic) window.dpEvaluateFormLogic(f);
            }})();
            </script>
        </div>"##
    )
}

/// Injects active plugin hooks (`head_html`, `body_top_html`, `footer_html`) into a rendered
/// public SSR HTML document.
pub fn inject_plugin_hooks_into_html(
    db: &Database,
    settings: &DaypressSettingsResponse,
    current_post: Option<&DaypressPostResponse>,
    html: String,
) -> String {
    // AST-rendered bodies never pass through `expand_shortcodes_in_content`, so expand any
    // shortcodes still present in the page's main content region here.
    let html = if current_post.is_some() {
        expand_shortcodes_in_main_region(db, settings, current_post, html)
    } else {
        html
    };
    let mut hooks = render_active_plugin_hooks(db, settings, current_post);

    let all_langs = db.list_languages().unwrap_or_default();
    let default_lang = db.get_default_language_code();
    let active_lang_code = current_post
        .map(|p| p.lang_code.trim())
        .filter(|s| !s.is_empty())
        .unwrap_or(&default_lang);

    // Emit og:locale and og:locale:alternate tags
    if let Some(active_l) = all_langs
        .iter()
        .find(|l| l.code.eq_ignore_ascii_case(active_lang_code))
    {
        let og_loc = if active_l.locale.contains('_') {
            active_l.locale.clone()
        } else {
            format!("{}_{}", active_l.code, active_l.code.to_uppercase())
        };
        hooks.head_html.push_str(&format!(
            "\n\t<meta property=\"og:locale\" content=\"{}\" />",
            crate::templates::utils::escape_html(&og_loc)
        ));
    }

    // Inject multilingual `<link rel="alternate" hreflang="..." href="..." />` tags into `<head>`
    // whenever a post has linked translations or the site has multiple active languages.
    if let Some(post) = current_post {
        if post.id > 0 {
            for hl in db.build_post_hreflang_links(post.id) {
                hooks.head_html.push_str(&format!(
                    "\n\t<link rel=\"alternate\" hreflang=\"{}\" href=\"{}\" />",
                    crate::templates::utils::escape_html(&hl.hreflang),
                    crate::templates::utils::escape_html(&hl.href)
                ));
                if hl.hreflang != "x-default" && !hl.hreflang.eq_ignore_ascii_case(active_lang_code)
                {
                    if let Some(alt_l) = all_langs
                        .iter()
                        .find(|l| l.code.eq_ignore_ascii_case(&hl.hreflang))
                    {
                        hooks.head_html.push_str(&format!(
                            "\n\t<meta property=\"og:locale:alternate\" content=\"{}\" />",
                            crate::templates::utils::escape_html(&alt_l.locale)
                        ));
                    }
                }
            }
        }
    } else if all_langs.len() > 1 {
        let base_url = settings.url.trim_end_matches('/');
        let prefix_def = db.prefix_default_language_enabled();
        for l in &all_langs {
            let href = if l.code.eq_ignore_ascii_case(&default_lang) && !prefix_def {
                format!("{}/", base_url)
            } else {
                format!("{}/{}/", base_url, l.code)
            };
            hooks.head_html.push_str(&format!(
                "\n\t<link rel=\"alternate\" hreflang=\"{}\" href=\"{}\" />",
                crate::templates::utils::escape_html(&l.code),
                crate::templates::utils::escape_html(&href)
            ));
        }
        let xdef = if prefix_def {
            format!("{}/{}/", base_url, default_lang)
        } else {
            format!("{}/", base_url)
        };
        hooks.head_html.push_str(&format!(
            "\n\t<link rel=\"alternate\" hreflang=\"x-default\" href=\"{}\" />",
            crate::templates::utils::escape_html(&xdef)
        ));
    }

    let mut html = html;
    if all_langs
        .iter()
        .any(|l| l.code.eq_ignore_ascii_case(active_lang_code) && l.is_rtl)
    {
        html = html.replacen("dir=\"ltr\"", "dir=\"rtl\"", 1);
    }

    if hooks.head_html.is_empty() && hooks.body_top_html.is_empty() && hooks.footer_html.is_empty()
    {
        return html;
    }

    let mut out = html;
    if !hooks.head_html.is_empty() {
        out = out.replacen("</head>", &format!("{}\n</head>", hooks.head_html), 1);
    }
    if !hooks.body_top_html.is_empty() {
        if let Some(body_idx) = out.find("<body") {
            if let Some(close_idx) = out[body_idx..].find('>') {
                let insert_pos = body_idx + close_idx + 1;
                out.insert_str(insert_pos, &format!("\n{}", hooks.body_top_html));
            }
        }
    }
    if !hooks.footer_html.is_empty() {
        out = out.replacen("</body>", &format!("{}\n</body>", hooks.footer_html), 1);
    }
    out
}

/// Runs [`expand_shortcodes_in_content`] on the `<main class="day-main">` region only, so
/// shortcode text quoted in `<head>` metadata or the theme header/footer is left alone.
fn expand_shortcodes_in_main_region(
    db: &Database,
    settings: &DaypressSettingsResponse,
    current_post: Option<&DaypressPostResponse>,
    html: String,
) -> String {
    const MAIN_OPEN: &str = "<main class=\"day-main\">";
    let Some(start) = html.find(MAIN_OPEN).map(|i| i + MAIN_OPEN.len()) else {
        return html;
    };
    let Some(len) = html[start..].find("</main>") else {
        return html;
    };
    let region = &html[start..start + len];
    let mut expanded = if region.contains('[') {
        expand_shortcodes_in_content_for_post(region, db, settings, current_post)
    } else {
        region.to_string()
    };

    // Hydrate Daystudio AST `<form ... class="dp-form" data-form-id="X" ...>` blocks that use
    // the generic placeholder (`<div class="dp-form-fields">`) when Form #X exists in SQLite.
    if expanded.contains("class=\"dp-form\" data-form-id=\"")
        && (db.is_plugin_active("daypress-forms")
            || !db.list_forms().unwrap_or_default().is_empty())
    {
        if let Ok(re) = regex::Regex::new(
            r#"(?s)<form[^>]*class="dp-form"[^>]*data-form-id="(\d+)"[^>]*>.*?<div class="dp-form-fields">.*?</form>"#,
        ) {
            expanded = re
                .replace_all(&expanded, |caps: &regex::Captures| {
                    let fid: i64 = caps
                        .get(1)
                        .and_then(|m| m.as_str().parse().ok())
                        .unwrap_or(0);
                    if fid > 0 && db.get_form(fid).ok().flatten().is_some() {
                        render_public_form_html(db, fid)
                    } else {
                        caps.get(0).map(|m| m.as_str()).unwrap_or("").to_string()
                    }
                })
                .into_owned();
        }
    }

    format!("{}{}{}", &html[..start], expanded, &html[start + len..])
}

/// Renders the `[daypress_newsletter]` inline subscription widget.
pub fn render_newsletter_widget_html(
    title: &str,
    subtitle: &str,
    list_name: &str,
    btn_label: &str,
) -> String {
    let t_esc = crate::templates::utils::escape_html(title);
    let sub_esc = crate::templates::utils::escape_html(subtitle);
    let list_esc = crate::templates::utils::escape_html(list_name);
    let btn_esc = crate::templates::utils::escape_html(btn_label);
    format!(
        r##"<section class="dp-newsletter-box" style="margin: 28px 0; padding: 28px; border-radius: 14px; border: 1px solid var(--border, #e4e4e7); background: var(--bg-surface, #fafaf9);">
            <h3 style="margin: 0 0 8px; font-size: 20px; font-weight: 700; color: var(--text-main, #18181b);">{t_esc}</h3>
            <p style="margin: 0 0 18px; font-size: 14.5px; line-height: 1.6; color: var(--text-muted, #52525b);">{sub_esc}</p>
            <form class="dp-newsletter-form" onsubmit="return window.submitDaypressNewsletter(event, this);" style="display: flex; flex-wrap: wrap; gap: 10px; align-items: stretch;">
                <input type="hidden" name="list" value="{list_esc}" />
                <input type="text" name="name" placeholder="Your name (optional)" aria-label="Your name" style="flex: 1 1 160px; padding: 11px 14px; border-radius: 8px; border: 1px solid var(--border, #d4d4d8); font-size: 14px;" />
                <input type="email" name="email" required placeholder="you@example.com" aria-label="Email address" style="flex: 2 1 220px; padding: 11px 14px; border-radius: 8px; border: 1px solid var(--border, #d4d4d8); font-size: 14px;" />
                <button type="submit" style="padding: 11px 22px; border-radius: 8px; border: none; background: var(--accent, #a8401b); color: #ffffff; font-weight: 600; font-size: 14px; cursor: pointer;">{btn_esc}</button>
            </form>
            <div class="dp-newsletter-status" style="display: none; margin-top: 12px; font-size: 13.5px; font-weight: 600;"></div>
            <script>
            if (!window.submitDaypressNewsletter) {{
                window.submitDaypressNewsletter = async function(e, formEl) {{
                    e.preventDefault();
                    var statusEl = formEl.parentElement.querySelector('.dp-newsletter-status');
                    var email = (formEl.querySelector('input[name="email"]').value || '').trim();
                    var nameEl = formEl.querySelector('input[name="name"]');
                    var name = nameEl ? (nameEl.value || '').trim() : '';
                    var listEl = formEl.querySelector('input[name="list"]');
                    var list = listEl ? (listEl.value || 'general') : 'general';
                    try {{
                        var res = await fetch('/wp-json/daypress/v1/newsletters/subscribe', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: JSON.stringify({{ email: email, name: name, lists: [list], source_url: window.location.pathname }})
                        }});
                        var data = await res.json().catch(function() {{ return {{}}; }});
                        if (statusEl) {{
                            statusEl.style.display = 'block';
                            statusEl.style.color = res.ok ? '#15803d' : '#dc2626';
                            statusEl.textContent = data.message || (res.ok ? '✓ Subscribed!' : 'Subscription failed.');
                        }}
                        if (res.ok) {{
                            formEl.reset();
                            if (data.unlocked) window.location.reload();
                        }}
                    }} catch (err) {{
                        if (statusEl) {{
                            statusEl.style.display = 'block';
                            statusEl.style.color = '#dc2626';
                            statusEl.textContent = 'Network error. Please try again.';
                        }}
                    }}
                    return false;
                }};
            }}
            </script>
        </section>"##
    )
}

/// Renders the Member-Gated Post paywall / subscriber unlock card (`subscribers_only` or `members_only`).
pub fn render_member_gate_paywall_html(post_title: &str, gate_mode: &str) -> String {
    let safe_title = crate::templates::utils::escape_html(post_title);
    if gate_mode == "members_only" {
        format!(
            r##"<section class="dp-member-gate-card" data-gate-mode="members_only" style="margin: 32px 0; padding: 36px 28px; border-radius: 16px; border: 2px solid var(--accent, #a8401b); background: linear-gradient(180deg, rgba(168,64,27,0.05) 0%, rgba(250,250,249,0.95) 100%); text-align: center;">
                <div style="display: inline-block; padding: 4px 12px; border-radius: 999px; background: rgba(168,64,27,0.12); color: var(--accent, #a8401b); font-size: 11.5px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; margin-bottom: 12px;">🔒 Members-Only Article</div>
                <h3 style="margin: 0 0 10px; font-size: 22px; font-weight: 800; color: var(--text-main, #18181b);">Sign in to continue reading &ldquo;{safe_title}&rdquo;</h3>
                <p style="margin: 0 auto 22px; max-width: 480px; font-size: 15px; line-height: 1.6; color: var(--text-muted, #52525b);">This post is reserved for registered site members. Sign in to your account to unlock the full article immediately.</p>
                <a href="/login" style="display: inline-block; padding: 12px 26px; border-radius: 8px; background: var(--accent, #a8401b); color: #ffffff; font-weight: 600; font-size: 14.5px; text-decoration: none;">Sign In to Read &rarr;</a>
            </section>"##
        )
    } else {
        format!(
            r##"<section class="dp-member-gate-card" data-gate-mode="subscribers_only" style="margin: 32px 0; padding: 36px 28px; border-radius: 16px; border: 2px solid var(--accent, #a8401b); background: linear-gradient(180deg, rgba(168,64,27,0.05) 0%, rgba(250,250,249,0.95) 100%); text-align: center;">
                <div style="display: inline-block; padding: 4px 12px; border-radius: 999px; background: rgba(168,64,27,0.12); color: var(--accent, #a8401b); font-size: 11.5px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.06em; margin-bottom: 12px;">✨ Subscriber-Exclusive Post</div>
                <h3 style="margin: 0 0 10px; font-size: 22px; font-weight: 800; color: var(--text-main, #18181b);">Subscribe for free to continue reading &ldquo;{safe_title}&rdquo;</h3>
                <p style="margin: 0 auto 22px; max-width: 520px; font-size: 15px; line-height: 1.6; color: var(--text-muted, #52525b);">Enter your email below to unlock the rest of this post and receive future dispatches. Already subscribed? Enter your email to unlock immediately.</p>
                <form class="dp-gate-unlock-form" onsubmit="return window.unlockDaypressSubscriberGate(event, this);" style="display: flex; flex-wrap: wrap; gap: 10px; max-width: 460px; margin: 0 auto; justify-content: center;">
                    <input type="email" name="email" required placeholder="Enter your email address..." aria-label="Email address" style="flex: 1 1 240px; padding: 12px 14px; border-radius: 8px; border: 1px solid var(--border, #d4d4d8); font-size: 14px;" />
                    <button type="submit" style="padding: 12px 22px; border-radius: 8px; border: none; background: var(--accent, #a8401b); color: #ffffff; font-weight: 600; font-size: 14px; cursor: pointer;">Unlock Article &rarr;</button>
                </form>
                <div class="dp-gate-status" style="display: none; margin-top: 12px; font-size: 13.5px; font-weight: 600;"></div>
                <script>
                if (!window.unlockDaypressSubscriberGate) {{
                    window.unlockDaypressSubscriberGate = async function(e, formEl) {{
                        e.preventDefault();
                        var statusEl = formEl.parentElement.querySelector('.dp-gate-status');
                        var email = (formEl.querySelector('input[name="email"]').value || '').trim();
                        try {{
                            var res = await fetch('/wp-json/daypress/v1/newsletters/unlock', {{
                                method: 'POST',
                                headers: {{ 'Content-Type': 'application/json' }},
                                body: JSON.stringify({{ email: email, source_url: window.location.pathname }})
                            }});
                            var data = await res.json().catch(function() {{ return {{}}; }});
                            if (res.ok && data.unlocked) {{
                                window.location.reload();
                                return false;
                            }}
                            if (statusEl) {{
                                statusEl.style.display = 'block';
                                statusEl.style.color = res.ok ? '#15803d' : '#dc2626';
                                statusEl.textContent = data.message || 'Check your inbox to confirm your subscription.';
                            }}
                        }} catch (err) {{
                            if (statusEl) {{
                                statusEl.style.display = 'block';
                                statusEl.style.color = '#dc2626';
                                statusEl.textContent = 'Network error. Please try again.';
                            }}
                        }}
                        return false;
                    }};
                }}
                </script>
            </section>"##
        )
    }
}

/// Returns a complete starter Community Plugin JSON manifest that developers can download and customize.
pub fn starter_plugin_sdk_template() -> PluginManifest {
    PluginManifest {
        slug: "promo-announcement-bar".to_string(),
        name: "Top Promo Announcement Bar".to_string(),
        version: "1.0.0".to_string(),
        author: "Community Developer".to_string(),
        description: "Displays a customizable sticky announcement banner at the top of every public page and provides a [promo_cta] shortcode.".to_string(),
        settings_url: None,
        is_builtin: false,
        settings_schema: vec![
            PluginSettingField {
                key: "banner_text".to_string(),
                label: "Announcement Text".to_string(),
                field_type: "text".to_string(),
                default_value: "🚀 Powered by Daypress Rust CMS — Sub-millisecond SSR!".to_string(),
                description: "Text displayed inside the top announcement bar.".to_string(),
                options: vec![],
            },
            PluginSettingField {
                key: "cta_label".to_string(),
                label: "Button Label".to_string(),
                field_type: "text".to_string(),
                default_value: "Learn More →".to_string(),
                description: "Call-to-action button label.".to_string(),
                options: vec![],
            },
            PluginSettingField {
                key: "cta_url".to_string(),
                label: "Button Link URL".to_string(),
                field_type: "url".to_string(),
                default_value: "/sample-page/".to_string(),
                description: "Destination URL when clicked.".to_string(),
                options: vec![],
            },
            PluginSettingField {
                key: "bg_color".to_string(),
                label: "Banner Background Color".to_string(),
                field_type: "color".to_string(),
                default_value: "#2563eb".to_string(),
                description: "Hex color code for the banner background.".to_string(),
                options: vec![],
            },
        ],
        hooks: PluginHooksManifest {
            head_html: String::new(),
            body_top_html: r#"<div class="dp-community-promo-bar" style="background: {{bg_color}}; color: #ffffff; text-align: center; padding: 8px 16px; font-size: 13px; font-weight: 600; display: flex; justify-content: center; align-items: center; gap: 12px;"><span>{{banner_text}}</span><a href="{{cta_url}}" style="background: rgba(255,255,255,0.2); color: #ffffff; padding: 3px 10px; border-radius: 999px; text-decoration: none; font-size: 12px;">{{cta_label}}</a></div>"#.to_string(),
            content_before_html: String::new(),
            content_after_html: String::new(),
            footer_html: String::new(),
            shortcode: "promo_cta".to_string(),
            shortcode_html: r#"<div style="padding: 16px; border-radius: 8px; background: {{bg_color}}; color: #fff; font-weight: 600; margin: 16px 0;">{{banner_text}} <a href="{{cta_url}}" style="color: #fff; text-decoration: underline; margin-left: 8px;">{{cta_label}}</a></div>"#.to_string(),
            on_post_publish_webhook: String::new(),
            on_form_submit_webhook: String::new(),
        },
    }
}
