//! First-Party Modules (Cookieless Analytics, SEO & Redirects, Forms, Hot Backups, Plugin SDK).

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

// =========================================================================
// NATIVE COOKIELESS ANALYTICS DASHBOARD HANDLER
// =========================================================================

#[derive(Debug, Deserialize, Default)]
pub struct AnalyticsDashboardQuery {
    pub range: Option<String>,
}

pub async fn handle_admin_analytics(
    State(db): State<Database>,
    Query(query): Query<AnalyticsDashboardQuery>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let range = match query.range.as_deref().unwrap_or("30d") {
        "24h" | "7d" | "30d" | "90d" | "12m" => query.range.as_deref().unwrap_or("30d"),
        _ => "30d",
    };
    let summary = db.get_analytics_dashboard(range).unwrap_or_default();

    let range_btn = |key: &str, label: &str| -> String {
        let active = summary.range == key;
        let bg = if active {
            "background: var(--press); color: #fff; border-color: var(--press);"
        } else {
            "background: var(--paper-card); color: var(--ink); border-color: var(--border);"
        };
        format!(
            r#"<a href="/d-admin/analytics?range={key}" class="btn btn-secondary" style="padding: 6px 12px; font-size: 12.5px; text-decoration: none; {bg}">{label}</a>"#
        )
    };
    let range_pills = format!(
        "{}{}{}{}{}",
        range_btn("24h", "Last 24h"),
        range_btn("7d", "7 Days"),
        range_btn("30d", "30 Days"),
        range_btn("90d", "90 Days"),
        range_btn("12m", "12 Months"),
    );

    // Build SVG time-series chart from `summary.buckets`
    let max_val = summary
        .buckets
        .iter()
        .map(|b| b.pageviews.max(b.visitors))
        .max()
        .unwrap_or(1)
        .max(1);
    let n_pts = summary.buckets.len().max(1);
    let chart_w = 860.0_f64;
    let chart_h = 200.0_f64;
    let pad_x = 24.0_f64;
    let pad_y = 20.0_f64;
    let usable_w = chart_w - pad_x * 2.0;
    let usable_h = chart_h - pad_y * 2.0;

    let mut pv_points = Vec::new();
    let mut vis_points = Vec::new();
    let mut circles_svg = String::new();

    for (idx, b) in summary.buckets.iter().enumerate() {
        let x = if n_pts <= 1 {
            pad_x + usable_w / 2.0
        } else {
            pad_x + (idx as f64 / (n_pts - 1) as f64) * usable_w
        };
        let y_pv = pad_y + usable_h - (b.pageviews as f64 / max_val as f64) * usable_h;
        let y_vis = pad_y + usable_h - (b.visitors as f64 / max_val as f64) * usable_h;
        pv_points.push(format!("{:.1},{:.1}", x, y_pv));
        vis_points.push(format!("{:.1},{:.1}", x, y_vis));
        let b_esc = crate::templates::utils::escape_html(&b.bucket);
        circles_svg.push_str(&format!(
            r#"<circle cx="{x:.1}" cy="{y_pv:.1}" r="3.5" fill="var(--press, #a8401b)"><title>{b_esc}: {pv} pageviews, {vis} visitors, {conv} conversions</title></circle>"#,
            x = x,
            y_pv = y_pv,
            b_esc = b_esc,
            pv = b.pageviews,
            vis = b.visitors,
            conv = b.conversions
        ));
    }

    let pv_polyline = pv_points.join(" ");
    let vis_polyline = vis_points.join(" ");
    let first_bucket = summary
        .buckets
        .first()
        .map(|b| crate::templates::utils::escape_html(&b.bucket))
        .unwrap_or_default();
    let last_bucket = summary
        .buckets
        .last()
        .map(|b| crate::templates::utils::escape_html(&b.bucket))
        .unwrap_or_default();

    let mut top_pages_rows = String::new();
    for p in &summary.top_pages {
        let url_esc = crate::templates::utils::escape_html(&p.page_url);
        top_pages_rows.push_str(&format!(
            r#"<tr>
                <td style="padding: 10px 12px; font-family: var(--font-mono); font-size: 12.5px;"><a href="{url_esc}" target="_blank" style="color: var(--press); text-decoration: none;">{url_esc} &#8599;</a></td>
                <td style="padding: 10px 12px; text-align: right; color: var(--ink-muted);">{visitors}</td>
                <td style="padding: 10px 12px; font-weight: 700; text-align: right;">{views}</td>
            </tr>"#,
            url_esc = url_esc,
            visitors = p.visitors,
            views = p.views
        ));
    }
    if summary.top_pages.is_empty() {
        top_pages_rows = r#"<tr><td colspan="3" style="padding: 20px; text-align: center; color: var(--ink-muted);">No pageviews recorded in this range yet.</td></tr>"#.to_string();
    }

    let render_breakdown_rows = |items: &[models::BreakdownMetric], empty_msg: &str| -> String {
        if items.is_empty() {
            return format!(
                r#"<tr><td colspan="3" style="padding: 20px; text-align: center; color: var(--ink-muted);">{empty_msg}</td></tr>"#
            );
        }
        let mut out = String::new();
        for item in items {
            let lbl = crate::templates::utils::escape_html(&item.label);
            out.push_str(&format!(
                r#"<tr>
                    <td style="padding: 10px 12px; font-size: 13px; font-weight: 500;">{lbl}</td>
                    <td style="padding: 10px 12px; text-align: right; color: var(--ink-muted);">{vis}</td>
                    <td style="padding: 10px 12px; font-weight: 700; text-align: right;">{vws}</td>
                </tr>"#,
                lbl = lbl,
                vis = item.visitors,
                vws = item.views
            ));
        }
        out
    };

    let referrer_rows = render_breakdown_rows(&summary.top_referrers, "No referrers recorded yet.");
    let utm_source_rows = render_breakdown_rows(&summary.utm_sources, "No ?utm_source= hits yet.");
    let device_rows = render_breakdown_rows(&summary.devices, "No device telemetry yet.");
    let country_rows = render_breakdown_rows(&summary.countries, "No country data yet.");

    let mut conversion_rows = String::new();
    for c in &summary.conversions {
        let g_esc = crate::templates::utils::escape_html(&c.goal_name);
        conversion_rows.push_str(&format!(
            r#"<tr>
                <td style="padding: 10px 12px; font-weight: 600;">{g_esc}</td>
                <td style="padding: 10px 12px; text-align: right;">{cnt}</td>
                <td style="padding: 10px 12px; text-align: right; color: var(--moss); font-weight: 700;">{rate:.1}%</td>
            </tr>"#,
            g_esc = g_esc,
            cnt = c.count,
            rate = c.conversion_rate_pct
        ));
    }
    if summary.conversions.is_empty() {
        conversion_rows = r#"<tr><td colspan="3" style="padding: 20px; text-align: center; color: var(--ink-muted);">No form submissions or goal events in this range yet. Add <code>data-dp-goal="Signup"</code> to any button or submit a FormStudio form.</td></tr>"#.to_string();
    }

    let content = format!(
        r##"
        <div class="page-header" style="display: flex; justify-content: space-between; align-items: flex-start; margin-bottom: 24px; flex-wrap: wrap; gap: 12px;">
            <div>
                <div style="display: flex; align-items: center; gap: 10px;">
                    <h1 class="page-title">Cookieless Analytics</h1>
                    <span class="badge badge-published" style="font-size: 11px;">GDPR / ePrivacy Native</span>
                </div>
                <p class="page-subtitle">Plausible-style visitor &amp; conversion telemetry via daily-rotating salted SHA-256 hashes. Zero cookies, zero cross-site tracking.</p>
            </div>
            <div style="display: flex; gap: 6px; flex-wrap: wrap; align-items: center;">
                {range_pills}
            </div>
        </div>

        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; margin-bottom: 24px;">
            <div class="day-card" style="padding: 20px;">
                <div style="font-size: 12px; color: var(--ink-muted); font-weight: 600; text-transform: uppercase;">Unique Daily Visitors ({range_label})</div>
                <div style="font-size: 32px; font-weight: 800; color: var(--ink); margin-top: 6px;">{visitors}</div>
                <div style="font-size: 12px; color: var(--moss); margin-top: 4px;">&#10003; Daily Salted SHA-256</div>
            </div>
            <div class="day-card" style="padding: 20px;">
                <div style="font-size: 12px; color: var(--ink-muted); font-weight: 600; text-transform: uppercase;">Total Pageviews</div>
                <div style="font-size: 32px; font-weight: 800; color: var(--press); margin-top: 6px;">{pageviews}</div>
                <div style="font-size: 12px; color: var(--ink-muted); margin-top: 4px;">Direct SQLite Ingest</div>
            </div>
            <div class="day-card" style="padding: 20px;">
                <div style="font-size: 12px; color: var(--ink-muted); font-weight: 600; text-transform: uppercase;">Bounce Rate</div>
                <div style="font-size: 32px; font-weight: 800; color: var(--ink); margin-top: 6px;">{bounce:.1}%</div>
                <div style="font-size: 12px; color: var(--ink-muted); margin-top: 4px;">Single-page sessions</div>
            </div>
            <div class="day-card" style="padding: 20px;">
                <div style="font-size: 12px; color: var(--ink-muted); font-weight: 600; text-transform: uppercase;">Goal &amp; Form Conversions</div>
                <div style="font-size: 32px; font-weight: 800; color: var(--moss); margin-top: 6px;">{conversions}</div>
                <div style="font-size: 12px; color: var(--ink-muted); margin-top: 4px;">FormStudio &amp; custom goals</div>
            </div>
        </div>

        <div class="day-card" style="padding: 24px; margin-bottom: 24px;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                <h2 style="font-size: 15px; font-weight: 700; margin: 0; color: var(--ink);">Traffic Over Time ({range_label})</h2>
                <div style="display: flex; gap: 16px; font-size: 12px; color: var(--ink-muted);">
                    <span><strong style="color: var(--press);">&bull;</strong> Pageviews</span>
                    <span><strong style="color: var(--moss);">&bull;</strong> Unique Visitors</span>
                </div>
            </div>
            <svg viewBox="0 0 {chart_w} {chart_h}" style="width: 100%; height: 210px; overflow: visible;">
                <line x1="{pad_x}" y1="{bottom_y}" x2="{right_x}" y2="{bottom_y}" stroke="var(--border, #e5e7eb)" stroke-width="1" />
                <polyline fill="none" stroke="var(--moss, #2b6e46)" stroke-width="2" stroke-dasharray="4 3" points="{vis_polyline}" />
                <polyline fill="none" stroke="var(--press, #a8401b)" stroke-width="2.5" points="{pv_polyline}" />
                {circles_svg}
            </svg>
            <div style="display: flex; justify-content: space-between; font-size: 11.5px; color: var(--ink-muted); font-family: var(--font-mono); margin-top: 4px;">
                <span>{first_bucket}</span>
                <span>Peak: {max_val} views/bucket</span>
                <span>{last_bucket}</span>
            </div>
        </div>

        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(380px, 1fr)); gap: 20px; margin-bottom: 24px;">
            <div class="day-card" style="padding: 20px;">
                <h2 style="font-size: 15px; font-weight: 700; margin: 0 0 12px 0; color: var(--ink);">Top Pages</h2>
                <table style="width: 100%; border-collapse: collapse;">
                    <thead>
                        <tr style="border-bottom: 1px solid var(--border); text-align: left; font-size: 11.5px; color: var(--ink-muted); text-transform: uppercase;">
                            <th style="padding: 8px 12px;">Page Path</th>
                            <th style="padding: 8px 12px; text-align: right;">Visitors</th>
                            <th style="padding: 8px 12px; text-align: right;">Views</th>
                        </tr>
                    </thead>
                    <tbody>{top_pages_rows}</tbody>
                </table>
            </div>

            <div class="day-card" style="padding: 20px;">
                <h2 style="font-size: 15px; font-weight: 700; margin: 0 0 12px 0; color: var(--ink);">Top Referrers</h2>
                <table style="width: 100%; border-collapse: collapse;">
                    <thead>
                        <tr style="border-bottom: 1px solid var(--border); text-align: left; font-size: 11.5px; color: var(--ink-muted); text-transform: uppercase;">
                            <th style="padding: 8px 12px;">Source / Referrer</th>
                            <th style="padding: 8px 12px; text-align: right;">Visitors</th>
                            <th style="padding: 8px 12px; text-align: right;">Views</th>
                        </tr>
                    </thead>
                    <tbody>{referrer_rows}</tbody>
                </table>
            </div>

            <div class="day-card" style="padding: 20px;">
                <h2 style="font-size: 15px; font-weight: 700; margin: 0 0 12px 0; color: var(--ink);">UTM Sources</h2>
                <table style="width: 100%; border-collapse: collapse;">
                    <thead>
                        <tr style="border-bottom: 1px solid var(--border); text-align: left; font-size: 11.5px; color: var(--ink-muted); text-transform: uppercase;">
                            <th style="padding: 8px 12px;">UTM Source</th>
                            <th style="padding: 8px 12px; text-align: right;">Visitors</th>
                            <th style="padding: 8px 12px; text-align: right;">Views</th>
                        </tr>
                    </thead>
                    <tbody>{utm_source_rows}</tbody>
                </table>
            </div>

            <div class="day-card" style="padding: 20px;">
                <h2 style="font-size: 15px; font-weight: 700; margin: 0 0 12px 0; color: var(--ink);">Goal &amp; FormStudio Conversions</h2>
                <table style="width: 100%; border-collapse: collapse;">
                    <thead>
                        <tr style="border-bottom: 1px solid var(--border); text-align: left; font-size: 11.5px; color: var(--ink-muted); text-transform: uppercase;">
                            <th style="padding: 8px 12px;">Goal / Form</th>
                            <th style="padding: 8px 12px; text-align: right;">Conversions</th>
                            <th style="padding: 8px 12px; text-align: right;">Conv. Rate</th>
                        </tr>
                    </thead>
                    <tbody>{conversion_rows}</tbody>
                </table>
            </div>

            <div class="day-card" style="padding: 20px;">
                <h2 style="font-size: 15px; font-weight: 700; margin: 0 0 12px 0; color: var(--ink);">Devices</h2>
                <table style="width: 100%; border-collapse: collapse;">
                    <thead>
                        <tr style="border-bottom: 1px solid var(--border); text-align: left; font-size: 11.5px; color: var(--ink-muted); text-transform: uppercase;">
                            <th style="padding: 8px 12px;">Device Type</th>
                            <th style="padding: 8px 12px; text-align: right;">Visitors</th>
                            <th style="padding: 8px 12px; text-align: right;">Views</th>
                        </tr>
                    </thead>
                    <tbody>{device_rows}</tbody>
                </table>
            </div>

            <div class="day-card" style="padding: 20px;">
                <h2 style="font-size: 15px; font-weight: 700; margin: 0 0 12px 0; color: var(--ink);">Countries</h2>
                <table style="width: 100%; border-collapse: collapse;">
                    <thead>
                        <tr style="border-bottom: 1px solid var(--border); text-align: left; font-size: 11.5px; color: var(--ink-muted); text-transform: uppercase;">
                            <th style="padding: 8px 12px;">Country (ISO)</th>
                            <th style="padding: 8px 12px; text-align: right;">Visitors</th>
                            <th style="padding: 8px 12px; text-align: right;">Views</th>
                        </tr>
                    </thead>
                    <tbody>{country_rows}</tbody>
                </table>
            </div>
        </div>
        "##,
        range_pills = range_pills,
        range_label = summary.range,
        visitors = summary.total_visitors,
        pageviews = summary.total_pageviews,
        bounce = summary.bounce_rate_pct,
        conversions = summary.total_conversions,
        chart_w = chart_w,
        chart_h = chart_h,
        pad_x = pad_x,
        bottom_y = chart_h - pad_y,
        right_x = chart_w - pad_x,
        vis_polyline = vis_polyline,
        pv_polyline = pv_polyline,
        circles_svg = circles_svg,
        first_bucket = first_bucket,
        last_bucket = last_bucket,
        max_val = max_val,
        top_pages_rows = top_pages_rows,
        referrer_rows = referrer_rows,
        utm_source_rows = utm_source_rows,
        conversion_rows = conversion_rows,
        device_rows = device_rows,
        country_rows = country_rows,
    );

    Html(templates::render_admin_shell(
        "analytics",
        "Cookieless Analytics",
        &content,
        &settings,
    ))
}

