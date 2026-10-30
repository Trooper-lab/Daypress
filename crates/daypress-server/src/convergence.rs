//! Universal WordPress Plugin Convergence Engine (`crates/daypress-server/src/convergence.rs`)
//!
//! Automatically detects and translates data and settings from the most popular WordPress
//! plugins (Yoast SEO, RankMath, AIOSEO, SEOPress, Contact Form 7, WPForms, Ninja Forms,
//! Redirection, Simple 301 Redirects, Joinchat, Click to Chat, WPCode, Insert Headers & Footers,
//! and WP Mail SMTP) into canonical Daypress tables (`postmeta`, `daypress_forms`,
//! `daypress_redirects`, `daypress_plugins`, and `options`) during 1-Click JSON
//! and WXR XML migrations.

use std::collections::HashMap;

use crate::models::{FormFieldSchema, FormSettings};

/// Normalizes WordPress SEO template placeholders (Yoast, RankMath, AIOSEO, SEOPress)
/// into concrete or clean title/description strings.
pub fn replace_seo_placeholders(raw: &str, post_title: &str, site_title: &str) -> String {
    if raw.trim().is_empty() {
        return String::new();
    }
    let out = raw
        .replace("%%title%%", post_title)
        .replace("%title%", post_title)
        .replace("#post_title", post_title)
        .replace("%%sitename%%", site_title)
        .replace("%sitename%", site_title)
        .replace("#site_title", site_title)
        .replace("%%sep%%", "|")
        .replace("%sep%", "|")
        .replace("#separator_sa", "|")
        .replace("%%page%%", "")
        .replace("%page%", "")
        .replace("%%primary_category%%", "")
        .replace("%primary_category%", "");

    // Collapse duplicate spaces
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Inspects a post's raw `postmeta` map for Yoast SEO, RankMath, All in One SEO (AIOSEO),
/// or SEOPress keys and populates canonical `_daypress_seo_*` postmeta keys if not already set.
/// Returns `true` if any legacy SEO plugin metadata was detected and converged.
pub fn converge_post_seo_meta(
    postmeta: &mut HashMap<String, String>,
    post_title: &str,
    site_title: &str,
) -> bool {
    let mut converged = false;

    // 1. SEO Title
    if !postmeta.contains_key("_daypress_seo_title") {
        let candidate = postmeta
            .get("_yoast_wpseo_title")
            .or_else(|| postmeta.get("rank_math_title"))
            .or_else(|| postmeta.get("_aioseo_title"))
            .or_else(|| postmeta.get("_seopress_titles_title"))
            .cloned();
        if let Some(val) = candidate {
            let cleaned = replace_seo_placeholders(&val, post_title, site_title);
            if !cleaned.is_empty() {
                postmeta.insert("_daypress_seo_title".to_string(), cleaned);
                converged = true;
            }
        }
    }

    // 2. SEO Meta Description
    if !postmeta.contains_key("_daypress_seo_desc") {
        let candidate = postmeta
            .get("_yoast_wpseo_metadesc")
            .or_else(|| postmeta.get("rank_math_description"))
            .or_else(|| postmeta.get("_aioseo_description"))
            .or_else(|| postmeta.get("_seopress_titles_desc"))
            .cloned();
        if let Some(val) = candidate {
            let cleaned = replace_seo_placeholders(&val, post_title, site_title);
            if !cleaned.is_empty() {
                postmeta.insert("_daypress_seo_desc".to_string(), cleaned);
                converged = true;
            }
        }
    }

    // 3. Focus Keyword
    if !postmeta.contains_key("_daypress_seo_keyword") {
        let candidate = postmeta
            .get("_yoast_wpseo_focuskw")
            .or_else(|| postmeta.get("rank_math_focus_keyword"))
            .or_else(|| postmeta.get("_seopress_analysis_target_kw"))
            .cloned();
        if let Some(val) = candidate {
            // RankMath stores comma-separated keywords; take the primary keyword or full string
            let kw = val.trim().to_string();
            if !kw.is_empty() {
                postmeta.insert("_daypress_seo_keyword".to_string(), kw);
                converged = true;
            }
        }
    }

    // 4. Canonical URL
    if !postmeta.contains_key("_daypress_seo_canonical") {
        let candidate = postmeta
            .get("_yoast_wpseo_canonical")
            .or_else(|| postmeta.get("rank_math_canonical_url"))
            .or_else(|| postmeta.get("_aioseo_canonical_url"))
            .or_else(|| postmeta.get("_seopress_robots_canonical"))
            .cloned();
        if let Some(val) = candidate {
            let url = val.trim().to_string();
            if !url.is_empty() {
                postmeta.insert("_daypress_seo_canonical".to_string(), url);
                converged = true;
            }
        }
    }

    // 5. Robots Noindex
    if !postmeta.contains_key("_daypress_seo_noindex") {
        let yoast_noindex = postmeta
            .get("_yoast_wpseo_meta-robots-noindex")
            .map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("noindex"))
            .unwrap_or(false);
        let rm_noindex = postmeta
            .get("rank_math_robots")
            .map(|v| v.to_ascii_lowercase().contains("noindex"))
            .unwrap_or(false);
        let seopress_noindex = postmeta
            .get("_seopress_robots_index")
            .map(|v| v.trim() == "yes" || v.to_ascii_lowercase().contains("noindex"))
            .unwrap_or(false);
        if yoast_noindex || rm_noindex || seopress_noindex {
            postmeta.insert("_daypress_seo_noindex".to_string(), "1".to_string());
            converged = true;
        }
    }

    // 6. OpenGraph Image
    if !postmeta.contains_key("_daypress_seo_og_image") {
        let candidate = postmeta
            .get("_yoast_wpseo_opengraph-image")
            .or_else(|| postmeta.get("rank_math_facebook_image"))
            .or_else(|| postmeta.get("_aioseo_og_image_custom_url"))
            .or_else(|| postmeta.get("_seopress_social_fb_img"))
            .cloned();
        if let Some(val) = candidate {
            let img = val.trim().to_string();
            if !img.is_empty() {
                postmeta.insert("_daypress_seo_og_image".to_string(), img);
                converged = true;
            }
        }
    }

    converged
}

