#![allow(unused_imports)]
use super::*;
use crate::auth::OptionalAuthUser;
use crate::db::Database;
use crate::models;
use crate::templates;
use axum::{
    extract::{Form, Path as AxPath, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{Html, IntoResponse},
};
use serde::Deserialize;

pub async fn handle_admin_newsletters_export_csv(State(db): State<Database>) -> impl IntoResponse {
    let csv = db.export_subscribers_csv().unwrap_or_default();
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "text/csv; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"daypress-subscribers.csv\""
            .parse()
            .unwrap(),
    );
    (StatusCode::OK, headers, csv).into_response()
}

pub async fn handle_admin_newsletters(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let stats = db.get_newsletter_stats().unwrap_or_default();
    let subscribers = db
        .list_subscribers(None, None, None, true)
        .unwrap_or_default();
    let campaigns = db.list_newsletter_campaigns().unwrap_or_default();
    let double_opt_in = db.is_newsletter_double_opt_in_enabled();
    let auto_digest = db.is_newsletter_auto_digest_enabled();
    let transport = db
        .get_option("daypress_email_transport")
        .ok()
        .flatten()
        .unwrap_or_else(|| "smtp".to_string());

    let mut sub_rows = String::new();
    for s in &subscribers {
        let badge_class = match s.status.as_str() {
            "subscribed" => "badge-published",
            "pending" => "badge-draft",
            _ => "badge-trash",
        };
        let lists_str = crate::templates::utils::escape_html(&s.lists.join(", "));
        let email_esc = crate::templates::utils::escape_html(&s.email);
        let name_esc = crate::templates::utils::escape_html(&s.name);
        let status_esc = crate::templates::utils::escape_html(&s.status);
        let created_esc = crate::templates::utils::escape_html(&s.created_at);
        let toggle_action = if s.status == "subscribed" {
            format!(
                r#"<button type="button" class="btn btn-secondary" style="font-size:11px;padding:4px 8px;" onclick="updateSubscriberStatus({}, 'unsubscribed')">Unsubscribe</button>"#,
                s.id
            )
        } else {
            format!(
                r#"<button type="button" class="btn btn-secondary" style="font-size:11px;padding:4px 8px;" onclick="updateSubscriberStatus({}, 'subscribed')">Confirm</button>"#,
                s.id
            )
        };
        sub_rows.push_str(&format!(
            r#"<tr>
                <td><strong>{email_esc}</strong><div style="font-size:11.5px;color:var(--ink-muted);">{name_esc}</div></td>
                <td><span class="badge {badge_class}">{status_esc}</span></td>
                <td><code>{lists_str}</code></td>
                <td style="font-size:12px;color:var(--ink-muted);">{created_esc}</td>
                <td style="text-align:right;display:flex;gap:6px;justify-content:flex-end;">
                    {toggle_action}
                    <button type="button" class="btn btn-secondary" style="font-size:11px;padding:4px 8px;color:var(--oxide);" onclick="deleteSubscriber({id})">Delete</button>
                </td>
            </tr>"#,
            id = s.id
        ));
    }
    if sub_rows.is_empty() {
        sub_rows = r#"<tr><td colspan="5" style="text-align:center;color:var(--ink-muted);padding:24px;">No subscribers yet. Embed <code>[daypress_newsletter]</code> on any page or add a subscriber below.</td></tr>"#.to_string();
    }

    let mut campaign_rows = String::new();
    for c in &campaigns {
        let subj_esc = crate::templates::utils::escape_html(&c.subject);
        let type_esc = crate::templates::utils::escape_html(&c.campaign_type);
        let list_esc = crate::templates::utils::escape_html(&c.target_list);
        let status_esc = crate::templates::utils::escape_html(&c.status);
        let sent_esc =
            crate::templates::utils::escape_html(c.sent_at.as_deref().unwrap_or(&c.created_at));
        campaign_rows.push_str(&format!(
            r#"<tr>
                <td><strong>{subj_esc}</strong></td>
                <td><span class="badge badge-draft">{type_esc}</span></td>
                <td><code>{list_esc}</code></td>
                <td><span class="badge badge-published">{status_esc}</span></td>
                <td style="font-family:var(--font-mono);font-size:12px;">{sent} / {recip}</td>
                <td style="font-size:12px;color:var(--ink-muted);">{sent_esc}</td>
            </tr>"#,
            sent = c.sent_count,
            recip = c.recipient_count
        ));
    }
    if campaign_rows.is_empty() {
        campaign_rows = r#"<tr><td colspan="6" style="text-align:center;color:var(--ink-muted);padding:24px;">No campaigns or automated digests sent yet.</td></tr>"#.to_string();
    }

    let doi_checked = if double_opt_in { "checked" } else { "" };
    let ad_checked = if auto_digest { "checked" } else { "" };
    let tr_smtp_sel = if transport == "smtp" { "selected" } else { "" };
    let tr_resend_sel = if transport == "resend" {
        "selected"
    } else {
        ""
    };
    let tr_ses_sel = if transport == "ses" { "selected" } else { "" };
    let tr_log_sel = if transport == "log_only" {
        "selected"
    } else {
        ""
    };

    let body = format!(
        r#"
        <div class="page-header">
            <div>
                <h1 class="page-title">Newsletters, Member Gate &amp; Automated Digests</h1>
                <p class="page-subtitle">MailPoet &amp; Ghost parity: Native SQLite subscriber lists, Double Opt-In, member-gated posts, and email broadcasts</p>
            </div>
            <div style="display:flex;gap:8px;align-items:center;">
                <a href="/d-admin/api/newsletters/export-csv" class="btn btn-secondary">⬇ Export Subscribers CSV</a>
            </div>
        </div>

        <div style="display:grid;grid-template-columns:repeat(4,1fr);gap:16px;margin-bottom:24px;">
            <div class="day-card"><div class="day-card-body"><div style="font-size:12px;color:var(--ink-muted);">Confirmed Subscribers</div><div style="font-size:28px;font-weight:800;color:var(--ink);">{confirmed}</div></div></div>
            <div class="day-card"><div class="day-card-body"><div style="font-size:12px;color:var(--ink-muted);">Pending Opt-In</div><div style="font-size:28px;font-weight:800;color:var(--ink);">{pending}</div></div></div>
            <div class="day-card"><div class="day-card-body"><div style="font-size:12px;color:var(--ink-muted);">Campaigns &amp; Digests Sent</div><div style="font-size:28px;font-weight:800;color:var(--ink);">{campaigns_sent}</div></div></div>
            <div class="day-card"><div class="day-card-body"><div style="font-size:12px;color:var(--ink-muted);">Total Emails Delivered</div><div style="font-size:28px;font-weight:800;color:var(--ink);">{delivered}</div></div></div>
        </div>

        <div style="display:grid;grid-template-columns:2fr 1fr;gap:24px;">
            <div style="display:flex;flex-direction:column;gap:20px;">
                <div class="day-card">
                    <div class="day-card-header" style="display:flex;justify-content:space-between;align-items:center;">
                        <h2 class="day-card-title">Subscribers ({total})</h2>
                        <span style="font-size:12px;color:var(--ink-muted);">Shortcode: <code>[daypress_newsletter]</code></span>
                    </div>
                    <div class="day-card-body" style="padding:0;overflow-x:auto;">
                        <table class="day-table">
                            <thead>
                                <tr>
                                    <th>Subscriber</th>
                                    <th>Status</th>
                                    <th>Lists</th>
                                    <th>Joined</th>
                                    <th style="text-align:right;">Actions</th>
                                </tr>
                            </thead>
                            <tbody>{sub_rows}</tbody>
                        </table>
                    </div>
                </div>

                <div class="day-card">
                    <div class="day-card-header"><h2 class="day-card-title">Broadcast &amp; Digest History</h2></div>
                    <div class="day-card-body" style="padding:0;overflow-x:auto;">
                        <table class="day-table">
                            <thead>
                                <tr>
                                    <th>Subject</th>
                                    <th>Type</th>
                                    <th>List</th>
                                    <th>Status</th>
                                    <th>Delivered</th>
                                    <th>Sent At</th>
                                </tr>
                            </thead>
                            <tbody>{campaign_rows}</tbody>
                        </table>
                    </div>
                </div>
            </div>

            <div style="display:flex;flex-direction:column;gap:20px;">
                <div class="day-card">
                    <div class="day-card-header"><h2 class="day-card-title">Compose Email Broadcast</h2></div>
                    <div class="day-card-body">
                        <div class="form-group">
                            <label class="form-label">Subject Line</label>
                            <input type="text" id="campSubject" class="form-input" placeholder="Monthly Dispatch: What we shipped..." />
                        </div>
                        <div class="form-group">
                            <label class="form-label">Target List</label>
                            <input type="text" id="campList" class="form-input" value="general" />
                        </div>
                        <div class="form-group">
                            <label class="form-label">HTML Body</label>
                            <textarea id="campBody" class="form-textarea" rows="5" placeholder="<p>Hello readers,</p><p>Here is our latest update...</p>"></textarea>
                        </div>
                        <button type="button" class="btn btn-primary" style="width:100%;" onclick="sendNewsletterBroadcast()">Send Broadcast Now &rarr;</button>
                    </div>
                </div>

                <div class="day-card">
                    <div class="day-card-header"><h2 class="day-card-title">Add Subscriber Directly</h2></div>
                    <div class="day-card-body">
                        <div class="form-group">
                            <label class="form-label">Email Address</label>
                            <input type="email" id="addSubEmail" class="form-input" placeholder="reader@example.com" />
                        </div>
                        <div class="form-group">
                            <label class="form-label">Name &amp; List</label>
                            <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;">
                                <input type="text" id="addSubName" class="form-input" placeholder="Jane Doe" />
                                <input type="text" id="addSubList" class="form-input" value="general" placeholder="general" />
                            </div>
                        </div>
                        <button type="button" class="btn btn-secondary" style="width:100%;" onclick="addSubscriberDirectly()">+ Add Confirmed Subscriber</button>
                    </div>
                </div>

                <div class="day-card">
                    <div class="day-card-header"><h2 class="day-card-title">Newsletter &amp; Transport Settings</h2></div>
                    <div class="day-card-body">
                        <label style="display:flex;align-items:center;gap:8px;font-size:13px;margin-bottom:10px;cursor:pointer;">
                            <input type="checkbox" id="nlDoubleOptIn" {doi_checked} />
                            <span>Require Double Opt-In confirmation email</span>
                        </label>
                        <label style="display:flex;align-items:center;gap:8px;font-size:13px;margin-bottom:14px;cursor:pointer;">
                            <input type="checkbox" id="nlAutoDigest" {ad_checked} />
                            <span>Automatically email subscribers when a new post is published</span>
                        </label>
                        <div class="form-group">
                            <label class="form-label">Delivery Transport</label>
                            <select id="nlTransport" class="form-select">
                                <option value="smtp" {tr_smtp_sel}>SMTP (Standard / Lettre)</option>
                                <option value="resend" {tr_resend_sel}>Resend HTTP API</option>
                                <option value="ses" {tr_ses_sel}>AWS SES Relay</option>
                                <option value="log_only" {tr_log_sel}>Local SQLite Audit Log Only</option>
                            </select>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Resend API Key <span style="font-weight:400;font-size:11px;color:var(--ink-muted);">(Optional)</span></label>
                            <input type="password" id="nlResendKey" class="form-input" placeholder="re_..." />
                        </div>
                        <button type="button" class="btn btn-secondary" style="width:100%;" onclick="saveNewsletterSettings()">Save Newsletter Settings</button>
                    </div>
                </div>
            </div>
        </div>

        <script>
            async function addSubscriberDirectly() {{
                const email = document.getElementById('addSubEmail').value.trim();
                const name = document.getElementById('addSubName').value.trim();
                const list = document.getElementById('addSubList').value.trim() || 'general';
                if (!email) {{ showToast('Please enter an email address', true); return; }}
                const res = await fetch('/wp-json/daypress/v1/newsletters/subscribe', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ email, name, lists: [list], skip_double_opt_in: true }})
                }});
                if (res.ok) {{
                    showToast('✓ Subscriber added to SQLite');
                    setTimeout(() => window.location.reload(), 350);
                }} else {{
                    showToast('Failed to add subscriber', true);
                }}
            }}

            async function updateSubscriberStatus(id, status) {{
                const res = await fetch('/wp-json/daypress/v1/newsletters/subscribers/' + id, {{
                    method: 'PATCH',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ status }})
                }});
                if (res.ok) {{
                    showToast('✓ Subscriber updated');
                    setTimeout(() => window.location.reload(), 300);
                }}
            }}

            async function deleteSubscriber(id) {{
                if (!confirm('Delete subscriber #' + id + '?')) return;
                const res = await fetch('/wp-json/daypress/v1/newsletters/subscribers/' + id, {{ method: 'DELETE' }});
                if (res.ok) {{
                    showToast('✓ Subscriber deleted');
                    setTimeout(() => window.location.reload(), 300);
                }}
            }}

            async function sendNewsletterBroadcast() {{
                const subject = document.getElementById('campSubject').value.trim();
                const target_list = document.getElementById('campList').value.trim() || 'general';
                const body_html = document.getElementById('campBody').value.trim();
                if (!subject || !body_html) {{ showToast('Subject and HTML body are required', true); return; }}
                const res = await fetch('/wp-json/daypress/v1/newsletters/campaigns', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ subject, target_list, body_html, send_now: true }})
                }});
                if (res.ok) {{
                    const c = await res.json();
                    showToast('✓ Broadcast sent to ' + c.sent_count + ' subscribers!');
                    setTimeout(() => window.location.reload(), 450);
                }} else {{
                    showToast('Failed to send broadcast', true);
                }}
            }}

            async function saveNewsletterSettings() {{
                const double_opt_in = document.getElementById('nlDoubleOptIn').checked;
                const auto_digest = document.getElementById('nlAutoDigest').checked;
                const email_transport = document.getElementById('nlTransport').value;
                const resend_api_key = document.getElementById('nlResendKey').value.trim();
                const res = await fetch('/wp-json/daypress/v1/newsletters/settings', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ double_opt_in, auto_digest, email_transport, resend_api_key: resend_api_key || undefined }})
                }});
                if (res.ok) {{
                    showToast('✓ Newsletter settings saved to SQLite');
                }} else {{
                    showToast('Failed to save settings', true);
                }}
            }}
        </script>
        "#,
        confirmed = stats.confirmed_subscribers,
        pending = stats.pending_subscribers,
        campaigns_sent = stats.total_campaigns_sent,
        delivered = stats.total_emails_delivered,
        total = stats.total_subscribers,
        sub_rows = sub_rows,
        campaign_rows = campaign_rows,
        doi_checked = doi_checked,
        ad_checked = ad_checked,
        tr_smtp_sel = tr_smtp_sel,
        tr_resend_sel = tr_resend_sel,
        tr_ses_sel = tr_ses_sel,
        tr_log_sel = tr_log_sel,
    );

    Html(templates::render_admin_shell(
        "newsletters",
        "Newsletters & Member Gate",
        &body,
        &settings,
    ))
}

