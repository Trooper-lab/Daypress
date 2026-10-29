//! Save, duplicate, trash, and media actions (`scripts/actions.rs`).

pub fn render_actions_script(
    post_type: &str,
    type_singular: &str,
    known_tags_json: &str,
) -> String {
    format!(
        r#"
            function copyPermalink() {{
                const slug = slugInput.value;
                const url = window.location.origin + '/' + slug + '/';
                navigator.clipboard.writeText(url);
                showToast('✓ Permalink copied: ' + url);
            }}

            function previewPage() {{
                const slug = slugInput.value;
                if (!slug) {{
                    showToast('Please enter a title or slug first', true);
                    return;
                }}
                window.open('/' + slug + '/', '_blank');
            }}

            // Media Picker Modal
            function openMediaPicker(target) {{
                currentMediaTarget = target;
                document.getElementById('mediaPickerModal').style.display = 'flex';
            }}

            function closeMediaPicker() {{
                document.getElementById('mediaPickerModal').style.display = 'none';
            }}

            function switchMediaTab(tab) {{
                if (tab === 'library') {{
                    document.getElementById('mediaTabLibrary').style.display = 'block';
                    document.getElementById('mediaTabUpload').style.display = 'none';
                    document.getElementById('tabLibraryBtn').className = 'btn btn-primary';
                    document.getElementById('tabUploadBtn').className = 'btn btn-secondary';
                }} else {{
                    document.getElementById('mediaTabLibrary').style.display = 'none';
                    document.getElementById('mediaTabUpload').style.display = 'block';
                    document.getElementById('tabLibraryBtn').className = 'btn btn-secondary';
                    document.getElementById('tabUploadBtn').className = 'btn btn-primary';
                }}
            }}

            function selectMediaItem(id, url, title) {{
                if (currentMediaTarget === 'featured') {{
                    document.getElementById('featured_media').value = id;
                    const elem = document.getElementById('featuredImgElem');
                    elem.src = url;
                    document.getElementById('featuredImgContainer').style.display = 'block';
                    showToast('✓ Featured image assigned!');
                }} else {{
                    const imgHtml = '<figure style="margin: 16px 0;"><img src="' + url + '" alt="' + (title || 'Image') + '" style="max-width: 100%; height: auto; border-radius: var(--radius-sm); box-shadow: 0 4px 6px -1px rgba(0,0,0,0.08);" /><figcaption style="font-size: 12px; color: var(--ink-muted); margin-top: 6px; text-align: center;">' + (title || '') + '</figcaption></figure><p></p>';
                    if (currentMode === 'visual') {{
                        visualEditor.focus();
                        document.execCommand('insertHTML', false, imgHtml);
                        syncVisualToTextarea();
                    }} else {{
                        insertRaw('\n' + imgHtml + '\n');
                    }}
                    showToast('✓ Image inserted into {type_singular}');
                }}
                markDirty();
                closeMediaPicker();
            }}

            function removeFeaturedImage() {{
                document.getElementById('featured_media').value = '0';
                document.getElementById('featuredImgContainer').style.display = 'none';
                showToast('Featured image removed');
                markDirty();
            }}

            async function uploadModalImage(file) {{
                if (!file) return;
                const formData = new FormData();
                formData.append('file', file);
                showToast('Uploading to Daypress storage...');
                try {{
                    const res = await fetch('/wp-json/wp/v2/media', {{ method: 'POST', body: formData }});
                    const data = await res.json();
                    if (res.ok && data.source_url) {{
                        selectMediaItem(data.id, data.source_url, data.slug);
                    }} else {{
                        showToast('Error uploading image', true);
                    }}
                }} catch(e) {{
                    showToast('Upload failed: ' + e, true);
                }}
            }}

            async function uploadEditorImage(file) {{
                uploadModalImage(file);
            }}

            async function duplicateCurrentPost() {{
                if (!activePostId) return;
                showToast('Cloning #' + activePostId + '...');
                try {{
                    const res = await fetch('/wp-json/daypress/v1/posts/' + activePostId + '/duplicate', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }}
                    }});
                    if (res.ok) {{
                        const cloned = await res.json();
                        showToast('✓ Cloned! Redirecting to Draft #' + cloned.id + '...');
                        setTimeout(() => {{
                            window.location.href = '/d-admin/post?post=' + cloned.id + '&action=edit';
                        }}, 500);
                    }} else {{
                        showToast('Failed to duplicate item', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            async function trashPost() {{
                const existingId = activePostId;
                if (!existingId) return;
                if (!confirm('Are you sure you want to permanently delete this {type_singular}?')) return;
                try {{
                    const res = await fetch('/wp-json/wp/v2/posts/' + existingId, {{ method: 'DELETE' }});
                    if (res.ok) {{
                        showToast('✓ {type_singular} deleted');
                        setTimeout(() => {{
                            window.location.href = '/d-admin/edit' + ('{post_type}' === 'page' ? '?post_type=page' : '');
                        }}, 400);
                    }}
                }} catch(e) {{
                    showToast('Error: ' + e, true);
                }}
            }}

            async function savePost(e) {{
                if (e) e.preventDefault();
                await submitData(document.getElementById('post_status').value || 'publish');
            }}

            async function saveDraft() {{
                await submitData('draft');
            }}

            async function submitData(status) {{
                const scheduleFields = dpScheduleOverrides(status, false);
                if (scheduleFields === null) return;
                if (currentMode === 'visual') {{
                    syncVisualToTextarea();
                }}

                const title = titleInput.value.trim();
                const slug = slugInput.value.trim();
                const content = contentArea.value;
                const excerpt = document.getElementById('excerpt').value;
                const post_type = '{post_type}';
                const existingId = activePostId;

                const parent = document.getElementById('post_parent') ? parseInt(document.getElementById('post_parent').value) || 0 : 0;
                const menu_order = document.getElementById('menu_order') ? parseInt(document.getElementById('menu_order').value) || 0 : 0;
                const featured_media = parseInt(document.getElementById('featured_media').value) || 0;

                const categories = Array.from(document.querySelectorAll('input[name="post_category"]:checked')).map(c => parseInt(c.value));
                const tplEl = document.getElementById('page_template') || document.getElementById('page-template-select');
                const template = tplEl ? tplEl.value : 'default';

                const knownTags = {known_tags_json};
                const tagsEl = document.getElementById('post_tags') || document.getElementById('post-tags-input');
                const tagsRaw = tagsEl ? tagsEl.value : '';
                const tagNames = tagsRaw.split(',').map(s => s.trim()).filter(Boolean);
                const tags = [];
                for (const tName of tagNames) {{
                    const lower = tName.toLowerCase();
                    if (knownTags[lower]) {{
                        tags.push(knownTags[lower]);
                    }} else {{
                        try {{
                            const tRes = await fetch('/wp-json/wp/v2/tags', {{
                                method: 'POST',
                                headers: {{ 'Content-Type': 'application/json' }},
                                body: JSON.stringify({{ name: tName }})
                            }});
                            if (tRes.ok) {{
                                const tData = await tRes.json();
                                if (tData && tData.id) {{
                                    knownTags[lower] = tData.id;
                                    tags.push(tData.id);
                                }}
                            }}
                        }} catch (_) {{}}
                    }}
                }}

                let url = '/wp-json/wp/v2/posts';
                let method = 'POST';
                if (existingId && existingId.length > 0) {{
                    url = '/wp-json/wp/v2/posts/' + existingId;
                    method = 'PUT';
                }}

                const seo_title = document.getElementById('seo_title') ? document.getElementById('seo_title').value.trim() : '';
                const seo_description = document.getElementById('seo_description') ? document.getElementById('seo_description').value.trim() : '';
                const seo_keyword = document.getElementById('seo_focus_kw') ? document.getElementById('seo_focus_kw').value.trim() : '';
                const seo_canonical = document.getElementById('seo_canonical') ? document.getElementById('seo_canonical').value.trim() : '';
                const seo_og_image = document.getElementById('seo_og_image') ? document.getElementById('seo_og_image').value.trim() : '';
                const seo_noindex = document.getElementById('seo_noindex') ? document.getElementById('seo_noindex').checked : false;
                const seo_schema_type = document.getElementById('seo_schema_type') ? document.getElementById('seo_schema_type').value : 'Article';

                const payload = {{
                    title: title || 'Untitled',
                    slug: slug || undefined,
                    content,
                    excerpt: excerpt || undefined,
                    status,
                    post_type,
                    parent,
                    menu_order,
                    template,
                    featured_media: featured_media > 0 ? featured_media : undefined,
                    categories,
                    tags,
                    seo_title,
                    seo_description,
                    seo_keyword,
                    seo_canonical,
                    seo_og_image,
                    seo_noindex,
                    seo_schema_type,
                    meta: collectAcfMeta(),
                    taxonomies: collectTaxonomies(),
                    ...scheduleFields
                }};

                try {{
                    const res = await fetch(url, {{
                        method: method,
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify(payload)
                    }});

                    if (res.ok) {{
                        const savedPost = await res.json().catch(() => null);
                        const savedId = (savedPost && savedPost.id) ? savedPost.id : (existingId ? parseInt(existingId, 10) : 0);
                        const hSel = document.getElementById('editorHeaderSelect');
                        const fSel = document.getElementById('editorFooterSelect');
                        if (savedId > 0 && hSel && fSel) {{
                            try {{
                                await fetch('/wp-json/daypress/v1/templates/assign-page', {{
                                    method: 'POST',
                                    headers: {{ 'Content-Type': 'application/json' }},
                                    body: JSON.stringify({{
                                        post_id: savedId,
                                        header: hSel.value,
                                        footer: fSel.value
                                    }})
                                }});
                            }} catch (_) {{}}
                        }}

                        isDirty = false;
                        const key = 'daypress_draft_' + (activePostId || 'new_{post_type}');
                        localStorage.removeItem(key);
                        showToast(savedPost && savedPost.status === 'future' ? '✓ {type_singular} scheduled' : '✓ {type_singular} saved to SQLite successfully!');
                        setTimeout(() => {{
                            window.location.href = '/d-admin/edit' + (post_type !== 'post' ? '?post_type=' + post_type : '');
                        }}, 500);
                    }} else {{
                        showToast('Error saving {type_singular}', true);
                    }}
                }} catch(e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            function updateEditorSlotSummary(slot, statusObj) {{
                const summaryEl = document.getElementById(slot === 'header' ? 'editorResolvedHeaderSummary' : 'editorResolvedFooterSummary');
                const editLinkEl = document.getElementById(slot === 'header' ? 'editorEditHeaderLink' : 'editorEditFooterLink');
                if (!summaryEl || !statusObj) return;
                summaryEl.innerHTML = '<strong style="color:var(--ink);">' + statusObj.display_name + '</strong><div style="font-size:10.5px;color:var(--ink-muted);margin-top:2px;">' + statusObj.reason + '</div>';
                if (editLinkEl) {{
                    if (statusObj.template_id) {{
                        editLinkEl.href = '/d-admin/builder?template_id=' + statusObj.template_id;
                        editLinkEl.style.display = 'inline';
                    }} else {{
                        editLinkEl.style.display = 'none';
                    }}
                }}
            }}

            async function loadEditorHeaderFooterState() {{
                const hSel = document.getElementById('editorHeaderSelect');
                const fSel = document.getElementById('editorFooterSelect');
                if (!hSel || !fSel) return;
                try {{
                    const res = await fetch('/wp-json/daypress/v1/templates/matrix');
                    if (!res.ok) return;
                    const matrix = await res.json();
                    const buildOpts = (templates, slotLabel, currentVal) => {{
                        let html = '<option value="inherit"' + (currentVal === 'inherit' ? ' selected' : '') + '>Inherit Global Rules</option>';
                        if (templates && templates.length > 0) {{
                            html += '<optgroup label="' + slotLabel + ' Templates">';
                            for (const t of templates) {{
                                const val = 'template:' + t.id;
                                const st = t.status === 'publish' ? '' : ' (Inactive)';
                                html += '<option value="' + val + '"' + (currentVal === val ? ' selected' : '') + '>Assign: ' + t.title + ' (#' + t.id + ')' + st + '</option>';
                            }}
                            html += '</optgroup>';
                        }}
                        html += '<optgroup label="System Overrides">';
                        html += '<option value="fallback"' + (currentVal === 'fallback' ? ' selected' : '') + '>Built-in Theme Fallback ' + slotLabel + '</option>';
                        html += '<option value="none"' + (currentVal === 'none' ? ' selected' : '') + '>Hide / No ' + slotLabel + ' on this Page</option>';
                        html += '</optgroup>';
                        return html;
                    }};

                    const pid = activePostId ? parseInt(activePostId, 10) : 0;
                    const allRows = (matrix.pages || []).concat(matrix.posts || []);
                    const matchedRow = pid > 0 ? allRows.find(r => r.post_id === pid) : null;
                    const defaultRouteKey = '{post_type}' === 'page' ? 'all_pages' : 'all_posts';
                    const fallbackRow = (matrix.routes || []).find(r => r.route_key === defaultRouteKey) || (matrix.routes || [])[0];
                    const activeRow = matchedRow || fallbackRow;

                    const hVal = matchedRow ? (matchedRow.header.mode || 'inherit') : 'inherit';
                    const fVal = matchedRow ? (matchedRow.footer.mode || 'inherit') : 'inherit';

                    const headers = (matrix.templates || []).filter(t => t.template_type === 'header');
                    const footers = (matrix.templates || []).filter(t => t.template_type === 'footer');

                    hSel.innerHTML = buildOpts(headers, 'Header', hVal);
                    fSel.innerHTML = buildOpts(footers, 'Footer', fVal);

                    if (activeRow) {{
                        updateEditorSlotSummary('header', activeRow.header);
                        updateEditorSlotSummary('footer', activeRow.footer);
                    }}
                }} catch (_) {{}}
            }}

            async function onEditorSlotChange() {{
                const pid = activePostId ? parseInt(activePostId, 10) : 0;
                const hSel = document.getElementById('editorHeaderSelect');
                const fSel = document.getElementById('editorFooterSelect');
                if (!hSel || !fSel) return;
                if (pid <= 0) {{
                    markDirty();
                    showToast('Header/Footer selection will be applied when you save this {type_singular}.');
                    return;
                }}
                try {{
                    const res = await fetch('/wp-json/daypress/v1/templates/assign-page', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{
                            post_id: pid,
                            header: hSel.value,
                            footer: fSel.value
                        }})
                    }});
                    if (res.ok) {{
                        const matrix = await res.json();
                        const allRows = (matrix.pages || []).concat(matrix.posts || []);
                        const matchedRow = allRows.find(r => r.post_id === pid);
                        if (matchedRow) {{
                            updateEditorSlotSummary('header', matchedRow.header);
                            updateEditorSlotSummary('footer', matchedRow.footer);
                        }}
                        showToast('✓ Header & Footer assignment updated in SQLite');
                    }} else {{
                        showToast('Failed to update Header/Footer assignment', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            document.addEventListener('DOMContentLoaded', () => {{
                loadEditorHeaderFooterState();
            }});
        "#,
        post_type = post_type,
        type_singular = type_singular,
        known_tags_json = known_tags_json,
    )
}
