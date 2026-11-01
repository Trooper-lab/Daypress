//! Admin "Publish & Static Edge Hosting" Screen (`/d-admin/publish`).

use crate::models::DaypressSettingsResponse;
use crate::publisher::PublishStatusResponse;
use crate::templates::admin::layout::render_admin_shell;
use crate::templates::utils::escape_html;

pub fn render_publish_view(
    settings: &DaypressSettingsResponse,
    status: &PublishStatusResponse,
) -> String {
    let ps = &status.settings;
    let active_provider = ps.provider.as_str();

    let tab_cf_cls = if active_provider == "cloudflare" {
        "active"
    } else {
        ""
    };
    let tab_nf_cls = if active_provider == "netlify" {
        "active"
    } else {
        ""
    };
    let tab_zip_cls = if active_provider != "cloudflare" && active_provider != "netlify" {
        "active"
    } else {
        ""
    };

    let target_origin = ps.resolved_target_origin();
    let live_url_badge = if let Some(ref last) = status.last_live_deployment {
        if last.target_url.starts_with("http://") || last.target_url.starts_with("https://") {
            format!(
                r#"<a href="{url}" target="_blank" rel="noopener" style="color: var(--moss); font-family: var(--font-mono); font-size: 12.5px; text-decoration: underline;">{url} &nearr;</a>"#,
                url = escape_html(&last.target_url)
            )
        } else {
            format!(
                r#"<span style="color: var(--ink-muted); font-family: var(--font-mono); font-size: 12.5px;">Exported to: {}</span>"#,
                escape_html(&last.target_url)
            )
        }
    } else if !target_origin.is_empty() {
        format!(
            r#"<span style="color: var(--ink-muted); font-family: var(--font-mono); font-size: 12.5px;">Target: {} (not published yet)</span>"#,
            escape_html(&target_origin)
        )
    } else {
        r#"<span style="color: var(--ink-muted); font-size: 12.5px;">Ready for first static publish</span>"#
            .to_string()
    };

    // Pre-flight warnings HTML
    let mut warnings_html = String::new();
    for w in &status.preflight_warnings {
        let (border_col, bg_col, icon) = if w.severity == "warning" {
            ("#d97706", "rgba(217, 119, 6, 0.08)", "&#9888;")
        } else {
            ("var(--rule-strong)", "var(--paper-sunk)", "&#8505;")
        };
        warnings_html.push_str(&format!(
            r#"<div style="display:flex;align-items:flex-start;gap:10px;padding:11px 14px;border-left:3px solid {border_col};background:{bg_col};border-radius:var(--radius-sm);margin-bottom:8px;font-size:13px;color:var(--ink);">
                <span>{icon}</span>
                <div><strong>{source}:</strong> {msg}</div>
            </div>"#,
            border_col = border_col,
            bg_col = bg_col,
            icon = icon,
            source = escape_html(&w.source),
            msg = escape_html(&w.message),
        ));
    }

    // Unpublished changes list HTML
    let changes_count = status.unpublished_changes.len();
    let mut changes_rows_html = String::new();
    if status.unpublished_changes.is_empty() {
        changes_rows_html.push_str(
            r#"<tr><td colspan="4" style="text-align:center;color:var(--moss);padding:18px;font-weight:500;">&#10003; Everything in SQLite matches your last published snapshot! You can still re-publish anytime.</td></tr>"#,
        );
    } else {
        for item in status.unpublished_changes.iter().take(25) {
            let badge_style = if item.change_type == "added" {
                "background:var(--moss-wash);color:var(--moss);"
            } else {
                "background:var(--oxide-wash);color:var(--oxide);"
            };
            changes_rows_html.push_str(&format!(
                r#"<tr>
                    <td><span style="font:600 11px/15px var(--font-mono);text-transform:uppercase;padding:2px 7px;border-radius:4px;{badge_style}">{change_type}</span></td>
                    <td style="font-weight:600;color:var(--ink);">{title} <span style="font-weight:400;color:var(--ink-muted);font-size:12px;">({kind})</span></td>
                    <td style="font-family:var(--font-mono);font-size:12px;color:var(--ink-muted);">{slug}</td>
                    <td style="font-family:var(--font-mono);font-size:12px;color:var(--ink-muted);">{modified}</td>
                </tr>"#,
                badge_style = badge_style,
                change_type = escape_html(&item.change_type),
                title = escape_html(&item.title),
                kind = escape_html(&item.kind),
                slug = escape_html(&item.slug_or_path),
                modified = escape_html(&item.modified_at),
            ));
        }
    }

    // Deployment history table HTML
    let mut history_rows_html = String::new();
    if status.deployments.is_empty() {
        history_rows_html.push_str(
            r#"<tr><td colspan="7" style="text-align:center;color:var(--ink-muted);padding:22px;">No deployments recorded yet. Click <strong>Publish to Live</strong> above to create Deployment #1.</td></tr>"#,
        );
    } else {
        for dep in &status.deployments {
            let status_badge = if dep.status == "live" {
                r#"<span style="background:var(--moss-wash);color:var(--moss);padding:2px 8px;border-radius:99px;font:600 11px var(--font-mono);">&#9679; LIVE</span>"#
            } else {
                r#"<span style="background:var(--oxide-wash);color:var(--oxide);padding:2px 8px;border-radius:99px;font:600 11px var(--font-mono);">FAILED</span>"#
            };
            let kb = (dep.total_bytes as f64) / 1024.0;
            let target_cell = if dep.target_url.starts_with("http://")
                || dep.target_url.starts_with("https://")
            {
                format!(
                    r#"<a href="{u}" target="_blank" rel="noopener" style="font-family:var(--font-mono);font-size:12px;">{u} &nearr;</a>"#,
                    u = escape_html(&dep.target_url)
                )
            } else {
                format!(
                    r#"<code style="font-size:12px;">{}</code>"#,
                    escape_html(&dep.target_url)
                )
            };

            let actions_cell = if dep.bundle_path.is_some() {
                format!(
                    r#"<div style="display:flex;gap:6px;justify-content:flex-end;">
                        <a href="/wp-json/daypress/v1/publish/deployments/{id}/download" class="btn btn-secondary" style="padding:4px 9px;font-size:11.5px;">Download .ZIP</a>
                        <button type="button" class="btn btn-secondary" style="padding:4px 9px;font-size:11.5px;" onclick="rollbackDeployment({id})">Rollback</button>
                    </div>"#,
                    id = dep.id
                )
            } else {
                String::from("&mdash;")
            };

            let commit_extra = if let Some(ref err) = dep.error_message {
                format!(
                    r#"<div style="color:var(--oxide);font-size:11.5px;margin-top:3px;">{}</div>"#,
                    escape_html(err)
                )
            } else {
                String::new()
            };

            history_rows_html.push_str(&format!(
                r#"<tr>
                    <td style="font-family:var(--font-mono);font-weight:600;">#{id}</td>
                    <td>{status_badge}</td>
                    <td><span style="font-family:var(--font-mono);font-size:12px;text-transform:uppercase;">{provider}</span><br/>{target_cell}</td>
                    <td><strong>{commit}</strong>{commit_extra}</td>
                    <td style="font-family:var(--font-mono);font-size:12px;">{pages} pages &middot; {assets} assets<br/><span style="color:var(--ink-muted);">{kb:.1} KB &middot; {ms} ms</span></td>
                    <td style="font-family:var(--font-mono);font-size:12px;color:var(--ink-muted);">{created_at}</td>
                    <td style="text-align:right;">{actions_cell}</td>
                </tr>"#,
                id = dep.id,
                status_badge = status_badge,
                provider = escape_html(&dep.provider),
                target_cell = target_cell,
                commit = escape_html(&dep.commit_message),
                commit_extra = commit_extra,
                pages = dep.pages_count,
                assets = dep.assets_count,
                kb = kb,
                ms = dep.duration_ms,
                created_at = escape_html(&dep.created_at),
                actions_cell = actions_cell,
            ));
        }
    }

    let chk_search = if ps.static_search_enabled {
        "checked"
    } else {
        ""
    };
    let chk_redirects = if ps.static_redirects_enabled {
        "checked"
    } else {
        ""
    };
    let chk_neutralize = if ps.neutralize_admin_links {
        "checked"
    } else {
        ""
    };

    let sel_form_auto = if ps.static_form_mode == "auto" {
        "selected"
    } else {
        ""
    };
    let sel_form_netlify = if ps.static_form_mode == "netlify" {
        "selected"
    } else {
        ""
    };
    let sel_form_webhook = if ps.static_form_mode == "webhook" {
        "selected"
    } else {
        ""
    };

    let active_domain = if !ps.cf_custom_domain.is_empty() {
        &ps.cf_custom_domain
    } else if !ps.netlify_custom_domain.is_empty() {
        &ps.netlify_custom_domain
    } else {
        &ps.production_url
    };

    let body = format!(
        r##"
        <style>
            .pub-hero {{
                background: var(--paper-raised);
                border: 1.5px solid var(--rule-strong);
                border-radius: var(--radius-md);
                padding: 26px 30px;
                margin-bottom: 22px;
                display: flex;
                justify-content: space-between;
                align-items: center;
                gap: 20px;
                flex-wrap: wrap;
            }}
            .pub-grid-2 {{
                display: grid;
                grid-template-columns: 1.15fr 0.85fr;
                gap: 20px;
                margin-bottom: 22px;
            }}
            @media (max-width: 980px) {{
                .pub-grid-2 {{ grid-template-columns: 1fr; }}
            }}
            .pub-card {{
                background: var(--paper-raised);
                border: 1px solid var(--rule-strong);
                border-radius: var(--radius-md);
                padding: 22px 24px;
                margin-bottom: 22px;
            }}
            .pub-card h2 {{
                font: 600 18px/1.3 var(--font-sans);
                color: var(--ink);
                margin: 0 0 6px;
            }}
            .pub-tabs {{
                display: flex;
                gap: 8px;
                margin: 14px 0 18px;
                border-bottom: 1px solid var(--rule);
                padding-bottom: 10px;
                flex-wrap: wrap;
            }}
            .pub-tab-btn {{
                background: var(--paper);
                border: 1px solid var(--rule-strong);
                color: var(--ink-muted);
                padding: 8px 14px;
                border-radius: var(--radius-sm);
                font: 600 12.5px/16px var(--font-sans);
                cursor: pointer;
            }}
            .pub-tab-btn.active {{
                background: var(--ink);
                color: var(--paper);
                border-color: var(--ink);
            }}
            .pub-pane {{ display: none; }}
            .pub-pane.active {{ display: block; }}
            .pub-field {{ margin-bottom: 14px; }}
            .pub-field label {{
                display: block;
                font: 600 12.5px/1.4 var(--font-sans);
                color: var(--ink);
                margin-bottom: 5px;
            }}
            .pub-field input[type="text"],
            .pub-field input[type="password"],
            .pub-field select {{
                width: 100%;
                padding: 9px 12px;
                border: 1px solid var(--rule-strong);
                border-radius: var(--radius-sm);
                background: var(--paper);
                color: var(--ink);
                font: 400 13.5px/1.4 var(--font-sans);
            }}
            .pub-dns-box {{
                background: #111318;
                color: #f3eee4;
                border-radius: var(--radius-sm);
                padding: 14px 16px;
                font: 400 12.5px/1.55 var(--font-mono);
                margin-top: 12px;
                border: 1px solid #272b35;
            }}
        </style>

        <!-- 1. HERO LAUNCHPAD & 1-CLICK PUBLISH -->
        <div class="pub-hero">
            <div style="max-width: 640px;">
                <div style="display:flex;align-items:center;gap:10px;margin-bottom:8px;flex-wrap:wrap;">
                    <span style="font:600 11px/16px var(--font-mono);text-transform:uppercase;letter-spacing:0.08em;background:var(--oxide-wash);color:var(--oxide);padding:3px 10px;border-radius:99px;">
                        Static Edge Publisher &bull; {changes_count} Unpublished Change(s)
                    </span>
                    {live_url_badge}
                </div>
                <h1 style="font:460 30px/1.15 var(--font-serif);color:var(--ink);margin:0 0 8px;">Publish from Local Daypress to the Global Edge</h1>
                <p style="font-size:14px;color:var(--ink-muted);margin:0;line-height:1.5;">
                    Pre-renders every page, post, archive, XML sitemap, RSS feed, <code>_redirects</code>, and client-side search index from your local SQLite database into a zero-server Static Edge bundle—pushed directly to <strong>Cloudflare Pages</strong>, <strong>Netlify</strong>, or a portable <strong>.ZIP</strong> archive.
                </p>
            </div>

            <div style="display:flex;flex-direction:column;gap:10px;min-width:300px;">
                <input type="text" id="pub-commit-msg" placeholder="Deployment note (e.g. Updated homepage hero)..." style="padding:9px 12px;border:1px solid var(--rule-strong);border-radius:var(--radius-sm);background:var(--paper);color:var(--ink);font-size:13px;" />
                <div style="display:flex;gap:8px;flex-wrap:wrap;">
                    <button type="button" class="btn btn-primary" id="btn-run-publish" onclick="triggerPublish(null)" style="flex:1;padding:11px 18px;font-size:14px;justify-content:center;">
                        &#128640; Publish to Live ({active_provider_upper})
                    </button>
                    <button type="button" class="btn btn-secondary" id="btn-run-zip" onclick="triggerPublish('zip')" style="padding:11px 14px;font-size:13px;" title="Compile static bundle and download .ZIP">
                        &#128230; Build .ZIP
                    </button>
                </div>
            </div>
        </div>

        <div id="pub-progress-banner" style="display:none;background:var(--paper-raised);border:1.5px solid var(--oxide);border-radius:var(--radius-md);padding:16px 20px;margin-bottom:20px;">
            <strong id="pub-progress-title" style="display:block;font-size:14.5px;color:var(--ink);">Compiling Static Edge Bundle...</strong>
            <span id="pub-progress-sub" style="font-size:13px;color:var(--ink-muted);">Crawling routes in-memory via Axum SSR and hashing assets...</span>
        </div>

        {warnings_html}

        <!-- 2. UNPUBLISHED LOCAL CHANGES QUEUE -->
        <div class="pub-card">
            <div style="display:flex;justify-content:space-between;align-items:center;margin-bottom:12px;flex-wrap:wrap;gap:8px;">
                <div>
                    <h2>Unpublished Local Changes ({changes_count})</h2>
                    <p style="font-size:13px;color:var(--ink-muted);margin:0;">Diff between your local SQLite content/templates and the most recent live deployment manifest.</p>
                </div>
            </div>
            <div class="table-wrap">
                <table class="wp-list-table">
                    <thead>
                        <tr>
                            <th style="width:100px;">Change</th>
                            <th>Item</th>
                            <th>Route / Path</th>
                            <th style="width:170px;">Last Modified</th>
                        </tr>
                    </thead>
                    <tbody>
                        {changes_rows_html}
                    </tbody>
                </table>
            </div>
        </div>

        <!-- 3. PROVIDER CONFIGURATION & CUSTOM DOMAIN -->
        <div class="pub-grid-2">
            <div class="pub-card" style="margin-bottom:0;">
                <h2>1. Hosting Target (Bring Your Own Provider)</h2>
                <p style="font-size:13px;color:var(--ink-muted);margin:0;">Select where Daypress pushes your static build when you click Publish.</p>

                <input type="hidden" id="pub-provider" value="{active_provider}" />
                <div class="pub-tabs" role="tablist">
                    <button type="button" class="pub-tab-btn {tab_cf_cls}" onclick="selectProviderTab('cloudflare', this)">&#9729; Cloudflare Pages</button>
                    <button type="button" class="pub-tab-btn {tab_nf_cls}" onclick="selectProviderTab('netlify', this)">&#9889; Netlify</button>
                    <button type="button" class="pub-tab-btn {tab_zip_cls}" onclick="selectProviderTab('zip', this)">&#128230; Local Folder &amp; .ZIP</button>
                </div>

                <!-- Cloudflare Pages Pane -->
                <div class="pub-pane {tab_cf_cls}" id="pub-pane-cloudflare">
                    <div class="pub-field">
                        <label for="cf_account_id">Cloudflare Account ID</label>
                        <input type="text" id="cf_account_id" value="{cf_account_id}" placeholder="e.g. 9a8b7c6d5e4f3a2b1c0d..." />
                    </div>
                    <div class="pub-field">
                        <label for="cf_api_token">Cloudflare API Token (Pages:Edit + Zone.DNS:Edit)</label>
                        <input type="password" id="cf_api_token" value="{cf_api_token}" placeholder="Paste Cloudflare API Token..." />
                    </div>
                    <div class="pub-field">
                        <label for="cf_project_name">Cloudflare Pages Project Name</label>
                        <input type="text" id="cf_project_name" value="{cf_project_name}" placeholder="my-daypress-site (creates my-daypress-site.pages.dev)" />
                    </div>
                    <div class="pub-field">
                        <label for="cf_custom_domain">Custom Domain on Cloudflare Pages (optional)</label>
                        <input type="text" id="cf_custom_domain" value="{cf_custom_domain}" placeholder="example.com or www.example.com" />
                    </div>
                </div>

                <!-- Netlify Pane -->
                <div class="pub-pane {tab_nf_cls}" id="pub-pane-netlify">
                    <div class="pub-field">
                        <label for="netlify_token">Netlify Personal Access Token</label>
                        <input type="password" id="netlify_token" value="{netlify_token}" placeholder="nfp_..." />
                    </div>
                    <div class="pub-field">
                        <label for="netlify_site_id">Netlify Site ID or Subdomain</label>
                        <input type="text" id="netlify_site_id" value="{netlify_site_id}" placeholder="e.g. my-daypress-site or site UUID" />
                    </div>
                    <div class="pub-field">
                        <label for="netlify_custom_domain">Custom Domain on Netlify (optional)</label>
                        <input type="text" id="netlify_custom_domain" value="{netlify_custom_domain}" placeholder="example.com" />
                    </div>
                </div>

                <!-- Local Folder / ZIP Pane -->
                <div class="pub-pane {tab_zip_cls}" id="pub-pane-zip">
                    <div class="pub-field">
                        <label for="local_export_dir">Local Static Export Directory</label>
                        <input type="text" id="local_export_dir" value="{local_export_dir}" placeholder="data/static-export" />
                        <span style="display:block;font-size:12px;color:var(--ink-muted);margin-top:4px;">Every publish writes the full static HTML/CSS/media tree here AND saves a downloadable <code>.zip</code> snapshot below.</span>
                    </div>
                </div>

                <hr style="border:none;border-top:1px solid var(--rule);margin:16px 0;" />

                <div class="pub-field">
                    <label for="production_url">Canonical Production URL (Rewrites <code>http://localhost:8090</code>)</label>
                    <input type="text" id="production_url" value="{production_url}" placeholder="https://example.com" />
                </div>

                <div style="display:flex;justify-content:flex-end;gap:10px;margin-top:14px;">
                    <button type="button" class="btn btn-primary" onclick="savePublishSettings()">Save Hosting Settings</button>
                </div>
            </div>

            <!-- Right Column: Custom Domain & Static Features -->
            <div style="display:flex;flex-direction:column;gap:20px;">
                <div class="pub-card" style="margin-bottom:0;">
                    <h2>2. Custom Domain, 1-Click Auto-DNS &amp; SSL</h2>
                    <p style="font-size:13px;color:var(--ink-muted);margin:0 0 12px;">
                        If your domain is in your connected Cloudflare account, Daypress automatically creates the proxied <code>CNAME</code> record and provisions SSL in one click. Otherwise, copy the CNAME record below to your registrar.
                    </p>
                    <div class="pub-field">
                        <label for="domain_action_input">Domain Name</label>
                        <input type="text" id="domain_action_input" value="{active_domain}" placeholder="example.com or www.example.com" />
                    </div>
                    <div style="display:flex;gap:8px;flex-wrap:wrap;">
                        <button type="button" class="btn btn-primary" onclick="configureCustomDomain()">&#9889; Bind Domain &amp; Auto-DNS</button>
                        <button type="button" class="btn btn-secondary" onclick="verifyCustomDomain()">&#128269; Verify DNS &amp; SSL</button>
                    </div>
                    <div class="pub-dns-box" id="domain-dns-output">
                        <div>Record Type: <strong>CNAME</strong></div>
                        <div>Host / Name: <strong id="dns-rec-name">{active_domain_or_root}</strong></div>
                        <div>Target: <strong id="dns-rec-target">{default_cname_target}</strong></div>
                        <div id="dns-verify-line" style="margin-top:6px;color:#94a3b8;">Click "Bind Domain &amp; Auto-DNS" or "Verify DNS &amp; SSL" to check live status.</div>
                    </div>
                </div>

                <div class="pub-card" style="margin-bottom:0;">
                    <h2>3. Static Edge Enhancements</h2>
                    <div style="display:flex;flex-direction:column;gap:10px;margin-top:10px;">
                        <label style="display:flex;align-items:center;gap:8px;font-size:13.5px;cursor:pointer;">
                            <input type="checkbox" id="static_search_enabled" {chk_search} />
                            <span><strong>Client-Side Static Search</strong> (<code>/search-index.json</code> + instant JS search)</span>
                        </label>
                        <label style="display:flex;align-items:center;gap:8px;font-size:13.5px;cursor:pointer;">
                            <input type="checkbox" id="static_redirects_enabled" {chk_redirects} />
                            <span><strong>Compile Edge <code>_redirects</code></strong> from Daypress SEO Redirect rules</span>
                        </label>
                        <label style="display:flex;align-items:center;gap:8px;font-size:13.5px;cursor:pointer;">
                            <input type="checkbox" id="neutralize_admin_links" {chk_neutralize} />
                            <span><strong>Neutralize Local <code>/d-admin/*</code> Links</strong> on public pages</span>
                        </label>
                    </div>
                    <div class="pub-field" style="margin-top:14px;">
                        <label for="static_form_mode">Static Form Submissions</label>
                        <select id="static_form_mode">
                            <option value="auto" {sel_form_auto}>Auto (Netlify Forms on Netlify, Webhook if URL set)</option>
                            <option value="netlify" {sel_form_netlify}>Force Netlify Forms (data-netlify="true")</option>
                            <option value="webhook" {sel_form_webhook}>Custom Webhook / Formspree / Worker Endpoint</option>
                        </select>
                    </div>
                    <div class="pub-field" style="margin-bottom:10px;">
                        <label for="static_form_webhook_url">Form Submission Webhook URL (optional)</label>
                        <input type="text" id="static_form_webhook_url" value="{static_form_webhook_url}" placeholder="https://formspree.io/f/... or Cloudflare Worker URL" />
                    </div>
                    <div style="display:flex;justify-content:flex-end;">
                        <button type="button" class="btn btn-secondary" onclick="savePublishSettings()">Save Enhancements</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- 4. DEPLOYMENT LEDGER & 1-CLICK ROLLBACK -->
        <div class="pub-card">
            <h2>Deployment History &amp; 1-Click Rollback</h2>
            <p style="font-size:13px;color:var(--ink-muted);margin:0 0 14px;">Every publish is archived in SQLite (`daypress_deployments`) with its complete file manifest and downloadable `.zip` snapshot.</p>
            <div class="table-wrap">
                <table class="wp-list-table">
                    <thead>
                        <tr>
                            <th style="width:65px;">ID</th>
                            <th style="width:95px;">Status</th>
                            <th>Target &amp; Provider</th>
                            <th>Deployment Note</th>
                            <th>Bundle Metrics</th>
                            <th style="width:160px;">Timestamp</th>
                            <th style="width:200px;text-align:right;">Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {history_rows_html}
                    </tbody>
                </table>
            </div>
        </div>

        <script>
            function apiHeaders() {{
                const h = {{ 'Content-Type': 'application/json' }};
                if (window.wpApiSettings && window.wpApiSettings.nonce) {{
                    h['X-WP-Nonce'] = window.wpApiSettings.nonce;
                }}
                return h;
            }}

            function selectProviderTab(provider, btn) {{
                document.getElementById('pub-provider').value = provider;
                document.querySelectorAll('.pub-tab-btn').forEach(b => b.classList.remove('active'));
                if (btn) btn.classList.add('active');
                ['cloudflare', 'netlify', 'zip'].forEach(p => {{
                    const pane = document.getElementById('pub-pane-' + p);
                    if (pane) pane.classList.toggle('active', p === provider);
                }});
                const runBtn = document.getElementById('btn-run-publish');
                if (runBtn) {{
                    runBtn.textContent = '🚀 Publish to Live (' + provider.toUpperCase() + ')';
                }}
            }}

            function collectSettingsPayload() {{
                return {{
                    provider: document.getElementById('pub-provider').value,
                    production_url: document.getElementById('production_url').value,
                    cf_account_id: document.getElementById('cf_account_id').value,
                    cf_api_token: document.getElementById('cf_api_token').value,
                    cf_project_name: document.getElementById('cf_project_name').value,
                    cf_custom_domain: document.getElementById('cf_custom_domain').value,
                    netlify_token: document.getElementById('netlify_token').value,
                    netlify_site_id: document.getElementById('netlify_site_id').value,
                    netlify_custom_domain: document.getElementById('netlify_custom_domain').value,
                    local_export_dir: document.getElementById('local_export_dir').value,
                    static_search_enabled: document.getElementById('static_search_enabled').checked,
                    static_redirects_enabled: document.getElementById('static_redirects_enabled').checked,
                    neutralize_admin_links: document.getElementById('neutralize_admin_links').checked,
                    static_form_mode: document.getElementById('static_form_mode').value,
                    static_form_webhook_url: document.getElementById('static_form_webhook_url').value
                }};
            }}

            async function savePublishSettings() {{
                try {{
                    const res = await fetch('/wp-json/daypress/v1/publish/settings', {{
                        method: 'POST',
                        headers: apiHeaders(),
                        body: JSON.stringify(collectSettingsPayload())
                    }});
                    const data = await res.json();
                    if (res.ok) {{
                        showToast('✓ Publish & Hosting settings saved to SQLite!');
                    }} else {{
                        showToast(data.message || data.error || 'Failed to save settings', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function triggerPublish(providerOverride) {{
                // First persist any unsaved form edits so publish uses the latest values
                await fetch('/wp-json/daypress/v1/publish/settings', {{
                    method: 'POST',
                    headers: apiHeaders(),
                    body: JSON.stringify(collectSettingsPayload())
                }}).catch(() => ({{}}));

                const banner = document.getElementById('pub-progress-banner');
                const title = document.getElementById('pub-progress-title');
                const sub = document.getElementById('pub-progress-sub');
                const btn = document.getElementById('btn-run-publish');
                if (banner) banner.style.display = 'block';
                if (btn) btn.disabled = true;
                if (title) title.textContent = 'Compiling Static Edge Bundle & Pushing...';
                if (sub) sub.textContent = 'Rendering routes via in-memory Axum SSR, rewriting URLs, and computing SHA-256 manifest...';

                try {{
                    const commitMsg = document.getElementById('pub-commit-msg').value;
                    const res = await fetch('/wp-json/daypress/v1/publish/run', {{
                        method: 'POST',
                        headers: apiHeaders(),
                        body: JSON.stringify({{
                            commit_message: commitMsg,
                            provider_override: providerOverride
                        }})
                    }});
                    const data = await res.json();
                    if (res.ok && data.deployment) {{
                        const dep = data.deployment;
                        if (title) title.textContent = '✓ Deployment #' + dep.id + ' Complete (' + dep.pages_count + ' pages, ' + dep.assets_count + ' assets in ' + dep.duration_ms + ' ms)';
                        if (sub) sub.textContent = 'Target: ' + dep.target_url;
                        showToast('✓ Published Deployment #' + dep.id + '!');
                        if (providerOverride === 'zip') {{
                            window.location.href = '/wp-json/daypress/v1/publish/deployments/' + dep.id + '/download';
                            setTimeout(() => window.location.reload(), 900);
                        }} else {{
                            setTimeout(() => window.location.reload(), 700);
                        }}
                    }} else {{
                        if (btn) btn.disabled = false;
                        if (title) title.textContent = 'Deployment Failed';
                        if (sub) sub.textContent = data.message || data.error || 'Check provider credentials.';
                        showToast(data.message || data.error || 'Publish failed', true);
                    }}
                }} catch (e) {{
                    if (btn) btn.disabled = false;
                    showToast('Publish error: ' + e, true);
                }}
            }}

            async function rollbackDeployment(id) {{
                try {{
                    showToast('Rolling back to Deployment #' + id + '...');
                    const res = await fetch('/wp-json/daypress/v1/publish/rollback/' + id, {{
                        method: 'POST',
                        headers: apiHeaders()
                    }});
                    const data = await res.json();
                    if (res.ok) {{
                        showToast('✓ Rolled back to Deployment #' + id + '!');
                        setTimeout(() => window.location.reload(), 600);
                    }} else {{
                        showToast(data.message || data.error || 'Rollback failed', true);
                    }}
                }} catch (e) {{
                    showToast('Rollback error: ' + e, true);
                }}
            }}

            async function configureCustomDomain() {{
                await savePublishSettings();
                const dom = document.getElementById('domain_action_input').value.trim();
                const prov = document.getElementById('pub-provider').value;
                const line = document.getElementById('dns-verify-line');
                if (line) line.textContent = 'Configuring domain & checking DNS zones...';
                try {{
                    const res = await fetch('/wp-json/daypress/v1/publish/domain/configure', {{
                        method: 'POST',
                        headers: apiHeaders(),
                        body: JSON.stringify({{ domain: dom, provider: prov }})
                    }});
                    const data = await res.json();
                    if (res.ok) {{
                        document.getElementById('dns-rec-name').textContent = data.record_name || dom;
                        document.getElementById('dns-rec-target').textContent = data.record_target || '';
                        if (line) {{
                            line.style.color = data.auto_dns_provisioned ? '#4ade80' : '#fbbf24';
                            line.textContent = data.message;
                        }}
                        showToast(data.message);
                    }} else {{
                        if (line) line.textContent = data.message || data.error || 'Domain configuration failed.';
                        showToast(data.message || data.error || 'Domain configuration failed', true);
                    }}
                }} catch (e) {{
                    showToast('Domain error: ' + e, true);
                }}
            }}

            async function verifyCustomDomain() {{
                const dom = document.getElementById('domain_action_input').value.trim();
                const line = document.getElementById('dns-verify-line');
                if (line) line.textContent = 'Checking live DNS resolution & SSL certificate...';
                try {{
                    const res = await fetch('/wp-json/daypress/v1/publish/domain/verify', {{
                        method: 'POST',
                        headers: apiHeaders(),
                        body: JSON.stringify({{ domain: dom }})
                    }});
                    const data = await res.json();
                    if (res.ok) {{
                        if (line) {{
                            line.style.color = data.dns_resolved ? '#4ade80' : '#fbbf24';
                            line.textContent = data.status_message;
                        }}
                        showToast(data.status_message, !data.dns_resolved);
                    }}
                }} catch (e) {{
                    showToast('Verification error: ' + e, true);
                }}
            }}
        </script>
        "##,
        changes_count = changes_count,
        live_url_badge = live_url_badge,
        active_provider = escape_html(active_provider),
        active_provider_upper = escape_html(&active_provider.to_ascii_uppercase()),
        warnings_html = warnings_html,
        changes_rows_html = changes_rows_html,
        tab_cf_cls = tab_cf_cls,
        tab_nf_cls = tab_nf_cls,
        tab_zip_cls = tab_zip_cls,
        cf_account_id = escape_html(&ps.cf_account_id),
        cf_api_token = escape_html(&ps.cf_api_token),
        cf_project_name = escape_html(&ps.cf_project_name),
        cf_custom_domain = escape_html(&ps.cf_custom_domain),
        netlify_token = escape_html(&ps.netlify_token),
        netlify_site_id = escape_html(&ps.netlify_site_id),
        netlify_custom_domain = escape_html(&ps.netlify_custom_domain),
        local_export_dir = escape_html(&ps.local_export_dir),
        production_url = escape_html(&ps.production_url),
        active_domain = escape_html(active_domain),
        active_domain_or_root = escape_html(if active_domain.is_empty() {
            "example.com"
        } else {
            active_domain
        }),
        default_cname_target = escape_html(&if !ps.cf_project_name.is_empty() {
            format!("{}.pages.dev", ps.cf_project_name)
        } else if !ps.netlify_site_id.is_empty() {
            format!("{}.netlify.app", ps.netlify_site_id)
        } else {
            "your-project.pages.dev".to_string()
        }),
        chk_search = chk_search,
        chk_redirects = chk_redirects,
        chk_neutralize = chk_neutralize,
        sel_form_auto = sel_form_auto,
        sel_form_netlify = sel_form_netlify,
        sel_form_webhook = sel_form_webhook,
        static_form_webhook_url = escape_html(&ps.static_form_webhook_url),
        history_rows_html = history_rows_html,
    );

    render_admin_shell("publish", "Publish & Hosting", &body, settings)
}
