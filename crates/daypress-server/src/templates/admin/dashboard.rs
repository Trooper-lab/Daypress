use super::layout::render_admin_shell;
use crate::models::{DaypressPostResponse, DaypressSettingsResponse, DbStats};

pub fn render_dashboard_view(
    posts: &[DaypressPostResponse],
    settings: &DaypressSettingsResponse,
    stats: &DbStats,
) -> String {
    let mut sorted_posts: Vec<&DaypressPostResponse> = posts
        .iter()
        .filter(|p| !p.title.rendered.trim().is_empty())
        .collect();
    sorted_posts.sort_by(|a, b| {
        let a_front = a.id == settings.page_on_front;
        let b_front = b.id == settings.page_on_front;
        b_front.cmp(&a_front).then_with(|| b.date.cmp(&a.date))
    });

    let mut recent_posts_html = String::new();
    for p in sorted_posts.iter().take(7) {
        let view_href = if p.id == settings.page_on_front {
            "/".to_string()
        } else if matches!(p.post_type.as_str(), "post" | "page") {
            format!("/{}/", p.slug)
        } else {
            format!("/{}/{}/", p.post_type, p.slug)
        };
        let front_badge = if p.id == settings.page_on_front {
            r#"<span style="font-size:10px;background:var(--moss-wash);color:var(--moss);border:1px solid var(--moss);padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:700;">Front Page</span>"#
        } else {
            ""
        };
        let ptype_badge = format!(
            r#"<span style="font-size:10px;background:var(--paper-sunk);color:var(--ink-muted);border:1px solid var(--rule);padding:1px 6px;border-radius:4px;margin-left:6px;text-transform:uppercase;font-weight:600;">{}</span>"#,
            p.post_type
        );
        recent_posts_html.push_str(&format!(
            r#"<li style="padding: 10px 0; border-bottom: 1px solid var(--rule); display: flex; justify-content: space-between; align-items: center;">
                <div>
                    <a href="/d-admin/post?post={id}&action=edit" style="font-weight: 600; text-decoration: none; color: var(--ink);">{title}</a>{front_badge}{ptype_badge}
                    <div style="font-size: 12px; color: var(--ink-muted); margin-top: 2px;">Published on {date} &bull; <code>{view_href}</code></div>
                </div>
                <div style="display: flex; gap: 6px;">
                    <a href="{view_href}" target="_blank" class="btn btn-secondary" style="padding: 3px 8px; font-size: 11px;">View &nearr;</a>
                    <a href="/d-admin/post?post={id}&action=edit" class="btn btn-secondary" style="padding: 3px 8px; font-size: 11px;">Edit Blocks &amp; ACF</a>
                    <a href="/d-admin/builder?post={id}" class="btn btn-secondary" style="padding: 3px 8px; font-size: 11px; color: var(--press); border-color: var(--press); background: var(--press-wash); font-weight: 600;">DayStudio</a>
                </div>
            </li>"#,
            id = p.id,
            date = p.date.chars().take(10).collect::<String>(),
            view_href = view_href,
            title = p.title.rendered,
            front_badge = front_badge,
            ptype_badge = ptype_badge
        ));
    }
    let total_published_items = if stats.total_posts > 0 {
        stats.total_posts
    } else {
        posts.len() as i64
    };

    let now = chrono::Local::now();
    let greeting = match chrono::Timelike::hour(&now) {
        5..=11 => "Good morning",
        12..=17 => "Good afternoon",
        _ => "Good evening",
    };
    let reader_name = crate::templates::utils::escape_html(
        &crate::auth::current_admin()
            .map(|a| a.display_name)
            .unwrap_or_default(),
    );
    let greeting = if reader_name.is_empty() {
        greeting.to_string()
    } else {
        format!("{greeting}, {reader_name}")
    };
    let today = now.format("%a %-d %b");

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">{greeting}</h1>
            </div>
            <div class="topbar-actions">
                <span class="day-meta">{today}</span>
                <a href="/d-admin/post-new" class="btn btn-primary">New post</a>
            </div>
        </div>

        <!-- 4 KPI Summary Cards -->
        <div class="kpi-grid">
            <div class="kpi-card">
                <div class="kpi-label">Total Posts</div>
                <div class="kpi-value">{total_posts}</div>
                <div class="kpi-subtext">Posts, Pages &amp; Custom Post Types</div>
            </div>
            <div class="kpi-card">
                <div class="kpi-label">Total Pages</div>
                <div class="kpi-value">{total_pages}</div>
                <div class="kpi-subtext">Static Site Pages</div>
            </div>
            <div class="kpi-card">
                <div class="kpi-label">Database Footprint</div>
                <div class="kpi-value">{db_size_kb} <span style="font-size: 14px; font-weight: 500;">KB</span></div>
                <div class="kpi-subtext">{page_count} Pages &bull; {journal_mode} mode</div>
            </div>
            <div class="kpi-card">
                <div class="kpi-label">Pending Comments</div>
                <div class="kpi-value">{pending_comments}</div>
                <div class="kpi-subtext"><a href="/d-admin/edit-comments?comment_status=pending" style="color: inherit;">Review moderation queue &rarr;</a></div>
            </div>
        </div>

        <!-- 2-Column Dashboard Grid -->
        <div style="display: grid; grid-template-columns: 3fr 2fr; gap: 24px;">
            <!-- Left: Quick Composer & Recent Activity -->
            <div>
                <div class="day-card">
                    <div class="day-card-header">
                        <h2 class="day-card-title">Quick Post Composer</h2>
                        <span style="font-size: 11.5px; color: var(--ink-muted);">Instant async save</span>
                    </div>
                    <div class="day-card-body">
                        <form onsubmit="quickPost(event)">
                            <div class="form-group" style="margin-bottom: 12px;">
                                <input type="text" id="quickTitle" class="form-input" placeholder="Title" required />
                            </div>
                            <div class="form-group" style="margin-bottom: 14px;">
                                <textarea id="quickContent" class="form-textarea" rows="3" placeholder="What's happening? Type markdown or plain text..." required></textarea>
                            </div>
                            <div style="display: flex; justify-content: flex-end; gap: 8px;">
                                <button type="button" class="btn btn-secondary" onclick="quickPost(event, 'draft')">Save as Draft</button>
                                <button type="submit" class="btn btn-primary">Publish Now</button>
                            </div>
                        </form>
                    </div>
                </div>

                <div class="day-card">
                    <div class="day-card-header">
                        <h2 class="day-card-title">Recent Activity</h2>
                        <a href="/d-admin/edit" style="font-size: 12px; color: var(--primary); text-decoration: none; font-weight: 500;">View All Posts &rarr;</a>
                    </div>
                    <div class="day-card-body" style="padding-top: 4px; padding-bottom: 8px;">
                        <ul style="list-style: none; margin: 0; padding: 0;">
                            {recent_posts_html}
                        </ul>
                    </div>
                </div>
            </div>

            <!-- Right: Storage Health & Verification -->
            <div>
                <div class="day-card">
                    <div class="day-card-header">
                        <h2 class="day-card-title">Embedded Rusqlite Storage</h2>
                        <span class="badge badge-published">Active (WAL)</span>
                    </div>
                    <div class="day-card-body">
                        <p style="font-size: 13px; color: var(--ink-muted); margin-bottom: 16px;">
                            Zero external MySQL/MariaDB dependencies. All data resides in an ACID-compliant, memory-mapped SQLite file.
                        </p>
                        <div style="background: var(--paper-raised); border: 1px solid var(--border-subtle); border-radius: var(--radius-sm); padding: 12px; margin-bottom: 16px; font-size: 12.5px;">
                            <div style="display: flex; justify-content: space-between; margin-bottom: 6px;">
                                <span style="color: var(--ink-muted);">Path:</span>
                                <code>{db_path}</code>
                            </div>
                            <div style="display: flex; justify-content: space-between; margin-bottom: 6px;">
                                <span style="color: var(--ink-muted);">Page Count:</span>
                                <strong>{page_count} pages ({page_size}B/page)</strong>
                            </div>
                            <div style="display: flex; justify-content: space-between;">
                                <span style="color: var(--ink-muted);">Disk Size:</span>
                                <strong id="dashDbSize">{db_size_kb} KB</strong>
                            </div>
                        </div>
                        <button type="button" class="btn btn-secondary" style="width: 100%; justify-content: center;" onclick="quickVacuum()">
                            Optimize Database (VACUUM)
                        </button>
                    </div>
                </div>

            </div>
        </div>

        <script>
            async function quickPost(e, customStatus) {{
                if (e) e.preventDefault();
                const title = document.getElementById('quickTitle').value;
                const content = document.getElementById('quickContent').value;
                const status = customStatus || 'publish';

                if (!title || !content) return;

                const res = await fetch('/wp-json/wp/v2/posts', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ title, content, status, post_type: 'post' }})
                }});

                if (res.ok) {{
                    showToast('✓ Post successfully saved to SQLite database!');
                    setTimeout(() => window.location.reload(), 800);
                }} else {{
                    showToast('Failed to save post', true);
                }}
            }}

            async function quickVacuum() {{
                try {{
                    const res = await fetch('/wp-json/daypress/v1/vacuum', {{ method: 'POST' }});
                    const data = await res.json();
                    if (res.ok) {{
                        document.getElementById('dashDbSize').textContent = Math.round(data.new_size_bytes / 1024) + ' KB';
                        showToast('✓ SQLite database optimized and compacted!');
                    }}
                }} catch (err) {{
                    showToast('VACUUM failed: ' + err, true);
                }}
            }}
        </script>
        "##,
        total_posts = total_published_items,
        total_pages = stats.total_pages,
        db_size_kb = stats.file_size_bytes / 1024,
        journal_mode = stats.journal_mode,
        recent_posts_html = recent_posts_html,
        pending_comments = settings.pending_comments_count,
        db_path = stats.db_path,
        page_count = stats.page_count,
        page_size = stats.page_size
    );

    render_admin_shell("dashboard", "Dashboard", &body, settings)
}