/// `/d-admin/webhooks`: Branded Shortlinks (`/go/*`) & HMAC-Signed Outbound Webhooks Studio (`v0.9`).
pub async fn handle_admin_webhooks(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let shortlinks = db.list_shortlinks().unwrap_or_default();
    let webhooks = db.list_webhook_endpoints().unwrap_or_default();
    let deliveries = db.list_webhook_deliveries(None, 50).unwrap_or_default();

    let total_clicks: i64 = shortlinks.iter().map(|s| s.clicks).sum();
    let total_unique: i64 = total_clicks;

    let mut shortlink_rows = String::new();
    for s in &shortlinks {
        let badges = r#"<span class="badge" style="background:rgba(161,161,170,0.16);color:var(--text-muted);margin-left:6px;">nofollow</span>"#;
        shortlink_rows.push_str(&format!(
            r#"<tr>
                <td>
                    <strong>{title}</strong>
                    <div style="margin-top:2px;"><a href="/go/{slug}" target="_blank" style="font-family:var(--font-mono);font-size:12px;color:var(--primary);text-decoration:none;">/go/{slug} &nearr;</a>{badges}</div>
                </td>
                <td style="max-width:280px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;font-family:var(--font-mono);font-size:12px;">{target}</td>
                <td><span class="badge">{code}</span></td>
                <td style="font-family:var(--font-mono);"><strong>{clicks}</strong></td>
                <td style="text-align:right;">
                    <button type="button" class="btn" style="padding:4px 10px;font-size:11px;" onclick="navigator.clipboard.writeText(window.location.origin + '/go/{slug}'); showToast('Copied /go/{slug}');">Copy</button>
                    <button type="button" class="btn" style="padding:4px 10px;font-size:11px;color:#b91c1c;" onclick="deleteShortlink({id})">Delete</button>
                </td>
            </tr>"#,
            id = s.id,
            title = templates::escape_html(if s.title.is_empty() { &s.slug } else { &s.title }),
            slug = templates::escape_html(&s.slug),
            target = templates::escape_html(&s.target_url),
            code = s.status_code,
            clicks = s.clicks,
            badges = badges,
        ));
    }
    if shortlink_rows.is_empty() {
        shortlink_rows.push_str(
            r#"<tr><td colspan="5" style="text-align:center;color:var(--text-muted);padding:24px;">No branded shortlinks created yet. Create your first <code>/go/slug</code> link below.</td></tr>"#,
        );
    }

    let mut webhook_rows = String::new();
    for w in &webhooks {
        let evts = if w.events.is_empty() {
            "*".to_string()
        } else {
            w.events.join(", ")
        };
        webhook_rows.push_str(&format!(
            r#"<tr>
                <td><strong>{name}</strong><div style="font-family:var(--font-mono);font-size:11px;color:var(--text-muted);">{url}</div></td>
                <td><span class="badge">{preset}</span></td>
                <td style="font-family:var(--font-mono);font-size:12px;">{evts}</td>
                <td style="font-family:var(--font-mono);font-size:11px;color:var(--text-muted);">{secret_preview}</td>
                <td style="text-align:right;">
                    <button type="button" class="btn" style="padding:4px 10px;font-size:11px;color:#b91c1c;" onclick="deleteWebhook({id})">Delete</button>
                </td>
            </tr>"#,
            id = w.id,
            name = templates::escape_html(&w.name),
            url = templates::escape_html(&w.target_url),
            preset = templates::escape_html(&w.preset),
            evts = templates::escape_html(&evts),
            secret_preview = templates::escape_html(&w.secret.chars().take(10).collect::<String>()),
        ));
    }
    if webhook_rows.is_empty() {
        webhook_rows.push_str(
            r#"<tr><td colspan="5" style="text-align:center;color:var(--text-muted);padding:24px;">No outbound webhook endpoints configured yet.</td></tr>"#,
        );
    }

    let mut delivery_rows = String::new();
    for d in &deliveries {
        let badge_style = if d.status == "delivered" || d.status == "replayed" {
            "background:rgba(16,185,129,0.14);color:#047857;"
        } else {
            "background:rgba(239,68,68,0.14);color:#b91c1c;"
        };
        delivery_rows.push_str(&format!(
            r#"<tr>
                <td style="font-family:var(--font-mono);font-size:12px;">#{id}</td>
                <td><strong>{evt}</strong></td>
                <td><span class="badge" style="{badge_style}">{status} (HTTP {code})</span></td>
                <td style="font-family:var(--font-mono);font-size:11px;color:var(--text-muted);">{sig}</td>
                <td style="font-size:12px;color:var(--text-muted);">{created}</td>
                <td style="text-align:right;">
                    <button type="button" class="btn" style="padding:4px 10px;font-size:11px;" onclick="replayWebhookDelivery({id})">Replay</button>
                </td>
            </tr>"#,
            id = d.id,
            evt = templates::escape_html(&d.event_name),
            badge_style = badge_style,
            status = templates::escape_html(&d.status),
            code = d.response_status,
            sig = templates::escape_html(&d.signature_header.chars().take(22).collect::<String>()),
            created = templates::escape_html(&d.created_at),
        ));
    }
    if delivery_rows.is_empty() {
        delivery_rows.push_str(
            r#"<tr><td colspan="6" style="text-align:center;color:var(--text-muted);padding:24px;">No webhook deliveries logged yet.</td></tr>"#,
        );
    }

    let body = format!(
        r#"
        <div class="page-header" style="display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:12px;">
            <div>
                <h1 class="page-title">Branded Shortlinks (<code>/go/*</code>) &amp; Outbound Webhooks</h1>
                <p class="page-subtitle">Pretty Links &amp; WP Webhooks parity: First-party affiliate/campaign shortlinks with UTM tagging + HMAC-SHA256 signed event webhooks with 1-click replay.</p>
            </div>
            <div style="display:flex;gap:8px;">
                <button type="button" class="btn" onclick="sendTestWebhook()">Send Test Event</button>
            </div>
        </div>

        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:16px;margin-bottom:24px;">
            <div class="card" style="padding:18px;">
                <div style="font-size:12px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;">Active Shortlinks</div>
                <div style="font-size:28px;font-weight:800;margin-top:4px;">{shortlink_count}</div>
            </div>
            <div class="card" style="padding:18px;">
                <div style="font-size:12px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;">Total Shortlink Clicks</div>
                <div style="font-size:28px;font-weight:800;margin-top:4px;">{total_clicks} <span style="font-size:13px;font-weight:500;color:var(--text-muted);">({total_unique} unique)</span></div>
            </div>
            <div class="card" style="padding:18px;">
                <div style="font-size:12px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;">Webhook Endpoints</div>
                <div style="font-size:28px;font-weight:800;margin-top:4px;">{webhook_count}</div>
            </div>
            <div class="card" style="padding:18px;">
                <div style="font-size:12px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;">Logged Deliveries</div>
                <div style="font-size:28px;font-weight:800;margin-top:4px;">{delivery_count}</div>
            </div>
        </div>

        <div style="display:grid;grid-template-columns:1.25fr 1fr;gap:24px;align-items:start;">
            <div style="display:flex;flex-direction:column;gap:24px;">
                <div class="card" style="padding:20px;">
                    <h3 style="margin:0 0 14px;font-size:16px;">Branded Shortlinks (<code>/go/*</code>)</h3>
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>Shortlink</th>
                                <th>Target URL</th>
                                <th>HTTP</th>
                                <th>Clicks</th>
                                <th style="text-align:right;">Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            {shortlink_rows}
                        </tbody>
                    </table>
                </div>

                <div class="card" style="padding:20px;">
                    <h3 style="margin:0 0 14px;font-size:16px;">Outbound Webhook Endpoints (<code>X-Daypress-Signature-256</code>)</h3>
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>Endpoint</th>
                                <th>Preset</th>
                                <th>Events</th>
                                <th>Secret</th>
                                <th style="text-align:right;">Actions</th>
                            </tr>
                        </thead>
                        <tbody>
                            {webhook_rows}
                        </tbody>
                    </table>
                </div>

                <div class="card" style="padding:20px;">
                    <h3 style="margin:0 0 14px;font-size:16px;">Webhook Delivery Log &amp; 1-Click Replay</h3>
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th>ID</th>
                                <th>Event</th>
                                <th>Status</th>
                                <th>HMAC Header</th>
                                <th>Timestamp</th>
                                <th style="text-align:right;">Replay</th>
                            </tr>
                        </thead>
                        <tbody>
                            {delivery_rows}
                        </tbody>
                    </table>
                </div>
            </div>

            <div style="display:flex;flex-direction:column;gap:20px;">
                <div class="card" style="padding:20px;">
                    <h3 style="margin:0 0 12px;font-size:16px;">Create Branded Shortlink</h3>
                    <div style="display:flex;flex-direction:column;gap:10px;">
                        <div>
                            <label class="form-label">Slug (<code>/go/&lt;slug&gt;</code>)</label>
                            <input type="text" id="slSlug" class="form-input" placeholder="e.g. github or partner-deal" />
                        </div>
                        <div>
                            <label class="form-label">Target Destination URL</label>
                            <input type="url" id="slTarget" class="form-input" placeholder="https://example.com/landing" />
                        </div>
                        <div style="display:grid;grid-template-columns:1fr 1fr;gap:10px;">
                            <div>
                                <label class="form-label">Label / Title</label>
                                <input type="text" id="slTitle" class="form-input" placeholder="Partner Deal" />
                            </div>
                            <div>
                                <label class="form-label">Redirect Code</label>
                                <select id="slCode" class="form-input">
                                    <option value="307" selected>307 Temporary</option>
                                    <option value="302">302 Found</option>
                                    <option value="301">301 Permanent</option>
                                </select>
                            </div>
                        </div>
                        <div style="display:grid;grid-template-columns:1fr 1fr 1fr;gap:8px;">
                            <input type="text" id="slUtmSource" class="form-input" placeholder="utm_source" />
                            <input type="text" id="slUtmMedium" class="form-input" placeholder="utm_medium" />
                            <input type="text" id="slUtmCampaign" class="form-input" placeholder="utm_campaign" />
                        </div>
                        <div style="display:flex;gap:16px;font-size:13px;">
                            <label style="display:flex;align-items:center;gap:6px;"><input type="checkbox" id="slNofollow" checked /> <code>nofollow</code></label>
                            <label style="display:flex;align-items:center;gap:6px;"><input type="checkbox" id="slSponsored" /> <code>sponsored</code></label>
                        </div>
                        <button type="button" class="btn btn-primary" onclick="createShortlink()">Save Shortlink</button>
                    </div>
                </div>

                <div class="card" style="padding:20px;">
                    <h3 style="margin:0 0 12px;font-size:16px;">Add Outbound Webhook Endpoint</h3>
                    <div style="display:flex;flex-direction:column;gap:10px;">
                        <div>
                            <label class="form-label">Endpoint Name</label>
                            <input type="text" id="whName" class="form-input" placeholder="Slack #marketing-leads" />
                        </div>
                        <div>
                            <label class="form-label">Target Webhook URL</label>
                            <input type="url" id="whUrl" class="form-input" placeholder="https://hooks.slack.com/services/..." />
                        </div>
                        <div style="display:grid;grid-template-columns:1fr 1fr;gap:10px;">
                            <div>
                                <label class="form-label">Payload Preset</label>
                                <select id="whPreset" class="form-input">
                                    <option value="generic">Generic JSON + HMAC</option>
                                    <option value="slack">Slack Incoming Webhook</option>
                                    <option value="discord">Discord Webhook</option>
                                    <option value="hubspot">HubSpot / CRM</option>
                                    <option value="zapier">Zapier / Make</option>
                                </select>
                            </div>
                            <div>
                                <label class="form-label">Events (comma-separated)</label>
                                <input type="text" id="whEvents" class="form-input" value="post.published,form.submitted,subscriber.confirmed" />
                            </div>
                        </div>
                        <button type="button" class="btn btn-primary" onclick="createWebhook()">Register Webhook</button>
                    </div>
                </div>
            </div>
        </div>

        <script>
            async function createShortlink() {{
                const slug = document.getElementById('slSlug').value.trim();
                const target_url = document.getElementById('slTarget').value.trim();
                const title = document.getElementById('slTitle').value.trim();
                const status_code = parseInt(document.getElementById('slCode').value, 10) || 307;
                const utm_source = document.getElementById('slUtmSource').value.trim();
                const utm_medium = document.getElementById('slUtmMedium').value.trim();
                const utm_campaign = document.getElementById('slUtmCampaign').value.trim();
                const nofollow = document.getElementById('slNofollow').checked;
                const sponsored = document.getElementById('slSponsored').checked;
                if (!slug || !target_url) {{
                    showToast('Slug and target URL are required', true);
                    return;
                }}
                const res = await fetch('/wp-json/daypress/v1/shortlinks', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ slug, target_url, title, status_code, utm_source, utm_medium, utm_campaign, nofollow, sponsored }})
                }});
                if (res.ok) {{
                    showToast('✓ Shortlink saved');
                    setTimeout(() => window.location.reload(), 400);
                }} else {{
                    showToast('Failed to save shortlink', true);
                }}
            }}

            async function deleteShortlink(id) {{
                const res = await fetch('/wp-json/daypress/v1/shortlinks/' + id, {{ method: 'DELETE' }});
                if (res.ok) {{
                    showToast('Shortlink removed');
                    setTimeout(() => window.location.reload(), 300);
                }}
            }}

            async function createWebhook() {{
                const name = document.getElementById('whName').value.trim();
                const target_url = document.getElementById('whUrl').value.trim();
                const preset = document.getElementById('whPreset').value;
                const events = document.getElementById('whEvents').value.split(',').map(s => s.trim()).filter(Boolean);
                if (!target_url) {{
                    showToast('Webhook target URL is required', true);
                    return;
                }}
                const res = await fetch('/wp-json/daypress/v1/webhooks', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ name: name || 'Outbound Webhook', target_url, preset, events }})
                }});
                if (res.ok) {{
                    showToast('✓ Webhook endpoint registered');
                    setTimeout(() => window.location.reload(), 400);
                }} else {{
                    showToast('Failed to register webhook', true);
                }}
            }}

            async function deleteWebhook(id) {{
                const res = await fetch('/wp-json/daypress/v1/webhooks/' + id, {{ method: 'DELETE' }});
                if (res.ok) {{
                    showToast('Webhook deleted');
                    setTimeout(() => window.location.reload(), 300);
                }}
            }}

            async function replayWebhookDelivery(id) {{
                const res = await fetch('/wp-json/daypress/v1/webhooks/deliveries/' + id + '/replay', {{ method: 'POST' }});
                if (res.ok) {{
                    showToast('✓ Delivery replayed');
                    setTimeout(() => window.location.reload(), 400);
                }} else {{
                    showToast('Failed to replay delivery', true);
                }}
            }}

            async function sendTestWebhook() {{
                const res = await fetch('/wp-json/daypress/v1/webhooks/test', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ event_type: 'post.published' }})
                }});
                if (res.ok) {{
                    showToast('✓ Test webhook event dispatched');
                    setTimeout(() => window.location.reload(), 500);
                }}
            }}
        </script>
        "#,
        shortlink_count = shortlinks.len(),
        total_clicks = total_clicks,
        total_unique = total_unique,
        webhook_count = webhooks.len(),
        delivery_count = deliveries.len(),
        shortlink_rows = shortlink_rows,
        webhook_rows = webhook_rows,
        delivery_rows = delivery_rows,
    );

    Html(templates::render_admin_shell(
        "webhooks",
        "Shortlinks & Webhooks",
        &body,
        &settings,
    ))
}
