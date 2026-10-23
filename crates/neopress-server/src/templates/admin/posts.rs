use super::layout::render_wp_admin_shell;
use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::escape_html;

pub fn render_wp_posts_view(
    posts: &[WpPostResponse],
    settings: &WpSettingsResponse,
    is_page: bool,
) -> String {
    let ptype = if is_page { "page" } else { "post" };
    render_wp_posts_view_for_type(posts, settings, ptype)
}

pub fn render_wp_posts_view_for_type(
    posts: &[WpPostResponse],
    settings: &WpSettingsResponse,
    post_type: &str,
) -> String {
    let is_page = post_type == "page";
    let formatted_cpt = crate::templates::admin::layout::format_cpt_title(post_type);
    let (post_type_label, singular) = match post_type {
        "page" => ("Pages", "Page"),
        "post" => ("Posts", "Post"),
        _ => (formatted_cpt.as_str(), formatted_cpt.as_str()),
    };
    let add_new_url = if post_type == "post" {
        "/n-admin/post-new.php".to_string()
    } else {
        format!("/n-admin/post-new.php?post_type={}", post_type)
    };

    let all_non_trash_count = posts.iter().filter(|p| p.status != "trash").count();
    let pub_count = posts.iter().filter(|p| p.status == "publish").count();
    let draft_count = posts.iter().filter(|p| p.status == "draft").count();
    let trash_count = posts.iter().filter(|p| p.status == "trash").count();

    let nav_slugs: std::collections::HashSet<String> = settings
        .nav_menu_items
        .iter()
        .map(|m| m.url.trim_matches('/').to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    let mut ordered_posts: Vec<&WpPostResponse> = posts.iter().collect();
    ordered_posts.sort_by(|a, b| {
        let a_front = a.id == settings.page_on_front;
        let b_front = b.id == settings.page_on_front;
        let a_pub = a.status == "publish";
        let b_pub = b.status == "publish";
        let a_nav = a_pub && nav_slugs.contains(&a.slug.to_lowercase());
        let b_nav = b_pub && nav_slugs.contains(&b.slug.to_lowercase());
        let a_titled = !a.title.rendered.trim().is_empty();
        let b_titled = !b.title.rendered.trim().is_empty();
        b_front
            .cmp(&a_front)
            .then_with(|| b_nav.cmp(&a_nav))
            .then_with(|| b_pub.cmp(&a_pub))
            .then_with(|| b_titled.cmp(&a_titled))
            .then_with(|| b.date.cmp(&a.date))
    });

    let mut rows_html = String::new();
    for p in ordered_posts {
        let edit_link = format!(
            "/n-admin/post.php?post={}&action=edit&post_type={}",
            p.id,
            escape_html(&p.post_type)
        );
        let view_link = if p.id == settings.page_on_front {
            "/".to_string()
        } else if matches!(p.post_type.as_str(), "post" | "page") {
            format!("/{}/", p.slug)
        } else {
            format!("/{}/{}/", p.post_type, p.slug)
        };
        let status_badge = match p.status.as_str() {
            "draft" => r#"<span class="badge badge-draft">Draft</span>"#,
            "trash" => r#"<span class="badge" style="background:#fef2f2;color:#dc2626;border:1px solid #fecaca;">Trash</span>"#,
            _ => r#"<span class="badge badge-published">Published</span>"#,
        };

        let author_display = if p.author_name.is_empty() || p.author_name == "Neopress Core" || p.author_name == "admin" {
            "Editor".to_string()
        } else {
            escape_html(&p.author_name)
        };
        let categories_display = if !p.category_names.is_empty() {
            escape_html(&p.category_names.join(", "))
        } else if p.template == "page_templates/leeg.php" {
            r#"<span style="font-size:11.5px;color:#475569;font-family:var(--font-mono);">leeg.php (Custom HTML)</span>"#.to_string()
        } else if p.content.rendered.contains("<section") {
            r#"<span style="font-size:11.5px;color:#2563eb;font-weight:600;">ACF + Gutenberg</span>"#.to_string()
        } else if is_page {
            "Default Template".to_string()
        } else {
            "—".to_string()
        };
        let date_short: String = p.date.chars().take(10).collect();

        let mut chips = String::new();
        if p.id == settings.page_on_front {
            chips.push_str(r#"<span style="display:inline-flex;align-items:center;gap:4px;font-size:10.5px;background:#dcfce7;color:#15803d;border:1px solid #86efac;padding:1px 7px;border-radius:4px;margin-left:6px;font-weight:700;">🏠 Front Page</span>"#);
        }
        if nav_slugs.contains(&p.slug.to_lowercase()) && p.status == "publish" {
            chips.push_str(r#"<span style="display:inline-flex;align-items:center;gap:4px;font-size:10.5px;background:#eff6ff;color:#1d4ed8;border:1px solid #bfdbfe;padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:600;">🧭 Main Menu</span>"#);
        }
        if !p.seo_keyword.is_empty() {
            chips.push_str(&format!(
                r#"<span style="display:inline-flex;align-items:center;gap:4px;font-size:10.5px;background:#ecfdf5;color:#059669;border:1px solid #a7f3d0;padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:600;" title="Focus Keyphrase">🎯 {}</span>"#,
                escape_html(&p.seo_keyword)
            ));
        } else if p.seo_noindex {
            chips.push_str(r#"<span style="display:inline-flex;align-items:center;font-size:10.5px;background:#fef2f2;color:#dc2626;border:1px solid #fecaca;padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:600;">noindex</span>"#);
        }

        let display_title = if p.title.rendered.trim().is_empty() {
            format!("(Untitled — #{})", p.id)
        } else {
            p.title.rendered.clone()
        };
        let initial_row_style = if p.status == "trash" {
            "display: none;"
        } else {
            ""
        };

        rows_html.push_str(&format!(
            r##"
            <tr id="post-{id}" data-status="{status}" style="{initial_row_style}">
                <td style="width: 32px;"><input type="checkbox" name="post[]" value="{id}"></td>
                <td class="table-title-cell">
                    <strong><a href="{edit_link}" style="color: #0f172a; text-decoration: none;">{title}</a></strong>{seo_chip}
                    <div class="table-row-slug">Permalink: {view_link}</div>
                    <div class="table-actions">
                        <a href="{edit_link}" style="font-weight: 600;">Edit Blocks &amp; ACF</a>
                        <a href="/n-admin/builder.php?post={id}" style="color: #2563eb; font-weight: 600;">⚡ Edit with NeoStudio</a>
                        <a href="#" onclick="duplicatePostRow({id}); return false;" style="color: #4f46e5; font-weight: 600;">Duplicate</a>
                        <a href="{view_link}" target="_blank">View &nearr;</a>
                        <a href="#" onclick="trashPostRow({id}); return false;" style="color: #d97706;">Trash</a>
                        <a href="#" onclick="deletePost({id}); return false;" class="trash-action">Delete</a>
                    </div>
                </td>
                <td style="width: 130px;"><span style="color: #64748b;">{author}</span></td>
                <td style="width: 160px;">{categories}</td>
                <td style="width: 110px;">{status_badge}</td>
                <td style="width: 130px; color: #64748b; font-size: 12.5px;">{date}</td>
            </tr>
            "##,
            id = p.id,
            status = escape_html(&p.status),
            initial_row_style = initial_row_style,
            edit_link = edit_link,
            view_link = view_link,
            title = display_title,
            seo_chip = chips,
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
                    const statusMatch = activeStatusFilter === 'all' ? (rowStatus !== 'trash') : (rowStatus === activeStatusFilter);
                    const textMatch = !filter || text.includes(filter);
                    r.style.display = (statusMatch && textMatch) ? '' : 'none';
                }});
            }}
        </script>
        "##,
        post_type_label = post_type_label,
        add_new_url = add_new_url,
        singular = singular,
        plural_lower = post_type_label.to_lowercase(),
        count = all_non_trash_count,
        pub_count = pub_count,
        draft_count = draft_count,
        trash_count = trash_count,
        rows_html = rows_html
    );

    let tab = if is_page {
        "pages"
    } else if post_type == "post" {
        "posts"
    } else {
        post_type
    };
    render_wp_admin_shell(tab, post_type_label, &body, settings)
}
