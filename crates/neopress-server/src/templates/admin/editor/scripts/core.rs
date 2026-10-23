//! Core editor state, events, and mode switching (`scripts/core.rs`).

pub fn render_core_script(site_title: &str, initial_id: &str, has_visual_layout: bool) -> String {
    format!(
        r#"
            let currentMode = 'blocks'; // 'blocks' (default), 'visual', or 'html'
            let currentMediaTarget = 'content'; // 'content' or 'featured'
            const siteTitle = '{site_title}';
            let activePostId = '{initial_id}' || '';
            let isDirty = false;
            let isAutosaving = false;
            let lastAutosaveTime = null;
            let acfBlocks = [];

            const titleInput = document.getElementById('title');
            const slugInput = document.getElementById('post_name');
            const contentArea = document.getElementById('content');
            const visualEditor = document.getElementById('visualEditor');

            // Auto slug generator from title
            titleInput.addEventListener('input', () => {{
                if (!slugInput.value || slugInput.dataset.auto !== 'false') {{
                    slugInput.value = titleInput.value.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
                    updateSerpPreview();
                }}
                markDirty();
            }});
            slugInput.addEventListener('input', () => {{
                slugInput.dataset.auto = 'false';
                updateSerpPreview();
                markDirty();
            }});

            // Initialize editor on DOM load: Gutenberg + ACF Blocks mode is active by default
            document.addEventListener('DOMContentLoaded', () => {{
                const raw = contentArea.value || '';
                visualEditor.innerHTML = cleanWpContent(raw);
                if (!visualEditor.innerHTML || visualEditor.innerHTML.trim() === '' || visualEditor.innerHTML === '<p></p>') {{
                    visualEditor.innerHTML = '<p><br></p>';
                }}

                parseAndRenderAcfBlocks();
                updateWordCount();
                updateSerpPreview();
                checkLocalDraftRecovery();
            }});

            function onAcfFieldChange() {{
                markDirty();
            }}

            const deletedMetaKeys = [];

            function collectAcfMeta() {{
                const meta = {{}};
                document.querySelectorAll('[data-acf-key], [data-meta-key]').forEach(el => {{
                    const key = el.getAttribute('data-acf-key') || el.getAttribute('data-meta-key');
                    if (key) {{
                        meta[key] = el.type === 'checkbox' ? (el.checked ? '1' : '0') : el.value;
                    }}
                }});
                document.querySelectorAll('.custom-meta-row').forEach(row => {{
                    const keyInput = row.querySelector('.custom-meta-key');
                    const valInput = row.querySelector('.custom-meta-val');
                    if (keyInput && valInput) {{
                        const k = keyInput.value.trim();
                        if (k) {{
                            meta[k] = valInput.value;
                        }}
                    }}
                }});
                deletedMetaKeys.forEach(k => {{
                    if (!(k in meta)) {{
                        meta[k] = null;
                    }}
                }});
                return meta;
            }}

            function deleteCustomMetaRow(btn) {{
                const row = btn.closest('.custom-meta-row');
                if (row) {{
                    const keyInput = row.querySelector('.custom-meta-key');
                    if (keyInput && keyInput.value.trim()) {{
                        deletedMetaKeys.push(keyInput.value.trim());
                    }}
                    row.remove();
                    markDirty();
                }}
            }}

            function addCustomMetaRow() {{
                const tbody = document.getElementById('custom-fields-body');
                if (!tbody) return;
                const hint = document.getElementById('custom-fields-empty-hint');
                if (hint) hint.remove();
                const tr = document.createElement('tr');
                tr.className = 'custom-meta-row';
                tr.innerHTML = `
                    <td style="padding: 8px 10px; width: 32%;">
                        <input type="text" class="form-input custom-meta-key" placeholder="meta_key" oninput="markDirty()" style="font-family: var(--font-mono); font-size: 12px; height: 30px;" />
                    </td>
                    <td style="padding: 8px 10px; width: 44%;">
                        <input type="text" class="form-input custom-meta-val" placeholder="meta_value" oninput="updateMetaTypeBadge(this); markDirty()" style="font-size: 12px; height: 30px;" />
                    </td>
                    <td style="padding: 8px 10px; width: 14%; text-align: center;">
                        <span class="badge custom-meta-type-badge" style="font-size: 10px; background: #f1f5f9; color: #475569; text-transform: uppercase;">TEXT</span>
                    </td>
                    <td style="padding: 8px 10px; width: 10%; text-align: center;">
                        <button type="button" class="btn btn-secondary btn-delete-meta" onclick="deleteCustomMetaRow(this)" title="Delete Custom Field" style="height: 26px; padding: 0 6px; font-size: 12px; color: #ef4444; border-color: #fecaca; cursor: pointer;">🗑️</button>
                    </td>
                `;
                tbody.appendChild(tr);
                const firstInput = tr.querySelector('.custom-meta-key');
                if (firstInput) firstInput.focus();
                markDirty();
            }}

            function updateMetaTypeBadge(el) {{
                const row = el.closest('.custom-meta-row');
                if (!row) return;
                const val = row.querySelector('.custom-meta-val')?.value || '';
                const badge = row.querySelector('.custom-meta-type-badge');
                if (!badge) return;
                const t = val.trim();
                let detected = 'TEXT';
                if (t === '1' || t === '0' || t === 'true' || t === 'false') {{
                    detected = 'BOOLEAN';
                }} else if (t !== '' && !isNaN(Number(t))) {{
                    detected = 'NUMBER';
                }} else if ((t.startsWith('{{') && t.endsWith('}}')) || (t.startsWith('[') && t.endsWith(']'))) {{
                    detected = 'JSON';
                }}
                badge.textContent = detected;
            }}

            function collectTaxonomies() {{
                const taxMap = {{}};
                document.querySelectorAll('.taxonomy-term-checkbox:checked').forEach(cb => {{
                    const tax = cb.getAttribute('data-taxonomy');
                    const tid = parseInt(cb.value);
                    if (tax && !isNaN(tid)) {{
                        if (!taxMap[tax]) taxMap[tax] = [];
                        taxMap[tax].push(tid);
                    }}
                }});
                return taxMap;
            }}

            function appendFlatTaxonomyTerm(taxSlug, termName) {{
                const inp = document.getElementById('taxonomy_input_' + taxSlug);
                if (!inp) return;
                const current = inp.value.split(',').map(s => s.trim()).filter(Boolean);
                if (!current.includes(termName)) {{
                    current.push(termName);
                    inp.value = current.join(', ');
                    markDirty();
                }}
            }}

            async function inlineAddTaxonomyTerm(taxSlug) {{
                const inp = document.getElementById('new_term_' + taxSlug);
                if (!inp || !inp.value.trim()) return;
                const name = inp.value.trim();
                try {{
                    const res = await fetch('/wp-json/wp/v2/' + taxSlug, {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ name }})
                    }});
                    const data = await res.json();
                    if (res.ok && data.id) {{
                        const list = document.getElementById('taxonomy-list-' + taxSlug);
                        if (list) {{
                            const label = document.createElement('label');
                            label.style = 'display: flex; align-items: center; gap: 8px; font-size: 13px; margin-bottom: 6px; cursor: pointer;';
                            label.innerHTML = `<input type="checkbox" class="taxonomy-term-checkbox" data-taxonomy="${{taxSlug}}" name="post_category" value="${{data.id}}" checked /> <span>${{name}}</span>`;
                            list.appendChild(label);
                        }}
                        inp.value = '';
                        markDirty();
                        showToast('✓ Added ' + name);
                    }} else {{
                        showToast('Error adding term: ' + (data.message || 'Server error'), true);
                    }}
                }} catch (e) {{
                    showToast('Failed to add term: ' + e, true);
                }}
            }}

            // Clicking empty visual canvas focuses into first paragraph
            visualEditor.addEventListener('click', (e) => {{
                if (e.target === visualEditor && (!visualEditor.innerHTML || visualEditor.innerHTML.trim() === '')) {{
                    visualEditor.innerHTML = '<p><br></p>';
                }}
            }});

            function cleanWpContent(raw) {{
                if (!raw) return '<p></p>';
                // Retain Gutenberg block comments (<!-- wp:... --> and <!-- /wp:... -->) without destructive stripping
                let cleaned = raw.replace(/<!--(?!\s*\/?wp:)[\s\S]*?-->/g, '').trim();
                if (!cleaned) return '<p></p>';
                if (!cleaned.includes('<p>') && !cleaned.includes('<div>') && !cleaned.includes('<h') &&
                    !cleaned.includes('<table') && !cleaned.includes('<section') &&
                    !cleaned.includes('<!-- wp:') && !cleaned.includes('[et_pb_')) {{
                    cleaned = cleaned.split('\n\n').map(p => '<p>' + p.trim().replace(/\n/g, '<br>') + '</p>').join('');
                }}
                return cleaned;
            }}

            function syncVisualToTextarea() {{
                contentArea.value = visualEditor.innerHTML;
            }}

            function onVisualInput() {{
                syncVisualToTextarea();
                updateWordCount();
                updateSerpPreview();
                markDirty();
            }}

            function onHtmlInput() {{
                updateWordCount();
                updateSerpPreview();
                markDirty();
            }}

            // Comprehensive Keyboard Shortcuts: Ctrl+S (Save), Ctrl+B, Ctrl+I, Ctrl+U, Ctrl+K (Link)
            document.addEventListener('keydown', (e) => {{
                const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
                const modifier = isMac ? e.metaKey : e.ctrlKey;

                if (modifier && (e.key === 's' || e.key === 'S')) {{
                    e.preventDefault();
                    triggerAutosave(true);
                    return;
                }}
                if (modifier && (e.key === 'k' || e.key === 'K')) {{
                    e.preventDefault();
                    insertLinkPrompt();
                    return;
                }}
                if (modifier && (e.key === 'b' || e.key === 'B')) {{
                    e.preventDefault();
                    execStyle('bold');
                    return;
                }}
                if (modifier && (e.key === 'i' || e.key === 'I')) {{
                    e.preventDefault();
                    execStyle('italic');
                    return;
                }}
                if (modifier && (e.key === 'u' || e.key === 'U')) {{
                    e.preventDefault();
                    execStyle('underline');
                    return;
                }}
                if (modifier && (e.key === 'h' || e.key === 'H') && (e.shiftKey || e.altKey)) {{
                    e.preventDefault();
                    switchEditorMode(currentMode === 'visual' ? 'html' : 'visual');
                    return;
                }}
            }});

            // Tab key support inside HTML textarea
            contentArea.addEventListener('keydown', (e) => {{
                if (e.key === 'Tab') {{
                    e.preventDefault();
                    const start = contentArea.selectionStart;
                    const end = contentArea.selectionEnd;
                    contentArea.value = contentArea.value.substring(0, start) + '  ' + contentArea.value.substring(end);
                    contentArea.selectionStart = contentArea.selectionEnd = start + 2;
                    markDirty();
                }}
            }});

            const isVisualLocked = {has_visual_layout};

            function switchToVisualBuilder() {{
                if (activePostId && activePostId.length > 0) {{
                    window.location.href = '/n-admin/builder?post=' + activePostId;
                }} else {{
                    alert('Please save this post first so we can initialize your visual layout.');
                }}
            }}

            async function confirmSwitchToStandard() {{
                if (!confirm("Are you sure you want to discard the visual layout?\n\nThis will unlock the standard text editor. Your compiled HTML text will be retained as base content.")) {{
                    return;
                }}
                if (!activePostId) return;
                try {{
                    const res = await fetch('/wp-json/neopress/v1/layout/' + activePostId, {{
                        method: 'DELETE'
                    }});
                    if (res.ok) {{
                        showToast('✓ Visual layout discarded. Unlocking standard text editor...');
                        setTimeout(() => {{
                            window.location.reload();
                        }}, 400);
                    }} else {{
                        showToast('Error removing layout', true);
                    }}
                }} catch(e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            function switchEditorMode(mode) {{
                if (mode === currentMode) return;
                if (currentMode === 'visual') {{
                    syncVisualToTextarea();
                }}
                currentMode = mode;

                const btnBlocks = document.getElementById('btnModeBlocks');
                const btnVisual = document.getElementById('btnModeVisual');
                const btnHtml = document.getElementById('btnModeHtml');
                const toolbar = document.getElementById('editorToolbar');
                const acfContainer = document.getElementById('acfBlocksContainer');

                [btnBlocks, btnVisual, btnHtml].forEach(b => {{
                    if (b) {{
                        b.style.background = 'transparent';
                        b.style.color = '#64748b';
                        b.style.boxShadow = 'none';
                    }}
                }});

                if (mode === 'blocks') {{
                    parseAndRenderAcfBlocks();
                    if (acfContainer) acfContainer.style.display = 'block';
                    visualEditor.style.display = 'none';
                    contentArea.style.display = 'none';
                    if (toolbar) toolbar.style.display = 'none';
                    if (btnBlocks) {{
                        btnBlocks.style.background = '#ffffff';
                        btnBlocks.style.color = '#0f172a';
                        btnBlocks.style.boxShadow = '0 1px 3px rgba(0,0,0,0.1)';
                    }}
                    showToast('Switched to Gutenberg + ACF Blocks View');
                }} else if (mode === 'visual') {{
                    visualEditor.innerHTML = cleanWpContent(contentArea.value);
                    if (!visualEditor.innerHTML || visualEditor.innerHTML.trim() === '') {{
                        visualEditor.innerHTML = '<p><br></p>';
                    }}
                    if (acfContainer) acfContainer.style.display = 'none';
                    visualEditor.style.display = 'block';
                    contentArea.style.display = 'none';
                    if (toolbar) toolbar.style.display = 'flex';
                    btnVisual.style.background = '#ffffff';
                    btnVisual.style.color = '#0f172a';
                    btnVisual.style.boxShadow = '0 1px 3px rgba(0,0,0,0.1)';
                    showToast('Switched to Live Theme Preview');
                }} else if (mode === 'html') {{
                    if (acfContainer) acfContainer.style.display = 'none';
                    visualEditor.style.display = 'none';
                    contentArea.style.display = 'block';
                    if (toolbar) toolbar.style.display = 'flex';
                    btnHtml.style.background = '#ffffff';
                    btnHtml.style.color = '#0f172a';
                    btnHtml.style.boxShadow = '0 1px 3px rgba(0,0,0,0.1)';
                    showToast('Switched to HTML Source View');
                }}
                updateWordCount();
            }}

            function updateWordCount() {{
                const text = currentMode === 'visual' ? (visualEditor.innerText || '') : (contentArea.value || '');
                const clean = text.replace(/<[^>]*>/g, ' ').trim();
                const words = clean ? clean.split(/\s+/).filter(Boolean).length : 0;
                const readTime = Math.ceil(words / 200) || 1;
                document.getElementById('wordCount').textContent = words + ' words';
                document.getElementById('readTime').textContent = readTime + ' min read';
            }}
        "#,
        site_title = site_title,
        initial_id = initial_id,
        has_visual_layout = has_visual_layout,
    )
}
