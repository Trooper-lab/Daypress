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
            r#"<tr><td colspan="4" style="text-align: center; padding: 36px; color: var(--ink-muted);">
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
                    r#"<button onclick="setCommentStatus({id}, '1')" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--moss);">Approve</button>
                       <button onclick="setCommentStatus({id}, 'spam')" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--ink);">Spam</button>
                       <button onclick="setCommentStatus({id}, 'trash')" class="btn btn-danger" style="padding: 2px 8px; font-size: 11px;">Trash</button>"#,
                    id = c.id
                ),
                "1" => format!(
                    r#"<button onclick="setCommentStatus({id}, '0')" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--ink);">Unapprove</button>
                       <button onclick="setCommentStatus({id}, 'spam')" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--ink);">Spam</button>
                       <button onclick="setCommentStatus({id}, 'trash')" class="btn btn-danger" style="padding: 2px 8px; font-size: 11px;">Trash</button>"#,
                    id = c.id
                ),
                "spam" => format!(
                    r#"<button onclick="setCommentStatus({id}, '1')" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--moss);">Not Spam</button>
                       <button onclick="deleteCommentPermanent({id})" class="btn btn-danger" style="padding: 2px 8px; font-size: 11px;">Delete</button>"#,
                    id = c.id
                ),
                "trash" => format!(
                    r#"<button onclick="setCommentStatus({id}, '1')" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px; color: var(--moss);">Restore</button>
                       <button onclick="deleteCommentPermanent({id})" class="btn btn-danger" style="padding: 2px 8px; font-size: 11px;">Delete Permanently</button>"#,
                    id = c.id
                ),
                _ => format!(
                    r#"<button onclick="setCommentStatus({id}, '0')" class="btn btn-secondary" style="padding: 2px 8px; font-size: 11px;">Unapprove</button>"#,
                    id = c.id
                ),
            };

            rows_html.push_str(&format!(
                r##"<tr id="comment-row-{id}">
                    <td style="width: 220px; vertical-align: top;">
                        <div style="font-weight: 600; color: var(--ink);">{author}</div>
                        <div style="font-size: 12px; color: var(--press); margin-top: 2px;">{email}</div>
                        <div style="font-size: 11px; color: var(--ink-muted); font-family: var(--font-mono); margin-top: 2px;">{ip}</div>
                        <div style="margin-top: 6px;">{status_badge}</div>
                    </td>
                    <td style="vertical-align: top;">
                        <div style="font-size: 13.5px; line-height: 1.6; color: var(--ink-muted); white-space: pre-line;">{content}</div>
                        <div style="display: flex; gap: 8px; margin-top: 10px;">
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
                <p class="page-subtitle">Moderate, approve, and manage reader conversations</p>
            </div>
            <div style="display: flex; gap: 8px;">
                <a href="/d-admin/edit-comments" class="btn {all_cls}" style="padding: 6px 12px; font-size: 12px;">All ({all_count})</a>
                <a href="/d-admin/edit-comments?comment_status=pending" class="btn {pend_cls}" style="padding: 6px 12px; font-size: 12px;">Pending ({pend_count})</a>
                <a href="/d-admin/edit-comments?comment_status=approved" class="btn {app_cls}" style="padding: 6px 12px; font-size: 12px;">Approved ({app_count})</a>
                <a href="/d-admin/edit-comments?comment_status=spam" class="btn {spam_cls}" style="padding: 6px 12px; font-size: 12px;">Spam ({spam_count})</a>
                <a href="/d-admin/edit-comments?comment_status=trash" class="btn {trash_cls}" style="padding: 6px 12px; font-size: 12px;">Trash ({trash_count})</a>
            </div>
        </div>

        <div class="day-table-wrapper">
            <table class="day-table">
                <thead>
                    <tr>
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
