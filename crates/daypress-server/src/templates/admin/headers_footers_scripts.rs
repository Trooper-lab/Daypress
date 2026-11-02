//! Client-side interactive script for the Theme Header & Footer Manager (`/d-admin/headers-footers`).

pub fn render_headers_footers_script(initial_matrix_json: &str, starters_json: &str) -> String {
    format!(
        r##"
        <script>
            let hfMatrix = {initial_matrix_json};
            const hfStarters = {starters_json};
            let currentMainTab = 'templates';
            let currentTemplateFilter = 'all';
            let editingConditionsTplId = null;
            let editingSettingsTplId = null;

            function escHtml(s) {{
                return String(s == null ? '' : s).replace(/[&<>"']/g, c => ({{'&':'&amp;','<':'&lt;','>':'&gt;','"':'&quot;',"'":'&#39;'}}[c]));
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

            function switchMainTab(tab) {{
                currentMainTab = tab;
                document.getElementById('tabBtnTemplates').classList.toggle('active', tab === 'templates');
                document.getElementById('tabBtnMatrix').classList.toggle('active', tab === 'matrix');
                document.getElementById('tabBtnLayouts').classList.toggle('active', tab === 'layouts');

                document.getElementById('tabPanelTemplates').style.display = tab === 'templates' ? 'block' : 'none';
                document.getElementById('tabPanelMatrix').style.display = tab === 'matrix' ? 'block' : 'none';
                document.getElementById('tabPanelLayouts').style.display = tab === 'layouts' ? 'block' : 'none';
                document.getElementById('templatesTypeFilterBar').style.display = (tab === 'templates' || tab === 'layouts') ? 'flex' : 'none';
            }}

            function setTemplateFilter(type, btn) {{
                currentTemplateFilter = type;
                document.querySelectorAll('.hf-filter-pill').forEach(b => {{
                    b.style.background = '';
                    b.style.color = '';
                }});
                if (!btn) {{
                    btn = document.querySelector(`.hf-filter-pill[data-type="${{type}}"]`);
                }}
                if (btn) {{
                    btn.style.background = 'var(--ink)';
                    btn.style.color = 'var(--paper-raised)';
                }}
                const isOtherType = ['single_post', 'single_page', 'archive', '404_page', 'section_symbol', 'popup'].includes(type);
                if (isOtherType && currentMainTab !== 'layouts') {{
                    switchMainTab('layouts');
                }} else if ((type === 'header' || type === 'footer') && currentMainTab !== 'templates') {{
                    switchMainTab('templates');
                }}
                document.getElementById('headersSectionWrap').style.display = (type === 'all' || type === 'header') ? 'block' : 'none';
                document.getElementById('footersSectionWrap').style.display = (type === 'all' || type === 'footer') ? 'block' : 'none';
                renderAllViews();
            }}

            function renderAllViews() {{
                const templates = hfMatrix.templates || [];
                const headers = templates.filter(t => t.template_type === 'header');
                const footers = templates.filter(t => t.template_type === 'footer');
                const others = templates.filter(t => t.template_type !== 'header' && t.template_type !== 'footer');
                const isOtherType = ['single_post', 'single_page', 'archive', '404_page', 'section_symbol', 'popup'].includes(currentTemplateFilter);
                const filteredOthers = isOtherType
                    ? others.filter(t => t.template_type === currentTemplateFilter)
                    : others;

                const activeH = headers.filter(t => t.is_active && t.status === 'publish').length;
                const activeF = footers.filter(t => t.is_active && t.status === 'publish').length;
                const pages = hfMatrix.pages || [];
                const routes = hfMatrix.routes || [];
                const overrides = [...pages, ...routes].filter(r => r.header.mode !== 'inherit' || r.footer.mode !== 'inherit').length;

                document.getElementById('kpiHeadersVal').innerHTML = `${{activeH}} <span style="font-size: 14px; font-weight: 500; color: var(--ink-muted);">/ ${{headers.length}} active</span>`;
                document.getElementById('kpiFootersVal').innerHTML = `${{activeF}} <span style="font-size: 14px; font-weight: 500; color: var(--ink-muted);">/ ${{footers.length}} active</span>`;
                document.getElementById('kpiPagesVal').textContent = String(pages.length);
                document.getElementById('kpiOverridesVal').textContent = String(overrides);
                document.getElementById('kpiOtherVal').textContent = String(others.length);

                document.getElementById('tabCountHF').textContent = String(headers.length + footers.length);
                document.getElementById('tabCountPages').textContent = String(pages.length);
                document.getElementById('tabCountOther').textContent = String(others.length);

                renderTemplateCards(document.getElementById('headersCardsContainer'), headers, 'header');
                renderTemplateCards(document.getElementById('footersCardsContainer'), footers, 'footer');
                renderTemplateCards(document.getElementById('otherLayoutsCardsContainer'), filteredOthers, isOtherType ? currentTemplateFilter : 'layout');
                renderMatrixTables();
            }}

            function renderTemplateCards(container, list, kindLabel) {{
                if (!container) return;
                if (!list || list.length === 0) {{
                    const defaultType = ['header','footer','single_post','single_page','archive','404_page','section_symbol','popup'].includes(kindLabel)
                        ? kindLabel
                        : 'single_post';
                    container.innerHTML = `
                        <div class="day-card" style="grid-column: 1 / -1; padding: 28px; text-align: center; background: var(--paper-raised); border: 1px dashed var(--rule-strong);">
                            <div style="font-size: 14px; font-weight: 700; color: var(--ink); margin-bottom: 6px;">No ${{escHtml(kindLabel)}} templates created yet</div>
                            <p style="font-size: 12.5px; color: var(--ink-muted); margin-bottom: 14px;">Create a new ${{escHtml(kindLabel)}} or install a 1-click starter template to customize your site shell.</p>
                            <div style="display: inline-flex; gap: 8px;">
                                <button type="button" class="btn btn-primary" style="font-size: 12px;" onclick="openCreateModal('${{defaultType}}')">+ Create ${{escHtml(kindLabel)}}</button>
                                <button type="button" class="btn btn-secondary" style="font-size: 12px;" onclick="openStarterCatalogModal()">Browse Starters</button>
                            </div>
                        </div>
                    `;
                    return;
                }}

                const covMap = hfMatrix.coverage || {{}};
                const pages = hfMatrix.pages || [];

                container.innerHTML = list.map(t => {{
                    const cov = covMap[t.id] || {{ matched_pages_count: 0, matched_pages: [], conditions_summary: 'Entire Site (Default)' }};
                    const isPublished = t.status === 'publish' && t.is_active;
                    const cardCls = isPublished ? 'hf-tpl-card is-active-tpl' : 'hf-tpl-card is-inactive-tpl';
                    const statusBadge = t.status === 'publish'
                        ? `<span class="badge badge-published" style="font-size: 10.5px;">Published</span>`
                        : `<span class="badge badge-draft" style="font-size: 10.5px;">Draft</span>`;

                    let behaviorBadges = `<span class="hf-chip hf-chip-rule" title="Evaluation Priority">Priority: ${{t.priority}}</span>`;
                    if (t.template_type === 'header' && t.settings) {{
                        if (t.settings.sticky) behaviorBadges += `<span class="hf-chip" style="background:var(--press-wash);color:var(--press);border:1px solid var(--press);">Sticky</span>`;
                        if (t.settings.transparent) behaviorBadges += `<span class="hf-chip" style="background:var(--press-wash);color:var(--press);border:1px solid var(--press);">✨ Transparent</span>`;
                        if (t.settings.scroll_shrink) behaviorBadges += `<span class="hf-chip" style="background:var(--moss-wash);color:var(--moss);border:1px solid var(--moss);">↕ Shrink</span>`;
                    }} else if (t.template_type === 'footer' && t.settings) {{
                        if (t.settings.sticky_footer) behaviorBadges += `<span class="hf-chip" style="background:var(--press-wash);color:var(--press);border:1px solid var(--press);">Sticky Footer</span>`;
                        if (t.settings.reveal_on_scroll) behaviorBadges += `<span class="hf-chip" style="background:var(--press-wash);color:var(--press);border:1px solid var(--press);">Reveal</span>`;
                    }} else if (t.template_type === 'popup' && t.settings) {{
                        const pVar = t.settings.popup_variant || 'modal';
                        const pTrig = t.settings.trigger || 'on_load';
                        behaviorBadges += `<span class="hf-chip" style="background:var(--press-wash);color:var(--press);border:1px solid var(--press);">Variant: ${{escHtml(pVar)}}</span>`;
                        behaviorBadges += `<span class="hf-chip" style="background:var(--moss-wash);color:var(--moss);border:1px solid var(--moss);">Trigger: ${{escHtml(pTrig)}}</span>`;
                        if (pTrig === 'on_load' && t.settings.delay_sec != null) {{
                            behaviorBadges += `<span class="hf-chip hf-chip-rule">Delay: ${{t.settings.delay_sec}}s</span>`;
                        }} else if (pTrig === 'on_scroll' && t.settings.scroll_pct != null) {{
                            behaviorBadges += `<span class="hf-chip hf-chip-rule">Scroll: ${{t.settings.scroll_pct}}%</span>`;
                        }}
                        if (t.settings.dismiss_days != null) {{
                            behaviorBadges += `<span class="hf-chip hf-chip-rule">Dismiss: ${{t.settings.dismiss_days}}d</span>`;
                        }}
                    }} else if (t.template_type === 'section_symbol') {{
                        behaviorBadges += `<span class="hf-chip" style="background:var(--press-wash);color:var(--press);border:1px solid var(--press);">Synced Global Symbol / Mega Menu Panel</span>`;
                    }}

                    const condPills = (t.conditions && t.conditions.length > 0)
                        ? t.conditions.map(c => {{
                            const isEx = c.condition_type === 'exclude';
                            const cls = isEx ? 'hf-chip hf-chip-exclude' : 'hf-chip hf-chip-rule';
                            let label = (isEx ? 'Exclude: ' : 'Include: ') + formatRuleTargetLabel(c.rule_target, c.target_id);
                            return `<span class="${{cls}}">${{escHtml(label)}}</span>`;
                        }}).join(' ')
                        : `<span class="hf-chip hf-chip-rule">Include: Entire Site (Default)</span>`;

                    const matchedList = cov.matched_pages || [];
                    let matchedHtml = '';
                    if (!isPublished) {{
                        matchedHtml = `<span style="font-size: 12px; color: var(--ink-muted);">Template is currently inactive or draft — enable toggle to apply on site.</span>`;
                    }} else if (matchedList.length === 0) {{
                        matchedHtml = `<span style="font-size: 12px; color: var(--ink);">Not currently winning on any page (assign a page below or edit conditions).</span>`;
                    }} else {{
                        const visibleChips = matchedList.slice(0, 10).map(mp => {{
                            const chipCls = mp.is_route ? 'hf-chip hf-chip-route' : 'hf-chip hf-chip-page';
                            const icon = mp.is_route ? '' : '';
                            return `<a href="${{escHtml(mp.permalink)}}" target="_blank" class="${{chipCls}}" title="View live: ${{escHtml(mp.permalink)}}">${{icon}} ${{escHtml(mp.title)}} &nearr;</a>`;
                        }}).join(' ');
                        const moreCount = matchedList.length > 10 ? `<span class="hf-chip hf-chip-rule">+${{matchedList.length - 10}} more</span>` : '';
                        matchedHtml = `${{visibleChips}} ${{moreCount}}`;
                    }}

                    const quickAssignOptions = pages.map(p => `<option value="page:${{p.post_id}}">Page: ${{escHtml(p.title)}} (${{escHtml(p.permalink)}})</option>`).join('');
                    const showQuickAssign = (t.template_type === 'header' || t.template_type === 'footer');

                    return `
                        <div class="${{cardCls}}" data-tpl-id="${{t.id}}">
                            <div style="display: flex; justify-content: space-between; align-items: flex-start; gap: 12px;">
                                <div>
                                    <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                                        <strong style="font-size: 16px; color: var(--ink);">${{escHtml(t.title)}}</strong>
                                        <span style="font-size: 11px; font-family: var(--font-mono); color: var(--ink-muted);">#${{t.id}}</span>
                                        <span class="hf-chip" style="background:var(--paper-sunk);color:var(--ink-muted);font-size:10.5px;text-transform:uppercase;">${{escHtml(t.template_type)}}</span>
                                        ${{statusBadge}}
                                    </div>
                                    <div style="display: flex; gap: 6px; flex-wrap: wrap; margin-top: 8px;">
                                        ${{behaviorBadges}}
                                    </div>
                                </div>
                                <div style="display: flex; align-items: center; gap: 8px;" title="Toggle Active in Public SSR">
                                    <span style="font-size: 11.5px; font-weight: 600; color: ${{t.is_active ? '#059669' : '#64748b'}};">${{t.is_active ? 'Active' : 'Inactive'}}</span>
                                    <label class="hf-switch">
                                        <input type="checkbox" ${{t.is_active ? 'checked' : ''}} onchange="toggleTemplateActive(${{t.id}}, this.checked)" />
                                        <span class="hf-slider"></span>
                                    </label>
                                </div>
                            </div>

                            <!-- Condition Rules -->
                            <div style="background: var(--paper-raised); border: 1px solid var(--rule); border-radius: var(--radius-sm); padding: 10px 12px;">
                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                                    <span style="font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.04em; color: var(--ink-muted);">Display Condition Rules</span>
                                    <button type="button" onclick="openConditionsModal(${{t.id}})" style="background: none; border: none; color: var(--press); font-size: 11.5px; font-weight: 600; cursor: pointer;">Edit Rules &rarr;</button>
                                </div>
                                <div style="display: flex; flex-wrap: wrap; gap: 6px;">
                                    ${{condPills}}
                                </div>
                            </div>

                            <!-- Resolved Pages Where Applied -->
                            <div style="background: var(--paper-raised); border: 1px solid var(--rule); border-radius: var(--radius-sm); padding: 10px 12px;">
                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; flex-wrap: wrap; gap: 8px;">
                                    <span style="font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.04em; color: var(--ink);">
                                        Applied To Pages &amp; Routes (${{matchedList.length}})
                                    </span>
                                    ${{showQuickAssign ? `
                                        <select class="form-select" style="width: auto; max-width: 220px; height: 26px; padding: 2px 8px; font-size: 11.5px;" onchange="quickAssignTemplateToTarget(${{t.id}}, '${{t.template_type}}', this.value); this.selectedIndex = 0;">
                                            <option value="">+ Quick Assign to Page / Route...</option>
                                            <option value="route:entire_site">Entire Site (Global Default)</option>
                                            <option value="route:front_page">Front Page Only</option>
                                            <option value="route:all_pages">All Static Pages</option>
                                            <option value="route:all_posts">All Blog Posts</option>
                                            <option value="route:archive">All Archives</option>
                                            <option value="route:not_found">404 Not Found Page</option>
                                            ${{quickAssignOptions}}
                                        </select>
                                    ` : ''}}
                                </div>
                                <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center;">
                                    ${{matchedHtml}}
                                </div>
                            </div>

                            <!-- Card Footer Actions -->
                            <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 6px; border-top: 1px solid var(--rule); flex-wrap: wrap; gap: 8px;">
                                <div style="display: flex; gap: 6px; flex-wrap: wrap;">
                                    <a href="/d-admin/builder?template_id=${{t.id}}" class="btn btn-primary" style="font-size: 12px; padding: 5px 12px; text-decoration: none;">Edit in DayStudio</a>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openConditionsModal(${{t.id}})">Conditions</button>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openSettingsModal(${{t.id}})">Settings</button>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openPreviewModal(${{t.id}}, '${{escHtml(t.title)}}')">Preview</button>
                                </div>
                                <div style="display: flex; gap: 6px;">
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 9px;" title="Duplicate Template" onclick="duplicateTemplate(${{t.id}})">⧉ Clone</button>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 9px; color: var(--oxide);" title="Delete Template" onclick="deleteTemplate(${{t.id}}, '${{escHtml(t.title)}}')">Trash</button>
                                </div>
                            </div>
                        </div>
                    `;
                }}).join('');
            }}

            function formatRuleTargetLabel(ruleTarget, targetId) {{
                const pages = hfMatrix.pages || [];
                const posts = hfMatrix.posts || [];
                if (ruleTarget === 'entire_site') return 'Entire Site';
                if (ruleTarget === 'front_page') return 'Front Page';
                if (ruleTarget === 'all_pages') return 'All Pages';
                if (ruleTarget === 'all_posts') return 'All Blog Posts';
                if (ruleTarget === 'archive') return 'All Archives';
                if (ruleTarget === 'not_found') return '404 Error Page';
                if (ruleTarget === 'page') {{
                    const found = pages.find(p => String(p.post_id) === String(targetId));
                    return found ? `Page: ${{found.title}} (#${{targetId}})` : `Page #${{targetId}}`;
                }}
                if (ruleTarget === 'post') {{
                    const found = posts.find(p => String(p.post_id) === String(targetId));
                    return found ? `Post: ${{found.title}} (#${{targetId}})` : `Post #${{targetId}}`;
                }}
                return targetId ? `${{ruleTarget}} (${{targetId}})` : ruleTarget;
            }}

            // -------------------------------------------------------------
            // MATRIX RENDERING & 1-CLICK ASSIGNMENT
            // -------------------------------------------------------------
            function renderMatrixTables() {{
                const scope = document.getElementById('matrixScopeSelect')?.value || 'all';
                const q = (document.getElementById('matrixSearchInput')?.value || '').trim().toLowerCase();

                const routesCard = document.getElementById('matrixRoutesCard');
                const pagesCard = document.getElementById('matrixPagesCard');
                const heading = document.getElementById('matrixItemsHeading');

                if (routesCard) routesCard.style.display = (scope === 'all' || scope === 'routes' || scope === 'overrides') ? 'block' : 'none';
                if (pagesCard) pagesCard.style.display = (scope !== 'routes') ? 'block' : 'none';

                const templates = hfMatrix.templates || [];
                const headerTemplates = templates.filter(t => t.template_type === 'header');
                const footerTemplates = templates.filter(t => t.template_type === 'footer');

                // 1. Routes tbody
                const routesTbody = document.getElementById('matrixRoutesTbody');
                if (routesTbody) {{
                    const filteredRoutes = (hfMatrix.routes || []).filter(r => {{
                        if (scope === 'overrides' && r.header.mode === 'inherit' && r.footer.mode === 'inherit') return false;
                        if (q && !r.title.toLowerCase().includes(q) && !r.permalink.toLowerCase().includes(q)) return false;
                        return true;
                    }});
                    routesTbody.innerHTML = filteredRoutes.map(r => renderMatrixRowHtml(r, headerTemplates, footerTemplates, true)).join('')
                        || `<tr><td colspan="5" style="text-align:center;color:var(--ink-muted);padding:20px;">No matching system routes</td></tr>`;
                }}

                // 2. Pages / Posts tbody
                const pagesTbody = document.getElementById('matrixPagesTbody');
                if (pagesTbody) {{
                    let sourceItems = [];
                    if (scope === 'posts') {{
                        sourceItems = hfMatrix.posts || [];
                        if (heading) heading.textContent = 'Individual Blog Posts';
                    }} else if (scope === 'overrides') {{
                        sourceItems = [...(hfMatrix.pages || []), ...(hfMatrix.posts || [])].filter(r => r.header.mode !== 'inherit' || r.footer.mode !== 'inherit');
                        if (heading) heading.textContent = 'Pages & Posts with Custom Header/Footer Overrides';
                    }} else {{
                        sourceItems = hfMatrix.pages || [];
                        if (heading) heading.textContent = 'Individual Site Pages';
                    }}

                    const filteredItems = sourceItems.filter(r => {{
                        if (q && !r.title.toLowerCase().includes(q) && !r.permalink.toLowerCase().includes(q)) return false;
                        return true;
                    }});

                    pagesTbody.innerHTML = filteredItems.map(r => renderMatrixRowHtml(r, headerTemplates, footerTemplates, false)).join('')
                        || `<tr><td colspan="5" style="text-align:center;color:var(--ink-muted);padding:24px;">No matching items found</td></tr>`;
                }}
            }}

            function renderMatrixRowHtml(row, headerTemplates, footerTemplates, isRoute) {{
                let badges = '';
                if (row.is_front_page && !isRoute) {{
                    badges += `<span style="display:inline-flex;align-items:center;font-size:10.5px;background:var(--moss-wash);color:var(--moss);border:1px solid var(--moss);padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:700;">Front Page</span>`;
                }}
                if (row.in_main_menu) {{
                    badges += `<span style="display:inline-flex;align-items:center;font-size:10.5px;background:var(--press-wash);color:var(--press);border:1px solid var(--press);padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:600;">Menu</span>`;
                }}
                if (row.status === 'draft') {{
                    badges += `<span class="badge badge-draft" style="font-size:10px;margin-left:6px;">Draft</span>`;
                }}

                const subLinks = isRoute
                    ? `<div class="table-row-slug">Route Pattern: <code>${{escHtml(row.permalink)}}</code></div>`
                    : `<div class="table-row-slug">
                        Permalink: <a href="${{escHtml(row.permalink)}}" target="_blank" style="color:var(--press);text-decoration:none;">${{escHtml(row.permalink)}} &nearr;</a>
                        &bull; <a href="/d-admin/post?post=${{row.post_id}}&action=edit&post_type=${{row.target_kind}}" style="color:var(--ink-muted);text-decoration:none;">Edit Page</a>
                        &bull; <a href="/d-admin/builder?post=${{row.post_id}}" style="color:var(--press);text-decoration:none;font-weight:600;">DayStudio</a>
                       </div>`;

                const hStudioLink = row.header.template_id
                    ? `<a href="/d-admin/builder?template_id=${{row.header.template_id}}" title="Edit this Header in DayStudio" style="font-size:11px;color:var(--press);text-decoration:none;font-weight:600;margin-left:4px;">Edit</a>`
                    : '';
                const fStudioLink = row.footer.template_id
                    ? `<a href="/d-admin/builder?template_id=${{row.footer.template_id}}" title="Edit this Footer in DayStudio" style="font-size:11px;color:var(--press);text-decoration:none;font-weight:600;margin-left:4px;">Edit</a>`
                    : '';

                const hSelectHtml = buildSlotSelectHtml('header', row, row.header, headerTemplates, isRoute);
                const fSelectHtml = buildSlotSelectHtml('footer', row, row.footer, footerTemplates, isRoute);

                return `
                    <tr>
                        <td>
                            <strong style="color:var(--ink);font-size:13.5px;">${{escHtml(row.title)}}</strong>${{badges}}
                            ${{subLinks}}
                        </td>
                        <td>
                            <div>
                                <span class="hf-slot-badge ${{escHtml(row.header.badge_kind)}}">${{escHtml(row.header.display_name)}}</span>
                                ${{hStudioLink}}
                            </div>
                            <div style="font-size:11px;color:var(--ink-muted);margin-top:3px;">${{escHtml(row.header.reason)}}</div>
                        </td>
                        <td>${{hSelectHtml}}</td>
                        <td>
                            <div>
                                <span class="hf-slot-badge ${{escHtml(row.footer.badge_kind)}}">${{escHtml(row.footer.display_name)}}</span>
                                ${{fStudioLink}}
                            </div>
                            <div style="font-size:11px;color:var(--ink-muted);margin-top:3px;">${{escHtml(row.footer.reason)}}</div>
                        </td>
                        <td>${{fSelectHtml}}</td>
                    </tr>
                `;
            }}

            function buildSlotSelectHtml(slot, row, slotStatus, templates, isRoute) {{
                const curMode = slotStatus.mode || 'inherit';
                const targetArg = isRoute ? `'route', '${{row.route_key}}'` : `'post', ${{row.post_id}}`;
                const tplOptions = templates.map(t => {{
                    const val = `template:${{t.id}}`;
                    const sel = curMode === val ? 'selected' : '';
                    return `<option value="${{val}}" ${{sel}}>Assign: ${{escHtml(t.title)}} (#${{t.id}})</option>`;
                }}).join('');

                return `
                    <select class="form-select" style="font-size:12px;padding:5px 8px;height:32px;" onchange="assignSlotFromMatrix(${{targetArg}}, '${{slot}}', this.value)">
                        <option value="inherit" ${{curMode === 'inherit' ? 'selected' : ''}}>Inherit Global Rules</option>
                        ${{tplOptions}}
                        <option value="fallback" ${{curMode === 'fallback' ? 'selected' : ''}}>Built-in Theme ${{slot === 'header' ? 'Header' : 'Footer'}}</option>
                        <option value="none" ${{curMode === 'none' ? 'selected' : ''}}>Hide (No ${{slot === 'header' ? 'Header' : 'Footer'}})</option>
                    </select>
                `;
            }}

            async function assignSlotFromMatrix(kind, idOrKey, slot, value) {{
                const payload = kind === 'route'
                    ? {{ route_key: String(idOrKey), [slot]: value }}
                    : {{ post_id: Number(idOrKey), [slot]: value }};
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
                        showToast('✓ Updated ' + slot + ' assignment in SQLite');
                    }} else {{
                        showToast('Failed to update assignment', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            async function quickAssignTemplateToTarget(templateId, slot, rawTarget) {{
                if (!rawTarget) return;
                const [kind, val] = rawTarget.split(':');
                if (kind === 'page') {{
                    await assignSlotFromMatrix('post', Number(val), slot, `template:${{templateId}}`);
                }} else if (kind === 'route') {{
                    await assignSlotFromMatrix('route', val, slot, `template:${{templateId}}`);
                }}
            }}

            async function toggleTemplateActive(templateId, isActive) {{
                try {{
                    const res = await fetch(`/wp-json/daypress/v1/templates/${{templateId}}`, {{
                        method: 'PUT',
                        headers: {{ 'Content-Type': 'application/json' }},
                        credentials: 'same-origin',
                        body: JSON.stringify({{ is_active: isActive, status: isActive ? 'publish' : undefined }})
                    }});
                    if (res.ok) {{
                        await refreshMatrixFromServer();
                        showToast(isActive ? '✓ Template activated for public SSR' : 'Template deactivated');
                    }} else {{
                        showToast('Failed to update template status', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            async function duplicateTemplate(templateId) {{
                try {{
                    const res = await fetch(`/wp-json/daypress/v1/templates/${{templateId}}/duplicate`, {{
                        method: 'POST',
                        credentials: 'same-origin'
                    }});
                    if (res.ok) {{
                        await refreshMatrixFromServer();
                        showToast('✓ Template duplicated in SQLite');
                    }} else {{
                        showToast('Failed to duplicate template', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            async function deleteTemplate(templateId, title) {{
                if (!confirm(`Permanently delete template "${{title}}" (#${{templateId}}) and its display conditions from SQLite?`)) return;
                try {{
                    const res = await fetch(`/wp-json/daypress/v1/templates/${{templateId}}?force=true`, {{
                        method: 'DELETE',
                        credentials: 'same-origin'
                    }});
                    if (res.ok) {{
                        await refreshMatrixFromServer();
                        showToast(`✓ Deleted "${{title}}"`);
                    }} else {{
                        showToast('Failed to delete template', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            // -------------------------------------------------------------
            // MODAL 1: CREATE TEMPLATE
            // -------------------------------------------------------------
            function openCreateModal(defaultType) {{
                const typeSel = document.getElementById('createTplType');
                if (typeSel && defaultType) typeSel.value = defaultType;
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
                if (popupBox) {{
                    popupBox.style.display = t === 'popup' ? 'flex' : 'none';
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
                    showToast('Please enter a template name', true);
                    return;
                }}
                const template_type = document.getElementById('createTplType').value;
                const starter = document.getElementById('createTplStarter').value;
                const ruleTarget = document.getElementById('createTplTarget').value;
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
                        if (!res.ok) throw new Error('Failed to create template');
                        created = await res.json();
                    }}

                    closeCreateModal();
                    if (openInStudio && created && created.id) {{
                        window.location.href = `/d-admin/builder?template_id=${{created.id}}`;
                        return;
                    }}
                    await refreshMatrixFromServer();
                    showToast(`✓ Created "${{title}}" in SQLite`);
                }} catch (e) {{
                    showToast('Error: ' + e.message, true);
                }}
            }}

            // -------------------------------------------------------------
            // MODAL 2: DISPLAY CONDITIONS & SPECIFIC PAGE PICKER
            // -------------------------------------------------------------
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

                container.innerHTML = conds.map((c, idx) => {{
                    const rt = c.rule_target || 'entire_site';
                    const showPageSel = rt === 'page';
                    const showPostSel = rt === 'post';
                    const showTextId = rt === 'category' || rt === 'tag' || rt === 'author';

                    const pageOpts = pages.map(p => `<option value="${{p.post_id}}" ${{String(c.target_id) === String(p.post_id) ? 'selected' : ''}}>${{escHtml(p.title)}} (${{escHtml(p.permalink)}})</option>`).join('');
                    const postOpts = posts.map(p => `<option value="${{p.post_id}}" ${{String(c.target_id) === String(p.post_id) ? 'selected' : ''}}>${{escHtml(p.title)}} (${{escHtml(p.permalink)}})</option>`).join('');

                    return `
                        <div class="hf-cond-row" style="display:flex;gap:8px;align-items:center;background:var(--paper-raised);border:1px solid var(--rule-strong);padding:10px;border-radius:var(--radius-sm);flex-wrap:wrap;">
                            <select class="form-select cond-row-type" style="width:120px;font-weight:700;">
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
                            <button type="button" class="btn btn-secondary" style="color:var(--oxide);padding:6px 10px;" onclick="this.closest('.hf-cond-row').remove()">✕</button>
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
                        showToast('✓ Display conditions saved & page coverage updated!');
                    }} else {{
                        showToast('Failed to save conditions', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            // -------------------------------------------------------------
            // MODAL 3: SETTINGS & BEHAVIOR
            // -------------------------------------------------------------
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
                        showToast('✓ Template settings saved to SQLite');
                    }} else {{
                        showToast('Failed to save template settings', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            // -------------------------------------------------------------
            // MODAL 4: LIVE PREVIEW
            // -------------------------------------------------------------
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

            // -------------------------------------------------------------
            // MODAL 5: 1-CLICK STARTERS CATALOG
            // -------------------------------------------------------------
            function openStarterCatalogModal() {{
                const grid = document.getElementById('hfStartersGrid');
                grid.innerHTML = hfStarters.map(s => `
                    <div class="day-card" style="padding: 16px; display: flex; flex-direction: column; justify-content: space-between; gap: 12px; border: 1px solid var(--rule-strong);">
                        <div>
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                                <strong style="font-size: 14px; color: var(--ink);">${{escHtml(s.title)}}</strong>
                                <span class="hf-chip" style="background:var(--press-wash);color:var(--press);font-size:10.5px;text-transform:uppercase;">${{escHtml(s.template_type)}}</span>
                            </div>
                            <p style="font-size: 12px; color: var(--ink-muted); margin: 0;">${{escHtml(s.description)}}</p>
                        </div>
                        <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 10px; border-top: 1px solid var(--rule);">
                            <span style="font-size: 11px; color: var(--ink-muted); font-family: var(--font-mono);">Category: ${{escHtml(s.category)}}</span>
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
                        showToast(`✓ Installed starter "${{title}}"`);
                    }} else {{
                        showToast('Failed to install starter template', true);
                    }}
                }} catch (e) {{
                    showToast('Network error: ' + e.message, true);
                }}
            }}

            document.addEventListener('DOMContentLoaded', () => {{
                const params = new URLSearchParams(window.location.search);
                if (params.get('tab') === 'matrix') {{
                    switchMainTab('matrix');
                }} else if (params.get('tab') === 'layouts') {{
                    switchMainTab('layouts');
                }}
                renderAllViews();
            }});
        </script>
        "##,
        initial_matrix_json = initial_matrix_json,
        starters_json = starters_json,
    )
}
