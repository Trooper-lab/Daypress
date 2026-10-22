use crate::db::Database;
use crate::models::{
    PluginHooksManifest, PluginManifest, PluginSettingField, WpPostResponse,
    WpSettingsResponse,
};
use std::collections::HashMap;

/// Returns the catalog of built-in Neopress first-party plugins.
pub fn builtin_plugins_catalog() -> Vec<PluginManifest> {
    vec![
        PluginManifest {
            slug: "neopress-seo".to_string(),
            name: "Neopress SEO & Redirection Suite".to_string(),
            version: "2.5.0".to_string(),
            author: "Neopress Core".to_string(),
            description: "Full Yoast & RankMath parity: Per-post SEO title/description/focus-keyphrase analyzer, Schema.org JSON-LD graph, OpenGraph cards, 301/302 Redirect Manager, and 404 Monitor.".to_string(),
            settings_url: Some("/n-admin/seo.php".to_string()),
            is_builtin: true,
            settings_schema: vec![
                PluginSettingField {
                    key: "title_separator".to_string(),
                    label: "Title Separator".to_string(),
                    field_type: "text".to_string(),
                    default_value: "–".to_string(),
                    description: "Character placed between post title and site name (e.g. –, |, •).".to_string(),
                    options: vec![],
                },
                PluginSettingField {
                    key: "org_name".to_string(),
                    label: "Knowledge Graph Organization Name".to_string(),
                    field_type: "text".to_string(),
                    default_value: "".to_string(),
                    description: "Used in Schema.org Publisher structured data.".to_string(),
                    options: vec![],
                },
                PluginSettingField {
                    key: "twitter_site".to_string(),
                    label: "Twitter / X Handle".to_string(),
                    field_type: "text".to_string(),
                    default_value: "@neopress".to_string(),
                    description: "Used in twitter:site card metadata.".to_string(),
                    options: vec![],
                },
            ],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "neopress-forms".to_string(),
            name: "Neopress Forms Studio & Lead Inbox".to_string(),
            version: "2.2.0".to_string(),
            author: "Neopress Core".to_string(),
            description: "Full Contact Form 7, WPForms & Ninja Forms parity: Declarative form builder, honeypot anti-spam protection, SQLite submissions inbox, CSV export, email notifications, and webhook triggers.".to_string(),
            settings_url: Some("/n-admin/forms.php".to_string()),
            is_builtin: true,
            settings_schema: vec![
                PluginSettingField {
                    key: "enable_honeypot".to_string(),
                    label: "Enable Honeypot Anti-Spam".to_string(),
                    field_type: "boolean".to_string(),
                    default_value: "true".to_string(),
                    description: "Blocks automated spam bots with zero CAPTCHA friction.".to_string(),
                    options: vec![],
                },
                PluginSettingField {
                    key: "default_notify_email".to_string(),
                    label: "Default Notification Email".to_string(),
                    field_type: "text".to_string(),
                    default_value: "admin@example.com".to_string(),
                    description: "Fallback email address for form submission alerts.".to_string(),
                    options: vec![],
                },
            ],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "neopress-backups".to_string(),
            name: "Snapshot & Hot Backup Manager".to_string(),
            version: "1.8.0".to_string(),
            author: "Neopress Core".to_string(),
            description: "UpdraftPlus & All-in-One WP Migration parity: Zero-downtime SQLite VACUUM INTO hot snapshots, 1-click point-in-time restore, and full JSON/WXR archive downloads.".to_string(),
            settings_url: Some("/n-admin/backups.php".to_string()),
            is_builtin: true,
            settings_schema: vec![
                PluginSettingField {
                    key: "max_snapshots".to_string(),
                    label: "Maximum Retained Snapshots".to_string(),
                    field_type: "number".to_string(),
                    default_value: "10".to_string(),
                    description: "Automatically prune oldest snapshots when limit is exceeded.".to_string(),
                    options: vec![],
                },
            ],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "whatsapp-chat".to_string(),
            name: "Floating WhatsApp & Lead Chat Button".to_string(),
            version: "1.5.0".to_string(),
            author: "Neopress Core".to_string(),
            description: "Joinchat & Click-to-Chat parity: Zero-JS floating WhatsApp chat launcher with customizable phone number, welcome message, CTA tooltip badge, and corner placement.".to_string(),
            settings_url: Some("/n-admin/plugin-settings.php?plugin=whatsapp-chat".to_string()),
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
            author: "Neopress Core".to_string(),
            description: "WPCode & Insert Headers and Footers parity: Safely inject custom HTML, CSS, tracking pixels (GTM, Meta Pixel, Plausible), or scripts into <head>, <body>, or <footer>.".to_string(),
            settings_url: Some("/n-admin/plugin-settings.php?plugin=code-snippets".to_string()),
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
            slug: "neopress-wal-optimizer".to_string(),
            name: "Neopress WAL Optimizer".to_string(),
            version: "1.0.0".to_string(),
            author: "Neopress Core".to_string(),
            description: "Automated Write-Ahead-Log checkpointing and SQLite B-tree compaction for sub-millisecond IOPS.".to_string(),
            settings_url: Some("/n-admin/neopress-engine.php".to_string()),
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
            settings_url: Some("/n-admin/options-media.php".to_string()),
            is_builtin: true,
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
        PluginManifest {
            slug: "wxr-streaming-xml-importer".to_string(),
            name: "WXR Streaming XML & Plugin Convergence Importer".to_string(),
            version: "2.0.0".to_string(),
            author: "Neopress Core".to_string(),
            description: "High-throughput WordPress XML & 1-Click JSON importer that automatically converges Yoast, RankMath, Contact Form 7, WPForms, Redirection, and Joinchat settings.".to_string(),
            settings_url: Some("/n-admin/import.php".to_string()),
            is_builtin: true,
            settings_schema: vec![],
            hooks: PluginHooksManifest::default(),
        },
    ]
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
    settings: &WpSettingsResponse,
    current_post: Option<&WpPostResponse>,
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
    settings: &WpSettingsResponse,
    current_post: Option<&WpPostResponse>,
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
<!-- Neopress Native WhatsApp Chat Widget -->
<div id="np-whatsapp-widget" style="position: fixed; bottom: 24px; {pos_css} z-index: 9998; display: flex; align-items: center; gap: 10px; font-family: var(--font-sans, sans-serif);">
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

/// Renders any `[neopress_form id="X"]` shortcodes or community plugin shortcodes inside post content.
pub fn expand_shortcodes_in_content(
    content: &str,
    db: &Database,
    settings: &WpSettingsResponse,
) -> String {
    let mut out = content.to_string();

    // 1. Expand [neopress_form id="X"]
    if out.contains("[neopress_form") {
        if let Ok(re) = regex::Regex::new(r#"\[neopress_form\s+id=["']?(\d+)["']?[^\]]*\]"#) {
            out = re
                .replace_all(&out, |caps: &regex::Captures| {
                    let form_id: i64 = caps
                        .get(1)
                        .and_then(|m| m.as_str().parse().ok())
                        .unwrap_or(1);
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
                        let rendered = interpolate_template(
                            &p.manifest.hooks.shortcode_html,
                            &p.config,
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

/// Renders an interactive HTML5 form from `wp_neopress_forms` wired to `/wp-json/neopress/v1/forms/{id}/submit`.
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
        r##"<div class="np-native-form-card" style="background: var(--bg-card, #111827); border: 1px solid var(--border, #1e293b); border-radius: 12px; padding: 28px; margin: 24px 0;">
            <h3 style="font-size: 18px; font-weight: 700; margin: 0 0 18px 0; color: var(--text-main, #f8fafc);">{title_esc}</h3>
            <form id="np-form-{id}" onsubmit="return submitNeopressForm(event, {id})">
                <div style="display:none;" aria-hidden="true">
                    <input type="text" name="_np_hp" value="" tabindex="-1" autocomplete="off" />
                </div>
                {fields_html}
                <button type="submit" class="np-button" style="cursor: pointer; border: none; padding: 11px 24px; border-radius: 8px; background: var(--accent, #2563eb); color: #ffffff; font-weight: 700; font-size: 14px;">{submit_text}</button>
                <div id="np-form-status-{id}" style="margin-top: 12px; font-size: 13.5px; font-weight: 600; display: none;" data-success="{success_msg}"></div>
            </form>
            <script>
            if (!window.submitNeopressForm) {{
                window.submitNeopressForm = async function(e, formId) {{
                    e.preventDefault();
                    const form = e.target;
                    const statusEl = document.getElementById('np-form-status-' + formId);
                    const formData = new FormData(form);
                    const data = {{}};
                    formData.forEach((v, k) => {{ data[k] = v; }});
                    try {{
                        const res = await fetch('/wp-json/neopress/v1/forms/' + formId + '/submit', {{
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
    settings: &WpSettingsResponse,
    current_post: Option<&WpPostResponse>,
    html: String,
) -> String {
    let hooks = render_active_plugin_hooks(db, settings, current_post);
    if hooks.head_html.is_empty()
        && hooks.body_top_html.is_empty()
        && hooks.footer_html.is_empty()
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
                default_value: "🚀 Powered by Neopress Rust CMS — Sub-millisecond SSR!".to_string(),
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
            body_top_html: r#"<div class="np-community-promo-bar" style="background: {{bg_color}}; color: #ffffff; text-align: center; padding: 8px 16px; font-size: 13px; font-weight: 600; display: flex; justify-content: center; align-items: center; gap: 12px;"><span>{{banner_text}}</span><a href="{{cta_url}}" style="background: rgba(255,255,255,0.2); color: #ffffff; padding: 3px 10px; border-radius: 999px; text-decoration: none; font-size: 12px;">{{cta_label}}</a></div>"#.to_string(),
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
