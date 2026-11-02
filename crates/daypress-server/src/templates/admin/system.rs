use super::layout::render_admin_shell;
use crate::hot_cache::HotCacheStats;
use crate::models::{DaypressCategoryResponse, DaypressSettingsResponse, DbStats};
use crate::templates::utils::escape_html;

// 5. Daypress Engine & Verification Loop View (/d-admin/daypress-engine)
pub fn render_engine_view(settings: &DaypressSettingsResponse, stats: &DbStats) -> String {
    let default_cache = HotCacheStats::default();
    render_engine_view_with_cache(settings, stats, &default_cache, "", false)
}

pub fn render_engine_view_with_cache(
    settings: &DaypressSettingsResponse,
    stats: &DbStats,
    cache_stats: &HotCacheStats,
    cf_zone_id: &str,
    has_cf_token: bool,
) -> String {
    let kb_size = stats.file_size_bytes as f64 / 1024.0;
    let cf_zone_esc = escape_html(cf_zone_id);
    let cf_token_placeholder = if has_cf_token {
        "•••••••••••••••• (Saved in SQLite)"
    } else {
        "Cloudflare API Token (Zone.Cache Purge)"
    };
    let last_purged = match cache_stats.last_purged_at.as_deref() {
        Some(ts) if !ts.is_empty() => escape_html(ts),
        _ => "Never (Fresh start)".to_string(),
    };
    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Daypress Rusqlite Engine &amp; Hot Cache</h1>
                <p class="page-subtitle">Native in-process SQLite diagnostics, &lt;50&micro;s SSR Hot Cache, and CDN edge purge</p>
            </div>
            <div style="display: flex; gap: 10px;">
                <button type="button" class="btn btn-secondary" onclick="purgeHotCache()">&#9889; Purge Hot Cache</button>
                <button type="button" class="btn btn-secondary" onclick="runVacuum()">VACUUM Storage</button>
                <button type="button" class="btn btn-primary" onclick="runIntegrityCheck()">Integrity Check</button>
            </div>
        </div>

        <div class="day-metrics-grid" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 16px; margin-bottom: 24px;">
            <div class="day-card" style="padding: 20px;">
                <div style="font-size: 12px; font-weight: 600; color: var(--ink-muted); text-transform: uppercase;">SSR Hot Cache Hit Ratio</div>
                <div style="font-size: 20px; font-weight: 700; color: var(--moss); margin-top: 4px;" id="hcHitRatio">{hit_ratio:.1}%</div>
                <div style="font-size: 12px; color: var(--ink-muted); margin-top: 4px;"><span id="hcHits">{hits}</span> hits &bull; <span id="hcMisses">{misses}</span> misses</div>
            </div>
            <div class="day-card" style="padding: 20px;">
                <div style="font-size: 12px; font-weight: 600; color: var(--ink-muted); text-transform: uppercase;">Pre-Compressed Routes</div>
                <div style="font-size: 20px; font-weight: 700; color: var(--ink); margin-top: 4px;"><span id="hcEntries">{entries}</span> routes</div>
                <div style="font-size: 12px; color: var(--ink-muted); margin-top: 4px;">{raw_kb:.1} KB HTML &rarr; {gz_kb:.1} KB Gzip</div>
            </div>
            <div class="day-card" style="padding: 20px;">
                <div style="font-size: 12px; font-weight: 600; color: var(--ink-muted); text-transform: uppercase;">Storage File</div>
                <div style="font-size: 20px; font-weight: 700; color: var(--ink); margin-top: 4px;">{kb_size:.1} KB</div>
                <div style="font-size: 12px; color: var(--moss); margin-top: 4px;">{journal} Mode</div>
            </div>
            <div class="day-card" style="padding: 20px;">
                <div style="font-size: 12px; font-weight: 600; color: var(--ink-muted); text-transform: uppercase;">Total Posts &amp; Pages</div>
                <div style="font-size: 20px; font-weight: 700; color: var(--ink); margin-top: 4px;">{posts} / {pages}</div>
                <div style="font-size: 12px; color: var(--ink-muted); margin-top: 4px;">Relational objects</div>
            </div>
            <div class="day-card" style="padding: 20px;">
                <div style="font-size: 12px; font-weight: 600; color: var(--ink-muted); text-transform: uppercase;">System Options</div>
                <div style="font-size: 20px; font-weight: 700; color: var(--ink); margin-top: 4px;">{options}</div>
                <div style="font-size: 12px; color: var(--ink-muted); margin-top: 4px;">Auto-loaded config</div>
            </div>
        </div>

        <div class="day-card" style="padding: 24px; margin-bottom: 24px;">
            <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom: 12px; flex-wrap:wrap; gap:12px;">
                <div>
                    <h3 style="font-size: 16px; font-weight: 600; color: var(--ink); margin: 0;">&lt;50&micro;s Pre-Compressed SSR Hot Cache &amp; Edge Purge</h3>
                    <p style="font-size: 13px; color: var(--ink-muted); margin: 4px 0 0 0;">Anonymous GET requests are served directly from RAM with pre-compressed Gzip payloads and ETag validation. Any content, layout, or option save automatically invalidates the cache (`{purges}` invalidations total; last: `{last_purged}`).</p>
                </div>
                <button type="button" class="btn btn-primary" onclick="purgeHotCache()">Purge Cache Now</button>
            </div>
            <div style="display:grid; grid-template-columns: 1fr 1fr auto; gap: 12px; align-items: end; margin-top: 16px; padding-top: 16px; border-top: 1px solid var(--border);">
                <div>
                    <label class="form-label">Cloudflare Zone ID (Optional auto-purge on save)</label>
                    <input type="text" id="cfZoneId" class="form-input" value="{cf_zone_esc}" placeholder="e.g. 023e105f4ecef8ad9ca31a8372d0c353" />
                </div>
                <div>
                    <label class="form-label">Cloudflare API Token</label>
                    <input type="password" id="cfApiToken" class="form-input" placeholder="{cf_token_placeholder}" />
                </div>
                <button type="button" class="btn btn-secondary" onclick="saveCfPurgeSettings()">Save &amp; Purge</button>
            </div>
        </div>

        <div class="day-card" style="padding: 24px; margin-bottom: 24px;">
            <h3 style="font-size: 16px; font-weight: 600; color: var(--ink); margin-bottom: 12px;">Database Configuration &amp; Storage Path</h3>
            <table class="day-table">
                <tbody>
                    <tr><td style="width: 240px; font-weight: 600;">Database Path</td><td style="font-family: var(--font-mono); font-size: 13px;">{db_path}</td></tr>
                    <tr><td style="font-weight: 600;">Journal Mode</td><td><span class="badge badge-published">{journal} (Zero Server Overhead)</span></td></tr>
                    <tr><td style="font-weight: 600;">Synchronous Pragma</td><td><code>NORMAL</code> (High Reliability + Max IOPS)</td></tr>
                    <tr><td style="font-weight: 600;">Foreign Keys</td><td><code>ON</code> (Relational Integrity Enforced)</td></tr>
                </tbody>
            </table>
        </div>

        <script>
        async function purgeHotCache() {{
            try {{
                const res = await fetch('/wp-json/daypress/v1/cache/purge', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{}})
                }});
                const data = await res.json();
                if (typeof showToast === 'function') {{
                    showToast('✓ SSR Hot Cache purged!');
                }} else {{
                    alert(data.message || 'Hot cache purged!');
                }}
                setTimeout(() => location.reload(), 350);
            }} catch (err) {{
                alert('Error purging hot cache: ' + err);
            }}
        }}
        async function saveCfPurgeSettings() {{
            const cf_zone_id = document.getElementById('cfZoneId').value.trim();
            const cf_api_token = document.getElementById('cfApiToken').value.trim();
            try {{
                const res = await fetch('/wp-json/daypress/v1/cache/purge', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ cf_zone_id, cf_api_token: cf_api_token || undefined }})
                }});
                if (res.ok) {{
                    if (typeof showToast === 'function') showToast('✓ Cloudflare purge settings saved & cache purged!');
                    setTimeout(() => location.reload(), 400);
                }}
            }} catch (err) {{
                alert('Error saving Cloudflare settings: ' + err);
            }}
        }}
        async function runVacuum() {{
            try {{
                const res = await fetch('/wp-json/daypress/v1/vacuum', {{ method: 'POST' }});
                const data = await res.json();
                alert(data.message || 'VACUUM completed!');
                location.reload();
            }} catch (err) {{
                alert('Error running VACUUM: ' + err);
            }}
        }}
        async function runIntegrityCheck() {{
            try {{
                const res = await fetch('/wp-json/daypress/v1/integrity-check');
                const data = await res.json();
                alert('Integrity Status: ' + (data.integrity || 'ok'));
            }} catch (err) {{
                alert('Error running integrity check: ' + err);
            }}
        }}
        </script>
        "##,
        hit_ratio = cache_stats.hit_rate_pct,
        hits = cache_stats.hits,
        misses = cache_stats.misses,
        entries = cache_stats.cached_entries,
        raw_kb = cache_stats.raw_bytes as f64 / 1024.0,
        gz_kb = cache_stats.compressed_bytes as f64 / 1024.0,
        purges = cache_stats.invalidations,
        last_purged = last_purged,
        cf_zone_esc = cf_zone_esc,
        cf_token_placeholder = cf_token_placeholder,
        kb_size = kb_size,
        journal = stats.journal_mode,
        posts = stats.total_posts,
        pages = stats.total_pages,
        options = stats.total_options,
        db_path = stats.db_path
    );

    render_admin_shell("engine", "Daypress Engine", &body, settings)
}

