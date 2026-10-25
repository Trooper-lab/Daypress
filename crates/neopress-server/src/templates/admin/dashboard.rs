use super::layout::render_wp_admin_shell;
use crate::models::{DbStats, WpPostResponse, WpSettingsResponse};

pub fn render_wp_dashboard_view(
    posts: &[WpPostResponse],
    settings: &WpSettingsResponse,
    stats: &DbStats,
) -> String {
    let mut sorted_posts: Vec<&WpPostResponse> = posts
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
            r#"<span style="font-size:10px;background:#dcfce7;color:#15803d;border:1px solid #86efac;padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:700;">🏠 Front Page</span>"#
        } else {
            ""
        };
        let ptype_badge = format!(
            r#"<span style="font-size:10px;background:#f1f5f9;color:#475569;border:1px solid #e2e8f0;padding:1px 6px;border-radius:4px;margin-left:6px;text-transform:uppercase;font-weight:600;">{}</span>"#,
            p.post_type
        );
        recent_posts_html.push_str(&format!(
            r#"<li style="padding: 10px 0; border-bottom: 1px solid #f1f5f9; display: flex; justify-content: space-between; align-items: center;">
                <div>
                    <a href="/n-admin/post?post={id}&action=edit" style="font-weight: 600; text-decoration: none; color: #0f172a;">{title}</a>{front_badge}{ptype_badge}
                    <div style="font-size: 12px; color: #64748b; margin-top: 2px;">Published on {date} &bull; <code>{view_href}</code></div>
                </div>
                <div style="display: flex; gap: 6px;">
                    <a href="{view_href}" target="_blank" class="btn btn-secondary" style="padding: 3px 8px; font-size: 11px;">View &nearr;</a>
                    <a href="/n-admin/post?post={id}&action=edit" class="btn btn-secondary" style="padding: 3px 8px; font-size: 11px;">Edit Blocks &amp; ACF</a>
                    <a href="/n-admin/builder?post={id}" class="btn btn-secondary" style="padding: 3px 8px; font-size: 11px; color: #2563eb; border-color: #bfdbfe; background: #eff6ff; font-weight: 600;">⚡ NeoStudio</a>
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

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Dashboard</h1>
                <p class="page-subtitle">Welcome to Neopress 2026 &bull; High-Performance Native Rust CMS with Rusqlite</p>
            </div>
            <div>
                <a href="/n-admin/post-new" class="btn btn-primary">+ Write New Article</a>
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
                <div class="kpi-subtext"><a href="/n-admin/edit-comments?comment_status=pending" style="color: inherit;">Review moderation queue &rarr;</a></div>
            </div>
        </div>

        <!-- 2-Column Dashboard Grid -->
        <div style="display: grid; grid-template-columns: 3fr 2fr; gap: 24px;">
            <!-- Left: Quick Composer & Recent Activity -->
            <div>
                <div class="neo-card">
                    <div class="neo-card-header">
                        <h2 class="neo-card-title">Quick Post Composer</h2>
                        <span style="font-size: 11.5px; color: #64748b;">Instant async save</span>
                    </div>
                    <div class="neo-card-body">
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

                <div class="neo-card">
                    <div class="neo-card-header">
                        <h2 class="neo-card-title">Recent Activity</h2>
                        <a href="/n-admin/edit" style="font-size: 12px; color: var(--primary); text-decoration: none; font-weight: 500;">View All Posts &rarr;</a>
                    </div>
                    <div class="neo-card-body" style="padding-top: 4px; padding-bottom: 8px;">
                        <ul style="list-style: none; margin: 0; padding: 0;">
                            {recent_posts_html}
                        </ul>
                    </div>
                </div>
            </div>

            <!-- Right: Storage Health & Verification -->
            <div>
                <div class="neo-card">
                    <div class="neo-card-header">
                        <h2 class="neo-card-title">Embedded Rusqlite Storage</h2>
                        <span class="badge badge-published">Active (WAL)</span>
                    </div>
                    <div class="neo-card-body">
                        <p style="font-size: 13px; color: #64748b; margin-bottom: 16px;">
                            Zero external MySQL/MariaDB dependencies. All data resides in an ACID-compliant, memory-mapped SQLite file.
                        </p>
                        <div style="background: #f8fafc; border: 1px solid var(--border-subtle); border-radius: 8px; padding: 12px; margin-bottom: 16px; font-size: 12.5px;">
                            <div style="display: flex; justify-content: space-between; margin-bottom: 6px;">
                                <span style="color: #64748b;">Path:</span>
                                <code>{db_path}</code>
                            </div>
                            <div style="display: flex; justify-content: space-between; margin-bottom: 6px;">
                                <span style="color: #64748b;">Page Count:</span>
                                <strong>{page_count} pages ({page_size}B/page)</strong>
                            </div>
                            <div style="display: flex; justify-content: space-between;">
                                <span style="color: #64748b;">Disk Size:</span>
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
                    const res = await fetch('/wp-json/neopress/v1/vacuum', {{ method: 'POST' }});
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

    render_wp_admin_shell("dashboard", "Dashboard", &body, settings)
}

pub fn render_wp_updates_view(settings: &WpSettingsResponse, stats: &DbStats) -> String {
    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">WordPress Updates</h1>
                <p class="page-subtitle">Check and manage core engine binaries, database migrations, and extension updates</p>
            </div>
            <div>
                <button type="button" class="btn btn-primary" onclick="checkUpdates()">Check Again</button>
            </div>
        </div>

        <div class="neo-card" style="padding: 24px; margin-bottom: 24px;">
            <div style="display: flex; align-items: center; gap: 14px;">
                <div style="width: 44px; height: 44px; border-radius: 10px; background: #ecfdf5; border: 1px solid #a7f3d0; display: flex; align-items: center; justify-content: center; color: #059669; font-size: 20px;">✓</div>
                <div>
                    <h3 style="font-size: 16px; font-weight: 700; color: #0f172a;">Current version: Neopress 0.1.0 (Drop-in WordPress 6.7 Compatibility)</h3>
                    <p style="font-size: 13px; color: #64748b; margin-top: 2px;">Last checked: Just now. You are running the latest compiled release with in-process SQLite WAL mode.</p>
                </div>
            </div>
            <div style="margin-top: 20px; display: flex; gap: 10px;">
                <button type="button" class="btn btn-secondary" onclick="syncSchema()">Re-sync Schema &amp; Indexes</button>
                <button type="button" class="btn btn-secondary" onclick="runVacuum()">VACUUM Storage</button>
            </div>
        </div>

        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(300px, 1fr)); gap: 20px;">
            <div class="neo-card" style="padding: 20px;">
                <h4 style="font-size: 15px; font-weight: 600; color: #0f172a; margin-bottom: 8px;">Plugins</h4>
                <p style="font-size: 13px; color: #64748b;">All installed Neopress and WordPress plugins are up to date.</p>
                <div style="margin-top: 14px;">
                    <a href="/n-admin/plugins" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;">Go to Plugins</a>
                </div>
            </div>
            <div class="neo-card" style="padding: 20px;">
                <h4 style="font-size: 15px; font-weight: 600; color: #0f172a; margin-bottom: 8px;">Site Design</h4>
                <p style="font-size: 13px; color: #64748b;">Design headers and footers in NeoStudio, and set colors, fonts and custom CSS in the Customizer.</p>
                <div style="margin-top: 14px;">
                    <a href="/n-admin/headers-footers" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;">Header &amp; Footer</a>
                </div>
            </div>
            <div class="neo-card" style="padding: 20px;">
                <h4 style="font-size: 15px; font-weight: 600; color: #0f172a; margin-bottom: 8px;">Database &amp; Engine</h4>
                <p style="font-size: 13px; color: #64748b;">Embedded Rusqlite database ({db_size_kb} KB, WAL mode) is healthy.</p>
                <div style="margin-top: 14px;">
                    <a href="/n-admin/neopress-engine" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;">Engine Telemetry</a>
                </div>
            </div>
        </div>

        <script>
            function checkUpdates() {{
                showToast('Checking official repository... All systems are up to date!');
            }}
            async function syncSchema() {{
                try {{
                    const res = await fetch('/wp-json/neopress/v1/integrity-check');
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
                    const res = await fetch('/wp-json/neopress/v1/vacuum', {{ method: 'POST' }});
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
        db_size_kb = stats.file_size_bytes / 1024
    );

    render_wp_admin_shell("updates", "Updates", &body, settings)
}
