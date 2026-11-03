//! Client-side JavaScript controller for the 4 Theme Architecture Studio pages:
//! - `/d-admin/headers-footers` (Site Headers, Site Footers & Unified Page Assignment Matrix)
//! - `/d-admin/theme-layouts` (Single Post, Single Page, Archive & 404 Layouts)
//! - `/d-admin/symbols` (Synced Global Symbols, Slot Inspector & Mega Menu Binder)
//! - `/d-admin/popups` (Announcement Bars, Slide-In Drawers & Conversion Modals)

pub fn render_headers_footers_script(
    page_mode: &str,
    initial_matrix_json: &str,
    starters_json: &str,
    nav_menu_items_json: &str,
) -> String {
    format!(
        r##"
        <script>
            window.tsPageMode = "{page_mode}";
            let hfMatrix = {initial_matrix_json};
            const hfStarters = {starters_json};
            let hfNavMenuItems = {nav_menu_items_json};

            let editingConditionsTplId = null;
            let editingSettingsTplId = null;
            let currentMatrixFilter = 'all';
            let currentLayoutFilter = 'all';
            let currentPopupFilter = 'all';
            let currentStarterFilter = 'all';
            const selectedMatrixKeys = new Set();

            function escHtml(str) {{
                return String(str == null ? '' : str)
                    .replace(/&/g, '&amp;')
                    .replace(/</g, '&lt;')
                    .replace(/>/g, '&gt;')
                    .replace(/"/g, '&quot;')
                    .replace(/'/g, '&#39;');
            }}

            function copyTextToClipboard(text) {{
                if (navigator.clipboard && navigator.clipboard.writeText) {{
                    navigator.clipboard.writeText(text).then(() => {{
                        showToast('Copied to clipboard: ' + text);
                    }}).catch(() => {{
                        showToast('Token: ' + text);
                    }});
                }} else {{
                    showToast('Token: ' + text);
                }}
            }}

            function switchMainTab(tab) {{
                if (window.tsPageMode !== 'headers_footers') {{
                    if (tab === 'matrix') {{
                        window.location.href = '/d-admin/headers-footers?tab=matrix';
                    }} else {{
                        window.location.href = '/d-admin/headers-footers';
                    }}
                    return;
                }}
                const pTemplates = document.getElementById('tabPanelTemplates');
                const pMatrix = document.getElementById('tabPanelMatrix');
                const linkHF = document.getElementById('suiteLinkHeadersFooters');
                const linkMat = document.getElementById('suiteLinkMatrix');
                if (pTemplates) pTemplates.style.display = tab === 'templates' ? 'block' : 'none';
                if (pMatrix) pMatrix.style.display = tab === 'matrix' ? 'block' : 'none';
                if (linkHF) linkHF.classList.toggle('active', tab === 'templates');
                if (linkMat) linkMat.classList.toggle('active', tab === 'matrix');
                try {{
                    const url = new URL(window.location.href);
                    if (tab === 'matrix') url.searchParams.set('tab', 'matrix');
                    else url.searchParams.delete('tab');
                    window.history.replaceState({{}}, '', url.toString());
                }} catch (e) {{}}
            }}

            async function refreshMatrixFromServer() {{
                try {{
                    const res = await fetch('/wp-json/daypress/v1/templates/matrix', {{ credentials: 'same-origin' }});
                    if (res.ok) {{
                        hfMatrix = await res.json();
                        renderAllViews();
                    }}
                }} catch (e) {{
                    console.error('Failed to refresh matrix:', e);
                }}
            }}

            function getGlobalDefaultTemplate(slot) {{
                const tpls = (hfMatrix.templates || []).filter(t => t.template_type === slot && t.is_active && t.status === 'publish');
                // Find highest priority template that includes entire_site (or has empty conditions)
                let best = null;
                tpls.forEach(t => {{
                    const conds = t.conditions || [];
                    const hasExcludeSite = conds.some(c => c.condition_type === 'exclude' && c.rule_target === 'entire_site');
                    if (hasExcludeSite) return;
                    const matchesGlobal = conds.length === 0 || conds.some(c => c.condition_type === 'include' && c.rule_target === 'entire_site');
                    if (matchesGlobal) {{
                        if (!best || (t.priority || 10) >= (best.priority || 10)) {{
                            best = t;
                        }}
                    }}
                }});
                return best;
            }}

            function getPrimaryTargetValue(tpl) {{
                const conds = tpl.conditions || [];
                if (conds.length === 1 && conds[0].condition_type === 'exclude' && conds[0].rule_target === 'entire_site') {{
                    return 'unassigned';
                }}
                const inc = conds.find(c => c.condition_type === 'include');
                if (!inc) return 'entire_site';
                if (inc.rule_target === 'page' || inc.rule_target === 'post' || inc.rule_target === 'category' || inc.rule_target === 'tag') {{
                    return 'custom';
                }}
                return inc.rule_target || 'entire_site';
            }}

            function renderAllViews() {{
                renderSuiteCountsAndKpiStrip();
                if (window.tsPageMode === 'headers_footers') {{
                    renderHeadersFootersCards();
                    renderMatrixView();
                }} else if (window.tsPageMode === 'theme_layouts') {{
                    renderThemeLayoutsCards();
                }} else if (window.tsPageMode === 'symbols') {{
                    renderSymbolsCards();
                }} else if (window.tsPageMode === 'popups') {{
                    renderPopupsCards();
                }}
            }}

            function renderSuiteCountsAndKpiStrip() {{
                const tpls = hfMatrix.templates || [];
                const headers = tpls.filter(t => t.template_type === 'header');
                const footers = tpls.filter(t => t.template_type === 'footer');
                const layouts = tpls.filter(t => ['single_post', 'single_page', 'archive', '404_page'].includes(t.template_type));
                const symbols = tpls.filter(t => t.template_type === 'section_symbol');
                const popups = tpls.filter(t => t.template_type === 'popup');

                const routes = hfMatrix.routes || [];
                const pages = hfMatrix.pages || [];
                const posts = hfMatrix.posts || [];
                const allTargets = [...routes, ...pages, ...posts];
                const overrideTargets = allTargets.filter(r =>
                    (r.header && r.header.mode !== 'inherit') ||
                    (r.footer && r.footer.mode !== 'inherit')
                );

                const setElText = (id, val) => {{
                    const el = document.getElementById(id);
                    if (el) el.textContent = String(val);
                }};

                setElText('suiteCountHF', headers.length + footers.length);
                setElText('suiteCountMatrix', allTargets.length);
                setElText('suiteCountLayouts', layouts.length);
                setElText('suiteCountSymbols', symbols.length);
                setElText('suiteCountPopups', popups.length);

                const globalHdr = getGlobalDefaultTemplate('header');
                const globalFtr = getGlobalDefaultTemplate('footer');

                if (window.tsPageMode === 'headers_footers') {{
                    const activeHdrs = headers.filter(t => t.is_active && t.status === 'publish').length;
                    const activeFtrs = footers.filter(t => t.is_active && t.status === 'publish').length;
                    setElText('kpiLabel1', 'Active site headers');
                    setElText('kpiValue1', `${{activeHdrs}} / ${{headers.length}}`);
                    setElText('kpiSub1', globalHdr ? `Default: ${{globalHdr.title}}` : 'Fallback: Built-in Theme Header');

                    setElText('kpiLabel2', 'Active site footers');
                    setElText('kpiValue2', `${{activeFtrs}} / ${{footers.length}}`);
                    setElText('kpiSub2', globalFtr ? `Default: ${{globalFtr.title}}` : 'Fallback: Built-in Theme Footer');

                    setElText('kpiLabel3', 'Page & route overrides');
                    setElText('kpiValue3', overrideTargets.length);
                    setElText('kpiSub3', `Across ${{routes.length}} system routes & ${{pages.length + posts.length}} items`);

                    setElText('kpiLabel4', 'SSR shell hydration');
                    setElText('kpiValue4', '<50 µs');
                    setElText('kpiSub4', 'Sub-millisecond Rust AST injection');
                }} else if (window.tsPageMode === 'theme_layouts') {{
                    const sp = layouts.filter(t => t.template_type === 'single_post');
                    const pg = layouts.filter(t => t.template_type === 'single_page');
                    const ar = layouts.filter(t => t.template_type === 'archive');
                    const e4 = layouts.filter(t => t.template_type === '404_page');

                    setElText('kpiLabel1', 'Single post layouts');
                    setElText('kpiValue1', sp.length);
                    setElText('kpiSub1', sp.some(t => t.is_active) ? 'Active on blog articles' : 'Using default post template');

                    setElText('kpiLabel2', 'Single page layouts');
                    setElText('kpiValue2', pg.length);
                    setElText('kpiSub2', pg.some(t => t.is_active) ? 'Active on static pages' : 'Using default page template');

                    setElText('kpiLabel3', 'Archive & loop layouts');
                    setElText('kpiValue3', ar.length);
                    setElText('kpiSub3', ar.some(t => t.is_active) ? 'Hydrating LoopGrid feeds' : 'Using standard archive feed');

                    setElText('kpiLabel4', '404 recovery layouts');
                    setElText('kpiValue4', e4.length);
                    setElText('kpiSub4', e4.some(t => t.is_active) ? 'Custom 404 route active' : 'Using built-in 404 screen');
                }} else if (window.tsPageMode === 'symbols') {{
                    let totalSlots = 0;
                    symbols.forEach(s => {{
                        totalSlots += inspectSymbolOverridableSlots(s.layout).length;
                    }});
                    const megaBoundCount = (hfNavMenuItems || []).filter(it => Number(it.mega_menu_template_id) > 0).length;

                    setElText('kpiLabel1', 'Synced global symbols');
                    setElText('kpiValue1', symbols.length);
                    setElText('kpiSub1', 'Single source of truth in SQLite');

                    setElText('kpiLabel2', 'Overridable child slots');
                    setElText('kpiValue2', totalSlots);
                    setElText('kpiSub2', 'Heading, text, button & image props');

                    setElText('kpiLabel3', 'Mega menu panels');
                    setElText('kpiValue3', megaBoundCount);
                    setElText('kpiSub3', `Bound across ${{(hfNavMenuItems || []).length}} nav menu items`);

                    setElText('kpiLabel4', 'Max recursion depth');
                    setElText('kpiValue4', '4 levels');
                    setElText('kpiSub4', 'Cycle-safe SSR symbol expansion');
                }} else if (window.tsPageMode === 'popups') {{
                    const activePopups = popups.filter(t => t.is_active && t.status === 'publish');
                    const bars = popups.filter(t => ['top_bar', 'bottom_bar', 'announcement_bar_top', 'announcement_bar_bottom'].includes((t.settings || {{}}).popup_variant));
                    const modalsAndDrawers = popups.filter(t => !['top_bar', 'bottom_bar', 'announcement_bar_top', 'announcement_bar_bottom'].includes((t.settings || {{}}).popup_variant));

                    setElText('kpiLabel1', 'Active popups & bars');
                    setElText('kpiValue1', `${{activePopups.length}} / ${{popups.length}}`);
                    setElText('kpiSub1', 'Injected into matched public routes');

                    setElText('kpiLabel2', 'Announcement bars');
                    setElText('kpiValue2', bars.length);
                    setElText('kpiSub2', 'Top & bottom viewport bars');

                    setElText('kpiLabel3', 'Modals & slide-in drawers');
                    setElText('kpiValue3', modalsAndDrawers.length);
                    setElText('kpiSub3', 'Scroll, delay & exit-intent triggers');

                    setElText('kpiLabel4', 'Dismissal cooldown');
                    setElText('kpiValue4', 'Active');
                    setElText('kpiSub4', 'Per-template localStorage frequency cap');
                }}
            }}

            // =========================================================================
            // PAGE 1: HEADERS & FOOTERS CARDS
            // =========================================================================
            function renderHeadersFootersCards() {{
                const tpls = hfMatrix.templates || [];
                const headers = tpls.filter(t => t.template_type === 'header');
                const footers = tpls.filter(t => t.template_type === 'footer');

                const hContainer = document.getElementById('headersCardsList');
                if (hContainer) {{
                    hContainer.innerHTML = headers.length === 0
                        ? renderEmptyStateCard('header', 'No custom site headers yet', 'Create a blank Header in DayStudio or install a pre-built Navigation Header starter.')
                        : headers.map(t => renderHeaderFooterCardHtml(t)).join('');
                }}

                const fContainer = document.getElementById('footersCardsList');
                if (fContainer) {{
                    fContainer.innerHTML = footers.length === 0
                        ? renderEmptyStateCard('footer', 'No custom site footers yet', 'Create a multi-column Footer in DayStudio or install a starter blueprint.')
                        : footers.map(t => renderHeaderFooterCardHtml(t)).join('');
                }}
            }}

            function renderEmptyStateCard(type, heading, desc) {{
                return `
                    <div class="day-card" style="padding: 28px; text-align: center; grid-column: 1 / -1; border: 1px dashed var(--rule-strong); background: var(--paper-raised);">
                        <div style="font-size: 15px; font-weight: 600; color: var(--ink); margin-bottom: 4px;">${{escHtml(heading)}}</div>
                        <p style="font-size: 13px; color: var(--ink-muted); margin: 0 auto 16px auto; max-width: 520px;">${{escHtml(desc)}}</p>
                        <div style="display: flex; gap: 10px; justify-content: center;">
                            <button type="button" class="btn btn-primary" onclick="openCreateModal('${{escHtml(type)}}')">Create ${{escHtml(type.replace('_', ' '))}}</button>
                            <button type="button" class="btn btn-secondary" onclick="openStarterCatalogModal('${{escHtml(type)}}')">Browse Starters</button>
                        </div>
                    </div>
                `;
            }}

            function renderHeaderFooterCardHtml(tpl) {{
                const cov = (hfMatrix.coverage && hfMatrix.coverage[tpl.id]) || {{
                    matched_pages_count: 0,
                    matched_pages: [],
                    conditions_summary: 'Entire Site'
                }};
                const isActive = tpl.is_active && tpl.status === 'publish';
                const s = tpl.settings || {{}};
                const primaryTarget = getPrimaryTargetValue(tpl);

                const matchedLinks = (cov.matched_pages || []).slice(0, 5).map(p =>
                    `<a href="${{escHtml(p.permalink)}}" target="_blank" style="font-size:11px;color:var(--ink);background:var(--paper-raised);border:1px solid var(--rule-strong);padding:2px 7px;border-radius:4px;text-decoration:none;font-weight:500;">${{escHtml(p.title)}}</a>`
                ).join('');
                const moreCount = Math.max(0, (cov.matched_pages_count || 0) - 5);
                const moreBadge = moreCount > 0 ? `<span style="font-size:11px;color:var(--ink-muted);font-family:var(--font-mono);">+${{moreCount}} more</span>` : '';

                const inlineBehaviorToggles = tpl.template_type === 'header'
                    ? `
                        <label class="ts-toggle-pill ${{s.sticky ? 'is-on' : ''}}">
                            <input type="checkbox" ${{s.sticky ? 'checked' : ''}} onchange="toggleInlineBehavior(${{tpl.id}}, 'sticky', this.checked)" />
                            <span>Sticky top</span>
                        </label>
                        <label class="ts-toggle-pill ${{s.transparent ? 'is-on' : ''}}">
                            <input type="checkbox" ${{s.transparent ? 'checked' : ''}} onchange="toggleInlineBehavior(${{tpl.id}}, 'transparent', this.checked)" />
                            <span>Transparent overlay</span>
                        </label>
                        <label class="ts-toggle-pill ${{s.scroll_shrink ? 'is-on' : ''}}">
                            <input type="checkbox" ${{s.scroll_shrink ? 'checked' : ''}} onchange="toggleInlineBehavior(${{tpl.id}}, 'scroll_shrink', this.checked)" />
                            <span>Shrink on scroll</span>
                        </label>
                    `
                    : `
                        <label class="ts-toggle-pill ${{s.sticky_footer ? 'is-on' : ''}}">
                            <input type="checkbox" ${{s.sticky_footer ? 'checked' : ''}} onchange="toggleInlineBehavior(${{tpl.id}}, 'sticky_footer', this.checked)" />
                            <span>Sticky bottom</span>
                        </label>
                        <label class="ts-toggle-pill ${{s.reveal_on_scroll ? 'is-on' : ''}}">
                            <input type="checkbox" ${{s.reveal_on_scroll ? 'checked' : ''}} onchange="toggleInlineBehavior(${{tpl.id}}, 'reveal_on_scroll', this.checked)" />
                            <span>Parallax reveal</span>
                        </label>
                    `;

                const schematicWireframe = tpl.template_type === 'header'
                    ? `<div class="ts-schematic">
                        <div style="display:flex;align-items:center;gap:8px;">
                            <span style="width:18px;height:10px;border-radius:2px;background:var(--ink);display:inline-block;"></span>
                            <span style="width:36px;height:6px;border-radius:2px;background:var(--rule-strong);display:inline-block;"></span>
                            <span style="width:36px;height:6px;border-radius:2px;background:var(--rule-strong);display:inline-block;"></span>
                        </div>
                        <span style="font-size:11px;font-family:var(--font-mono);color:var(--ink-muted);">header.dp-theme-header &middot; z:${{s.z_index || 999}}</span>
                    </div>`
                    : `<div class="ts-schematic">
                        <div style="display:flex;align-items:center;gap:6px;">
                            <span style="width:28px;height:12px;border-radius:2px;border:1px solid var(--rule-strong);display:inline-block;"></span>
                            <span style="width:28px;height:12px;border-radius:2px;border:1px solid var(--rule-strong);display:inline-block;"></span>
                            <span style="width:28px;height:12px;border-radius:2px;border:1px solid var(--rule-strong);display:inline-block;"></span>
                        </div>
                        <span style="font-size:11px;font-family:var(--font-mono);color:var(--ink-muted);">footer.dp-theme-footer</span>
                    </div>`;

                return `
                    <div class="hf-tpl-card ${{isActive ? '' : 'is-inactive'}}">
                        <div style="display: flex; justify-content: space-between; align-items: flex-start; gap: 12px;">
                            <div>
                                <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                                    <strong style="font-size: 15px; color: var(--ink);">${{escHtml(tpl.title)}}</strong>
                                    <span style="font-size: 11px; font-family: var(--font-mono); color: var(--ink-muted);">#${{tpl.id}}</span>
                                    <span class="hf-chip ${{isActive ? 'hf-chip-global' : 'hf-chip-fallback'}}">${{isActive ? 'Active' : 'Inactive'}}</span>
                                </div>
                                <div style="font-size: 11.5px; color: var(--ink-muted); margin-top: 3px; font-family: var(--font-mono);">
                                    Priority ${{tpl.priority}} &middot; Updated ${{escHtml((tpl.updated_at || '').slice(0, 16))}}
                                </div>
                            </div>
                            <label style="display: flex; align-items: center; gap: 6px; font-size: 12px; font-weight: 600; cursor: pointer; user-select: none;">
                                <input type="checkbox" ${{isActive ? 'checked' : ''}} onchange="toggleTemplateActive(${{tpl.id}}, this.checked)" />
                                <span>Enabled</span>
                            </label>
                        </div>

                        ${{schematicWireframe}}

                        <!-- Inline Behavior Toggles -->
                        <div style="display: flex; gap: 8px; flex-wrap: wrap; align-items: center;">
                            ${{inlineBehaviorToggles}}
                        </div>

                        <!-- Route Target & Page Coverage Box -->
                        <div class="ts-inline-controls">
                            <div class="ts-inline-row">
                                <span style="font-size: 11.5px; font-weight: 600; color: var(--ink-muted);">Target Scope</span>
                                <div style="display: flex; gap: 6px; align-items: center;">
                                    <select class="form-select" style="font-size: 12px; padding: 3px 8px; height: 28px;" onchange="quickAssignPrimaryCondition(${{tpl.id}}, this.value)">
                                        <option value="entire_site" ${{primaryTarget === 'entire_site' ? 'selected' : ''}}>Entire Site (Global Default)</option>
                                        <option value="front_page" ${{primaryTarget === 'front_page' ? 'selected' : ''}}>Front Page Only</option>
                                        <option value="all_pages" ${{primaryTarget === 'all_pages' ? 'selected' : ''}}>All Static Pages</option>
                                        <option value="all_posts" ${{primaryTarget === 'all_posts' ? 'selected' : ''}}>All Blog Posts</option>
                                        <option value="unassigned" ${{primaryTarget === 'unassigned' ? 'selected' : ''}}>Unassigned (Pin per page)</option>
                                        ${{primaryTarget === 'custom' ? '<option value="custom" selected>Custom Page / Rule Set</option>' : ''}}
                                    </select>
                                    <button type="button" class="btn btn-secondary" style="font-size: 11.5px; padding: 3px 8px;" onclick="openConditionsModal(${{tpl.id}})">Rules (${{(tpl.conditions || []).length}})</button>
                                </div>
                            </div>
                            <div style="font-size: 12px; color: var(--ink); font-weight: 500;">
                                ${{escHtml(cov.conditions_summary || 'Entire Site')}}
                            </div>
                            <div style="display: flex; gap: 6px; flex-wrap: wrap; align-items: center;">
                                ${{matchedLinks || '<span style="font-size:11.5px;color:var(--ink-muted);">No published pages currently match this rule</span>'}}
                                ${{moreBadge}}
                            </div>
                        </div>

                        <!-- Actions Footer -->
                        <div style="display: flex; justify-content: space-between; align-items: center; gap: 8px; padding-top: 6px; border-top: 1px solid var(--rule); flex-wrap: wrap;">
                            <div style="display: flex; gap: 6px; flex-wrap: wrap;">
                                <a href="/d-admin/builder?template_id=${{tpl.id}}" class="btn btn-primary" style="font-size: 12px; padding: 5px 12px;">Open in DayStudio &nearr;</a>
                                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openPreviewModal(${{tpl.id}}, '${{escHtml(tpl.title)}}')">Preview</button>
                                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openSettingsModal(${{tpl.id}})">Settings</button>
                            </div>
                            <div style="display: flex; gap: 6px;">
                                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 9px;" onclick="duplicateTemplate(${{tpl.id}})">Duplicate</button>
                                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 9px; color: var(--oxide);" onclick="deleteTemplate(${{tpl.id}}, '${{escHtml(tpl.title)}}')">Delete</button>
                            </div>
                        </div>
                    </div>
                `;
            }}

            async function toggleInlineBehavior(templateId, key, checked) {{
                const tpl = (hfMatrix.templates || []).find(t => t.id === templateId);
                if (!tpl) return;
                const settings = Object.assign({{}}, tpl.settings || {{}}, {{ [key]: !!checked }});
                try {{
                    const res = await fetch(`/wp-json/daypress/v1/templates/${{templateId}}`, {{
                        method: 'PUT',
                        headers: {{ 'Content-Type': 'application/json' }},
                        credentials: 'same-origin',
                        body: JSON.stringify({{ settings }})
                    }});
                    if (res.ok) {{
                        await refreshMatrixFromServer();
                        showToast(`Updated ${{key.replace(/_/g, ' ')}} on "${{tpl.title}}"`);
                    }} else {{
                        showToast('Failed to update behavior', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            async function quickAssignPrimaryCondition(templateId, target) {{
                if (target === 'custom') {{
                    openConditionsModal(templateId);
                    return;
                }}
                let conditions = [];
                if (target === 'unassigned') {{
                    conditions = [{{ condition_type: 'exclude', rule_target: 'entire_site', target_id: '', priority: 10 }}];
                }} else {{
                    const priority = (target === 'front_page' || target === 'not_found') ? 50 : (target === 'entire_site' ? 10 : 20);
                    conditions = [{{ condition_type: 'include', rule_target: target, target_id: '', priority }}];
                }}
                try {{
                    const res = await fetch(`/wp-json/daypress/v1/templates/${{templateId}}/conditions`, {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        credentials: 'same-origin',
                        body: JSON.stringify({{ conditions }})
                    }});
                    if (res.ok) {{
                        await refreshMatrixFromServer();
                        showToast('Target scope saved to SQLite');
                    }} else {{
                        showToast('Failed to update target scope', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            // =========================================================================
            // REDESIGNED USER-FRIENDLY PAGE & ROUTE ASSIGNMENT MATRIX
            // =========================================================================
            function setMatrixFilter(filter) {{
                currentMatrixFilter = filter;
                ['All', 'Routes', 'Pages', 'Posts', 'Overrides'].forEach(k => {{
                    const btn = document.getElementById('matFilter' + k);
                    if (btn) btn.classList.toggle('active', k.toLowerCase() === filter);
                }});
                renderMatrixView();
            }}

            async function onGlobalDefaultSlotChange(slot, val) {{
                const tpls = (hfMatrix.templates || []).filter(t => t.template_type === slot);
                try {{
                    if (val === 'fallback') {{
                        // Remove entire_site include from any active template of this slot
                        for (const t of tpls) {{
                            const conds = (t.conditions || []).filter(c => !(c.condition_type === 'include' && c.rule_target === 'entire_site'));
                            const nextConds = conds.length > 0 ? conds : [{{ condition_type: 'exclude', rule_target: 'entire_site', target_id: '', priority: 10 }}];
                            await fetch(`/wp-json/daypress/v1/templates/${{t.id}}/conditions`, {{
                                method: 'POST',
                                headers: {{ 'Content-Type': 'application/json' }},
                                credentials: 'same-origin',
                                body: JSON.stringify({{ conditions: nextConds }})
                            }});
                        }}
                        await refreshMatrixFromServer();
                        showToast(`Global default ${{slot}} set to Built-in Theme Fallback`);
                        return;
                    }}

                    const targetId = parseInt(val, 10);
                    if (!targetId) return;

                    // Ensure chosen template is active & has entire_site include, and demote other entire_site defaults
                    for (const t of tpls) {{
                        if (t.id === targetId) {{
                            await fetch(`/wp-json/daypress/v1/templates/${{t.id}}`, {{
                                method: 'PUT',
                                headers: {{ 'Content-Type': 'application/json' }},
                                credentials: 'same-origin',
                                body: JSON.stringify({{ is_active: true, status: 'publish' }})
                            }});
                            const otherConds = (t.conditions || []).filter(c => !(c.rule_target === 'entire_site'));
                            otherConds.unshift({{ condition_type: 'include', rule_target: 'entire_site', target_id: '', priority: 10 }});
                            await fetch(`/wp-json/daypress/v1/templates/${{t.id}}/conditions`, {{
                                method: 'POST',
                                headers: {{ 'Content-Type': 'application/json' }},
                                credentials: 'same-origin',
                                body: JSON.stringify({{ conditions: otherConds }})
                            }});
                        }} else {{
                            const hadEntireSite = (t.conditions || []).some(c => c.condition_type === 'include' && c.rule_target === 'entire_site');
                            if (hadEntireSite) {{
                                const rem = (t.conditions || []).filter(c => !(c.condition_type === 'include' && c.rule_target === 'entire_site'));
                                const nextConds = rem.length > 0 ? rem : [{{ condition_type: 'exclude', rule_target: 'entire_site', target_id: '', priority: 10 }}];
                                await fetch(`/wp-json/daypress/v1/templates/${{t.id}}/conditions`, {{
                                    method: 'POST',
                                    headers: {{ 'Content-Type': 'application/json' }},
                                    credentials: 'same-origin',
                                    body: JSON.stringify({{ conditions: nextConds }})
                                }});
                            }}
                        }}
                    }}
                    await refreshMatrixFromServer();
                    showToast(`Updated Global Default ${{slot}} across the site`);
                }} catch (e) {{
                    showToast('Failed to update global default: ' + e.message, true);
                }}
            }}

            function resolveBodyLayoutBadgeForTarget(rowKind, row) {{
                const layouts = (hfMatrix.templates || []).filter(t => t.is_active && t.status === 'publish');
                let targetType = 'single_page';
                if (rowKind === 'route') {{
                    if (row.route_key === 'all_posts') targetType = 'single_post';
                    else if (row.route_key === 'archive') targetType = 'archive';
                    else if (row.route_key === 'not_found') targetType = '404_page';
                    else targetType = 'single_page';
                }} else if (row.post_type === 'post') {{
                    targetType = 'single_post';
                }}
                const matched = layouts.find(t => t.template_type === targetType && !(t.conditions || []).some(c => c.condition_type === 'exclude' && c.rule_target === 'entire_site'));
                if (matched) {{
                    return `<a href="/d-admin/builder?template_id=${{matched.id}}" class="hf-chip hf-chip-custom" style="text-decoration:none;" title="Edit ${{escHtml(matched.title)}} in DayStudio">${{escHtml(matched.title)}} &nearr;</a>`;
                }}
                return `<a href="/d-admin/theme-layouts" class="hf-chip hf-chip-fallback" style="text-decoration:none;" title="Configure in Theme Layouts">Standard Theme</a>`;
            }}

            function renderUnifiedSlotCell(targetKind, targetId, slot, resolved, slotTemplates, globalDefaultTpl) {{
                const mode = (resolved && resolved.mode) || 'inherit';
                const isOverridden = mode !== 'inherit';
                const boxClass = mode.startsWith('template:')
                    ? 'slot-custom'
                    : (mode === 'none' ? 'slot-hidden' : '');

                let pillClass = 'hf-chip-global';
                let pillText = 'Inherited';
                if (mode.startsWith('template:')) {{
                    pillClass = 'hf-chip-custom';
                    pillText = 'Pinned Override';
                }} else if (mode === 'none') {{
                    pillClass = 'hf-chip-hidden';
                    pillText = 'Hidden';
                }} else if (mode === 'fallback' || (resolved && resolved.badge_kind === 'fallback')) {{
                    pillClass = 'hf-chip-fallback';
                    pillText = mode === 'fallback' ? 'Pinned Fallback' : 'Theme Fallback';
                }}

                const globalName = globalDefaultTpl ? globalDefaultTpl.title : `Built-in Theme ${{slot === 'header' ? 'Header' : 'Footer'}}`;
                const editBtn = (resolved && resolved.template_id)
                    ? `<a href="/d-admin/builder?template_id=${{resolved.template_id}}" style="font-size:11px;font-weight:600;color:var(--press);text-decoration:none;">Edit &nearr;</a>`
                    : '';
                const resetBtn = isOverridden
                    ? `<button type="button" class="hf-slot-reset-btn" onclick="assignMatrixTargetSlot('${{targetKind}}', '${{targetId}}', '${{slot}}', 'inherit')" title="Reset override back to Inherit Global">Reset</button>`
                    : '';

                const tplOptionsHtml = slotTemplates.map(t => {{
                    const val = `template:${{t.id}}`;
                    const sel = mode === val ? 'selected' : '';
                    return `<option value="${{val}}" ${{sel}}>Pinned: ${{escHtml(t.title)}} (#${{t.id}})</option>`;
                }}).join('');

                return `
                    <div class="hf-slot-box ${{boxClass}}">
                        <div class="hf-slot-top">
                            <div style="display:flex;align-items:center;gap:6px;min-width:0;overflow:hidden;">
                                <span class="hf-chip ${{pillClass}}">${{pillText}}</span>
                                <span style="font-size:11px;color:var(--ink-muted);white-space:nowrap;overflow:hidden;text-overflow:ellipsis;" title="${{escHtml((resolved && resolved.reason) || '')}}">
                                    ${{escHtml((resolved && resolved.reason) || 'Global rule')}}
                                </span>
                            </div>
                            <div style="display:flex;align-items:center;gap:6px;flex-shrink:0;">
                                ${{editBtn}}
                                ${{resetBtn}}
                            </div>
                        </div>
                        <select class="hf-slot-select" onchange="assignMatrixTargetSlot('${{targetKind}}', '${{targetId}}', '${{slot}}', this.value)">
                            <option value="inherit" ${{mode === 'inherit' ? 'selected' : ''}}>Inherit Global (&rarr; ${{escHtml(globalName)}})</option>
                            <optgroup label="Pin Specific ${{slot === 'header' ? 'Header' : 'Footer'}}">
                                ${{tplOptionsHtml}}
                            </optgroup>
                            <optgroup label="Special Slot Modes">
                                <option value="fallback" ${{mode === 'fallback' ? 'selected' : ''}}>Built-in Theme ${{slot === 'header' ? 'Header' : 'Footer'}}</option>
                                <option value="none" ${{mode === 'none' ? 'selected' : ''}}>Hide ${{slot === 'header' ? 'Header' : 'Footer'}} on this target</option>
                            </optgroup>
                        </select>
                    </div>
                `;
            }}

            function renderMatrixView() {{
                const tbody = document.getElementById('unifiedMatrixTbody');
                if (!tbody) return;

                const tpls = hfMatrix.templates || [];
                const headers = tpls.filter(t => t.template_type === 'header');
                const footers = tpls.filter(t => t.template_type === 'footer');
                const layouts = tpls.filter(t => ['single_post', 'single_page', 'archive', '404_page'].includes(t.template_type));

                const globalHdr = getGlobalDefaultTemplate('header');
                const globalFtr = getGlobalDefaultTemplate('footer');

                // Populate Top Site Shell Blueprint Bar
                const gHdrSel = document.getElementById('globalDefaultHeaderSelect');
                if (gHdrSel) {{
                    gHdrSel.innerHTML = `<option value="fallback" ${{!globalHdr ? 'selected' : ''}}>Built-in Theme Header (Fallback)</option>` +
                        headers.map(h => `<option value="${{h.id}}" ${{globalHdr && globalHdr.id === h.id ? 'selected' : ''}}>${{escHtml(h.title)}} (#${{h.id}})</option>`).join('');
                }}
                const editGHdr = document.getElementById('bpEditGlobalHeader');
                if (editGHdr) {{
                    if (globalHdr) {{
                        editGHdr.style.display = 'inline';
                        editGHdr.href = `/d-admin/builder?template_id=${{globalHdr.id}}`;
                    }} else {{
                        editGHdr.style.display = 'none';
                    }}
                }}

                const gFtrSel = document.getElementById('globalDefaultFooterSelect');
                if (gFtrSel) {{
                    gFtrSel.innerHTML = `<option value="fallback" ${{!globalFtr ? 'selected' : ''}}>Built-in Theme Footer (Fallback)</option>` +
                        footers.map(f => `<option value="${{f.id}}" ${{globalFtr && globalFtr.id === f.id ? 'selected' : ''}}>${{escHtml(f.title)}} (#${{f.id}})</option>`).join('');
                }}
                const editGFtr = document.getElementById('bpEditGlobalFooter');
                if (editGFtr) {{
                    if (globalFtr) {{
                        editGFtr.style.display = 'inline';
                        editGFtr.href = `/d-admin/builder?template_id=${{globalFtr.id}}`;
                    }} else {{
                        editGFtr.style.display = 'none';
                    }}
                }}

                const bpLayouts = document.getElementById('bpActiveLayoutsSummary');
                if (bpLayouts) {{
                    const activeLayouts = layouts.filter(l => l.is_active && l.status === 'publish');
                    bpLayouts.innerHTML = activeLayouts.length === 0
                        ? `<span class="hf-chip hf-chip-fallback">Standard Theme Templates</span>`
                        : activeLayouts.map(l => `<a href="/d-admin/builder?template_id=${{l.id}}" class="hf-chip hf-chip-custom" style="text-decoration:none;">${{escHtml(l.title)}}</a>`).join('');
                }}

                // Populate Bulk Bar Dropdowns
                const bulkH = document.getElementById('bulkHeaderSelect');
                if (bulkH && !bulkH.dataset.ready) {{
                    bulkH.innerHTML = `<option value="">Header: No change</option><option value="inherit">Header: Inherit Global</option>` +
                        headers.map(h => `<option value="template:${{h.id}}">Header: ${{escHtml(h.title)}}</option>`).join('') +
                        `<option value="fallback">Header: Built-in Fallback</option><option value="none">Header: Hide Header</option>`;
                }}
                const bulkF = document.getElementById('bulkFooterSelect');
                if (bulkF && !bulkF.dataset.ready) {{
                    bulkF.innerHTML = `<option value="">Footer: No change</option><option value="inherit">Footer: Inherit Global</option>` +
                        footers.map(f => `<option value="template:${{f.id}}">Footer: ${{escHtml(f.title)}}</option>`).join('') +
                        `<option value="fallback">Footer: Built-in Fallback</option><option value="none">Footer: Hide Footer</option>`;
                }}

                // Build Unified Target Rows
                const routes = (hfMatrix.routes || []).map(r => ({{
                    rowKey: `route:${{r.route_key}}`,
                    targetKind: 'route',
                    targetId: r.route_key,
                    title: r.label,
                    permalink: r.route_pattern,
                    header: r.header,
                    footer: r.footer,
                    raw: r,
                    isOverridden: (r.header && r.header.mode !== 'inherit') || (r.footer && r.footer.mode !== 'inherit')
                }}));
                const pages = (hfMatrix.pages || []).map(p => ({{
                    rowKey: `post:${{p.post_id}}`,
                    targetKind: 'page',
                    targetId: String(p.post_id),
                    title: p.title,
                    permalink: p.permalink,
                    header: p.header,
                    footer: p.footer,
                    raw: p,
                    isOverridden: (p.header && p.header.mode !== 'inherit') || (p.footer && p.footer.mode !== 'inherit')
                }}));
                const posts = (hfMatrix.posts || []).map(p => ({{
                    rowKey: `post:${{p.post_id}}`,
                    targetKind: 'post',
                    targetId: String(p.post_id),
                    title: p.title,
                    permalink: p.permalink,
                    header: p.header,
                    footer: p.footer,
                    raw: p,
                    isOverridden: (p.header && p.header.mode !== 'inherit') || (p.footer && p.footer.mode !== 'inherit')
                }}));

                const allRows = [...routes, ...pages, ...posts];
                const overridesRows = allRows.filter(r => r.isOverridden);

                const setCnt = (id, n) => {{
                    const el = document.getElementById(id);
                    if (el) el.textContent = String(n);
                }};
                setCnt('matCountAll', allRows.length);
                setCnt('matCountRoutes', routes.length);
                setCnt('matCountPages', pages.length);
                setCnt('matCountPosts', posts.length);
                setCnt('matCountOverrides', overridesRows.length);

                const resetAllBtn = document.getElementById('matResetAllBtn');
                if (resetAllBtn) {{
                    resetAllBtn.style.display = overridesRows.length > 0 ? 'inline-flex' : 'none';
                    resetAllBtn.textContent = `Reset All Overrides (${{overridesRows.length}})`;
                }}

                let filtered = allRows;
                if (currentMatrixFilter === 'routes') filtered = routes;
                else if (currentMatrixFilter === 'pages') filtered = pages;
                else if (currentMatrixFilter === 'posts') filtered = posts;
                else if (currentMatrixFilter === 'overrides') filtered = overridesRows;

                const q = (document.getElementById('matrixPageSearch')?.value || '').trim().toLowerCase();
                if (q) {{
                    filtered = filtered.filter(r =>
                        (r.title || '').toLowerCase().includes(q) ||
                        (r.permalink || '').toLowerCase().includes(q)
                    );
                }}

                if (filtered.length === 0) {{
                    tbody.innerHTML = `<tr><td colspan="5" style="text-align:center;padding:28px;color:var(--ink-muted);">No routes or pages match the current filter.</td></tr>`;
                    updateMatrixBulkBarUI();
                    return;
                }}

                tbody.innerHTML = filtered.map(r => {{
                    const isChecked = selectedMatrixKeys.has(r.rowKey);
                    let metaBadges = '';
                    if (r.targetKind === 'route') {{
                        metaBadges += `<span class="hf-chip hf-chip-behavior">System Route</span>`;
                    }} else {{
                        if (r.raw.is_front_page) metaBadges += `<span class="hf-chip hf-chip-custom">Front Page</span>`;
                        if (r.raw.in_nav_menu) metaBadges += `<span class="hf-chip hf-chip-global">Nav Menu</span>`;
                        if (r.targetKind === 'post') metaBadges += `<span class="hf-chip hf-chip-fallback">Post</span>`;
                        if (r.raw.status !== 'publish') metaBadges += `<span class="hf-chip hf-chip-fallback">${{escHtml(r.raw.status)}}</span>`;
                    }}

                    const quickLinks = r.targetKind === 'route'
                        ? `<span style="font-size:11.5px;color:var(--ink-muted);font-family:var(--font-mono);">${{escHtml(r.permalink)}}</span>`
                        : `<div style="display:flex;align-items:center;gap:8px;margin-top:3px;">
                            <a href="${{escHtml(r.permalink)}}" target="_blank" style="font-size:11.5px;color:var(--ink-muted);font-family:var(--font-mono);text-decoration:none;">${{escHtml(r.permalink)}} &nearr;</a>
                            <a href="/d-admin/post?post=${{r.raw.post_id}}&action=edit" style="font-size:11px;color:var(--press);text-decoration:none;font-weight:600;">Edit Page</a>
                        </div>`;

                    return `
                        <tr class="${{r.isOverridden ? 'is-overridden-row' : ''}}">
                            <td style="text-align: center; vertical-align: middle;">
                                <input type="checkbox" class="matrix-row-chk" data-row-key="${{escHtml(r.rowKey)}}" ${{isChecked ? 'checked' : ''}} onchange="toggleMatrixRowSelection('${{escHtml(r.rowKey)}}', this.checked)" />
                            </td>
                            <td>
                                <div style="display:flex;align-items:center;gap:6px;flex-wrap:wrap;">
                                    <strong style="color:var(--ink);font-size:13.5px;">${{escHtml(r.title)}}</strong>
                                    ${{metaBadges}}
                                </div>
                                ${{quickLinks}}
                            </td>
                            <td>${{renderUnifiedSlotCell(r.targetKind, r.targetId, 'header', r.header, headers, globalHdr)}}</td>
                            <td>${{renderUnifiedSlotCell(r.targetKind, r.targetId, 'footer', r.footer, footers, globalFtr)}}</td>
                            <td style="vertical-align: middle;">${{resolveBodyLayoutBadgeForTarget(r.targetKind, r.raw)}}</td>
                        </tr>
                    `;
                }}).join('');

                updateMatrixBulkBarUI();
            }}

            function toggleMatrixRowSelection(rowKey, checked) {{
                if (checked) selectedMatrixKeys.add(rowKey);
                else selectedMatrixKeys.delete(rowKey);
                updateMatrixBulkBarUI();
            }}

            function toggleSelectAllMatrixRows(checked) {{
                document.querySelectorAll('#unifiedMatrixTbody .matrix-row-chk').forEach(chk => {{
                    chk.checked = checked;
                    const k = chk.getAttribute('data-row-key');
                    if (k) {{
                        if (checked) selectedMatrixKeys.add(k);
                        else selectedMatrixKeys.delete(k);
                    }}
                }});
                updateMatrixBulkBarUI();
            }}

            function clearMatrixSelection() {{
                selectedMatrixKeys.clear();
                const allChk = document.getElementById('matrixSelectAllChk');
                if (allChk) allChk.checked = false;
                document.querySelectorAll('#unifiedMatrixTbody .matrix-row-chk').forEach(chk => chk.checked = false);
                updateMatrixBulkBarUI();
            }}

            function updateMatrixBulkBarUI() {{
                const bar = document.getElementById('matrixBulkBar');
                const lbl = document.getElementById('matrixBulkCountLabel');
                if (!bar || !lbl) return;
                const count = selectedMatrixKeys.size;
                bar.style.display = count > 0 ? 'flex' : 'none';
                lbl.textContent = `${{count}} ${{count === 1 ? 'target' : 'targets'}} selected`;
            }}

            async function assignMatrixTargetSlot(targetKind, targetId, slot, value) {{
                const payload = {{ [slot]: value }};
                if (targetKind === 'route') {{
                    payload.route_key = String(targetId);
                }} else {{
                    payload.post_id = parseInt(targetId, 10);
                }}
                try {{
                    const res = await fetch('/wp-json/daypress/v1/templates/assign-page', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        credentials: 'same-origin',
                        body: JSON.stringify(payload)
                    }});
                    if (res.ok) {{
                        hfMatrix = await res.json();
                        renderAllViews();
                        showToast(`Saved ${{slot}} assignment in SQLite`);
                    }} else {{
                        showToast('Failed to assign template slot', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            async function applyBulkMatrixAssignment() {{
                const hVal = document.getElementById('bulkHeaderSelect')?.value || '';
                const fVal = document.getElementById('bulkFooterSelect')?.value || '';
                if (!hVal && !fVal) {{
                    showToast('Select a Header or Footer mode to apply', true);
                    return;
                }}
                const keys = Array.from(selectedMatrixKeys);
                if (keys.length === 0) return;

                try {{
                    for (const rk of keys) {{
                        const [kind, id] = rk.split(':');
                        const payload = {{}};
                        if (kind === 'route') payload.route_key = id;
                        else payload.post_id = parseInt(id, 10);
                        if (hVal) payload.header = hVal;
                        if (fVal) payload.footer = fVal;
                        await fetch('/wp-json/daypress/v1/templates/assign-page', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            credentials: 'same-origin',
                            body: JSON.stringify(payload)
                        }});
                    }}
                    clearMatrixSelection();
                    await refreshMatrixFromServer();
                    showToast(`Applied assignment to ${{keys.length}} targets in SQLite`);
                }} catch (e) {{
                    showToast('Bulk assignment failed: ' + e.message, true);
                }}
            }}

            async function resetAllMatrixOverrides() {{
                const routes = hfMatrix.routes || [];
                const pages = hfMatrix.pages || [];
                const posts = hfMatrix.posts || [];
                const tasks = [];

                routes.forEach(r => {{
                    if ((r.header && r.header.mode !== 'inherit') || (r.footer && r.footer.mode !== 'inherit')) {{
                        tasks.push({{ route_key: r.route_key, header: 'inherit', footer: 'inherit' }});
                    }}
                }});
                [...pages, ...posts].forEach(p => {{
                    if ((p.header && p.header.mode !== 'inherit') || (p.footer && p.footer.mode !== 'inherit')) {{
                        tasks.push({{ post_id: p.post_id, header: 'inherit', footer: 'inherit' }});
                    }}
                }});

                if (tasks.length === 0) return;
                try {{
                    for (const payload of tasks) {{
                        await fetch('/wp-json/daypress/v1/templates/assign-page', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            credentials: 'same-origin',
                            body: JSON.stringify(payload)
                        }});
                    }}
                    await refreshMatrixFromServer();
                    showToast(`Reset ${{tasks.length}} overrides to Inherit Global`);
                }} catch (e) {{
                    showToast('Failed to reset overrides: ' + e.message, true);
                }}
            }}

            // =========================================================================
            // PAGE 2: THEME LAYOUTS (Single Post, Single Page, Archive, 404)
            // =========================================================================
            function setLayoutFilter(filter) {{
                currentLayoutFilter = filter;
                const map = {{
                    all: 'layoutFilterAll',
                    single_post: 'layoutFilterSinglePost',
                    single_page: 'layoutFilterSinglePage',
                    archive: 'layoutFilterArchive',
                    '404_page': 'layoutFilter404'
                }};
                Object.entries(map).forEach(([k, id]) => {{
                    const el = document.getElementById(id);
                    if (el) el.classList.toggle('active', k === filter);
                }});
                renderThemeLayoutsCards();
            }}

            function inspectLayoutDynamicBindings(layout) {{
                const tokens = new Set();
                const widgets = new Set();
                const jsonStr = JSON.stringify(layout || {{}});
                const matches = jsonStr.match(/\{{\{{[a-zA-Z0-9_.]+\}}\}}/g) || [];
                matches.forEach(m => tokens.add(m));

                function walk(node) {{
                    if (!node || typeof node !== 'object') return;
                    const t = node.type || '';
                    if (['PostContent', 'LoopGrid', 'Breadcrumbs', 'SymbolInstance', 'NativeForm'].includes(t)) {{
                        widgets.add(t);
                    }}
                    if (Array.isArray(node.children)) node.children.forEach(walk);
                }}
                if (layout && layout.root) walk(layout.root);
                return {{
                    tokens: Array.from(tokens),
                    widgets: Array.from(widgets),
                    childCount: (layout && layout.root && Array.isArray(layout.root.children)) ? layout.root.children.length : 0
                }};
            }}

            function renderThemeLayoutsCards() {{
                const container = document.getElementById('layoutsCardsList');
                if (!container) return;

                const allLayouts = (hfMatrix.templates || []).filter(t =>
                    ['single_post', 'single_page', 'archive', '404_page'].includes(t.template_type)
                );

                const setCnt = (id, n) => {{
                    const el = document.getElementById(id);
                    if (el) el.textContent = String(n);
                }};
                setCnt('layoutCountAll', allLayouts.length);
                setCnt('layoutCountSinglePost', allLayouts.filter(t => t.template_type === 'single_post').length);
                setCnt('layoutCountSinglePage', allLayouts.filter(t => t.template_type === 'single_page').length);
                setCnt('layoutCountArchive', allLayouts.filter(t => t.template_type === 'archive').length);
                setCnt('layoutCount404', allLayouts.filter(t => t.template_type === '404_page').length);

                const filtered = currentLayoutFilter === 'all'
                    ? allLayouts
                    : allLayouts.filter(t => t.template_type === currentLayoutFilter);

                if (filtered.length === 0) {{
                    const createType = currentLayoutFilter === 'all' ? 'single_post' : currentLayoutFilter;
                    container.innerHTML = `
                        <div class="day-card" style="padding: 32px; text-align: center; grid-column: 1 / -1; border: 1px dashed var(--rule-strong); background: var(--paper-raised);">
                            <div style="font-size: 15px; font-weight: 600; color: var(--ink); margin-bottom: 6px;">No theme layouts in this view yet</div>
                            <p style="font-size: 13px; color: var(--ink-muted); margin: 0 auto 18px auto; max-width: 540px;">
                                Create a Single Post, Single Page, Category Archive, or 404 Error Layout, or install one of the pre-built editorial starters in 1 click.
                            </p>
                            <div style="display: flex; gap: 8px; justify-content: center; flex-wrap: wrap;">
                                <button type="button" class="btn btn-primary" onclick="openCreateModal('${{escHtml(createType)}}')">Create Layout</button>
                                <button type="button" class="btn btn-secondary" onclick="installStarterNow('editorial-single-post', 'Editorial Single Post Layout')">Install Editorial Post Starter</button>
                                <button type="button" class="btn btn-secondary" onclick="installStarterNow('magazine-archive-grid', 'Magazine Category Archive')">Install Archive Grid Starter</button>
                                <button type="button" class="btn btn-secondary" onclick="installStarterNow('modern-error-404', 'Minimal 404 Recovery Page')">Install 404 Starter</button>
                            </div>
                        </div>
                    `;
                    return;
                }}

                const typeLabels = {{
                    single_post: 'Single Post Layout',
                    single_page: 'Single Page Layout',
                    archive: 'Category & Archive Layout',
                    '404_page': '404 Recovery Layout'
                }};

                container.innerHTML = filtered.map(tpl => {{
                    const isActive = tpl.is_active && tpl.status === 'publish';
                    const primaryTarget = getPrimaryTargetValue(tpl);
                    const info = inspectLayoutDynamicBindings(tpl.layout);
                    const cov = (hfMatrix.coverage && hfMatrix.coverage[tpl.id]) || {{ conditions_summary: 'Default Route' }};

                    const bindingChips = [
                        ...info.widgets.map(w => `<span class="hf-chip hf-chip-custom">${{escHtml(w)}}</span>`),
                        ...info.tokens.map(tk => `<code class="hf-chip-token">${{escHtml(tk)}}</code>`)
                    ].join('');

                    const emptySkeletonWarning = info.childCount === 0
                        ? `<div style="display:flex;align-items:center;justify-content:space-between;gap:8px;padding:8px 10px;background:#fffbeb;border:1px solid #fde68a;border-radius:6px;font-size:12px;color:#92400e;">
                            <span>Canvas has 0 child blocks. Inject a dynamic starter skeleton?</span>
                            <button type="button" class="btn btn-secondary" style="font-size:11px;padding:3px 8px;" onclick="injectDynamicLayoutSkeleton(${{tpl.id}}, '${{escHtml(tpl.template_type)}}')">Populate Skeleton</button>
                        </div>`
                        : '';

                    return `
                        <div class="hf-tpl-card ${{isActive ? '' : 'is-inactive'}}">
                            <div style="display: flex; justify-content: space-between; align-items: flex-start; gap: 12px;">
                                <div>
                                    <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                                        <strong style="font-size: 15px; color: var(--ink);">${{escHtml(tpl.title)}}</strong>
                                        <span class="hf-chip hf-chip-custom">${{escHtml(typeLabels[tpl.template_type] || tpl.template_type)}}</span>
                                        <span class="hf-chip ${{isActive ? 'hf-chip-global' : 'hf-chip-fallback'}}">${{isActive ? 'Active' : 'Inactive'}}</span>
                                    </div>
                                    <div style="font-size: 11.5px; color: var(--ink-muted); margin-top: 3px; font-family: var(--font-mono);">
                                        #${{tpl.id}} &middot; Priority ${{tpl.priority}} &middot; ${{info.childCount}} root blocks
                                    </div>
                                </div>
                                <label style="display: flex; align-items: center; gap: 6px; font-size: 12px; font-weight: 600; cursor: pointer;">
                                    <input type="checkbox" ${{isActive ? 'checked' : ''}} onchange="toggleTemplateActive(${{tpl.id}}, this.checked)" />
                                    <span>Enabled</span>
                                </label>
                            </div>

                            ${{emptySkeletonWarning}}

                            <!-- Dynamic Data Bindings Inspector -->
                            <div class="ts-inline-controls">
                                <div class="ts-inline-row">
                                    <span style="font-size: 11.5px; font-weight: 600; color: var(--ink-muted);">Hydrated Dynamic Tokens &amp; AST Nodes</span>
                                    <span style="font-size: 11px; font-family: var(--font-mono); color: var(--ink-muted);">${{info.tokens.length + info.widgets.length}} bindings</span>
                                </div>
                                <div style="display: flex; gap: 6px; flex-wrap: wrap; align-items: center;">
                                    ${{bindingChips || '<span style="font-size:11.5px;color:var(--ink-muted);">Static AST layout (add {{post.title}}, {{post.content}}, or LoopGrid in DayStudio)</span>'}}
                                </div>
                            </div>

                            <!-- Route Assignment -->
                            <div class="ts-inline-controls">
                                <div class="ts-inline-row">
                                    <span style="font-size: 11.5px; font-weight: 600; color: var(--ink-muted);">Assigned Route Scope</span>
                                    <div style="display: flex; gap: 6px; align-items: center;">
                                        <select class="form-select" style="font-size: 12px; padding: 3px 8px; height: 28px;" onchange="quickAssignPrimaryCondition(${{tpl.id}}, this.value)">
                                            <option value="all_posts" ${{primaryTarget === 'all_posts' ? 'selected' : ''}}>All Blog Posts</option>
                                            <option value="all_pages" ${{primaryTarget === 'all_pages' ? 'selected' : ''}}>All Static Pages</option>
                                            <option value="front_page" ${{primaryTarget === 'front_page' ? 'selected' : ''}}>Front Page Only</option>
                                            <option value="archive" ${{primaryTarget === 'archive' ? 'selected' : ''}}>All Category &amp; Author Archives</option>
                                            <option value="not_found" ${{primaryTarget === 'not_found' ? 'selected' : ''}}>404 Not Found Route</option>
                                            <option value="entire_site" ${{primaryTarget === 'entire_site' ? 'selected' : ''}}>Entire Site</option>
                                            <option value="unassigned" ${{primaryTarget === 'unassigned' ? 'selected' : ''}}>Unassigned Draft</option>
                                            ${{primaryTarget === 'custom' ? '<option value="custom" selected>Specific Page / Category Rule</option>' : ''}}
                                        </select>
                                        <button type="button" class="btn btn-secondary" style="font-size: 11.5px; padding: 3px 8px;" onclick="openConditionsModal(${{tpl.id}})">Conditions</button>
                                    </div>
                                </div>
                                <div style="font-size: 12px; color: var(--ink);">${{escHtml(cov.conditions_summary || '')}}</div>
                            </div>

                            <div style="display: flex; justify-content: space-between; align-items: center; gap: 8px; padding-top: 6px; border-top: 1px solid var(--rule); flex-wrap: wrap;">
                                <div style="display: flex; gap: 6px;">
                                    <a href="/d-admin/builder?template_id=${{tpl.id}}" class="btn btn-primary" style="font-size: 12px; padding: 5px 12px;">Open in DayStudio &nearr;</a>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openPreviewModal(${{tpl.id}}, '${{escHtml(tpl.title)}}')">Preview</button>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openSettingsModal(${{tpl.id}})">Settings</button>
                                </div>
                                <div style="display: flex; gap: 6px;">
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 9px;" onclick="duplicateTemplate(${{tpl.id}})">Duplicate</button>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 9px; color: var(--oxide);" onclick="deleteTemplate(${{tpl.id}}, '${{escHtml(tpl.title)}}')">Delete</button>
                                </div>
                            </div>
                        </div>
                    `;
                }}).join('');
            }}

            async function injectDynamicLayoutSkeleton(templateId, templateType) {{
                let children = [];
                if (templateType === 'archive') {{
                    children = [
                        {{ id: 'arch_hdr', type: 'Heading', level: 1, text: '{{archive.title}}', styles: {{ marginBottom: '8px' }}, children: [] }},
                        {{ id: 'arch_sub', type: 'Text', content_html: '<p>{{archive.subtitle}}</p>', styles: {{ marginBottom: '24px' }}, children: [] }},
                        {{ id: 'arch_grid', type: 'LoopGrid', post_type: 'post', posts_per_page: 6, columns: 3, show_featured_image: true, show_excerpt: true, show_date: true, show_read_more: true, styles: {{}}, children: [] }}
                    ];
                }} else if (templateType === '404_page') {{
                    children = [
                        {{ id: 'err_hdr', type: 'Heading', level: 1, text: '404 — Page Not Found', styles: {{ marginBottom: '12px' }}, children: [] }},
                        {{ id: 'err_txt', type: 'Text', content_html: '<p>The requested URL could not be found on {{site.title}}.</p>', styles: {{ marginBottom: '20px' }}, children: [] }},
                        {{ id: 'err_btn', type: 'Button', label: 'Return to Front Page', url: '/', variant: 'primary', styles: {{}}, children: [] }}
                    ];
                }} else {{
                    children = [
                        {{ id: 'post_title_node', type: 'Heading', level: 1, text: '{{post.title}}', styles: {{ marginBottom: '10px' }}, children: [] }},
                        {{ id: 'post_meta_node', type: 'Text', content_html: '<p>Published {{post.date}} by {{author.name}}</p>', styles: {{ marginBottom: '24px' }}, children: [] }},
                        {{ id: 'post_body_node', type: 'PostContent', styles: {{}}, children: [] }}
                    ];
                }}
                const layout = {{
                    version: 1,
                    root: {{ id: 'root', type: 'Root', styles: {{ paddingTop: '48px', paddingBottom: '64px' }}, children }}
                }};
                try {{
                    const res = await fetch(`/wp-json/daypress/v1/templates/${{templateId}}`, {{
                        method: 'PUT',
                        headers: {{ 'Content-Type': 'application/json' }},
                        credentials: 'same-origin',
                        body: JSON.stringify({{ layout }})
                    }});
                    if (res.ok) {{
                        await refreshMatrixFromServer();
                        showToast('Dynamic AST skeleton injected and saved to SQLite');
                    }} else {{
                        showToast('Failed to inject skeleton', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            // =========================================================================
            // PAGE 3: SYNCED SYMBOLS & MEGA MENU PANEL BINDER
            // =========================================================================
            function inspectSymbolOverridableSlots(layout) {{
                const slots = [];
                function walk(node) {{
                    if (!node || typeof node !== 'object') return;
                    const t = node.type || '';
                    if (t === 'Heading') {{
                        slots.push({{ id: node.id, kind: 'Heading', preview: node.text || '' }});
                    }} else if (t === 'Text') {{
                        const raw = (node.content_html || '').replace(/<[^>]*>/g, '').trim();
                        slots.push({{ id: node.id, kind: 'Text', preview: raw.slice(0, 48) }});
                    }} else if (t === 'Button') {{
                        slots.push({{ id: node.id, kind: 'Button', preview: `${{node.label || 'Button'}} (${{node.url || '#'}})` }});
                    }} else if (t === 'Image') {{
                        slots.push({{ id: node.id, kind: 'Image', preview: node.alt || node.src || 'Image slot' }});
                    }}
                    if (Array.isArray(node.children)) node.children.forEach(walk);
                }}
                if (layout && layout.root) walk(layout.root);
                return slots;
            }}

            function renderSymbolsCards() {{
                const container = document.getElementById('symbolsCardsList');
                if (!container) return;

                const symbols = (hfMatrix.templates || []).filter(t => t.template_type === 'section_symbol');
                if (symbols.length === 0) {{
                    container.innerHTML = renderEmptyStateCard(
                        'section_symbol',
                        'No Synced Symbols created yet',
                        'Create a reusable global section (CTA banner, pricing block, or Mega Menu dropdown panel) that stays in sync across your entire site.'
                    );
                    return;
                }}

                container.innerHTML = symbols.map(tpl => {{
                    const slots = inspectSymbolOverridableSlots(tpl.layout);
                    const boundMenuItems = (hfNavMenuItems || []).map((item, idx) => ({{ item, idx }})).filter(x => Number(x.item.mega_menu_template_id) === Number(tpl.id));

                    const slotRowsHtml = slots.slice(0, 5).map(s => `
                        <div style="display:flex;justify-content:space-between;align-items:center;gap:8px;padding:4px 0;border-bottom:1px solid var(--rule);font-size:12px;">
                            <div style="display:flex;align-items:center;gap:6px;">
                                <span class="hf-chip hf-chip-behavior">${{escHtml(s.kind)}}</span>
                                <code style="font-family:var(--font-mono);font-size:11px;color:var(--ink);">${{escHtml(s.id)}}</code>
                            </div>
                            <span style="color:var(--ink-muted);font-size:11.5px;max-width:220px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;">${{escHtml(s.preview)}}</span>
                        </div>
                    `).join('');

                    const menuOptionsHtml = (hfNavMenuItems || []).map((it, idx) =>
                        `<option value="${{idx}}">${{escHtml(it.title)}} (${{escHtml(it.url)}})</option>`
                    ).join('');

                    const boundMenuPills = boundMenuItems.map(b => `
                        <span class="hf-chip hf-chip-global" style="display:inline-flex;align-items:center;gap:6px;">
                            <span>Mega Menu: ${{escHtml(b.item.title)}}</span>
                            <button type="button" onclick="detachSymbolFromNavMenuItem(${{b.idx}})" style="background:none;border:none;color:inherit;cursor:pointer;font-weight:700;padding:0;" title="Detach Mega Menu">&times;</button>
                        </span>
                    `).join('');

                    return `
                        <div class="hf-tpl-card">
                            <div style="display: flex; justify-content: space-between; align-items: flex-start; gap: 12px;">
                                <div>
                                    <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                                        <strong style="font-size: 15px; color: var(--ink);">${{escHtml(tpl.title)}}</strong>
                                        <button type="button" class="hf-chip hf-chip-behavior" style="cursor:pointer;" onclick="copyTextToClipboard('${{tpl.id}}')" title="Click to copy Symbol ID">symbol_id: #${{tpl.id}}</button>
                                        <span class="hf-chip hf-chip-global">Synced Component</span>
                                    </div>
                                    <div style="font-size: 11.5px; color: var(--ink-muted); margin-top: 3px; font-family: var(--font-mono);">
                                        ${{slots.length}} overridable slots &middot; Updated ${{escHtml((tpl.updated_at || '').slice(0, 16))}}
                                    </div>
                                </div>
                            </div>

                            <!-- Overridable Child Slots Well -->
                            <div class="ts-inline-controls">
                                <div class="ts-inline-row">
                                    <span style="font-size: 11.5px; font-weight: 600; color: var(--ink-muted);">Overridable Instance Slots (DayStudio <code>SymbolInstance</code>)</span>
                                    <span style="font-size: 11px; font-family: var(--font-mono); color: var(--ink-muted);">${{slots.length}} slots</span>
                                </div>
                                ${{slotRowsHtml || '<span style="font-size:12px;color:var(--ink-muted);">Open in DayStudio to add Heading, Text, Button, or Image slots.</span>'}}
                            </div>

                            <!-- 1-Click Mega Menu Panel Binder -->
                            <div class="ts-inline-controls">
                                <div class="ts-inline-row">
                                    <span style="font-size: 11.5px; font-weight: 600; color: var(--ink-muted);">Primary Navigation Mega Menu Binding</span>
                                    <a href="/d-admin/nav-menus" style="font-size: 11px; color: var(--press); text-decoration: none; font-weight: 600;">Edit Menus &rarr;</a>
                                </div>
                                <div style="display: flex; gap: 6px; flex-wrap: wrap; align-items: center;">
                                    ${{boundMenuPills || '<span style="font-size:11.5px;color:var(--ink-muted);">Not currently attached to a navigation item</span>'}}
                                </div>
                                ${{(hfNavMenuItems && hfNavMenuItems.length > 0) ? `
                                    <div style="display: flex; gap: 8px; align-items: center; margin-top: 4px;">
                                        <select id="megaBindSelect_${{tpl.id}}" class="form-select" style="flex: 1; font-size: 12px; padding: 4px 8px; height: 30px;">
                                            ${{menuOptionsHtml}}
                                        </select>
                                        <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 4px 10px;" onclick="attachSymbolToNavMenuItem(${{tpl.id}})">Attach as Mega Menu</button>
                                    </div>
                                ` : `<div style="font-size:11.5px;color:var(--ink-muted);">Add items in <a href="/d-admin/nav-menus" style="color:var(--press);">Appearance &rarr; Menus</a> to bind Mega Menu panels.</div>`}}
                            </div>

                            <div style="display: flex; justify-content: space-between; align-items: center; gap: 8px; padding-top: 6px; border-top: 1px solid var(--rule); flex-wrap: wrap;">
                                <div style="display: flex; gap: 6px;">
                                    <a href="/d-admin/builder?template_id=${{tpl.id}}" class="btn btn-primary" style="font-size: 12px; padding: 5px 12px;">Edit Symbol in DayStudio &nearr;</a>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openPreviewModal(${{tpl.id}}, '${{escHtml(tpl.title)}}')">Preview</button>
                                </div>
                                <div style="display: flex; gap: 6px;">
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 9px;" onclick="duplicateTemplate(${{tpl.id}})">Duplicate</button>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 9px; color: var(--oxide);" onclick="deleteTemplate(${{tpl.id}}, '${{escHtml(tpl.title)}}')">Delete</button>
                                </div>
                            </div>
                        </div>
                    `;
                }}).join('');
            }}

            async function attachSymbolToNavMenuItem(symbolId) {{
                const sel = document.getElementById('megaBindSelect_' + symbolId);
                if (!sel) return;
                const idx = parseInt(sel.value, 10);
                if (isNaN(idx) || !hfNavMenuItems[idx]) return;
                hfNavMenuItems[idx].mega_menu_template_id = Number(symbolId);
                await persistNavMenuItems(`Attached Symbol #${{symbolId}} as Mega Menu for "${{hfNavMenuItems[idx].title}}"`);
            }}

            async function detachSymbolFromNavMenuItem(idx) {{
                if (!hfNavMenuItems[idx]) return;
                const title = hfNavMenuItems[idx].title;
                delete hfNavMenuItems[idx].mega_menu_template_id;
                await persistNavMenuItems(`Detached Mega Menu from "${{title}}"`);
            }}

            async function persistNavMenuItems(toastMsg) {{
                try {{
                    const res = await fetch('/d-admin/api/menus', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        credentials: 'same-origin',
                        body: JSON.stringify({{ items: hfNavMenuItems }})
                    }});
                    if (res.ok) {{
                        const data = await res.json();
                        if (Array.isArray(data.items)) hfNavMenuItems = data.items;
                        renderAllViews();
                        showToast(toastMsg);
                    }} else {{
                        showToast('Failed to update navigation menu binding', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            // =========================================================================
            // PAGE 4: POPUPS, SLIDE-INS & ANNOUNCEMENT BARS
            // =========================================================================
            function setPopupFilter(filter) {{
                currentPopupFilter = filter;
                const map = {{
                    all: 'popupFilterAll',
                    bars: 'popupFilterBars',
                    modals: 'popupFilterModals',
                    drawers: 'popupFilterDrawers'
                }};
                Object.entries(map).forEach(([k, id]) => {{
                    const el = document.getElementById(id);
                    if (el) el.classList.toggle('active', k === filter);
                }});
                renderPopupsCards();
            }}

            function openCreatePopupWithVariant(variant) {{
                openCreateModal('popup');
                const vSel = document.getElementById('createTplPopupVariant');
                if (vSel) vSel.value = variant;
                const tInput = document.getElementById('createTplTitle');
                if (tInput) {{
                    const names = {{
                        top_bar: 'Top Site Announcement Bar',
                        bottom_bar: 'Bottom Sticky Promo Bar',
                        slide_in_right: 'Slide-In Right Conversion Drawer',
                        slide_in_left: 'Slide-In Left Drawer',
                        modal: 'Center Conversion Modal'
                    }};
                    tInput.value = names[variant] || 'Site Announcement Popup';
                }}
            }}

            function renderPopupsCards() {{
                const container = document.getElementById('popupsCardsList');
                if (!container) return;

                const allPopups = (hfMatrix.templates || []).filter(t => t.template_type === 'popup');
                const normVar = v => (v === 'announcement_bar_top' ? 'top_bar' : (v === 'announcement_bar_bottom' ? 'bottom_bar' : (v || 'modal')));
                const bars = allPopups.filter(t => ['top_bar', 'bottom_bar'].includes(normVar((t.settings || {{}}).popup_variant)));
                const modals = allPopups.filter(t => normVar((t.settings || {{}}).popup_variant) === 'modal');
                const drawers = allPopups.filter(t => ['slide_in_right', 'slide_in_left'].includes(normVar((t.settings || {{}}).popup_variant)));

                const setCnt = (id, n) => {{
                    const el = document.getElementById(id);
                    if (el) el.textContent = String(n);
                }};
                setCnt('popupCountAll', allPopups.length);
                setCnt('popupCountBars', bars.length);
                setCnt('popupCountModals', modals.length);
                setCnt('popupCountDrawers', drawers.length);

                let filtered = allPopups;
                if (currentPopupFilter === 'bars') filtered = bars;
                else if (currentPopupFilter === 'modals') filtered = modals;
                else if (currentPopupFilter === 'drawers') filtered = drawers;

                if (filtered.length === 0) {{
                    container.innerHTML = `
                        <div class="day-card" style="padding: 32px; text-align: center; grid-column: 1 / -1; border: 1px dashed var(--rule-strong); background: var(--paper-raised);">
                            <div style="font-size: 15px; font-weight: 600; color: var(--ink); margin-bottom: 6px;">No popups or announcement bars in this view</div>
                            <p style="font-size: 13px; color: var(--ink-muted); margin: 0 auto 18px auto; max-width: 520px;">
                                Launch a top announcement bar, slide-in newsletter drawer, or exit-intent modal with built-in visitor cooldown rules.
                            </p>
                            <div style="display: flex; gap: 8px; justify-content: center; flex-wrap: wrap;">
                                <button type="button" class="btn btn-primary" onclick="openCreatePopupWithVariant('top_bar')">Create Top Announcement Bar</button>
                                <button type="button" class="btn btn-secondary" onclick="openCreatePopupWithVariant('modal')">Create Center Modal</button>
                                <button type="button" class="btn btn-secondary" onclick="installStarterNow('top-announcement-popup', 'Top Announcement Bar')">Install Announcement Bar Starter</button>
                            </div>
                        </div>
                    `;
                    return;
                }}

                container.innerHTML = filtered.map(tpl => {{
                    const isActive = tpl.is_active && tpl.status === 'publish';
                    const s = tpl.settings || {{}};
                    const variant = normVar(s.popup_variant);
                    const trigger = s.trigger || 'on_load';
                    const delaySec = s.delay_sec != null ? s.delay_sec : 0;
                    const scrollPct = s.scroll_pct != null ? s.scroll_pct : 50;
                    const dismissDays = s.dismiss_days != null ? s.dismiss_days : 7;
                    const primaryTarget = getPrimaryTargetValue(tpl);
                    const cov = (hfMatrix.coverage && hfMatrix.coverage[tpl.id]) || {{ conditions_summary: 'Entire Site' }};

                    return `
                        <div class="hf-tpl-card ${{isActive ? '' : 'is-inactive'}}">
                            <div style="display: flex; justify-content: space-between; align-items: flex-start; gap: 12px;">
                                <div>
                                    <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                                        <strong style="font-size: 15px; color: var(--ink);">${{escHtml(tpl.title)}}</strong>
                                        <span style="font-size: 11px; font-family: var(--font-mono); color: var(--ink-muted);">#${{tpl.id}}</span>
                                        <span class="hf-chip ${{isActive ? 'hf-chip-global' : 'hf-chip-fallback'}}">${{isActive ? 'Active' : 'Paused'}}</span>
                                    </div>
                                    <div style="font-size: 11.5px; color: var(--ink-muted); margin-top: 3px; font-family: var(--font-mono);">
                                        Cooldown key: <code>dp_popup_dismissed_${{tpl.id}}</code>
                                    </div>
                                </div>
                                <label style="display: flex; align-items: center; gap: 6px; font-size: 12px; font-weight: 600; cursor: pointer;">
                                    <input type="checkbox" ${{isActive ? 'checked' : ''}} onchange="toggleTemplateActive(${{tpl.id}}, this.checked)" />
                                    <span>Enabled</span>
                                </label>
                            </div>

                            <!-- Inline Trigger, Placement & Cooldown Controls -->
                            <div class="ts-inline-controls" id="popupInlineBox_${{tpl.id}}">
                                <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px;">
                                    <div>
                                        <label class="form-label" style="font-size: 11px; margin-bottom: 3px;">Placement Variant</label>
                                        <select class="form-select pop-inline-variant" style="font-size: 12px; padding: 4px 8px; height: 30px;" onchange="saveInlinePopupSettings(${{tpl.id}})">
                                            <option value="top_bar" ${{variant === 'top_bar' ? 'selected' : ''}}>Top Announcement Bar</option>
                                            <option value="bottom_bar" ${{variant === 'bottom_bar' ? 'selected' : ''}}>Bottom Sticky Bar</option>
                                            <option value="modal" ${{variant === 'modal' ? 'selected' : ''}}>Center Modal Dialog</option>
                                            <option value="slide_in_right" ${{variant === 'slide_in_right' ? 'selected' : ''}}>Slide-In Right Drawer</option>
                                            <option value="slide_in_left" ${{variant === 'slide_in_left' ? 'selected' : ''}}>Slide-In Left Drawer</option>
                                        </select>
                                    </div>
                                    <div>
                                        <label class="form-label" style="font-size: 11px; margin-bottom: 3px;">Trigger Event</label>
                                        <select class="form-select pop-inline-trigger" style="font-size: 12px; padding: 4px 8px; height: 30px;" onchange="saveInlinePopupSettings(${{tpl.id}})">
                                            <option value="on_load" ${{trigger === 'on_load' ? 'selected' : ''}}>On Page Load</option>
                                            <option value="on_scroll" ${{trigger === 'on_scroll' ? 'selected' : ''}}>On Scroll Depth (%)</option>
                                            <option value="exit_intent" ${{trigger === 'exit_intent' ? 'selected' : ''}}>On Exit Intent</option>
                                        </select>
                                    </div>
                                </div>

                                <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px;">
                                    <div>
                                        <label class="form-label" style="font-size: 11px; margin-bottom: 3px;">Load Delay (s)</label>
                                        <input type="number" class="form-input pop-inline-delay" value="${{delaySec}}" min="0" max="120" style="font-size: 12px; padding: 4px 8px; height: 30px; font-variant-numeric: tabular-nums;" onchange="saveInlinePopupSettings(${{tpl.id}})" />
                                    </div>
                                    <div>
                                        <label class="form-label" style="font-size: 11px; margin-bottom: 3px;">Scroll Depth (%)</label>
                                        <input type="number" class="form-input pop-inline-scroll" value="${{scrollPct}}" min="5" max="100" style="font-size: 12px; padding: 4px 8px; height: 30px; font-variant-numeric: tabular-nums;" onchange="saveInlinePopupSettings(${{tpl.id}})" />
                                    </div>
                                    <div>
                                        <label class="form-label" style="font-size: 11px; margin-bottom: 3px;">Cooldown (days)</label>
                                        <input type="number" class="form-input pop-inline-dismiss" value="${{dismissDays}}" min="0" max="365" style="font-size: 12px; padding: 4px 8px; height: 30px; font-variant-numeric: tabular-nums;" onchange="saveInlinePopupSettings(${{tpl.id}})" />
                                    </div>
                                </div>
                            </div>

                            <!-- Target Scope -->
                            <div class="ts-inline-controls">
                                <div class="ts-inline-row">
                                    <span style="font-size: 11.5px; font-weight: 600; color: var(--ink-muted);">Target Pages &amp; Routes</span>
                                    <div style="display: flex; gap: 6px; align-items: center;">
                                        <select class="form-select" style="font-size: 12px; padding: 3px 8px; height: 28px;" onchange="quickAssignPrimaryCondition(${{tpl.id}}, this.value)">
                                            <option value="entire_site" ${{primaryTarget === 'entire_site' ? 'selected' : ''}}>Entire Site</option>
                                            <option value="front_page" ${{primaryTarget === 'front_page' ? 'selected' : ''}}>Front Page Only</option>
                                            <option value="all_posts" ${{primaryTarget === 'all_posts' ? 'selected' : ''}}>All Blog Posts</option>
                                            <option value="all_pages" ${{primaryTarget === 'all_pages' ? 'selected' : ''}}>All Static Pages</option>
                                            <option value="unassigned" ${{primaryTarget === 'unassigned' ? 'selected' : ''}}>Paused / Unassigned</option>
                                            ${{primaryTarget === 'custom' ? '<option value="custom" selected>Specific Page Rule</option>' : ''}}
                                        </select>
                                        <button type="button" class="btn btn-secondary" style="font-size: 11.5px; padding: 3px 8px;" onclick="openConditionsModal(${{tpl.id}})">Conditions</button>
                                    </div>
                                </div>
                                <div style="font-size: 12px; color: var(--ink);">${{escHtml(cov.conditions_summary || 'Entire Site')}}</div>
                            </div>

                            <div style="display: flex; justify-content: space-between; align-items: center; gap: 8px; padding-top: 6px; border-top: 1px solid var(--rule); flex-wrap: wrap;">
                                <div style="display: flex; gap: 6px;">
                                    <a href="/d-admin/builder?template_id=${{tpl.id}}" class="btn btn-primary" style="font-size: 12px; padding: 5px 12px;">Design in DayStudio &nearr;</a>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openPreviewModal(${{tpl.id}}, '${{escHtml(tpl.title)}}')">Preview</button>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openSettingsModal(${{tpl.id}})">Settings</button>
                                </div>
                                <div style="display: flex; gap: 6px;">
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 9px;" onclick="duplicateTemplate(${{tpl.id}})">Duplicate</button>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 9px; color: var(--oxide);" onclick="deleteTemplate(${{tpl.id}}, '${{escHtml(tpl.title)}}')">Delete</button>
                                </div>
                            </div>
                        </div>
                    `;
                }}).join('');
            }}

            async function saveInlinePopupSettings(templateId) {{
                const tpl = (hfMatrix.templates || []).find(t => t.id === templateId);
                const box = document.getElementById('popupInlineBox_' + templateId);
                if (!tpl || !box) return;

                const popup_variant = box.querySelector('.pop-inline-variant')?.value || 'modal';
                const trigger = box.querySelector('.pop-inline-trigger')?.value || 'on_load';
                const delay_sec = parseInt(box.querySelector('.pop-inline-delay')?.value || '0', 10) || 0;
                const scroll_pct = parseInt(box.querySelector('.pop-inline-scroll')?.value || '50', 10) || 50;
                const dismiss_days = parseInt(box.querySelector('.pop-inline-dismiss')?.value || '7', 10) || 0;

                const settings = Object.assign({{}}, tpl.settings || {{}}, {{
                    popup_variant,
                    trigger,
                    delay_sec,
                    scroll_pct,
                    dismiss_days
                }});

                try {{
                    const res = await fetch(`/wp-json/daypress/v1/templates/${{templateId}}`, {{
                        method: 'PUT',
                        headers: {{ 'Content-Type': 'application/json' }},
                        credentials: 'same-origin',
                        body: JSON.stringify({{ settings }})
                    }});
                    if (res.ok) {{
                        await refreshMatrixFromServer();
                        showToast(`Saved popup trigger & cooldown rules for "${{tpl.title}}"`);
                    }} else {{
                        showToast('Failed to save popup rules', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            // =========================================================================
            // COMMON TEMPLATE CRUD ACTIONS & MODALS
            // =========================================================================
            async function toggleTemplateActive(id, active) {{
                try {{
                    const res = await fetch(`/wp-json/daypress/v1/templates/${{id}}`, {{
                        method: 'PUT',
                        headers: {{ 'Content-Type': 'application/json' }},
                        credentials: 'same-origin',
                        body: JSON.stringify({{ is_active: active, status: active ? 'publish' : 'draft' }})
                    }});
                    if (res.ok) {{
                        await refreshMatrixFromServer();
                        showToast(active ? 'Component activated in SQLite' : 'Component paused');
                    }} else {{
                        showToast('Failed to toggle component status', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            async function duplicateTemplate(id) {{
                try {{
                    const res = await fetch(`/wp-json/daypress/v1/templates/${{id}}/duplicate`, {{
                        method: 'POST',
                        credentials: 'same-origin'
                    }});
                    if (res.ok) {{
                        await refreshMatrixFromServer();
                        showToast('Component duplicated in SQLite');
                    }} else {{
                        showToast('Failed to duplicate component', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            async function deleteTemplate(id, title) {{
                if (!confirm(`Delete "${{title}}" (#${{id}})? Pages using this component will automatically fall back to global rules.`)) {{
                    return;
                }}
                try {{
                    const res = await fetch(`/wp-json/daypress/v1/templates/${{id}}?force=true`, {{
                        method: 'DELETE',
                        credentials: 'same-origin'
                    }});
                    if (res.ok) {{
                        await refreshMatrixFromServer();
                        showToast(`Deleted "${{title}}"`);
                    }} else {{
                        showToast('Failed to delete component', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            // MODAL 1: CREATE
            function openCreateModal(defaultType) {{
                const sel = document.getElementById('createTplType');
                sel.value = defaultType || sel.getAttribute('data-default-type') || 'header';
                onCreateTypeChange();
                populateCreatePageOptions();
                document.getElementById('hfCreateModal').classList.add('open');
                setTimeout(() => document.getElementById('createTplTitle')?.focus(), 20);
            }}

            function closeCreateModal() {{
                document.getElementById('hfCreateModal').classList.remove('open');
            }}

            function onCreateTypeChange() {{
                const t = document.getElementById('createTplType').value;
                const starterSel = document.getElementById('createTplStarter');
                starterSel.innerHTML = `<option value="blank">Blank Canvas (Clean Root AST)</option>`;
                hfStarters.filter(s => s.template_type === t).forEach(s => {{
                    starterSel.innerHTML += `<option value="${{escHtml(s.id)}}">Starter: ${{escHtml(s.title)}}</option>`;
                }});
                const titleInput = document.getElementById('createTplTitle');
                const targetSel = document.getElementById('createTplTarget');
                const popupBox = document.getElementById('createPopupBehaviors');
                const assignRow = document.getElementById('createAssignmentRow');
                if (popupBox) {{
                    popupBox.style.display = t === 'popup' ? 'flex' : 'none';
                }}
                if (assignRow) {{
                    assignRow.style.display = t === 'section_symbol' ? 'none' : 'grid';
                }}
                const defaultTitles = {{
                    header: 'Custom Site Header',
                    footer: 'Custom Site Footer',
                    single_post: 'Single Blog Post Layout',
                    single_page: 'Single Static Page Layout',
                    archive: 'Category & Author Archive Layout',
                    '404_page': 'Custom 404 Error Page',
                    section_symbol: 'Synced Global Section Symbol',
                    popup: 'Site Announcement / Promo Popup'
                }};
                const defaultTargets = {{
                    header: 'entire_site',
                    footer: 'entire_site',
                    single_post: 'all_posts',
                    single_page: 'all_pages',
                    archive: 'archive',
                    '404_page': 'not_found',
                    section_symbol: 'unassigned',
                    popup: 'entire_site'
                }};
                if (titleInput && defaultTitles[t]) {{
                    titleInput.value = defaultTitles[t];
                }}
                if (targetSel && defaultTargets[t]) {{
                    targetSel.value = defaultTargets[t];
                    onCreateTargetChange();
                }}
            }}

            function onCreateStarterChange() {{
                const stId = document.getElementById('createTplStarter').value;
                if (stId !== 'blank') {{
                    const found = hfStarters.find(s => s.id === stId);
                    if (found) document.getElementById('createTplTitle').value = found.title;
                }}
            }}

            function onCreateTargetChange() {{
                const target = document.getElementById('createTplTarget').value;
                document.getElementById('createTplPageWrap').style.display = target === 'page' ? 'block' : 'none';
            }}

            function populateCreatePageOptions() {{
                const sel = document.getElementById('createTplPageId');
                const pages = hfMatrix.pages || [];
                sel.innerHTML = pages.map(p => `<option value="${{p.post_id}}">${{escHtml(p.title)}} (${{escHtml(p.permalink)}})</option>`).join('');
            }}

            async function submitCreateTemplate(openInStudio) {{
                const title = document.getElementById('createTplTitle').value.trim();
                if (!title) {{
                    showToast('Please enter a component name', true);
                    return;
                }}
                const template_type = document.getElementById('createTplType').value;
                const starter = document.getElementById('createTplStarter').value;
                const ruleTarget = template_type === 'section_symbol' ? 'unassigned' : document.getElementById('createTplTarget').value;
                const pageId = document.getElementById('createTplPageId').value;
                const priority = parseInt(document.getElementById('createTplPriority').value, 10) || 10;

                let conditions = [];
                if (ruleTarget === 'unassigned') {{
                    conditions = [{{ condition_type: 'exclude', rule_target: 'entire_site', target_id: '', priority: 10 }}];
                }} else if (ruleTarget === 'page' && pageId) {{
                    conditions = [{{ condition_type: 'include', rule_target: 'page', target_id: String(pageId), priority: 100 }}];
                }} else {{
                    conditions = [{{ condition_type: 'include', rule_target: ruleTarget, target_id: '', priority }}];
                }}

                let initialSettings = {{}};
                if (template_type === 'popup') {{
                    initialSettings = {{
                        popup_variant: document.getElementById('createTplPopupVariant')?.value || 'modal',
                        trigger: document.getElementById('createTplPopupTrigger')?.value || 'on_load',
                        delay_sec: parseInt(document.getElementById('createTplPopupDelaySec')?.value, 10) || 0,
                        scroll_pct: parseInt(document.getElementById('createTplPopupScrollPct')?.value, 10) || 50,
                        dismiss_days: parseInt(document.getElementById('createTplPopupDismissDays')?.value, 10) || 7
                    }};
                }}

                try {{
                    let created = null;
                    if (starter !== 'blank') {{
                        const res = await fetch('/wp-json/daypress/v1/templates/starter', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            credentials: 'same-origin',
                            body: JSON.stringify({{ starter_id: starter }})
                        }});
                        if (!res.ok) throw new Error('Failed to install starter');
                        created = await res.json();
                        await fetch(`/wp-json/daypress/v1/templates/${{created.id}}`, {{
                            method: 'PUT',
                            headers: {{ 'Content-Type': 'application/json' }},
                            credentials: 'same-origin',
                            body: JSON.stringify({{ title, priority, conditions, settings: Object.assign({{}}, created.settings || {{}}, initialSettings) }})
                        }});
                    }} else {{
                        const res = await fetch('/wp-json/daypress/v1/templates', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            credentials: 'same-origin',
                            body: JSON.stringify({{
                                title,
                                template_type,
                                status: 'publish',
                                is_active: true,
                                priority,
                                conditions,
                                settings: initialSettings,
                                layout: {{ version: 1, root: {{ id: 'root', type: 'Root', children: [] }} }}
                            }})
                        }});
                        if (!res.ok) throw new Error('Failed to create component');
                        created = await res.json();
                    }}

                    closeCreateModal();
                    if (openInStudio && created && created.id) {{
                        window.location.href = `/d-admin/builder?template_id=${{created.id}}`;
                        return;
                    }}
                    await refreshMatrixFromServer();
                    showToast(`Created "${{title}}" in SQLite`);
                }} catch (e) {{
                    showToast('Error: ' + e.message, true);
                }}
            }}

            // MODAL 2: DISPLAY CONDITIONS
            function openConditionsModal(templateId) {{
                const tpl = (hfMatrix.templates || []).find(t => t.id === templateId);
                if (!tpl) return;
                editingConditionsTplId = templateId;
                document.getElementById('hfCondModalTitle').textContent = `Display Conditions: ${{tpl.title}} (#${{tpl.id}})`;

                const conds = (tpl.conditions && tpl.conditions.length > 0)
                    ? tpl.conditions.filter(c => !(c.condition_type === 'exclude' && c.rule_target === 'entire_site' && tpl.conditions.length === 1))
                    : [{{ condition_type: 'include', rule_target: 'entire_site', target_id: '', priority: 10 }}];

                renderConditionModalRows(conds);
                document.getElementById('hfConditionsModal').classList.add('open');
            }}

            function closeConditionsModal() {{
                document.getElementById('hfConditionsModal').classList.remove('open');
                editingConditionsTplId = null;
            }}

            function renderConditionModalRows(conds) {{
                const container = document.getElementById('hfCondRowsContainer');
                const pages = hfMatrix.pages || [];
                const posts = hfMatrix.posts || [];

                container.innerHTML = conds.map((c) => {{
                    const rt = c.rule_target || 'entire_site';
                    const showPageSel = rt === 'page';
                    const showPostSel = rt === 'post';
                    const showTextId = rt === 'category' || rt === 'tag' || rt === 'author';

                    const pageOpts = pages.map(p => `<option value="${{p.post_id}}" ${{String(c.target_id) === String(p.post_id) ? 'selected' : ''}}>${{escHtml(p.title)}} (${{escHtml(p.permalink)}})</option>`).join('');
                    const postOpts = posts.map(p => `<option value="${{p.post_id}}" ${{String(c.target_id) === String(p.post_id) ? 'selected' : ''}}>${{escHtml(p.title)}} (${{escHtml(p.permalink)}})</option>`).join('');

                    return `
                        <div class="hf-cond-row" style="display:flex;gap:8px;align-items:center;background:var(--paper);border:1px solid var(--rule-strong);padding:10px;border-radius:8px;flex-wrap:wrap;">
                            <select class="form-select cond-row-type" style="width:120px;font-weight:600;">
                                <option value="include" ${{c.condition_type === 'include' ? 'selected' : ''}}>INCLUDE</option>
                                <option value="exclude" ${{c.condition_type === 'exclude' ? 'selected' : ''}}>EXCLUDE</option>
                            </select>
                            <select class="form-select cond-row-target" style="flex:1;min-width:180px;" onchange="onCondRowTargetChange(this)">
                                <option value="entire_site" ${{rt === 'entire_site' ? 'selected' : ''}}>Entire Site (All Pages &amp; Posts)</option>
                                <option value="front_page" ${{rt === 'front_page' ? 'selected' : ''}}>Front Page Only</option>
                                <option value="all_pages" ${{rt === 'all_pages' ? 'selected' : ''}}>All Static Pages</option>
                                <option value="all_posts" ${{rt === 'all_posts' ? 'selected' : ''}}>All Blog Posts</option>
                                <option value="page" ${{rt === 'page' ? 'selected' : ''}}>Specific Page...</option>
                                <option value="post" ${{rt === 'post' ? 'selected' : ''}}>Specific Blog Post...</option>
                                <option value="archive" ${{rt === 'archive' ? 'selected' : ''}}>All Archives</option>
                                <option value="category" ${{rt === 'category' ? 'selected' : ''}}>Category Slug/ID...</option>
                                <option value="tag" ${{rt === 'tag' ? 'selected' : ''}}>Tag Slug/ID...</option>
                                <option value="not_found" ${{rt === 'not_found' ? 'selected' : ''}}>404 Not Found Page</option>
                            </select>
                            <select class="form-select cond-row-page-id" style="width:210px;display:${{showPageSel ? 'block' : 'none'}};">
                                ${{pageOpts || '<option value="">No pages found</option>'}}
                            </select>
                            <select class="form-select cond-row-post-id" style="width:210px;display:${{showPostSel ? 'block' : 'none'}};">
                                ${{postOpts || '<option value="">No posts found</option>'}}
                            </select>
                            <input type="text" class="form-input cond-row-custom-id" placeholder="Slug or ID" value="${{escHtml(c.target_id || '')}}" style="width:140px;display:${{showTextId ? 'block' : 'none'}};" />
                            <input type="number" class="form-input cond-row-priority" value="${{c.priority || 10}}" title="Rule Priority" style="width:76px;" />
                            <button type="button" class="btn btn-secondary" style="color:var(--oxide);padding:6px 10px;" onclick="this.closest('.hf-cond-row').remove()">Remove</button>
                        </div>
                    `;
                }}).join('');
            }}

            function onCondRowTargetChange(sel) {{
                const row = sel.closest('.hf-cond-row');
                const v = sel.value;
                row.querySelector('.cond-row-page-id').style.display = v === 'page' ? 'block' : 'none';
                row.querySelector('.cond-row-post-id').style.display = v === 'post' ? 'block' : 'none';
                row.querySelector('.cond-row-custom-id').style.display = (v === 'category' || v === 'tag' || v === 'author') ? 'block' : 'none';
            }}

            function collectConditionModalRows() {{
                const rows = document.querySelectorAll('#hfCondRowsContainer .hf-cond-row');
                const out = [];
                rows.forEach(r => {{
                    const condition_type = r.querySelector('.cond-row-type').value;
                    const rule_target = r.querySelector('.cond-row-target').value;
                    let target_id = '';
                    if (rule_target === 'page') target_id = r.querySelector('.cond-row-page-id').value;
                    else if (rule_target === 'post') target_id = r.querySelector('.cond-row-post-id').value;
                    else if (rule_target === 'category' || rule_target === 'tag' || rule_target === 'author') target_id = r.querySelector('.cond-row-custom-id').value.trim();
                    const priority = parseInt(r.querySelector('.cond-row-priority').value, 10) || 10;
                    out.push({{ condition_type, rule_target, target_id: String(target_id || ''), priority }});
                }});
                return out;
            }}

            function addConditionModalRow() {{
                const current = collectConditionModalRows();
                current.push({{ condition_type: 'include', rule_target: 'page', target_id: (hfMatrix.pages?.[0]?.post_id || ''), priority: 100 }});
                renderConditionModalRows(current);
            }}

            async function saveConditionsModal() {{
                if (!editingConditionsTplId) return;
                let conditions = collectConditionModalRows();
                if (conditions.length === 0) {{
                    conditions = [{{ condition_type: 'exclude', rule_target: 'entire_site', target_id: '', priority: 10 }}];
                }}
                try {{
                    const res = await fetch(`/wp-json/daypress/v1/templates/${{editingConditionsTplId}}/conditions`, {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        credentials: 'same-origin',
                        body: JSON.stringify({{ conditions }})
                    }});
                    if (res.ok) {{
                        closeConditionsModal();
                        await refreshMatrixFromServer();
                        showToast('Display conditions saved & page coverage updated');
                    }} else {{
                        showToast('Failed to save conditions', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            // MODAL 3: SETTINGS & BEHAVIOR
            function openSettingsModal(templateId) {{
                const tpl = (hfMatrix.templates || []).find(t => t.id === templateId);
                if (!tpl) return;
                editingSettingsTplId = templateId;
                document.getElementById('hfSettingsModalTitle').textContent = `Settings: ${{tpl.title}} (#${{tpl.id}})`;
                document.getElementById('setTplTitle').value = tpl.title || '';
                document.getElementById('setTplStatus').value = tpl.status || 'publish';
                document.getElementById('setTplPriority').value = tpl.priority || 10;

                const s = tpl.settings || {{}};
                document.getElementById('setTplSticky').checked = !!s.sticky;
                document.getElementById('setTplTransparent').checked = !!s.transparent;
                document.getElementById('setTplScrollShrink').checked = !!s.scroll_shrink;
                document.getElementById('setTplZIndex').value = s.z_index != null ? s.z_index : '';
                document.getElementById('setTplStickyFooter').checked = !!s.sticky_footer;
                document.getElementById('setTplRevealScroll').checked = !!s.reveal_on_scroll;
                document.getElementById('setTplCustomCss').value = s.custom_css || '';

                if (document.getElementById('setTplPopupVariant')) {{
                    document.getElementById('setTplPopupVariant').value = s.popup_variant || 'modal';
                }}
                if (document.getElementById('setTplPopupTrigger')) {{
                    document.getElementById('setTplPopupTrigger').value = s.trigger || 'on_load';
                }}
                if (document.getElementById('setTplPopupDelaySec')) {{
                    document.getElementById('setTplPopupDelaySec').value = s.delay_sec != null ? s.delay_sec : 0;
                }}
                if (document.getElementById('setTplPopupScrollPct')) {{
                    document.getElementById('setTplPopupScrollPct').value = s.scroll_pct != null ? s.scroll_pct : 50;
                }}
                if (document.getElementById('setTplPopupDismissDays')) {{
                    document.getElementById('setTplPopupDismissDays').value = s.dismiss_days != null ? s.dismiss_days : 7;
                }}

                document.getElementById('setHeaderBehaviors').style.display = tpl.template_type === 'header' ? 'flex' : 'none';
                document.getElementById('setFooterBehaviors').style.display = tpl.template_type === 'footer' ? 'flex' : 'none';
                if (document.getElementById('setPopupBehaviors')) {{
                    document.getElementById('setPopupBehaviors').style.display = tpl.template_type === 'popup' ? 'flex' : 'none';
                }}

                document.getElementById('hfSettingsModal').classList.add('open');
            }}

            function closeSettingsModal() {{
                document.getElementById('hfSettingsModal').classList.remove('open');
                editingSettingsTplId = null;
            }}

            async function saveSettingsModal() {{
                if (!editingSettingsTplId) return;
                const tpl = (hfMatrix.templates || []).find(t => t.id === editingSettingsTplId);
                if (!tpl) return;

                const title = document.getElementById('setTplTitle').value.trim() || tpl.title;
                const status = document.getElementById('setTplStatus').value;
                const priority = parseInt(document.getElementById('setTplPriority').value, 10) || 10;
                const zVal = parseInt(document.getElementById('setTplZIndex').value, 10);
                const delayVal = parseInt(document.getElementById('setTplPopupDelaySec')?.value, 10);
                const scrollVal = parseInt(document.getElementById('setTplPopupScrollPct')?.value, 10);
                const dismissVal = parseInt(document.getElementById('setTplPopupDismissDays')?.value, 10);

                const settings = Object.assign({{}}, tpl.settings || {{}}, {{
                    sticky: document.getElementById('setTplSticky').checked,
                    transparent: document.getElementById('setTplTransparent').checked,
                    scroll_shrink: document.getElementById('setTplScrollShrink').checked,
                    z_index: isNaN(zVal) ? null : zVal,
                    sticky_footer: document.getElementById('setTplStickyFooter').checked,
                    reveal_on_scroll: document.getElementById('setTplRevealScroll').checked,
                    popup_variant: document.getElementById('setTplPopupVariant')?.value || 'modal',
                    trigger: document.getElementById('setTplPopupTrigger')?.value || 'on_load',
                    delay_sec: isNaN(delayVal) ? 0 : delayVal,
                    scroll_pct: isNaN(scrollVal) ? 50 : scrollVal,
                    dismiss_days: isNaN(dismissVal) ? 7 : dismissVal,
                    custom_css: document.getElementById('setTplCustomCss').value
                }});

                try {{
                    const res = await fetch(`/wp-json/daypress/v1/templates/${{editingSettingsTplId}}`, {{
                        method: 'PUT',
                        headers: {{ 'Content-Type': 'application/json' }},
                        credentials: 'same-origin',
                        body: JSON.stringify({{ title, status, priority, settings }})
                    }});
                    if (res.ok) {{
                        closeSettingsModal();
                        await refreshMatrixFromServer();
                        showToast('Component settings saved to SQLite');
                    }} else {{
                        showToast('Failed to save component settings', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            // MODAL 4: LIVE PREVIEW
            function openPreviewModal(templateId, title) {{
                document.getElementById('hfPreviewModalTitle').textContent = `Live Preview: ${{title}} (#${{templateId}})`;
                document.getElementById('hfPreviewOpenStudioBtn').href = `/d-admin/builder?template_id=${{templateId}}`;
                const iframe = document.getElementById('hfPreviewIframe');
                iframe.style.width = '100%';
                iframe.src = `/wp-json/daypress/v1/templates/preview/${{templateId}}`;
                document.getElementById('hfPreviewModal').classList.add('open');
            }}

            function closePreviewModal() {{
                document.getElementById('hfPreviewModal').classList.remove('open');
                document.getElementById('hfPreviewIframe').src = 'about:blank';
            }}

            function setPreviewViewport(w) {{
                document.getElementById('hfPreviewIframe').style.width = w;
            }}

            // MODAL 5: STARTERS CATALOG
            function openStarterCatalogModal(filterGroup) {{
                currentStarterFilter = filterGroup || 'all';
                const grid = document.getElementById('hfStartersGrid');
                let items = hfStarters;
                if (currentStarterFilter === 'header_footer') {{
                    items = hfStarters.filter(s => s.template_type === 'header' || s.template_type === 'footer');
                }} else if (currentStarterFilter === 'layouts') {{
                    items = hfStarters.filter(s => ['single_post', 'single_page', 'archive', '404_page'].includes(s.template_type));
                }} else if (currentStarterFilter !== 'all') {{
                    items = hfStarters.filter(s => s.template_type === currentStarterFilter);
                }}
                if (items.length === 0) items = hfStarters;

                grid.innerHTML = items.map(s => `
                    <div class="day-card" style="padding: 16px; display: flex; flex-direction: column; justify-content: space-between; gap: 12px; border: 1px solid var(--rule-strong);">
                        <div>
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                                <strong style="font-size: 14px; color: var(--ink);">${{escHtml(s.title)}}</strong>
                                <span class="hf-chip hf-chip-behavior">${{escHtml(s.template_type)}}</span>
                            </div>
                            <p style="font-size: 12px; color: var(--ink-muted); margin: 0;">${{escHtml(s.description)}}</p>
                        </div>
                        <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 10px; border-top: 1px solid var(--rule);">
                            <span style="font-size: 11px; color: var(--ink-muted); font-family: var(--font-mono);">${{escHtml(s.category)}}</span>
                            <button type="button" class="btn btn-primary" style="font-size: 12px; padding: 5px 12px;" onclick="installStarterNow('${{escHtml(s.id)}}', '${{escHtml(s.title)}}')">Install Starter</button>
                        </div>
                    </div>
                `).join('');
                document.getElementById('hfStartersModal').classList.add('open');
            }}

            function closeStarterCatalogModal() {{
                document.getElementById('hfStartersModal').classList.remove('open');
            }}

            async function installStarterNow(starterId, title) {{
                try {{
                    const res = await fetch('/wp-json/daypress/v1/templates/starter', {{
                        method: 'POST',
                        headers: {{ 'Content-Type': 'application/json' }},
                        credentials: 'same-origin',
                        body: JSON.stringify({{ starter_id: starterId, title }})
                    }});
                    if (res.ok) {{
                        closeStarterCatalogModal();
                        await refreshMatrixFromServer();
                        showToast(`Installed starter "${{title}}" in SQLite`);
                    }} else {{
                        showToast('Failed to install starter blueprint', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            document.addEventListener('DOMContentLoaded', () => {{
                const params = new URLSearchParams(window.location.search);
                if (window.tsPageMode === 'headers_footers') {{
                    if (params.get('tab') === 'matrix') {{
                        switchMainTab('matrix');
                    }} else if (params.get('tab') === 'layouts') {{
                        window.location.href = '/d-admin/theme-layouts';
                        return;
                    }}
                }}
                renderAllViews();
            }});
        </script>
        "##,
        page_mode = page_mode,
        initial_matrix_json = initial_matrix_json,
        starters_json = starters_json,
        nav_menu_items_json = nav_menu_items_json,
    )
}