pub fn render_updates_view(settings: &DaypressSettingsResponse, stats: &DbStats) -> String {
    let default_status = crate::updater::UpdateStatusResponse {
        environment: crate::updater::UpdateEnvironment {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            target_triple: crate::updater::current_target_triple(),
            hosting_mode: "native".to_string(),
            is_container: false,
            can_self_update: true,
            executable_path: "daypress".to_string(),
            rollback_available: false,
            rollback_version_hint: None,
        },
        settings: crate::updater::UpdateSettings::default(),
        state: crate::updater::UpdateState::default(),
        update_available: false,
        history: Vec::new(),
    };
    render_updates_view_with_status(settings, stats, &default_status)
}

pub fn render_updates_view_with_status(
    settings: &DaypressSettingsResponse,
    stats: &DbStats,
    update_status: &crate::updater::UpdateStatusResponse,
) -> String {
    use crate::templates::utils::escape_html;

    let env = &update_status.environment;
    let state = &update_status.state;
    let upd_settings = &update_status.settings;

    let mode_badge = match env.hosting_mode.as_str() {
        "docker" => {
            r#"<span style="display:inline-flex;align-items:center;gap:6px;padding:3px 10px;border-radius:999px;font-size:12px;font-weight:600;background:var(--paper-sunk);border:1px solid var(--rule);color:var(--ink);">🐳 Docker Container (Managed Volume)</span>"#
        }
        "readonly" => {
            r#"<span style="display:inline-flex;align-items:center;gap:6px;padding:3px 10px;border-radius:999px;font-size:12px;font-weight:600;background:var(--paper-sunk);border:1px solid var(--rule);color:var(--ink-muted);">🔒 System Binary (Read-Only Path)</span>"#
        }
        _ => {
            r#"<span style="display:inline-flex;align-items:center;gap:6px;padding:3px 10px;border-radius:999px;font-size:12px;font-weight:600;background:var(--moss-wash, rgba(34,139,34,0.1));border:1px solid var(--moss, #2e7d32);color:var(--moss, #2e7d32);">⚡ Native Binary (1-Click Update Ready)</span>"#
        }
    };

    let last_checked_label = state
        .last_checked_at
        .as_deref()
        .map(escape_html)
        .unwrap_or_else(|| "Never checked yet".to_string());

    let latest_ver_display = state
        .latest_version
        .as_deref()
        .map(escape_html)
        .unwrap_or_else(|| escape_html(&env.current_version));

    let release_notes_html = state
        .release_notes
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|notes| {
            format!(
                r#"<div style="margin-top:14px;padding:12px 14px;background:var(--paper-sunk);border:1px solid var(--rule);border-radius:var(--radius-sm);font-size:12.5px;white-space:pre-wrap;max-height:220px;overflow-y:auto;color:var(--ink);font-family:var(--font-mono, monospace);">{}</div>"#,
                escape_html(notes)
            )
        })
        .unwrap_or_default();

    let sha256_badge = state
        .sha256_digest
        .as_deref()
        .map(|hash| {
            format!(
                r#"<div style="margin-top:8px;font-size:12px;color:var(--ink-muted);">SHA-256: <code style="font-size:11px;padding:2px 6px;background:var(--paper-sunk);border-radius:4px;">{}</code></div>"#,
                escape_html(hash)
            )
        })
        .unwrap_or_default();

    let error_banner_html = state
        .last_error
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map(|err| {
            format!(
                r#"<div style="margin-top:12px;padding:10px 14px;border-radius:var(--radius-sm);background:rgba(217,48,37,0.08);border:1px solid rgba(217,48,37,0.35);color:#b3261e;font-size:12.5px;">Last check warning: {}</div>"#,
                escape_html(err)
            )
        })
        .unwrap_or_default();

    let update_action_card_html = if update_status.update_available {
        let action_controls = if env.can_self_update {
            format!(
                r#"<div style="margin-top:16px;display:flex;flex-wrap:wrap;gap:10px;align-items:center;">
                    <button type="button" id="btn-apply-update" class="btn btn-primary" onclick="applyGithubUpdate()">⬇ Update to Daypress v{latest_ver_display} Now</button>
                    <span style="font-size:12px;color:var(--ink-muted);">Automatically creates a SQLite backup snapshot, verifies SHA-256, smoke-tests the binary, and restarts.</span>
                </div>"#
            )
        } else {
            format!(
                r#"<div style="margin-top:16px;padding:14px;background:var(--paper-sunk);border:1px solid var(--rule);border-radius:var(--radius-sm);">
                    <div style="font-size:13px;font-weight:600;color:var(--ink);margin-bottom:6px;">Container / Read-Only Binary Upgrade Steps</div>
                    <p style="font-size:12.5px;color:var(--ink-muted);margin-bottom:10px;">1. Click below to save a pre-update SQLite snapshot into your persistent <code>/data/backups</code> volume.<br>2. Pull or rebuild the container image on your host machine:</p>
                    <div style="display:flex;gap:10px;flex-wrap:wrap;align-items:center;margin-bottom:10px;">
                        <button type="button" class="btn btn-primary" onclick="createPreUpdateBackup('v{latest_ver_display}')">1. Create Pre-Update SQLite Snapshot</button>
                    </div>
                    <pre style="margin:0;padding:10px 12px;background:#1e1e1e;color:#f5f5f5;border-radius:6px;font-size:12px;overflow-x:auto;"><code>git pull &amp;&amp; docker compose up -d --build</code></pre>
                </div>"#
            )
        };

        format!(
            r#"<div class="day-card" style="padding:24px;margin-bottom:24px;border-left:4px solid var(--press, #2271b1);">
                <div style="display:flex;justify-content:space-between;align-items:flex-start;gap:16px;flex-wrap:wrap;">
                    <div>
                        <span style="display:inline-block;padding:2px 8px;border-radius:4px;font-size:11px;font-weight:700;text-transform:uppercase;background:var(--press-wash, rgba(34,113,177,0.12));color:var(--press, #2271b1);margin-bottom:6px;">New Release Available</span>
                        <h3 style="font-size:18px;font-weight:700;color:var(--ink);">Daypress v{latest_ver_display} is ready</h3>
                        <p style="font-size:13px;color:var(--ink-muted);margin-top:2px;">Target asset: <code>{asset_name}</code></p>
                        {sha256_badge}
                    </div>
                    <div>
                        <a href="{release_url}" target="_blank" rel="noopener" class="btn btn-secondary" style="font-size:12px;">Release Notes on GitHub &nearr;</a>
                    </div>
                </div>
                {release_notes_html}
                {action_controls}
            </div>"#,
            asset_name = escape_html(state.asset_name.as_deref().unwrap_or(
                &crate::updater::expected_asset_filename(&latest_ver_display, &env.target_triple)
            )),
            release_url = escape_html(
                state
                    .release_url
                    .as_deref()
                    .unwrap_or("https://github.com/Trooper-lab/daypress/releases")
            ),
        )
    } else {
        format!(
            r#"<div class="day-card" style="padding:20px 24px;margin-bottom:24px;background:var(--paper);">
                <div style="display:flex;align-items:center;justify-content:space-between;gap:14px;flex-wrap:wrap;">
                    <div style="display:flex;align-items:center;gap:12px;">
                        <div style="width:36px;height:36px;border-radius:50%;background:var(--moss-wash, rgba(34,139,34,0.12));color:var(--moss, #2e7d32);display:flex;align-items:center;justify-content:center;font-weight:700;">✓</div>
                        <div>
                            <div style="font-size:14px;font-weight:700;color:var(--ink);">You are running the latest version of Daypress (v{cur_ver})</div>
                            <div style="font-size:12.5px;color:var(--ink-muted);">Last checked: <span id="last-checked-text">{last_checked_label}</span> &bull; Channel: <code>{channel}</code></div>
                        </div>
                    </div>
                </div>
            </div>"#,
            cur_ver = escape_html(&env.current_version),
            channel = escape_html(&upd_settings.channel),
        )
    };

    let rollback_card_html = if env.rollback_available {
        let prev_ver = env
            .rollback_version_hint
            .as_deref()
            .map(|v| format!("v{}", escape_html(v)))
            .unwrap_or_else(|| "previous binary".to_string());
        let last_backup = update_status
            .history
            .first()
            .and_then(|h| h.backup_filename.as_deref())
            .unwrap_or("");
        let restore_checkbox = if !last_backup.is_empty() {
            format!(
                r#"<label style="display:flex;align-items:center;gap:8px;font-size:12.5px;color:var(--ink);margin-top:10px;">
                    <input type="checkbox" id="rollback-restore-db" value="{}" />
                    Also restore pre-update SQLite snapshot (<code>{}</code>)
                </label>"#,
                escape_html(last_backup),
                escape_html(last_backup)
            )
        } else {
            String::new()
        };
        format!(
            r#"<div class="day-card" style="padding:20px 24px;margin-bottom:24px;border:1px solid var(--rule);">
                <div style="display:flex;justify-content:space-between;align-items:center;gap:16px;flex-wrap:wrap;">
                    <div>
                        <h4 style="font-size:15px;font-weight:700;color:var(--ink);">Instant Rollback Available ({prev_ver})</h4>
                        <p style="font-size:12.5px;color:var(--ink-muted);margin-top:2px;">Your previous executable (<code>daypress.prev</code>) is preserved on disk. If anything looks wrong after an update, revert in one click.</p>
                        {restore_checkbox}
                    </div>
                    <button type="button" class="btn btn-secondary" onclick="rollbackUpdate()">↩ Rollback to {prev_ver}</button>
                </div>
            </div>"#
        )
    } else {
        String::new()
    };

    let mut history_rows_html = String::new();
    for item in &update_status.history {
        let backup_cell = item
            .backup_filename
            .as_deref()
            .map(|b| {
                format!(
                    r#"<a href="/d-admin/backups/download/{}" style="font-family:var(--font-mono, monospace);font-size:11.5px;">{}</a>"#,
                    escape_html(b),
                    escape_html(b)
                )
            })
            .unwrap_or_else(|| "—".to_string());
        let short_sha: String = item.sha256_digest.chars().take(12).collect();
        history_rows_html.push_str(&format!(
            r#"<tr>
                <td style="padding:8px 10px;border-bottom:1px solid var(--rule);font-size:12px;color:var(--ink-muted);">{ts}</td>
                <td style="padding:8px 10px;border-bottom:1px solid var(--rule);font-size:12px;font-weight:600;">{action}</td>
                <td style="padding:8px 10px;border-bottom:1px solid var(--rule);font-size:12px;">v{from} &rarr; v{to}</td>
                <td style="padding:8px 10px;border-bottom:1px solid var(--rule);font-size:12px;">{backup_cell}</td>
                <td style="padding:8px 10px;border-bottom:1px solid var(--rule);font-size:12px;font-family:var(--font-mono, monospace);">{sha}&hellip;</td>
                <td style="padding:8px 10px;border-bottom:1px solid var(--rule);font-size:12px;color:var(--moss, #2e7d32);font-weight:600;">{status}</td>
            </tr>"#,
            ts = escape_html(&item.timestamp),
            action = escape_html(&item.action),
            from = escape_html(&item.from_version),
            to = escape_html(&item.to_version),
            sha = escape_html(&short_sha),
            status = escape_html(&item.status),
        ));
    }

    let history_table_html = if history_rows_html.is_empty() {
        r#"<p style="font-size:12.5px;color:var(--ink-muted);margin:0;">No in-app updates or rollbacks recorded yet.</p>"#.to_string()
    } else {
        format!(
            r#"<div style="overflow-x:auto;">
                <table style="width:100%;border-collapse:collapse;text-align:left;">
                    <thead>
                        <tr style="border-bottom:1px solid var(--rule);font-size:11.5px;color:var(--ink-muted);text-transform:uppercase;">
                            <th style="padding:6px 10px;">Timestamp</th>
                            <th style="padding:6px 10px;">Action</th>
                            <th style="padding:6px 10px;">Version</th>
                            <th style="padding:6px 10px;">Pre-Update Snapshot</th>
                            <th style="padding:6px 10px;">SHA-256</th>
                            <th style="padding:6px 10px;">Status</th>
                        </tr>
                    </thead>
                    <tbody>{history_rows_html}</tbody>
                </table>
            </div>"#
        )
    };

    let auto_check_checked = if upd_settings.auto_check_enabled {
        "checked"
    } else {
        ""
    };
    let channel_stable_selected = if upd_settings.channel != "prerelease" {
        "selected"
    } else {
        ""
    };
    let channel_prerelease_selected = if upd_settings.channel == "prerelease" {
        "selected"
    } else {
        ""
    };

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Updates</h1>
                <p class="page-subtitle">In-app self-updater, release verification, and database maintenance</p>
            </div>
            <div style="display:flex;gap:10px;align-items:center;">
                <button type="button" id="btn-check-updates" class="btn btn-secondary" onclick="checkForUpdatesNow()">↻ Check for Updates Now</button>
                <a href="https://github.com/Trooper-lab/daypress/releases" target="_blank" rel="noopener" class="btn btn-primary">View Releases on GitHub</a>
            </div>
        </div>

        <!-- Live Update Progress Stepper (shown while applying an update or rollback) -->
        <div id="update-progress-panel" class="day-card" style="display:none;padding:24px;margin-bottom:24px;border:2px solid var(--press, #2271b1);">
            <h3 id="update-progress-title" style="font-size:16px;font-weight:700;color:var(--ink);margin-bottom:8px;">Updating Daypress...</h3>
            <p id="update-progress-step" style="font-size:13px;color:var(--ink-muted);margin-bottom:14px;">1/4 Creating pre-update SQLite backup snapshot and verifying SHA-256 checksum...</p>
            <div style="width:100%;height:8px;background:var(--paper-sunk);border-radius:999px;overflow:hidden;">
                <div id="update-progress-bar" style="width:25%;height:100%;background:var(--press, #2271b1);transition:width 0.35s ease;"></div>
            </div>
        </div>

        <div class="day-card" style="padding: 24px; margin-bottom: 24px;">
            <div style="display: flex; align-items: flex-start; justify-content: space-between; gap: 16px; flex-wrap: wrap;">
                <div style="display: flex; align-items: center; gap: 14px;">
                    <div style="width: 44px; height: 44px; border-radius: var(--radius-sm); background: var(--paper-sunk); border: 1px solid var(--rule); display: flex; align-items: center; justify-content: center; color: var(--ink-muted); font-size: 20px;">🦀</div>
                    <div>
                        <div style="display:flex;align-items:center;gap:10px;flex-wrap:wrap;">
                            <h3 style="font-size: 16px; font-weight: 700; color: var(--ink);">Current version: Daypress {version}</h3>
                            {mode_badge}
                        </div>
                        <p style="font-size: 13px; color: var(--ink-muted); margin-top: 4px;">
                            Target architecture: <code>{target_triple}</code> &bull; Binary: <code>{exe_path}</code>
                        </p>
                    </div>
                </div>
            </div>
            {error_banner_html}
            <div style="margin-top: 20px; display: flex; gap: 10px; flex-wrap: wrap;">
                <button type="button" class="btn btn-secondary" onclick="syncSchema()">Check Database Integrity</button>
                <button type="button" class="btn btn-secondary" onclick="runVacuum()">VACUUM Storage</button>
            </div>
        </div>

        {update_action_card_html}
        {rollback_card_html}

        <!-- Manual Archive Upload & Update Preferences Grid -->
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(340px, 1fr)); gap: 20px; margin-bottom: 24px;">
            <div class="day-card" style="padding: 20px;">
                <h4 style="font-size: 15px; font-weight: 600; color: var(--ink); margin-bottom: 6px;">Manual Release Archive Upload</h4>
                <p style="font-size: 12.5px; color: var(--ink-muted); margin-bottom: 14px;">
                    Upload a release archive (<code>.tar.gz</code> or <code>.zip</code>) or compiled <code>daypress</code> binary directly. Daypress will smoke-test the binary, back up SQLite, and hot-swap it.
                </p>
                <form id="manual-update-form" onsubmit="uploadUpdateArchive(event)" style="display:flex;flex-direction:column;gap:10px;">
                    <div>
                        <label style="display:block;font-size:12px;font-weight:600;margin-bottom:4px;">Release Archive (<code>.tar.gz</code>, <code>.zip</code>, or binary)</label>
                        <input type="file" id="update-archive-file" name="archive" accept=".gz,.tgz,.zip,.exe,application/gzip,application/zip,application/octet-stream" required class="form-input" style="width:100%;font-size:12px;" />
                    </div>
                    <div style="display:grid;grid-template-columns: 1fr 2fr;gap:8px;">
                        <div>
                            <label style="display:block;font-size:12px;font-weight:600;margin-bottom:4px;">Version Tag (optional)</label>
                            <input type="text" id="update-archive-version" name="version" placeholder="e.g. 0.2.0" class="form-input" style="width:100%;font-size:12px;" />
                        </div>
                        <div>
                            <label style="display:block;font-size:12px;font-weight:600;margin-bottom:4px;">Expected SHA-256 (optional)</label>
                            <input type="text" id="update-archive-sha256" name="sha256" placeholder="64-character hex digest from SHA256SUMS" class="form-input" style="width:100%;font-size:12px;font-family:var(--font-mono, monospace);" />
                        </div>
                    </div>
                    <div style="margin-top:4px;">
                        <button type="submit" id="btn-upload-update" class="btn btn-secondary">⬆ Verify, Backup &amp; Install Archive</button>
                    </div>
                </form>
            </div>

            <div class="day-card" style="padding: 20px;">
                <h4 style="font-size: 15px; font-weight: 600; color: var(--ink); margin-bottom: 6px;">Update Check Preferences</h4>
                <p style="font-size: 12.5px; color: var(--ink-muted); margin-bottom: 14px;">
                    Configure how Daypress checks GitHub Releases for new versions. All preferences are stored in your SQLite database.
                </p>
                <div style="display:flex;flex-direction:column;gap:12px;">
                    <label style="display:flex;align-items:center;gap:8px;font-size:13px;color:var(--ink);">
                        <input type="checkbox" id="upd-auto-check" {auto_check_checked} />
                        Automatically check for new releases every 24 hours
                    </label>
                    <div>
                        <label style="display:block;font-size:12px;font-weight:600;margin-bottom:4px;">Release Channel</label>
                        <select id="upd-channel" class="form-input" style="width:100%;font-size:13px;">
                            <option value="stable" {channel_stable_selected}>Stable Releases Only (Recommended)</option>
                            <option value="prerelease" {channel_prerelease_selected}>Include Pre-Releases (Beta / RC tags)</option>
                        </select>
                    </div>
                    <div>
                        <label style="display:block;font-size:12px;font-weight:600;margin-bottom:4px;">GitHub Repository</label>
                        <input type="text" id="upd-github-repo" value="{github_repo}" class="form-input" style="width:100%;font-size:12.5px;font-family:var(--font-mono, monospace);" />
                    </div>
                    <div>
                        <button type="button" class="btn btn-secondary" onclick="saveUpdateSettings()">Save Update Preferences</button>
                    </div>
                </div>
            </div>
        </div>

        <!-- Update History Card -->
        <div class="day-card" style="padding: 20px; margin-bottom: 24px;">
            <h4 style="font-size: 15px; font-weight: 600; color: var(--ink); margin-bottom: 12px;">Update &amp; Rollback History</h4>
            {history_table_html}
        </div>

        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px;">
            <div class="day-card" style="padding: 20px;">
                <h4 style="font-size: 15px; font-weight: 600; color: var(--ink); margin-bottom: 8px;">Plugins</h4>
                <p style="font-size: 13px; color: var(--ink-muted);">Built-in modules (forms, SEO, backups) update together with Daypress. WordPress plugins do not run in Daypress.</p>
                <div style="margin-top: 14px;">
                    <a href="/d-admin/plugins" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;">Go to Plugins</a>
                </div>
            </div>
            <div class="day-card" style="padding: 20px;">
                <h4 style="font-size: 15px; font-weight: 600; color: var(--ink); margin-bottom: 8px;">Site Design</h4>
                <p style="font-size: 13px; color: var(--ink-muted);">Design headers and footers in DayStudio, and set colors, fonts and custom CSS in the Customizer.</p>
                <div style="margin-top: 14px;">
                    <a href="/d-admin/headers-footers" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;">Header &amp; Footer</a>
                </div>
            </div>
            <div class="day-card" style="padding: 20px;">
                <h4 style="font-size: 15px; font-weight: 600; color: var(--ink); margin-bottom: 8px;">Database &amp; Engine</h4>
                <p style="font-size: 13px; color: var(--ink-muted);">Embedded Rusqlite database ({db_size_kb} KB, WAL mode) is healthy.</p>
                <div style="margin-top: 14px;">
                    <a href="/d-admin/daypress-engine" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;">Engine Telemetry</a>
                </div>
            </div>
        </div>

        <script>
            function apiHeaders(extra) {{
                const h = Object.assign({{}}, extra || {{}});
                if (window.wpApiSettings && window.wpApiSettings.nonce) {{
                    h['X-WP-Nonce'] = window.wpApiSettings.nonce;
                }}
                return h;
            }}

            async function checkForUpdatesNow() {{
                const btn = document.getElementById('btn-check-updates');
                if (btn) {{ btn.disabled = true; btn.textContent = 'Checking GitHub...'; }}
                try {{
                    const res = await fetch('/wp-json/daypress/v1/update/check', {{
                        method: 'POST',
                        headers: apiHeaders({{ 'Content-Type': 'application/json' }})
                    }});
                    const data = await res.json();
                    if (res.ok) {{
                        showToast('✓ Release check completed');
                        setTimeout(() => window.location.reload(), 500);
                    }} else {{
                        showToast(data.message || data.error || 'Could not check for updates', true);
                    }}
                }} catch (e) {{
                    showToast('Network error while checking for updates', true);
                }} finally {{
                    if (btn) {{ btn.disabled = false; btn.textContent = '↻ Check for Updates Now'; }}
                }}
            }}

            async function saveUpdateSettings() {{
                const payload = {{
                    auto_check_enabled: !!document.getElementById('upd-auto-check').checked,
                    channel: document.getElementById('upd-channel').value,
                    github_repo: document.getElementById('upd-github-repo').value.trim()
                }};
                try {{
                    const res = await fetch('/wp-json/daypress/v1/update/settings', {{
                        method: 'POST',
                        headers: apiHeaders({{ 'Content-Type': 'application/json' }}),
                        body: JSON.stringify(payload)
                    }});
                    if (res.ok) {{
                        showToast('✓ Update preferences saved to SQLite');
                    }} else {{
                        const err = await res.json().catch(() => ({{}}));
                        showToast(err.message || 'Failed to save update preferences', true);
                    }}
                }} catch (e) {{
                    showToast('Network error saving preferences', true);
                }}
            }}

            function showProgress(title, stepText, pct) {{
                const panel = document.getElementById('update-progress-panel');
                if (panel) panel.style.display = 'block';
                const t = document.getElementById('update-progress-title');
                if (t) t.textContent = title;
                const s = document.getElementById('update-progress-step');
                if (s) s.textContent = stepText;
                const b = document.getElementById('update-progress-bar');
                if (b) b.style.width = pct + '%';
            }}

            async function waitForServerRestart() {{
                showProgress('Restarting Daypress Server...', '3/4 Waiting for the updated Daypress binary to bind port and pass health check...', 75);
                for (let attempt = 0; attempt < 30; attempt++) {{
                    await new Promise(r => setTimeout(r, 900));
                    try {{
                        const ping = await fetch('/wp-json/daypress/v1/update/status', {{
                            headers: apiHeaders(),
                            cache: 'no-store'
                        }});
                        if (ping.ok) {{
                            showProgress('Update Complete!', '4/4 Daypress is back online! Reloading dashboard...', 100);
                            showToast('✓ Daypress updated and restarted!');
                            setTimeout(() => window.location.reload(), 600);
                            return;
                        }}
                    }} catch (_) {{
                        // Server is mid-restart; continue polling
                    }}
                }}
                showToast('Update applied. Reloading page...', false);
                window.location.reload();
            }}

            async function applyGithubUpdate() {{
                const btn = document.getElementById('btn-apply-update');
                if (btn) btn.disabled = true;
                showProgress('Installing Daypress Update...', '1/4 Downloading release archive, verifying SHA-256 digest, and creating SQLite backup snapshot...', 35);
                try {{
                    const res = await fetch('/wp-json/daypress/v1/update/apply', {{
                        method: 'POST',
                        headers: apiHeaders({{ 'Content-Type': 'application/json' }})
                    }});
                    const data = await res.json().catch(() => ({{}}));
                    if (res.ok) {{
                        showProgress('Activating New Binary...', '2/4 Backup created (' + (data.backup_filename || 'ok') + '). Swapping executable...', 65);
                        await waitForServerRestart();
                    }} else {{
                        if (btn) btn.disabled = false;
                        document.getElementById('update-progress-panel').style.display = 'none';
                        showToast(data.message || data.error || 'Update failed', true);
                    }}
                }} catch (e) {{
                    // If the connection dropped right as the server restarted, poll for restart
                    await waitForServerRestart();
                }}
            }}

            async function uploadUpdateArchive(ev) {{
                ev.preventDefault();
                const fileInput = document.getElementById('update-archive-file');
                if (!fileInput || !fileInput.files || !fileInput.files[0]) {{
                    showToast('Please choose a release archive file first', true);
                    return;
                }}
                const fd = new FormData();
                fd.append('archive', fileInput.files[0]);
                const ver = document.getElementById('update-archive-version').value.trim();
                if (ver) fd.append('version', ver);
                const sha = document.getElementById('update-archive-sha256').value.trim();
                if (sha) fd.append('sha256', sha);

                const btn = document.getElementById('btn-upload-update');
                if (btn) btn.disabled = true;
                showProgress('Installing Uploaded Release Archive...', '1/4 Uploading archive, verifying SHA-256, smoke-testing binary, and backing up SQLite...', 40);

                try {{
                    const res = await fetch('/wp-json/daypress/v1/update/upload', {{
                        method: 'POST',
                        headers: apiHeaders(),
                        body: fd
                    }});
                    const data = await res.json().catch(() => ({{}}));
                    if (res.ok) {{
                        showProgress('Activating Uploaded Binary...', '2/4 Pre-update backup saved (' + (data.backup_filename || 'ok') + '). Restarting...', 70);
                        await waitForServerRestart();
                    }} else {{
                        if (btn) btn.disabled = false;
                        document.getElementById('update-progress-panel').style.display = 'none';
                        showToast(data.message || data.error || 'Archive update failed', true);
                    }}
                }} catch (e) {{
                    await waitForServerRestart();
                }}
            }}

            async function rollbackUpdate() {{
                const restoreCheckbox = document.getElementById('rollback-restore-db');
                const restoreFile = (restoreCheckbox && restoreCheckbox.checked) ? restoreCheckbox.value : null;
                showProgress('Rolling Back Daypress...', '1/4 Restoring previous binary and creating safety SQLite snapshot...', 45);
                try {{
                    const res = await fetch('/wp-json/daypress/v1/update/rollback', {{
                        method: 'POST',
                        headers: apiHeaders({{ 'Content-Type': 'application/json' }}),
                        body: JSON.stringify({{ restore_backup_filename: restoreFile }})
                    }});
                    const data = await res.json().catch(() => ({{}}));
                    if (res.ok) {{
                        await waitForServerRestart();
                    }} else {{
                        document.getElementById('update-progress-panel').style.display = 'none';
                        showToast(data.message || data.error || 'Rollback failed', true);
                    }}
                }} catch (e) {{
                    await waitForServerRestart();
                }}
            }}

            async function createPreUpdateBackup(versionLabel) {{
                try {{
                    const res = await fetch('/d-admin/api/backups', {{
                        method: 'POST',
                        headers: apiHeaders({{ 'Content-Type': 'application/json' }}),
                        body: JSON.stringify({{ action: 'create', label: 'pre-update-' + versionLabel }})
                    }});
                    const data = await res.json().catch(() => ({{}}));
                    if (res.ok && data.snapshot) {{
                        showToast('✓ Pre-update SQLite snapshot saved: ' + data.snapshot.filename);
                    }} else {{
                        showToast(data.error || 'Backup creation failed', true);
                    }}
                }} catch (e) {{
                    showToast('Network error creating backup', true);
                }}
            }}

            async function syncSchema() {{
                try {{
                    const res = await fetch('/wp-json/daypress/v1/integrity-check');
                    if (res.ok) {{
                        showToast('✓ SQLite database schema and foreign keys verified: ok');
                    }} else {{
                        showToast('Integrity check reported warnings', true);
                    }}
                }} catch (e) {{
                    showToast('Failed to sync schema', true);
                }}
            }}
            async function runVacuum() {{
                try {{
                    const res = await fetch('/wp-json/daypress/v1/vacuum', {{ method: 'POST' }});
                    if (res.ok) {{
                        showToast('✓ Storage compacted and VACUUM completed');
                    }} else {{
                        showToast('VACUUM failed', true);
                    }}
                }} catch (e) {{
                    showToast('Network error during VACUUM', true);
                }}
            }}
        </script>
        "##,
        db_size_kb = stats.file_size_bytes / 1024,
        version = escape_html(&env.current_version),
        mode_badge = mode_badge,
        target_triple = escape_html(&env.target_triple),
        exe_path = escape_html(&env.executable_path),
        error_banner_html = error_banner_html,
        update_action_card_html = update_action_card_html,
        rollback_card_html = rollback_card_html,
        auto_check_checked = auto_check_checked,
        channel_stable_selected = channel_stable_selected,
        channel_prerelease_selected = channel_prerelease_selected,
        github_repo = escape_html(&upd_settings.github_repo),
        history_table_html = history_table_html,
    );

    render_admin_shell("updates", "Updates", &body, settings)
}
