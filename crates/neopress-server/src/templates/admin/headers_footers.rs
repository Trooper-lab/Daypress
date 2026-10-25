//! Theme Header & Footer Manager Admin Screen (`/n-admin/headers-footers`).
//!
//! Provides a unified, zero-mock production interface to:
//! 1. View and manage all Header, Footer, and Layout templates stored in `wp_neopress_templates`.
//! 2. Inspect exactly which Pages and System Routes each Header/Footer is applied to.
//! 3. Directly assign or override the Header and Footer on any individual Page, Post, or System Route
//!    via the two-way Page Assignment Matrix (`wp_neopress_template_conditions` + `wp_postmeta`).
//! 4. Configure Sticky/Transparent/Priority settings, edit display conditions with specific Page/Post
//!    selectors, preview templates live, install 1-Click Starters, and launch directly into ⚡ NeoStudio.

use super::layout::render_wp_admin_shell;
use crate::models::WpSettingsResponse;
use crate::templates::utils::escape_html;
use crate::theme_builder::models::{HeaderFooterMatrixResponse, StarterTemplateInfo, TemplateType};

pub fn render_wp_headers_footers_view(
    settings: &WpSettingsResponse,
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
        "Neopress Modern"
    } else {
        settings.active_theme.trim()
    };

    let body = format!(
        r##"
        <style>
            .hf-tabs {{
                display: inline-flex;
                background: #f1f5f9;
                padding: 4px;
                border-radius: 10px;
                border: 1px solid #e2e8f0;
                gap: 4px;
                margin-bottom: 20px;
            }}
            .hf-tab-btn {{
                padding: 8px 16px;
                border-radius: 7px;
                border: none;
                background: transparent;
                color: #475569;
                font-size: 13px;
                font-weight: 600;
                cursor: pointer;
                display: inline-flex;
                align-items: center;
                gap: 8px;
                transition: all 0.15s ease;
            }}
            .hf-tab-btn.active {{
                background: #ffffff;
                color: #0f172a;
                box-shadow: 0 1px 3px rgba(15, 23, 42, 0.1);
            }}
            .hf-tab-count {{
                font-size: 11px;
                padding: 1px 7px;
                border-radius: 999px;
                background: #e2e8f0;
                color: #334155;
                font-family: var(--font-mono);
            }}
            .hf-tab-btn.active .hf-tab-count {{
                background: #eff6ff;
                color: #2563eb;
            }}
            .hf-kpi-grid {{
                display: grid;
                grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
                gap: 16px;
                margin-bottom: 22px;
            }}
            .hf-kpi-card {{
                background: #ffffff;
                border: 1px solid var(--border-subtle);
                border-radius: 10px;
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
                color: #64748b;
            }}
            .hf-kpi-value {{
                font-size: 24px;
                font-weight: 800;
                color: #0f172a;
                letter-spacing: -0.02em;
            }}
            .hf-kpi-sub {{
                font-size: 12px;
                color: #64748b;
            }}
            .hf-tpl-grid {{
                display: grid;
                grid-template-columns: repeat(auto-fill, minmax(440px, 1fr));
                gap: 18px;
                margin-bottom: 28px;
            }}
            .hf-tpl-card {{
                background: #ffffff;
                border: 1px solid #cbd5e1;
                border-radius: 12px;
                padding: 20px;
                display: flex;
                flex-direction: column;
                gap: 14px;
                transition: border-color 0.15s ease, box-shadow 0.15s ease;
            }}
            .hf-tpl-card.is-active-tpl {{
                border-color: #93c5fd;
                box-shadow: 0 4px 14px -4px rgba(37, 99, 235, 0.1);
            }}
            .hf-tpl-card.is-inactive-tpl {{
                opacity: 0.78;
                background: #f8fafc;
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
                background: #eff6ff;
                color: #1d4ed8;
                border: 1px solid #bfdbfe;
            }}
            .hf-chip-route {{
                background: #f0fdf4;
                color: #15803d;
                border: 1px solid #bbf7d0;
            }}
            .hf-chip-rule {{
                background: #f8fafc;
                color: #334155;
                border: 1px solid #cbd5e1;
                font-family: var(--font-mono);
                font-size: 11px;
            }}
            .hf-chip-exclude {{
                background: #fef2f2;
                color: #dc2626;
                border: 1px solid #fecaca;
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
                background: #dbeafe;
                color: #1e40af;
                border: 1px solid #93c5fd;
            }}
            .hf-slot-badge.global {{
                background: #ecfdf5;
                color: #065f46;
                border: 1px solid #a7f3d0;
            }}
            .hf-slot-badge.hidden {{
                background: #fef2f2;
                color: #991b1b;
                border: 1px solid #fecaca;
            }}
            .hf-slot-badge.fallback {{
                background: #f1f5f9;
                color: #475569;
                border: 1px solid #cbd5e1;
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
                background: #ffffff;
                border-radius: 12px;
                width: min(680px, 95vw);
                max-height: 90vh;
                display: flex;
                flex-direction: column;
                box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.35);
                overflow: hidden;
            }}
            .hf-modal-header {{
                padding: 16px 22px;
                border-bottom: 1px solid #e2e8f0;
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
                border-top: 1px solid #e2e8f0;
                background: #f8fafc;
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
                background-color: #cbd5e1;
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
                background-color: #10b981;
            }}
            .hf-switch input:checked + .hf-slider:before {{
                transform: translateX(16px);
            }}
        </style>

        <div class="page-header">
            <div>
                <div style="display: flex; align-items: center; gap: 10px;">
                    <h1 class="page-title">Header &amp; Footer Manager</h1>
                    <span class="badge badge-published" style="font-size: 11px;">Theme: {active_theme}</span>
                </div>
                <p class="page-subtitle">See every Header &amp; Footer created on your site, inspect which pages they apply to, and assign overrides per page or route</p>
            </div>
            <div style="display: flex; gap: 8px; flex-wrap: wrap;">
                <button type="button" class="btn btn-secondary" onclick="openStarterCatalogModal()">⚡ 1-Click Starters</button>
                <button type="button" class="btn btn-secondary" onclick="openCreateModal('footer')">+ New Footer</button>
                <button type="button" class="btn btn-primary" onclick="openCreateModal('header')">+ New Header</button>
            </div>
        </div>

        <!-- KPI Summary Strip -->
        <div class="hf-kpi-grid">
            <div class="hf-kpi-card">
                <span class="hf-kpi-label">Headers</span>
                <div class="hf-kpi-value" id="kpiHeadersVal">{active_headers} <span style="font-size: 14px; font-weight: 500; color: #64748b;">/ {total_headers} active</span></div>
                <span class="hf-kpi-sub">Site navigation &amp; top bars</span>
            </div>
            <div class="hf-kpi-card">
                <span class="hf-kpi-label">Footers</span>
                <div class="hf-kpi-value" id="kpiFootersVal">{active_footers} <span style="font-size: 14px; font-weight: 500; color: #64748b;">/ {total_footers} active</span></div>
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
                <span class="hf-kpi-label">Layout Templates</span>
                <div class="hf-kpi-value" id="kpiOtherVal">{other_templates}</div>
                <span class="hf-kpi-sub">Single, Archive, 404 &amp; Popups</span>
            </div>
        </div>

        <!-- Main Workspace Navigation Tabs -->
        <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 12px;">
            <div class="hf-tabs" role="tablist">
                <button type="button" class="hf-tab-btn active" id="tabBtnTemplates" onclick="switchMainTab('templates')">
                    <span>📐 Headers &amp; Footers</span>
                    <span class="hf-tab-count" id="tabCountHF">{hf_total}</span>
                </button>
                <button type="button" class="hf-tab-btn" id="tabBtnMatrix" onclick="switchMainTab('matrix')">
                    <span>🗺️ Page Assignment Matrix</span>
                    <span class="hf-tab-count" id="tabCountPages">{total_pages}</span>
                </button>
                <button type="button" class="hf-tab-btn" id="tabBtnLayouts" onclick="switchMainTab('layouts')">
                    <span>🧩 Other Layout Templates</span>
                    <span class="hf-tab-count" id="tabCountOther">{other_templates}</span>
                </button>
            </div>

            <div id="templatesTypeFilterBar" style="display: flex; gap: 6px;">
                <button type="button" class="btn btn-secondary hf-filter-pill" data-type="all" style="background: #0f172a; color: #fff; font-size: 12px; padding: 5px 12px;" onclick="setTemplateFilter('all', this)">All Headers &amp; Footers</button>
                <button type="button" class="btn btn-secondary hf-filter-pill" data-type="header" style="font-size: 12px; padding: 5px 12px;" onclick="setTemplateFilter('header', this)">Headers Only</button>
                <button type="button" class="btn btn-secondary hf-filter-pill" data-type="footer" style="font-size: 12px; padding: 5px 12px;" onclick="setTemplateFilter('footer', this)">Footers Only</button>
            </div>
        </div>

        <!-- TAB 1: HEADERS & FOOTERS OVERVIEW -->
        <div id="tabPanelTemplates">
            <div id="headersSectionWrap">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                    <div>
                        <h2 style="font-size: 16px; font-weight: 700; color: #0f172a; margin: 0;">Created Headers</h2>
                        <p style="font-size: 12.5px; color: #64748b; margin: 2px 0 0 0;">Higher priority and more specific page rules automatically win during SSR.</p>
                    </div>
                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 12px;" onclick="openCreateModal('header')">+ Add Header</button>
                </div>
                <div id="headersCardsContainer" class="hf-tpl-grid"></div>
            </div>

            <div id="footersSectionWrap" style="margin-top: 12px;">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px;">
                    <div>
                        <h2 style="font-size: 16px; font-weight: 700; color: #0f172a; margin: 0;">Created Footers</h2>
                        <p style="font-size: 12.5px; color: #64748b; margin: 2px 0 0 0;">Manage global and page-specific footers and inspect their resolved page coverage.</p>
                    </div>
                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 12px;" onclick="openCreateModal('footer')">+ Add Footer</button>
                </div>
                <div id="footersCardsContainer" class="hf-tpl-grid"></div>
            </div>
        </div>

        <!-- TAB 2: PAGE ASSIGNMENT MATRIX -->
        <div id="tabPanelMatrix" style="display: none;">
            <div class="neo-card" style="padding: 18px 20px; margin-bottom: 20px; background: #f8fafc; border: 1px solid #cbd5e1;">
                <div style="display: flex; justify-content: space-between; align-items: center; flex-wrap: wrap; gap: 12px;">
                    <div>
                        <strong style="font-size: 14px; color: #0f172a;">Live Page &amp; Route Assignment Matrix</strong>
                        <p style="font-size: 12.5px; color: #475569; margin: 2px 0 0 0;">
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
            <div id="matrixRoutesCard" class="neo-card" style="margin-bottom: 24px; overflow: hidden;">
                <div style="padding: 14px 18px; background: #f8fafc; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center;">
                    <div>
                        <strong style="font-size: 13.5px; color: #0f172a;">System Routes &amp; Post-Type Defaults</strong>
                        <span style="font-size: 12px; color: #64748b; margin-left: 8px;">Controls default headers &amp; footers for Front Page, All Pages, All Posts, Archives, and 404</span>
                    </div>
                </div>
                <div class="neo-table-wrapper" style="border: none; border-radius: 0;">
                    <table class="neo-table">
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
            <div id="matrixPagesCard" class="neo-card" style="overflow: hidden;">
                <div style="padding: 14px 18px; background: #f8fafc; border-bottom: 1px solid var(--border-subtle); display: flex; justify-content: space-between; align-items: center;">
                    <div>
                        <strong style="font-size: 13.5px; color: #0f172a;" id="matrixItemsHeading">Individual Site Pages</strong>
                        <span style="font-size: 12px; color: #64748b; margin-left: 8px;">Assign a specific Header or Footer to any individual page, or set to Inherit / Hidden</span>
                    </div>
                </div>
                <div class="neo-table-wrapper" style="border: none; border-radius: 0;">
                    <table class="neo-table">
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
            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 14px;">
                <div>
                    <h2 style="font-size: 16px; font-weight: 700; color: #0f172a; margin: 0;">Single, Archive, 404 &amp; Popup Templates</h2>
                    <p style="font-size: 12.5px; color: #64748b; margin: 2px 0 0 0;">Theme Builder layout templates for blog articles, category archives, 404 error screens, and popups.</p>
                </div>
                <button type="button" class="btn btn-primary" style="font-size: 12.5px;" onclick="openCreateModal('single_post')">+ New Layout Template</button>
            </div>
            <div id="otherLayoutsCardsContainer" class="hf-tpl-grid"></div>
        </div>

        <!-- MODAL 1: CREATE TEMPLATE / INSTALL STARTER -->
        <div class="hf-modal-backdrop" id="hfCreateModal" onclick="if(event.target===this) closeCreateModal()">
            <div class="hf-modal">
                <div class="hf-modal-header">
                    <strong style="font-size: 15px; color: #0f172a;" id="hfCreateModalTitle">Create New Template</strong>
                    <button type="button" onclick="closeCreateModal()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: #64748b;">&times;</button>
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
                                <option value="header">Header</option>
                                <option value="footer">Footer</option>
                                <option value="single_post">Single Post Layout</option>
                                <option value="single_page">Single Page Layout</option>
                                <option value="archive">Archive / Category Layout</option>
                                <option value="404_page">404 Error Page Layout</option>
                                <option value="popup">Popup / Modal</option>
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
                                <option value="page">Specific Page...</option>
                                <option value="unassigned">Unassigned (Do not apply yet)</option>
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
                </div>
                <div class="hf-modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="closeCreateModal()">Cancel</button>
                    <button type="button" class="btn btn-secondary" onclick="submitCreateTemplate(false)">Create Template</button>
                    <button type="button" class="btn btn-primary" onclick="submitCreateTemplate(true)">⚡ Create &amp; Open in NeoStudio</button>
                </div>
            </div>
        </div>

        <!-- MODAL 2: DISPLAY CONDITIONS & PAGE ASSIGNMENTS -->
        <div class="hf-modal-backdrop" id="hfConditionsModal" onclick="if(event.target===this) closeConditionsModal()">
            <div class="hf-modal" style="width: min(760px, 96vw);">
                <div class="hf-modal-header">
                    <div>
                        <strong style="font-size: 15px; color: #0f172a;" id="hfCondModalTitle">Display Conditions</strong>
                        <div style="font-size: 12px; color: #64748b;">Configure where this template renders across your publication</div>
                    </div>
                    <button type="button" onclick="closeConditionsModal()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: #64748b;">&times;</button>
                </div>
                <div class="hf-modal-body">
                    <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 10px 14px; font-size: 12px; color: #475569; margin-bottom: 14px;">
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
                    <strong style="font-size: 15px; color: #0f172a;" id="hfSettingsModalTitle">Template Settings &amp; Behavior</strong>
                    <button type="button" onclick="closeSettingsModal()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: #64748b;">&times;</button>
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

                    <div id="setHeaderBehaviors" style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 14px; display: flex; flex-direction: column; gap: 10px;">
                        <strong style="font-size: 12.5px; color: #0f172a;">Header Behavior Controls</strong>
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px;">
                            <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer;">
                                <input type="checkbox" id="setTplSticky" />
                                <span>Sticky Header on Scroll (<code>.np-sticky</code>)</span>
                            </label>
                            <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer;">
                                <input type="checkbox" id="setTplTransparent" />
                                <span>Transparent Overlay Header (<code>.np-transparent</code>)</span>
                            </label>
                            <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer;">
                                <input type="checkbox" id="setTplScrollShrink" />
                                <span>Shrink Height on Scroll</span>
                            </label>
                            <div style="display: flex; align-items: center; gap: 8px;">
                                <label style="font-size: 12.5px; color: #475569;">Z-Index:</label>
                                <input type="number" id="setTplZIndex" class="form-input" style="width: 90px; height: 30px;" placeholder="999" />
                            </div>
                        </div>
                    </div>

                    <div id="setFooterBehaviors" style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 14px; display: none; flex-direction: column; gap: 10px;">
                        <strong style="font-size: 12.5px; color: #0f172a;">Footer Behavior Controls</strong>
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
                        <strong style="font-size: 15px; color: #0f172a;" id="hfPreviewModalTitle">Live Template Preview</strong>
                        <div style="display: inline-flex; gap: 4px; background: #f1f5f9; padding: 3px; border-radius: 6px;">
                            <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 3px 8px;" onclick="setPreviewViewport('100%')">Desktop</button>
                            <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 3px 8px;" onclick="setPreviewViewport('768px')">Tablet</button>
                            <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 3px 8px;" onclick="setPreviewViewport('390px')">Mobile</button>
                        </div>
                    </div>
                    <div style="display: flex; gap: 8px; align-items: center;">
                        <a id="hfPreviewOpenStudioBtn" href="/n-admin/builder" class="btn btn-primary" style="font-size: 12px; padding: 5px 12px;">⚡ Edit in NeoStudio</a>
                        <button type="button" onclick="closePreviewModal()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: #64748b;">&times;</button>
                    </div>
                </div>
                <div class="hf-modal-body" style="background: #0f172a; padding: 16px; display: flex; justify-content: center; align-items: stretch;">
                    <iframe id="hfPreviewIframe" src="about:blank" style="width: 100%; height: 100%; border: 1px solid #334155; border-radius: 8px; background: #ffffff; transition: width 0.2s ease;"></iframe>
                </div>
            </div>
        </div>

        <!-- MODAL 5: 1-CLICK STARTERS CATALOG -->
        <div class="hf-modal-backdrop" id="hfStartersModal" onclick="if(event.target===this) closeStarterCatalogModal()">
            <div class="hf-modal" style="width: min(760px, 95vw);">
                <div class="hf-modal-header">
                    <div>
                        <strong style="font-size: 15px; color: #0f172a;">⚡ 1-Click Ready-to-Use Header &amp; Footer Starters</strong>
                        <div style="font-size: 12px; color: #64748b;">Install a production AST Header or Footer template directly into SQLite</div>
                    </div>
                    <button type="button" onclick="closeStarterCatalogModal()" style="background: none; border: none; font-size: 20px; cursor: pointer; color: #64748b;">&times;</button>
                </div>
                <div class="hf-modal-body">
                    <div id="hfStartersGrid" style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px;"></div>
                </div>
                <div class="hf-modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="closeStarterCatalogModal()">Close</button>
                </div>
            </div>
        </div>

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
                    const res = await fetch('/wp-json/neopress/v1/templates/matrix', {{ credentials: 'same-origin' }});
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
                document.getElementById('templatesTypeFilterBar').style.display = tab === 'templates' ? 'flex' : 'none';
            }}

            function setTemplateFilter(type, btn) {{
                currentTemplateFilter = type;
                document.querySelectorAll('.hf-filter-pill').forEach(b => {{
                    b.style.background = '';
                    b.style.color = '';
                }});
                if (btn) {{
                    btn.style.background = '#0f172a';
                    btn.style.color = '#ffffff';
                }}
                document.getElementById('headersSectionWrap').style.display = (type === 'all' || type === 'header') ? 'block' : 'none';
                document.getElementById('footersSectionWrap').style.display = (type === 'all' || type === 'footer') ? 'block' : 'none';
            }}

            function renderAllViews() {{
                const templates = hfMatrix.templates || [];
                const headers = templates.filter(t => t.template_type === 'header');
                const footers = templates.filter(t => t.template_type === 'footer');
                const others = templates.filter(t => t.template_type !== 'header' && t.template_type !== 'footer');

                const activeH = headers.filter(t => t.is_active && t.status === 'publish').length;
                const activeF = footers.filter(t => t.is_active && t.status === 'publish').length;
                const pages = hfMatrix.pages || [];
                const routes = hfMatrix.routes || [];
                const overrides = [...pages, ...routes].filter(r => r.header.mode !== 'inherit' || r.footer.mode !== 'inherit').length;

                document.getElementById('kpiHeadersVal').innerHTML = `${{activeH}} <span style="font-size: 14px; font-weight: 500; color: #64748b;">/ ${{headers.length}} active</span>`;
                document.getElementById('kpiFootersVal').innerHTML = `${{activeF}} <span style="font-size: 14px; font-weight: 500; color: #64748b;">/ ${{footers.length}} active</span>`;
                document.getElementById('kpiPagesVal').textContent = String(pages.length);
                document.getElementById('kpiOverridesVal').textContent = String(overrides);
                document.getElementById('kpiOtherVal').textContent = String(others.length);

                document.getElementById('tabCountHF').textContent = String(headers.length + footers.length);
                document.getElementById('tabCountPages').textContent = String(pages.length);
                document.getElementById('tabCountOther').textContent = String(others.length);

                renderTemplateCards(document.getElementById('headersCardsContainer'), headers, 'header');
                renderTemplateCards(document.getElementById('footersCardsContainer'), footers, 'footer');
                renderTemplateCards(document.getElementById('otherLayoutsCardsContainer'), others, 'layout');
                renderMatrixTables();
            }}

            function renderTemplateCards(container, list, kindLabel) {{
                if (!container) return;
                if (!list || list.length === 0) {{
                    const defaultType = kindLabel === 'footer' ? 'footer' : (kindLabel === 'header' ? 'header' : 'single_post');
                    container.innerHTML = `
                        <div class="neo-card" style="grid-column: 1 / -1; padding: 28px; text-align: center; background: #f8fafc; border: 1px dashed #cbd5e1;">
                            <div style="font-size: 14px; font-weight: 700; color: #0f172a; margin-bottom: 6px;">No ${{escHtml(kindLabel)}} templates created yet</div>
                            <p style="font-size: 12.5px; color: #64748b; margin-bottom: 14px;">Create a new ${{escHtml(kindLabel)}} or install a 1-click starter template to customize your site shell.</p>
                            <div style="display: inline-flex; gap: 8px;">
                                <button type="button" class="btn btn-primary" style="font-size: 12px;" onclick="openCreateModal('${{defaultType}}')">+ Create ${{escHtml(kindLabel)}}</button>
                                <button type="button" class="btn btn-secondary" style="font-size: 12px;" onclick="openStarterCatalogModal()">⚡ Browse Starters</button>
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
                        if (t.settings.sticky) behaviorBadges += `<span class="hf-chip" style="background:#eff6ff;color:#1d4ed8;border:1px solid #bfdbfe;">📌 Sticky</span>`;
                        if (t.settings.transparent) behaviorBadges += `<span class="hf-chip" style="background:#f5f3ff;color:#6d28d9;border:1px solid #ddd6fe;">✨ Transparent</span>`;
                        if (t.settings.scroll_shrink) behaviorBadges += `<span class="hf-chip" style="background:#f0fdf4;color:#15803d;border:1px solid #bbf7d0;">↕ Shrink</span>`;
                    }} else if (t.template_type === 'footer' && t.settings) {{
                        if (t.settings.sticky_footer) behaviorBadges += `<span class="hf-chip" style="background:#eff6ff;color:#1d4ed8;border:1px solid #bfdbfe;">📌 Sticky Footer</span>`;
                        if (t.settings.reveal_on_scroll) behaviorBadges += `<span class="hf-chip" style="background:#f5f3ff;color:#6d28d9;border:1px solid #ddd6fe;">🎭 Reveal</span>`;
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
                        matchedHtml = `<span style="font-size: 12px; color: #94a3b8;">Template is currently inactive or draft — enable toggle to apply on site.</span>`;
                    }} else if (matchedList.length === 0) {{
                        matchedHtml = `<span style="font-size: 12px; color: #d97706;">Not currently winning on any page (assign a page below or edit conditions).</span>`;
                    }} else {{
                        const visibleChips = matchedList.slice(0, 10).map(mp => {{
                            const chipCls = mp.is_route ? 'hf-chip hf-chip-route' : 'hf-chip hf-chip-page';
                            const icon = mp.is_route ? '🌐' : '📄';
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
                                        <strong style="font-size: 16px; color: #0f172a;">${{escHtml(t.title)}}</strong>
                                        <span style="font-size: 11px; font-family: var(--font-mono); color: #64748b;">#${{t.id}}</span>
                                        <span class="hf-chip" style="background:#f1f5f9;color:#334155;font-size:10.5px;text-transform:uppercase;">${{escHtml(t.template_type)}}</span>
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
                            <div style="background: #f8fafc; border: 1px solid #e2e8f0; border-radius: 8px; padding: 10px 12px;">
                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                                    <span style="font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.04em; color: #475569;">Display Condition Rules</span>
                                    <button type="button" onclick="openConditionsModal(${{t.id}})" style="background: none; border: none; color: #2563eb; font-size: 11.5px; font-weight: 600; cursor: pointer;">Edit Rules &rarr;</button>
                                </div>
                                <div style="display: flex; flex-wrap: wrap; gap: 6px;">
                                    ${{condPills}}
                                </div>
                            </div>

                            <!-- Resolved Pages Where Applied -->
                            <div style="background: #ffffff; border: 1px solid #e2e8f0; border-radius: 8px; padding: 10px 12px;">
                                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px; flex-wrap: wrap; gap: 8px;">
                                    <span style="font-size: 11px; font-weight: 700; text-transform: uppercase; letter-spacing: 0.04em; color: #0f172a;">
                                        Applied To Pages &amp; Routes (${{matchedList.length}})
                                    </span>
                                    ${{showQuickAssign ? `
                                        <select class="form-select" style="width: auto; max-width: 220px; height: 26px; padding: 2px 8px; font-size: 11.5px;" onchange="quickAssignTemplateToTarget(${{t.id}}, '${{t.template_type}}', this.value); this.selectedIndex = 0;">
                                            <option value="">+ Quick Assign to Page / Route...</option>
                                            <option value="route:entire_site">🌐 Entire Site (Global Default)</option>
                                            <option value="route:front_page">🏠 Front Page Only</option>
                                            <option value="route:all_pages">📄 All Static Pages</option>
                                            <option value="route:all_posts">📝 All Blog Posts</option>
                                            <option value="route:archive">🗂️ All Archives</option>
                                            <option value="route:not_found">⚠️ 404 Not Found Page</option>
                                            ${{quickAssignOptions}}
                                        </select>
                                    ` : ''}}
                                </div>
                                <div style="display: flex; flex-wrap: wrap; gap: 6px; align-items: center;">
                                    ${{matchedHtml}}
                                </div>
                            </div>

                            <!-- Card Footer Actions -->
                            <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 6px; border-top: 1px solid #f1f5f9; flex-wrap: wrap; gap: 8px;">
                                <div style="display: flex; gap: 6px; flex-wrap: wrap;">
                                    <a href="/n-admin/builder?template_id=${{t.id}}" class="btn btn-primary" style="font-size: 12px; padding: 5px 12px; text-decoration: none;">⚡ Edit in NeoStudio</a>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openConditionsModal(${{t.id}})">Conditions</button>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openSettingsModal(${{t.id}})">Settings</button>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openPreviewModal(${{t.id}}, '${{escHtml(t.title)}}')">Preview</button>
                                </div>
                                <div style="display: flex; gap: 6px;">
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 9px;" title="Duplicate Template" onclick="duplicateTemplate(${{t.id}})">⧉ Clone</button>
                                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 9px; color: #dc2626;" title="Delete Template" onclick="deleteTemplate(${{t.id}}, '${{escHtml(t.title)}}')">Trash</button>
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
                        || `<tr><td colspan="5" style="text-align:center;color:#64748b;padding:20px;">No matching system routes</td></tr>`;
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
                        || `<tr><td colspan="5" style="text-align:center;color:#64748b;padding:24px;">No matching items found</td></tr>`;
                }}
            }}

            function renderMatrixRowHtml(row, headerTemplates, footerTemplates, isRoute) {{
                let badges = '';
                if (row.is_front_page && !isRoute) {{
                    badges += `<span style="display:inline-flex;align-items:center;font-size:10.5px;background:#dcfce7;color:#15803d;border:1px solid #86efac;padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:700;">🏠 Front Page</span>`;
                }}
                if (row.in_main_menu) {{
                    badges += `<span style="display:inline-flex;align-items:center;font-size:10.5px;background:#eff6ff;color:#1d4ed8;border:1px solid #bfdbfe;padding:1px 6px;border-radius:4px;margin-left:6px;font-weight:600;">🧭 Menu</span>`;
                }}
                if (row.status === 'draft') {{
                    badges += `<span class="badge badge-draft" style="font-size:10px;margin-left:6px;">Draft</span>`;
                }}

                const subLinks = isRoute
                    ? `<div class="table-row-slug">Route Pattern: <code>${{escHtml(row.permalink)}}</code></div>`
                    : `<div class="table-row-slug">
                        Permalink: <a href="${{escHtml(row.permalink)}}" target="_blank" style="color:#2563eb;text-decoration:none;">${{escHtml(row.permalink)}} &nearr;</a>
                        &bull; <a href="/n-admin/post?post=${{row.post_id}}&action=edit&post_type=${{row.target_kind}}" style="color:#475569;text-decoration:none;">Edit Page</a>
                        &bull; <a href="/n-admin/builder?post=${{row.post_id}}" style="color:#2563eb;text-decoration:none;font-weight:600;">⚡ NeoStudio</a>
                       </div>`;

                const hStudioLink = row.header.template_id
                    ? `<a href="/n-admin/builder?template_id=${{row.header.template_id}}" title="Edit this Header in NeoStudio" style="font-size:11px;color:#2563eb;text-decoration:none;font-weight:600;margin-left:4px;">⚡ Edit</a>`
                    : '';
                const fStudioLink = row.footer.template_id
                    ? `<a href="/n-admin/builder?template_id=${{row.footer.template_id}}" title="Edit this Footer in NeoStudio" style="font-size:11px;color:#2563eb;text-decoration:none;font-weight:600;margin-left:4px;">⚡ Edit</a>`
                    : '';

                const hSelectHtml = buildSlotSelectHtml('header', row, row.header, headerTemplates, isRoute);
                const fSelectHtml = buildSlotSelectHtml('footer', row, row.footer, footerTemplates, isRoute);

                return `
                    <tr>
                        <td>
                            <strong style="color:#0f172a;font-size:13.5px;">${{escHtml(row.title)}}</strong>${{badges}}
                            ${{subLinks}}
                        </td>
                        <td>
                            <div>
                                <span class="hf-slot-badge ${{escHtml(row.header.badge_kind)}}">${{escHtml(row.header.display_name)}}</span>
                                ${{hStudioLink}}
                            </div>
                            <div style="font-size:11px;color:#64748b;margin-top:3px;">${{escHtml(row.header.reason)}}</div>
                        </td>
                        <td>${{hSelectHtml}}</td>
                        <td>
                            <div>
                                <span class="hf-slot-badge ${{escHtml(row.footer.badge_kind)}}">${{escHtml(row.footer.display_name)}}</span>
                                ${{fStudioLink}}
                            </div>
                            <div style="font-size:11px;color:#64748b;margin-top:3px;">${{escHtml(row.footer.reason)}}</div>
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
                    return `<option value="${{val}}" ${{sel}}>📌 Assign: ${{escHtml(t.title)}} (#${{t.id}})</option>`;
                }}).join('');

                return `
                    <select class="form-select" style="font-size:12px;padding:5px 8px;height:32px;" onchange="assignSlotFromMatrix(${{targetArg}}, '${{slot}}', this.value)">
                        <option value="inherit" ${{curMode === 'inherit' ? 'selected' : ''}}>Inherit Global Rules</option>
                        ${{tplOptions}}
                        <option value="fallback" ${{curMode === 'fallback' ? 'selected' : ''}}>Built-in Theme ${{slot === 'header' ? 'Header' : 'Footer'}}</option>
                        <option value="none" ${{curMode === 'none' ? 'selected' : ''}}>🚫 Hide (No ${{slot === 'header' ? 'Header' : 'Footer'}})</option>
                    </select>
                `;
            }}

            async function assignSlotFromMatrix(kind, idOrKey, slot, value) {{
                const payload = kind === 'route'
                    ? {{ route_key: String(idOrKey), [slot]: value }}
                    : {{ post_id: Number(idOrKey), [slot]: value }};
                try {{
                    const res = await fetch('/wp-json/neopress/v1/templates/assign-page', {{
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
                    const res = await fetch(`/wp-json/neopress/v1/templates/${{templateId}}`, {{
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
                    const res = await fetch(`/wp-json/neopress/v1/templates/${{templateId}}/duplicate`, {{
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
                    const res = await fetch(`/wp-json/neopress/v1/templates/${{templateId}}?force=true`, {{
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
                const titleInput = document.getElementById('createTplTitle');
                if (typeSel && defaultType) typeSel.value = defaultType;
                onCreateTypeChange();
                if (titleInput) {{
                    const t = typeSel ? typeSel.value : 'header';
                    titleInput.value = t === 'header' ? 'Custom Site Header' : (t === 'footer' ? 'Custom Site Footer' : 'Custom Layout Template');
                }}
                populateCreatePageOptions();
                document.getElementById('hfCreateModal').classList.add('open');
                setTimeout(() => titleInput?.focus(), 20);
            }}

            function closeCreateModal() {{
                document.getElementById('hfCreateModal').classList.remove('open');
            }}

            function onCreateTypeChange() {{
                const t = document.getElementById('createTplType').value;
                const starterSel = document.getElementById('createTplStarter');
                starterSel.innerHTML = `<option value="blank">Blank Canvas (Clean Root AST)</option>`;
                hfStarters.filter(s => s.template_type === t).forEach(s => {{
                    starterSel.innerHTML += `<option value="${{escHtml(s.id)}}">⚡ Starter: ${{escHtml(s.title)}}</option>`;
                }});
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

                try {{
                    let created = null;
                    if (starter !== 'blank') {{
                        const res = await fetch('/wp-json/neopress/v1/templates/starter', {{
                            method: 'POST',
                            headers: {{ 'Content-Type': 'application/json' }},
                            credentials: 'same-origin',
                            body: JSON.stringify({{ starter_id: starter }})
                        }});
                        if (!res.ok) throw new Error('Failed to install starter');
                        created = await res.json();
                        await fetch(`/wp-json/neopress/v1/templates/${{created.id}}`, {{
                            method: 'PUT',
                            headers: {{ 'Content-Type': 'application/json' }},
                            credentials: 'same-origin',
                            body: JSON.stringify({{ title, priority, conditions }})
                        }});
                    }} else {{
                        const res = await fetch('/wp-json/neopress/v1/templates', {{
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
                                layout: {{ version: 1, root: {{ id: 'root', type: 'Root', children: [] }} }}
                            }})
                        }});
                        if (!res.ok) throw new Error('Failed to create template');
                        created = await res.json();
                    }}

                    closeCreateModal();
                    if (openInStudio && created && created.id) {{
                        window.location.href = `/n-admin/builder?template_id=${{created.id}}`;
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
                        <div class="hf-cond-row" style="display:flex;gap:8px;align-items:center;background:#f8fafc;border:1px solid #cbd5e1;padding:10px;border-radius:8px;flex-wrap:wrap;">
                            <select class="form-select cond-row-type" style="width:120px;font-weight:700;">
                                <option value="include" ${{c.condition_type === 'include' ? 'selected' : ''}}>INCLUDE</option>
                                <option value="exclude" ${{c.condition_type === 'exclude' ? 'selected' : ''}}>EXCLUDE</option>
                            </select>
                            <select class="form-select cond-row-target" style="flex:1;min-width:180px;" onchange="onCondRowTargetChange(this)">
                                <option value="entire_site" ${{rt === 'entire_site' ? 'selected' : ''}}>🌐 Entire Site (All Pages &amp; Posts)</option>
                                <option value="front_page" ${{rt === 'front_page' ? 'selected' : ''}}>🏠 Front Page Only</option>
                                <option value="all_pages" ${{rt === 'all_pages' ? 'selected' : ''}}>📄 All Static Pages</option>
                                <option value="all_posts" ${{rt === 'all_posts' ? 'selected' : ''}}>📝 All Blog Posts</option>
                                <option value="page" ${{rt === 'page' ? 'selected' : ''}}>📌 Specific Page...</option>
                                <option value="post" ${{rt === 'post' ? 'selected' : ''}}>📌 Specific Blog Post...</option>
                                <option value="archive" ${{rt === 'archive' ? 'selected' : ''}}>🗂️ All Archives</option>
                                <option value="category" ${{rt === 'category' ? 'selected' : ''}}>🏷️ Category Slug/ID...</option>
                                <option value="tag" ${{rt === 'tag' ? 'selected' : ''}}>🏷️ Tag Slug/ID...</option>
                                <option value="not_found" ${{rt === 'not_found' ? 'selected' : ''}}>⚠️ 404 Not Found Page</option>
                            </select>
                            <select class="form-select cond-row-page-id" style="width:210px;display:${{showPageSel ? 'block' : 'none'}};">
                                ${{pageOpts || '<option value="">No pages found</option>'}}
                            </select>
                            <select class="form-select cond-row-post-id" style="width:210px;display:${{showPostSel ? 'block' : 'none'}};">
                                ${{postOpts || '<option value="">No posts found</option>'}}
                            </select>
                            <input type="text" class="form-input cond-row-custom-id" placeholder="Slug or ID" value="${{escHtml(c.target_id || '')}}" style="width:140px;display:${{showTextId ? 'block' : 'none'}};" />
                            <input type="number" class="form-input cond-row-priority" value="${{c.priority || 10}}" title="Rule Priority" style="width:76px;" />
                            <button type="button" class="btn btn-secondary" style="color:#dc2626;padding:6px 10px;" onclick="this.closest('.hf-cond-row').remove()">✕</button>
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
                    const res = await fetch(`/wp-json/neopress/v1/templates/${{editingConditionsTplId}}/conditions`, {{
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

                document.getElementById('setHeaderBehaviors').style.display = tpl.template_type === 'header' ? 'flex' : 'none';
                document.getElementById('setFooterBehaviors').style.display = tpl.template_type === 'footer' ? 'flex' : 'none';

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

                const settings = Object.assign({{}}, tpl.settings || {{}}, {{
                    sticky: document.getElementById('setTplSticky').checked,
                    transparent: document.getElementById('setTplTransparent').checked,
                    scroll_shrink: document.getElementById('setTplScrollShrink').checked,
                    z_index: isNaN(zVal) ? null : zVal,
                    sticky_footer: document.getElementById('setTplStickyFooter').checked,
                    reveal_on_scroll: document.getElementById('setTplRevealScroll').checked,
                    custom_css: document.getElementById('setTplCustomCss').value
                }});

                try {{
                    const res = await fetch(`/wp-json/neopress/v1/templates/${{editingSettingsTplId}}`, {{
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
                document.getElementById('hfPreviewOpenStudioBtn').href = `/n-admin/builder?template_id=${{templateId}}`;
                const iframe = document.getElementById('hfPreviewIframe');
                iframe.style.width = '100%';
                iframe.src = `/wp-json/neopress/v1/templates/preview/${{templateId}}`;
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
                    <div class="neo-card" style="padding: 16px; display: flex; flex-direction: column; justify-content: space-between; gap: 12px; border: 1px solid #cbd5e1;">
                        <div>
                            <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 6px;">
                                <strong style="font-size: 14px; color: #0f172a;">${{escHtml(s.title)}}</strong>
                                <span class="hf-chip" style="background:#eff6ff;color:#1d4ed8;font-size:10.5px;text-transform:uppercase;">${{escHtml(s.template_type)}}</span>
                            </div>
                            <p style="font-size: 12px; color: #64748b; margin: 0;">${{escHtml(s.description)}}</p>
                        </div>
                        <div style="display: flex; justify-content: space-between; align-items: center; padding-top: 10px; border-top: 1px solid #f1f5f9;">
                            <span style="font-size: 11px; color: #64748b; font-family: var(--font-mono);">Category: ${{escHtml(s.category)}}</span>
                            <button type="button" class="btn btn-primary" style="font-size: 12px; padding: 5px 12px;" onclick="installStarterNow('${{escHtml(s.id)}}', '${{escHtml(s.title)}}')">⚡ Install Starter</button>
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
                    const res = await fetch('/wp-json/neopress/v1/templates/starter', {{
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
        active_theme = escape_html(active_theme_name),
        active_headers = active_headers,
        total_headers = total_headers,
        active_footers = active_footers,
        total_footers = total_footers,
        total_pages = total_pages,
        custom_overrides = custom_overrides,
        other_templates = other_templates,
        hf_total = total_headers + total_footers,
        initial_matrix_json = initial_matrix_json,
        starters_json = starters_json,
    );

    render_wp_admin_shell(
        "headers-footers",
        "Header & Footer Manager",
        &body,
        settings,
    )
}
