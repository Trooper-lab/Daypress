//! Client-side Block & Classic Editor runtime script (`templates/admin/editor/scripts.rs`).

pub fn render_editor_scripts(
    site_title: &str,
    initial_id: &str,
    has_visual_layout: bool,
    type_singular: &str,
    post_type: &str,
    known_tags_json: &str,
) -> String {
    format!(
        r##"
            let currentMode = 'visual'; // 'visual' (default) or 'html'
            let currentMediaTarget = 'content'; // 'content' or 'featured'
            const siteTitle = '{site_title}';
            let activePostId = '{initial_id}' || '';
            let isDirty = false;
            let isAutosaving = false;
            let lastAutosaveTime = null;

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

            // Initialize editor on DOM load: Live Visual Editor is active by default
            document.addEventListener('DOMContentLoaded', () => {{
                const raw = contentArea.value || '';
                visualEditor.innerHTML = cleanWpContent(raw);
                if (!visualEditor.innerHTML || visualEditor.innerHTML.trim() === '' || visualEditor.innerHTML === '<p></p>') {{
                    visualEditor.innerHTML = '<p><br></p>';
                }}

                updateWordCount();
                updateSerpPreview();
                checkLocalDraftRecovery();
            }});

            // Clicking empty visual canvas focuses into first paragraph
            visualEditor.addEventListener('click', (e) => {{
                if (e.target === visualEditor && (!visualEditor.innerHTML || visualEditor.innerHTML.trim() === '')) {{
                    visualEditor.innerHTML = '<p><br></p>';
                }}
            }});

            function cleanWpContent(raw) {{
                if (!raw) return '<p></p>';
                // Strip WordPress Gutenberg block comments safely
                let cleaned = raw.replace(/<!--[\\s\\S]*?-->/g, '').trim();
                if (!cleaned) return '<p></p>';
                // If pure plain text without tags, wrap into clean paragraphs
                if (!cleaned.includes('<p>') && !cleaned.includes('<div>') && !cleaned.includes('<h') && !cleaned.includes('<table')) {{
                    cleaned = cleaned.split('\\n\\n').map(p => '<p>' + p.trim().replace(/\\n/g, '<br>') + '</p>').join('');
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

            // Comprehensive Keyboard Shortcuts: Ctrl+S (Save), Ctrl+B, Ctrl+I, Ctrl+U, Ctrl+K (Link), Ctrl+Shift+H / Alt+Shift+H (Toggle View)
            document.addEventListener('keydown', (e) => {{
                const isMac = navigator.platform.toUpperCase().indexOf('MAC') >= 0;
                const modifier = isMac ? e.metaKey : e.ctrlKey;

                if (modifier && (e.key === 's' || e.key === 'S')) {{
                    e.preventDefault();
                    triggerAutosave(true); // explicit save with toast
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
                    alert('Please save this ' + '{type_singular}' + ' first so we can initialize your visual layout.');
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
                currentMode = mode;

                const btnVisual = document.getElementById('btnModeVisual');
                const btnHtml = document.getElementById('btnModeHtml');
                const btnBuilder = document.getElementById('btnModeBuilder');
                const toolbar = document.getElementById('editorToolbar');
                const inlineBuilder = document.getElementById('inlineBuilderContainer');

                // Reset all tabs
                [btnVisual, btnHtml, btnBuilder].forEach(b => {{
                    if (b) {{
                        b.style.background = 'transparent';
                        b.style.color = '#64748b';
                        b.style.boxShadow = 'none';
                    }}
                }});

                if (mode === 'visual') {{
                    visualEditor.innerHTML = cleanWpContent(contentArea.value);
                    if (!visualEditor.innerHTML || visualEditor.innerHTML.trim() === '') {{
                        visualEditor.innerHTML = '<p><br></p>';
                    }}
                    visualEditor.style.display = 'block';
                    contentArea.style.display = 'none';
                    if (inlineBuilder) inlineBuilder.style.display = 'none';
                    if (toolbar) toolbar.style.display = 'flex';

                    btnVisual.style.background = '#ffffff';
                    btnVisual.style.color = '#0f172a';
                    btnVisual.style.boxShadow = '0 1px 3px rgba(0,0,0,0.1)';
                    showToast('Switched to Visual Live Editor');
                }} else if (mode === 'html') {{
                    syncVisualToTextarea();
                    visualEditor.style.display = 'none';
                    contentArea.style.display = 'block';
                    if (inlineBuilder) inlineBuilder.style.display = 'none';
                    if (toolbar) toolbar.style.display = 'flex';

                    btnHtml.style.background = '#ffffff';
                    btnHtml.style.color = '#0f172a';
                    btnHtml.style.boxShadow = '0 1px 3px rgba(0,0,0,0.1)';
                    showToast('Switched to HTML Source View');
                }} else if (mode === 'builder') {{
                    visualEditor.style.display = 'none';
                    contentArea.style.display = 'none';
                    if (toolbar) toolbar.style.display = 'none';
                    if (inlineBuilder) inlineBuilder.style.display = 'block';

                    btnBuilder.style.background = '#ffffff';
                    btnBuilder.style.color = '#4f46e5';
                    btnBuilder.style.boxShadow = '0 1px 3px rgba(0,0,0,0.1)';

                    if (!inlineLayoutState) {{
                        initInlineLayout();
                    }}
                    renderInlineCanvas();
                    showToast('⚡ Neopress Visual Drag-and-Drop Builder Active');
                }}
                updateWordCount();
            }}

            function initInlineLayout() {{
                inlineLayoutState = {{
                    version: 1,
                    root: {{
                        id: 'sec_root',
                        type: 'Section',
                        styles: {{ padding_top: '30px', padding_bottom: '30px', background_color: '#f8fafc' }},
                        children: [
                            {{
                                id: 'node_h1',
                                type: 'Heading',
                                data: {{ text: titleInput.value || 'Crafted with Neopress Visual Builder', level: 1 }},
                                styles: {{ font_size: '34px', text_color: '#0f172a', margin_bottom: '14px', text_align: 'center' }}
                            }},
                            {{
                                id: 'node_p1',
                                type: 'Text',
                                data: {{ content_html: '<p>Drag, drop, and edit semantic components rendered directly in native Rust. Sub-millisecond performance with zero runtime JS.</p>' }},
                                styles: {{ font_size: '16px', text_color: '#64748b', margin_bottom: '24px', text_align: 'center' }}
                            }},
                            {{
                                id: 'node_btn1',
                                type: 'Button',
                                data: {{ text: 'Get Started Now ⚡', url: '#' }},
                                styles: {{ background_color: '#4f46e5', text_color: '#ffffff', padding_top: '10px', padding_bottom: '10px', padding_left: '24px', padding_right: '24px', border_radius: '6px', text_align: 'center' }}
                            }}
                        ]
                    }}
                }};
            }}

            function renderInlineCanvas() {{
                const root = document.getElementById('inlineCanvasRoot');
                if (!root || !inlineLayoutState) return;

                function renderNode(node) {{
                    const s = node.styles || {{}};
                    const styleStr = Object.entries(s).map(([k, v]) => `${{k.replace(/_/g, '-')}}: ${{v}}`).join('; ');
                    const type = node.type || 'Container';

                    const actionBtns = `
                        <div style="position: absolute; top: 4px; right: 4px; display: none; gap: 4px; background: #1e293b; padding: 2px 6px; border-radius: 4px; font-size: 10px;" class="inline-node-actions">
                            <span style="color: #a5b4fc; font-weight: 700;">${{type}}</span>
                            <button type="button" onclick="event.stopPropagation(); deleteInlineNode('${{node.id}}')" style="color: #ef4444; background: none; border: none; cursor: pointer;">✕</button>
                        </div>
                    `;

                    let inner = '';
                    if (node.children && node.children.length > 0) {{
                        inner = node.children.map(renderNode).join('');
                    }}

                    if (type === 'Heading') {{
                        const text = (node.data && node.data.text) || 'Heading';
                        const lvl = (node.data && node.data.level) || 2;
                        return `<div id="${{node.id}}" class="inline-block-item" style="position: relative; padding: 6px; margin: 4px 0; border: 1px dashed transparent; ${{styleStr}}" onmouseover="this.querySelector('.inline-node-actions').style.display='flex'" onmouseout="this.querySelector('.inline-node-actions').style.display='none'">
                            ${{actionBtns}}
                            <h${{lvl}} contenteditable="true" onblur="inlineUpdateText('${{node.id}}', this.innerText)" style="margin: 0; outline: none;">${{text}}</h${{lvl}}>
                        </div>`;
                    }}

                    if (type === 'Text') {{
                        const html = (node.data && node.data.content_html) || '<p>Paragraph text</p>';
                        return `<div id="${{node.id}}" class="inline-block-item" style="position: relative; padding: 6px; margin: 4px 0; border: 1px dashed transparent; line-height: 1.6; ${{styleStr}}" onmouseover="this.querySelector('.inline-node-actions').style.display='flex'" onmouseout="this.querySelector('.inline-node-actions').style.display='none'">
                            ${{actionBtns}}
                            <div contenteditable="true" onblur="inlineUpdateHtml('${{node.id}}', this.innerHTML)" style="outline: none;">${{html}}</div>
                        </div>`;
                    }}

                    if (type === 'Button') {{
                        const btnText = (node.data && node.data.text) || 'Button';
                        return `<div id="${{node.id}}" class="inline-block-item" style="position: relative; padding: 6px; margin: 4px 0; border: 1px dashed transparent; text-align: ${{s.text_align || 'left'}};" onmouseover="this.querySelector('.inline-node-actions').style.display='flex'" onmouseout="this.querySelector('.inline-node-actions').style.display='none'">
                            ${{actionBtns}}
                            <span contenteditable="true" onblur="inlineUpdateText('${{node.id}}', this.innerText)" style="display: inline-block; cursor: pointer; outline: none; ${{styleStr}}">${{btnText}}</span>
                        </div>`;
                    }}

                    if (type === 'Form') {{
                        return `<div id="${{node.id}}" class="inline-block-item" style="position: relative; padding: 16px; margin: 8px 0; background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 6px; ${{styleStr}}" onmouseover="this.querySelector('.inline-node-actions').style.display='flex'" onmouseout="this.querySelector('.inline-node-actions').style.display='none'">
                            ${{actionBtns}}
                            <div style="font-weight: 700; color: #0f172a; margin-bottom: 6px;">⚡ Declarative Honeypot Form</div>
                            <div style="display: flex; gap: 8px; margin-bottom: 8px;">
                                <input type="text" placeholder="Name" style="flex: 1; padding: 6px 10px; border: 1px solid #cbd5e1; border-radius: 4px; font-size: 12px;" disabled />
                                <input type="email" placeholder="Email" style="flex: 1; padding: 6px 10px; border: 1px solid #cbd5e1; border-radius: 4px; font-size: 12px;" disabled />
                            </div>
                        </div>`;
                    }}

                    if (type === 'Counter') {{
                        return `<div id="${{node.id}}" class="inline-block-item" style="position: relative; text-align: center; padding: 14px; margin: 6px 0; border: 1px dashed transparent; ${{styleStr}}" onmouseover="this.querySelector('.inline-node-actions').style.display='flex'" onmouseout="this.querySelector('.inline-node-actions').style.display='none'">
                            ${{actionBtns}}
                            <div style="font-size: 32px; font-weight: 800; color: #4f46e5;">99.9%</div>
                            <div style="font-size: 12px; color: #64748b;">Sub-millisecond SLA</div>
                        </div>`;
                    }}

                    return `<div id="${{node.id}}" class="inline-block-item" style="position: relative; padding: 10px; margin: 6px 0; border: 1px dashed #e2e8f0; border-radius: 6px; ${{styleStr}}" onmouseover="this.querySelector('.inline-node-actions').style.display='flex'" onmouseout="this.querySelector('.inline-node-actions').style.display='none'">
                        ${{actionBtns}}
                        ${{inner}}
                    </div>`;
                }}

                root.innerHTML = renderNode(inlineLayoutState.root);
                syncInlineToHtml();
            }}

            function syncInlineToHtml() {{
                const root = document.getElementById('inlineCanvasRoot');
                if (root) {{
                    // Clone and clean action bars
                    const clone = root.cloneNode(true);
                    clone.querySelectorAll('.inline-node-actions').forEach(el => el.remove());
                    contentArea.value = clone.innerHTML;
                    markDirty();
                }}
            }}

            function inlineUpdateText(id, text) {{
                function update(n) {{
                    if (n.id === id) {{
                        if (!n.data) n.data = {{}};
                        n.data.text = text;
                        return true;
                    }}
                    if (n.children) for (const c of n.children) if (update(c)) return true;
                    return false;
                }}
                if (inlineLayoutState) {{
                    update(inlineLayoutState.root);
                    syncInlineToHtml();
                }}
            }}

            function inlineUpdateHtml(id, html) {{
                function update(n) {{
                    if (n.id === id) {{
                        if (!n.data) n.data = {{}};
                        n.data.content_html = html;
                        return true;
                    }}
                    if (n.children) for (const c of n.children) if (update(c)) return true;
                    return false;
                }}
                if (inlineLayoutState) {{
                    update(inlineLayoutState.root);
                    syncInlineToHtml();
                }}
            }}

            function addInlineBlock(type) {{
                if (!inlineLayoutState) initInlineLayout();
                const uid = 'inline_' + Math.random().toString(36).substr(2, 6);
                let node = {{ id: uid, type, styles: {{}}, children: [] }};

                if (type === 'Section') {{
                    node.styles = {{ padding_top: '24px', padding_bottom: '24px', background_color: '#f1f5f9' }};
                    node.children = [
                        {{ id: uid + '_h', type: 'Heading', data: {{ text: 'New Feature Spotlight', level: 2 }}, styles: {{ font_size: '24px', text_color: '#0f172a' }} }},
                        {{ id: uid + '_t', type: 'Text', data: {{ content_html: '<p>Built with native Rust ergonomics and zero plugin bloat.</p>' }}, styles: {{ font_size: '14px', text_color: '#64748b' }} }}
                    ];
                }} else if (type === 'Heading') {{
                    node.data = {{ text: 'Section Heading', level: 2 }};
                    node.styles = {{ font_size: '26px', text_color: '#0f172a', margin_bottom: '8px' }};
                }} else if (type === 'Text') {{
                    node.data = {{ content_html: '<p>Add rich editorial prose, formatted quotes, and bullet points.</p>' }};
                    node.styles = {{ font_size: '15px', text_color: '#475569' }};
                }} else if (type === 'Button') {{
                    node.data = {{ text: 'Explore Documentation ↗', url: '#' }};
                    node.styles = {{ background_color: '#4f46e5', text_color: '#ffffff', padding_top: '8px', padding_bottom: '8px', padding_left: '18px', padding_right: '18px', border_radius: '6px' }};
                }} else if (type === 'Grid2') {{
                    node.styles = {{ display: 'grid', grid_template_columns: '1fr 1fr', gap: '16px', margin: '12px 0' }};
                    node.children = [
                        {{ id: uid + '_col1', type: 'Text', data: {{ content_html: '<strong>Column One:</strong> High-performance SQLite engine.' }}, styles: {{ background_color: '#ffffff', padding_top: '12px', padding_bottom: '12px', padding_left: '12px', padding_right: '12px', border_radius: '6px' }} }},
                        {{ id: uid + '_col2', type: 'Text', data: {{ content_html: '<strong>Column Two:</strong> Streaming WXR XML parser.' }}, styles: {{ background_color: '#ffffff', padding_top: '12px', padding_bottom: '12px', padding_left: '12px', padding_right: '12px', border_radius: '6px' }} }}
                    ];
                }} else if (type === 'Grid3') {{
                    node.styles = {{ display: 'grid', grid_template_columns: '1fr 1fr 1fr', gap: '14px', margin: '12px 0' }};
                    node.children = [
                        {{ id: uid + '_c1', type: 'Text', data: {{ content_html: '<strong>Fast:</strong> &lt;1ms TTFB' }}, styles: {{ background_color: '#ffffff', padding_top: '10px', padding_bottom: '10px', padding_left: '10px', padding_right: '10px', border_radius: '6px' }} }},
                        {{ id: uid + '_c2', type: 'Text', data: {{ content_html: '<strong>Safe:</strong> Zero CVEs' }}, styles: {{ background_color: '#ffffff', padding_top: '10px', padding_bottom: '10px', padding_left: '10px', padding_right: '10px', border_radius: '6px' }} }},
                        {{ id: uid + '_c3', type: 'Text', data: {{ content_html: '<strong>Portable:</strong> Single binary' }}, styles: {{ background_color: '#ffffff', padding_top: '10px', padding_bottom: '10px', padding_left: '10px', padding_right: '10px', border_radius: '6px' }} }}
                    ];
                }}

                inlineLayoutState.root.children.push(node);
                renderInlineCanvas();
                showToast(`Added ${{type}} block`);
            }}

            function deleteInlineNode(id) {{
                function remove(parent) {{
                    if (!parent.children) return false;
                    const idx = parent.children.findIndex(c => c.id === id);
                    if (idx !== -1) {{
                        parent.children.splice(idx, 1);
                        return true;
                    }}
                    for (const c of parent.children) if (remove(c)) return true;
                    return false;
                }}
                if (inlineLayoutState) {{
                    remove(inlineLayoutState.root);
                    renderInlineCanvas();
                    showToast('Block removed');
                }}
            }}

            function openSchemaModal() {{
                if (!inlineLayoutState) initInlineLayout();
                alert('⚡ Neopress AST Schema JSON:\n\n' + JSON.stringify(inlineLayoutState, null, 2));
            }}

            function updateWordCount() {{
                const text = currentMode === 'visual' ? (visualEditor.innerText || '') : (contentArea.value || '');
                const clean = text.replace(/<[^>]*>/g, ' ').trim();
                const words = clean ? clean.split(/\\s+/).filter(Boolean).length : 0;
                const readTime = Math.ceil(words / 200) || 1;
                document.getElementById('wordCount').textContent = words + ' words';
                document.getElementById('readTime').textContent = readTime + ' min read';
            }}

            function updateSerpPreview() {{
                const rawTitle = titleInput.value || 'Untitled {type_singular}';
                const seoTitleEl = document.getElementById('seo_title');
                const customSeoTitle = seoTitleEl ? seoTitleEl.value.trim() : '';
                const displayTitle = customSeoTitle || (rawTitle + ' – ' + siteTitle);
                const slug = slugInput.value || 'slug';
                const textSource = currentMode === 'visual' ? (visualEditor.innerText || '') : contentArea.value.replace(/<[^>]*>/g, ' ');
                const seoDescEl = document.getElementById('seo_description');
                const customSeoDesc = seoDescEl ? seoDescEl.value.trim() : '';
                const excerpt = customSeoDesc || document.getElementById('excerpt').value || (textSource ? textSource.substring(0, 155) + '...' : 'No meta description provided...');

                document.getElementById('serpTitle').textContent = displayTitle;
                document.getElementById('serpUrl').textContent = window.location.origin + '/' + slug + '/';
                document.getElementById('serpDesc').textContent = excerpt;
                document.getElementById('viewPageLink').href = '/' + slug + '/';

                // Compute live SEO & Readability score
                const kwEl = document.getElementById('seo_focus_kw');
                const kw = kwEl ? kwEl.value.trim().toLowerCase() : '';
                let score = 50;
                const checks = [];
                if (rawTitle.length >= 10) {{ score += 15; checks.push('✓ Title length'); }}
                if (excerpt.length >= 50 && excerpt.length <= 165) {{ score += 15; checks.push('✓ Meta length'); }}
                if (textSource.trim().split(/\\s+/).filter(Boolean).length >= 50) {{ score += 10; }}
                if (kw) {{
                    if (displayTitle.toLowerCase().includes(kw)) {{ score += 5; checks.push('✓ KW in title'); }}
                    if (excerpt.toLowerCase().includes(kw) || textSource.toLowerCase().includes(kw)) {{ score += 5; checks.push('✓ KW in body'); }}
                }} else {{
                    score += 5;
                }}
                if (score > 100) score = 100;
                const scoreBadge = document.getElementById('seoScoreBadge');
                if (scoreBadge) {{
                    scoreBadge.textContent = 'SEO Score: ' + score + ' / 100';
                    if (score >= 80) {{
                        scoreBadge.style.background = '#ecfdf5';
                        scoreBadge.style.color = '#059669';
                        scoreBadge.style.borderColor = '#a7f3d0';
                    }} else {{
                        scoreBadge.style.background = '#fffbeb';
                        scoreBadge.style.color = '#d97706';
                        scoreBadge.style.borderColor = '#fde68a';
                    }}
                }}
                const summaryEl = document.getElementById('seoChecklistSummary');
                if (summaryEl) {{
                    summaryEl.textContent = checks.join(' • ');
                }}
            }}

            async function duplicateCurrentPost() {{
                if (!activePostId) return;
                showToast('⚡ Cloning #' + activePostId + '...');
                try {{
                    const res = await fetch('/wp-json/neopress/v1/posts/' + activePostId + '/duplicate', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }}
                    }});
                    if (res.ok) {{
                        const cloned = await res.json();
                        showToast('✓ Cloned! Redirecting to Draft #' + cloned.id + '...');
                        setTimeout(() => {{
                            window.location.href = '/n-admin/post.php?post=' + cloned.id + '&action=edit';
                        }}, 500);
                    }} else {{
                        showToast('Failed to duplicate item', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}

            // Formatting toolbar actions
            function formatBlockType(tag) {{
                if (!tag) return;
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    document.execCommand('formatBlock', false, tag);
                    syncVisualToTextarea();
                }} else {{
                    wrapSelectionInTextarea(tag);
                }}
                markDirty();
            }}

            function execStyle(cmd) {{
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    document.execCommand(cmd, false, null);
                    syncVisualToTextarea();
                }} else {{
                    const map = {{ 'bold': 'strong', 'italic': 'em', 'underline': 'u', 'strikeThrough': 's' }};
                    wrapSelectionInTextarea(map[cmd] || cmd);
                }}
                markDirty();
            }}

            function wrapSelectionInTextarea(tag) {{
                const start = contentArea.selectionStart;
                const end = contentArea.selectionEnd;
                const sel = contentArea.value.substring(start, end);
                const before = contentArea.value.substring(0, start);
                const after = contentArea.value.substring(end);
                const replacement = '<' + tag + '>' + (sel || 'text') + '</' + tag + '>';
                contentArea.value = before + replacement + after;
                contentArea.focus();
                contentArea.selectionStart = start + tag.length + 2;
                contentArea.selectionEnd = start + tag.length + 2 + (sel || 'text').length;
                updateWordCount();
                markDirty();
            }}

            function insertInlineCode() {{
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    const sel = window.getSelection();
                    const text = sel ? sel.toString() : '';
                    document.execCommand('insertHTML', false, '<code>' + (text || 'code') + '</code>');
                    syncVisualToTextarea();
                }} else {{
                    wrapSelectionInTextarea('code');
                }}
                markDirty();
            }}

            function insertList(type) {{
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    if (type === 'ul') {{
                        document.execCommand('insertUnorderedList', false, null);
                    }} else {{
                        document.execCommand('insertOrderedList', false, null);
                    }}
                    syncVisualToTextarea();
                }} else {{
                    const text = '\\n<' + type + '>\\n  <li>List item 1</li>\\n  <li>List item 2</li>\\n</' + type + '>\\n';
                    insertRaw(text);
                }}
                markDirty();
            }}

            function insertQuote() {{
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    document.execCommand('formatBlock', false, '<blockquote>');
                    syncVisualToTextarea();
                }} else {{
                    wrapSelectionInTextarea('blockquote');
                }}
                markDirty();
            }}

            function insertLinkPrompt() {{
                const url = prompt("Enter URL destination (e.g. https://... or /about-us/):", "https://");
                if (!url) return;
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    document.execCommand('createLink', false, url);
                    syncVisualToTextarea();
                }} else {{
                    const start = contentArea.selectionStart;
                    const end = contentArea.selectionEnd;
                    const sel = contentArea.value.substring(start, end) || url;
                    const replace = '<a href="' + url + '">' + sel + '</a>';
                    contentArea.value = contentArea.value.substring(0, start) + replace + contentArea.value.substring(end);
                    contentArea.focus();
                    updateWordCount();
                }}
                markDirty();
            }}

            function clearFormatting() {{
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    document.execCommand('removeFormat', false, null);
                    syncVisualToTextarea();
                }}
                markDirty();
            }}

            function insertTable() {{
                const tableHtml = '\\n<table class="neo-table" style="width: 100%; border-collapse: collapse; margin: 16px 0;">\\n  <thead>\\n    <tr style="background: #f8fafc;">\\n      <th style="border: 1px solid #cbd5e1; padding: 8px 12px; text-align: left;">Header 1</th>\\n      <th style="border: 1px solid #cbd5e1; padding: 8px 12px; text-align: left;">Header 2</th>\\n      <th style="border: 1px solid #cbd5e1; padding: 8px 12px; text-align: left;">Header 3</th>\\n    </tr>\\n  </thead>\\n  <tbody>\\n    <tr>\\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data A1</td>\\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data A2</td>\\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data A3</td>\\n    </tr>\\n    <tr>\\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data B1</td>\\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data B2</td>\\n      <td style="border: 1px solid #cbd5e1; padding: 8px 12px;">Data B3</td>\\n    </tr>\\n  </tbody>\\n</table>\\n<p></p>\\n';
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    document.execCommand('insertHTML', false, tableHtml);
                    syncVisualToTextarea();
                }} else {{
                    insertRaw(tableHtml);
                }}
                markDirty();
            }}

            function insertCallout() {{
                const calloutHtml = '\\n<div style="background: #eff6ff; border-left: 4px solid #2563eb; padding: 14px 18px; border-radius: 6px; margin: 16px 0; font-size: 14px; color: #1e3a8a;">\\n    <strong>💡 Note:</strong> Add important highlighted details or tips here.\\n</div>\\n<p></p>\\n';
                if (currentMode === 'visual') {{
                    visualEditor.focus();
                    document.execCommand('insertHTML', false, calloutHtml);
                    syncVisualToTextarea();
                }} else {{
                    insertRaw(calloutHtml);
                }}
                markDirty();
            }}

            function insertRaw(text) {{
                const start = contentArea.selectionStart;
                const end = contentArea.selectionEnd;
                contentArea.value = contentArea.value.substring(0, start) + text + contentArea.value.substring(end);
                contentArea.focus();
                updateWordCount();
                markDirty();
            }}

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
                    const imgHtml = '<figure style="margin: 16px 0;"><img src="' + url + '" alt="' + (title || 'Image') + '" style="max-width: 100%; height: auto; border-radius: 8px; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.08);" /><figcaption style="font-size: 12px; color: #64748b; margin-top: 6px; text-align: center;">' + (title || '') + '</figcaption></figure><p></p>';
                    if (currentMode === 'visual') {{
                        visualEditor.focus();
                        document.execCommand('insertHTML', false, imgHtml);
                        syncVisualToTextarea();
                    }} else {{
                        insertRaw('\\n' + imgHtml + '\\n');
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
                showToast('Uploading to Neopress storage...');
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

            function addTagChip(tag) {{
                const input = document.getElementById('post_tags');
                if (!input) return;
                const current = input.value.split(',').map(s => s.trim()).filter(Boolean);
                if (!current.includes(tag)) {{
                    current.push(tag);
                    input.value = current.join(', ');
                    showToast('✓ Tag added: ' + tag);
                    markDirty();
                }}
            }}

            async function inlineAddCategory() {{
                const input = document.getElementById('newCatName');
                const name = input.value.trim();
                if (!name) return;
                try {{
                    const res = await fetch('/wp-json/wp/v2/categories', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify({{ name }})
                    }});
                    const data = await res.json();
                    if (res.ok) {{
                        const list = document.getElementById('categoryList');
                        const label = document.createElement('label');
                        label.style = "display: flex; align-items: center; gap: 8px; font-size: 13px; margin-bottom: 6px; cursor: pointer;";
                        label.innerHTML = '<input type="checkbox" name="post_category" value="' + data.id + '" checked /> <span>' + data.name + '</span>';
                        list.appendChild(label);
                        input.value = '';
                        showToast('✓ Category created: ' + name);
                        markDirty();
                    }}
                }} catch(e) {{
                    showToast('Failed to create category', true);
                }}
            }}

            // Autosave Engine
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

            // Periodic background autosave interval (every 15 seconds)
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
                if (!title && !content) return; // Don't autosave empty

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
                                content: isVisualLocked ? undefined : content,
                                excerpt: excerpt || undefined,
                                status,
                                parent,
                                menu_order,
                                featured_media: featured_media > 0 ? featured_media : undefined,
                                categories
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

            async function trashPost() {{
                const existingId = activePostId;
                if (!existingId) return;
                if (!confirm('Are you sure you want to permanently delete this {type_singular}?')) return;
                try {{
                    const res = await fetch('/wp-json/wp/v2/posts/' + existingId, {{ method: 'DELETE' }});
                    if (res.ok) {{
                        showToast('✓ {type_singular} deleted');
                        setTimeout(() => {{
                            window.location.href = '/n-admin/edit.php' + ('{post_type}' === 'page' ? '?post_type=page' : '');
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
                    content: isVisualLocked ? undefined : content,
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
                    seo_schema_type
                }};

                try {{
                    const res = await fetch(url, {{
                        method: method,
                        headers: {{ 'Content-Type': 'application/json' }},
                        body: JSON.stringify(payload)
                    }});

                    if (res.ok) {{
                        isDirty = false;
                        const key = 'neopress_draft_' + (activePostId || 'new_{post_type}');
                        localStorage.removeItem(key);
                        showToast('✓ {type_singular} saved to SQLite successfully!');
                        setTimeout(() => {{
                            window.location.href = '/n-admin/edit.php' + (post_type === 'page' ? '?post_type=page' : '');
                        }}, 500);
                    }} else {{
                        showToast('Error saving {type_singular}', true);
                    }}
                }} catch(e) {{
                    showToast('Network error: ' + e, true);
                }}
            }}
"##,
        site_title = site_title,
        initial_id = initial_id,
        has_visual_layout = has_visual_layout,
        type_singular = type_singular,
        post_type = post_type,
        known_tags_json = known_tags_json,
    )
}
