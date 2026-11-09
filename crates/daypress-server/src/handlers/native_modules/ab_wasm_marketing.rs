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

/// `/d-admin/ab-testing`: Zero-Flicker Server-Side A/B Split Testing & Real-User Core Web Vitals (`v0.9`).
pub async fn handle_admin_ab_testing(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let ab_tests = db.list_ab_tests(None).unwrap_or_default();
    let vitals = db.get_core_web_vitals_summary().unwrap_or_default();
    let all_posts = db
        .get_posts("any", Some("publish"), None, 100)
        .unwrap_or_default();

    let mut post_options = String::new();
    for p in &all_posts {
        post_options.push_str(&format!(
            r#"<option value="{id}">#{id} — {title} ({pt})</option>"#,
            id = p.id,
            title = templates::escape_html(&p.title.rendered),
            pt = templates::escape_html(&p.post_type),
        ));
    }

    let mut ab_rows = String::new();
    for t in &ab_tests {
        let status_badge = if t.status == "completed" {
            format!(
                r#"<span class="badge" style="background:rgba(16,185,129,0.15);color:#047857;">Winner: Variant {}</span>"#,
                templates::escape_html(t.winner.as_deref().unwrap_or("B"))
            )
        } else {
            r#"<span class="badge" style="background:rgba(59,130,246,0.14);color:#1d4ed8;">Running</span>"#.to_string()
        };
        let uplift_color = if t.uplift_pct > 0.0 {
            "#047857"
        } else if t.uplift_pct < 0.0 {
            "#b91c1c"
        } else {
            "var(--text-muted)"
        };
        let promote_btns = if t.status == "running" {
            format!(
                r#"<button type="button" class="btn btn-primary" style="padding:4px 10px;font-size:11px;" onclick="promoteAbWinner({id}, 'B')">Promote B</button>
                   <button type="button" class="btn" style="padding:4px 10px;font-size:11px;" onclick="promoteAbWinner({id}, 'A')">Keep A</button>"#,
                id = t.id
            )
        } else {
            String::new()
        };

        ab_rows.push_str(&format!(
            r#"<tr>
                <td>
                    <strong>{name}</strong>
                    <div style="font-size:12px;color:var(--text-muted);">{post_title} &middot; Goal: <code>{goal}</code> ({split}% B)</div>
                </td>
                <td style="font-size:12px;">
                    <div><span class="badge">A</span> {ctrl_txt}</div>
                    <div style="margin-top:4px;"><span class="badge" style="background:rgba(168,64,27,0.14);color:#a8401b;">B</span> {var_txt}</div>
                </td>
                <td style="font-family:var(--font-mono);font-size:12px;">
                    <div>A: {ca}/{va} (<strong>{cra}%</strong>)</div>
                    <div>B: {cb}/{vb} (<strong>{crb}%</strong>)</div>
                </td>
                <td style="font-family:var(--font-mono);font-size:12px;">
                    <div style="color:{uplift_color};font-weight:700;">{uplift:+.1}%</div>
                    <div style="color:var(--text-muted);font-size:11px;">{conf:.1}% conf (Z={z:.2})</div>
                </td>
                <td>{status_badge}</td>
                <td style="text-align:right;white-space:nowrap;">
                    {promote_btns}
                    <button type="button" class="btn" style="padding:4px 10px;font-size:11px;color:#b91c1c;" onclick="deleteAbTest({id})">Delete</button>
                </td>
            </tr>"#,
            id = t.id,
            name = templates::escape_html(&t.name),
            post_title = templates::escape_html(&t.post_title),
            goal = templates::escape_html(&t.goal_event),
            split = t.traffic_split,
            ctrl_txt = templates::escape_html(&t.control_text),
            var_txt = templates::escape_html(&t.variant_text),
            ca = t.control_conversions,
            va = t.control_views,
            cra = t.control_cr_pct,
            cb = t.variant_conversions,
            vb = t.variant_views,
            crb = t.variant_cr_pct,
            uplift_color = uplift_color,
            uplift = t.uplift_pct,
            conf = t.confidence_pct,
            z = t.z_score,
            status_badge = status_badge,
            promote_btns = promote_btns,
        ));
    }
    if ab_rows.is_empty() {
        ab_rows.push_str(
            r#"<tr><td colspan="6" style="text-align:center;color:var(--text-muted);padding:24px;">No A/B split experiments created yet. Launch a zero-flicker server-side experiment on the right.</td></tr>"#,
        );
    }

    let body = format!(
        r#"
        <div class="page-header" style="display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:12px;">
            <div>
                <h1 class="page-title">Zero-Flicker A/B Split Testing &amp; Core Web Vitals</h1>
                <p class="page-subtitle">Nelio A/B Testing &amp; NitroPack Telemetry parity: Server-side deterministic split testing with two-proportion Z-test significance + real-user LCP / INP / CLS p75 telemetry.</p>
            </div>
        </div>

        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:16px;margin-bottom:24px;">
            <div class="card" style="padding:18px;">
                <div style="font-size:12px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;">LCP (p75) &middot; &le;2500ms</div>
                <div style="font-size:28px;font-weight:800;margin-top:4px;">{lcp_p75:.0} ms <span class="badge" style="font-size:11px;vertical-align:middle;">{lcp_status}</span></div>
            </div>
            <div class="card" style="padding:18px;">
                <div style="font-size:12px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;">INP (p75) &middot; &le;200ms</div>
                <div style="font-size:28px;font-weight:800;margin-top:4px;">{inp_p75:.0} ms <span class="badge" style="font-size:11px;vertical-align:middle;">{inp_status}</span></div>
            </div>
            <div class="card" style="padding:18px;">
                <div style="font-size:12px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;">CLS (p75) &middot; &le;0.10</div>
                <div style="font-size:28px;font-weight:800;margin-top:4px;">{cls_p75:.3} <span class="badge" style="font-size:11px;vertical-align:middle;">{cls_status}</span></div>
            </div>
            <div class="card" style="padding:18px;">
                <div style="font-size:12px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;">TTFB (p75) &middot; {samples} samples</div>
                <div style="font-size:28px;font-weight:800;margin-top:4px;">{ttfb_p75:.0} ms</div>
            </div>
        </div>

        <div style="display:grid;grid-template-columns:1.4fr 1fr;gap:24px;align-items:start;">
            <div class="card" style="padding:20px;">
                <h3 style="margin:0 0 14px;font-size:16px;">Server-Side A/B Split Experiments</h3>
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>Experiment</th>
                            <th>Control (A) vs Variant (B)</th>
                            <th>Conversions</th>
                            <th>Uplift &amp; Confidence</th>
                            <th>Status</th>
                            <th style="text-align:right;">Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {ab_rows}
                    </tbody>
                </table>
            </div>

            <div class="card" style="padding:20px;">
                <h3 style="margin:0 0 12px;font-size:16px;">Launch Zero-Flicker A/B Experiment</h3>
                <div style="display:flex;flex-direction:column;gap:12px;">
                    <div>
                        <label class="form-label">Experiment Name</label>
                        <input type="text" id="abName" class="form-input" placeholder="Hero CTA Copy Test" />
                    </div>
                    <div>
                        <label class="form-label">Target Post / Page</label>
                        <select id="abPostId" class="form-input">
                            {post_options}
                        </select>
                    </div>
                    <div>
                        <label class="form-label">Control Text / HTML (Variant A — exact text on page)</label>
                        <input type="text" id="abControl" class="form-input" placeholder="Start Your Free Trial" />
                    </div>
                    <div>
                        <label class="form-label">Challenger Text / HTML (Variant B — swapped server-side)</label>
                        <input type="text" id="abVariant" class="form-input" placeholder="Launch Your Site in 60 Seconds" />
                    </div>
                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:10px;">
                        <div>
                            <label class="form-label">Conversion Goal</label>
                            <select id="abGoal" class="form-input">
                                <option value="conversion">Any Conversion</option>
                                <option value="form_submit">Form Submission</option>
                                <option value="newsletter_signup">Newsletter Signup</option>
                                <option value="shortlink">Shortlink Click</option>
                            </select>
                        </div>
                        <div>
                            <label class="form-label">Traffic to Variant B (%)</label>
                            <input type="number" id="abSplit" class="form-input" value="50" min="1" max="99" />
                        </div>
                    </div>
                    <button type="button" class="btn btn-primary" onclick="createAbTest()">Start A/B Experiment</button>
                </div>
            </div>
        </div>

        <script>
            async function createAbTest() {{
                const name = document.getElementById('abName').value.trim();
                const post_id = parseInt(document.getElementById('abPostId').value, 10) || 0;
                const control_text = document.getElementById('abControl').value.trim();
                const variant_text = document.getElementById('abVariant').value.trim();
                const goal_event = document.getElementById('abGoal').value;
                const traffic_split = parseInt(document.getElementById('abSplit').value, 10) || 50;
                if (!name || !post_id || !control_text || !variant_text) {{
                    showToast('Name, target page, Control text, and Variant text are required', true);
                    return;
                }}
                const res = await fetch('/wp-json/daypress/v1/ab-tests', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ name, post_id, control_text, variant_text, goal_event, traffic_split }})
                }});
                if (res.ok) {{
                    showToast('✓ A/B experiment started');
                    setTimeout(() => window.location.reload(), 400);
                }} else {{
                    showToast('Failed to start A/B experiment', true);
                }}
            }}

            async function promoteAbWinner(id, winner) {{
                const res = await fetch('/wp-json/daypress/v1/ab-tests/' + id + '/promote', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ winner }})
                }});
                if (res.ok) {{
                    showToast('✓ Variant ' + winner + ' promoted to live post');
                    setTimeout(() => window.location.reload(), 400);
                }} else {{
                    showToast('Failed to promote winner', true);
                }}
            }}

            async function deleteAbTest(id) {{
                const res = await fetch('/wp-json/daypress/v1/ab-tests/' + id, {{ method: 'DELETE' }});
                if (res.ok) {{
                    showToast('Experiment deleted');
                    setTimeout(() => window.location.reload(), 300);
                }}
            }}
        </script>
        "#,
        lcp_p75 = vitals.lcp_p75_ms,
        lcp_status = templates::escape_html(&vitals.lcp_status),
        inp_p75 = vitals.inp_p75_ms,
        inp_status = templates::escape_html(&vitals.inp_status),
        cls_p75 = vitals.cls_p75,
        cls_status = templates::escape_html(&vitals.cls_status),
        ttfb_p75 = vitals.ttfb_p75_ms,
        samples = vitals.sample_count,
        ab_rows = ab_rows,
        post_options = post_options,
    );

    Html(templates::render_admin_shell(
        "ab-testing",
        "A/B Testing & Core Web Vitals",
        &body,
        &settings,
    ))
}

