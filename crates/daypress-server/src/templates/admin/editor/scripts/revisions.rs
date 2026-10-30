//! Editor script for Post & Page Revisions (`scripts/revisions.rs`).
//!
//! Fetches revision snapshots from `/wp-json/wp/v2/posts/{id}/revisions` and provides
//! 1-click Restore and Delete handlers.

pub fn render_revisions_script(type_singular: &str) -> String {
    format!(
        r#"
            async function loadEditorRevisions() {{
                const pid = activePostId ? parseInt(activePostId, 10) : 0;
                const card = document.getElementById('editorRevisionsCard');
                const listEl = document.getElementById('editorRevisionsList');
                const countEl = document.getElementById('editorRevisionsCount');
                if (!pid || !card || !listEl || !countEl) return;
                try {{
                    const res = await fetch('/wp-json/wp/v2/posts/' + pid + '/revisions');
                    if (!res.ok) return;
                    const revs = await res.json();
                    card.style.display = 'block';
                    countEl.textContent = String(revs.length || 0);
                    if (!revs || revs.length === 0) {{
                        listEl.innerHTML = '<div style="font-size: 12px; color: var(--ink-muted);">No revisions yet. Updating this ' + '{type_singular}' + ' automatically saves a revision snapshot.</div>';
                        return;
                    }}
                    listEl.innerHTML = revs.map(r => {{
                        const titleText = (r.title && r.title.rendered) ? r.title.rendered : ('Revision #' + r.id);
                        const dateText = (r.date || '').replace('T', ' ');
                        return '<div class="revision-item" style="display:flex;justify-content:space-between;align-items:center;gap:8px;padding:8px;border:1px solid var(--rule);border-radius:6px;background:var(--paper-raised);">'
                            + '<div style="min-width:0;flex:1;">'
                            + '<div style="font-size:12px;font-weight:600;color:var(--ink);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">#' + r.id + ' &middot; ' + titleText + '</div>'
                            + '<div style="font-size:11px;color:var(--ink-muted);font-family:var(--font-mono);">' + dateText + '</div>'
                            + '</div>'
                            + '<div style="display:flex;gap:4px;flex-shrink:0;">'
                            + '<button type="button" class="btn btn-secondary" style="font-size:11px;padding:3px 8px;" onclick="restorePostRevision(' + r.id + ')">Restore</button>'
                            + '<button type="button" class="btn btn-secondary" style="font-size:11px;padding:3px 7px;color:var(--oxide);" onclick="deletePostRevision(' + r.id + ')" title="Delete revision">&times;</button>'
                            + '</div>'
                            + '</div>';
                    }}).join('');
                }} catch (_) {{}}
            }}

            async function restorePostRevision(revId) {{
                const pid = activePostId ? parseInt(activePostId, 10) : 0;
                if (!pid || !revId) return;
                if (!confirm('Restore revision #' + revId + '? The current state will also be preserved as a new revision snapshot.')) return;
                try {{
                    const res = await fetch('/wp-json/wp/v2/posts/' + pid + '/revisions/' + revId, {{ method: 'POST' }});
                    if (res.ok) {{
                        showToast('✓ Restored revision #' + revId + '! Reloading...');
                        isDirty = false;
                        setTimeout(() => window.location.reload(), 400);
                    }} else {{
                        showToast('Failed to restore revision', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function deletePostRevision(revId) {{
                const pid = activePostId ? parseInt(activePostId, 10) : 0;
                if (!pid || !revId) return;
                try {{
                    const res = await fetch('/wp-json/wp/v2/posts/' + pid + '/revisions/' + revId, {{ method: 'DELETE' }});
                    if (res.ok) {{
                        showToast('✓ Revision #' + revId + ' deleted');
                        loadEditorRevisions();
                    }} else {{
                        showToast('Failed to delete revision', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            document.addEventListener('DOMContentLoaded', () => {{
                loadEditorRevisions();
            }});
"#,
        type_singular = type_singular,
    )
}
