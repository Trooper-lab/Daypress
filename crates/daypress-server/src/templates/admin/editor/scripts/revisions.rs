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
                            + '<button type="button" class="btn btn-secondary" style="font-size:11px;padding:3px 8px;color:var(--press);font-weight:600;" onclick="openEditorRevisionDiff(' + r.id + ')">Diff</button>'
                            + '<button type="button" class="btn btn-secondary" style="font-size:11px;padding:3px 8px;" onclick="restorePostRevision(' + r.id + ')">Restore</button>'
                            + '<button type="button" class="btn btn-secondary" style="font-size:11px;padding:3px 7px;color:var(--oxide);" onclick="deletePostRevision(' + r.id + ')" title="Delete revision">&times;</button>'
                            + '</div>'
                            + '</div>';
                    }}).join('');
                }} catch (_) {{}}
            }}

            async function openEditorRevisionDiff(revId) {{
                const pid = activePostId ? parseInt(activePostId, 10) : 0;
                if (!pid || !revId) return;
                window.open('/d-admin/builder?post=' + pid + '&diff_rev=' + revId, '_blank');
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

            async function loadEditorI18nAndEditorialState() {{
                const langSel = document.getElementById('post_lang_code');
                const urlParams = new URLSearchParams(window.location.search);
                const queryLang = (urlParams.get('lang') || '').trim().toLowerCase();
                let configuredLangs = [];
                if (langSel) {{
                    const initialLang = queryLang || langSel.getAttribute('data-initial-lang') || 'en';
                    try {{
                        const res = await fetch('/wp-json/daypress/v1/languages');
                        if (res.ok) {{
                            const data = await res.json();
                            configuredLangs = data.languages || [];
                            window.__DAYPRESS_PREFIX_DEFAULT_LANG__ = !!data.prefix_default_language;
                            const defLangObj = configuredLangs.find(l => l.is_default);
                            window.__DAYPRESS_DEFAULT_LANG__ = defLangObj ? defLangObj.code : 'en';
                            if (configuredLangs.length > 0) {{
                                langSel.innerHTML = configuredLangs.map(l =>
                                    '<option value="' + l.code + '"' + (l.code === initialLang ? ' selected' : '') + '>' +
                                    (l.flag_emoji || '🌐') + ' ' + l.native_name + ' (' + l.code + ')' + (l.is_default ? ' — Default' : '') +
                                    '</option>'
                                ).join('');
                            }}
                            const targetSel = document.getElementById('translateTargetLang');
                            const otherLangs = configuredLangs.filter(l => l.code !== initialLang);
                            if (targetSel && otherLangs.length > 0) {{
                                targetSel.innerHTML = otherLangs.map(l =>
                                    '<option value="' + l.code + '">' + (l.flag_emoji || '🌐') + ' ' + l.native_name + ' (' + l.code + ')</option>'
                                ).join('');
                            }}
                        }}
                    }} catch (_) {{}}
                }}
                try {{
                    const uRes = await fetch('/wp-json/wp/v2/users');
                    if (uRes.ok) {{
                        const users = await uRes.json();
                        window.__DAYPRESS_EDITOR_USERS__ = Array.isArray(users) ? users : [];
                        const addSel = document.getElementById('coauthorAddSelect');
                        if (addSel && window.__DAYPRESS_EDITOR_USERS__.length > 0) {{
                            addSel.innerHTML = '<option value="">+ Select user to add...</option>' +
                                window.__DAYPRESS_EDITOR_USERS__.map(u =>
                                    '<option value="' + u.id + '">' + (u.name || u.slug || ('User #' + u.id)) + ' (#' + u.id + ')</option>'
                                ).join('');
                        }}
                        renderCoauthorPillsFromInput();
                    }}
                }} catch (_) {{}}
                const pid = activePostId ? parseInt(activePostId, 10) : 0;
                if (!pid) return;
                try {{
                    const postRes = await fetch('/wp-json/wp/v2/posts/' + pid);
                    if (postRes.ok) {{
                        const postData = await postRes.json();
                        const trBox = document.getElementById('editorTranslationsLinksList');
                        if (trBox && postData && postData.translations) {{
                            const entries = Object.entries(postData.translations).filter(([code, tid]) => tid !== pid);
                            if (entries.length > 0) {{
                                trBox.innerHTML = entries.map(([code, tid]) => {{
                                    const lObj = configuredLangs.find(l => l.code === code);
                                    const label = lObj ? ((lObj.flag_emoji || '🌐') + ' ' + lObj.native_name + ' (' + code + ')') : code.toUpperCase();
                                    return '<div style="display:flex;justify-content:space-between;align-items:center;padding:4px 8px;border:1px solid var(--rule);border-radius:5px;background:var(--paper-raised);">' +
                                        '<span>' + label + '</span>' +
                                        '<a href="/d-admin/post?post=' + tid + '&action=edit" style="color:var(--press);font-weight:600;text-decoration:none;">Edit #' + tid + ' &rarr;</a>' +
                                        '</div>';
                                }}).join('');
                            }}
                        }}
                    }}
                }} catch (_) {{}}
                try {{
                    const pRes = await fetch('/wp-json/daypress/v1/posts/' + pid + '/preview-links');
                    if (pRes.ok) {{
                        const links = await pRes.json();
                        renderEditorPreviewLinks((links && links.tokens) ? links.tokens : (Array.isArray(links) ? links : []));
                    }}
                }} catch (_) {{}}
            }}

            async function linkEditorExistingTranslation() {{
                const pid = activePostId ? parseInt(activePostId, 10) : 0;
                if (!pid) {{ showToast('Save this {type_singular} first before linking translations.', true); return; }}
                const inp = document.getElementById('linkExistingTranslationPostId');
                const target_id = inp ? (parseInt(inp.value, 10) || 0) : 0;
                if (!target_id) {{ showToast('Enter a target Post ID to link', true); return; }}
                const langSel = document.getElementById('post_lang_code');
                const source_lang = langSel ? (langSel.value || 'en') : 'en';
                const targetSel = document.getElementById('translateTargetLang');
                const target_lang = targetSel ? (targetSel.value || 'de') : 'de';
                const res = await fetch('/wp-json/daypress/v1/translations/link', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{ object_type: 'post', source_id: pid, source_lang, target_id, target_lang }})
                }});
                if (res.ok) {{
                    showToast('✓ Linked #' + pid + ' (' + source_lang + ') ↔ #' + target_id + ' (' + target_lang + ')');
                    loadEditorI18nAndEditorialState();
                }} else {{
                    showToast('Failed to link translation', true);
                }}
            }}

            function getCoauthorIdsArray() {{
                const el = document.getElementById('post_coauthor_ids');
                if (!el) return [1];
                const ids = (el.value || '').split(',').map(s => parseInt(s.trim(), 10)).filter(n => !isNaN(n) && n > 0);
                return Array.from(new Set(ids));
            }}

            function setCoauthorIdsArray(ids) {{
                const el = document.getElementById('post_coauthor_ids');
                if (!el) return;
                const clean = ids.filter(n => !isNaN(n) && n > 0);
                el.value = (clean.length > 0 ? clean : [1]).join(', ');
                renderCoauthorPillsFromInput();
                markDirty();
            }}

            function renderCoauthorPillsFromInput() {{
                const box = document.getElementById('coauthorsPillBox');
                if (!box) return;
                const ids = getCoauthorIdsArray();
                const users = window.__DAYPRESS_EDITOR_USERS__ || [];
                if (ids.length === 0) return;
                box.innerHTML = ids.map((uid, idx) => {{
                    const u = users.find(x => x.id === uid);
                    const display = u ? (u.name || u.slug || ('User #' + uid)) : ('Author #' + uid);
                    const initial = String(display || 'U').trim().charAt(0).toUpperCase();
                    const roleBadge = idx === 0
                        ? '<span style="font-size:10px;font-family:var(--font-mono);color:var(--moss);background:var(--moss-wash);border:1px solid var(--moss);padding:1px 5px;border-radius:4px;">Primary</span>'
                        : '<span style="font-size:10px;font-family:var(--font-mono);color:var(--ink-muted);">Co-Author</span>';
                    const upBtn = idx > 0
                        ? '<button type="button" class="btn btn-secondary" style="font-size:10px;padding:1px 5px;" onclick="moveCoauthorPill(' + uid + ', -1)" title="Move up">&uarr;</button>'
                        : '';
                    const removeBtn = ids.length > 1
                        ? '<button type="button" class="btn btn-secondary" style="font-size:10px;padding:1px 5px;color:var(--oxide);" onclick="removeCoauthorPill(' + uid + ')" title="Remove">&times;</button>'
                        : '';
                    return '<div style="display:flex;align-items:center;gap:6px;font-size:12px;padding:4px 8px;border:1px solid var(--rule);border-radius:5px;background:var(--paper-raised);">' +
                        '<div class="user-avatar" style="width:20px;height:20px;font-size:10px;">' + initial + '</div>' +
                        '<span style="flex:1;font-weight:600;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;">' + display + ' <code style="font-size:10.5px;color:var(--ink-muted);">#' + uid + '</code></span>' +
                        roleBadge + upBtn + removeBtn +
                        '</div>';
                }}).join('');
            }}

            function addCoauthorFromSelect() {{
                const sel = document.getElementById('coauthorAddSelect');
                if (!sel || !sel.value) return;
                const uid = parseInt(sel.value, 10);
                if (!uid) return;
                const ids = getCoauthorIdsArray();
                if (!ids.includes(uid)) {{
                    ids.push(uid);
                    setCoauthorIdsArray(ids);
                }}
                sel.value = '';
            }}

            function removeCoauthorPill(uid) {{
                const ids = getCoauthorIdsArray().filter(id => id !== uid);
                setCoauthorIdsArray(ids.length > 0 ? ids : [1]);
            }}

            function moveCoauthorPill(uid, dir) {{
                const ids = getCoauthorIdsArray();
                const idx = ids.indexOf(uid);
                if (idx < 0) return;
                const targetIdx = idx + dir;
                if (targetIdx < 0 || targetIdx >= ids.length) return;
                const tmp = ids[idx];
                ids[idx] = ids[targetIdx];
                ids[targetIdx] = tmp;
                setCoauthorIdsArray(ids);
            }}

            function renderEditorPreviewLinks(links) {{
                const box = document.getElementById('editorPreviewLinksList');
                if (!box) return;
                const active = (links || []).filter(l => !l.revoked_at);
                if (active.length === 0) {{
                    box.innerHTML = '<div style="color:var(--ink-muted);">No active preview tokens.</div>';
                    return;
                }}
                box.innerHTML = active.map(l =>
                    '<div style="display:flex;justify-content:space-between;align-items:center;padding:5px 8px;border:1px solid var(--rule);border-radius:5px;background:var(--paper-raised);">' +
                    '<span style="font-family:var(--font-mono);font-size:11px;">Token #' + l.id + ' &middot; exp ' + (l.expires_at || '').slice(0, 16) + '</span>' +
                    '<button type="button" class="btn btn-secondary" style="font-size:10.5px;padding:2px 6px;color:var(--oxide);" onclick="revokeSecretPreviewLink(' + l.id + ')">Revoke</button>' +
                    '</div>'
                ).join('');
            }}

            async function createPostTranslationDraft() {{
                const pid = activePostId ? parseInt(activePostId, 10) : 0;
                if (!pid) {{ showToast('Save this {type_singular} first before creating a translation draft.', true); return; }}
                const targetSel = document.getElementById('translateTargetLang');
                const lang_code = targetSel ? targetSel.value : 'de';
                showToast('Creating ' + lang_code.toUpperCase() + ' translation draft...');
                try {{
                    const res = await fetch('/wp-json/daypress/v1/posts/' + pid + '/translate', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ lang_code }})
                    }});
                    if (res.ok) {{
                        const cloned = await res.json();
                        showToast('✓ Created ' + lang_code.toUpperCase() + ' translation draft #' + cloned.id + '!');
                        setTimeout(() => {{ window.location.href = '/d-admin/post?post=' + cloned.id + '&action=edit'; }}, 450);
                    }} else {{
                        showToast('Failed to create translation draft', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function generateSecretPreviewLink() {{
                const pid = activePostId ? parseInt(activePostId, 10) : 0;
                if (!pid) {{ showToast('Save this {type_singular} first before generating a preview link.', true); return; }}
                const ttlEl = document.getElementById('previewTtlHours');
                const ttl_hours = ttlEl ? (parseInt(ttlEl.value, 10) || 72) : 72;
                try {{
                    const res = await fetch('/wp-json/daypress/v1/posts/' + pid + '/preview-links', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ ttl_hours, label: 'Editor Share Link' }})
                    }});
                    if (res.ok) {{
                        const data = await res.json();
                        const fullUrl = window.location.origin + data.preview_url;
                        try {{ await navigator.clipboard.writeText(fullUrl); }} catch (_) {{}}
                        showToast('✓ Secret Preview Link copied: ' + fullUrl);
                        loadEditorI18nAndEditorialState();
                    }} else {{
                        showToast('Failed to generate preview link', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function revokeSecretPreviewLink(tokenId) {{
                const pid = activePostId ? parseInt(activePostId, 10) : 0;
                if (!pid || !tokenId) return;
                try {{
                    const res = await fetch('/wp-json/daypress/v1/posts/' + pid + '/preview-links/' + tokenId, {{ method: 'DELETE' }});
                    if (res.ok) {{
                        showToast('✓ Preview link revoked');
                        loadEditorI18nAndEditorialState();
                    }}
                }} catch (_) {{}}
            }}

            async function scheduleLiveRevisionUpdate() {{
                const pid = activePostId ? parseInt(activePostId, 10) : 0;
                if (!pid) {{ showToast('Save & publish this {type_singular} first.', true); return; }}
                const dtEl = document.getElementById('scheduleRevisionAtInput');
                const scheduled_for = dtEl ? (dtEl.value || '').trim() : '';
                if (!scheduled_for) {{ showToast('Select a future UTC date/time first.', true); return; }}
                if (currentMode === 'visual') syncVisualToTextarea();
                try {{
                    const res = await fetch('/wp-json/daypress/v1/posts/' + pid + '/schedule-revision', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{
                            scheduled_for: scheduled_for.replace('T', ' ') + ':00',
                            title: titleInput.value.trim(),
                            content: contentArea.value,
                            excerpt: document.getElementById('excerpt') ? document.getElementById('excerpt').value : undefined
                        }})
                    }});
                    if (res.ok) {{
                        const srev = await res.json();
                        const badge = document.getElementById('editorScheduledRevisionBadge');
                        if (badge) {{
                            badge.style.display = 'block';
                            badge.innerHTML = '<strong>&#9201; Scheduled Revision #' + srev.id + '</strong> goes live at <code>' + srev.scheduled_for + ' UTC</code>';
                        }}
                        showToast('✓ Scheduled update #' + srev.id + ' queued without unpublishing live page!');
                    }} else {{
                        showToast('Failed to schedule update', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            document.addEventListener('DOMContentLoaded', () => {{
                loadEditorRevisions();
                loadEditorI18nAndEditorialState();
            }});
"#,
        type_singular = type_singular,
    )
}
