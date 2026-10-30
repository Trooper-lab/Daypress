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
            name: "Daypress SEO & Redirection Suite".to_string(),
            version: "2.5.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Full Yoast & RankMath parity: Per-post SEO title/description/focus-keyphrase analyzer, Schema.org JSON-LD graph, OpenGraph cards, 301/302 Redirect Manager, and 404 Monitor.".to_string(),
            settings_url: Some("/d-admin/seo".to_string()),
            is_builtin: true,
            // Global SEO settings live in `options` and are edited on /d-admin/seo
            // (see `Database::save_global_seo_settings`), so no duplicate plugin-config schema here.
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "daypress-forms".to_string(),
            name: "Daypress Forms Studio & Lead Inbox".to_string(),
            version: "2.2.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Full Contact Form 7, WPForms & Ninja Forms parity: Declarative form builder, honeypot anti-spam protection, SQLite submissions inbox, CSV export, email notifications, and webhook triggers.".to_string(),
            settings_url: Some("/d-admin/forms".to_string()),
            is_builtin: true,
            // Per-form settings (notifications, honeypot, webhooks) are stored on each form
            // in `daypress_forms.settings_json` and edited on /d-admin/forms.
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "daypress-backups".to_string(),
            name: "Snapshot & Hot Backup Manager".to_string(),
            version: "1.8.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "UpdraftPlus & All-in-One WP Migration parity: Zero-downtime SQLite VACUUM INTO hot snapshots, 1-click point-in-time restore, and full JSON/WXR archive downloads.".to_string(),
            settings_url: Some("/d-admin/backups".to_string()),
            is_builtin: true,
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "whatsapp-chat".to_string(),
            name: "Floating WhatsApp & Lead Chat Button".to_string(),
            version: "1.5.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Joinchat & Click-to-Chat parity: Zero-JS floating WhatsApp chat launcher with customizable phone number, welcome message, CTA tooltip badge, and corner placement.".to_string(),
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
            name: "WPCode Header, Footer & Custom Snippets".to_string(),
            version: "1.4.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "WPCode & Insert Headers and Footers parity: Safely inject custom HTML, CSS, tracking pixels (GTM, Meta Pixel, Plausible), or scripts into <head>, <body>, or <footer>.".to_string(),
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
            name: "Daypress WAL Optimizer".to_string(),
            version: "1.0.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "Automated Write-Ahead-Log checkpointing: while active, runs PRAGMA wal_checkpoint(TRUNCATE) every 5 minutes to keep the -wal file small and reads fast.".to_string(),
            settings_url: Some("/d-admin/daypress-engine".to_string()),
            is_builtin: true,
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "webp-image-engine".to_string(),
            name: "WebP Image Engine".to_string(),
            version: "2.4.1".to_string(),
            author: "ImageRS Engine".to_string(),
            description: "On-the-fly dynamic image dimension scaling, auto-crop matrix, and WebP transcoding.".to_string(),
            settings_url: Some("/d-admin/options-media".to_string()),
            is_builtin: true,
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "wxr-streaming-xml-importer".to_string(),
            name: "WXR Streaming XML & Plugin Convergence Importer".to_string(),
            version: "2.0.0".to_string(),
            author: "Daypress Core".to_string(),
            description: "High-throughput WordPress XML & 1-Click JSON importer that automatically converges Yoast, RankMath, Contact Form 7, WPForms, Redirection, and Joinchat settings.".to_string(),
            settings_url: Some("/d-admin/import".to_string()),
            is_builtin: true,
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
    ]
}