/// `/d-admin/wasm`: Sandboxed WebAssembly (`.wasm`) Extension Runtime Studio (`v0.9`).
pub async fn handle_admin_wasm_extensions(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let exts = db.list_wasm_extensions().unwrap_or_default();

    let mut ext_rows = String::new();
    for e in &exts {
        let mut caps_badges = Vec::new();
        if e.capabilities.shortcodes {
            caps_badges.push("shortcodes");
        }
        if e.capabilities.kv_storage {
            caps_badges.push("kv_storage");
        }
        if e.capabilities.read_options {
            caps_badges.push("read_options");
        }
        if e.capabilities.content_filter {
            caps_badges.push("content_filter");
        }
        let err_html = if let Some(ref err) = e.last_error {
            format!(
                r#"<div style="color:#b91c1c;font-size:11px;margin-top:4px;">Trap: {}</div>"#,
                templates::escape_html(err)
            )
        } else {
            String::new()
        };
        ext_rows.push_str(&format!(
            r#"<tr>
                <td>
                    <strong>{name}</strong> <span style="font-family:var(--font-mono);font-size:11px;color:var(--text-muted);">v{version}</span>
                    <div style="font-family:var(--font-mono);font-size:11px;color:var(--text-muted);">[{sc}] &middot; {size} B &middot; sha256:{sha}</div>
                    {err_html}
                </td>
                <td style="font-family:var(--font-mono);font-size:12px;">{caps}</td>
                <td style="font-family:var(--font-mono);font-size:12px;">{fuel} ops / {pages}p</td>
                <td style="font-family:var(--font-mono);font-size:12px;">{execs}</td>
                <td style="text-align:right;white-space:nowrap;">
                    <button type="button" class="btn" style="padding:4px 10px;font-size:11px;" onclick="testRunWasmExt('{slug}')">Run</button>
                    <button type="button" class="btn" style="padding:4px 10px;font-size:11px;color:#b91c1c;" onclick="deleteWasmExt('{slug}')">Remove</button>
                </td>
            </tr>"#,
            slug = templates::escape_html(&e.slug),
            name = templates::escape_html(&e.name),
            version = templates::escape_html(&e.version),
            sc = templates::escape_html(&e.shortcode),
            size = e.wasm_size_bytes,
            sha = templates::escape_html(&e.wasm_sha256.chars().take(12).collect::<String>()),
            err_html = err_html,
            caps = templates::escape_html(&caps_badges.join(", ")),
            fuel = e.fuel_limit,
            pages = e.max_memory_pages,
            execs = e.executions,
        ));
    }
    if ext_rows.is_empty() {
        ext_rows.push_str(
            r#"<tr><td colspan="5" style="text-align:center;color:var(--text-muted);padding:24px;">No WebAssembly (.wasm) extensions installed yet. Compile or upload one on the right.</td></tr>"#,
        );
    }

    let body = format!(
        r#"
        <div class="page-header">
            <div>
                <h1 class="page-title">Sandboxed WebAssembly (<code>.wasm</code>) Extension Runtime</h1>
                <p class="page-subtitle">Memory-isolated, instruction-fuel-metered WebAssembly VM with explicit capability permissions (<code>shortcodes</code>, <code>kv_storage</code>, <code>read_options</code>).</p>
            </div>
        </div>

        <div style="display:grid;grid-template-columns:1.35fr 1fr;gap:24px;align-items:start;">
            <div class="card" style="padding:20px;">
                <h3 style="margin:0 0 14px;font-size:16px;">Installed <code>.wasm</code> Extensions ({count})</h3>
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>Extension &amp; Binary Hash</th>
                            <th>Capabilities</th>
                            <th>Sandbox Limits</th>
                            <th>Runs</th>
                            <th style="text-align:right;">Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {ext_rows}
                    </tbody>
                </table>
                <div id="wasmExecResultBox" style="display:none;margin-top:16px;padding:14px 16px;border-radius:6px;background:var(--bg-surface-alt, #ede7da);border:1px solid var(--border, #d6cfc2);font-family:var(--font-mono);font-size:12px;"></div>
            </div>

            <div class="card" style="padding:20px;">
                <h3 style="margin:0 0 12px;font-size:16px;">Compile or Upload <code>.wasm</code> Extension</h3>
                <div style="display:flex;flex-direction:column;gap:10px;">
                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:10px;">
                        <div>
                            <label class="form-label">Extension Slug</label>
                            <input type="text" id="wasmSlug" class="form-input" placeholder="hit-counter-wasm" />
                        </div>
                        <div>
                            <label class="form-label">Shortcode Tag</label>
                            <input type="text" id="wasmShortcode" class="form-input" placeholder="wasm_counter" />
                        </div>
                    </div>
                    <div>
                        <label class="form-label">Display Name</label>
                        <input type="text" id="wasmName" class="form-input" placeholder="Wasm Live Hit Counter" />
                    </div>
                    <div>
                        <label class="form-label">Template / Source (compiled to <code>\0asm</code> v1 binary)</label>
                        <textarea id="wasmTemplate" class="form-input" rows="3" style="font-family:var(--font-mono);font-size:12px;">&lt;span class="dp-wasm-counter"&gt;Site: {{{{option:blogname}}}} &bull; Hits: {{{{kv_incr:hits}}}}&lt;/span&gt;</textarea>
                    </div>
                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:10px;">
                        <div>
                            <label class="form-label">Fuel Limit (instructions)</label>
                            <input type="number" id="wasmFuel" class="form-input" value="50000" />
                        </div>
                        <div>
                            <label class="form-label">Max Memory (64KiB pages)</label>
                            <input type="number" id="wasmPages" class="form-input" value="16" />
                        </div>
                    </div>
                    <div style="display:flex;gap:14px;flex-wrap:wrap;font-size:13px;">
                        <label><input type="checkbox" id="capShortcodes" checked /> <code>shortcodes</code></label>
                        <label><input type="checkbox" id="capKv" checked /> <code>kv_storage</code></label>
                        <label><input type="checkbox" id="capOpts" checked /> <code>read_options</code></label>
                    </div>
                    <button type="button" class="btn btn-primary" onclick="installWasmExt()">Compile &amp; Install .wasm</button>
                </div>
            </div>
        </div>

        <script>
            async function installWasmExt() {{
                const slug = document.getElementById('wasmSlug').value.trim();
                const shortcode = document.getElementById('wasmShortcode').value.trim();
                const name = document.getElementById('wasmName').value.trim() || slug;
                const template_source = document.getElementById('wasmTemplate').value;
                const fuel_limit = parseInt(document.getElementById('wasmFuel').value, 10) || 50000;
                const max_memory_pages = parseInt(document.getElementById('wasmPages').value, 10) || 16;
                const capabilities = {{
                    shortcodes: document.getElementById('capShortcodes').checked,
                    kv_storage: document.getElementById('capKv').checked,
                    read_options: document.getElementById('capOpts').checked,
                    content_filter: false,
                    http_outbound: false
                }};
                if (!slug || !shortcode) {{
                    showToast('Slug and shortcode tag are required', true);
                    return;
                }}
                const res = await fetch('/wp-json/daypress/v1/wasm-extensions', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ slug, name, shortcode, template_source, fuel_limit, max_memory_pages, capabilities }})
                }});
                if (res.ok) {{
                    showToast('✓ .wasm binary compiled & installed');
                    setTimeout(() => window.location.reload(), 400);
                }} else {{
                    showToast('Failed to install .wasm extension', true);
                }}
            }}

            async function testRunWasmExt(slug) {{
                const box = document.getElementById('wasmExecResultBox');
                const res = await fetch('/wp-json/daypress/v1/wasm-extensions/' + encodeURIComponent(slug) + '/execute', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ export_name: 'render_shortcode', attrs: {{ label: 'AdminPreview' }} }})
                }});
                const data = await res.json().catch(() => ({{}}));
                if (box) {{
                    box.style.display = 'block';
                    if (res.ok) {{
                        box.style.color = 'var(--ink, #221f1c)';
                        box.textContent = '✓ Output (' + data.fuel_consumed + ' fuel used, ' + data.fuel_remaining + ' remaining): ' + data.output_html;
                    }} else {{
                        box.style.color = '#b91c1c';
                        box.textContent = '✗ Trap: ' + (data.message || data.error || 'Execution trapped');
                    }}
                }}
            }}

            async function deleteWasmExt(slug) {{
                const res = await fetch('/wp-json/daypress/v1/wasm-extensions/' + encodeURIComponent(slug), {{ method: 'DELETE' }});
                if (res.ok) {{
                    showToast('Wasm extension removed');
                    setTimeout(() => window.location.reload(), 300);
                }}
            }}
        </script>
        "#,
        count = exts.len(),
        ext_rows = ext_rows,
    );

    Html(templates::render_admin_shell(
        "wasm",
        "WebAssembly (.wasm) Runtime",
        &body,
        &settings,
    ))
}

