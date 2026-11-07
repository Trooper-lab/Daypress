use super::layout::render_admin_shell;
use crate::models::{CommentCounts, DaypressCommentAdminItem, DaypressSettingsResponse};
use crate::templates::utils::escape_html;

pub fn render_comments_view(
    settings: &DaypressSettingsResponse,
    comments: &[DaypressCommentAdminItem],
    counts: &CommentCounts,
    current_status: Option<&str>,
) -> String {
    let active_filter = current_status.unwrap_or("all");

    let mut rows_html = String::new();
    if comments.is_empty() {
        rows_html.push_str(
            r#"<tr><td colspan="5" style="text-align: center; padding: 36px; color: var(--ink-muted);">
                <div style="font-size: 15px; font-weight: 500;">No comments found</div>
                <div style="font-size: 12.5px; margin-top: 4px;">Comments submitted on posts will appear here for moderation.</div>
            </td></tr>"#,
        );
    } else {
        for c in comments {
            let author_display = if c.author.is_empty() {
                "Anonymous"
            } else {
                &c.author
            };

            let status_badge = match c.status.as_str() {
                "1" => {
                    r#"<span class="badge badge-published" style="font-size: 11px;">Approved</span>"#
                }
                "0" => r#"<span class="badge badge-draft" style="font-size: 11px;">Pending</span>"#,
                "spam" => {
                    r#"<span class="badge" style="background: var(--oxide-wash); color: var(--oxide); border: 1px solid var(--oxide); font-size: 11px;">Spam</span>"#
                }
                "trash" => {
                    r#"<span class="badge" style="background: var(--paper-sunk); color: var(--ink-muted); font-size: 11px;">Trash</span>"#
                }
                _ => r#"<span class="badge badge-published">Approved</span>"#,
            };

            let action_buttons = match c.status.as_str() {
                "0" => format!(
                    r#"<button type="button" onclick="setCommentStatus({id}, '1')" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--moss);">Approve</button>
                       <button type="button" onclick="openCommentReply({id}, {post_id})" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--press);">Reply</button>
                       <button type="button" onclick="openCommentQuickEdit({id})" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px;">Quick Edit</button>
                       <button type="button" onclick="setCommentStatus({id}, 'spam')" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--ink);">Spam</button>
                       <button type="button" onclick="setCommentStatus({id}, 'trash')" class="btn btn-danger" style="padding: 2px 8px; font-size: 11px;">Trash</button>"#,
                    id = c.id,
                    post_id = c.post_id
                ),
                "1" => format!(
                    r#"<button type="button" onclick="setCommentStatus({id}, '0')" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--ink);">Unapprove</button>
                       <button type="button" onclick="openCommentReply({id}, {post_id})" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--press);">Reply</button>
                       <button type="button" onclick="openCommentQuickEdit({id})" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px;">Quick Edit</button>
                       <button type="button" onclick="setCommentStatus({id}, 'spam')" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--ink);">Spam</button>
                       <button type="button" onclick="setCommentStatus({id}, 'trash')" class="btn btn-danger" style="padding: 2px 8px; font-size: 11px;">Trash</button>"#,
                    id = c.id,
                    post_id = c.post_id
                ),
                "spam" => format!(
                    r#"<button type="button" onclick="setCommentStatus({id}, '1')" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--moss);">Not Spam</button>
                       <button type="button" onclick="deleteCommentPermanent({id})" class="btn btn-danger" style="padding: 2px 8px; font-size: 11px;">Delete</button>"#,
                    id = c.id
                ),
                "trash" => format!(
                    r#"<button type="button" onclick="setCommentStatus({id}, '1')" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--moss);">Restore</button>
                       <button type="button" onclick="deleteCommentPermanent({id})" class="btn btn-danger" style="padding: 2px 8px; font-size: 11px;">Delete Permanently</button>"#,
                    id = c.id
                ),
                _ => format!(
                    r#"<button type="button" onclick="setCommentStatus({id}, '0')" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px;">Unapprove</button>"#,
                    id = c.id
                ),
            };

            rows_html.push_str(&format!(
                r##"<tr id="comment-row-{id}" data-comment-id="{id}" data-author="{author}" data-email="{email}" data-url="{author_url}" data-content="{content}">
                    <td style="width: 32px; vertical-align: top;"><input type="checkbox" class="comment-select-cb" value="{id}" /></td>
                    <td style="width: 220px; vertical-align: top;">
                        <div style="font-weight: 600; color: var(--ink);">{author}</div>
                        <div style="font-size: 12px; color: var(--press); margin-top: 2px;">{email}</div>
                        <div style="font-size: 11px; color: var(--ink-muted); font-family: var(--font-mono); margin-top: 2px;">{ip}</div>
                        <div style="margin-top: 6px;">{status_badge}</div>
                    </td>
                    <td style="vertical-align: top;">
                        <div style="font-size: 13.5px; line-height: 1.6; color: var(--ink-muted); white-space: pre-line;">{content}</div>
                        <div style="display: flex; gap: 8px; margin-top: 10px; flex-wrap: wrap;">
                            {action_buttons}
                        </div>
                    </td>
                    <td style="width: 200px; vertical-align: top;">
                        <a href="/d-admin/post?post={post_id}&action=edit" style="font-weight: 500; text-decoration: none; color: var(--press); font-size: 13px;">{post_title}</a>
                        <div style="margin-top: 4px;">
                            <a href="/?p={post_id}" target="_blank" style="font-size: 11.5px; color: var(--ink-muted); text-decoration: none;">View Post &nearr;</a>
                        </div>
                    </td>
                    <td style="width: 140px; font-size: 12px; color: var(--ink-muted); vertical-align: top;">
                        <div>{date}</div>
                    </td>
                </tr>"##,
                id = c.id,
                author = escape_html(author_display),
                email = escape_html(&c.author_email),
                author_url = escape_html(&c.author_url),
                ip = escape_html(&c.author_ip),
                status_badge = status_badge,
                content = escape_html(&c.content),
                action_buttons = action_buttons,
                post_id = c.post_id,
                post_title = escape_html(&c.post_title),
                date = c.date.chars().take(16).collect::<String>()
            ));
        }
    }

    let filter_active = |f: &str| {
        if active_filter == f {
            "btn-primary"
        } else {
            "btn-secondary"
        }
    };

    let body = format!(
        r##"
        <div class="page-header">
            <div>
                <h1 class="page-title">Comments</h1>
                <p class="page-subtitle">Moderate, approve, reply inline, and manage reader conversations</p>
            </div>
            <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                <a href="/d-admin/edit-comments" class="btn {all_cls}" style="padding: 6px 12px; font-size: 12px;">All ({all_count})</a>
                <a href="/d-admin/edit-comments?comment_status=pending" class="btn {pend_cls}" style="padding: 6px 12px; font-size: 12px;">Pending ({pend_count})</a>
                <a href="/d-admin/edit-comments?comment_status=approved" class="btn {app_cls}" style="padding: 6px 12px; font-size: 12px;">Approved ({app_count})</a>
                <a href="/d-admin/edit-comments?comment_status=spam" class="btn {spam_cls}" style="padding: 6px 12px; font-size: 12px;">Spam ({spam_count})</a>
                <a href="/d-admin/edit-comments?comment_status=trash" class="btn {trash_cls}" style="padding: 6px 12px; font-size: 12px;">Trash ({trash_count})</a>
            </div>
        </div>

        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; flex-wrap: wrap; gap: 10px;">
            <div style="display: flex; gap: 8px; align-items: center; flex-wrap: wrap;">
                <select id="bulkCommentActionSelect" class="form-input" style="width: auto; padding: 6px 10px; font-size: 13px;">
                    <option value="">Bulk actions</option>
                    <option value="approve">Approve</option>
                    <option value="unapprove">Unapprove</option>
                    <option value="spam">Mark as Spam</option>
                    <option value="trash">Move to Trash</option>
                    <option value="delete">Delete Permanently</option>
                </select>
                <button type="button" class="btn btn-secondary" style="padding: 6px 12px; font-size: 13px;" onclick="applyBulkCommentActions()">Apply</button>
                <button type="button" class="btn btn-secondary" style="padding: 6px 12px; font-size: 12px; color: var(--oxide);" onclick="purgeCommentsByStatus('empty_spam')">Empty Spam</button>
                <button type="button" class="btn btn-secondary" style="padding: 6px 12px; font-size: 12px; color: var(--oxide);" onclick="purgeCommentsByStatus('empty_trash')">Empty Trash</button>
            </div>
            <input type="text" id="commentSearchInput" class="form-input" placeholder="Search comments..." style="width: 220px; padding: 6px 12px; font-size: 13px;" oninput="filterCommentRows(this.value)" />
        </div>

        <div class="day-table-wrapper">
            <table class="day-table" id="commentsTable">
                <thead>
                    <tr>
                        <th style="width: 32px;"><input type="checkbox" id="selectAllComments" onchange="toggleSelectAllComments(this)" /></th>
                        <th style="width: 220px;">Author</th>
                        <th>Comment</th>
                        <th style="width: 200px;">In Response To</th>
                        <th style="width: 140px;">Submitted On</th>
                    </tr>
                </thead>
                <tbody>
                    {rows_html}
                </tbody>
            </table>
        </div>

        <script>
            function toggleSelectAllComments(master) {{
                document.querySelectorAll('#commentsTable .comment-select-cb').forEach(cb => {{
                    const r = cb.closest('tr');
                    if (r && r.style.display !== 'none') cb.checked = master.checked;
                }});
            }}

            function filterCommentRows(q) {{
                const filter = (q || '').toLowerCase();
                document.querySelectorAll('#commentsTable tbody tr[data-comment-id]').forEach(r => {{
                    r.style.display = (!filter || r.textContent.toLowerCase().includes(filter)) ? '' : 'none';
                }});
            }}

            async function applyBulkCommentActions() {{
                const action = document.getElementById('bulkCommentActionSelect')?.value || '';
                if (!action) {{ showToast('Select a bulk action first', true); return; }}
                const ids = Array.from(document.querySelectorAll('#commentsTable .comment-select-cb:checked')).map(c => parseInt(c.value, 10)).filter(Boolean);
                if (ids.length === 0) {{ showToast('Select at least one comment', true); return; }}
                try {{
                    const res = await fetch('/wp-json/daypress/v1/comments/bulk', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ comment_ids: ids, action }})
                    }});
                    if (res.ok) {{
                        showToast('✓ Applied ' + action + ' to ' + ids.length + ' comment(s)');
                        setTimeout(() => location.reload(), 450);
                    }} else showToast('Bulk moderation failed', true);
                }} catch (e) {{ showToast('Network error: ' + e, true); }}
            }}

            async function purgeCommentsByStatus(action) {{
                if (!confirm('Permanently delete all comments in this bucket?')) return;
                try {{
                    const res = await fetch('/wp-json/daypress/v1/comments/bulk', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ comment_ids: [], action }})
                    }});
                    if (res.ok) {{
                        showToast('✓ Emptied comments');
                        setTimeout(() => location.reload(), 450);
                    }}
                }} catch (e) {{ showToast('Network error: ' + e, true); }}
            }}

            function openCommentReply(id, postId) {{
                const existing = document.getElementById('comment-reply-' + id);
                if (existing) {{ existing.remove(); return; }}
                document.querySelectorAll('.dp-comment-inline-row').forEach(el => el.remove());
                const row = document.getElementById('comment-row-' + id);
                if (!row) return;
                const tr = document.createElement('tr');
                tr.id = 'comment-reply-' + id;
                tr.className = 'dp-comment-inline-row';
                tr.style.background = 'var(--paper-raised)';
                tr.innerHTML = `<td colspan="5" style="padding: 14px 18px; border-top: 2px solid var(--press); border-bottom: 2px solid var(--press);">
                    <div style="font-size: 12.5px; font-weight: 700; color: var(--press); margin-bottom: 8px;">↩ Reply to Comment #${{id}}</div>
                    <textarea id="reply-content-${{id}}" class="form-textarea" rows="3" placeholder="Write your official reply..." style="margin-bottom: 10px;"></textarea>
                    <div style="display: flex; justify-content: flex-end; gap: 8px;">
                        <button type="button" class="btn btn-secondary" onclick="document.getElementById('comment-reply-${{id}}')?.remove()">Cancel</button>
                        <button type="button" class="btn btn-primary" onclick="submitCommentReply(${{id}}, ${{postId}})">Approve &amp; Reply</button>
                    </div>
                </td>`;
                row.insertAdjacentElement('afterend', tr);
                document.getElementById('reply-content-' + id)?.focus();
            }}

            async function submitCommentReply(id, postId) {{
                const content = document.getElementById('reply-content-' + id)?.value.trim() || '';
                if (!content) {{ showToast('Please enter a reply', true); return; }}
                try {{
                    await fetch('/wp-json/wp/v2/comments/' + id, {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ status: '1' }})
                    }});
                    const res = await fetch('/wp-json/wp/v2/comments', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ post: postId, parent: id, author_name: 'Admin', content, status: 'approved' }})
                    }});
                    if (res.ok) {{
                        showToast('✓ Reply published!');
                        setTimeout(() => location.reload(), 450);
                    }} else showToast('Failed to post reply', true);
                }} catch (e) {{ showToast('Network error: ' + e, true); }}
            }}

            function openCommentQuickEdit(id) {{
                const existing = document.getElementById('comment-qe-' + id);
                if (existing) {{ existing.remove(); return; }}
                document.querySelectorAll('.dp-comment-inline-row').forEach(el => el.remove());
                const row = document.getElementById('comment-row-' + id);
                if (!row) return;
                const author = row.getAttribute('data-author') || '';
                const email = row.getAttribute('data-email') || '';
                const url = row.getAttribute('data-url') || '';
                const content = row.getAttribute('data-content') || '';
                const tr = document.createElement('tr');
                tr.id = 'comment-qe-' + id;
                tr.className = 'dp-comment-inline-row';
                tr.style.background = 'var(--paper-raised)';
                tr.innerHTML = `<td colspan="5" style="padding: 14px 18px; border-top: 2px solid var(--press); border-bottom: 2px solid var(--press);">
                    <div style="font-size: 12.5px; font-weight: 700; color: var(--press); margin-bottom: 10px;">⚡ Quick Edit Comment #${{id}}</div>
                    <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 10px; margin-bottom: 10px;">
                        <div><label class="form-label" style="font-size:11px;">Author Name</label><input type="text" id="cqe-author-${{id}}" class="form-input" value="${{author.replace(/"/g, '&quot;')}}" /></div>
                        <div><label class="form-label" style="font-size:11px;">Author Email</label><input type="text" id="cqe-email-${{id}}" class="form-input" value="${{email.replace(/"/g, '&quot;')}}" /></div>
                        <div><label class="form-label" style="font-size:11px;">Author URL</label><input type="text" id="cqe-url-${{id}}" class="form-input" value="${{url.replace(/"/g, '&quot;')}}" /></div>
                    </div>
                    <textarea id="cqe-content-${{id}}" class="form-textarea" rows="3" style="margin-bottom: 10px;">${{content}}</textarea>
                    <div style="display: flex; justify-content: flex-end; gap: 8px;">
                        <button type="button" class="btn btn-secondary" onclick="document.getElementById('comment-qe-${{id}}')?.remove()">Cancel</button>
                        <button type="button" class="btn btn-primary" onclick="saveCommentQuickEdit(${{id}})">Update Comment</button>
                    </div>
                </td>`;
                row.insertAdjacentElement('afterend', tr);
            }}

            async function saveCommentQuickEdit(id) {{
                const author_name = document.getElementById('cqe-author-' + id)?.value.trim() || 'Anonymous';
                const author_email = document.getElementById('cqe-email-' + id)?.value.trim() || '';
                const author_url = document.getElementById('cqe-url-' + id)?.value.trim() || '';
                const content = document.getElementById('cqe-content-' + id)?.value || '';
                try {{
                    const res = await fetch('/wp-json/wp/v2/comments/' + id, {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ author_name, author_email, author_url, content }})
                    }});
                    if (res.ok) {{
                        showToast('✓ Comment #' + id + ' updated');
                        setTimeout(() => location.reload(), 450);
                    }} else showToast('Failed to update comment', true);
                }} catch (e) {{ showToast('Network error: ' + e, true); }}
            }}

            async function setCommentStatus(id, status) {{
                try {{
                    const res = await fetch('/wp-json/wp/v2/comments/' + id, {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ status }})
                    }});
                    if (res.ok) {{
                        showToast('✓ Comment status updated to ' + status);
                        setTimeout(() => location.reload(), 500);
                    }} else {{
                        showToast('Failed to update comment status', true);
                    }}
                }} catch (e) {{
                    showToast('Network error updating comment', true);
                }}
            }}

            async function deleteCommentPermanent(id) {{
                if (!confirm('Are you sure you want to permanently delete this comment?')) return;
                try {{
                    const res = await fetch('/wp-json/wp/v2/comments/' + id + '?force=true', {{
                        method: 'DELETE'
                    }});
                    if (res.ok) {{
                        showToast('✓ Comment permanently deleted');
                        const row = document.getElementById('comment-row-' + id);
                        if (row) row.remove();
                    }} else {{
                        showToast('Failed to delete comment', true);
                    }}
                }} catch (e) {{
                    showToast('Network error deleting comment', true);
                }}
            }}
        </script>
        "##,
        all_cls = filter_active("all"),
        pend_cls = filter_active("pending"),
        app_cls = filter_active("approved"),
        spam_cls = filter_active("spam"),
        trash_cls = filter_active("trash"),
        all_count = counts.all,
        pend_count = counts.pending,
        app_count = counts.approved,
        spam_count = counts.spam,
        trash_count = counts.trash,
        rows_html = rows_html
    );

    render_admin_shell("comments", "Comments", &body, settings)
}
