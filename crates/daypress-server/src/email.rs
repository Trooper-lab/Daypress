use crate::db::Database;
use crate::errors::AppError;
use lettre::message::{header::ContentType, Mailbox, MultiPart, SinglePart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub from_email: String,
    pub from_name: String,
    pub encryption: String, // "tls", "starttls", "none"
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: "localhost".to_string(),
            smtp_port: 587,
            smtp_user: String::new(),
            smtp_pass: String::new(),
            from_email: "noreply@daypress.local".to_string(),
            from_name: "Daypress System".to_string(),
            encryption: "tls".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailMessage {
    pub to: String,
    pub subject: String,
    pub body_html: String,
    #[serde(default)]
    pub body_text: String,
}

fn env_value(name: &str) -> Option<String> {
    crate::legacy::env_var(name).filter(|v| !v.trim().is_empty())
}

/// SMTP settings from the environment, falling back to the options converged from
/// WP Mail SMTP during a migration. Returns `None` when no SMTP host is configured.
///
/// | Variable | Meaning |
/// | --- | --- |
/// | `DAYPRESS_SMTP_HOST` | SMTP server (required to send) |
/// | `DAYPRESS_SMTP_PORT` | Port (default 465 for `tls`, 587 for `starttls`, 25 for `none`) |
/// | `DAYPRESS_SMTP_SECURITY` | `starttls` (default), `tls`, or `none` |
/// | `DAYPRESS_SMTP_USER` / `DAYPRESS_SMTP_PASSWORD` | Credentials, if the server needs them |
/// | `DAYPRESS_SMTP_FROM` / `DAYPRESS_SMTP_FROM_NAME` | Sender (default: admin email / site title) |
pub fn smtp_config(db: &Database) -> Option<EmailConfig> {
    let option = |name: &str| {
        db.get_option(name)
            .ok()
            .flatten()
            .filter(|v| !v.trim().is_empty())
    };
    let smtp_host = env_value("DAYPRESS_SMTP_HOST").or_else(|| option("daypress_smtp_host"))?;
    let encryption = env_value("DAYPRESS_SMTP_SECURITY")
        .map(|v| v.trim().to_lowercase())
        .unwrap_or_else(|| "starttls".to_string());
    let default_port = match encryption.as_str() {
        "tls" | "ssl" => 465,
        "none" => 25,
        _ => 587,
    };
    let smtp_port = env_value("DAYPRESS_SMTP_PORT")
        .or_else(|| option("daypress_smtp_port"))
        .and_then(|p| p.trim().parse().ok())
        .unwrap_or(default_port);
    let settings = db.get_settings().unwrap_or_default();
    let from_email = env_value("DAYPRESS_SMTP_FROM")
        .or_else(|| option("daypress_smtp_from_email"))
        .unwrap_or(settings.email);
    let from_name = env_value("DAYPRESS_SMTP_FROM_NAME")
        .or_else(|| option("daypress_smtp_from_name"))
        .unwrap_or(settings.title);
    Some(EmailConfig {
        smtp_host: smtp_host.trim().to_string(),
        smtp_port,
        smtp_user: env_value("DAYPRESS_SMTP_USER").unwrap_or_default(),
        smtp_pass: env_value("DAYPRESS_SMTP_PASSWORD").unwrap_or_default(),
        from_email: from_email.trim().to_string(),
        from_name: from_name.trim().to_string(),
        encryption,
    })
}

/// Plain-text alternative for an HTML body when the caller did not provide one.
fn html_to_text(html: &str) -> String {
    let with_breaks = html
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("</p>", "\n\n")
        .replace("</li>", "\n")
        .replace("</h3>", "\n");
    regex::Regex::new(r"<[^>]*>")
        .map(|re| re.replace_all(&with_breaks, "").into_owned())
        .unwrap_or(with_breaks)
        .trim()
        .to_string()
}

fn build_message(cfg: &EmailConfig, msg: &EmailMessage) -> Result<Message, String> {
    let from: Mailbox = if cfg.from_name.is_empty() {
        cfg.from_email.parse()
    } else {
        format!(
            "{} <{}>",
            cfg.from_name.replace(['<', '>', '"'], ""),
            cfg.from_email
        )
        .parse()
    }
    .map_err(|e| format!("Invalid sender address '{}': {}", cfg.from_email, e))?;

    let mut builder = Message::builder().from(from).subject(msg.subject.as_str());
    let mut has_recipient = false;
    for addr in msg.to.split(',').map(str::trim).filter(|a| !a.is_empty()) {
        let mailbox: Mailbox = addr
            .parse()
            .map_err(|e| format!("Invalid recipient '{}': {}", addr, e))?;
        builder = builder.to(mailbox);
        has_recipient = true;
    }
    if !has_recipient {
        return Err("No recipient address".to_string());
    }

    let text = if msg.body_text.trim().is_empty() {
        html_to_text(&msg.body_html)
    } else {
        msg.body_text.clone()
    };
    builder
        .multipart(
            MultiPart::alternative()
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_PLAIN)
                        .body(text),
                )
                .singlepart(
                    SinglePart::builder()
                        .header(ContentType::TEXT_HTML)
                        .body(msg.body_html.clone()),
                ),
        )
        .map_err(|e| format!("Could not build message: {}", e))
}

