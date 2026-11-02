//! Theme Header & Footer Manager Admin Screen (`/d-admin/headers-footers`).
//!
//! Provides a unified, zero-mock production interface to:
//! 1. View and manage all Header, Footer, and Layout templates stored in `daypress_templates`.
//! 2. Inspect exactly which Pages and System Routes each Header/Footer is applied to.
//! 3. Directly assign or override the Header and Footer on any individual Page, Post, or System Route
//!    via the two-way Page Assignment Matrix (`daypress_template_conditions` + `postmeta`).
//! 4. Configure Sticky/Transparent/Priority settings, edit display conditions with specific Page/Post
//!    selectors, preview templates live, install 1-Click Starters, and launch directly into DayStudio.

use super::layout::render_admin_shell;
use crate::models::DaypressSettingsResponse;
use crate::templates::utils::escape_html;
use crate::theme_builder::models::{HeaderFooterMatrixResponse, StarterTemplateInfo, TemplateType};

pub fn render_headers_footers_view(
    settings: &DaypressSettingsResponse,
    matrix: &HeaderFooterMatrixResponse,
    starters: &[StarterTemplateInfo],
) -> String {
    let total_headers = matrix
        .templates
        .iter()
        .filter(|t| t.template_type == TemplateType::Header)
        .count();
    let active_headers = matrix
        .templates
        .iter()
        .filter(|t| t.template_type == TemplateType::Header && t.is_active && t.status == "publish")
        .count();

    let total_footers = matrix
        .templates
        .iter()
        .filter(|t| t.template_type == TemplateType::Footer)
        .count();
    let active_footers = matrix
        .templates
        .iter()
        .filter(|t| t.template_type == TemplateType::Footer && t.is_active && t.status == "publish")
        .count();

    let other_templates = matrix
        .templates
        .iter()
        .filter(|t| !matches!(t.template_type, TemplateType::Header | TemplateType::Footer))
        .count();

    let total_pages = matrix.pages.len();
    let custom_overrides = matrix
        .pages
        .iter()
        .chain(matrix.routes.iter())
        .filter(|r| r.header.mode != "inherit" || r.footer.mode != "inherit")
        .count();

    let initial_matrix_json = serde_json::to_string(matrix)
        .unwrap_or_else(|_| "{}".to_string())
        .replace("</script", "<\\/script")
        .replace("</SCRIPT", "<\\/SCRIPT");

    let starters_json = serde_json::to_string(starters)
        .unwrap_or_else(|_| "[]".to_string())
        .replace("</script", "<\\/script")
        .replace("</SCRIPT", "<\\/SCRIPT");

    let active_theme_name = if settings.active_theme.trim().is_empty() {
        "Daypress Modern"
    } else {
        settings.active_theme.trim()
    };

    let body = format!(
        r##"
        <style>
            .hf-tabs {{
                display: inline-flex;
                background: var(--paper-sunk);
                padding: 4px;
                border-radius: var(--radius-sm);
                border: 1px solid var(--rule);
                gap: 4px;
                margin-bottom: 20px;
            }}
            .hf-tab-btn {{
                padding: 8px 16px;
                border-radius: 7px;
                border: none;
                background: transparent;
                color: var(--ink-muted);
                font-size: 13px;
                font-weight: 600;
                cursor: pointer;
                display: inline-flex;
                align-items: center;
                gap: 8px;
                transition: all 0.15s ease;
            }}
            .hf-tab-btn.active {{
                background: var(--paper-raised);
                color: var(--ink);
                box-shadow: 0 1px 3px rgba(15, 23, 42, 0.1);
            }}
            .hf-tab-count {{
                font-size: 11px;
                padding: 1px 7px;
                border-radius: 999px;
                background: var(--paper-sunk);
                color: var(--ink-muted);
                font-family: var(--font-mono);
            }}
            .hf-tab-btn.active .hf-tab-count {{
                background: var(--press-wash);
                color: var(--press);
            }}
            .hf-kpi-grid {{
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
                gap: 16px;
                margin-bottom: 22px;
            }}
            .hf-kpi-card {{
                background: var(--paper-raised);
                border: 1px solid var(--border-subtle);
                border-radius: var(--radius-sm);
                padding: 16px 18px;
                display: flex;
                flex-direction: column;
                gap: 4px;
            }}
            .hf-kpi-label {{
                font-size: 11.5px;
                font-weight: 600;
                text-transform: uppercase;
                letter-spacing: 0.04em;
                color: var(--ink-muted);
            }}
            .hf-kpi-value {{
                font-size: 24px;
                font-weight: 800;
                color: var(--ink);
                letter-spacing: -0.02em;
            }}
            .hf-kpi-sub {{
                font-size: 12px;
                color: var(--ink-muted);
            }}
            .hf-tpl-grid {{
                display: grid;
                grid-template-columns: repeat(auto-fill, minmax(440px, 1fr));
                gap: 18px;
                margin-bottom: 28px;
            }}
            .hf-tpl-card {{
                background: var(--paper-raised);
                border: 1px solid var(--rule-strong);
                border-radius: var(--radius-sm);
                padding: 20px;
                display: flex;
                flex-direction: column;
                gap: 14px;
                transition: border-color 0.15s ease, box-shadow 0.15s ease;
            }}
            .hf-tpl-card.is-active-tpl {{
                border-color: var(--press);
                box-shadow: 0 4px 14px -4px rgba(37, 99, 235, 0.1);
            }}
            .hf-tpl-card.is-inactive-tpl {{
                opacity: 0.78;
                background: var(--paper-raised);
            }}
            .hf-chip {{
                display: inline-flex;
                align-items: center;
                gap: 5px;
                padding: 3px 9px;
                border-radius: 6px;
                font-size: 11.5px;
                font-weight: 600;
                text-decoration: none;
            }}
            .hf-chip-page {{
                background: var(--press-wash);
                color: var(--press);
                border: 1px solid var(--press);
            }}
            .hf-chip-route {{
                background: var(--moss-wash);
                color: var(--moss);
                border: 1px solid var(--moss);
            }}
            .hf-chip-rule {{
                background: var(--paper-raised);
                color: var(--ink-muted);
                border: 1px solid var(--rule-strong);
                font-family: var(--font-mono);
                font-size: 11px;
            }}
            .hf-chip-exclude {{
                background: var(--oxide-wash);
                color: var(--oxide);
                border: 1px solid var(--oxide);
                font-family: var(--font-mono);
                font-size: 11px;
            }}
            .hf-slot-badge {{
                display: inline-flex;
                align-items: center;
                gap: 6px;
                padding: 3px 10px;
                border-radius: 6px;
                font-size: 12px;
                font-weight: 600;
            }}
            .hf-slot-badge.custom {{
                background: var(--press-wash);
                color: var(--press);
                border: 1px solid var(--press);
            }}
            .hf-slot-badge.global {{
                background: var(--moss-wash);
                color: var(--moss);
                border: 1px solid var(--moss);
            }}
            .hf-slot-badge.hidden {{
                background: var(--oxide-wash);
                color: var(--oxide);
                border: 1px solid var(--oxide);
            }}
            .hf-slot-badge.fallback {{
                background: var(--paper-sunk);
                color: var(--ink-muted);
                border: 1px solid var(--rule-strong);
            }}
            .hf-modal-backdrop {{
                display: none;
                position: fixed;
                inset: 0;
                background: rgba(15, 23, 42, 0.55);
                z-index: 99999;
                align-items: center;
                justify-content: center;
                padding: 20px;
            }}
            .hf-modal-backdrop.open {{
                display: flex;
            }}
            .hf-modal {{
                background: var(--paper-raised);
                border-radius: var(--radius-sm);
                width: min(680px, 95vw);
                max-height: 90vh;
                display: flex;
                flex-direction: column;
                box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.35);
                overflow: hidden;
            }}
            .hf-modal-header {{
                padding: 16px 22px;
                border-bottom: 1px solid var(--rule);
                display: flex;
                justify-content: space-between;
                align-items: center;
            }}
            .hf-modal-body {{
                padding: 20px 22px;
                overflow-y: auto;
                flex: 1;
            }}
            .hf-modal-footer {{
                padding: 14px 22px;
                border-top: 1px solid var(--rule);
                background: var(--paper-raised);
                display: flex;
                justify-content: flex-end;
                gap: 10px;
            }}
            .hf-switch {{
                position: relative;
                display: inline-block;
                width: 38px;
                height: 22px;
            }}
            .hf-switch input {{
                opacity: 0;
                width: 0;
                height: 0;
            }}
            .hf-slider {{
                position: absolute;
                cursor: pointer;
                inset: 0;
                background-color: var(--paper-sunk);
                transition: .2s;
                border-radius: 22px;
            }}
            .hf-slider:before {{
                position: absolute;
                content: "";
                height: 16px;
                width: 16px;
                left: 3px;
                bottom: 3px;
                background-color: white;
                transition: .2s;
                border-radius: 50%;
            }}
            .hf-switch input:checked + .hf-slider {{
                background-color: var(--moss);
            }}
            .hf-switch input:checked + .hf-slider:before {{
                transform: translateX(16px);
            }}
        </style>

        <div class="page-header">
            <div>
                <div style="display: flex; align-items: center; gap: 10px;">
                    <h1 class="page-title">Theme Builder &amp; Template Manager</h1>
                    <span class="badge badge-published" style="font-size: 11px;">Theme: {active_theme}</span>
                </div>
                <p class="page-subtitle">Manage Headers, Footers, Single Post/Page layouts, Archives, 404 pages, Synced Symbols, Mega Menus, and Popups</p>
            </div>
            <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                <button type="button" class="btn btn-secondary" onclick="openStarterCatalogModal()">1-Click Starters</button>
                <button type="button" class="btn btn-secondary" onclick="openCreateModal('section_symbol')">+ New Symbol</button>
                <button type="button" class="btn btn-secondary" onclick="openCreateModal('popup')">+ New Popup</button>
                <button type="button" class="btn btn-secondary" onclick="openCreateModal('footer')">+ New Footer</button>
                <button type="button" class="btn btn-primary" onclick="openCreateModal('header')">+ New Header</button>
            </div>
        </div>

        <!-- KPI Summary Strip -->
        <div class="hf-kpi-grid">
            <div class="hf-kpi-card">
                <span class="hf-kpi-label">Headers</span>
                <div class="hf-kpi-value" id="kpiHeadersVal">{active_headers} <span style="font-size: 14px; font-weight: 500; color: var(--ink-muted);">/ {total_headers} active</span></div>
                <span class="hf-kpi-sub">Site navigation &amp; top bars</span>
            </div>
            <div class="hf-kpi-card">
                <span class="hf-kpi-label">Footers</span>
                <div class="hf-kpi-value" id="kpiFootersVal">{active_footers} <span style="font-size: 14px; font-weight: 500; color: var(--ink-muted);">/ {total_footers} active</span></div>
                <span class="hf-kpi-sub">Site-wide &amp; page-specific footers</span>
            </div>
            <div class="hf-kpi-card">
                <span class="hf-kpi-label">Site Pages Tracked</span>
                <div class="hf-kpi-value" id="kpiPagesVal">{total_pages}</div>
                <span class="hf-kpi-sub">100% SSR condition resolution</span>
            </div>
            <div class="hf-kpi-card">
                <span class="hf-kpi-label">Page &amp; Route Overrides</span>
                <div class="hf-kpi-value" id="kpiOverridesVal">{custom_overrides}</div>
                <span class="hf-kpi-sub">Specific page &amp; hidden rules</span>
            </div>
            <div class="hf-kpi-card">
                <span class="hf-kpi-label">Layout &amp; Symbol Templates</span>
                <div class="hf-kpi-value" id="kpiOtherVal">{other_templates}</div>
                <span class="hf-kpi-sub">Single, Archive, 404, Symbols &amp; Popups</span>
            </div>
        </div>

        <!-- Main Workspace Navigation Tabs -->
        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 12px; margin-bottom: 14px;">
            <div class="hf-tabs" role="tablist" style="margin-bottom: 0;">
                <button type="button" class="hf-tab-btn active" id="tabBtnTemplates" onclick="switchMainTab('templates')">
                    <span>Headers &amp; Footers</span>
                    <span class="hf-tab-count" id="tabCountHF">{hf_total}</span>
                </button>
                <button type="button" class="hf-tab-btn" id="tabBtnMatrix" onclick="switchMainTab('matrix')">
                    <span>Page Assignment Matrix</span>
                    <span class="hf-tab-count" id="tabCountPages">{total_pages}</span>
                </button>
                <button type="button" class="hf-tab-btn" id="tabBtnLayouts" onclick="switchMainTab('layouts')">
                    <span>Theme Layouts, Symbols &amp; Popups</span>
                    <span class="hf-tab-count" id="tabCountOther">{other_templates}</span>
                </button>
            </div>

            <div id="templatesTypeFilterBar" style="display: flex; gap: 6px; flex-wrap: wrap;">
                <button type="button" class="btn btn-secondary hf-filter-pill" data-type="all" style="background: var(--ink); color: var(--paper-raised); font-size: 12px; padding: 5px 11px;" onclick="setTemplateFilter('all', this)">All Templates</button>
                <button type="button" class="btn btn-secondary hf-filter-pill" data-type="header" style="font-size: 12px; padding: 5px 11px;" onclick="setTemplateFilter('header', this)">Header</button>
                <button type="button" class="btn btn-secondary hf-filter-pill" data-type="footer" style="font-size: 12px; padding: 5px 11px;" onclick="setTemplateFilter('footer', this)">Footer</button>
                <button type="button" class="btn btn-secondary hf-filter-pill" data-type="single_post" style="font-size: 12px; padding: 5px 11px;" onclick="setTemplateFilter('single_post', this)">Single Post</button>
                <button type="button" class="btn btn-secondary hf-filter-pill" data-type="single_page" style="font-size: 12px; padding: 5px 11px;" onclick="setTemplateFilter('single_page', this)">Single Page</button>
                <button type="button" class="btn btn-secondary hf-filter-pill" data-type="archive" style="font-size: 12px; padding: 5px 11px;" onclick="setTemplateFilter('archive', this)">Archive</button>
                <button type="button" class="btn btn-secondary hf-filter-pill" data-type="404_page" style="font-size: 12px; padding: 5px 11px;" onclick="setTemplateFilter('404_page', this)">404 Page</button>
                <button type="button" class="btn btn-secondary hf-filter-pill" data-type="section_symbol" style="font-size: 12px; padding: 5px 11px;" onclick="setTemplateFilter('section_symbol', this)">Synced Symbols</button>
                <button type="button" class="btn btn-secondary hf-filter-pill" data-type="popup" style="font-size: 12px; padding: 5px 11px;" onclick="setTemplateFilter('popup', this)">Popups &amp; Bars</button>
            </div>
        </div>

        <!-- TAB 1: HEADERS & FOOTERS OVERVIEW -->
        <div id="tabPanelTemplates">
            <div id="headersSectionWrap">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                    <div>
                        <h2 style="font-size: 16px; font-weight: 700; color: var(--ink); margin: 0;">Created Headers</h2>
                        <p style="font-size: 12.5px; color: var(--ink-muted); margin: 2px 0 0 0;">Higher priority and more specific page rules automatically win during SSR.</p>
                    </div>
                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 12px;" onclick="openCreateModal('header')">+ Add Header</button>
                </div>
                <div id="headersCardsContainer" class="hf-tpl-grid"></div>
            </div>

            <div id="footersSectionWrap" style="margin-top: 12px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                    <div>
                        <h2 style="font-size: 16px; font-weight: 700; color: var(--ink); margin: 0;">Created Footers</h2>
                        <p style="font-size: 12.5px; color: var(--ink-muted); margin: 2px 0 0 0;">Manage global and page-specific footers and inspect their resolved page coverage.</p>
                    </div>
                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 12px;" onclick="openCreateModal('footer')">+ Add Footer</button>
                </div>
                <div id="footersCardsContainer" class="hf-tpl-grid"></div>
            </div>
        </div>

        <!-- TAB 2: PAGE ASSIGNMENT MATRIX -->
        <div id="tabPanelMatrix" style="display: none;">
            <div class="day-card" style="padding: 18px 20px; margin-bottom: 20px; background: var(--paper-raised); border: 1px solid var(--rule-strong);">
                <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 12px;">
                    <div>
                        <strong style="font-size: 14px; color: var(--ink);">Live Page &amp; Route Assignment Matrix</strong>
                        <p style="font-size: 12.5px; color: var(--ink-muted); margin: 2px 0 0 0;">
                            Every row shows the <strong>effective Header and Footer</strong> currently rendered on that page or route, and lets you immediately override it with a specific template, built-in fallback, or hide it.
                        </p>
                    </div>
                    <div style="display: flex; gap: 10px; align-items: center; flex-wrap: wrap;">
                        <select id="matrixScopeSelect" class="form-select" style="width: auto; font-size: 12.5px; padding: 6px 10px;" onchange="renderMatrixTables()">
                            <option value="all">Show: System Routes + All Pages</option>
                            <option value="pages">Show: Pages Only</option>
                            <option value="routes">Show: System Routes Only</option>
                            <option value="posts">Show: Blog Posts</option>
                            <option value="overrides">Show: Custom Overrides Only</option>
                        </select>
                        <input type="search" id="matrixSearchInput" class="form-input" placeholder="Search page title or /slug..." style="width: 240px; font-size: 12.5px; padding: 6px 12px;" oninput="renderMatrixTables()" />
                    </div>
                </div>
            </div>

            <!-- System Routes Matrix -->
            <div id="matrixRoutesCard" class="day-card" style="margin-bottom: 24px; overflow: hidden;">
                <div style="padding: 14px 18px; background: var(--paper-raised); border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center;">
                    <div>
                        <strong style="font-size: 13.5px; color: var(--ink);">System Routes &amp; Post-Type Defaults</strong>
                        <span style="font-size: 12px; color: var(--ink-muted); margin-left: 8px;">Controls default headers &amp; footers for Front Page, All Pages, All Posts, Archives, and 404</span>
                    </div>
                </div>
                <div class="day-table-wrapper" style="border: none; border-radius: 0;">
                    <table class="day-table">
                        <thead>
                            <tr>
                                <th style="width: 24%;">System Route</th>
                                <th style="width: 19%;">Effective Header</th>
                                <th style="width: 19%;">Assign Route Header</th>
                                <th style="width: 19%;">Effective Footer</th>
                                <th style="width: 19%;">Assign Route Footer</th>
                            </tr>
                        </thead>
                        <tbody id="matrixRoutesTbody"></tbody>
                    </table>
                </div>
            </div>

            <!-- Individual Pages / Posts Matrix -->
            <div id="matrixPagesCard" class="day-card" style="overflow: hidden;">
                <div style="padding: 14px 18px; background: var(--paper-raised); border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center;">
                    <div>
                        <strong style="font-size: 13.5px; color: var(--ink);" id="matrixItemsHeading">Individual Site Pages</strong>
                        <span style="font-size: 12px; color: var(--ink-muted); margin-left: 8px;">Assign a specific Header or Footer to any individual page, or set to Inherit / Hidden</span>
                    </div>
                </div>
                <div class="day-table-wrapper" style="border: none; border-radius: 0;">
                    <table class="day-table">
                        <thead>
                            <tr>
                                <th style="width: 24%;">Page / Permalink</th>
                                <th style="width: 19%;">Effective Header</th>
                                <th style="width: 19%;">Page Header Override</th>
                                <th style="width: 19%;">Effective Footer</th>
                                <th style="width: 19%;">Page Footer Override</th>
                            </tr>
                        </thead>
                        <tbody id="matrixPagesTbody"></tbody>
                    </table>
                </div>
            </div>
        </div>

        <!-- TAB 3: OTHER LAYOUT TEMPLATES -->
        <div id="tabPanelLayouts" style="display: none;">
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px; flex-wrap: wrap; gap: 10px;">
                <div>
                    <h2 style="font-size: 16px; font-weight: 700; color: var(--ink); margin: 0;" id="otherLayoutsSectionHeading">Single, Archive, 404, Synced Symbols &amp; Popup Templates</h2>
                    <p style="font-size: 12.5px; color: var(--ink-muted); margin: 2px 0 0 0;">Theme Builder layout templates for blog articles, static pages, category/author archives, 404 screens, global synced symbols, mega menu panels, and announcement/popup modals.</p>
                </div>
                <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                    <button type="button" class="btn btn-secondary" style="font-size: 12px;" onclick="openCreateModal('section_symbol')">+ New Synced Symbol</button>
                    <button type="button" class="btn btn-secondary" style="font-size: 12px;" onclick="openCreateModal('popup')">+ New Popup / Bar</button>
                    <button type="button" class="btn btn-primary" style="font-size: 12.5px;" onclick="openCreateModal('single_post')">+ New Layout Template</button>
                </div>
            </div>
            <div id="otherLayoutsCardsContainer" class="hf-tpl-grid"></div>
        </div>

        <!-- MODAL 1: CREATE TEMPLATE / INSTALL STARTER -->
        <div class="hf-modal-backdrop" id="hfCreateModal" onclick="if(event.target===this) closeCreateModal()">
            <div class="hf-modal">
                <div class="hf-modal-header">
                    <strong style="font-size: 15px; color: var(--ink);" id="hfCreateModalTitle">Create New Template</strong>
                    <button type="button" onclick="closeCreateModal()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: var(--ink-muted);">&times;</button>
                </div>
                <div class="hf-modal-body" style="display: flex; flex-direction: column; gap: 14px;">
                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">Template Name *</label>
                        <input type="text" id="createTplTitle" class="form-input" placeholder="e.g. Main Sticky Header" />
                    </div>
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px;">
                        <div class="form-group" style="margin-bottom: 0;">
                            <label class="form-label">Template Type</label>
                            <select id="createTplType" class="form-select" onchange="onCreateTypeChange()">
                                <option value="header">Header (header)</option>
                                <option value="footer">Footer (footer)</option>
                                <option value="single_post">Single Post (single_post)</option>
                                <option value="single_page">Single Page (single_page)</option>
                                <option value="archive">Archive / Category / Author (archive)</option>
                                <option value="404_page">404 Page (404_page)</option>
                                <option value="section_symbol">Synced Symbol / Mega Menu Panel (section_symbol)</option>
                                <option value="popup">Announcement Bar / Slide-In Modal / Popup (popup)</option>
                            </select>
                        </div>
                        <div class="form-group" style="margin-bottom: 0;">
                            <label class="form-label">Starter Foundation</label>
                            <select id="createTplStarter" class="form-select" onchange="onCreateStarterChange()">
                                <option value="blank">Blank Canvas (Clean Root AST)</option>
                            </select>
                        </div>
                    </div>
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px;">
                        <div class="form-group" style="margin-bottom: 0;">
                            <label class="form-label">Initial Display Target</label>
                            <select id="createTplTarget" class="form-select" onchange="onCreateTargetChange()">
                                <option value="entire_site">Entire Site (All Pages &amp; Posts)</option>
                                <option value="front_page">Front Page Only</option>
                                <option value="all_pages">All Static Pages</option>
                                <option value="all_posts">All Blog Posts</option>
                                <option value="archive">All Archives (Category / Tag / Author)</option>
                                <option value="not_found">404 Error Page</option>
                                <option value="page">Specific Page...</option>
                                <option value="unassigned">Unassigned (Reusable Symbol / Do not apply yet)</option>
                            </select>
                        </div>
                        <div class="form-group" id="createTplPageWrap" style="margin-bottom: 0; display: none;">
                            <label class="form-label">Select Page</label>
                            <select id="createTplPageId" class="form-select"></select>
                        </div>
                        <div class="form-group" style="margin-bottom: 0;">
                            <label class="form-label">Priority Score</label>
                            <input type="number" id="createTplPriority" class="form-input" value="10" />
                        </div>
                    </div>

                    <!-- Popup Creation Quick Settings (shown when type === 'popup') -->
                    <div id="createPopupBehaviors" style="background: var(--paper-sunk); border: 1px solid var(--rule); border-radius: var(--radius-sm); padding: 14px; display: none; flex-direction: column; gap: 10px;">
                        <strong style="font-size: 12.5px; color: var(--ink);">Popup / Announcement Bar Trigger &amp; Variant</strong>
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px;">
                            <div class="form-group" style="margin-bottom: 0;">
                                <label class="form-label" style="font-size: 11.5px;">Popup Variant (<code>popup_variant</code>)</label>
                                <select id="createTplPopupVariant" class="form-select">
                                    <option value="announcement_bar_top">Announcement Bar (Top)</option>
                                    <option value="announcement_bar_bottom">Announcement Bar (Bottom)</option>
                                    <option value="slide_in_right">Slide-In Drawer (Right)</option>
                                    <option value="slide_in_left">Slide-In Drawer (Left)</option>
                                    <option value="modal" selected>Center Modal / Popup</option>
                                </select>
                            </div>
                            <div class="form-group" style="margin-bottom: 0;">
                                <label class="form-label" style="font-size: 11.5px;">Trigger Rule (<code>trigger</code>)</label>
                                <select id="createTplPopupTrigger" class="form-select">
                                    <option value="on_load" selected>On Page Load (on_load)</option>
                                    <option value="on_scroll">On Scroll Depth (on_scroll)</option>
                                    <option value="exit_intent">Exit Intent (exit_intent)</option>
                                </select>
                            </div>
                        </div>
                        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px;">
                            <div class="form-group" style="margin-bottom: 0;">
                                <label class="form-label" style="font-size: 11.5px;">Delay Sec (<code>delay_sec</code>)</label>
                                <input type="number" id="createTplPopupDelaySec" class="form-input" min="0" max="300" value="0" />
                            </div>
                            <div class="form-group" style="margin-bottom: 0;">
                                <label class="form-label" style="font-size: 11.5px;">Scroll % (<code>scroll_pct</code>)</label>
                                <input type="number" id="createTplPopupScrollPct" class="form-input" min="0" max="100" value="50" />
                            </div>
                            <div class="form-group" style="margin-bottom: 0;">
                                <label class="form-label" style="font-size: 11.5px;">Dismiss Days (<code>dismiss_days</code>)</label>
                                <input type="number" id="createTplPopupDismissDays" class="form-input" min="0" max="365" value="7" />
                            </div>
                        </div>
                    </div>
                </div>
                <div class="hf-modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="closeCreateModal()">Cancel</button>
                    <button type="button" class="btn btn-secondary" onclick="submitCreateTemplate(false)">Create Template</button>
                    <button type="button" class="btn btn-primary" onclick="submitCreateTemplate(true)">Create &amp; Open in DayStudio</button>
                </div>
            </div>
        </div>

        <!-- MODAL 2: DISPLAY CONDITIONS & PAGE ASSIGNMENTS -->
        <div class="hf-modal-backdrop" id="hfConditionsModal" onclick="if(event.target===this) closeConditionsModal()">
            <div class="hf-modal" style="width: min(760px, 96vw);">
                <div class="hf-modal-header">
                    <div>
                        <strong style="font-size: 15px; color: var(--ink);" id="hfCondModalTitle">Display Conditions</strong>
                        <div style="font-size: 12px; color: var(--ink-muted);">Configure where this template renders across your publication</div>
                    </div>
                    <button type="button" onclick="closeConditionsModal()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: var(--ink-muted);">&times;</button>
                </div>
                <div class="hf-modal-body">
                    <div style="background: var(--paper-raised); border: 1px solid var(--rule); border-radius: var(--radius-sm); padding: 10px 14px; font-size: 12px; color: var(--ink-muted); margin-bottom: 14px;">
                        <strong>Specificity Hierarchy:</strong> Specific Page/Post ID (40) &gt; Front Page / 404 (40) &gt; Category/Tag (30) &gt; All Pages / All Posts (20) &gt; Entire Site (10). Exclusions strictly override inclusions.
                    </div>
                    <div id="hfCondRowsContainer" style="display: flex; flex-direction: column; gap: 10px;"></div>
                    <button type="button" class="btn btn-secondary" style="margin-top: 12px; font-size: 12px;" onclick="addConditionModalRow()">+ Add Condition Rule</button>
                </div>
                <div class="hf-modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="closeConditionsModal()">Cancel</button>
                    <button type="button" class="btn btn-primary" onclick="saveConditionsModal()">Save Conditions to SQLite</button>
                </div>
            </div>
        </div>

        <!-- MODAL 3: TEMPLATE SETTINGS & BEHAVIOR -->
        <div class="hf-modal-backdrop" id="hfSettingsModal" onclick="if(event.target===this) closeSettingsModal()">
            <div class="hf-modal">
                <div class="hf-modal-header">
                    <strong style="font-size: 15px; color: var(--ink);" id="hfSettingsModalTitle">Template Settings &amp; Behavior</strong>
                    <button type="button" onclick="closeSettingsModal()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: var(--ink-muted);">&times;</button>
                </div>
                <div class="hf-modal-body" style="display: flex; flex-direction: column; gap: 14px;">
                    <div style="display: grid; grid-template-columns: 2fr 1fr 1fr; gap: 12px;">
                        <div class="form-group" style="margin-bottom: 0;">
                            <label class="form-label">Template Title</label>
                            <input type="text" id="setTplTitle" class="form-input" />
                        </div>
                        <div class="form-group" style="margin-bottom: 0;">
                            <label class="form-label">Status</label>
                            <select id="setTplStatus" class="form-select">
                                <option value="publish">Published</option>
                                <option value="draft">Draft</option>
                            </select>
                        </div>
                        <div class="form-group" style="margin-bottom: 0;">
                            <label class="form-label">Priority</label>
                            <input type="number" id="setTplPriority" class="form-input" value="10" />
                        </div>
                    </div>

                    <div id="setHeaderBehaviors" style="background: var(--paper-raised); border: 1px solid var(--rule); border-radius: var(--radius-sm); padding: 14px; display: flex; flex-direction: column; gap: 10px;">
                        <strong style="font-size: 12.5px; color: var(--ink);">Header Behavior Controls</strong>
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px;">
                            <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer;">
                                <input type="checkbox" id="setTplSticky" />
                                <span>Sticky Header on Scroll (<code>.dp-sticky</code>)</span>
                            </label>
                            <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer;">
                                <input type="checkbox" id="setTplTransparent" />
                                <span>Transparent Overlay Header (<code>.dp-transparent</code>)</span>
                            </label>
                            <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer;">
                                <input type="checkbox" id="setTplScrollShrink" />
                                <span>Shrink Height on Scroll</span>
                            </label>
                            <div style="display: flex; align-items: center; gap: 8px;">
                                <label style="font-size: 12.5px; color: var(--ink-muted);">Z-Index:</label>
                                <input type="number" id="setTplZIndex" class="form-input" style="width: 90px; height: 30px;" placeholder="999" />
                            </div>
                        </div>
                    </div>

                    <div id="setFooterBehaviors" style="background: var(--paper-raised); border: 1px solid var(--rule); border-radius: var(--radius-sm); padding: 14px; display: none; flex-direction: column; gap: 10px;">
                        <strong style="font-size: 12.5px; color: var(--ink);">Footer Behavior Controls</strong>
                        <div style="display: flex; gap: 20px;">
                            <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer;">
                                <input type="checkbox" id="setTplStickyFooter" />
                                <span>Sticky Bottom Footer</span>
                            </label>
                            <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer;">
                                <input type="checkbox" id="setTplRevealScroll" />
                                <span>Curtain Reveal on Scroll</span>
                            </label>
                        </div>
                    </div>

                    <div id="setPopupBehaviors" style="background: var(--paper-raised); border: 1px solid var(--rule); border-radius: var(--radius-sm); padding: 14px; display: none; flex-direction: column; gap: 10px;">
                        <strong style="font-size: 12.5px; color: var(--ink);">Popup, Slide-In &amp; Announcement Bar Behavior Controls</strong>
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px;">
                            <div class="form-group" style="margin-bottom: 0;">
                                <label class="form-label" style="font-size: 12px;">Popup Variant (<code>popup_variant</code>)</label>
                                <select id="setTplPopupVariant" class="form-select">
                                    <option value="announcement_bar_top">Announcement Bar (Top) — announcement_bar_top</option>
                                    <option value="announcement_bar_bottom">Announcement Bar (Bottom) — announcement_bar_bottom</option>
                                    <option value="slide_in_right">Slide-In Drawer (Right) — slide_in_right</option>
                                    <option value="slide_in_left">Slide-In Drawer (Left) — slide_in_left</option>
                                    <option value="modal">Center Modal / Popup — modal</option>
                                </select>
                            </div>
                            <div class="form-group" style="margin-bottom: 0;">
                                <label class="form-label" style="font-size: 12px;">Trigger Rule (<code>trigger</code>)</label>
                                <select id="setTplPopupTrigger" class="form-select">
                                    <option value="on_load">On Page Load — on_load</option>
                                    <option value="on_scroll">On Scroll Depth — on_scroll</option>
                                    <option value="exit_intent">On Exit Intent — exit_intent</option>
                                </select>
                            </div>
                        </div>
                        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px;">
                            <div class="form-group" style="margin-bottom: 0;">
                                <label class="form-label" style="font-size: 12px;">Delay Seconds (<code>delay_sec</code>)</label>
                                <input type="number" id="setTplPopupDelaySec" class="form-input" min="0" max="300" value="0" />
                            </div>
                            <div class="form-group" style="margin-bottom: 0;">
                                <label class="form-label" style="font-size: 12px;">Scroll Depth % (<code>scroll_pct</code>)</label>
                                <input type="number" id="setTplPopupScrollPct" class="form-input" min="0" max="100" value="50" />
                            </div>
                            <div class="form-group" style="margin-bottom: 0;">
                                <label class="form-label" style="font-size: 12px;">Dismiss Cookie Days (<code>dismiss_days</code>)</label>
                                <input type="number" id="setTplPopupDismissDays" class="form-input" min="0" max="365" value="7" />
                            </div>
                        </div>
                    </div>

                    <div class="form-group" style="margin-bottom: 0;">
                        <label class="form-label">Scoped Custom CSS</label>
                        <textarea id="setTplCustomCss" class="form-textarea" style="min-height: 90px; font-family: var(--font-mono); font-size: 12px;" placeholder="/* Optional custom CSS for this template */"></textarea>
                    </div>
                </div>
                <div class="hf-modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="closeSettingsModal()">Cancel</button>
                    <button type="button" class="btn btn-primary" onclick="saveSettingsModal()">Save Settings</button>
                </div>
            </div>
        </div>

        <!-- MODAL 4: LIVE IFRAME PREVIEW -->
        <div class="hf-modal-backdrop" id="hfPreviewModal" onclick="if(event.target===this) closePreviewModal()">
            <div class="hf-modal" style="width: min(1100px, 96vw); height: 82vh;">
                <div class="hf-modal-header">
                    <div style="display: flex; align-items: center; gap: 12px;">
                        <strong style="font-size: 15px; color: var(--ink);" id="hfPreviewModalTitle">Live Template Preview</strong>
                        <div style="display: inline-flex; gap: 4px; background: var(--paper-sunk); padding: 3px; border-radius: 6px;">
                            <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 3px 8px;" onclick="setPreviewViewport('100%')">Desktop</button>
                            <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 3px 8px;" onclick="setPreviewViewport('768px')">Tablet</button>
                            <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 3px 8px;" onclick="setPreviewViewport('390px')">Mobile</button>
                        </div>
                    </div>
                    <div style="display: flex; gap: 8px; align-items: center;">
                        <a id="hfPreviewOpenStudioBtn" href="/d-admin/builder" class="btn btn-primary" style="font-size: 12px; padding: 5px 12px;">Edit in DayStudio</a>
                        <button type="button" onclick="closePreviewModal()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: var(--ink-muted);">&times;</button>
                    </div>
                </div>
                <div class="hf-modal-body" style="background: var(--ink); padding: 16px; display: flex; justify-content: center; align-items: stretch;">
                    <iframe id="hfPreviewIframe" src="about:blank" style="width: 100%; height: 100%; border: 1px solid var(--rule-strong); border-radius: var(--radius-sm); background: var(--paper-raised); transition: width 0.2s ease;"></iframe>
                </div>
            </div>
        </div>

        <!-- MODAL 5: 1-CLICK STARTERS CATALOG -->
        <div class="hf-modal-backdrop" id="hfStartersModal" onclick="if(event.target===this) closeStarterCatalogModal()">
            <div class="hf-modal" style="width: min(760px, 95vw);">
                <div class="hf-modal-header">
                    <div>
                        <strong style="font-size: 15px; color: var(--ink);">1-Click Ready-to-Use Header &amp; Footer Starters</strong>
                        <div style="font-size: 12px; color: var(--ink-muted);">Install a production AST Header or Footer template directly into SQLite</div>
                    </div>
                    <button type="button" onclick="closeStarterCatalogModal()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: var(--ink-muted);">&times;</button>
                </div>
                <div class="hf-modal-body">
                    <div id="hfStartersGrid" style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px;"></div>
                </div>
                <div class="hf-modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="closeStarterCatalogModal()">Close</button>
                </div>
            </div>
        </div>

        {script_block}
        "##,
        active_theme = escape_html(active_theme_name),
        active_headers = active_headers,
        total_headers = total_headers,
        active_footers = active_footers,
        total_footers = total_footers,
        total_pages = total_pages,
        custom_overrides = custom_overrides,
        other_templates = other_templates,
        hf_total = total_headers + total_footers,
        script_block = super::headers_footers_scripts::render_headers_footers_script(
            &initial_matrix_json,
            &starters_json
        ),
    );

    render_admin_shell(
        "headers-footers",
        "Header & Footer Manager",
        &body,
        settings,
    )
}
