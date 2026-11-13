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
    pub page: Option<String>,
    pub referrer: Option<String>,
    pub channel: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub device: Option<String>,
    pub browser: Option<String>,
    pub os: Option<String>,
    pub country: Option<String>,
    pub goal: Option<String>,
}

impl AnalyticsDashboardQuery {
    fn to_filter(&self) -> models::AnalyticsFilter {
        models::AnalyticsFilter {
            page: self.page.clone(),
            referrer: self.referrer.clone(),
            channel: self.channel.clone(),
            utm_source: self.utm_source.clone(),
            utm_medium: self.utm_medium.clone(),
            utm_campaign: self.utm_campaign.clone(),
            device: self.device.clone(),
            browser: self.browser.clone(),
            os: self.os.clone(),
            country: self.country.clone(),
            goal: self.goal.clone(),
        }
        .normalized()
    }
}

fn url_encode_param(val: &str) -> String {
    let mut out = String::with_capacity(val.len() + 8);
    for b in val.as_bytes() {
        match *b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn build_analytics_href(range: &str, f: &models::AnalyticsFilter) -> String {
    let mut parts = vec![format!("range={}", url_encode_param(range))];
    let mut push_opt = |k: &str, v: &Option<String>| {
        if let Some(ref s) = v {
            if !s.is_empty() {
                parts.push(format!("{k}={}", url_encode_param(s)));
            }
        }
    };
    push_opt("page", &f.page);
    push_opt("referrer", &f.referrer);
    push_opt("channel", &f.channel);
    push_opt("utm_source", &f.utm_source);
    push_opt("utm_medium", &f.utm_medium);
    push_opt("utm_campaign", &f.utm_campaign);
    push_opt("device", &f.device);
    push_opt("browser", &f.browser);
    push_opt("os", &f.os);
    push_opt("country", &f.country);
    push_opt("goal", &f.goal);
    format!("/d-admin/analytics?{}", parts.join("&"))
}

fn with_toggled_filter(
    base: &models::AnalyticsFilter,
    key: &str,
    val: &str,
) -> models::AnalyticsFilter {
    let mut next = base.clone();
    let toggle = |slot: &mut Option<String>| {
        if slot.as_deref() == Some(val) {
            *slot = None;
        } else {
            *slot = Some(val.to_string());
        }
    };
    match key {
        "page" => toggle(&mut next.page),
        "referrer" => toggle(&mut next.referrer),
        "channel" => toggle(&mut next.channel),
        "utm_source" => toggle(&mut next.utm_source),
        "utm_medium" => toggle(&mut next.utm_medium),
        "utm_campaign" => toggle(&mut next.utm_campaign),
        "device" => toggle(&mut next.device),
        "browser" => toggle(&mut next.browser),
        "os" => toggle(&mut next.os),
        "country" => toggle(&mut next.country),
        "goal" => toggle(&mut next.goal),
        _ => {}
    }
    next
}

fn format_int(n: i64) -> String {
    let neg = n < 0;
    let digits = n.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    if neg {
        out.push('-');
    }
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

fn format_duration_human(secs: f64) -> String {
    let total = secs.round() as i64;
    if total <= 0 {
        return "—".to_string();
    }
    if total < 60 {
        format!("{total}s")
    } else {
        let mins = total / 60;
        let rem = total % 60;
        format!("{mins}m {rem:02}s")
    }
}

fn render_delta_badge(curr: f64, prev: f64, invert_good: bool) -> String {
    if prev <= 0.0001 {
        if curr > 0.0 {
            return r#"<span class="dp-an-delta dp-an-delta-neutral" title="No prior baseline in previous window">New</span>"#.to_string();
        }
        return r#"<span class="dp-an-delta dp-an-delta-neutral">—</span>"#.to_string();
    }
    let pct = ((curr - prev) / prev) * 100.0;
    if pct.abs() < 0.15 {
        return r#"<span class="dp-an-delta dp-an-delta-neutral">0.0%</span>"#.to_string();
    }
    let is_up = pct > 0.0;
    let is_good = if invert_good { !is_up } else { is_up };
    let cls = if is_good {
        "dp-an-delta-good"
    } else {
        "dp-an-delta-bad"
    };
    let arrow = if is_up { "&#9650;" } else { "&#9660;" };
    let sign = if is_up { "+" } else { "" };
    format!(
        r#"<span class="dp-an-delta {cls}" title="Compared to previous period">{arrow} {sign}{pct:.1}%</span>"#
    )
}

fn format_country_display(code: &str) -> String {
    let upper = code.trim().to_ascii_uppercase();
    if upper.is_empty() || upper == "XX" || upper == "ZZ" || upper == "T1" {
        return "Local / Unresolved (XX)".to_string();
    }
    let name = match upper.as_str() {
        "US" => "United States",
        "GB" | "UK" => "United Kingdom",
        "NL" => "Netherlands",
        "DE" => "Germany",
        "FR" => "France",
        "CA" => "Canada",
        "AU" => "Australia",
        "ES" => "Spain",
        "IT" => "Italy",
        "SE" => "Sweden",
        "NO" => "Norway",
        "DK" => "Denmark",
        "FI" => "Finland",
        "CH" => "Switzerland",
        "AT" => "Austria",
        "BE" => "Belgium",
        "IE" => "Ireland",
        "PL" => "Poland",
        "PT" => "Portugal",
        "CZ" => "Czechia",
        "JP" => "Japan",
        "KR" => "South Korea",
        "SG" => "Singapore",
        "IN" => "India",
        "BR" => "Brazil",
        "MX" => "Mexico",
        "AR" => "Argentina",
        "NZ" => "New Zealand",
        "ZA" => "South Africa",
        "IL" => "Israel",
        "AE" => "United Arab Emirates",
        "RO" => "Romania",
        "UA" => "Ukraine",
        "TR" => "Turkey",
        "ID" => "Indonesia",
        "PH" => "Philippines",
        "VN" => "Vietnam",
        "TH" => "Thailand",
        "MY" => "Malaysia",
        "HK" => "Hong Kong",
        "TW" => "Taiwan",
        _ => "",
    };
    let flag = if upper.len() == 2 && upper.bytes().all(|b| b.is_ascii_uppercase()) {
        let bytes = upper.as_bytes();
        let c1 = char::from_u32(0x1F1E6 + u32::from(bytes[0] - b'A')).unwrap_or(' ');
        let c2 = char::from_u32(0x1F1E6 + u32::from(bytes[1] - b'A')).unwrap_or(' ');
        format!("{c1}{c2} ")
    } else {
        String::new()
    };
    if name.is_empty() {
        format!("{flag}{upper}")
    } else {
        format!("{flag}{name} ({upper})")
    }
}

pub async fn handle_admin_analytics(
    State(db): State<Database>,
    Query(query): Query<AnalyticsDashboardQuery>,
) -> Html<String> {
    use crate::templates::utils::escape_html;

    let settings = db.get_settings().unwrap_or_default();
    let range = match query.range.as_deref().unwrap_or("30d") {
        "today" | "24h" => "24h",
        "7d" => "7d",
        "30d" => "30d",
        "90d" => "90d",
        "12m" => "12m",
        "all" => "all",
        _ => "30d",
    };
    let filter = query.to_filter();
    let summary = db
        .get_analytics_dashboard_filtered(range, &filter)
        .unwrap_or_default();
    let vitals = db
        .get_core_web_vitals_summary_for_range(Some(range))
        .unwrap_or_default();
    let telemetry_cfg = db.get_analytics_telemetry_settings().unwrap_or_default();
    let top_404s = db.list_404_logs(6).unwrap_or_default();

    let range_human = match range {
        "24h" => "Last 24 Hours",
        "7d" => "Last 7 Days",
        "90d" => "Last 90 Days",
        "12m" => "Last 12 Months",
        "all" => "All Time",
        _ => "Last 30 Days",
    };

    let range_btn = |key: &str, label: &str| -> String {
        let active = range == key;
        let cls = if active {
            "dp-an-seg-btn is-active"
        } else {
            "dp-an-seg-btn"
        };
        let href = escape_html(&build_analytics_href(key, &filter));
        format!(r#"<a href="{href}" class="{cls}">{label}</a>"#)
    };
    let range_pills = format!(
        "{}{}{}{}{}{}",
        range_btn("24h", "24h"),
        range_btn("7d", "7d"),
        range_btn("30d", "30d"),
        range_btn("90d", "90d"),
        range_btn("12m", "12m"),
        range_btn("all", "All"),
    );

    // Active drilldown filter chips
    let filter_bar_html = if filter.is_empty() {
        String::new()
    } else {
        let mut chips = String::new();
        let mut push_chip = |key: &str, label: &str, val_opt: &Option<String>| {
            if let Some(ref v) = val_opt {
                let cleared = with_toggled_filter(&filter, key, v);
                let href = escape_html(&build_analytics_href(range, &cleared));
                chips.push_str(&format!(
                    r#"<a href="{href}" class="dp-an-filter-chip" title="Remove filter"><span class="dp-an-chip-k">{label}:</span> <strong>{val}</strong> <span aria-hidden="true">&times;</span></a>"#,
                    href = href,
                    label = escape_html(label),
                    val = escape_html(v),
                ));
            }
        };
        push_chip("page", "Page", &filter.page);
        push_chip("channel", "Channel", &filter.channel);
        push_chip("referrer", "Referrer", &filter.referrer);
        push_chip("utm_source", "UTM Source", &filter.utm_source);
        push_chip("utm_medium", "UTM Medium", &filter.utm_medium);
        push_chip("utm_campaign", "UTM Campaign", &filter.utm_campaign);
        push_chip("device", "Device", &filter.device);
        push_chip("browser", "Browser", &filter.browser);
        push_chip("os", "OS", &filter.os);
        push_chip("country", "Country", &filter.country);
        push_chip("goal", "Goal", &filter.goal);

        let reset_href = escape_html(&build_analytics_href(
            range,
            &models::AnalyticsFilter::default(),
        ));
        format!(
            r#"<div class="dp-an-filter-bar">
                <span class="dp-an-filter-title">Filtered View</span>
                <div class="dp-an-filter-chips">{chips}</div>
                <a href="{reset_href}" class="dp-an-filter-reset">Clear all</a>
            </div>"#
        )
    };

    // Live active visitors pulse badge
    let live_badge_html = if summary.live_visitors > 0 {
        format!(
            r#"<span class="dp-an-live-pill is-online"><span class="dp-an-live-dot"></span>{} active now</span>"#,
            format_int(summary.live_visitors)
        )
    } else {
        r#"<span class="dp-an-live-pill"><span class="dp-an-live-dot"></span>0 active (5m)</span>"#
            .to_string()
    };

    // Build SVG area + dual-line chart + interactive hit columns
    let max_val = summary
        .buckets
        .iter()
        .map(|b| b.pageviews.max(b.visitors))
        .max()
        .unwrap_or(1)
        .max(1);
    let mid_val = (max_val as f64 / 2.0).ceil() as i64;
    let n_pts = summary.buckets.len().max(1);
    let chart_w = 920.0_f64;
    let chart_h = 224.0_f64;
    let pad_left = 44.0_f64;
    let pad_right = 18.0_f64;
    let pad_top = 18.0_f64;
    let pad_bottom = 26.0_f64;
    let usable_w = chart_w - pad_left - pad_right;
    let usable_h = chart_h - pad_top - pad_bottom;
    let bottom_y = pad_top + usable_h;
    let mid_y = pad_top + usable_h * 0.5;

    let mut pv_points = Vec::with_capacity(n_pts);
    let mut vis_points = Vec::with_capacity(n_pts);
    let mut hover_rects_svg = String::new();
    let mut conv_dots_svg = String::new();
    let step_w = if n_pts <= 1 {
        usable_w
    } else {
        usable_w / (n_pts - 1) as f64
    };

    for (idx, b) in summary.buckets.iter().enumerate() {
        let x = if n_pts <= 1 {
            pad_left + usable_w / 2.0
        } else {
            pad_left + (idx as f64 / (n_pts - 1) as f64) * usable_w
        };
        let y_pv = pad_top + usable_h - (b.pageviews as f64 / max_val as f64) * usable_h;
        let y_vis = pad_top + usable_h - (b.visitors as f64 / max_val as f64) * usable_h;
        pv_points.push(format!("{x:.1},{y_pv:.1}"));
        vis_points.push(format!("{x:.1},{y_vis:.1}"));

        if b.conversions > 0 {
            conv_dots_svg.push_str(&format!(
                r#"<circle cx="{x:.1}" cy="{cy:.1}" r="3.2" fill="var(--ochre, #92400e)" opacity="0.85"></circle>"#,
                x = x,
                cy = bottom_y - 3.0
            ));
        }

        let rx = if n_pts <= 1 {
            pad_left
        } else {
            (x - step_w / 2.0).max(pad_left)
        };
        let rw = if n_pts <= 1 {
            usable_w
        } else {
            step_w.min(chart_w - pad_right - rx).max(4.0)
        };
        let b_esc = escape_html(&b.bucket);
        hover_rects_svg.push_str(&format!(
            r#"<rect class="dp-an-hit" x="{rx:.1}" y="{pad_top:.1}" width="{rw:.1}" height="{usable_h:.1}" fill="transparent" data-x="{x:.1}" data-ypv="{y_pv:.1}" data-yvis="{y_vis:.1}" data-bucket="{b_esc}" data-pv="{pv}" data-vis="{vis}" data-conv="{conv}"><title>{b_esc}: {pv} pageviews, {vis} visitors, {conv} conversions</title></rect>"#,
            rx = rx,
            pad_top = pad_top,
            rw = rw,
            usable_h = usable_h,
            x = x,
            y_pv = y_pv,
            y_vis = y_vis,
            b_esc = b_esc,
            pv = b.pageviews,
            vis = b.visitors,
            conv = b.conversions
        ));
    }

    let first_x = if n_pts <= 1 {
        pad_left + usable_w / 2.0
    } else {
        pad_left
    };
    let last_x = if n_pts <= 1 {
        pad_left + usable_w / 2.0
    } else {
        pad_left + usable_w
    };
    let pv_polyline = pv_points.join(" ");
    let vis_polyline = vis_points.join(" ");
    let pv_area_points = if pv_points.is_empty() {
        String::new()
    } else {
        format!("{first_x:.1},{bottom_y:.1} {pv_polyline} {last_x:.1},{bottom_y:.1}")
    };

    let first_bucket = summary
        .buckets
        .first()
        .map(|b| escape_html(&b.bucket))
        .unwrap_or_default();
    let mid_bucket = if summary.buckets.len() >= 3 {
        escape_html(&summary.buckets[summary.buckets.len() / 2].bucket)
    } else {
        String::new()
    };
    let last_bucket = summary
        .buckets
        .last()
        .map(|b| escape_html(&b.bucket))
        .unwrap_or_default();

    // Helper: Render page table rows with proportion bars, external open link, and engagement badges
    let render_page_rows = |items: &[models::TopPageMetric],
                            metric_col_label: &str,
                            empty_msg: &str|
     -> String {
        if items.is_empty() {
            return format!(r#"<div class="dp-an-empty">{empty_msg}</div>"#);
        }
        let max_v = items.iter().map(|i| i.views).max().unwrap_or(1).max(1);
        let mut rows = String::new();
        for p in items {
            let bar_pct = ((p.views as f64 / max_v as f64) * 100.0).clamp(2.0, 100.0);
            let drill_href = escape_html(&build_analytics_href(
                range,
                &with_toggled_filter(&filter, "page", &p.page_url),
            ));
            let is_selected = filter.page.as_deref() == Some(p.page_url.as_str());
            let row_cls = if is_selected {
                "dp-an-bar-row is-selected"
            } else {
                "dp-an-bar-row"
            };
            let url_esc = escape_html(&p.page_url);
            let mut eng_meta = String::new();
            if p.avg_duration_secs >= 1.0 || p.avg_scroll_depth_pct >= 1.0 {
                let d_txt = if p.avg_duration_secs >= 1.0 {
                    format_duration_human(p.avg_duration_secs)
                } else {
                    String::new()
                };
                let s_txt = if p.avg_scroll_depth_pct >= 1.0 {
                    format!("{:.0}%&#8595;", p.avg_scroll_depth_pct)
                } else {
                    String::new()
                };
                let joined = match (!d_txt.is_empty(), !s_txt.is_empty()) {
                    (true, true) => format!("{d_txt} &middot; {s_txt}"),
                    (true, false) => d_txt,
                    (false, true) => s_txt,
                    (false, false) => String::new(),
                };
                if !joined.is_empty() {
                    eng_meta = format!(r#"<span class="dp-an-row-meta">{joined}</span>"#);
                }
            }
            rows.push_str(&format!(
                r#"<div class="{row_cls}">
                    <div class="dp-an-bar-fill dp-an-bar-press" style="width: {bar_pct:.1}%;"></div>
                    <div class="dp-an-bar-main">
                        <a href="{drill_href}" class="dp-an-bar-link dp-an-mono" title="Filter dashboard by {url_esc}">{url_esc}</a>
                        <a href="{url_esc}" target="_blank" rel="noopener" class="dp-an-ext-link" title="Open page in new tab">&#8599;</a>
                        {eng_meta}
                    </div>
                    <div class="dp-an-bar-nums">
                        <span class="dp-an-num-muted">{vis}</span>
                        <span class="dp-an-num-strong">{vws}</span>
                    </div>
                </div>"#,
                row_cls = row_cls,
                bar_pct = bar_pct,
                drill_href = drill_href,
                url_esc = url_esc,
                eng_meta = eng_meta,
                vis = format_int(p.visitors),
                vws = format_int(p.views),
            ));
        }
        format!(
            r#"<div class="dp-an-list-head"><span>Page Path</span><span><span>Visitors</span><span>{metric_col_label}</span></span></div>{rows}"#
        )
    };

    let top_pages_html = render_page_rows(
        &summary.top_pages,
        "Views",
        "No pageviews recorded in this range yet.",
    );
    let entry_pages_html = render_page_rows(
        &summary.entry_pages,
        "Entrances",
        "No session entry pages recorded in this range yet.",
    );
    let exit_pages_html = render_page_rows(
        &summary.exit_pages,
        "Exits",
        "No session exit pages recorded in this range yet.",
    );

    // Helper: Render breakdown rows with proportion bars & click-to-filter links
    let render_breakdown_list = |items: &[models::BreakdownMetric],
                                 filter_key: &str,
                                 col_title: &str,
                                 empty_msg: &str,
                                 bar_tone: &str|
     -> String {
        if items.is_empty() {
            return format!(r#"<div class="dp-an-empty">{empty_msg}</div>"#);
        }
        let max_vis = items
            .iter()
            .map(|i| i.visitors.max(1))
            .max()
            .unwrap_or(1)
            .max(1);
        let denom_vis = summary.total_visitors.max(1) as f64;
        let mut rows = String::new();
        for item in items {
            let bar_pct =
                ((item.visitors.max(1) as f64 / max_vis as f64) * 100.0).clamp(2.0, 100.0);
            let share_pct = ((item.visitors as f64 / denom_vis) * 100.0).clamp(0.0, 100.0);
            let display_label = if filter_key == "country" {
                escape_html(&format_country_display(&item.label))
            } else if filter_key == "device" {
                let cap = match item.label.as_str() {
                    "mobile" => "Mobile",
                    "tablet" => "Tablet",
                    "desktop" => "Desktop",
                    other => other,
                };
                escape_html(cap)
            } else {
                escape_html(&item.label)
            };
            let drill_href = escape_html(&build_analytics_href(
                range,
                &with_toggled_filter(&filter, filter_key, &item.label),
            ));
            rows.push_str(&format!(
                r#"<div class="dp-an-bar-row">
                    <div class="dp-an-bar-fill {bar_tone}" style="width: {bar_pct:.1}%;"></div>
                    <div class="dp-an-bar-main">
                        <a href="{drill_href}" class="dp-an-bar-link" title="Filter by {raw_esc}">{display_label}</a>
                    </div>
                    <div class="dp-an-bar-nums">
                        <span class="dp-an-num-strong">{vis}</span>
                        <span class="dp-an-num-pct">{share_pct:.0}%</span>
                    </div>
                </div>"#,
                bar_tone = bar_tone,
                bar_pct = bar_pct,
                drill_href = drill_href,
                raw_esc = escape_html(&item.label),
                display_label = display_label,
                vis = format_int(item.visitors),
                share_pct = share_pct,
            ));
        }
        format!(
            r#"<div class="dp-an-list-head"><span>{col_title}</span><span><span>Visitors</span><span>Share</span></span></div>{rows}"#
        )
    };

    let channels_html = render_breakdown_list(
        &summary.channels,
        "channel",
        "Acquisition Channel",
        "No channel telemetry recorded yet.",
        "dp-an-bar-oxide",
    );
    let referrers_html = render_breakdown_list(
        &summary.top_referrers,
        "referrer",
        "Source / Domain",
        "No external referrers recorded yet.",
        "dp-an-bar-oxide",
    );
    let utm_source_html = render_breakdown_list(
        &summary.utm_sources,
        "utm_source",
        "UTM Source",
        "No ?utm_source= campaign hits yet.",
        "dp-an-bar-oxide",
    );
    let utm_medium_html = render_breakdown_list(
        &summary.utm_mediums,
        "utm_medium",
        "UTM Medium",
        "No ?utm_medium= campaign hits yet.",
        "dp-an-bar-oxide",
    );
    let utm_campaign_html = render_breakdown_list(
        &summary.utm_campaigns,
        "utm_campaign",
        "UTM Campaign",
        "No ?utm_campaign= hits yet.",
        "dp-an-bar-oxide",
    );

    let countries_html = render_breakdown_list(
        &summary.countries,
        "country",
        "Country",
        "No country headers (CF-IPCountry / X-Country) recorded yet.",
        "dp-an-bar-press",
    );
    let devices_html = render_breakdown_list(
        &summary.devices,
        "device",
        "Device Form Factor",
        "No device telemetry yet.",
        "dp-an-bar-press",
    );
    let browsers_html = render_breakdown_list(
        &summary.browsers,
        "browser",
        "Browser Family",
        "No browser UA telemetry recorded in this window yet.",
        "dp-an-bar-press",
    );
    let os_html = render_breakdown_list(
        &summary.operating_systems,
        "os",
        "Operating System",
        "No OS telemetry recorded in this window yet.",
        "dp-an-bar-press",
    );

    let render_conversion_list = |items: &[models::ConversionMetric],
                                  col_title: &str,
                                  empty_msg: &str|
     -> String {
        if items.is_empty() {
            return format!(r#"<div class="dp-an-empty">{empty_msg}</div>"#);
        }
        let max_c = items
            .iter()
            .map(|i| i.count.max(1))
            .max()
            .unwrap_or(1)
            .max(1);
        let mut rows = String::new();
        for c in items {
            let bar_pct = ((c.count as f64 / max_c as f64) * 100.0).clamp(2.0, 100.0);
            let g_esc = escape_html(&c.goal_name);
            let drill_href = escape_html(&build_analytics_href(
                range,
                &with_toggled_filter(&filter, "goal", &c.goal_name),
            ));
            rows.push_str(&format!(
                    r#"<div class="dp-an-bar-row">
                        <div class="dp-an-bar-fill dp-an-bar-moss" style="width: {bar_pct:.1}%;"></div>
                        <div class="dp-an-bar-main">
                            <a href="{drill_href}" class="dp-an-bar-link" title="Filter to visitors who triggered {g_esc}">{g_esc}</a>
                        </div>
                        <div class="dp-an-bar-nums">
                            <span class="dp-an-num-strong">{cnt}</span>
                            <span class="dp-an-num-conv">{rate:.1}%</span>
                        </div>
                    </div>"#,
                    bar_pct = bar_pct,
                    drill_href = drill_href,
                    g_esc = g_esc,
                    cnt = format_int(c.count),
                    rate = c.conversion_rate_pct,
                ));
        }
        format!(
            r#"<div class="dp-an-list-head"><span>{col_title}</span><span><span>Triggers</span><span>CR%</span></span></div>{rows}"#
        )
    };

    let goals_html = render_conversion_list(
        &summary.conversions,
        "Goal / Form / Shortlink",
        r#"No conversions recorded in this range yet. Add <code>data-dp-goal="Signup"</code> to any button or submit a FormStudio form."#,
    );
    let outbound_html = render_conversion_list(
        &summary.outbound_clicks,
        "External Destination",
        "No outbound link clicks recorded in this range yet. External <code>&lt;a href&gt;</code> clicks are tracked automatically.",
    );
    let downloads_html = render_conversion_list(
        &summary.file_downloads,
        "Downloaded File",
        "No file downloads (<code>.pdf</code>, <code>.zip</code>, <code>.csv</code>, etc.) recorded in this range yet.",
    );

    // Core Web Vitals strip helpers
    let cwv_badge = |status: &str| -> (&'static str, &'static str) {
        match status {
            "poor" => ("Poor", "dp-an-cwv-poor"),
            "needs_improvement" => ("Needs Work", "dp-an-cwv-warn"),
            _ => ("Good", "dp-an-cwv-good"),
        }
    };
    let (ov_lbl, ov_cls) = if vitals.sample_count == 0 {
        ("Awaiting RUM Beacons", "dp-an-cwv-neutral")
    } else {
        cwv_badge(&vitals.overall_status)
    };
    let (_, lcp_cls) = cwv_badge(&vitals.lcp_status);
    let (_, inp_cls) = cwv_badge(&vitals.inp_status);
    let (_, cls_cls) = cwv_badge(&vitals.cls_status);
    let ttfb_cls = if vitals.sample_count == 0 || vitals.ttfb_p75_ms <= 800.0 {
        "dp-an-cwv-good"
    } else if vitals.ttfb_p75_ms <= 1800.0 {
        "dp-an-cwv-warn"
    } else {
        "dp-an-cwv-poor"
    };

    let lcp_disp = if vitals.sample_count == 0 {
        "—".to_string()
    } else {
        format!("{:.0} ms", vitals.lcp_p75_ms)
    };
    let inp_disp = if vitals.sample_count == 0 {
        "—".to_string()
    } else {
        format!("{:.0} ms", vitals.inp_p75_ms)
    };
    let cls_disp = if vitals.sample_count == 0 {
        "—".to_string()
    } else {
        format!("{:.3}", vitals.cls_p75)
    };
    let ttfb_disp = if vitals.sample_count == 0 {
        "—".to_string()
    } else {
        format!("{:.0} ms", vitals.ttfb_p75_ms)
    };

    let mut cwv_routes_html = String::new();
    if !vitals.routes.is_empty() {
        cwv_routes_html.push_str(r#"<div class="dp-an-cwv-routes"><div class="dp-an-list-head"><span>Route (p75 RUM)</span><span><span>LCP</span><span>INP</span><span>CLS</span></span></div>"#);
        for r in &vitals.routes {
            let (st_lbl, st_cls) = cwv_badge(&r.status);
            let p_esc = escape_html(&r.page_url);
            let drill = escape_html(&build_analytics_href(
                range,
                &with_toggled_filter(&filter, "page", &r.page_url),
            ));
            cwv_routes_html.push_str(&format!(
                r#"<div class="dp-an-cwv-route-row">
                    <div style="display:flex;align-items:center;gap:8px;min-width:0;">
                        <span class="dp-an-cwv-dot {st_cls}" title="{st_lbl}"></span>
                        <a href="{drill}" class="dp-an-bar-link dp-an-mono">{p_esc}</a>
                        <span class="dp-an-row-meta">{samples} samples</span>
                    </div>
                    <div class="dp-an-cwv-route-nums">
                        <span>{lcp:.0}ms</span>
                        <span>{inp:.0}ms</span>
                        <span>{cls:.3}</span>
                    </div>
                </div>"#,
                st_cls = st_cls,
                st_lbl = st_lbl,
                drill = drill,
                p_esc = p_esc,
                samples = r.sample_count,
                lcp = r.lcp_p75_ms,
                inp = r.inp_p75_ms,
                cls = r.cls_p75,
            ));
        }
        cwv_routes_html.push_str("</div>");
    }

    // 404 Not-Found Health Card rows
    let mut not_found_html = String::new();
    if top_404s.is_empty() {
        not_found_html.push_str(
            r#"<div class="dp-an-empty">Zero 404 broken routes recorded. Inbound links are clean.</div>"#,
        );
    } else {
        not_found_html.push_str(
            r#"<div class="dp-an-list-head"><span>Missing Path</span><span>Hits</span></div>"#,
        );
        for nf in &top_404s {
            let path_esc = escape_html(&nf.path);
            let ref_note = if nf.referrer.trim().is_empty() {
                String::new()
            } else {
                format!(
                    r#"<span class="dp-an-row-meta" title="Last Referrer">from {}</span>"#,
                    escape_html(&nf.referrer.chars().take(42).collect::<String>())
                )
            };
            not_found_html.push_str(&format!(
                r#"<div class="dp-an-bar-row">
                    <div class="dp-an-bar-main">
                        <span class="dp-an-mono" style="font-size:12px;color:var(--danger);">{path_esc}</span>
                        {ref_note}
                    </div>
                    <div class="dp-an-bar-nums">
                        <span class="dp-an-num-strong">{hits}</span>
                    </div>
                </div>"#,
                path_esc = path_esc,
                ref_note = ref_note,
                hits = format_int(nf.hits),
            ));
        }
    }

    // KPI Deltas & Formatted Values
    let vis_delta = render_delta_badge(
        summary.total_visitors as f64,
        summary.prev_visitors as f64,
        false,
    );
    let pv_delta = render_delta_badge(
        summary.total_pageviews as f64,
        summary.prev_pageviews as f64,
        false,
    );
    let dur_delta = render_delta_badge(
        summary.avg_duration_secs,
        summary.prev_avg_duration_secs,
        false,
    );
    let bounce_delta =
        render_delta_badge(summary.bounce_rate_pct, summary.prev_bounce_rate_pct, true);
    let conv_delta = render_delta_badge(
        summary.total_conversions as f64,
        summary.prev_conversions as f64,
        false,
    );

    let ret_sel = |val: i64| -> &'static str {
        if telemetry_cfg.retention_days == val {
            "selected"
        } else {
            ""
        }
    };
    let chk_bots = if telemetry_cfg.filter_bots {
        "checked"
    } else {
        ""
    };
    let chk_admins = if telemetry_cfg.exclude_admins {
        "checked"
    } else {
        ""
    };
    let chk_dnt = if telemetry_cfg.honor_dnt_gpc {
        "checked"
    } else {
        ""
    };

    let hygiene_pills = format!(
        r#"<span class="dp-an-shield" title="Visitor hashes rotate at 00:00 UTC with a 256-bit CSPRNG secret">Salted SHA-256</span>{}{}{}"#,
        if telemetry_cfg.filter_bots {
            r#"<span class="dp-an-shield" title="Bots, AI crawlers, headless browsers &amp; prefetches dropped">Bot Shield</span>"#
        } else {
            ""
        },
        if telemetry_cfg.exclude_admins {
            r#"<span class="dp-an-shield" title="Logged-in Administrators &amp; Editors excluded">Staff Excluded</span>"#
        } else {
            ""
        },
        if telemetry_cfg.honor_dnt_gpc {
            r#"<span class="dp-an-shield" title="DNT: 1 and Sec-GPC: 1 headers respected">DNT/GPC</span>"#
        } else {
            ""
        },
    );

    let content = format!(
        r##"
        <style>
            .dp-an-header {{
                display: flex;
                justify-content: space-between;
                align-items: flex-start;
                gap: 16px;
                margin-bottom: 18px;
                flex-wrap: wrap;
            }}
            .dp-an-title-row {{
                display: flex;
                align-items: center;
                gap: 10px;
                flex-wrap: wrap;
            }}
            .dp-an-live-pill {{
                display: inline-flex;
                align-items: center;
                gap: 6px;
                font-family: var(--font-mono);
                font-size: 11.5px;
                font-weight: 500;
                padding: 3px 10px;
                border-radius: var(--radius-pill, 999px);
                background: var(--paper-sunk);
                border: 1px solid var(--rule);
                color: var(--ink-muted);
            }}
            .dp-an-live-pill.is-online {{
                background: var(--moss-wash);
                border-color: rgba(30, 126, 52, 0.3);
                color: var(--moss);
            }}
            .dp-an-live-dot {{
                width: 7px;
                height: 7px;
                border-radius: 50%;
                background: var(--rule-strong);
            }}
            .dp-an-live-pill.is-online .dp-an-live-dot {{
                background: var(--moss);
                box-shadow: 0 0 0 3px rgba(30, 126, 52, 0.18);
            }}
            .dp-an-shield {{
                font-family: var(--font-mono);
                font-size: 10.5px;
                text-transform: uppercase;
                letter-spacing: 0.04em;
                padding: 2px 7px;
                border-radius: var(--radius-sm, 3px);
                background: var(--paper-raised);
                border: 1px solid var(--rule);
                color: var(--ink-muted);
            }}
            .dp-an-controls {{
                display: flex;
                align-items: center;
                gap: 8px;
                flex-wrap: wrap;
            }}
            .dp-an-seg {{
                display: inline-flex;
                background: var(--paper-raised);
                border: 1px solid var(--rule);
                border-radius: var(--radius-md, 4px);
                overflow: hidden;
            }}
            .dp-an-seg-btn {{
                padding: 6px 11px;
                font-size: 12px;
                font-weight: 600;
                font-family: var(--font-mono);
                color: var(--ink-muted);
                text-decoration: none;
                border-right: 1px solid var(--rule);
                transition: background 0.12s ease, color 0.12s ease;
            }}
            .dp-an-seg-btn:last-child {{ border-right: none; }}
            .dp-an-seg-btn:hover {{ background: var(--paper-sunk); color: var(--ink); }}
            .dp-an-seg-btn.is-active {{
                background: var(--ink);
                color: var(--paper-raised);
            }}
            .dp-an-drawer {{
                display: none;
                margin-bottom: 18px;
                padding: 18px 20px;
                background: var(--paper-raised);
                border: 1px solid var(--rule-strong);
                border-top: 2px solid var(--oxide);
                border-radius: var(--radius-md, 4px);
            }}
            .dp-an-drawer.is-open {{ display: block; }}
            .dp-an-filter-bar {{
                display: flex;
                align-items: center;
                gap: 10px;
                flex-wrap: wrap;
                padding: 10px 14px;
                margin-bottom: 18px;
                background: var(--press-wash);
                border: 1px solid rgba(34, 113, 177, 0.25);
                border-radius: var(--radius-md, 4px);
            }}
            .dp-an-filter-title {{
                font-family: var(--font-mono);
                font-size: 11px;
                font-weight: 600;
                text-transform: uppercase;
                letter-spacing: 0.05em;
                color: var(--press);
            }}
            .dp-an-filter-chips {{ display: flex; gap: 6px; flex-wrap: wrap; flex: 1; }}
            .dp-an-filter-chip {{
                display: inline-flex;
                align-items: center;
                gap: 5px;
                padding: 3px 9px;
                font-size: 12px;
                background: var(--paper-raised);
                border: 1px solid var(--rule-strong);
                border-radius: var(--radius-sm, 3px);
                color: var(--ink);
                text-decoration: none;
            }}
            .dp-an-filter-chip:hover {{ border-color: var(--danger); color: var(--danger); }}
            .dp-an-chip-k {{ color: var(--ink-muted); font-size: 11.5px; }}
            .dp-an-filter-reset {{
                font-size: 12px;
                font-weight: 600;
                color: var(--press);
                text-decoration: none;
            }}
            .dp-an-kpis {{
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(185px, 1fr));
                gap: 14px;
                margin-bottom: 20px;
            }}
            .dp-an-kpi {{
                background: var(--paper-raised);
                border: 1px solid var(--rule);
                border-radius: var(--radius-md, 4px);
                padding: 16px 18px;
                display: flex;
                flex-direction: column;
                justify-content: space-between;
            }}
            .dp-an-kpi-top {{
                display: flex;
                justify-content: space-between;
                align-items: center;
                gap: 8px;
            }}
            .dp-an-kpi-label {{
                font-size: 11.5px;
                font-weight: 700;
                text-transform: uppercase;
                letter-spacing: 0.04em;
                color: var(--ink-muted);
            }}
            .dp-an-kpi-val {{
                font-size: 28px;
                font-weight: 800;
                line-height: 1.15;
                letter-spacing: -0.02em;
                color: var(--ink);
                margin: 8px 0 6px;
                font-variant-numeric: tabular-nums;
            }}
            .dp-an-kpi-sub {{
                font-size: 12px;
                color: var(--ink-muted);
                font-variant-numeric: tabular-nums;
            }}
            .dp-an-delta {{
                font-family: var(--font-mono);
                font-size: 11px;
                font-weight: 600;
                padding: 2px 6px;
                border-radius: var(--radius-sm, 3px);
                white-space: nowrap;
            }}
            .dp-an-delta-good {{ background: var(--moss-wash); color: var(--moss); }}
            .dp-an-delta-bad {{ background: var(--oxide-wash); color: var(--danger); }}
            .dp-an-delta-neutral {{ background: var(--paper-sunk); color: var(--ink-muted); }}

            .dp-an-chart-card {{
                background: var(--paper-raised);
                border: 1px solid var(--rule);
                border-radius: var(--radius-md, 4px);
                padding: 20px 22px 16px;
                margin-bottom: 20px;
                position: relative;
            }}
            .dp-an-tooltip {{
                position: absolute;
                pointer-events: none;
                display: none;
                z-index: 20;
                background: var(--desk, #1d2327);
                color: var(--desk-ink, #f0f0f1);
                padding: 8px 11px;
                border-radius: var(--radius-md, 4px);
                font-size: 12px;
                line-height: 1.45;
                box-shadow: 0 8px 24px rgba(0, 0, 0, 0.22);
                white-space: nowrap;
            }}
            .dp-an-grid-2 {{
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
                gap: 18px;
                margin-bottom: 20px;
            }}
            @media (max-width: 520px) {{
                .dp-an-grid-2 {{ grid-template-columns: 1fr; }}
            }}
            .dp-an-panel {{
                background: var(--paper-raised);
                border: 1px solid var(--rule);
                border-radius: var(--radius-md, 4px);
                padding: 18px 20px;
                display: flex;
                flex-direction: column;
            }}
            .dp-an-panel-head {{
                display: flex;
                justify-content: space-between;
                align-items: center;
                gap: 10px;
                margin-bottom: 12px;
                flex-wrap: wrap;
            }}
            .dp-an-panel-title {{
                font-size: 14.5px;
                font-weight: 700;
                margin: 0;
                color: var(--ink);
            }}
            .dp-an-tabs {{
                display: inline-flex;
                gap: 2px;
                background: var(--paper-sunk);
                padding: 2px;
                border-radius: var(--radius-sm, 3px);
                border: 1px solid var(--rule);
            }}
            .dp-an-tab {{
                border: none;
                background: transparent;
                color: var(--ink-muted);
                font-size: 11.5px;
                font-weight: 600;
                padding: 4px 8px;
                border-radius: 2px;
                cursor: pointer;
                font-family: var(--font-sans);
            }}
            .dp-an-tab:hover {{ color: var(--ink); }}
            .dp-an-tab.is-active {{
                background: var(--paper-raised);
                color: var(--ink);
                box-shadow: 0 1px 2px rgba(0,0,0,0.06);
            }}
            .dp-an-pane {{ display: none; }}
            .dp-an-pane.is-active {{ display: block; }}
            .dp-an-list-head {{
                display: flex;
                justify-content: space-between;
                align-items: center;
                font-size: 11px;
                font-weight: 700;
                text-transform: uppercase;
                letter-spacing: 0.04em;
                color: var(--ink-muted);
                padding: 0 8px 7px;
                border-bottom: 1px solid var(--rule);
                margin-bottom: 6px;
            }}
            .dp-an-list-head > span:last-child {{
                display: inline-flex;
                gap: 18px;
            }}
            .dp-an-bar-row {{
                position: relative;
                display: flex;
                justify-content: space-between;
                align-items: center;
                gap: 12px;
                padding: 7px 10px;
                margin-bottom: 4px;
                border-radius: var(--radius-sm, 3px);
                overflow: hidden;
            }}
            .dp-an-bar-row.is-selected {{
                outline: 1.5px solid var(--press);
            }}
            .dp-an-bar-fill {{
                position: absolute;
                left: 0;
                top: 0;
                bottom: 0;
                z-index: 0;
                border-radius: var(--radius-sm, 3px);
                pointer-events: none;
            }}
            .dp-an-bar-press {{ background: var(--press-wash, rgba(34, 113, 177, 0.1)); }}
            .dp-an-bar-oxide {{ background: var(--oxide-wash, rgba(184, 67, 27, 0.1)); }}
            .dp-an-bar-moss {{ background: var(--moss-wash, rgba(30, 126, 52, 0.11)); }}
            .dp-an-bar-main {{
                position: relative;
                z-index: 1;
                display: flex;
                align-items: center;
                gap: 7px;
                min-width: 0;
                flex: 1;
            }}
            .dp-an-bar-link {{
                color: var(--ink);
                font-size: 13px;
                font-weight: 500;
                text-decoration: none;
                overflow: hidden;
                text-overflow: ellipsis;
                white-space: nowrap;
            }}
            .dp-an-bar-link:hover {{ color: var(--press); text-decoration: underline; }}
            .dp-an-mono {{ font-family: var(--font-mono); font-size: 12px; }}
            .dp-an-ext-link {{
                color: var(--ink-muted);
                font-size: 11.5px;
                text-decoration: none;
                flex-shrink: 0;
                opacity: 0.7;
            }}
            .dp-an-ext-link:hover {{ color: var(--press); opacity: 1; }}
            .dp-an-row-meta {{
                font-family: var(--font-mono);
                font-size: 10.5px;
                color: var(--ink-muted);
                background: var(--paper-raised);
                border: 1px solid var(--rule);
                padding: 1px 5px;
                border-radius: 2px;
                white-space: nowrap;
                flex-shrink: 0;
            }}
            .dp-an-bar-nums {{
                position: relative;
                z-index: 1;
                display: flex;
                align-items: center;
                gap: 14px;
                font-family: var(--font-mono);
                font-size: 12px;
                font-variant-numeric: tabular-nums;
                flex-shrink: 0;
            }}
            .dp-an-num-muted {{ color: var(--ink-muted); min-width: 44px; text-align: right; }}
            .dp-an-num-strong {{ color: var(--ink); font-weight: 700; min-width: 44px; text-align: right; }}
            .dp-an-num-pct {{ color: var(--ink-muted); min-width: 38px; text-align: right; }}
            .dp-an-num-conv {{ color: var(--moss); font-weight: 700; min-width: 44px; text-align: right; }}
            .dp-an-empty {{
                padding: 26px 14px;
                text-align: center;
                color: var(--ink-muted);
                font-size: 13px;
                line-height: 1.55;
            }}
            .dp-an-health-grid {{
                display: grid;
                grid-template-columns: 1.6fr 1fr;
                gap: 18px;
                margin-bottom: 24px;
            }}
            @media (max-width: 960px) {{
                .dp-an-health-grid {{ grid-template-columns: 1fr; }}
            }}
            .dp-an-cwv-tiles {{
                display: grid;
                grid-template-columns: repeat(4, 1fr);
                gap: 10px;
                margin-bottom: 14px;
            }}
            @media (max-width: 640px) {{
                .dp-an-cwv-tiles {{ grid-template-columns: repeat(2, 1fr); }}
            }}
            .dp-an-cwv-tile {{
                padding: 12px;
                border: 1px solid var(--rule);
                border-radius: var(--radius-sm, 3px);
                background: var(--paper-sunk);
            }}
            .dp-an-cwv-good {{ color: var(--moss); background: var(--moss-wash); border-color: rgba(30, 126, 52, 0.28); }}
            .dp-an-cwv-warn {{ color: var(--ochre); background: var(--ochre-wash); border-color: rgba(146, 64, 14, 0.28); }}
            .dp-an-cwv-poor {{ color: var(--danger); background: var(--oxide-wash); border-color: rgba(179, 45, 46, 0.28); }}
            .dp-an-cwv-neutral {{ color: var(--ink-muted); background: var(--paper-sunk); border-color: var(--rule); }}
            .dp-an-cwv-dot {{
                width: 8px;
                height: 8px;
                border-radius: 50%;
                display: inline-block;
                flex-shrink: 0;
            }}
            .dp-an-cwv-dot.dp-an-cwv-good {{ background: var(--moss); }}
            .dp-an-cwv-dot.dp-an-cwv-warn {{ background: var(--ochre); }}
            .dp-an-cwv-dot.dp-an-cwv-poor {{ background: var(--danger); }}
            .dp-an-cwv-route-row {{
                display: flex;
                justify-content: space-between;
                align-items: center;
                gap: 10px;
                padding: 6px 8px;
                border-bottom: 1px solid var(--rule);
                font-size: 12px;
            }}
            .dp-an-cwv-route-row:last-child {{ border-bottom: none; }}
            .dp-an-cwv-route-nums {{
                display: inline-flex;
                gap: 14px;
                font-family: var(--font-mono);
                font-size: 11.5px;
                color: var(--ink);
                font-variant-numeric: tabular-nums;
            }}
            .dp-an-cwv-route-nums > span {{ min-width: 48px; text-align: right; }}
        </style>

        <div class="dp-an-header">
            <div>
                <div class="dp-an-title-row">
                    <h1 class="page-title" style="margin:0;">Cookieless Analytics</h1>
                    {live_badge_html}
                    {hygiene_pills}
                </div>
                <p class="page-subtitle" style="margin:6px 0 0;">First-party visitor, engagement &amp; conversion telemetry ({range_human}). Click any page, source, device, country, or goal to drill down.</p>
            </div>
            <div class="dp-an-controls">
                <div class="dp-an-seg" role="group" aria-label="Date range">
                    {range_pills}
                </div>
                <button type="button" class="btn btn-secondary" style="padding: 6px 11px; font-size: 12px;" onclick="document.getElementById('dp-an-settings-drawer').classList.toggle('is-open')">
                    &#9881; Telemetry &amp; Retention
                </button>
            </div>
        </div>

        <section id="dp-an-settings-drawer" class="dp-an-drawer" aria-label="Telemetry and storage settings">
            <div style="display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:10px;margin-bottom:12px;">
                <div>
                    <h2 style="margin:0;font-size:14px;font-weight:700;color:var(--ink);">Ingestion Hygiene, Privacy &amp; SQLite Rollup Retention</h2>
                    <p style="margin:3px 0 0;font-size:12px;color:var(--ink-muted);">
                        Raw Event Rows: <strong id="dp-an-stat-raw">{raw_rows}</strong> &middot;
                        Daily Rollup Rows (<code>daypress_analytics_daily</code>): <strong id="dp-an-stat-daily">{daily_rows}</strong> &middot;
                        Daily 256-bit Secret Salt: <strong>Active</strong>
                    </p>
                </div>
                <span id="dp-an-save-status" style="font-family:var(--font-mono);font-size:12px;color:var(--moss);"></span>
            </div>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(230px,1fr));gap:14px;align-items:end;">
                <div>
                    <label class="form-label" for="dp-an-retention" style="font-size:12px;font-weight:600;display:block;margin-bottom:4px;">Raw Event Retention Window</label>
                    <select id="dp-an-retention" class="form-select" style="width:100%;font-size:12.5px;">
                        <option value="30" {ret_30}>30 Days (Compact — keep daily rollups forever)</option>
                        <option value="90" {ret_90}>90 Days (Balanced)</option>
                        <option value="180" {ret_180}>180 Days (Recommended)</option>
                        <option value="365" {ret_365}>365 Days (1 Year granular)</option>
                        <option value="0" {ret_0}>Keep All Raw Events Indefinitely</option>
                    </select>
                </div>
                <div style="display:flex;flex-direction:column;gap:6px;font-size:12.5px;">
                    <label style="display:flex;align-items:center;gap:7px;cursor:pointer;">
                        <input type="checkbox" id="dp-an-chk-bots" {chk_bots} />
                        <span>Drop bots, AI crawlers, headless &amp; prefetches</span>
                    </label>
                    <label style="display:flex;align-items:center;gap:7px;cursor:pointer;">
                        <input type="checkbox" id="dp-an-chk-admins" {chk_admins} />
                        <span>Exclude logged-in Admins &amp; Editors</span>
                    </label>
                    <label style="display:flex;align-items:center;gap:7px;cursor:pointer;">
                        <input type="checkbox" id="dp-an-chk-dnt" {chk_dnt} />
                        <span>Honor <code>DNT: 1</code> &amp; <code>Sec-GPC: 1</code> headers</span>
                    </label>
                </div>
                <div style="display:flex;justify-content:flex-end;gap:8px;">
                    <button type="button" class="btn btn-primary" style="padding:7px 14px;font-size:12.5px;" onclick="window.dpSaveAnalyticsSettings(this)">
                        Save &amp; Rollup Now
                    </button>
                </div>
            </div>
        </section>

        {filter_bar_html}

        <div class="dp-an-kpis">
            <div class="dp-an-kpi">
                <div class="dp-an-kpi-top">
                    <span class="dp-an-kpi-label">Unique Visitors</span>
                    {vis_delta}
                </div>
                <div class="dp-an-kpi-val">{visitors_fmt}</div>
                <div class="dp-an-kpi-sub">Salted daily SHA-256</div>
            </div>
            <div class="dp-an-kpi">
                <div class="dp-an-kpi-top">
                    <span class="dp-an-kpi-label">Total Pageviews</span>
                    {pv_delta}
                </div>
                <div class="dp-an-kpi-val" style="color:var(--oxide);">{pageviews_fmt}</div>
                <div class="dp-an-kpi-sub">{views_per_visit:.2} views / visitor</div>
            </div>
            <div class="dp-an-kpi">
                <div class="dp-an-kpi-top">
                    <span class="dp-an-kpi-label">Engaged Time</span>
                    {dur_delta}
                </div>
                <div class="dp-an-kpi-val">{avg_dur_fmt}</div>
                <div class="dp-an-kpi-sub">Avg. scroll depth: {avg_scroll:.0}%</div>
            </div>
            <div class="dp-an-kpi">
                <div class="dp-an-kpi-top">
                    <span class="dp-an-kpi-label">Bounce Rate</span>
                    {bounce_delta}
                </div>
                <div class="dp-an-kpi-val">{bounce:.1}%</div>
                <div class="dp-an-kpi-sub">1-page &lt;10s, 0 conversions</div>
            </div>
            <div class="dp-an-kpi">
                <div class="dp-an-kpi-top">
                    <span class="dp-an-kpi-label">Conversions</span>
                    {conv_delta}
                </div>
                <div class="dp-an-kpi-val" style="color:var(--moss);">{conversions_fmt}</div>
                <div class="dp-an-kpi-sub">{conv_rate:.1}% visitor conv. rate</div>
            </div>
        </div>

        <div class="dp-an-chart-card" id="dp-an-chart-wrap">
            <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:10px;flex-wrap:wrap;gap:10px;">
                <h2 style="font-size:14.5px;font-weight:700;margin:0;color:var(--ink);">Traffic &amp; Conversion Curve ({range_human})</h2>
                <div style="display:flex;gap:16px;font-size:12px;color:var(--ink-muted);align-items:center;">
                    <span><strong style="color:var(--oxide);">&bull;</strong> Pageviews</span>
                    <span><strong style="color:var(--moss);">&bull;</strong> Unique Visitors</span>
                    <span><strong style="color:var(--ochre);">&bull;</strong> Conversions</span>
                </div>
            </div>
            <div id="dp-an-tooltip" class="dp-an-tooltip"></div>
            <svg id="dp-an-svg" viewBox="0 0 {chart_w} {chart_h}" style="width:100%;height:224px;overflow:visible;display:block;">
                <defs>
                    <linearGradient id="dpAnPvGrad" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stop-color="var(--oxide, #b8431b)" stop-opacity="0.20" />
                        <stop offset="100%" stop-color="var(--oxide, #b8431b)" stop-opacity="0.0" />
                    </linearGradient>
                </defs>
                <!-- Horizontal Y-axis reference rules -->
                <line x1="{pad_left}" y1="{pad_top}" x2="{right_x}" y2="{pad_top}" stroke="var(--rule)" stroke-dasharray="3 3" stroke-width="1" />
                <line x1="{pad_left}" y1="{mid_y}" x2="{right_x}" y2="{mid_y}" stroke="var(--rule)" stroke-dasharray="3 3" stroke-width="1" />
                <line x1="{pad_left}" y1="{bottom_y}" x2="{right_x}" y2="{bottom_y}" stroke="var(--rule-strong)" stroke-width="1" />
                <text x="{axis_x}" y="{top_lbl_y}" text-anchor="end" font-size="10.5" font-family="var(--font-mono)" fill="var(--ink-muted)">{max_val_fmt}</text>
                <text x="{axis_x}" y="{mid_lbl_y}" text-anchor="end" font-size="10.5" font-family="var(--font-mono)" fill="var(--ink-muted)">{mid_val_fmt}</text>
                <text x="{axis_x}" y="{bot_lbl_y}" text-anchor="end" font-size="10.5" font-family="var(--font-mono)" fill="var(--ink-muted)">0</text>

                <!-- Series -->
                <polygon points="{pv_area_points}" fill="url(#dpAnPvGrad)" />
                <polyline fill="none" stroke="var(--moss, #1e7e34)" stroke-width="2" stroke-dasharray="4 3" points="{vis_polyline}" />
                <polyline fill="none" stroke="var(--oxide, #b8431b)" stroke-width="2.4" stroke-linejoin="round" stroke-linecap="round" points="{pv_polyline}" />
                {conv_dots_svg}

                <!-- Dynamic Hover Crosshair -->
                <line id="dp-an-crosshair" x1="0" y1="{pad_top}" x2="0" y2="{bottom_y}" stroke="var(--ink)" stroke-width="1" stroke-dasharray="2 2" style="display:none;pointer-events:none;" />
                <circle id="dp-an-dot-pv" cx="0" cy="0" r="4.5" fill="var(--oxide, #b8431b)" stroke="#fff" stroke-width="1.5" style="display:none;pointer-events:none;" />
                <circle id="dp-an-dot-vis" cx="0" cy="0" r="4" fill="var(--moss, #1e7e34)" stroke="#fff" stroke-width="1.5" style="display:none;pointer-events:none;" />

                <!-- Hit columns -->
                {hover_rects_svg}
            </svg>
            <div style="display:flex;justify-content:space-between;font-size:11px;color:var(--ink-muted);font-family:var(--font-mono);margin-top:4px;padding-left:{pad_left}px;">
                <span>{first_bucket}</span>
                <span>{mid_bucket}</span>
                <span>{last_bucket}</span>
            </div>
        </div>

        <div class="dp-an-grid-2">
            <!-- Quadrant 1: Content & Session Flow -->
            <div class="dp-an-panel" data-tab-group="pages">
                <div class="dp-an-panel-head">
                    <h2 class="dp-an-panel-title">Pages &amp; Session Flow</h2>
                    <div class="dp-an-tabs" role="tablist">
                        <button type="button" class="dp-an-tab is-active" onclick="window.dpSwitchTab(this,'pages-top')">Top Pages</button>
                        <button type="button" class="dp-an-tab" onclick="window.dpSwitchTab(this,'pages-entry')">Entry Pages</button>
                        <button type="button" class="dp-an-tab" onclick="window.dpSwitchTab(this,'pages-exit')">Exit Pages</button>
                    </div>
                </div>
                <div class="dp-an-pane is-active" data-pane="pages-top">{top_pages_html}</div>
                <div class="dp-an-pane" data-pane="pages-entry">{entry_pages_html}</div>
                <div class="dp-an-pane" data-pane="pages-exit">{exit_pages_html}</div>
            </div>

            <!-- Quadrant 2: Acquisition Channels, Referrers & UTMs -->
            <div class="dp-an-panel" data-tab-group="sources">
                <div class="dp-an-panel-head">
                    <h2 class="dp-an-panel-title">Acquisition &amp; Campaigns</h2>
                    <div class="dp-an-tabs" role="tablist">
                        <button type="button" class="dp-an-tab is-active" onclick="window.dpSwitchTab(this,'src-channels')">Channels</button>
                        <button type="button" class="dp-an-tab" onclick="window.dpSwitchTab(this,'src-referrers')">Referrers</button>
                        <button type="button" class="dp-an-tab" onclick="window.dpSwitchTab(this,'src-utm-s')">Source</button>
                        <button type="button" class="dp-an-tab" onclick="window.dpSwitchTab(this,'src-utm-m')">Medium</button>
                        <button type="button" class="dp-an-tab" onclick="window.dpSwitchTab(this,'src-utm-c')">Campaign</button>
                    </div>
                </div>
                <div class="dp-an-pane is-active" data-pane="src-channels">{channels_html}</div>
                <div class="dp-an-pane" data-pane="src-referrers">{referrers_html}</div>
                <div class="dp-an-pane" data-pane="src-utm-s">{utm_source_html}</div>
                <div class="dp-an-pane" data-pane="src-utm-m">{utm_medium_html}</div>
                <div class="dp-an-pane" data-pane="src-utm-c">{utm_campaign_html}</div>
            </div>

            <!-- Quadrant 3: Geography, Devices, Browsers & OS -->
            <div class="dp-an-panel" data-tab-group="env">
                <div class="dp-an-panel-head">
                    <h2 class="dp-an-panel-title">Locations &amp; Environment</h2>
                    <div class="dp-an-tabs" role="tablist">
                        <button type="button" class="dp-an-tab is-active" onclick="window.dpSwitchTab(this,'env-countries')">Countries</button>
                        <button type="button" class="dp-an-tab" onclick="window.dpSwitchTab(this,'env-devices')">Devices</button>
                        <button type="button" class="dp-an-tab" onclick="window.dpSwitchTab(this,'env-browsers')">Browsers</button>
                        <button type="button" class="dp-an-tab" onclick="window.dpSwitchTab(this,'env-os')">OS</button>
                    </div>
                </div>
                <div class="dp-an-pane is-active" data-pane="env-countries">{countries_html}</div>
                <div class="dp-an-pane" data-pane="env-devices">{devices_html}</div>
                <div class="dp-an-pane" data-pane="env-browsers">{browsers_html}</div>
                <div class="dp-an-pane" data-pane="env-os">{os_html}</div>
            </div>

            <!-- Quadrant 4: Conversions, Outbound Clicks & Downloads -->
            <div class="dp-an-panel" data-tab-group="conv">
                <div class="dp-an-panel-head">
                    <h2 class="dp-an-panel-title">Goals, Outbound &amp; Downloads</h2>
                    <div class="dp-an-tabs" role="tablist">
                        <button type="button" class="dp-an-tab is-active" onclick="window.dpSwitchTab(this,'conv-goals')">Goals &amp; Forms</button>
                        <button type="button" class="dp-an-tab" onclick="window.dpSwitchTab(this,'conv-outbound')">Outbound Links</button>
                        <button type="button" class="dp-an-tab" onclick="window.dpSwitchTab(this,'conv-downloads')">Downloads</button>
                    </div>
                </div>
                <div class="dp-an-pane is-active" data-pane="conv-goals">{goals_html}</div>
                <div class="dp-an-pane" data-pane="conv-outbound">{outbound_html}</div>
                <div class="dp-an-pane" data-pane="conv-downloads">{downloads_html}</div>
            </div>
        </div>

        <!-- Real-User Core Web Vitals (p75) & 404 Broken Link Health Strip -->
        <div class="dp-an-health-grid">
            <div class="dp-an-panel">
                <div class="dp-an-panel-head">
                    <div>
                        <h2 class="dp-an-panel-title">Real-User Core Web Vitals (p75)</h2>
                        <p style="margin:2px 0 0;font-size:12px;color:var(--ink-muted);">{vitals_samples} RUM beacons in window</p>
                    </div>
                    <span class="dp-an-delta {ov_cls}" style="padding:4px 9px;font-size:11.5px;">{ov_lbl}</span>
                </div>
                <div class="dp-an-cwv-tiles">
                    <div class="dp-an-cwv-tile {lcp_cls}">
                        <div style="font-size:11px;font-weight:700;text-transform:uppercase;">LCP (p75)</div>
                        <div style="font-size:20px;font-weight:800;margin:4px 0 2px;font-family:var(--font-mono);">{lcp_disp}</div>
                        <div style="font-size:11px;opacity:0.85;">Good &le; 2500 ms</div>
                    </div>
                    <div class="dp-an-cwv-tile {inp_cls}">
                        <div style="font-size:11px;font-weight:700;text-transform:uppercase;">INP (p75)</div>
                        <div style="font-size:20px;font-weight:800;margin:4px 0 2px;font-family:var(--font-mono);">{inp_disp}</div>
                        <div style="font-size:11px;opacity:0.85;">Good &le; 200 ms</div>
                    </div>
                    <div class="dp-an-cwv-tile {cls_cls}">
                        <div style="font-size:11px;font-weight:700;text-transform:uppercase;">CLS (p75)</div>
                        <div style="font-size:20px;font-weight:800;margin:4px 0 2px;font-family:var(--font-mono);">{cls_disp}</div>
                        <div style="font-size:11px;opacity:0.85;">Good &le; 0.100</div>
                    </div>
                    <div class="dp-an-cwv-tile {ttfb_cls}">
                        <div style="font-size:11px;font-weight:700;text-transform:uppercase;">TTFB (p75)</div>
                        <div style="font-size:20px;font-weight:800;margin:4px 0 2px;font-family:var(--font-mono);">{ttfb_disp}</div>
                        <div style="font-size:11px;opacity:0.85;">Good &le; 800 ms</div>
                    </div>
                </div>
                {cwv_routes_html}
            </div>

            <div class="dp-an-panel">
                <div class="dp-an-panel-head">
                    <div>
                        <h2 class="dp-an-panel-title">Top 404 Not-Found Hits</h2>
                        <p style="margin:2px 0 0;font-size:12px;color:var(--ink-muted);">Broken inbound URLs to rescue with 301s</p>
                    </div>
                    <a href="/d-admin/seo" class="btn btn-secondary" style="padding:4px 10px;font-size:11.5px;text-decoration:none;">Manage Redirects &rarr;</a>
                </div>
                {not_found_html}
            </div>
        </div>

        <script>
        (function() {{
            window.dpSwitchTab = function(btn, paneId) {{
                var panel = btn.closest('.dp-an-panel');
                if (!panel) return;
                panel.querySelectorAll('.dp-an-tab').forEach(function(b) {{ b.classList.remove('is-active'); }});
                btn.classList.add('is-active');
                panel.querySelectorAll('.dp-an-pane').forEach(function(p) {{
                    p.classList.toggle('is-active', p.getAttribute('data-pane') === paneId);
                }});
            }};

            window.dpSaveAnalyticsSettings = function(btn) {{
                var retEl = document.getElementById('dp-an-retention');
                var botsEl = document.getElementById('dp-an-chk-bots');
                var adminsEl = document.getElementById('dp-an-chk-admins');
                var dntEl = document.getElementById('dp-an-chk-dnt');
                var statusEl = document.getElementById('dp-an-save-status');
                if (!retEl) return;
                btn.disabled = true;
                if (statusEl) statusEl.textContent = 'Saving...';
                fetch('/wp-json/daypress/v1/analytics/settings', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{
                        retention_days: parseInt(retEl.value, 10) || 0,
                        filter_bots: !!(botsEl && botsEl.checked),
                        exclude_admins: !!(adminsEl && adminsEl.checked),
                        honor_dnt_gpc: !!(dntEl && dntEl.checked)
                    }})
                }})
                .then(function(r) {{ return r.json(); }})
                .then(function(data) {{
                    btn.disabled = false;
                    if (data && data.saved) {{
                        if (data.settings) {{
                            var rStat = document.getElementById('dp-an-stat-raw');
                            var dStat = document.getElementById('dp-an-stat-daily');
                            if (rStat) rStat.textContent = String(data.settings.raw_events_count);
                            if (dStat) dStat.textContent = String(data.settings.daily_rollup_rows);
                        }}
                        if (statusEl) {{
                            statusEl.textContent = 'Saved (' + (data.pruned_rows || 0) + ' expired rows pruned)';
                        }}
                    }} else if (statusEl) {{
                        statusEl.textContent = 'Error saving settings';
                    }}
                }})
                .catch(function() {{
                    btn.disabled = false;
                    if (statusEl) statusEl.textContent = 'Network error';
                }});
            }};

            // Interactive SVG Chart Crosshair & Tooltip
            var svg = document.getElementById('dp-an-svg');
            var wrap = document.getElementById('dp-an-chart-wrap');
            var tip = document.getElementById('dp-an-tooltip');
            var cross = document.getElementById('dp-an-crosshair');
            var dotPv = document.getElementById('dp-an-dot-pv');
            var dotVis = document.getElementById('dp-an-dot-vis');
            if (svg && wrap && tip && cross && dotPv && dotVis) {{
                svg.querySelectorAll('.dp-an-hit').forEach(function(rect) {{
                    rect.addEventListener('mouseenter', function() {{
                        var x = rect.getAttribute('data-x');
                        var yPv = rect.getAttribute('data-ypv');
                        var yVis = rect.getAttribute('data-yvis');
                        var b = rect.getAttribute('data-bucket') || '';
                        var pv = rect.getAttribute('data-pv') || '0';
                        var vis = rect.getAttribute('data-vis') || '0';
                        var conv = rect.getAttribute('data-conv') || '0';
                        cross.setAttribute('x1', x);
                        cross.setAttribute('x2', x);
                        cross.style.display = 'block';
                        dotPv.setAttribute('cx', x);
                        dotPv.setAttribute('cy', yPv);
                        dotPv.style.display = 'block';
                        dotVis.setAttribute('cx', x);
                        dotVis.setAttribute('cy', yVis);
                        dotVis.style.display = 'block';
                        tip.textContent = '';
                        var strong = document.createElement('div');
                        strong.style.fontWeight = '700';
                        strong.style.marginBottom = '2px';
                        strong.textContent = b;
                        var line1 = document.createElement('div');
                        line1.textContent = 'Pageviews: ' + pv + '  ·  Visitors: ' + vis;
                        tip.appendChild(strong);
                        tip.appendChild(line1);
                        if (conv !== '0') {{
                            var line2 = document.createElement('div');
                            line2.style.color = '#fbbf24';
                            line2.textContent = 'Conversions: ' + conv;
                            tip.appendChild(line2);
                        }}
                        tip.style.display = 'block';
                        var svgBox = svg.getBoundingClientRect();
                        var wrapBox = wrap.getBoundingClientRect();
                        var ratioX = (parseFloat(x) || 0) / {chart_w};
                        var px = (svgBox.left - wrapBox.left) + ratioX * svgBox.width;
                        var leftPos = Math.min(Math.max(12, px - tip.offsetWidth / 2), wrapBox.width - tip.offsetWidth - 12);
                        tip.style.left = leftPos + 'px';
                        tip.style.top = '44px';
                    }});
                }});
                svg.addEventListener('mouseleave', function() {{
                    cross.style.display = 'none';
                    dotPv.style.display = 'none';
                    dotVis.style.display = 'none';
                    tip.style.display = 'none';
                }});
            }}
        }})();
        </script>
        "##,
        live_badge_html = live_badge_html,
        hygiene_pills = hygiene_pills,
        range_human = range_human,
        range_pills = range_pills,
        raw_rows = format_int(telemetry_cfg.raw_events_count),
        daily_rows = format_int(telemetry_cfg.daily_rollup_rows),
        ret_30 = ret_sel(30),
        ret_90 = ret_sel(90),
        ret_180 = ret_sel(180),
        ret_365 = ret_sel(365),
        ret_0 = ret_sel(0),
        chk_bots = chk_bots,
        chk_admins = chk_admins,
        chk_dnt = chk_dnt,
        filter_bar_html = filter_bar_html,
        vis_delta = vis_delta,
        visitors_fmt = format_int(summary.total_visitors),
        pv_delta = pv_delta,
        pageviews_fmt = format_int(summary.total_pageviews),
        views_per_visit = summary.views_per_visit,
        dur_delta = dur_delta,
        avg_dur_fmt = format_duration_human(summary.avg_duration_secs),
        avg_scroll = summary.avg_scroll_depth_pct,
        bounce_delta = bounce_delta,
        bounce = summary.bounce_rate_pct,
        conv_delta = conv_delta,
        conversions_fmt = format_int(summary.total_conversions),
        conv_rate = summary.conversion_rate_pct,
        chart_w = chart_w,
        chart_h = chart_h,
        pad_left = pad_left,
        pad_top = pad_top,
        right_x = chart_w - pad_right,
        mid_y = mid_y,
        bottom_y = bottom_y,
        axis_x = pad_left - 8.0,
        top_lbl_y = pad_top + 4.0,
        mid_lbl_y = mid_y + 4.0,
        bot_lbl_y = bottom_y + 4.0,
        max_val_fmt = format_int(max_val),
        mid_val_fmt = format_int(mid_val),
        pv_area_points = pv_area_points,
        vis_polyline = vis_polyline,
        pv_polyline = pv_polyline,
        conv_dots_svg = conv_dots_svg,
        hover_rects_svg = hover_rects_svg,
        first_bucket = first_bucket,
        mid_bucket = mid_bucket,
        last_bucket = last_bucket,
        top_pages_html = top_pages_html,
        entry_pages_html = entry_pages_html,
        exit_pages_html = exit_pages_html,
        channels_html = channels_html,
        referrers_html = referrers_html,
        utm_source_html = utm_source_html,
        utm_medium_html = utm_medium_html,
        utm_campaign_html = utm_campaign_html,
        countries_html = countries_html,
        devices_html = devices_html,
        browsers_html = browsers_html,
        os_html = os_html,
        goals_html = goals_html,
        outbound_html = outbound_html,
        downloads_html = downloads_html,
        vitals_samples = format_int(vitals.sample_count),
        ov_cls = ov_cls,
        ov_lbl = ov_lbl,
        lcp_cls = lcp_cls,
        lcp_disp = lcp_disp,
        inp_cls = inp_cls,
        inp_disp = inp_disp,
        cls_cls = cls_cls,
        cls_disp = cls_disp,
        ttfb_cls = ttfb_cls,
        ttfb_disp = ttfb_disp,
        cwv_routes_html = cwv_routes_html,
        not_found_html = not_found_html,
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