/// Checks if a WordPress post is a Contact Form 7 (`wpcf7_contact_form`) or WPForms (`wpforms`)
/// custom post type and converts its template/JSON into Daypress's native `(Vec<FormFieldSchema>, FormSettings)`.
pub fn parse_cf7_or_wpforms_to_schema(
    post_type: &str,
    title: &str,
    content: &str,
    postmeta: &HashMap<String, String>,
) -> Option<(Vec<FormFieldSchema>, FormSettings)> {
    match post_type {
        "wpcf7_contact_form" => Some(parse_cf7_form(title, content, postmeta)),
        "wpforms" => Some(parse_wpforms_post(title, content)),
        _ => None,
    }
}

fn parse_cf7_form(
    _title: &str,
    content: &str,
    postmeta: &HashMap<String, String>,
) -> (Vec<FormFieldSchema>, FormSettings) {
    let raw_form = postmeta
        .get("_form")
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.as_str())
        .unwrap_or(content);

    let mut fields = Vec::new();
    let mut submit_label = "Send Message".to_string();

    // Parse CF7 tags such as:
    // [text* your-name placeholder "Your Name"]
    // [email* your-email]
    // [tel your-phone]
    // [textarea* your-message]
    // [select your-topic "Sales" "Support"]
    // [submit "Send Now"]
    let mut rest = raw_form;
    while let Some(start) = rest.find('[') {
        let after_start = &rest[start + 1..];
        let Some(end) = after_start.find(']') else {
            break;
        };
        let tag_body = after_start[..end].trim();
        rest = &after_start[end + 1..];

        if tag_body.is_empty() || tag_body.starts_with('/') {
            continue;
        }

        let tokens = tokenize_cf7_tag(tag_body);
        if tokens.is_empty() {
            continue;
        }

        let raw_type = tokens[0].to_ascii_lowercase();
        let required = raw_type.ends_with('*');
        let base_type = raw_type.trim_end_matches('*');

        if base_type == "submit" {
            if let Some(lbl) = tokens.get(1) {
                let clean = lbl.trim_matches('"').trim();
                if !clean.is_empty() {
                    submit_label = clean.to_string();
                }
            }
            continue;
        }

        let field_type = match base_type {
            "text" => "text",
            "email" => "email",
            "tel" => "tel",
            "url" => "url",
            "number" | "range" => "number",
            "date" => "date",
            "textarea" => "textarea",
            "select" | "radio" => "select",
            "checkbox" | "acceptance" => "checkbox",
            _ => continue,
        };

        let name = tokens
            .get(1)
            .map(|s| s.trim_matches('"').to_string())
            .filter(|s| !s.is_empty() && !s.contains(':'))
            .unwrap_or_else(|| format!("field_{}", fields.len() + 1));

        // Extract placeholder or quoted strings
        let mut placeholder = String::new();
        let mut options = Vec::new();
        let mut idx = 2;
        while idx < tokens.len() {
            let tok = &tokens[idx];
            if tok.eq_ignore_ascii_case("placeholder") {
                if let Some(next_tok) = tokens.get(idx + 1) {
                    placeholder = next_tok.trim_matches('"').to_string();
                    idx += 2;
                    continue;
                }
            } else if tok.starts_with('"') && tok.ends_with('"') {
                let unquoted = tok.trim_matches('"').to_string();
                if field_type == "select" {
                    options.push(unquoted);
                } else if placeholder.is_empty() {
                    placeholder = unquoted;
                }
            }
            idx += 1;
        }

        let label = humanize_field_name(&name);
        fields.push(FormFieldSchema {
            name,
            label,
            field_type: field_type.to_string(),
            required,
            placeholder,
            options,
        });
    }

    if fields.is_empty() {
        fields = default_contact_fields();
    }

    let mut settings = FormSettings {
        submit_label,
        ..FormSettings::default()
    };

    // Inspect `_mail` meta if JSON or PHP serialized string contains a recipient email
    if let Some(mail_meta) = postmeta.get("_mail") {
        if let Some(email) = extract_email_from_meta(mail_meta) {
            settings.recipient_email = email;
        }
    }

    (fields, settings)
}