/// `/d-admin/marketing`: Unified Marketing Hub aggregating FormStudio, Newsletters, and Cookieless Analytics.
pub async fn handle_admin_marketing(State(db): State<Database>) -> Html<String> {
    let settings = db.get_settings().unwrap_or_default();
    let has_plugin = |slug: &str| settings.active_plugins.iter().any(|p| p == slug);
    let forms_active = has_plugin("daypress-forms");
    let nl_active = has_plugin("daypress-newsletters");
    let wa_active = has_plugin("whatsapp-chat");
    let wh_active = has_plugin("daypress-webhooks");
    let ab_active = has_plugin("daypress-ab-testing");

    let forms = db.list_forms().unwrap_or_default();
    let total_forms = forms.len();
    let total_submissions: i64 = forms.iter().map(|(_, count)| *count).sum();
    let unread_entries = db
        .list_form_entries_filtered(None, Some("unread"), None, 500)
        .map(|v| v.len())
        .unwrap_or(0);

    let nl_stats = db.get_newsletter_stats().unwrap_or_default();
    let analytics = db.get_analytics_summary().unwrap_or_default();
    let vitals = db.get_core_web_vitals_summary().unwrap_or_default();
    let shortlinks = db.list_shortlinks().unwrap_or_default();
    let ab_tests = db.list_ab_tests(None).unwrap_or_default();
    let redirects = db.list_redirects().unwrap_or_default();

    let mut form_rows = String::new();
    for (f, count) in forms.iter().take(5) {
        form_rows.push_str(&format!(
            r#"<tr>
                <td><strong>{title}</strong> <span style="font-family:var(--font-mono);font-size:11px;color:var(--text-muted);">[daypress_form id="{id}"]</span></td>
                <td style="font-family:var(--font-mono);"><strong>{count}</strong> entries</td>
                <td style="text-align:right;"><a href="/d-admin/forms?form_id={id}" class="btn" style="padding:4px 10px;font-size:11px;">View Vault &rarr;</a></td>
            </tr>"#,
            id = f.id,
            title = templates::escape_html(&f.title),
            count = count,
        ));
    }
    if form_rows.is_empty() {
        form_rows.push_str(
            r#"<tr><td colspan="3" style="text-align:center;color:var(--text-muted);padding:18px;">No forms created yet.</td></tr>"#,
        );
    }

    let mut top_page_rows = String::new();
    for p in analytics.top_pages.iter().take(5) {
        top_page_rows.push_str(&format!(
            r#"<tr>
                <td style="font-family:var(--font-mono);font-size:12px;">{path}</td>
                <td style="text-align:right;font-family:var(--font-mono);"><strong>{views}</strong> views</td>
            </tr>"#,
            path = templates::escape_html(&p.page_url),
            views = p.views,
        ));
    }
    if top_page_rows.is_empty() {
        top_page_rows.push_str(
            r#"<tr><td colspan="2" style="text-align:center;color:var(--text-muted);padding:18px;">No pageviews recorded in the last 30 days.</td></tr>"#,
        );
    }

    let mut header_btns = String::new();
    header_btns.push_str(r#"<a href="/d-admin/seo" class="btn">SEO &amp; Redirects</a>"#);
    if forms_active {
        header_btns.push_str(r#"<a href="/d-admin/forms" class="btn">FormStudio</a>"#);
    }
    if nl_active {
        header_btns.push_str(r#"<a href="/d-admin/newsletters" class="btn">Newsletters</a>"#);
    }
    if wa_active {
        header_btns.push_str(
            r#"<a href="/d-admin/plugin-settings?plugin=whatsapp-chat" class="btn">WhatsApp Chat</a>"#,
        );
    }
    header_btns.push_str(
        r#"<a href="/d-admin/analytics" class="btn btn-primary">Full Analytics &rarr;</a>"#,
    );

    let forms_card = if forms_active {
        format!(
            r#"<div class="card" style="padding:22px;">
                <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;">
                    <div>
                        <h3 style="margin:0;font-size:16px;">FormStudio &amp; Submissions Vault</h3>
                        <p style="margin:4px 0 0;font-size:12.5px;color:var(--text-muted);">{total_forms} active forms &middot; {total_submissions} total entries</p>
                    </div>
                    <a href="/d-admin/forms" class="btn btn-primary" style="padding:6px 12px;font-size:12px;">Open Forms &rarr;</a>
                </div>
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>Form</th>
                            <th>Submissions</th>
                            <th style="text-align:right;">Vault</th>
                        </tr>
                    </thead>
                    <tbody>
                        {form_rows}
                    </tbody>
                </table>
            </div>"#
        )
    } else {
        String::new()
    };

    let nl_card = if nl_active {
        format!(
            r#"<div class="card" style="padding:22px;">
                <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;">
                    <div>
                        <h3 style="margin:0;font-size:16px;">Dispatches, Newsletters &amp; Member Gate</h3>
                        <p style="margin:4px 0 0;font-size:12.5px;color:var(--text-muted);">{confirmed_subs} confirmed &middot; {pending_subs} pending opt-in &middot; {emails_delivered} delivered</p>
                    </div>
                    <a href="/d-admin/newsletters" class="btn btn-primary" style="padding:6px 12px;font-size:12px;">Open Newsletters &rarr;</a>
                </div>
                <div style="padding:14px;border-radius:8px;background:var(--bg-surface-alt, #ede7da);border:1px solid var(--border, #d6cfc2);margin-bottom:12px;">
                    <div style="font-size:12.5px;font-weight:700;margin-bottom:8px;">Quick Add Subscriber</div>
                    <div style="display:flex;gap:8px;flex-wrap:wrap;">
                        <input type="email" id="mktQuickSubEmail" class="form-input" placeholder="subscriber@example.com" style="flex:1;min-width:180px;" />
                        <button type="button" class="btn btn-primary" onclick="quickAddMarketingSubscriber()">Add Subscriber</button>
                    </div>
                </div>
                <div style="font-size:12px;color:var(--text-muted);line-height:1.5;">
                    Embed subscription forms anywhere with <code>[daypress_newsletter]</code> or gate premium articles using <code>subscribers_only</code> / <code>members_only</code>.
                </div>
            </div>"#,
            confirmed_subs = nl_stats.confirmed_subscribers,
            pending_subs = nl_stats.pending_subscribers,
            emails_delivered = nl_stats.total_emails_delivered,
        )
    } else {
        String::new()
    };

    let mut bottom_cards = String::new();
    bottom_cards.push_str(&format!(
        r#"<div class="card" style="padding:20px;display:flex;justify-content:space-between;align-items:center;gap:16px;">
            <div>
                <h4 style="margin:0 0 4px;font-size:15px;">Search, Schema &amp; Redirection Studio</h4>
                <p style="margin:0;font-size:12.5px;color:var(--text-muted);">Core &middot; {redirect_count} active redirect rules &middot; JSON-LD, OpenGraph &amp; 404 Monitor</p>
            </div>
            <a href="/d-admin/seo" class="btn">Open SEO &rarr;</a>
        </div>"#,
        redirect_count = redirects.len(),
    ));
    if wa_active {
        bottom_cards.push_str(
            r#"<div class="card" style="padding:20px;display:flex;justify-content:space-between;align-items:center;gap:16px;">
                <div>
                    <h4 style="margin:0 0 4px;font-size:15px;">Direct Chat &amp; WhatsApp Launcher</h4>
                    <p style="margin:0;font-size:12.5px;color:var(--text-muted);">Status: <strong>Active</strong> &middot; Floating lead chat button</p>
                </div>
                <a href="/d-admin/plugin-settings?plugin=whatsapp-chat" class="btn">Configure &rarr;</a>
            </div>"#,
        );
    }
    if wh_active {
        bottom_cards.push_str(&format!(
            r#"<div class="card" style="padding:20px;display:flex;justify-content:space-between;align-items:center;gap:16px;">
                <div>
                    <h4 style="margin:0 0 4px;font-size:15px;">Branded Shortlinks (<code>/go/*</code>) &amp; Signed Webhooks</h4>
                    <p style="margin:0;font-size:12.5px;color:var(--text-muted);">{shortlink_count} active shortlinks &middot; Located under <strong>Tools &rarr; Shortlinks &amp; Webhooks</strong></p>
                </div>
                <a href="/d-admin/webhooks" class="btn">Open &rarr;</a>
            </div>"#,
            shortlink_count = shortlinks.len(),
        ));
    }
    if ab_active {
        bottom_cards.push_str(&format!(
            r#"<div class="card" style="padding:20px;display:flex;justify-content:space-between;align-items:center;gap:16px;">
                <div>
                    <h4 style="margin:0 0 4px;font-size:15px;">Zero-Flicker A/B Experiments &amp; Web Vitals</h4>
                    <p style="margin:0;font-size:12.5px;color:var(--text-muted);">{ab_count} split experiments &middot; Located under <strong>Tools &rarr; A/B Testing &amp; Vitals</strong></p>
                </div>
                <a href="/d-admin/ab-testing" class="btn">Open &rarr;</a>
            </div>"#,
            ab_count = ab_tests.len(),
        ));
    }

    let inactive_notice = if !forms_active || !nl_active || !wa_active {
        r#"<div class="card" style="padding:18px 22px;margin-top:20px;display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:12px;">
            <div>
                <strong style="font-size:14px;">Enable Additional Marketing Modules</strong>
                <div style="font-size:12.5px;color:var(--text-muted);margin-top:2px;">Activate <em>FormStudio &amp; Submissions Vault</em>, <em>Dispatches, Newsletters &amp; Member Gate</em>, or <em>Direct Chat &amp; WhatsApp Launcher</em> on the Plugins page to add them to your Marketing sidebar.</div>
            </div>
            <a href="/d-admin/plugins" class="btn">Manage Plugins &rarr;</a>
        </div>"#
    } else {
        ""
    };

    let body = format!(
        r#"
        <div class="page-header" style="display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:12px;">
            <div>
                <h1 class="page-title">Marketing Hub</h1>
                <p class="page-subtitle">Unified command center for Cookieless Analytics and active marketing modules.</p>
            </div>
            <div style="display:flex;gap:8px;flex-wrap:wrap;">
                {header_btns}
            </div>
        </div>

        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(210px,1fr));gap:16px;margin-bottom:24px;">
            <div class="card" style="padding:18px;">
                <div style="font-size:12px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;">30d Pageviews / Visitors</div>
                <div style="font-size:28px;font-weight:800;margin-top:4px;">{views_30d} <span style="font-size:13px;font-weight:500;color:var(--text-muted);">({visitors_30d} unique)</span></div>
            </div>
            <div class="card" style="padding:18px;">
                <div style="font-size:12px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;">30d Goal Conversions</div>
                <div style="font-size:28px;font-weight:800;margin-top:4px;">{conversions_30d}</div>
            </div>
            <div class="card" style="padding:18px;">
                <div style="font-size:12px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;">Form Submissions</div>
                <div style="font-size:28px;font-weight:800;margin-top:4px;">{total_submissions} <span style="font-size:13px;font-weight:500;color:var(--text-muted);">({unread_entries} unread)</span></div>
            </div>
            <div class="card" style="padding:18px;">
                <div style="font-size:12px;color:var(--text-muted);text-transform:uppercase;letter-spacing:0.06em;">Confirmed Subscribers</div>
                <div style="font-size:28px;font-weight:800;margin-top:4px;">{confirmed_subs} <span style="font-size:13px;font-weight:500;color:var(--text-muted);">({campaigns_sent} campaigns)</span></div>
            </div>
        </div>

        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(340px,1fr));gap:24px;align-items:start;margin-bottom:24px;">
            {forms_card}
            {nl_card}
            <div class="card" style="padding:22px;">
                <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;">
                    <div>
                        <h3 style="margin:0;font-size:16px;">Cookieless Analytics (30 Days)</h3>
                        <p style="margin:4px 0 0;font-size:12.5px;color:var(--text-muted);">LCP p75: {lcp_p75:.0}ms &middot; INP p75: {inp_p75:.0}ms &middot; CLS p75: {cls_p75:.3}</p>
                    </div>
                    <a href="/d-admin/analytics" class="btn btn-primary" style="padding:6px 12px;font-size:12px;">Open Analytics &rarr;</a>
                </div>
                <table class="data-table">
                    <thead>
                        <tr>
                            <th>Top Route</th>
                            <th style="text-align:right;">Views</th>
                        </tr>
                    </thead>
                    <tbody>
                        {top_page_rows}
                    </tbody>
                </table>
            </div>
        </div>

        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(300px,1fr));gap:20px;">
            {bottom_cards}
        </div>

        {inactive_notice}

        <script>
            async function quickAddMarketingSubscriber() {{
                const email = document.getElementById('mktQuickSubEmail').value.trim();
                if (!email) {{
                    showToast('Please enter an email address', true);
                    return;
                }}
                const res = await fetch('/wp-json/daypress/v1/newsletters/subscribe', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ email, lists: ['general'], skip_double_opt_in: true }})
                }});
                if (res.ok) {{
                    showToast('✓ Subscriber added');
                    setTimeout(() => window.location.reload(), 350);
                }} else {{
                    showToast('Failed to add subscriber', true);
                }}
            }}
        </script>
        "#,
        header_btns = header_btns,
        forms_card = forms_card,
        nl_card = nl_card,
        bottom_cards = bottom_cards,
        inactive_notice = inactive_notice,
        total_submissions = total_submissions,
        unread_entries = unread_entries,
        confirmed_subs = nl_stats.confirmed_subscribers,
        campaigns_sent = nl_stats.total_campaigns_sent,
        views_30d = analytics.total_pageviews,
        visitors_30d = analytics.total_visitors,
        conversions_30d = analytics.total_conversions,
        lcp_p75 = vitals.lcp_p75_ms,
        inp_p75 = vitals.inp_p75_ms,
        cls_p75 = vitals.cls_p75,
        top_page_rows = top_page_rows,
    );

    Html(templates::render_admin_shell(
        "marketing",
        "Marketing Hub",
        &body,
        &settings,
    ))
}