async fn deliver(cfg: &EmailConfig, message: Message) -> Result<(), String> {
    let host = cfg.smtp_host.as_str();
    let builder = match cfg.encryption.as_str() {
        "tls" | "ssl" => AsyncSmtpTransport::<Tokio1Executor>::relay(host),
        "none" => Ok(AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(
            host,
        )),
        _ => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(host),
    }
    .map_err(|e| format!("SMTP setup failed: {}", e))?;
    let mut builder = builder
        .port(cfg.smtp_port)
        .timeout(Some(Duration::from_secs(20)));
    if !cfg.smtp_user.is_empty() {
        builder = builder.credentials(Credentials::new(
            cfg.smtp_user.clone(),
            cfg.smtp_pass.clone(),
        ));
    }
    builder
        .build()
        .send(message)
        .await
        .map(|_| ())
        .map_err(|e| format!("SMTP delivery failed: {}", e))
}

/// Sends an email over the configured transport (`smtp`, `resend`, `ses`, or `log_only`) and records the outcome in the email log.
///
/// Without SMTP/Resend/SES settings (see [`smtp_config`]) nothing is sent: the attempt is logged as
/// `not_sent` and an error is returned, so callers never report a delivery that did not happen.
pub async fn send_transactional_email(db: &Database, msg: &EmailMessage) -> Result<(), AppError> {
    let start = Instant::now();
    let option = |name: &str| {
        db.get_option(name)
            .ok()
            .flatten()
            .filter(|v| !v.trim().is_empty())
    };
    let transport_pref = env_value("DAYPRESS_EMAIL_TRANSPORT")
        .or_else(|| option("daypress_email_transport"))
        .map(|s| s.trim().to_ascii_lowercase())
        .unwrap_or_default();

    if transport_pref == "log_only" {
        db.log_email(&msg.to, &msg.subject, "log_only", "delivered", 0, None)?;
        return Ok(());
    }

    // Resend HTTP API transport (`POST https://api.resend.com/emails`)
    let resend_key = env_value("DAYPRESS_RESEND_API_KEY")
        .or_else(|| option("daypress_resend_api_key"))
        .or_else(|| option("daypress_email_api_key"));
    if transport_pref == "resend" || (transport_pref.is_empty() && resend_key.is_some()) {
        let Some(api_key) = resend_key else {
            let reason = "Resend transport selected, but DAYPRESS_RESEND_API_KEY is not set.";
            db.log_email(&msg.to, &msg.subject, "resend", "not_sent", 0, Some(reason))?;
            return Err(AppError::BadRequest(reason.to_string()));
        };
        let settings = db.get_settings().unwrap_or_default();
        let from_email = env_value("DAYPRESS_SMTP_FROM")
            .or_else(|| option("daypress_smtp_from_email"))
            .unwrap_or(settings.email);
        let from_name = env_value("DAYPRESS_SMTP_FROM_NAME")
            .or_else(|| option("daypress_smtp_from_name"))
            .unwrap_or(settings.title);
        let from_str = if from_name.trim().is_empty() {
            from_email.trim().to_string()
        } else {
            format!("{} <{}>", from_name.trim(), from_email.trim())
        };
        let endpoint = option("daypress_resend_endpoint")
            .unwrap_or_else(|| "https://api.resend.com/emails".to_string());
        let recipients: Vec<String> = msg
            .to
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        let text_body = if msg.body_text.trim().is_empty() {
            html_to_text(&msg.body_html)
        } else {
            msg.body_text.clone()
        };
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        let resp = client
            .post(&endpoint)
            .bearer_auth(api_key.trim())
            .json(&serde_json::json!({
                "from": from_str,
                "to": recipients,
                "subject": msg.subject,
                "html": msg.body_html,
                "text": text_body,
            }))
            .send()
            .await;
        let latency_ms = start.elapsed().as_millis() as i64;
        return match resp {
            Ok(r) if r.status().is_success() => {
                db.log_email(
                    &msg.to,
                    &msg.subject,
                    "resend",
                    "delivered",
                    latency_ms,
                    None,
                )?;
                Ok(())
            }
            Ok(r) => {
                let err_txt = format!("Resend HTTP {}", r.status().as_u16());
                db.log_email(
                    &msg.to,
                    &msg.subject,
                    "resend",
                    "failed",
                    latency_ms,
                    Some(&err_txt),
                )?;
                Err(AppError::Internal(err_txt))
            }
            Err(e) => {
                let err_txt = format!("Resend request failed: {}", e);
                db.log_email(
                    &msg.to,
                    &msg.subject,
                    "resend",
                    "failed",
                    latency_ms,
                    Some(&err_txt),
                )?;
                Err(AppError::Internal(err_txt))
            }
        };
    }

    // AWS SES SMTP relay (`email-smtp.<region>.amazonaws.com:587`) when transport == "ses"
    let cfg_opt = if transport_pref == "ses" {
        let region = env_value("DAYPRESS_SES_REGION")
            .or_else(|| option("daypress_ses_region"))
            .unwrap_or_else(|| "us-east-1".to_string());
        let ses_host = option("daypress_smtp_host")
            .unwrap_or_else(|| format!("email-smtp.{}.amazonaws.com", region.trim()));
        let settings = db.get_settings().unwrap_or_default();
        let from_email = env_value("DAYPRESS_SMTP_FROM")
            .or_else(|| option("daypress_smtp_from_email"))
            .unwrap_or(settings.email);
        let from_name = env_value("DAYPRESS_SMTP_FROM_NAME")
            .or_else(|| option("daypress_smtp_from_name"))
            .unwrap_or(settings.title);
        Some(EmailConfig {
            smtp_host: ses_host,
            smtp_port: 587,
            smtp_user: env_value("DAYPRESS_SES_SMTP_USER")
                .or_else(|| option("daypress_ses_smtp_user"))
                .or_else(|| env_value("DAYPRESS_SMTP_USER"))
                .unwrap_or_default(),
            smtp_pass: env_value("DAYPRESS_SES_SMTP_PASSWORD")
                .or_else(|| option("daypress_ses_smtp_password"))
                .or_else(|| env_value("DAYPRESS_SMTP_PASSWORD"))
                .unwrap_or_default(),
            from_email,
            from_name,
            encryption: "starttls".to_string(),
        })
    } else {
        smtp_config(db)
    };

    let Some(cfg) = cfg_opt else {
        let reason = "SMTP is not configured; set DAYPRESS_SMTP_HOST (see README).";
        db.log_email(&msg.to, &msg.subject, "none", "not_sent", 0, Some(reason))?;
        return Err(AppError::BadRequest(reason.to_string()));
    };

    let transport_label = if transport_pref == "ses" {
        "ses"
    } else {
        "smtp"
    };
    let outcome = match build_message(&cfg, msg) {
        Ok(message) => deliver(&cfg, message).await,
        Err(e) => Err(e),
    };
    let latency_ms = start.elapsed().as_millis() as i64;
    match outcome {
        Ok(()) => {
            db.log_email(
                &msg.to,
                &msg.subject,
                transport_label,
                "delivered",
                latency_ms,
                None,
            )?;
            Ok(())
        }
        Err(e) => {
            db.log_email(
                &msg.to,
                &msg.subject,
                transport_label,
                "failed",
                latency_ms,
                Some(&e),
            )?;
            Err(AppError::Internal(e))
        }
    }
}

