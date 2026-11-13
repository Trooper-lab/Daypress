#![allow(unused_imports)]
use super::*;
use crate::auth::{AuthUser, OptionalAuthUser};
use crate::db::Database;
use crate::errors::AppError;
use crate::models::*;
use axum::{
    body::to_bytes,
    extract::{FromRequest, Multipart, Path as AxPath, Query, Request, State},
    http::{header, HeaderMap, StatusCode},
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::io::Cursor;
use std::path::Path;

// --- Native Subsystems: Forms & Webhooks Engine ---

pub(crate) fn interpolate_merge_tags(
    template: &str,
    data: &serde_json::Map<String, Value>,
    form_title: &str,
    entry_id: i64,
    source_url: &str,
) -> String {
    let mut out = template
        .replace("{{form_title}}", form_title)
        .replace("{{entry_id}}", &entry_id.to_string())
        .replace("{{source_url}}", source_url);
    for (k, v) in data {
        let placeholder = format!("{{{{{}}}}}", k);
        let val_str = match v {
            Value::String(s) => s.clone(),
            Value::Null => String::new(),
            other => other.to_string(),
        };
        out = out.replace(&placeholder, &val_str);
    }
    out
}

fn evaluate_condition_match(actual: &str, operator: &str, expected: &str) -> bool {
    let a = actual.trim().to_lowercase();
    let e = expected.trim().to_lowercase();
    match operator.trim() {
        "not_equals" | "!=" => a != e,
        "contains" => a.contains(&e),
        "not_empty" => !a.is_empty(),
        "empty" => a.is_empty(),
        _ => a == e,
    }
}

pub(crate) async fn deliver_form_submission_notifications(
    db: &Database,
    form_rec: &DaypressForm,
    entry_id: i64,
    data_map: &serde_json::Map<String, Value>,
    attachments: &[FormAttachmentMeta],
    source_url: &str,
) -> (String, String, String) {
    let settings: FormSettings = serde_json::from_str(&form_rec.settings_json).unwrap_or_default();

    // 1. Collect target admin recipients (default + matching conditional routes)
    let mut recipients: Vec<String> = Vec::new();
    for part in settings.recipient_email.split(',') {
        let trimmed = part.trim();
        if !trimmed.is_empty() && !recipients.iter().any(|r| r.eq_ignore_ascii_case(trimmed)) {
            recipients.push(trimmed.to_string());
        }
    }
    for route in &settings.conditional_routes {
        let field_val = data_map
            .get(&route.field)
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if evaluate_condition_match(field_val, &route.operator, &route.value) {
            for part in route.recipient_email.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty()
                    && !recipients.iter().any(|r| r.eq_ignore_ascii_case(trimmed))
                {
                    recipients.push(trimmed.to_string());
                }
            }
        }
    }

    let mut email_status = "disabled".to_string();
    let mut email_error = String::new();

    if !recipients.is_empty() {
        let subj_tpl = if settings.email_subject_template.trim().is_empty() {
            "[Daypress Form] New submission: {{form_title}}"
        } else {
            settings.email_subject_template.trim()
        };
        let subject =
            interpolate_merge_tags(subj_tpl, data_map, &form_rec.title, entry_id, source_url);

        let mut summary_rows = Vec::new();
        let mut text_lines = Vec::new();
        for (k, v) in data_map {
            if k.starts_with("_np_") || k.starts_with("_dp_") {
                continue;
            }
            let val_str = v
                .as_str()
                .map(|s| s.to_string())
                .unwrap_or_else(|| v.to_string());
            summary_rows.push(format!(
                "<tr><td style=\"padding:8px 12px;border-bottom:1px solid #e5e7eb;font-weight:600;color:#111827;\">{}</td><td style=\"padding:8px 12px;border-bottom:1px solid #e5e7eb;color:#374151;\">{}</td></tr>",
                crate::templates::utils::escape_html(k),
                crate::templates::utils::escape_html(&val_str)
            ));
            text_lines.push(format!("{}: {}", k, val_str));
        }
        if !attachments.is_empty() {
            let att_names: Vec<String> = attachments
                .iter()
                .map(|a| format!("{} ({} B)", a.filename, a.size_bytes))
                .collect();
            summary_rows.push(format!(
                "<tr><td style=\"padding:8px 12px;border-bottom:1px solid #e5e7eb;font-weight:600;color:#111827;\">Attachments</td><td style=\"padding:8px 12px;border-bottom:1px solid #e5e7eb;color:#374151;\">{}</td></tr>",
                crate::templates::utils::escape_html(&att_names.join(", "))
            ));
        }

        let body_html = format!(
            "<div style=\"font-family:sans-serif;max-width:600px;\"><h3 style=\"margin-bottom:12px;\">New submission for {} (#{})</h3><table style=\"width:100%;border-collapse:collapse;border:1px solid #e5e7eb;\"><tbody>{}</tbody></table></div>",
            crate::templates::utils::escape_html(&form_rec.title),
            entry_id,
            summary_rows.join("")
        );
        let body_text = format!(
            "New submission for {} (#{}):\n{}",
            form_rec.title,
            entry_id,
            text_lines.join("\n")
        );

        email_status = "sent".to_string();
        for rcpt in &recipients {
            let msg = crate::email::EmailMessage {
                to: rcpt.clone(),
                subject: subject.clone(),
                body_html: body_html.clone(),
                body_text: body_text.clone(),
            };
            if let Err(e) = crate::email::send_transactional_email(db, &msg).await {
                email_status = "failed".to_string();
                email_error = e.to_string();
            }
        }
    }

    // 2. Optional User Auto-Responder
    if settings.auto_responder_enabled {
        let email_field_key = if settings.auto_responder_email_field.trim().is_empty() {
            "email"
        } else {
            settings.auto_responder_email_field.trim()
        };
        let user_email = data_map
            .get(email_field_key)
            .or_else(|| data_map.get("email"))
            .and_then(|v| v.as_str())
            .map(str::trim)
            .unwrap_or("");
        if user_email.contains('@') {
            let subj_tpl = if settings.auto_responder_subject.trim().is_empty() {
                "We received your message — {{form_title}}"
            } else {
                settings.auto_responder_subject.trim()
            };
            let body_tpl = if settings.auto_responder_body.trim().is_empty() {
                "Hi,\n\nThank you for contacting us via {{form_title}}. We have received your submission (#{{entry_id}}) and will get back to you shortly."
            } else {
                settings.auto_responder_body.trim()
            };
            let auto_subj =
                interpolate_merge_tags(subj_tpl, data_map, &form_rec.title, entry_id, source_url);
            let auto_text =
                interpolate_merge_tags(body_tpl, data_map, &form_rec.title, entry_id, source_url);
            let auto_html = format!(
                "<div style=\"font-family:sans-serif;line-height:1.6;color:#1f2937;\">{}</div>",
                crate::templates::utils::escape_html(&auto_text).replace('\n', "<br/>")
            );
            let auto_msg = crate::email::EmailMessage {
                to: user_email.to_string(),
                subject: auto_subj,
                body_html: auto_html,
                body_text: auto_text,
            };
            match crate::email::send_transactional_email(db, &auto_msg).await {
                Ok(_) => {
                    if email_status == "disabled" {
                        email_status = "sent".to_string();
                    }
                }
                Err(e) => {
                    email_status = "failed".to_string();
                    if email_error.is_empty() {
                        email_error = format!("Auto-responder failed: {}", e);
                    }
                }
            }
        }
    }

    // 3. Optional Outbound Webhook (validated against SSRF guard before dispatch)
    let mut webhook_status = "none".to_string();
    let wh_url = settings.webhook_url.trim();
    if !wh_url.is_empty() {
        webhook_status = "failed".to_string();
        if Database::validate_webhook_target_url_ssrf(wh_url).is_ok() {
            if let Ok(client) =
                crate::adapters::database::webhooks_shortlinks::build_ssrf_safe_http_client(
                    std::time::Duration::from_secs(6),
                )
            {
                let wh_payload = serde_json::json!({
                    "event": "form.submitted",
                    "form_id": form_rec.id,
                    "form_title": form_rec.title,
                    "entry_id": entry_id,
                    "source_url": source_url,
                    "data": data_map,
                    "attachments": attachments,
                });
                if let Ok(resp) = client.post(wh_url).json(&wh_payload).send().await {
                    if resp.status().is_success() {
                        webhook_status = "sent".to_string();
                    }
                }
            }
        } else {
            webhook_status = "blocked_ssrf".to_string();
        }
    }

    let _ = db.update_form_entry_delivery_status(
        entry_id,
        &email_status,
        &email_error,
        &webhook_status,
    );

    (email_status, email_error, webhook_status)
}

