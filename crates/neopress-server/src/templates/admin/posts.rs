use super::layout::render_wp_admin_shell;
use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::escape_html;

pub fn render_wp_posts_view(
    posts: &[WpPostResponse],
    settings: &WpSettingsResponse,
    is_page: bool,
) -> String {
    let post_type_label = if is_page { "Pages" } else { "Posts" };
    let add_new_url = if is_page {
        "/n-admin/post-new.php?post_type=page"
    } else {
        "/n-admin/post-new.php"
    };

    let pub_count = posts.iter().filter(|p| p.status == "publish").count();
    let draft_count = posts.iter().filter(|p| p.status == "draft").count();
    let trash_count = posts.iter().filter(|p| p.status == "trash").count();

    let mut rows_html = String::new();
    for p in posts {
        let edit_link = format!("/n-admin/post.php?post={}&action=edit", p.id);
        let view_link = format!("/{}/", p.slug);
        let status_badge = match p.status.as_str() {
            "draft" => r#"<span class="badge badge-draft">Draft</span>"#,
            "trash" => r#"<span class="badge" style="background:#fef2f2;color:#dc2626;border:1px solid #fecaca;">Trash</span>"#,
            _ => r#"<span class="badge badge-published">Published</span>"#,
        };

        let author_display = if p.author_name.is_empty() {
            "admin".to_string()
        } else {
            escape_html(&p.author_name)
        };
        let categories_display = if p.category_names.is_empty() {
            if is_page {
                "—".to_string()
            } else {
                "Uncategorized".to_string()
            }
        } else {
            escape_html(&p.category_names.join(", "))
        };
        let date_short: String = p.date.chars().take(10).collect();

        let seo_chip = if !p.seo_keyword.is_empty() {
            format!(
                r#"<span style="display:inline-flex;align-items:center;gap:4px;font-size:10.5px;background:#ecfdf5;color:#059669;border:1px solid #a7f3d0;padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:600;" title="Focus Keyphrase">🎯 {}</span>"#,
                escape_html(&p.seo_keyword)
            )
        } else if p.seo_noindex {
            r#"<span style="display:inline-flex;align-items:center;font-size:10.5px;background:#fef2f2;color:#dc2626;border:1px solid #fecaca;padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:600;">noindex</span>"#.to_string()
        } else {
            String::new()
        };

        rows_html.push_str(&format!(
            r##"
            <tr id="post-{id}" data-status="{status}">
                <td style="width: 32px;"><input type="checkbox" name="post[]" value="{id}"></td>
                <td class="table-title-cell">
                    <strong><a href="{edit_link}" style="color: #0f172a; text-decoration: none;">{title}</a></strong>{seo_chip}
                    <div class="table-row-slug">Permalink: /{slug}/</div>
                    <div class="table-actions">
                        <a href="{edit_link}">Edit</a>
                        <a href="#" onclick="duplicatePostRow({id}); return false;" style="color: #4f46e5; font-weight: 600;">Duplicate</a>
                        <a href="{view_link}" target="_blank">View &nearr;</a>
                        <a href="#" onclick="trashPostRow({id}); return false;" style="color: #d97706;">Trash</a>
                        <a href="#" onclick="deletePost({id}); return false;" class="trash-action">Delete</a>
                    </div>
                </td>
                <td style="width: 120px;"><span style="color: #64748b;">{author}</span></td>
                <td style="width: 140px;">{categories}</td>
                <td style="width: 110px;">{status_badge}</td>
                <td style="width: 130px; color: #64748b; font-size: 12.5px;">{date}</td>
            </tr>
            "##,
            id = p.id,
            status = escape_html(&p.status),
            edit_link = edit_link,
            view_link = view_link,
            title = p.title.rendered,
            seo_chip = seo_chip,
            slug = p.slug,
            author = author_display,
            categories = categories_display,
            status_badge = status_badge,
            date = escape_html(&date_short)
        ));
    }

    if posts.is_empty() {
        rows_html = format!(
            r#"<tr><td colspan="6" style="text-align: center; color: #64748b; padding: 32px;">No {} found.</td></tr>"#,
            post_type_label.to_lowercase()
        );
    }

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">{post_type_label}</h1>
                <p class="page-subtitle">Manage, create, and organize your publication articles</p>
            </div>
            <div>
                <a href="{add_new_url}" class="btn btn-primary">+ Add New {singular}</a>
            </div>
        </div>

        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
            <div style="display: flex; gap: 8px;" id="statusFilterTabs">
                <button type="button" class="btn btn-secondary status-tab" data-filter="all" style="background: #0f172a; color: #fff;" onclick="filterByStatus('all', this)">All ({count})</button>
                <button type="button" class="btn btn-secondary status-tab" data-filter="publish" onclick="filterByStatus('publish', this)">Published ({pub_count})</button>
                <button type="button" class="btn btn-secondary status-tab" data-filter="draft" onclick="filterByStatus('draft', this)">Draft ({draft_count})</button>
                <button type="button" class="btn btn-secondary status-tab" data-filter="trash" onclick="filterByStatus('trash', this)">Trash ({trash_count})</button>
            </div>
            <div>
                <input type="text" id="postSearchInput" placeholder="Search {plural_lower}..." class="form-input" style="width: 240px; padding: 6px 12px; font-size: 13px;" oninput="filterRows(this.value)" />
            </div>
        </div>

        <div class="neo-table-wrapper">
            <table class="neo-table" id="postsTable">
                <thead>
                    <tr>
                        <th><input type="checkbox"></th>
                        <th>Title</th>
                        <th>Author</th>
                        <th>Category</th>
                        <th>Status</th>
                        <th>Date</th>
                    </tr>
                </thead>
                <tbody>
                    {rows_html}
                </tbody>
            </table>
        </div>

        <script>
            let activeStatusFilter = 'all';

            function filterByStatus(status, btn) {{
                activeStatusFilter = status;
                document.querySelectorAll('#statusFilterTabs .status-tab').forEach(b => {{
                    b.style.background = '';
                    b.style.color = '';
                }});
                if (btn) {{
                    btn.style.background = '#0f172a';
                    btn.style.color = '#fff';
                }}
                const q = document.getElementById('postSearchInput') ? document.getElementById('postSearchInput').value : '';
                filterRows(q);
            }}

            async function duplicatePostRow(id) {{
                showToast('⚡ Cloning #' + id + ' (AST layout, metadata & terms)...');
                try {{
                    const res = await fetch('/wp-json/neopress/v1/posts/' + id + '/duplicate', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }}
                    }});
                    if (res.ok) {{
                        const cloned = await res.json();
                        showToast('✓ Cloned as Draft #' + cloned.id + ': ' + (cloned.title && cloned.title.rendered ? cloned.title.rendered : 'Copy'));
                        setTimeout(() => window.location.reload(), 550);
                    }} else {{
                        showToast('Error duplicating item', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function trashPostRow(id) {{
                const res = await fetch('/wp-json/wp/v2/posts/' + id, {{
                    method: 'PUT',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ status: 'trash' }})
                }});
                if (res.ok) {{
                    showToast('✓ Moved #' + id + ' to Trash');
                    setTimeout(() => window.location.reload(), 500);
                }} else {{
                    showToast('Error trashing item', true);
                }}
            }}

            async function deletePost(id) {{
                if (!confirm('Permanently delete #' + id + '?')) return;
                const res = await fetch('/wp-json/wp/v2/posts/' + id + '?force=true', {{ method: 'DELETE' }});
                if (res.ok) {{
                    showToast('✓ Permanently deleted');
                    const row = document.getElementById('post-' + id);
                    if (row) row.remove();
                }} else {{
                    showToast('Error deleting post', true);
                }}
            }}

            function filterRows(q) {{
                const filter = (q || '').toLowerCase();
                const rows = document.querySelectorAll('#postsTable tbody tr[data-status]');
                rows.forEach(r => {{
                    const text = r.textContent.toLowerCase();
                    const rowStatus = r.getAttribute('data-status') || 'publish';
                    const statusMatch = activeStatusFilter === 'all' || rowStatus === activeStatusFilter;
                    const textMatch = !filter || text.includes(filter);
                    r.style.display = (statusMatch && textMatch) ? '' : 'none';
                }});
            }}
        </script>
        "##,
        post_type_label = post_type_label,
        add_new_url = add_new_url,
        singular = if is_page { "Page" } else { "Post" },
        plural_lower = post_type_label.to_lowercase(),
        count = posts.len(),
        pub_count = pub_count,
        draft_count = draft_count,
        trash_count = trash_count,
        rows_html = rows_html
    );

    let tab = if is_page { "pages" } else { "posts" };
    render_wp_admin_shell(tab, post_type_label, &body, settings)
}
