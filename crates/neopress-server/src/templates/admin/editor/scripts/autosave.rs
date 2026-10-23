//! Autosave and local draft recovery engine (`scripts/autosave.rs`).

pub fn render_autosave_script(post_type: &str) -> String {
    format!(
        r#"
            function markDirty() {{
                isDirty = true;
                updateAutosaveBadge('editing');
                saveToLocalStorage();
            }}

            function updateAutosaveBadge(state) {{
                const badge = document.getElementById('autosaveBadge');
                const dot = document.getElementById('autosaveDot');
                const label = document.getElementById('autosaveLabel');
                if (!badge || !dot || !label) return;

                if (state === 'editing') {{
                    badge.style.background = '#fef3c7';
                    badge.style.borderColor = '#fde68a';
                    badge.style.color = '#b45309';
                    dot.style.background = '#f59e0b';
                    label.textContent = 'Unsaved changes';
                }} else if (state === 'saving') {{
                    badge.style.background = '#eff6ff';
                    badge.style.borderColor = '#bfdbfe';
                    badge.style.color = '#1d4ed8';
                    dot.style.background = '#3b82f6';
                    label.textContent = 'Autosaving...';
                }} else if (state === 'synced') {{
                    badge.style.background = '#ecfdf5';
                    badge.style.borderColor = '#a7f3d0';
                    badge.style.color = '#059669';
                    dot.style.background = '#10b981';
                    const timeStr = lastAutosaveTime ? lastAutosaveTime.toLocaleTimeString([], {{ hour: '2-digit', minute: '2-digit', second: '2-digit' }}) : 'now';
                    label.textContent = 'Autosaved ' + timeStr;
                }} else if (state === 'error') {{
                    badge.style.background = '#fef2f2';
                    badge.style.borderColor = '#fecaca';
                    badge.style.color = '#b91c1c';
                    dot.style.background = '#ef4444';
                    label.textContent = 'Autosave offline (Local Backup)';
                }}
            }}

            function saveToLocalStorage() {{
                try {{
                    const key = 'neopress_draft_' + (activePostId || 'new_{post_type}');
                    const draft = {{
                        title: titleInput.value,
                        slug: slugInput.value,
                        content: currentMode === 'visual' ? visualEditor.innerHTML : contentArea.value,
                        excerpt: document.getElementById('excerpt').value,
                        timestamp: Date.now()
                    }};
                    localStorage.setItem(key, JSON.stringify(draft));
                }} catch(e) {{}}
            }}

            function checkLocalDraftRecovery() {{
                try {{
                    const key = 'neopress_draft_' + (activePostId || 'new_{post_type}');
                    const raw = localStorage.getItem(key);
                    if (!raw) return;
                    const draft = JSON.parse(raw);
                    if (draft && draft.timestamp && (Date.now() - draft.timestamp) < 7 * 86400 * 1000) {{
                        const banner = document.getElementById('draftRecoveryBanner');
                        const timeElem = document.getElementById('recoveryTime');
                        if (banner && timeElem) {{
                            timeElem.textContent = new Date(draft.timestamp).toLocaleTimeString();
                            banner.style.display = 'flex';
                        }}
                    }}
                }} catch(e) {{}}
            }}

            function restoreLocalDraft() {{
                try {{
                    const key = 'neopress_draft_' + (activePostId || 'new_{post_type}');
                    const raw = localStorage.getItem(key);
                    if (!raw) return;
                    const draft = JSON.parse(raw);
                    if (draft.title) titleInput.value = draft.title;
                    if (draft.slug) slugInput.value = draft.slug;
                    if (draft.content) {{
                        contentArea.value = draft.content;
                        visualEditor.innerHTML = cleanWpContent(draft.content);
                    }}
                    if (draft.excerpt) document.getElementById('excerpt').value = draft.excerpt;
                    updateWordCount();
                    updateSerpPreview();
                    dismissLocalDraft();
                    showToast('✓ Local draft restored!');
                }} catch(e) {{}}
            }}

            function dismissLocalDraft() {{
                try {{
                    const key = 'neopress_draft_' + (activePostId || 'new_{post_type}');
                    localStorage.removeItem(key);
                    const banner = document.getElementById('draftRecoveryBanner');
                    if (banner) banner.style.display = 'none';
                }} catch(e) {{}}
            }}

            setInterval(() => {{
                if (isDirty && !isAutosaving) {{
                    triggerAutosave(false);
                }}
            }}, 15000);

            async function triggerAutosave(showExplicitToast = false) {{
                if (isAutosaving || isVisualLocked) return;
                if (currentMode === 'visual') syncVisualToTextarea();

                const title = titleInput.value.trim();
                const content = contentArea.value.trim();
                if (!title && !content) return;

                isAutosaving = true;
                updateAutosaveBadge('saving');

                const slug = slugInput.value.trim();
                const excerpt = document.getElementById('excerpt').value;
                const post_type = '{post_type}';
                const status = document.getElementById('post_status').value || 'publish';
                const parent = document.getElementById('post_parent') ? parseInt(document.getElementById('post_parent').value) || 0 : 0;
                const menu_order = document.getElementById('menu_order') ? parseInt(document.getElementById('menu_order').value) || 0 : 0;
                const featured_media = parseInt(document.getElementById('featured_media').value) || 0;
                const categories = Array.from(document.querySelectorAll('input[name="post_category"]:checked')).map(c => parseInt(c.value));

                try {{
                    let res, data;
                    if (activePostId && activePostId.length > 0) {{
                        res = await fetch('/wp-json/wp/v2/posts/' + activePostId, {{
                            method: 'PUT',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: JSON.stringify({{
                                title: title || 'Draft ' + new Date().toLocaleTimeString(),
                                slug: slug || undefined,
                                content,
                                excerpt: excerpt || undefined,
                                status,
                                parent,
                                menu_order,
                                featured_media: featured_media > 0 ? featured_media : undefined,
                                categories,
                                meta: collectAcfMeta()
                            }})
                        }});
                        data = await res.json();
                    }} else {{
                        res = await fetch('/wp-json/wp/v2/posts', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            body: JSON.stringify({{
                                title: title || 'Draft ' + new Date().toLocaleTimeString(),
                                slug: slug || undefined,
                                content,
                                excerpt: excerpt || undefined,
                                status: 'draft',
                                post_type,
                                parent,
                                menu_order,
                                featured_media: featured_media > 0 ? featured_media : undefined,
                                categories
                            }})
                        }});
                        data = await res.json();
                        if (res.ok && data.id) {{
                            activePostId = data.id.toString();
                            const newUrl = '/n-admin/post.php?post=' + data.id + '&action=edit' + (post_type === 'page' ? '&post_type=page' : '');
                            window.history.replaceState({{ id: data.id }}, '', newUrl);
                        }}
                    }}

                    if (res.ok) {{
                        isDirty = false;
                        lastAutosaveTime = new Date();
                        updateAutosaveBadge('synced');
                        const key = 'neopress_draft_' + (activePostId || 'new_{post_type}');
                        localStorage.removeItem(key);
                        if (showExplicitToast) {{
                            showToast('✓ Saved to SQLite (WAL)');
                        }}
                    }} else {{
                        updateAutosaveBadge('error');
                    }}
                }} catch(e) {{
                    console.error('Autosave error:', e);
                    updateAutosaveBadge('error');
                }} finally {{
                    isAutosaving = false;
                }}
            }}
        "#,
        post_type = post_type,
    )
}