// =========================================================================
// FIRST-PARTY MODULES & COMMUNITY PLUGIN ECOSYSTEM HANDLERS
// =========================================================================

pub async fn handle_admin_seo(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let seo = db.get_global_seo_settings().unwrap_or_default();
    let redirects = db.list_redirects().unwrap_or_default();
    let logs_404 = db.list_404_logs(100).unwrap_or_default();
    Html(templates::render_seo_view(
        &settings, &seo, &redirects, &logs_404,
    ))
}

pub async fn handle_api_seo_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<models::GlobalSeoSettings>,
) -> impl IntoResponse {
    match db.save_global_seo_settings(&payload) {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true, "seo": payload })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct RedirectActionPayload {
    pub action: Option<String>,
    pub id: Option<i64>,
    pub source_path: Option<String>,
    pub target_url: Option<String>,
    pub status_code: Option<i64>,
}

pub async fn handle_api_redirects(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<RedirectActionPayload>,
) -> impl IntoResponse {
    let action = payload.action.as_deref().unwrap_or("create");
    match action {
        "delete" => {
            if let Some(id) = payload.id {
                let _ = db.delete_redirect(id);
            }
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        "clear_404" => {
            let _ = db.clear_404_logs();
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        _ => {
            let req = models::CreateRedirectRequest {
                source_path: payload.source_path.unwrap_or_default(),
                target_url: payload.target_url.unwrap_or_else(|| "/".to_string()),
                status_code: payload.status_code.unwrap_or(301),
            };
            match db.create_redirect(&req) {
                Ok(rule) => (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({ "success": true, "redirect": rule })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct FormsQueryParams {
    pub form_id: Option<i64>,
    pub status: Option<String>,
    pub s: Option<String>,
}

pub async fn handle_admin_forms(
    State(db): State<Database>,
    Query(params): Query<FormsQueryParams>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let forms = db.list_forms().unwrap_or_default();
    let entries = db
        .list_form_entries_filtered(
            params.form_id,
            params.status.as_deref(),
            params.s.as_deref(),
            300,
        )
        .unwrap_or_default();
    Html(templates::render_forms_view_with_filters(
        &settings,
        &forms,
        &entries,
        params.form_id,
        params.status.as_deref().unwrap_or("all"),
        params.s.as_deref().unwrap_or(""),
    ))
}

#[derive(Debug, Deserialize, Default)]
pub struct FormsActionPayload {
    pub action: Option<String>,
    pub id: Option<i64>,
    pub ids: Option<Vec<i64>>,
    pub status: Option<String>,
    pub is_starred: Option<bool>,
    pub title: Option<String>,
    pub fields: Option<Vec<models::FormFieldSchema>>,
    pub settings: Option<models::FormSettings>,
}

pub async fn handle_api_forms(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<FormsActionPayload>,
) -> impl IntoResponse {
    let action = payload.action.as_deref().unwrap_or("save_form");
    match action {
        "delete_form" => {
            if let Some(id) = payload.id {
                let _ = db.delete_form(id);
            }
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        "delete_entry" => {
            if let Some(id) = payload.id {
                let _ = db.delete_form_entry(id);
            }
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        "update_entry_status" => {
            if let Some(id) = payload.id {
                let _ = db.update_form_entry_workflow(
                    id,
                    payload.status.as_deref(),
                    payload.is_starred,
                );
            }
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        "toggle_entry_star" => {
            if let Some(id) = payload.id {
                let next_star = match payload.is_starred {
                    Some(v) => v,
                    None => db
                        .get_form_entry_by_id(id)
                        .ok()
                        .flatten()
                        .map(|e| !e.is_starred)
                        .unwrap_or(true),
                };
                let _ = db.update_form_entry_workflow(id, None, Some(next_star));
                return (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({ "success": true, "is_starred": next_star })),
                )
                    .into_response();
            }
            (
                StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Missing entry id" })),
            )
                .into_response()
        }
        "bulk_entries" => {
            let ids = payload.ids.unwrap_or_default();
            let target_status = payload.status.as_deref().unwrap_or("read");
            for eid in ids {
                if target_status == "delete" {
                    let _ = db.delete_form_entry(eid);
                } else {
                    let _ = db.update_form_entry_workflow(eid, Some(target_status), None);
                }
            }
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true })),
            )
                .into_response()
        }
        "retry_email" => {
            let Some(entry_id) = payload.id else {
                return (
                    StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({ "error": "Missing entry id" })),
                )
                    .into_response();
            };
            let Some(entry) = db.get_form_entry_by_id(entry_id).ok().flatten() else {
                return (
                    StatusCode::NOT_FOUND,
                    axum::Json(serde_json::json!({ "error": "Form entry not found" })),
                )
                    .into_response();
            };
            let form_rec =
                db.get_form(entry.form_id)
                    .ok()
                    .flatten()
                    .unwrap_or(models::DaypressForm {
                        id: entry.form_id,
                        title: entry.form_title.clone(),
                        schema_json: "[]".to_string(),
                        settings_json: "{}".to_string(),
                        created_at: entry.created_at.clone(),
                        updated_at: entry.created_at.clone(),
                    });
            let data_map: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(&entry.data_json).unwrap_or_default();
            let (email_status, email_error, webhook_status) =
                crate::api::daypress_v1::deliver_form_submission_notifications(
                    &db,
                    &form_rec,
                    entry_id,
                    &data_map,
                    &entry.attachments,
                    &entry.source_url,
                )
                .await;
            (
                StatusCode::OK,
                axum::Json(serde_json::json!({
                    "success": email_status == "sent",
                    "entry_id": entry_id,
                    "email_status": email_status,
                    "email_error": email_error,
                    "webhook_status": webhook_status
                })),
            )
                .into_response()
        }
        _ => {
            let req = models::CreateFormRequest {
                id: payload.id,
                title: payload.title.unwrap_or_else(|| "Contact Form".to_string()),
                fields: payload.fields.unwrap_or_default(),
                settings: payload.settings.unwrap_or_default(),
            };
            match db.save_form_record(&req) {
                Ok(form) => (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({ "success": true, "form": form })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
    }
}

pub async fn handle_admin_forms_export_csv(
    State(db): State<Database>,
    Query(params): Query<FormsQueryParams>,
) -> impl IntoResponse {
    let entries = db
        .list_form_entries_filtered(
            params.form_id,
            params.status.as_deref(),
            params.s.as_deref(),
            5000,
        )
        .unwrap_or_default();
    let mut csv = String::from(
        "Entry ID,Form ID,Form Title,Status,Starred,Email Status,Webhook Status,Source URL,Created At,IP Hash,Payload JSON\n",
    );
    for e in entries {
        let escaped_json = e.data_json.replace('"', "\"\"");
        let escaped_title = e.form_title.replace('"', "\"\"");
        let escaped_source = e.source_url.replace('"', "\"\"");
        csv.push_str(&format!(
            "{},{},\"{}\",\"{}\",{},\"{}\",\"{}\",\"{}\",\"{}\",\"{}\",\"{}\"\n",
            e.id,
            e.form_id,
            escaped_title,
            e.status,
            if e.is_starred { "yes" } else { "no" },
            e.email_status,
            e.webhook_status.as_deref().unwrap_or("none"),
            escaped_source,
            e.created_at,
            e.ip_hash,
            escaped_json
        ));
    }
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("text/csv; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_static("attachment; filename=\"daypress-form-entries.csv\""),
    );
    (StatusCode::OK, headers, csv)
}

pub async fn handle_admin_forms_export_json(
    State(db): State<Database>,
    Query(params): Query<FormsQueryParams>,
) -> impl IntoResponse {
    let entries = db
        .list_form_entries_filtered(
            params.form_id,
            params.status.as_deref(),
            params.s.as_deref(),
            5000,
        )
        .unwrap_or_default();
    let body = serde_json::to_string_pretty(&entries).unwrap_or_else(|_| "[]".to_string());
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json; charset=utf-8"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        header::HeaderValue::from_static("attachment; filename=\"daypress-form-entries.json\""),
    );
    (StatusCode::OK, headers, body)
}

pub async fn handle_admin_form_attachment_download(
    State(db): State<Database>,
    AxPath((entry_id, filename)): AxPath<(i64, String)>,
) -> impl IntoResponse {
    let Some(safe_name) = crate::media::sanitize_filename(&filename) else {
        return (StatusCode::BAD_REQUEST, "Invalid attachment filename").into_response();
    };
    let Some(entry) = db.get_form_entry_by_id(entry_id).ok().flatten() else {
        return (StatusCode::NOT_FOUND, "Form submission not found").into_response();
    };
    let Some(att) = entry.attachments.iter().find(|a| a.filename == safe_name) else {
        return (StatusCode::NOT_FOUND, "Attachment not found on entry").into_response();
    };
    let path = std::path::Path::new(&att.stored_path);
    if !path.starts_with("uploads/form-submissions") || !path.exists() {
        return (StatusCode::NOT_FOUND, "Attachment file missing from disk").into_response();
    }
    match std::fs::read(path) {
        Ok(bytes) => {
            let mut headers = HeaderMap::new();
            if let Ok(ct) = att.mime_type.parse() {
                headers.insert(header::CONTENT_TYPE, ct);
            } else {
                headers.insert(
                    header::CONTENT_TYPE,
                    header::HeaderValue::from_static("application/octet-stream"),
                );
            }
            if let Ok(disp) = format!("attachment; filename=\"{}\"", safe_name).parse() {
                headers.insert(header::CONTENT_DISPOSITION, disp);
            }
            (StatusCode::OK, headers, bytes).into_response()
        }
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to read attachment",
        )
            .into_response(),
    }
}

pub async fn handle_admin_backups(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let snapshots = db.list_backup_snapshots().unwrap_or_default();
    Html(templates::render_backups_view(&settings, &snapshots))
}

#[derive(Debug, Deserialize)]
pub struct BackupActionPayload {
    pub action: String,
    pub label: Option<String>,
    pub filename: Option<String>,
}

pub async fn handle_api_backups(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<BackupActionPayload>,
) -> impl IntoResponse {
    match payload.action.as_str() {
        "create" => match db.create_sqlite_snapshot(payload.label.as_deref()) {
            Ok(snap) => (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true, "snapshot": snap })),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        },
        "delete" => {
            let fname = payload.filename.unwrap_or_default();
            match db.delete_backup_snapshot(&fname) {
                Ok(deleted) => (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({ "success": deleted })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        "restore" => {
            let fname = payload.filename.unwrap_or_default();
            match db.restore_sqlite_snapshot(&fname) {
                Ok(_) => (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({
                        "success": true,
                        "message": format!("Snapshot {} restored into live SQLite database.", fname)
                    })),
                )
                    .into_response(),
                Err(e) => (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response(),
            }
        }
        _ => (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Unknown backup action" })),
        )
            .into_response(),
    }
}

pub async fn handle_admin_backup_download(AxPath(filename): AxPath<String>) -> impl IntoResponse {
    let safe_name: String = filename
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_' || *c == '.')
        .collect();
    let path = std::path::Path::new("backups").join(&safe_name);
    if !path.exists() {
        return (StatusCode::NOT_FOUND, "Snapshot not found").into_response();
    }
    match std::fs::read(&path) {
        Ok(bytes) => {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::CONTENT_TYPE,
                "application/octet-stream".parse().unwrap(),
            );
            if let Ok(disp) = format!("attachment; filename=\"{}\"", safe_name).parse() {
                headers.insert(header::CONTENT_DISPOSITION, disp);
            }
            (StatusCode::OK, headers, bytes).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read snapshot").into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct PluginSettingsQuery {
    pub plugin: Option<String>,
}

pub async fn handle_admin_plugin_settings(
    State(db): State<Database>,
    Query(params): Query<PluginSettingsQuery>,
) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let slug = params.plugin.unwrap_or_else(|| "whatsapp-chat".to_string());
    let plugin = db.get_installed_plugin(&slug).ok().flatten();
    Html(templates::render_plugin_settings_view(
        &settings,
        plugin.as_ref(),
    ))
}

#[derive(Debug, Deserialize)]
pub struct SavePluginConfigPayload {
    pub slug: String,
    pub config: std::collections::HashMap<String, String>,
}

pub async fn handle_api_plugin_settings_save(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<SavePluginConfigPayload>,
) -> impl IntoResponse {
    match db.save_plugin_config(&payload.slug, &payload.config) {
        Ok(_) => (
            StatusCode::OK,
            axum::Json(serde_json::json!({ "success": true })),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": e.to_string() })),
        )
            .into_response(),
    }
}

#[derive(Debug, Deserialize)]
pub struct CustomPluginActionPayload {
    pub action: Option<String>,
    pub slug: Option<String>,
    pub manifest: Option<models::PluginManifest>,
}

pub async fn handle_api_plugin_install_custom(
    State(db): State<Database>,
    axum::Json(payload): axum::Json<CustomPluginActionPayload>,
) -> impl IntoResponse {
    if payload.action.as_deref() == Some("uninstall") {
        let slug = payload.slug.unwrap_or_default();
        match db.uninstall_custom_plugin(&slug) {
            Ok(removed) => {
                return (
                    StatusCode::OK,
                    axum::Json(serde_json::json!({ "success": removed })),
                )
                    .into_response()
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    axum::Json(serde_json::json!({ "error": e.to_string() })),
                )
                    .into_response()
            }
        }
    }

    if let Some(manifest) = payload.manifest {
        match db.install_custom_plugin(&manifest, None) {
            Ok(_) => (
                StatusCode::OK,
                axum::Json(serde_json::json!({ "success": true, "slug": manifest.slug })),
            )
                .into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": e.to_string() })),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Missing plugin manifest" })),
        )
            .into_response()
    }
}