/// Built-in modules that are compiled into the server and cannot be switched off
/// (the code paths they describe — hot backups, WebP variants in the media pipeline and the
/// WXR/JSON importer — always run). The admin UI shows them as "Core · Always On" and the
/// toggle API refuses to deactivate them instead of pretending to.
pub const CORE_ALWAYS_ON_PLUGINS: &[&str] = &[
    "daypress-backups",
    "webp-image-engine",
    "wxr-streaming-xml-importer",
];

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
    let installed = db.get_all_installed_plugins().unwrap_or_default();

    for record in installed.into_iter().filter(|r| r.active) {
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
    let mut out = content.to_string();
    // Form shortcodes are provided by the built-in "daypress-forms" plugin; like WordPress
    // with Contact Form 7 deactivated, the raw shortcode is left untouched when it is inactive.
    let forms_active = out.contains('[') && db.is_plugin_active("daypress-forms");

    // 1. Expand [daypress_form id="X"] (or a bare [daypress_form] -> the first form)
    if forms_active && out.contains("[daypress_form") {
        if let Ok(re) = regex::Regex::new(r#"\[daypress_form(?:\s+id=["']?(\d+)["']?)?[^\]]*\]"#) {
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
    }

    // 2. Also expand any un-converged [contact-form-7 ...] or [wpforms ...] or [ninja_forms ...] shortcodes on the fly
    for pattern in [
        r#"\[contact-form-7[^\]]*\]"#,
        r#"\[wpforms[^\]]*\]"#,
        r#"\[ninja_forms[^\]]*\]"#,
    ] {
        if !forms_active {
            break;
        }
        if let Ok(re) = regex::Regex::new(pattern) {
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
    }

    // 3. Community plugin custom shortcodes
    if out.contains('[') {
        if let Ok(plugins) = db.get_all_installed_plugins() {
            for p in plugins.into_iter().filter(|r| r.active) {
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
                    options: vec![],
                },
                crate::models::FormFieldSchema {
                    name: "your_email".to_string(),
                    label: "Your Email".to_string(),
                    field_type: "email".to_string(),
                    required: true,
                    placeholder: "jane@example.com".to_string(),
                    options: vec![],
                },
                crate::models::FormFieldSchema {
                    name: "your_message".to_string(),
                    label: "Message".to_string(),
                    field_type: "textarea".to_string(),
                    required: true,
                    placeholder: "How can we help you?".to_string(),
                    options: vec![],
                },
            ],
            crate::models::FormSettings::default(),
        ),
    };

    let mut fields_html = String::new();
    for field in &fields {
        let req_attr = if field.required { "required" } else { "" };
        let req_star = if field.required {
            r#" <span style="color: #ef4444;">*</span>"#
        } else {
            ""
        };
        let fname = crate::templates::utils::escape_html(&field.name);
        let flabel = crate::templates::utils::escape_html(&field.label);
        let fph = crate::templates::utils::escape_html(&field.placeholder);

        let input_el = match field.field_type.as_str() {
            "textarea" => format!(
                r#"<textarea name="{fname}" rows="4" placeholder="{fph}" {req_attr} style="width: 100%; padding: 10px 14px; border-radius: 8px; border: 1px solid var(--border, #334155); background: var(--bg-base, #090d16); color: var(--text-main, #f8fafc); font-family: inherit; font-size: 14px;"></textarea>"#
            ),
            "select" => {
                let mut opts = String::new();
                for o in &field.options {
                    let o_esc = crate::templates::utils::escape_html(o);
                    opts.push_str(&format!(r#"<option value="{o_esc}">{o_esc}</option>"#));
                }
                format!(
                    r#"<select name="{fname}" {req_attr} style="width: 100%; padding: 10px 14px; border-radius: 8px; border: 1px solid var(--border, #334155); background: var(--bg-base, #090d16); color: var(--text-main, #f8fafc); font-size: 14px;">{opts}</select>"#
                )
            }
            "checkbox" => format!(
                r#"<label style="display: flex; align-items: center; gap: 8px; cursor: pointer;"><input type="checkbox" name="{fname}" value="yes" {req_attr} /> <span>{flabel}</span></label>"#
            ),
            other => {
                let itype = match other {
                    "email" | "tel" | "url" | "number" | "date" => other,
                    _ => "text",
                };
                format!(
                    r#"<input type="{itype}" name="{fname}" placeholder="{fph}" {req_attr} style="width: 100%; padding: 10px 14px; border-radius: 8px; border: 1px solid var(--border, #334155); background: var(--bg-base, #090d16); color: var(--text-main, #f8fafc); font-size: 14px;" />"#
                )
            }
        };

        if field.field_type == "checkbox" {
            fields_html.push_str(&format!(
                r#"<div style="margin-bottom: 16px;">{input_el}</div>"#
            ));
        } else {
            fields_html.push_str(&format!(
                r#"<div style="margin-bottom: 16px;">
                    <label style="display: block; font-size: 13px; font-weight: 600; margin-bottom: 6px; color: var(--text-main, #f8fafc);">{flabel}{req_star}</label>
                    {input_el}
                </div>"#
            ));
        }
    }

    let submit_text = crate::templates::utils::escape_html(&form_settings.submit_label);
    let success_msg = crate::templates::utils::escape_html(&form_settings.success_message);
    let title_esc = crate::templates::utils::escape_html(&title);

    format!(
        r##"<div class="dp-native-form-card" style="background: var(--bg-card, #111827); border: 1px solid var(--border, #1e293b); border-radius: 12px; padding: 28px; margin: 24px 0;">
            <h3 style="font-size: 18px; font-weight: 700; margin: 0 0 18px 0; color: var(--text-main, #f8fafc);">{title_esc}</h3>
            <form id="dp-form-{id}" onsubmit="return submitDaypressForm(event, {id})">
                <div style="display:none;" aria-hidden="true">
                    <input type="text" name="_np_hp" value="" tabindex="-1" autocomplete="off" />
                </div>
                {fields_html}
                <button type="submit" class="dp-button" style="cursor: pointer; border: none; padding: 11px 24px; border-radius: 8px; background: var(--accent, #2563eb); color: #ffffff; font-weight: 700; font-size: 14px;">{submit_text}</button>
                <div id="dp-form-status-{id}" style="margin-top: 12px; font-size: 13.5px; font-weight: 600; display: none;" data-success="{success_msg}"></div>
            </form>
            <script>
            if (!window.submitDaypressForm) {{
                window.submitDaypressForm = async function(e, formId) {{
                    e.preventDefault();
                    const form = e.target;
                    const statusEl = document.getElementById('dp-form-status-' + formId);
                    const formData = new FormData(form);
                    const data = {{}};
                    formData.forEach((v, k) => {{ data[k] = v; }});
                    try {{
                        const res = await fetch('/wp-json/daypress/v1/forms/' + formId + '/submit', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: JSON.stringify({{ data }})
                        }});
                        if (res.ok) {{
                            form.reset();
                            if (statusEl) {{
                                statusEl.style.display = 'block';
                                statusEl.style.color = '#10b981';
                                statusEl.textContent = '✓ ' + (statusEl.getAttribute('data-success') || 'Submitted successfully!');
                            }}
                        }} else {{
                            if (statusEl) {{
                                statusEl.style.display = 'block';
                                statusEl.style.color = '#ef4444';
                                statusEl.textContent = 'Submission failed. Please check your inputs.';
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
        expand_shortcodes_in_main_region(db, settings, html)
    } else {
        html
    };
    let hooks = render_active_plugin_hooks(db, settings, current_post);
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
    if !region.contains('[') {
        return html;
    }
    let expanded = expand_shortcodes_in_content(region, db, settings);
    format!("{}{}{}", &html[..start], expanded, &html[start + len..])
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
