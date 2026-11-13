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
        Err(e) => crate::adapters::web::internal_json_error("SEO save failed", e),
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
                Err(e) => crate::adapters::web::internal_json_error("Redirect creation failed", e),
            }
        }
    }
}