fn parse_wpforms_post(_title: &str, content: &str) -> (Vec<FormFieldSchema>, FormSettings) {
    let mut fields = Vec::new();
    let mut settings = FormSettings::default();

    if let Ok(json) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(fields_obj) = json.get("fields").and_then(|v| v.as_object()) {
            for (_id, f) in fields_obj {
                let raw_type = f
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("text")
                    .to_ascii_lowercase();
                let field_type = match raw_type.as_str() {
                    "email" => "email",
                    "textarea" => "textarea",
                    "phone" | "tel" => "tel",
                    "select" | "radio" => "select",
                    "checkbox" | "gdpr-checkbox" => "checkbox",
                    "number" => "number",
                    _ => "text",
                };
                let label = f
                    .get("label")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Field")
                    .to_string();
                let name = slugify_field_name(&label, fields.len() + 1);
                let required = f
                    .get("required")
                    .map(|v| v.as_str() == Some("1") || v.as_bool() == Some(true))
                    .unwrap_or(false);
                let placeholder = f
                    .get("placeholder")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let mut options = Vec::new();
                if let Some(choices) = f.get("choices").and_then(|v| v.as_object()) {
                    for (_cid, choice) in choices {
                        if let Some(lbl) = choice.get("label").and_then(|v| v.as_str()) {
                            options.push(lbl.to_string());
                        }
                    }
                }

                fields.push(FormFieldSchema {
                    name,
                    label,
                    field_type: field_type.to_string(),
                    required,
                    placeholder,
                    options,
                });
            }
        }
        if let Some(submit_text) = json
            .get("settings")
            .and_then(|s| s.get("submit_text"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
        {
            settings.submit_label = submit_text.to_string();
        }
    }

    if fields.is_empty() {
        fields = default_contact_fields();
    }

    (fields, settings)
}

fn tokenize_cf7_tag(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for ch in input.chars() {
        match ch {
            '"' => {
                in_quotes = !in_quotes;
                current.push(ch);
            }
            c if c.is_whitespace() && !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            c => current.push(c),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

fn humanize_field_name(name: &str) -> String {
    let stripped = name
        .strip_prefix("your-")
        .or_else(|| name.strip_prefix("your_"))
        .unwrap_or(name);
    let words: Vec<String> = stripped
        .split(['-', '_'])
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect();
    if words.is_empty() {
        "Field".to_string()
    } else {
        words.join(" ")
    }
}

fn slugify_field_name(label: &str, index: usize) -> String {
    let slug: String = label
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    let trimmed = slug.trim_matches('_');
    if trimmed.is_empty() {
        format!("field_{index}")
    } else {
        trimmed.to_string()
    }
}

fn extract_email_from_meta(raw: &str) -> Option<String> {
    for token in raw.split(['"', '\'', ' ', ';', ',', '\n', '\r']) {
        let trimmed = token.trim();
        if trimmed.contains('@')
            && trimmed.contains('.')
            && !trimmed.starts_with('[')
            && !trimmed.contains("example.com")
        {
            return Some(trimmed.to_string());
        }
    }
    None
}

pub fn default_contact_fields() -> Vec<FormFieldSchema> {
    vec![
        FormFieldSchema {
            name: "name".to_string(),
            label: "Your Name".to_string(),
            field_type: "text".to_string(),
            required: true,
            placeholder: "Jane Doe".to_string(),
            options: vec![],
        },
        FormFieldSchema {
            name: "email".to_string(),
            label: "Email Address".to_string(),
            field_type: "email".to_string(),
            required: true,
            placeholder: "jane@example.com".to_string(),
            options: vec![],
        },
        FormFieldSchema {
            name: "message".to_string(),
            label: "Message".to_string(),
            field_type: "textarea".to_string(),
            required: true,
            placeholder: "How can we help you?".to_string(),
            options: vec![],
        },
    ]
}

/// Rewrites legacy WordPress form shortcodes (`[contact-form-7 id="123" ...]`,
/// `[wpforms id="123"]`, `[ninja_forms id="123"]`, `[gravityform id="123"]`)
/// into native `[daypress_form id="NEW"]` shortcodes using the deterministic
/// `old_form_id -> new_form_id` mapping.
pub fn rewrite_legacy_form_shortcodes(
    content: &str,
    form_id_map: &HashMap<i64, i64>,
    default_form_id: Option<i64>,
) -> String {
    if !content.contains("[contact-form-7")
        && !content.contains("[wpforms")
        && !content.contains("[ninja_forms")
        && !content.contains("[gravityform")
    {
        return content.to_string();
    }

    let prefixes = [
        "[contact-form-7",
        "[wpforms",
        "[ninja_forms",
        "[gravityform",
    ];
    let mut output = content.to_string();

    for prefix in prefixes {
        let mut rebuilt = String::with_capacity(output.len());
        let mut cursor = output.as_str();

        while let Some(start) = cursor.find(prefix) {
            rebuilt.push_str(&cursor[..start]);
            let after_start = &cursor[start..];
            if let Some(end_offset) = after_start.find(']') {
                let full_shortcode = &after_start[..=end_offset];
                let old_id = extract_id_attr_from_shortcode(full_shortcode);
                let resolved_id = old_id
                    .and_then(|id| form_id_map.get(&id).copied())
                    .or(default_form_id)
                    .or(old_id)
                    .unwrap_or(1);
                rebuilt.push_str(&format!("[daypress_form id=\"{resolved_id}\"]"));
                cursor = &after_start[end_offset + 1..];
            } else {
                rebuilt.push_str(after_start);
                cursor = "";
                break;
            }
        }
        rebuilt.push_str(cursor);
        output = rebuilt;
    }

    output
}

fn extract_id_attr_from_shortcode(shortcode: &str) -> Option<i64> {
    let lower = shortcode.to_ascii_lowercase();
    let idx = lower.find("id=")?;
    let after = &shortcode[idx + 3..].trim_start();
    let digits: String = after
        .trim_start_matches(['"', '\''])
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect();
    digits.parse::<i64>().ok()
}

/// Converges global WordPress plugin options from `plugin_options` into Daypress's native
/// `options`, `daypress_plugins`, and `daypress_redirects` tables inside an active
/// SQLite transaction. Returns a human-readable list of converged plugin names.
pub fn converge_plugin_options(
    tx: &rusqlite::Transaction,
    plugin_options: &HashMap<String, serde_json::Value>,
) -> Vec<String> {
    let mut converged_list = Vec::new();

    // 1. Global SEO Options (Yoast `wpseo_titles` / `wpseo_social` or RankMath `rank-math-options-titles`)
    let yoast_titles = plugin_options.get("wpseo_titles");
    let rm_titles = plugin_options.get("rank-math-options-titles");
    if yoast_titles.is_some() || rm_titles.is_some() {
        if let Some(obj) = yoast_titles
            .and_then(|v| v.as_object())
            .or_else(|| rm_titles.and_then(|v| v.as_object()))
        {
            if let Some(sep) = obj
                .get("separator")
                .or_else(|| obj.get("title_separator"))
                .and_then(|v| v.as_str())
            {
                let normalized_sep = match sep {
                    "sc-dash" | "-" => "-",
                    "sc-ndash" | "–" => "–",
                    "sc-mdash" | "—" => "—",
                    "sc-bull" | "•" => "•",
                    "sc-pipe" | "|" => "|",
                    other if !other.starts_with("sc-") => other,
                    _ => "|",
                };
                let _ = upsert_option_tx(tx, "daypress_seo_separator", normalized_sep);
            }
            if let Some(org_name) = obj
                .get("company_name")
                .or_else(|| obj.get("knowledgegraph_name"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
            {
                let _ = upsert_option_tx(tx, "daypress_seo_org_name", org_name.trim());
            }
            if let Some(org_logo) = obj
                .get("company_logo")
                .or_else(|| obj.get("knowledgegraph_logo"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
            {
                let _ = upsert_option_tx(tx, "daypress_seo_org_logo", org_logo.trim());
            }
            if let Some(default_og) = obj
                .get("og_default_image")
                .or_else(|| obj.get("open_graph_image"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
            {
                let _ = upsert_option_tx(tx, "daypress_seo_default_og_image", default_og.trim());
            }
        }
        let _ = activate_builtin_plugin_tx(tx, "daypress-seo", None);
        converged_list.push("Yoast / RankMath SEO -> Daypress SEO Toolkit".to_string());
    }

    // 2. Floating WhatsApp Chat (`joinchat`, `ht_ctc_chat_options`, `qlwapp`)
    let whatsapp_source = plugin_options
        .get("joinchat")
        .or_else(|| plugin_options.get("ht_ctc_chat_options"))
        .or_else(|| plugin_options.get("qlwapp"));
    if let Some(wa_val) = whatsapp_source {
        let mut phone = String::new();
        let mut message = "Hi! I have a question about your website.".to_string();
        let mut label = "Chat on WhatsApp".to_string();
        let mut position = "bottom-right".to_string();

        if let Some(obj) = wa_val.as_object() {
            if let Some(p) = obj
                .get("telephone")
                .or_else(|| obj.get("number"))
                .or_else(|| obj.get("phone"))
                .and_then(|v| v.as_str())
            {
                phone = p
                    .chars()
                    .filter(|c| c.is_ascii_digit() || *c == '+')
                    .collect();
            }
            if let Some(m) = obj
                .get("message_send")
                .or_else(|| obj.get("pre_filled"))
                .or_else(|| obj.get("message"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
            {
                message = m.trim().to_string();
            }
            if let Some(cta) = obj
                .get("button_tip")
                .or_else(|| obj.get("call_to_action"))
                .or_else(|| obj.get("text"))
                .and_then(|v| v.as_str())
                .filter(|s| !s.trim().is_empty())
            {
                label = cta.trim().to_string();
            }
            if let Some(pos) = obj.get("position").and_then(|v| v.as_str()) {
                if pos.eq_ignore_ascii_case("left") || pos.contains("left") {
                    position = "bottom-left".to_string();
                }
            }
        }

        if !phone.is_empty() {
            let mut cfg = HashMap::new();
            cfg.insert("phone_number".to_string(), phone);
            cfg.insert("welcome_message".to_string(), message);
            cfg.insert("tooltip_text".to_string(), label);
            cfg.insert("position".to_string(), position);
            cfg.insert("accent_color".to_string(), "#25D366".to_string());
            let _ = activate_builtin_plugin_tx(tx, "whatsapp-chat", Some(&cfg));
            converged_list.push("WhatsApp Chat Plugin -> Daypress Floating WhatsApp".to_string());
        }
    }

    // 3. Header / Footer Code Snippets (`ihaf_insert_header`, `ihaf_insert_footer`, `ihaf_insert_body`, `wpcode_snippets`)
    let header_code = plugin_options
        .get("ihaf_insert_header")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let body_code = plugin_options
        .get("ihaf_insert_body")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    let footer_code = plugin_options
        .get("ihaf_insert_footer")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();

    if !header_code.is_empty() || !body_code.is_empty() || !footer_code.is_empty() {
        let mut cfg = HashMap::new();
        cfg.insert("head_html".to_string(), header_code);
        cfg.insert("body_top_html".to_string(), body_code);
        cfg.insert("footer_html".to_string(), footer_code);
        let _ = activate_builtin_plugin_tx(tx, "code-snippets", Some(&cfg));
        converged_list
            .push("Insert Headers & Footers / WPCode -> Daypress Code Snippets".to_string());
    }

    // 4. Simple 301 Redirects (`301_redirects`) or RankMath / Redirection rules in `plugin_options`
    if let Some(redirs_obj) = plugin_options
        .get("301_redirects")
        .and_then(|v| v.as_object())
    {
        let mut count = 0usize;
        for (src, dst_val) in redirs_obj {
            if let Some(dst) = dst_val.as_str() {
                let clean_src = src.trim();
                let clean_dst = dst.trim();
                if !clean_src.is_empty() && !clean_dst.is_empty() {
                    let _ = tx.execute(
                        "INSERT INTO daypress_redirects (source_path, target_url, status_code, is_active)
                         VALUES (?1, ?2, 301, 1)
                         ON CONFLICT(source_path) DO UPDATE SET target_url = excluded.target_url",
                        rusqlite::params![clean_src, clean_dst],
                    );
                    count += 1;
                }
            }
        }
        if count > 0 {
            converged_list.push(format!(
                "Simple 301 Redirects ({count} rules) -> Daypress Redirect Manager"
            ));
        }
    }

    // 5. WP Mail SMTP (`wp_mail_smtp`)
    if let Some(smtp_obj) = plugin_options
        .get("wp_mail_smtp")
        .and_then(|v| v.as_object())
    {
        if let Some(mail) = smtp_obj.get("mail").and_then(|v| v.as_object()) {
            if let Some(from_email) = mail.get("from_email").and_then(|v| v.as_str()) {
                let _ = upsert_option_tx(tx, "daypress_smtp_from_email", from_email.trim());
            }
            if let Some(from_name) = mail.get("from_name").and_then(|v| v.as_str()) {
                let _ = upsert_option_tx(tx, "daypress_smtp_from_name", from_name.trim());
            }
        }
        if let Some(smtp) = smtp_obj.get("smtp").and_then(|v| v.as_object()) {
            if let Some(host) = smtp.get("host").and_then(|v| v.as_str()) {
                let _ = upsert_option_tx(tx, "daypress_smtp_host", host.trim());
            }
            if let Some(port) = smtp.get("port") {
                let port_str = port
                    .as_i64()
                    .map(|n| n.to_string())
                    .or_else(|| port.as_str().map(|s| s.to_string()))
                    .unwrap_or_default();
                if !port_str.is_empty() {
                    let _ = upsert_option_tx(tx, "daypress_smtp_port", &port_str);
                }
            }
        }
        converged_list.push("WP Mail SMTP -> Daypress Native Mailer Settings".to_string());
    }

    converged_list
}

fn upsert_option_tx(tx: &rusqlite::Transaction, name: &str, value: &str) -> rusqlite::Result<()> {
    tx.execute(
        "INSERT INTO options (option_name, option_value, autoload) VALUES (?1, ?2, 'yes')
         ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value",
        rusqlite::params![name, value],
    )?;
    Ok(())
}

pub fn activate_builtin_plugin_tx(
    tx: &rusqlite::Transaction,
    slug: &str,
    config_override: Option<&HashMap<String, String>>,
) -> rusqlite::Result<()> {
    let builtins = crate::plugins::builtin_plugins_catalog();
    let Some(manifest) = builtins.into_iter().find(|m| m.slug == slug) else {
        return Ok(());
    };

    let mut default_cfg = HashMap::new();
    for field in &manifest.settings_schema {
        default_cfg.insert(field.key.clone(), field.default_value.clone());
    }
    if let Some(overrides) = config_override {
        for (k, v) in overrides {
            default_cfg.insert(k.clone(), v.clone());
        }
    }

    let manifest_json = serde_json::to_string(&manifest).unwrap_or_else(|_| "{}".to_string());
    let config_json = serde_json::to_string(&default_cfg).unwrap_or_else(|_| "{}".to_string());

    tx.execute(
        "INSERT INTO daypress_plugins (slug, name, version, description, category, is_builtin, is_active, manifest_json, config_json, updated_at)
         VALUES (?1, ?2, ?3, ?4, 'builtin', 1, 1, ?5, ?6, datetime('now'))
         ON CONFLICT(slug) DO UPDATE SET
            is_active = 1,
            config_json = CASE WHEN ?7 = 1 THEN excluded.config_json ELSE daypress_plugins.config_json END,
            updated_at = datetime('now')",
        rusqlite::params![
            manifest.slug,
            manifest.name,
            manifest.version,
            manifest.description,
            manifest_json,
            config_json,
            if config_override.is_some() { 1 } else { 0 },
        ],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_converge_post_seo_meta_yoast_and_rankmath() {
        let mut meta = HashMap::new();
        meta.insert(
            "_yoast_wpseo_title".to_string(),
            "%%title%% %%sep%% %%sitename%%".to_string(),
        );
        meta.insert(
            "_yoast_wpseo_metadesc".to_string(),
            "Best solar panels for %%sitename%% customers.".to_string(),
        );
        meta.insert(
            "rank_math_focus_keyword".to_string(),
            "solar panels".to_string(),
        );
        meta.insert("rank_math_robots".to_string(), "noindex,follow".to_string());

        let changed = converge_post_seo_meta(&mut meta, "Solar Guide", "Acme Solar");
        assert!(changed);
        assert_eq!(
            meta.get("_daypress_seo_title").map(String::as_str),
            Some("Solar Guide | Acme Solar")
        );
        assert_eq!(
            meta.get("_daypress_seo_desc").map(String::as_str),
            Some("Best solar panels for Acme Solar customers.")
        );
        assert_eq!(
            meta.get("_daypress_seo_keyword").map(String::as_str),
            Some("solar panels")
        );
        assert_eq!(
            meta.get("_daypress_seo_noindex").map(String::as_str),
            Some("1")
        );
    }

    #[test]
    fn test_parse_cf7_form_and_rewrite_shortcodes() {
        let mut meta = HashMap::new();
        meta.insert(
            "_form".to_string(),
            "[text* your-name placeholder \"Full Name\"]\n[email* your-email]\n[tel phone-number]\n[textarea* your-message]\n[submit \"Request Quote\"]".to_string(),
        );
        meta.insert(
            "_mail".to_string(),
            "{\"recipient\":\"sales@acme-solar.test\"}".to_string(),
        );

        let (fields, settings) =
            parse_cf7_or_wpforms_to_schema("wpcf7_contact_form", "Quote Form", "", &meta)
                .expect("should parse CF7 post_type");
        assert_eq!(fields.len(), 4);
        assert_eq!(fields[0].name, "your-name");
        assert_eq!(fields[0].label, "Name");
        assert_eq!(fields[0].placeholder, "Full Name");
        assert!(fields[0].required);
        assert_eq!(fields[1].field_type, "email");
        assert_eq!(fields[2].field_type, "tel");
        assert_eq!(fields[3].field_type, "textarea");
        assert_eq!(settings.submit_label, "Request Quote");
        assert_eq!(settings.recipient_email, "sales@acme-solar.test");

        let mut id_map = HashMap::new();
        id_map.insert(42, 7);
        let rewritten = rewrite_legacy_form_shortcodes(
            "<p>Contact us:</p>[contact-form-7 id=\"42\" title=\"Quote Form\"]",
            &id_map,
            Some(1),
        );
        assert_eq!(rewritten, "<p>Contact us:</p>[daypress_form id=\"7\"]");
    }
}