/// Dispatches a newsletter email (double opt-in confirmation, broadcast campaign, or automated post digest).
/// When an external transport (`smtp`, `resend`, or `ses`) is configured, it spawns async delivery;
/// when running in local/self-hosted zero-config mode, it logs the delivery in `daypress_email_logs`
/// with transport `"newsletter_local"` so double opt-in and digests are auditable out-of-the-box.
pub fn dispatch_newsletter_email(db: &Database, msg: EmailMessage) -> Result<(), AppError> {
    let has_external_transport = smtp_config(db).is_some()
        || env_value("DAYPRESS_RESEND_API_KEY").is_some()
        || db
            .get_option("daypress_resend_api_key")
            .ok()
            .flatten()
            .is_some_and(|v| !v.trim().is_empty())
        || db
            .get_option("daypress_email_transport")
            .ok()
            .flatten()
            .is_some_and(|v| matches!(v.trim(), "smtp" | "resend" | "ses"));

    if has_external_transport {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let db_clone = db.clone();
            handle.spawn(async move {
                let _ = send_transactional_email(&db_clone, &msg).await;
            });
            return Ok(());
        }
    }
    db.log_email(
        &msg.to,
        &msg.subject,
        "newsletter_local",
        "delivered",
        1,
        None,
    )
    .map_err(AppError::from)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_multipart_message_and_rejects_bad_addresses() {
        let cfg = EmailConfig {
            smtp_host: "smtp.example.com".to_string(),
            from_email: "site@example.com".to_string(),
            from_name: "Example Site".to_string(),
            ..Default::default()
        };
        let msg = EmailMessage {
            to: "a@example.com, b@example.com".to_string(),
            subject: "Hello".to_string(),
            body_html: "<p>Hi <b>there</b></p>".to_string(),
            body_text: String::new(),
        };
        assert!(build_message(&cfg, &msg).is_ok());
        assert_eq!(html_to_text(&msg.body_html), "Hi there");

        let bad = EmailMessage {
            to: "not-an-address".to_string(),
            ..msg
        };
        assert!(build_message(&cfg, &bad).is_err());
    }
}
