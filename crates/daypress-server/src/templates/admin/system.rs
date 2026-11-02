use super::layout::render_admin_shell;
use crate::models::{DaypressCategoryResponse, DaypressSettingsResponse, DbStats};
use crate::templates::utils::escape_html;

// 5. Daypress Engine & Verification Loop View (/d-admin/daypress-engine)
pub fn render_engine_view(settings: &DaypressSettingsResponse, stats: &DbStats) -> String {
    let kb_size = stats.file_size_bytes as f64 / 1024.0;
    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Daypress Rusqlite Engine</h1>
                <p class="page-subtitle">Native in-process database diagnostics and architectural telemetry</p>
            </div>
            <div style="display: flex; gap: 10px;">
                <button type="button" class="btn btn-secondary" onclick="runVacuum()">VACUUM Storage</button>
                <button type="button" class="btn btn-primary" onclick="runIntegrityCheck()">Integrity Check</button>
            </div>
        </div>

        <div class="day-metrics-grid" style="display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 16px; margin-bottom: 24px;">
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
                <div style="font-size: 12px; font-weight: 600; color: var(--ink-muted); text-transform: uppercase;">Page Size / Count</div>
                <div style="font-size: 20px; font-weight: 700; color: var(--ink); margin-top: 4px;">{page_size} B &bull; {page_count}</div>
                <div style="font-size: 12px; color: var(--ink-muted); margin-top: 4px;">B-Tree Allocation</div>
            </div>
            <div class="day-card" style="padding: 20px;">
                <div style="font-size: 12px; font-weight: 600; color: var(--ink-muted); text-transform: uppercase;">System Options</div>
                <div style="font-size: 20px; font-weight: 700; color: var(--ink); margin-top: 4px;">{options}</div>
                <div style="font-size: 12px; color: var(--ink-muted); margin-top: 4px;">Auto-loaded config</div>
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
        kb_size = kb_size,
        journal = stats.journal_mode,
        posts = stats.total_posts,
        pages = stats.total_pages,
        page_size = stats.page_size,
        page_count = stats.page_count,
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