pub(crate) fn extract_country_from_headers(headers: &HeaderMap) -> String {
    for key in ["cf-ipcountry", "x-vercel-ip-country", "x-country-code"] {
        if let Some(val) = headers.get(key).and_then(|v| v.to_str().ok()) {
            let clean = val.trim().to_uppercase();
            if clean.len() == 2 && clean != "XX" {
                return clean;
            }
        }
    }
    if let Some(lang) = headers
        .get(header::ACCEPT_LANGUAGE)
        .and_then(|v| v.to_str().ok())
    {
        if let Some(first_tag) = lang.split(',').next() {
            let tag = first_tag.split(';').next().unwrap_or("").trim();
            if let Some((_, region)) = tag.split_once('-') {
                let r = region.trim().to_uppercase();
                if r.len() == 2 && r.chars().all(|c| c.is_ascii_alphabetic()) {
                    return r;
                }
            }
            let primary = tag.to_uppercase();
            if primary.len() == 2 && primary.chars().all(|c| c.is_ascii_alphabetic()) {
                return primary;
            }
        }
    }
    "XX".to_string()
}

pub(crate) async fn submit_form_handler(
    State(db): State<Database>,
    AxPath(id): AxPath<i64>,
    request: Request,
) -> Result<Json<Value>, AppError> {
    let (parts, body) = request.into_parts();
    let content_type = parts
        .headers
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let peer_ip = crate::auth::client_ip(&parts);
    let ip = crate::auth::resolve_client_ip(&parts.headers, peer_ip.as_deref());

    let ua = parts
        .headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");

    let referer_hdr = parts
        .headers
        .get(header::REFERER)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(ip.as_bytes());
    let ip_hash = hex::encode(hasher.finalize());

    let form_rec_opt = db.get_form(id)?;
    let (form_settings, form_fields) = if let Some(ref frec) = form_rec_opt {
        let s: FormSettings = serde_json::from_str(&frec.settings_json).unwrap_or_default();
        let f: Vec<FormFieldSchema> = serde_json::from_str(&frec.schema_json).unwrap_or_default();
        (s, f)
    } else {
        (FormSettings::default(), Vec::new())
    };

    // 1. Per-IP Rate Limiter (HTTP 429 on flood)
    let limit_per_min = if form_settings.rate_limit_per_minute == 0 {
        10
    } else {
        form_settings.rate_limit_per_minute
    };
    if !db
        .form_rate_limiter
        .check_and_record(id, &ip_hash, limit_per_min)
    {
        return Err(AppError::TooManyRequests(
            "Too many form submissions from your IP address. Please wait a minute and try again."
                .to_string(),
        ));
    }

    let bytes = to_bytes(body, 25 * 1024 * 1024)
        .await
        .map_err(|e| AppError::BadRequest(e.to_string()))?;

    let mut data_map = serde_json::Map::new();
    let mut attachments: Vec<FormAttachmentMeta> = Vec::new();

    if content_type.starts_with("multipart/form-data") {
        let req_with_body = Request::from_parts(parts.clone(), axum::body::Body::from(bytes));
        let mut multipart = Multipart::from_request(req_with_body, &db)
            .await
            .map_err(|e| AppError::BadRequest(format!("Multipart form error: {}", e)))?;

        while let Some(field) = multipart
            .next_field()
            .await
            .map_err(|e| AppError::BadRequest(e.to_string()))?
        {
            let field_name = field.name().unwrap_or("field").to_string();
            if let Some(raw_fname) = field.file_name().map(|s| s.to_string()) {
                let mime_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                let file_bytes = field
                    .bytes()
                    .await
                    .map_err(|e| AppError::BadRequest(e.to_string()))?;
                if file_bytes.is_empty() || raw_fname.trim().is_empty() {
                    continue;
                }
                let Some(safe_fname) = crate::media::sanitize_filename(&raw_fname) else {
                    return Err(AppError::BadRequest("Invalid attachment filename.".into()));
                };
                let ext = Path::new(&safe_fname)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                // Block executable/script extensions unconditionally
                if matches!(
                    ext.as_str(),
                    "php"
                        | "phtml"
                        | "exe"
                        | "sh"
                        | "bat"
                        | "cmd"
                        | "js"
                        | "mjs"
                        | "html"
                        | "htm"
                        | "svg"
                ) {
                    return Err(AppError::BadRequest(format!(
                        "File extension .{} is not allowed.",
                        ext
                    )));
                }

                // Check field-level extension allowlist & max MB
                let field_schema = form_fields.iter().find(|f| f.name == field_name);
                let max_mb = field_schema
                    .map(|f| f.max_file_size_mb.max(1))
                    .unwrap_or(10) as usize;
                if file_bytes.len() > max_mb * 1024 * 1024 {
                    return Err(AppError::BadRequest(format!(
                        "File '{}' exceeds the {} MB limit.",
                        safe_fname, max_mb
                    )));
                }
                if let Some(fschema) = field_schema {
                    if !fschema.accept_extensions.trim().is_empty() {
                        let allowed: Vec<String> = fschema
                            .accept_extensions
                            .split(',')
                            .map(|s| s.trim().trim_start_matches('.').to_lowercase())
                            .filter(|s| !s.is_empty())
                            .collect();
                        if !allowed.is_empty() && !allowed.iter().any(|a| a == &ext) {
                            return Err(AppError::BadRequest(format!(
                                "File '{}' must be one of: {}",
                                safe_fname, fschema.accept_extensions
                            )));
                        }
                    }
                }

                let dir = Path::new("data")
                    .join("form-submissions")
                    .join(id.to_string());
                let _ = fs::create_dir_all(&dir);
                let unique_name =
                    format!("{}_{}", chrono::Utc::now().timestamp_millis(), safe_fname);
                let stored_path = dir.join(&unique_name);
                fs::write(&stored_path, &file_bytes)
                    .map_err(|e| AppError::Internal(format!("Failed to store upload: {}", e)))?;

                let stored_path_str = stored_path.to_string_lossy().replace('\\', "/");
                attachments.push(FormAttachmentMeta {
                    field_name: field_name.clone(),
                    filename: unique_name.clone(),
                    original_name: safe_fname.clone(),
                    stored_path: stored_path_str,
                    mime_type,
                    size_bytes: file_bytes.len() as u64,
                });
                data_map.insert(field_name, Value::String(unique_name));
            } else {
                let txt = field.text().await.unwrap_or_default();
                if let Some(existing) = data_map.get_mut(&field_name) {
                    let prev = existing.as_str().unwrap_or("").to_string();
                    *existing = Value::String(if prev.is_empty() {
                        txt
                    } else {
                        format!("{}, {}", prev, txt)
                    });
                } else {
                    data_map.insert(field_name, Value::String(txt));
                }
            }
        }
    } else {
        let raw_val: Value = serde_json::from_slice(&bytes)
            .map_err(|e| AppError::BadRequest(format!("Invalid form submission: {}", e)))?;
        let payload_data = match raw_val.get("data") {
            Some(d) if d.is_object() => d.clone(),
            _ => raw_val,
        };
        if let Some(obj) = payload_data.as_object() {
            for (k, v) in obj {
                data_map.insert(k.clone(), v.clone());
            }
        }
    }

    // 2. Optional Turnstile / hCaptcha Verification (HTTP 400 on invalid token)
    let provider = form_settings.captcha_provider.trim().to_lowercase();
    if (provider == "turnstile" || provider == "hcaptcha")
        && !form_settings.captcha_secret_key.trim().is_empty()
    {
        let token = data_map
            .get("cf-turnstile-response")
            .or_else(|| data_map.get("h-captcha-response"))
            .or_else(|| data_map.get("_captcha_token"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();

        if token.is_empty() {
            return Err(AppError::BadRequest(
                "Captcha verification token is required.".to_string(),
            ));
        }

        let secret = form_settings.captcha_secret_key.trim();
        let is_test_bypass = secret.starts_with("test_") && token == "PASS_TEST_TOKEN";
        if !is_test_bypass {
            let verify_url = if provider == "turnstile" {
                "https://challenges.cloudflare.com/turnstile/v0/siteverify"
            } else {
                "https://api.hcaptcha.com/siteverify"
            };
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|e| AppError::Internal(e.to_string()))?;
            let resp = client
                .post(verify_url)
                .header(
                    reqwest::header::CONTENT_TYPE,
                    "application/x-www-form-urlencoded",
                )
                .body(format!("secret={}&response={}", secret, token.as_str()))
                .send()
                .await
                .map_err(|_| AppError::BadRequest("Captcha verification request failed.".into()))?;
            let body_json: Value = resp.json().await.unwrap_or_default();
            if body_json.get("success").and_then(|v| v.as_bool()) != Some(true) {
                return Err(AppError::BadRequest(
                    "Captcha verification failed.".to_string(),
                ));
            }
        }
    }

    // 3. Anti-Spam Honeypot (`_np_hp`) & Minimum Submit Timestamp Trap (`_dp_ts`)
    // Quarantined into `status = "spam"` in SQLite so false positives are never lost!
    let mut is_spam = false;
    if let Some(hp) = data_map.get("_np_hp").and_then(|v| v.as_str()) {
        if !hp.trim().is_empty() {
            is_spam = true;
        }
    }
    if !is_spam && form_settings.min_submit_seconds > 0 {
        if let Some(ts_val) = data_map.get("_dp_ts") {
            let rendered_ms = ts_val
                .as_i64()
                .or_else(|| ts_val.as_str().and_then(|s| s.trim().parse::<i64>().ok()))
                .unwrap_or(0);
            if rendered_ms > 0 {
                // Support both seconds and milliseconds timestamps
                let normalized_ms = if rendered_ms < 10_000_000_000 {
                    rendered_ms * 1000
                } else {
                    rendered_ms
                };
                let now_ms = chrono::Utc::now().timestamp_millis();
                let elapsed_ms = now_ms - normalized_ms;
                if elapsed_ms >= 0 && elapsed_ms < (form_settings.min_submit_seconds as i64 * 1000)
                {
                    is_spam = true;
                }
            }
        }
    }

    let source_url = data_map
        .get("_source_url")
        .and_then(|v| v.as_str())
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
        .unwrap_or(referer_hdr);

    // Clean internal verification tokens before storing in data_json
    let mut clean_map = data_map.clone();
    clean_map.remove("_np_hp");
    clean_map.remove("_dp_ts");
    clean_map.remove("cf-turnstile-response");
    clean_map.remove("h-captcha-response");
    clean_map.remove("_captcha_token");

    let data_str =
        serde_json::to_string(&Value::Object(clean_map.clone())).unwrap_or_else(|_| "{}".into());
    let attachments_str = serde_json::to_string(&attachments).unwrap_or_else(|_| "[]".to_string());

    if is_spam {
        let _ = db.submit_form_entry_full(
            id,
            &data_str,
            &attachments_str,
            "spam",
            "disabled",
            "none",
            &source_url,
            &ip_hash,
            ua,
        )?;
        return Err(AppError::BadRequest(
            "Spam submission detected and quarantined.".to_string(),
        ));
    }

    let has_email_targets = !form_settings.recipient_email.trim().is_empty()
        || !form_settings.conditional_routes.is_empty()
        || form_settings.auto_responder_enabled;
    let initial_email_status = if has_email_targets {
        "queued"
    } else {
        "disabled"
    };
    let initial_webhook_status = if !form_settings.webhook_url.trim().is_empty() {
        "queued"
    } else {
        "none"
    };

    // 4. Fail-Safe SQLite Vault Commit BEFORE SMTP or Webhook dispatch
    let entry_id = db.submit_form_entry_full(
        id,
        &data_str,
        &attachments_str,
        "unread",
        initial_email_status,
        initial_webhook_status,
        &source_url,
        &ip_hash,
        ua,
    )?;

    // 5. Record Cookieless Analytics Conversion (`form_submit`)
    let visitor_hash = db.compute_daily_visitor_hash_from_parts(&ip, ua);
    let (browser, os, fallback_device) =
        crate::adapters::database::analytics::parse_ua_telemetry(ua);
    let country = extract_country_from_headers(&parts.headers);
    let utm_source = clean_map.get("_utm_source").and_then(|v| v.as_str());
    let utm_medium = clean_map.get("_utm_medium").and_then(|v| v.as_str());
    let utm_campaign = clean_map.get("_utm_campaign").and_then(|v| v.as_str());
    let req_host = parts
        .headers
        .get(header::HOST)
        .and_then(|v| v.to_str().ok());
    let form_title_for_goal = form_rec_opt
        .as_ref()
        .map(|f| f.title.as_str())
        .unwrap_or("Form");
    let _ = db.record_analytics_beacon(
        &visitor_hash,
        if source_url.is_empty() {
            "/"
        } else {
            &source_url
        },
        "",
        req_host,
        fallback_device,
        browser,
        os,
        &country,
        utm_source,
        utm_medium,
        utm_campaign,
        "form_submit",
        Some(&format!("Form: {}", form_title_for_goal)),
        None,
        None,
    );

    // 6. Dispatch Admin Notifications, User Auto-Responder, and Outbound Webhook
    let _ = db.dispatch_outbound_webhook_event(
        "form.submitted",
        serde_json::json!({
            "form_id": id,
            "form_title": form_title_for_goal,
            "entry_id": entry_id,
            "source_url": source_url,
            "fields": clean_map,
        }),
    );

    if let Some(form_rec) = form_rec_opt {
        let (email_status, _email_error, webhook_status) = deliver_form_submission_notifications(
            &db,
            &form_rec,
            entry_id,
            &clean_map,
            &attachments,
            &source_url,
        )
        .await;

        return Ok(Json(serde_json::json!({
            "success": true,
            "entry_id": entry_id,
            "status": "unread",
            "email_status": email_status,
            "webhook_status": webhook_status,
            "message": form_settings.success_message,
            "redirect_url": form_settings.redirect_url
        })));
    }

    Ok(Json(serde_json::json!({
        "success": true,
        "entry_id": entry_id,
        "status": "unread",
        "message": "Form submission received successfully."
    })))
}