// 11. Taxonomy View (/d-admin/edit-tags)
pub fn render_tags_view(
    settings: &DaypressSettingsResponse,
    taxonomy: &str,
    terms: &[DaypressCategoryResponse],
) -> String {
    let is_cat = taxonomy == "category";
    let title_label = if is_cat { "Categories" } else { "Tags" };
    let singular = if is_cat { "Category" } else { "Tag" };
    let endpoint = if is_cat {
        "/wp-json/wp/v2/categories"
    } else {
        "/wp-json/wp/v2/tags"
    };

    let mut rows_html = String::new();
    if terms.is_empty() {
        rows_html.push_str(&format!(
            r#"<tr><td colspan="4" style="text-align: center; color: var(--ink-muted); padding: 24px;">No {} found.</td></tr>"#,
            title_label.to_lowercase()
        ));
    } else {
        for t in terms {
            let name_esc = escape_html(&t.name);
            let slug_esc = escape_html(&t.slug);
            let count_label = if t.count == 1 { "1 post" } else { "posts" };
            let delete_action = if is_cat && t.id == 1 {
                r#"<span style="font-size: 11.5px; color: var(--ink-muted);">Default</span>"#
                    .to_string()
            } else {
                format!(
                    r#"<button type="button" class="btn btn-secondary" style="padding: 3px 8px; font-size: 11.5px; color: var(--oxide); border-color: var(--oxide);" onclick="deleteTerm({})">Delete</button>"#,
                    t.id
                )
            };
            rows_html.push_str(&format!(
                r#"<tr id="term-row-{id}">
                    <td><strong style="color: var(--ink);">{name}</strong></td>
                    <td style="font-family: var(--font-mono); color: var(--ink-muted);">{slug}</td>
                    <td><span class="badge badge-published">{count} {count_label}</span></td>
                    <td>{delete_action}</td>
                </tr>"#,
                id = t.id,
                name = name_esc,
                slug = slug_esc,
                count = t.count,
                count_label = count_label,
                delete_action = delete_action
            ));
        }
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">{title_label}</h1>
                <p class="page-subtitle">Organize and structure your publication topics</p>
            </div>
        </div>

        <div style="display: grid; grid-template-columns: 340px 1fr; gap: 24px;">
            <div class="day-card" style="padding: 20px;">
                <h2 style="font-size: 15px; font-weight: 600; margin: 0 0 16px 0;">Add New {singular}</h2>
                <form onsubmit="createTerm(event)">
                    <div class="form-group">
                        <label class="form-label">Name</label>
                        <input type="text" id="termName" class="form-input" required />
                    </div>
                    <div class="form-group">
                        <label class="form-label">Slug</label>
                        <input type="text" id="termSlug" class="form-input" />
                    </div>
                    <div class="form-group">
                        <label class="form-label">Description</label>
                        <textarea id="termDesc" class="form-textarea" rows="3"></textarea>
                    </div>
                    <button type="submit" class="btn btn-primary" style="width: 100%; justify-content: center;">Add New {singular}</button>
                </form>
            </div>

            <div class="day-table-wrapper">
                <table class="day-table">
                    <thead>
                        <tr>
                            <th>Name</th>
                            <th>Slug</th>
                            <th>Count</th>
                            <th style="width: 110px;">Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {rows_html}
                    </tbody>
                </table>
            </div>
        </div>

        <script>
            async function createTerm(e) {{
                e.preventDefault();
                const name = document.getElementById('termName').value.trim();
                const slug = document.getElementById('termSlug').value.trim();
                const description = document.getElementById('termDesc').value.trim();
                if (!name) return;
                try {{
                    const res = await fetch('{endpoint}', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{
                            name,
                            slug: slug || undefined,
                            description: description || undefined,
                            parent: 0
                        }})
                    }});
                    if (res.ok) {{
                        showToast('✓ {singular} created in SQLite!');
                        setTimeout(() => window.location.reload(), 400);
                    }} else {{
                        const err = await res.text();
                        showToast('Failed to create {singular}: ' + err, true);
                    }}
                }} catch (err) {{
                    showToast('Network error: ' + err, true);
                }}
            }}

            async function deleteTerm(id) {{
                if (!confirm('Delete this {singular}?')) return;
                try {{
                    const res = await fetch('{endpoint}/' + id + '?force=true', {{
                        method: 'DELETE'
                    }});
                    if (res.ok) {{
                        const row = document.getElementById('term-row-' + id);
                        if (row) row.remove();
                        showToast('✓ {singular} deleted');
                    }} else {{
                        showToast('Failed to delete {singular}', true);
                    }}
                }} catch (err) {{
                    showToast('Network error: ' + err, true);
                }}
            }}
        </script>
        "##,
        title_label = title_label,
        singular = singular,
        endpoint = endpoint,
        rows_html = rows_html
    );

    render_admin_shell("posts", title_label, &body, settings)
}

#[allow(dead_code)]
pub fn render_categories_view(
    settings: &DaypressSettingsResponse,
    taxonomy: &str,
    terms: &[DaypressCategoryResponse],
) -> String {
    render_tags_view(settings, taxonomy, terms)
}