pub async fn handle_api_plugin_sdk_template() -> impl IntoResponse {
    let tpl = crate::plugins::starter_plugin_sdk_template();
    let json = serde_json::to_string_pretty(&tpl).unwrap_or_else(|_| "{}".to_string());
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/json; charset=utf-8".parse().unwrap(),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"daypress-plugin-starter.json\""
            .parse()
            .unwrap(),
    );
    (StatusCode::OK, headers, json)
}

pub async fn handle_api_download_migrator_plugin() -> impl IntoResponse {
    let zip_bytes = match std::fs::read("daypress-migrator.zip") {
        Ok(b) => b,
        Err(_) => include_bytes!("../../../../daypress-migrator.zip").to_vec(),
    };
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, "application/zip".parse().unwrap());
    headers.insert(
        header::CONTENT_DISPOSITION,
        "attachment; filename=\"daypress-migrator.zip\""
            .parse()
            .unwrap(),
    );
    (StatusCode::OK, headers, zip_bytes)
}

// =========================================================================
// MULTILINGUAL (i18n) & LANGUAGES ADMIN PORTAL (/d-admin/languages)
// =========================================================================

pub async fn handle_admin_languages(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let languages = db.list_languages().unwrap_or_default();
    let prefix_default = db.prefix_default_language_enabled();

    let mut lang_rows = String::new();
    for lang in &languages {
        let code_esc = templates::utils::escape_html(&lang.code);
        let name_esc = templates::utils::escape_html(&lang.name);
        let native_esc = templates::utils::escape_html(&lang.native_name);
        let locale_esc = templates::utils::escape_html(&lang.locale);
        let flag_esc = templates::utils::escape_html(&lang.flag_emoji);
        let default_badge = if lang.is_default {
            r#"<span class="badge badge-published">Default</span>"#.to_string()
        } else if lang.is_rtl {
            r#"<span class="badge badge-draft">Secondary</span> <span class="badge" style="background:var(--ochre-wash);color:var(--ink);border:1px solid var(--ochre);margin-left:4px;">RTL</span>"#.to_string()
        } else {
            r#"<span class="badge badge-draft">Secondary</span>"#.to_string()
        };
        let url_pattern = if lang.is_default && !prefix_default {
            "/slug/".to_string()
        } else {
            format!("/{}/slug/", code_esc)
        };
        let actions_html = if lang.is_default {
            r#"<span style="font-size:11.5px;color:var(--ink-muted);">Primary Site Locale</span>"#
                .to_string()
        } else {
            format!(
                r#"<button type="button" class="btn btn-secondary" style="font-size:11.5px;padding:4px 10px;" onclick="setAsDefaultLang('{code_esc}', '{name_esc}', '{native_esc}', '{locale_esc}', '{flag_esc}')">Make Default</button>
                <button type="button" class="btn btn-secondary" style="font-size:11.5px;padding:4px 10px;color:var(--oxide);" onclick="removeLang('{code_esc}')">Remove</button>"#
            )
        };
        lang_rows.push_str(&format!(
            r#"<tr>
                <td style="font-size:18px;width:44px;text-align:center;">{flag_esc}</td>
                <td><strong>{native_esc}</strong> <span style="color:var(--ink-muted);font-size:12px;">({name_esc})</span></td>
                <td><code>{code_esc}</code></td>
                <td><code>{locale_esc}</code></td>
                <td><code>{url_pattern}</code></td>
                <td>{default_badge}</td>
                <td style="text-align:right;display:flex;gap:6px;justify-content:flex-end;">{actions_html}</td>
            </tr>"#
        ));
    }

    let default_lang_code = languages
        .iter()
        .find(|l| l.is_default)
        .map(|l| l.code.clone())
        .unwrap_or_else(|| "en".to_string());
    let mut matrix_headers = String::new();
    for lang in &languages {
        matrix_headers.push_str(&format!(
            r#"<th style="text-align:center;">{} {}</th>"#,
            templates::utils::escape_html(&lang.flag_emoji),
            templates::utils::escape_html(&lang.code.to_uppercase())
        ));
    }
    let mut all_items = db
        .get_posts("page", Some("any"), None, 25)
        .unwrap_or_default();
    let mut recent_posts = db
        .get_posts("post", Some("any"), None, 25)
        .unwrap_or_default();
    all_items.append(&mut recent_posts);
    let primary_items: Vec<_> = all_items
        .iter()
        .filter(|p| {
            p.status != "trash"
                && (p.lang_code.is_empty() || p.lang_code.eq_ignore_ascii_case(&default_lang_code))
        })
        .take(20)
        .collect();
    let mut matrix_rows = String::new();
    for item in primary_items {
        let mut cells = String::new();
        for lang in &languages {
            if lang.code.eq_ignore_ascii_case(&default_lang_code) {
                cells.push_str(&format!(
                    r#"<td style="text-align:center;"><a href="/d-admin/post?post={}&action=edit" class="badge badge-published" style="text-decoration:none;">#{} Source</a></td>"#,
                    item.id, item.id
                ));
            } else if let Some(&target_id) = item.translations.get(&lang.code) {
                cells.push_str(&format!(
                    r#"<td style="text-align:center;"><a href="/d-admin/post?post={}&action=edit" class="badge badge-published" style="text-decoration:none;">✓ #{}</a></td>"#,
                    target_id, target_id
                ));
            } else {
                let lcode = templates::utils::escape_html(&lang.code);
                cells.push_str(&format!(
                    r#"<td style="text-align:center;"><button type="button" class="btn btn-secondary" style="font-size:11px;padding:2px 8px;" onclick="quickCreateTranslation({}, '{}')">+ Translate</button></td>"#,
                    item.id, lcode
                ));
            }
        }
        let title_esc = templates::utils::escape_html(&item.title.rendered);
        let ptype_esc = templates::utils::escape_html(&item.post_type.to_uppercase());
        matrix_rows.push_str(&format!(
            r#"<tr>
                <td><strong>{title_esc}</strong> <span style="font-size:10.5px;color:var(--ink-muted);margin-left:6px;">{ptype_esc}</span></td>
                {cells}
            </tr>"#
        ));
    }
    if matrix_rows.is_empty() {
        matrix_rows = format!(
            r#"<tr><td colspan="{}" style="text-align:center;color:var(--ink-muted);padding:18px;">No primary-language posts or pages yet.</td></tr>"#,
            languages.len() + 1
        );
    }

    let unprefixed_checked = if !prefix_default { "checked" } else { "" };
    let prefixed_checked = if prefix_default { "checked" } else { "" };

    let body = format!(
        r#"
        <div class="page-header">
            <div>
                <h1 class="page-title">Languages &amp; Multilingual (i18n)</h1>
                <p class="page-subtitle">Configure Polylang/WPML-compatible localized URL routing, hreflang SEO tags, and translation groups</p>
            </div>
        </div>

        <div style="display:grid;grid-template-columns:2fr 1fr;gap:24px;align-items:start;">
            <div style="display:flex;flex-direction:column;gap:20px;">
                <div class="day-card">
                    <div class="day-card-header">
                        <h2 class="day-card-title">Configured Site Languages</h2>
                    </div>
                    <div class="day-card-body" style="padding:0;">
                        <table class="day-table">
                            <thead>
                                <tr>
                                    <th>Flag</th>
                                    <th>Language</th>
                                    <th>ISO Code</th>
                                    <th>Locale</th>
                                    <th>URL Prefix</th>
                                    <th>Role</th>
                                    <th style="text-align:right;">Actions</th>
                                </tr>
                            </thead>
                            <tbody>
                                {lang_rows}
                            </tbody>
                        </table>
                    </div>
                </div>

                <div class="day-card">
                    <div class="day-card-header">
                        <h2 class="day-card-title">URL Routing &amp; hreflang SEO Strategy</h2>
                    </div>
                    <div class="day-card-body">
                        <div style="display:flex;flex-direction:column;gap:10px;margin-bottom:14px;">
                            <label style="display:flex;align-items:flex-start;gap:10px;cursor:pointer;">
                                <input type="radio" name="i18n_prefix_mode" value="false" {unprefixed_checked} onchange="savePrefixMode(false)" style="margin-top:3px;" />
                                <div>
                                    <div style="font-weight:600;font-size:13.5px;">Unprefixed Default Locale (Recommended — Polylang / WPML Default)</div>
                                    <div style="font-size:12px;color:var(--ink-muted);">Default language serves at <code>/slug/</code>; secondary languages serve at <code>/{{lang}}/{{slug}}/</code> (e.g. <code>/de/willkommen/</code>).</div>
                                </div>
                            </label>
                            <label style="display:flex;align-items:flex-start;gap:10px;cursor:pointer;">
                                <input type="radio" name="i18n_prefix_mode" value="true" {prefixed_checked} onchange="savePrefixMode(true)" style="margin-top:3px;" />
                                <div>
                                    <div style="font-weight:600;font-size:13.5px;">Prefix All Locales Including Default</div>
                                    <div style="font-size:12px;color:var(--ink-muted);">Every language uses its ISO prefix: <code>/en/slug/</code>, <code>/de/slug/</code>, <code>/nl/slug/</code>.</div>
                                </div>
                            </label>
                        </div>
                        <div style="font-size:12px;color:var(--ink-muted);background:var(--paper-sunk);padding:10px 12px;border-radius:6px;">
                            <strong>Automatic SEO &amp; Sitemap:</strong> Linked translations automatically emit <code>&lt;link rel="alternate" hreflang="..."&gt;</code> (plus <code>x-default</code>) in SSR <code>&lt;head&gt;</code> and <code>&lt;xhtml:link rel="alternate"&gt;</code> in <code>/sitemap.xml</code>. Use <code>[daypress_language_switcher]</code> or the DayStudio <strong>Language Switcher</strong> widget anywhere.
                        </div>
                    </div>
                </div>

                <div class="day-card">
                    <div class="day-card-header">
                        <h2 class="day-card-title">Translation Coverage Matrix</h2>
                    </div>
                    <div class="day-card-body" style="padding:0;overflow-x:auto;">
                        <table class="day-table">
                            <thead>
                                <tr>
                                    <th>Content Item</th>
                                    {matrix_headers}
                                </tr>
                            </thead>
                            <tbody>
                                {matrix_rows}
                            </tbody>
                        </table>
                    </div>
                </div>
            </div>

            <div style="display:flex;flex-direction:column;gap:20px;">
                <div class="day-card">
                    <div class="day-card-header">
                        <h2 class="day-card-title">Add Language</h2>
                    </div>
                    <div class="day-card-body">
                        <div class="form-group">
                            <label class="form-label">Quick Preset</label>
                            <select id="langPresetSelect" class="form-select" onchange="applyLangPreset(this.value)">
                                <option value="">Choose a language preset...</option>
                                <option value="de|German|Deutsch|de_DE|🇩🇪">🇩🇪 Deutsch (de_DE)</option>
                                <option value="nl|Dutch|Nederlands|nl_NL|🇳🇱">🇳🇱 Nederlands (nl_NL)</option>
                                <option value="fr|French|Français|fr_FR|🇫🇷">🇫🇷 Français (fr_FR)</option>
                                <option value="es|Spanish|Español|es_ES|🇪🇸">🇪🇸 Español (es_ES)</option>
                                <option value="it|Italian|Italiano|it_IT|🇮🇹">🇮🇹 Italiano (it_IT)</option>
                                <option value="pt|Portuguese|Português|pt_PT|🇵🇹">🇵🇹 Português (pt_PT)</option>
                                <option value="pl|Polish|Polski|pl_PL|🇵🇱">🇵🇱 Polski (pl_PL)</option>
                                <option value="sv|Swedish|Svenska|sv_SE|🇸🇪">🇸🇪 Svenska (sv_SE)</option>
                                <option value="ja|Japanese|日本語|ja_JP|🇯🇵">🇯🇵 日本語 (ja_JP)</option>
                                <option value="ar|Arabic|العربية|ar|🇸🇦">🇸🇦 العربية (ar)</option>
                            </select>
                        </div>
                        <div style="display:grid;grid-template-columns:1fr 1fr;gap:10px;">
                            <div class="form-group">
                                <label class="form-label">ISO Code</label>
                                <input type="text" id="newLangCode" class="form-input" placeholder="de" />
                            </div>
                            <div class="form-group">
                                <label class="form-label">Flag Emoji</label>
                                <input type="text" id="newLangFlag" class="form-input" placeholder="🇩🇪" />
                            </div>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Native Name</label>
                            <input type="text" id="newLangNative" class="form-input" placeholder="Deutsch" />
                        </div>
                        <div class="form-group">
                            <label class="form-label">English Name &amp; Locale</label>
                            <div style="display:grid;grid-template-columns:1fr 1fr;gap:8px;">
                                <input type="text" id="newLangName" class="form-input" placeholder="German" />
                                <input type="text" id="newLangLocale" class="form-input" placeholder="de_DE" />
                            </div>
                        </div>
                        <div class="form-group" style="margin-bottom:12px;">
                            <label style="display:flex;align-items:center;gap:8px;font-size:12.5px;cursor:pointer;">
                                <input type="checkbox" id="newLangRtl" />
                                <span>Right-to-Left (RTL) Script (e.g. Arabic, Hebrew)</span>
                            </label>
                        </div>
                        <button type="button" class="btn btn-primary" style="width:100%;" onclick="addSiteLanguage()">+ Add Language</button>
                    </div>
                </div>

                <div class="day-card">
                    <div class="day-card-header">
                        <h2 class="day-card-title">Link Existing Translations</h2>
                    </div>
                    <div class="day-card-body">
                        <div class="form-group">
                            <label class="form-label">Source Post ID + Lang</label>
                            <div style="display:grid;grid-template-columns:1fr 90px;gap:8px;">
                                <input type="number" id="linkSrcId" class="form-input" placeholder="Post ID (e.g. 1)" />
                                <input type="text" id="linkSrcLang" class="form-input" value="en" placeholder="en" />
                            </div>
                        </div>
                        <div class="form-group">
                            <label class="form-label">Target Post ID + Lang</label>
                            <div style="display:grid;grid-template-columns:1fr 90px;gap:8px;">
                                <input type="number" id="linkTargetId" class="form-input" placeholder="Post ID (e.g. 42)" />
                                <input type="text" id="linkTargetLang" class="form-input" value="de" placeholder="de" />
                            </div>
                        </div>
                        <button type="button" class="btn btn-secondary" style="width:100%;" onclick="linkTranslationsManual()">Link Translation Group</button>
                    </div>
                </div>
            </div>
        </div>

        <script>
            function applyLangPreset(val) {{
                if (!val) return;
                const [code, name, nativeName, locale, flag] = val.split('|');
                document.getElementById('newLangCode').value = code || '';
                document.getElementById('newLangName').value = name || '';
                document.getElementById('newLangNative').value = nativeName || '';
                document.getElementById('newLangLocale').value = locale || '';
                document.getElementById('newLangFlag').value = flag || '🌐';
                const rtlBox = document.getElementById('newLangRtl');
                if (rtlBox) rtlBox.checked = (code === 'ar' || code === 'he' || code === 'fa' || code === 'ur');
            }}

            async function addSiteLanguage() {{
                const code = document.getElementById('newLangCode').value.trim().toLowerCase();
                if (!code) {{ showToast('Please enter an ISO code (e.g. de, nl, fr)', true); return; }}
                const rtlBox = document.getElementById('newLangRtl');
                const payload = {{
                    code,
                    name: document.getElementById('newLangName').value.trim() || code.toUpperCase(),
                    native_name: document.getElementById('newLangNative').value.trim() || code.toUpperCase(),
                    locale: document.getElementById('newLangLocale').value.trim() || code,
                    flag_emoji: document.getElementById('newLangFlag').value.trim() || '🌐',
                    is_rtl: rtlBox ? rtlBox.checked : (code === 'ar' || code === 'he'),
                    is_default: false,
                    is_active: true
                }};
                const res = await fetch('/wp-json/daypress/v1/languages', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify(payload)
                }});
                if (res.ok) {{
                    showToast('✓ Language ' + code.toUpperCase() + ' saved to SQLite');
                    setTimeout(() => window.location.reload(), 400);
                }} else {{
                    showToast('Failed to add language', true);
                }}
            }}

            async function setAsDefaultLang(code, name, native_name, locale, flag_emoji) {{
                const res = await fetch('/wp-json/daypress/v1/languages', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'set_default', code, name, native_name, locale, flag_emoji, is_default: true, is_active: true }})
                }});
                if (res.ok) {{
                    showToast('✓ Default language updated to ' + code.toUpperCase());
                    setTimeout(() => window.location.reload(), 400);
                }}
            }}

            async function removeLang(code) {{
                if (!confirm('Remove language ' + code.toUpperCase() + '?')) return;
                const res = await fetch('/wp-json/daypress/v1/languages/' + encodeURIComponent(code), {{ method: 'DELETE' }});
                if (res.ok) {{
                    showToast('✓ Removed language ' + code.toUpperCase());
                    setTimeout(() => window.location.reload(), 400);
                }} else {{
                    showToast('Cannot delete default language', true);
                }}
            }}

            async function savePrefixMode(prefixDefault) {{
                const res = await fetch('/wp-json/daypress/v1/languages', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ action: 'set_prefix_default', prefix_default: !!prefixDefault }})
                }});
                if (res.ok) {{
                    showToast('✓ URL prefix routing updated in SQLite');
                    setTimeout(() => window.location.reload(), 400);
                }}
            }}

            async function quickCreateTranslation(postId, targetLang) {{
                showToast('Creating ' + targetLang.toUpperCase() + ' translation draft...');
                const res = await fetch('/wp-json/daypress/v1/posts/' + postId + '/translate', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ lang_code: targetLang }})
                }});
                if (res.ok) {{
                    const cloned = await res.json();
                    showToast('✓ Created ' + targetLang.toUpperCase() + ' draft #' + cloned.id);
                    setTimeout(() => {{ window.location.href = '/d-admin/post?post=' + cloned.id + '&action=edit'; }}, 400);
                }} else {{
                    showToast('Failed to create translation draft', true);
                }}
            }}

            async function linkTranslationsManual() {{
                const source_id = parseInt(document.getElementById('linkSrcId').value, 10) || 0;
                const target_id = parseInt(document.getElementById('linkTargetId').value, 10) || 0;
                const source_lang = document.getElementById('linkSrcLang').value.trim() || 'en';
                const target_lang = document.getElementById('linkTargetLang').value.trim() || 'de';
                if (!source_id || !target_id) {{ showToast('Enter both source and target Post IDs', true); return; }}
                const res = await fetch('/wp-json/daypress/v1/translations/link', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ object_type: 'post', source_id, source_lang, target_id, target_lang }})
                }});
                if (res.ok) {{
                    const data = await res.json();
                    showToast('✓ Linked posts into translation group: ' + data.translation_group_id);
                }} else {{
                    showToast('Failed to link translations', true);
                }}
            }}
        </script>
        "#
    );

    Html(templates::render_admin_shell(
        "languages",
        "Languages & Multilingual (i18n)",
        &body,
        &settings,
    ))
}
