//! Theme Architecture Studio: Dedicated Admin Screens for
//! 1. Header & Footer Manager + Interactive Page & Route Assignment Matrix (`/d-admin/headers-footers`)
//! 2. Dynamic Theme Layouts (`/d-admin/theme-layouts`, `/d-admin/theme-builder`)
//! 3. Synced Global Symbols & Mega Menu Panels (`/d-admin/symbols`)
//! 4. Site-Wide Popups, Slide-In Drawers & Announcement Bars (`/d-admin/popups`)
//!
//! Built to `/design-taste-frontend` (`framed-sheet` + `references/app-screens.md`) standards:
//! - Dedicated view per area with its own `<h1>`, divided 1px KPI strip (`tabular-nums`), and zero decorative emoji.
//! - Unified Interactive Slot Matrix with Site Shell Blueprint bar, bulk page assignment, and 1-click override reset.
//! - Purpose-built cards with live inline SQLite persistence for Header/Footer behaviors, Layout tokens, Symbol Mega Menu bindings, and Popup triggers.

use super::headers_footers_scripts::render_headers_footers_script;
use super::layout::render_admin_shell;
use crate::models::DaypressSettingsResponse;
use crate::templates::utils::escape_html;
use crate::theme_builder::{HeaderFooterMatrixResponse, StarterTemplateInfo};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeStudioPageMode {
    HeadersFooters,
    ThemeLayouts,
    SyncedSymbols,
    PopupsBars,
}

impl ThemeStudioPageMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ThemeStudioPageMode::HeadersFooters => "headers_footers",
            ThemeStudioPageMode::ThemeLayouts => "theme_layouts",
            ThemeStudioPageMode::SyncedSymbols => "symbols",
            ThemeStudioPageMode::PopupsBars => "popups",
        }
    }

    pub fn nav_tab(&self) -> &'static str {
        match self {
            ThemeStudioPageMode::HeadersFooters => "headers-footers",
            ThemeStudioPageMode::ThemeLayouts => "theme-layouts",
            ThemeStudioPageMode::SyncedSymbols => "symbols",
            ThemeStudioPageMode::PopupsBars => "popups",
        }
    }

    pub fn title(&self) -> &'static str {
        match self {
            ThemeStudioPageMode::HeadersFooters => "Header & Footer Manager",
            ThemeStudioPageMode::ThemeLayouts => "Theme Layouts",
            ThemeStudioPageMode::SyncedSymbols => "Synced Symbols",
            ThemeStudioPageMode::PopupsBars => "Popups & Announcement Bars",
        }
    }

    pub fn subtitle(&self) -> &'static str {
        match self {
            ThemeStudioPageMode::HeadersFooters => {
                "Design site-wide headers and footers in DayStudio and assign them to system routes or individual pages."
            }
            ThemeStudioPageMode::ThemeLayouts => {
                "Build dynamic Single Post, Single Page, Category Archive, and 404 recovery templates with live token hydration."
            }
            ThemeStudioPageMode::SyncedSymbols => {
                "Create Figma-style reusable global components with per-instance slot overrides and 1-click Mega Menu panel bindings."
            }
            ThemeStudioPageMode::PopupsBars => {
                "Configure announcement bars, slide-in drawers, and conversion modals with scroll, delay, exit-intent, and cooldown rules."
            }
        }
    }

    pub fn default_create_type(&self) -> &'static str {
        match self {
            ThemeStudioPageMode::HeadersFooters => "header",
            ThemeStudioPageMode::ThemeLayouts => "single_post",
            ThemeStudioPageMode::SyncedSymbols => "section_symbol",
            ThemeStudioPageMode::PopupsBars => "popup",
        }
    }
}

pub fn render_headers_footers_view(
    settings: &DaypressSettingsResponse,
    matrix: &HeaderFooterMatrixResponse,
    starters: &[StarterTemplateInfo],
) -> String {
    render_theme_studio_page(
        ThemeStudioPageMode::HeadersFooters,
        settings,
        matrix,
        starters,
    )
}

pub fn render_theme_layouts_view(
    settings: &DaypressSettingsResponse,
    matrix: &HeaderFooterMatrixResponse,
    starters: &[StarterTemplateInfo],
) -> String {
    render_theme_studio_page(
        ThemeStudioPageMode::ThemeLayouts,
        settings,
        matrix,
        starters,
    )
}

pub fn render_symbols_view(
    settings: &DaypressSettingsResponse,
    matrix: &HeaderFooterMatrixResponse,
    starters: &[StarterTemplateInfo],
) -> String {
    render_theme_studio_page(
        ThemeStudioPageMode::SyncedSymbols,
        settings,
        matrix,
        starters,
    )
}

pub fn render_popups_view(
    settings: &DaypressSettingsResponse,
    matrix: &HeaderFooterMatrixResponse,
    starters: &[StarterTemplateInfo],
) -> String {
    render_theme_studio_page(ThemeStudioPageMode::PopupsBars, settings, matrix, starters)
}

fn render_theme_studio_page(
    mode: ThemeStudioPageMode,
    settings: &DaypressSettingsResponse,
    matrix: &HeaderFooterMatrixResponse,
    starters: &[StarterTemplateInfo],
) -> String {
    let initial_matrix_json = serde_json::to_string(matrix)
        .unwrap_or_else(|_| {
            r#"{"templates":[],"coverage":{},"routes":[],"pages":[],"posts":[]}"#.to_string()
        })
        .replace("</", "<\\/");

    let starters_json = serde_json::to_string(starters)
        .unwrap_or_else(|_| "[]".to_string())
        .replace("</", "<\\/");

    let nav_menu_items_json = serde_json::to_string(&settings.nav_menu_items)
        .unwrap_or_else(|_| "[]".to_string())
        .replace("</", "<\\/");

    let script_block = render_headers_footers_script(
        mode.as_str(),
        &initial_matrix_json,
        &starters_json,
        &nav_menu_items_json,
    );

    let primary_actions = match mode {
        ThemeStudioPageMode::HeadersFooters => {
            r##"
            <button type="button" class="btn btn-secondary" onclick="openStarterCatalogModal('header_footer')">Starter Catalog</button>
            <button type="button" class="btn btn-secondary" onclick="openCreateModal('footer')">New Footer</button>
            <button type="button" class="btn btn-primary" onclick="openCreateModal('header')">New Header</button>
            "##
        }
        ThemeStudioPageMode::ThemeLayouts => {
            r##"
            <button type="button" class="btn btn-secondary" onclick="openStarterCatalogModal('layouts')">Layout Starters</button>
            <button type="button" class="btn btn-secondary" onclick="openCreateModal('archive')">New Archive Layout</button>
            <button type="button" class="btn btn-primary" onclick="openCreateModal('single_post')">New Theme Layout</button>
            "##
        }
        ThemeStudioPageMode::SyncedSymbols => {
            r##"
            <a href="/d-admin/nav-menus" class="btn btn-secondary">Navigation Menus</a>
            <button type="button" class="btn btn-secondary" onclick="openStarterCatalogModal('section_symbol')">Symbol Starters</button>
            <button type="button" class="btn btn-primary" onclick="openCreateModal('section_symbol')">New Synced Symbol</button>
            "##
        }
        ThemeStudioPageMode::PopupsBars => {
            r##"
            <button type="button" class="btn btn-secondary" onclick="openStarterCatalogModal('popup')">Popup Starters</button>
            <button type="button" class="btn btn-secondary" onclick="openCreatePopupWithVariant('top_bar')">New Announcement Bar</button>
            <button type="button" class="btn btn-primary" onclick="openCreatePopupWithVariant('modal')">New Popup Modal</button>
            "##
        }
    };

    let suite_nav_hf = if mode == ThemeStudioPageMode::HeadersFooters {
        "active"
    } else {
        ""
    };
    let suite_nav_layouts = if mode == ThemeStudioPageMode::ThemeLayouts {
        "active"
    } else {
        ""
    };
    let suite_nav_symbols = if mode == ThemeStudioPageMode::SyncedSymbols {
        "active"
    } else {
        ""
    };
    let suite_nav_popups = if mode == ThemeStudioPageMode::PopupsBars {
        "active"
    } else {
        ""
    };

    let main_workspace_html = match mode {
        ThemeStudioPageMode::HeadersFooters => render_headers_footers_workspace(),
        ThemeStudioPageMode::ThemeLayouts => render_theme_layouts_workspace(),
        ThemeStudioPageMode::SyncedSymbols => render_symbols_workspace(),
        ThemeStudioPageMode::PopupsBars => render_popups_workspace(),
    };

    let body = format!(
        r##"
        <style>
            .ts-suite-bar {{
                display: flex;
                align-items: center;
                justify-content: space-between;
                gap: 16px;
                border-bottom: 1px solid var(--rule-strong);
                margin-bottom: 20px;
                flex-wrap: wrap;
            }}
            .ts-suite-tabs {{
                display: flex;
                align-items: center;
                gap: 24px;
                overflow-x: auto;
            }}
            .ts-suite-link {{
                display: inline-flex;
                align-items: center;
                gap: 8px;
                padding: 10px 0 11px 0;
                font-size: 13.5px;
                font-weight: 500;
                color: var(--ink-muted);
                text-decoration: none;
                border-bottom: 2px solid transparent;
                margin-bottom: -1px;
                white-space: nowrap;
                transition: color 0.12s ease, border-color 0.12s ease;
                cursor: pointer;
                background: none;
                border-top: none;
                border-left: none;
                border-right: none;
            }}
            .ts-suite-link:hover {{
                color: var(--ink);
            }}
            .ts-suite-link.active {{
                color: var(--ink);
                font-weight: 600;
                border-bottom-color: var(--ink);
            }}
            .ts-count-pill {{
                font-family: var(--font-mono);
                font-variant-numeric: tabular-nums;
                font-size: 11px;
                font-weight: 600;
                padding: 1px 6px;
                border-radius: 4px;
                background: var(--paper-sunk, rgba(15, 23, 42, 0.05));
                color: var(--ink-muted);
                border: 1px solid var(--rule);
            }}
            .ts-suite-link.active .ts-count-pill {{
                background: var(--ink);
                color: var(--paper-raised);
                border-color: var(--ink);
            }}

            /* Divided KPI Strip (design-taste-frontend §5: single divided strip with 1px hairlines) */
            .ts-kpi-strip {{
                display: grid;
                grid-template-columns: repeat(4, 1fr);
                background: var(--paper-raised);
                border: 1px solid var(--rule-strong);
                border-radius: 10px;
                margin-bottom: 24px;
                overflow: hidden;
            }}
            .ts-kpi-cell {{
                padding: 16px 20px;
                border-right: 1px solid var(--rule);
                display: flex;
                flex-direction: column;
                gap: 4px;
                min-width: 0;
            }}
            .ts-kpi-cell:last-child {{
                border-right: none;
            }}
            .ts-kpi-label {{
                font-size: 12px;
                font-weight: 500;
                color: var(--ink-muted);
                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
            }}
            .ts-kpi-value {{
                font-size: 26px;
                font-weight: 600;
                color: var(--ink);
                line-height: 1.15;
                letter-spacing: -0.02em;
                font-variant-numeric: tabular-nums;
            }}
            .ts-kpi-sub {{
                font-size: 12px;
                color: var(--ink-muted);
                white-space: nowrap;
                overflow: hidden;
                text-overflow: ellipsis;
            }}

            /* Cards & Grids */
            .hf-grid-2col {{
                display: grid;
                grid-template-columns: repeat(auto-fill, minmax(460px, 1fr));
                gap: 20px;
            }}
            .hf-tpl-card {{
                background: var(--paper-raised);
                border: 1px solid var(--rule-strong);
                border-radius: 10px;
                padding: 18px 20px;
                display: flex;
                flex-direction: column;
                gap: 14px;
                position: relative;
                transition: border-color 0.14s ease;
            }}
            .hf-tpl-card:hover {{
                border-color: var(--ink-muted);
            }}
            .hf-tpl-card.is-inactive {{
                opacity: 0.68;
                background: var(--paper);
            }}
            .hf-chip {{
                display: inline-flex;
                align-items: center;
                gap: 5px;
                font-size: 11px;
                font-weight: 600;
                padding: 2px 8px;
                border-radius: 5px;
                line-height: 1.4;
                font-variant-numeric: tabular-nums;
            }}
            .hf-chip-global {{ background: #ecfdf5; color: #047857; border: 1px solid #a7f3d0; }}
            .hf-chip-custom {{ background: #eff6ff; color: #1d4ed8; border: 1px solid #bfdbfe; }}
            .hf-chip-fallback {{ background: var(--paper); color: var(--ink-muted); border: 1px solid var(--rule-strong); }}
            .hf-chip-hidden {{ background: #fef2f2; color: #b91c1c; border: 1px solid #fecaca; }}
            .hf-chip-behavior {{ background: var(--paper); color: var(--ink); border: 1px solid var(--rule-strong); font-family: var(--font-mono); font-size: 10.5px; }}
            .hf-chip-token {{ background: var(--paper); color: var(--ink); border: 1px solid var(--rule-strong); font-family: var(--font-mono); font-size: 11px; padding: 2px 7px; border-radius: 4px; }}

            /* Schematic Wireframe Header/Footer/Layout/Popup Preview */
            .ts-schematic {{
                background: var(--paper);
                border: 1px solid var(--rule);
                border-radius: 6px;
                padding: 10px 12px;
                display: flex;
                align-items: center;
                justify-content: space-between;
                gap: 10px;
            }}
            .ts-inline-controls {{
                background: var(--paper);
                border: 1px solid var(--rule);
                border-radius: 8px;
                padding: 12px 14px;
                display: flex;
                flex-direction: column;
                gap: 10px;
            }}
            .ts-inline-row {{
                display: flex;
                align-items: center;
                justify-content: space-between;
                gap: 10px;
                flex-wrap: wrap;
            }}
            .ts-toggle-pill {{
                display: inline-flex;
                align-items: center;
                gap: 6px;
                padding: 4px 10px;
                border-radius: 6px;
                border: 1px solid var(--rule-strong);
                background: var(--paper-raised);
                font-size: 12px;
                font-weight: 500;
                color: var(--ink);
                cursor: pointer;
                user-select: none;
            }}
            .ts-toggle-pill input {{
                margin: 0;
                cursor: pointer;
            }}
            .ts-toggle-pill.is-on {{
                border-color: var(--ink);
                background: var(--paper);
                font-weight: 600;
            }}

            /* Redesigned Site Shell Blueprint & Unified Slot Matrix */
            .ts-blueprint-bar {{
                display: grid;
                grid-template-columns: repeat(3, 1fr);
                background: var(--paper-raised);
                border: 1px solid var(--rule-strong);
                border-radius: 10px;
                margin-bottom: 18px;
                overflow: hidden;
            }}
            .ts-blueprint-slot {{
                padding: 14px 18px;
                border-right: 1px solid var(--rule);
                display: flex;
                flex-direction: column;
                gap: 8px;
            }}
            .ts-blueprint-slot:last-child {{
                border-right: none;
            }}
            .ts-seg-group {{
                display: inline-flex;
                background: var(--paper);
                border: 1px solid var(--rule-strong);
                border-radius: 7px;
                padding: 2px;
                gap: 2px;
                flex-wrap: wrap;
            }}
            .ts-seg-btn {{
                border: none;
                background: transparent;
                color: var(--ink-muted);
                font-size: 12px;
                font-weight: 500;
                padding: 5px 11px;
                border-radius: 5px;
                cursor: pointer;
                display: inline-flex;
                align-items: center;
                gap: 6px;
                font-variant-numeric: tabular-nums;
            }}
            .ts-seg-btn:hover {{
                color: var(--ink);
            }}
            .ts-seg-btn.active {{
                background: var(--paper-raised);
                color: var(--ink);
                font-weight: 600;
                box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
            }}

            .hf-matrix-table {{
                width: 100%;
                border-collapse: collapse;
            }}
            .hf-matrix-table th {{
                text-align: left;
                padding: 10px 14px;
                font-size: 11.5px;
                font-weight: 600;
                color: var(--ink-muted);
                border-bottom: 1px solid var(--rule-strong);
                background: var(--paper);
            }}
            .hf-matrix-table td {{
                padding: 12px 14px;
                border-bottom: 1px solid var(--rule);
                vertical-align: top;
                font-size: 13px;
            }}
            .hf-matrix-table tr:last-child td {{
                border-bottom: none;
            }}
            .hf-matrix-table tr:hover td {{
                background: rgba(15, 23, 42, 0.015);
            }}
            .hf-matrix-table tr.is-overridden-row td {{
                background: rgba(29, 95, 191, 0.02);
            }}

            /* Unified Interactive Slot Cell */
            .hf-slot-box {{
                border: 1px solid var(--rule-strong);
                border-radius: 8px;
                padding: 8px 10px;
                background: var(--paper-raised);
                display: flex;
                flex-direction: column;
                gap: 6px;
            }}
            .hf-slot-box.slot-custom {{
                border-color: #93c5fd;
                background: #f8fbff;
            }}
            .hf-slot-box.slot-hidden {{
                border-color: #fca5a5;
                background: #fffafa;
            }}
            .hf-slot-top {{
                display: flex;
                align-items: center;
                justify-content: space-between;
                gap: 8px;
            }}
            .hf-slot-select {{
                width: 100%;
                height: 32px;
                padding: 4px 8px;
                font-size: 12.5px;
                font-weight: 500;
                border: 1px solid var(--rule-strong);
                border-radius: 6px;
                background: var(--paper);
                color: var(--ink);
                cursor: pointer;
            }}
            .hf-slot-reset-btn {{
                font-size: 11px;
                font-weight: 600;
                color: var(--oxide);
                background: var(--paper);
                border: 1px solid var(--rule-strong);
                border-radius: 4px;
                padding: 1px 7px;
                cursor: pointer;
            }}
            .hf-slot-reset-btn:hover {{
                border-color: var(--oxide);
            }}

            /* Modals */
            .hf-modal-backdrop {{
                position: fixed;
                inset: 0;
                background: rgba(15, 23, 42, 0.48);
                z-index: 9999;
                display: none;
                align-items: center;
                justify-content: center;
                padding: 20px;
            }}
            .hf-modal-backdrop.open {{
                display: flex;
            }}
            .hf-modal-card {{
                background: var(--paper-raised);
                border: 1px solid var(--rule-strong);
                border-radius: 12px;
                width: 100%;
                max-width: 660px;
                max-height: 90vh;
                display: flex;
                flex-direction: column;
                overflow: hidden;
            }}
            .hf-modal-header {{
                padding: 16px 20px;
                border-bottom: 1px solid var(--rule);
                display: flex;
                justify-content: space-between;
                align-items: center;
            }}
            .hf-modal-body {{
                padding: 20px;
                overflow-y: auto;
                display: flex;
                flex-direction: column;
                gap: 16px;
            }}
            .hf-modal-footer {{
                padding: 14px 20px;
                border-top: 1px solid var(--rule);
                background: var(--paper);
                display: flex;
                justify-content: flex-end;
                gap: 10px;
            }}
            @media (max-width: 960px) {{
                .ts-kpi-strip {{ grid-template-columns: repeat(2, 1fr); }}
                .ts-blueprint-bar {{ grid-template-columns: 1fr; }}
                .hf-grid-2col {{ grid-template-columns: 1fr; }}
            }}
        </style>

        <!-- Page Header -->
        <div class="page-header" style="margin-bottom: 14px;">
            <div>
                <h1 class="page-title">{page_title}</h1>
                <p class="page-subtitle">{page_subtitle}</p>
            </div>
            <div style="display: flex; gap: 8px; align-items: center; flex-wrap: wrap;">
                {primary_actions}
            </div>
        </div>

        <!-- Theme Architecture Suite Switcher (framed-sheet 2px underline tabs) -->
        <div class="ts-suite-bar">
            <div class="ts-suite-tabs" role="navigation" aria-label="Theme Architecture Suite">
                <a href="/d-admin/headers-footers" id="suiteLinkHeadersFooters" class="ts-suite-link {suite_nav_hf}" onclick="if (window.tsPageMode === 'headers_footers') {{ event.preventDefault(); switchMainTab('templates'); }}">
                    <span>Headers &amp; Footers</span>
                    <span class="ts-count-pill" id="suiteCountHF">0</span>
                </a>
                <a href="/d-admin/headers-footers?tab=matrix" id="suiteLinkMatrix" class="ts-suite-link" onclick="if (window.tsPageMode === 'headers_footers') {{ event.preventDefault(); switchMainTab('matrix'); }}">
                    <span>Page Assignment Matrix</span>
                    <span class="ts-count-pill" id="suiteCountMatrix">0</span>
                </a>
                <a href="/d-admin/theme-layouts" id="suiteLinkLayouts" class="ts-suite-link {suite_nav_layouts}">
                    <span>Theme Layouts</span>
                    <span class="ts-count-pill" id="suiteCountLayouts">0</span>
                </a>
                <a href="/d-admin/symbols" id="suiteLinkSymbols" class="ts-suite-link {suite_nav_symbols}">
                    <span>Synced Symbols</span>
                    <span class="ts-count-pill" id="suiteCountSymbols">0</span>
                </a>
                <a href="/d-admin/popups" id="suiteLinkPopups" class="ts-suite-link {suite_nav_popups}">
                    <span>Popups &amp; Bars</span>
                    <span class="ts-count-pill" id="suiteCountPopups">0</span>
                </a>
            </div>
            <div style="font-size: 12px; color: var(--ink-muted); font-family: var(--font-mono); padding-bottom: 6px;">
                SQLite AST Engine &middot; Hot SSR Cache Active
            </div>
        </div>

        <!-- Divided KPI Strip (populated live by renderKpiStrip()) -->
        <div class="ts-kpi-strip" id="tsKpiStrip">
            <div class="ts-kpi-cell">
                <span class="ts-kpi-label" id="kpiLabel1">Active components</span>
                <span class="ts-kpi-value" id="kpiValue1">0</span>
                <span class="ts-kpi-sub" id="kpiSub1">Loading SQLite state...</span>
            </div>
            <div class="ts-kpi-cell">
                <span class="ts-kpi-label" id="kpiLabel2">Global coverage</span>
                <span class="ts-kpi-value" id="kpiValue2">0</span>
                <span class="ts-kpi-sub" id="kpiSub2"> Evaluating routes...</span>
            </div>
            <div class="ts-kpi-cell">
                <span class="ts-kpi-label" id="kpiLabel3">Route &amp; page rules</span>
                <span class="ts-kpi-value" id="kpiValue3">0</span>
                <span class="ts-kpi-sub" id="kpiSub3">Condition priority engine</span>
            </div>
            <div class="ts-kpi-cell">
                <span class="ts-kpi-label" id="kpiLabel4">SSR hydration</span>
                <span class="ts-kpi-value" id="kpiValue4">&lt;50 &micro;s</span>
                <span class="ts-kpi-sub" id="kpiSub4">Zero-flicker native injection</span>
            </div>
        </div>

        <!-- Dedicated Page Workspace -->
        {main_workspace_html}

        <!-- SHARED MODALS (Create, Display Conditions, Settings, Preview, Starters) -->
        {modals_html}

        {script_block}
        "##,
        page_title = escape_html(mode.title()),
        page_subtitle = escape_html(mode.subtitle()),
        primary_actions = primary_actions,
        suite_nav_hf = suite_nav_hf,
        suite_nav_layouts = suite_nav_layouts,
        suite_nav_symbols = suite_nav_symbols,
        suite_nav_popups = suite_nav_popups,
        main_workspace_html = main_workspace_html,
        modals_html = render_shared_modals_html(mode.default_create_type()),
        script_block = script_block,
    );

    render_admin_shell(mode.nav_tab(), mode.title(), &body, settings)
}

fn render_headers_footers_workspace() -> String {
    r##"
    <!-- Sub-view 1: Site Headers & Site Footers Cards -->
    <div id="tabPanelTemplates">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; flex-wrap: wrap; gap: 12px;">
            <div style="display: flex; align-items: center; gap: 10px;">
                <span style="font-size: 13px; color: var(--ink-muted);">Manage global and page-specific headers and footers, or switch to the</span>
                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 4px 10px;" onclick="switchMainTab('matrix')">Page Assignment Matrix &rarr;</button>
            </div>
        </div>

        <div style="display: flex; flex-direction: column; gap: 28px;">
            <!-- Site Headers Section -->
            <section>
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--rule);">
                    <div>
                        <h2 style="font-size: 16px; font-weight: 600; color: var(--ink); margin: 0;">Site Headers</h2>
                        <p style="font-size: 12.5px; color: var(--ink-muted); margin: 2px 0 0 0;">Injected at the top of matched routes (`header.dp-theme-header`). Toggle sticky, transparent, or scroll-shrink behavior inline.</p>
                    </div>
                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 12px;" onclick="openCreateModal('header')">New Header</button>
                </div>
                <div id="headersCardsList" class="hf-grid-2col"></div>
            </section>

            <!-- Site Footers Section -->
            <section>
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 12px; padding-bottom: 8px; border-bottom: 1px solid var(--rule);">
                    <div>
                        <h2 style="font-size: 16px; font-weight: 600; color: var(--ink); margin: 0;">Site Footers</h2>
                        <p style="font-size: 12.5px; color: var(--ink-muted); margin: 2px 0 0 0;">Injected at the bottom of matched routes (`footer.dp-theme-footer`) with optional sticky or parallax reveal behaviors.</p>
                    </div>
                    <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 12px;" onclick="openCreateModal('footer')">New Footer</button>
                </div>
                <div id="footersCardsList" class="hf-grid-2col"></div>
            </section>
        </div>
    </div>

    <!-- Sub-view 2: Redesigned Interactive Page & Route Assignment Matrix -->
    <div id="tabPanelMatrix" style="display: none;">
        <!-- Site Shell Global Default Blueprint Bar -->
        <div class="ts-blueprint-bar">
            <div class="ts-blueprint-slot">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="font-size: 11.5px; font-weight: 600; color: var(--ink-muted);">Global Default Header (Entire Site)</span>
                    <a id="bpEditGlobalHeader" href="/d-admin/headers-footers" style="font-size: 11.5px; color: var(--press); text-decoration: none; font-weight: 600; display: none;">Edit in DayStudio &nearr;</a>
                </div>
                <select id="globalDefaultHeaderSelect" class="form-select" style="font-size: 12.5px; font-weight: 600;" onchange="onGlobalDefaultSlotChange('header', this.value)"></select>
                <span style="font-size: 11.5px; color: var(--ink-muted);">Inherited by every route and page set to <strong>Inherit Global</strong>.</span>
            </div>

            <div class="ts-blueprint-slot">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="font-size: 11.5px; font-weight: 600; color: var(--ink-muted);">Active Content Body Layouts</span>
                    <a href="/d-admin/theme-layouts" style="font-size: 11.5px; color: var(--press); text-decoration: none; font-weight: 600;">Manage Layouts &rarr;</a>
                </div>
                <div id="bpActiveLayoutsSummary" style="display: flex; gap: 6px; flex-wrap: wrap; align-items: center; min-height: 32px;"></div>
                <span style="font-size: 11.5px; color: var(--ink-muted);">Hydrates `{{post.*}}`, `{{archive.*}}`, and `LoopGrid` slots.</span>
            </div>

            <div class="ts-blueprint-slot">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                    <span style="font-size: 11.5px; font-weight: 600; color: var(--ink-muted);">Global Default Footer (Entire Site)</span>
                    <a id="bpEditGlobalFooter" href="/d-admin/headers-footers" style="font-size: 11.5px; color: var(--press); text-decoration: none; font-weight: 600; display: none;">Edit in DayStudio &nearr;</a>
                </div>
                <select id="globalDefaultFooterSelect" class="form-select" style="font-size: 12.5px; font-weight: 600;" onchange="onGlobalDefaultSlotChange('footer', this.value)"></select>
                <span style="font-size: 11.5px; color: var(--ink-muted);">Inherited by every route and page set to <strong>Inherit Global</strong>.</span>
            </div>
        </div>

        <!-- Matrix Filter, Search & Bulk Assignment Bar -->
        <div class="day-card" style="overflow: hidden;">
            <div style="padding: 14px 18px; border-bottom: 1px solid var(--rule-strong); display: flex; justify-content: space-between; align-items: center; gap: 12px; flex-wrap: wrap; background: var(--paper-raised);">
                <div class="ts-seg-group" role="tablist" aria-label="Matrix Target Filter">
                    <button type="button" class="ts-seg-btn active" id="matFilterAll" onclick="setMatrixFilter('all')">All Targets <span id="matCountAll" style="font-family:var(--font-mono);font-size:11px;">0</span></button>
                    <button type="button" class="ts-seg-btn" id="matFilterRoutes" onclick="setMatrixFilter('routes')">System Routes <span id="matCountRoutes" style="font-family:var(--font-mono);font-size:11px;">0</span></button>
                    <button type="button" class="ts-seg-btn" id="matFilterPages" onclick="setMatrixFilter('pages')">Static Pages <span id="matCountPages" style="font-family:var(--font-mono);font-size:11px;">0</span></button>
                    <button type="button" class="ts-seg-btn" id="matFilterPosts" onclick="setMatrixFilter('posts')">Blog Posts <span id="matCountPosts" style="font-family:var(--font-mono);font-size:11px;">0</span></button>
                    <button type="button" class="ts-seg-btn" id="matFilterOverrides" onclick="setMatrixFilter('overrides')">Custom Overrides <span id="matCountOverrides" style="font-family:var(--font-mono);font-size:11px;">0</span></button>
                </div>

                <div style="display: flex; align-items: center; gap: 10px; flex-wrap: wrap;">
                    <button type="button" id="matResetAllBtn" class="btn btn-secondary" style="display: none; font-size: 12px; padding: 5px 10px; color: var(--oxide);" onclick="resetAllMatrixOverrides()">Reset All Overrides</button>
                    <input type="search" id="matrixPageSearch" class="form-input" placeholder="Search title or /slug..." style="width: 240px; font-size: 12.5px; padding: 6px 10px;" oninput="renderMatrixView()" />
                </div>
            </div>

            <!-- Bulk Assignment Bar (visible when rows are selected) -->
            <div id="matrixBulkBar" style="display: none; padding: 10px 18px; background: #eff6ff; border-bottom: 1px solid #bfdbfe; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap;">
                <div style="display: flex; align-items: center; gap: 8px;">
                    <span id="matrixBulkCountLabel" style="font-size: 12.5px; font-weight: 600; color: #1d4ed8; font-variant-numeric: tabular-nums;">0 targets selected</span>
                    <button type="button" class="btn btn-secondary" style="font-size: 11.5px; padding: 3px 8px;" onclick="clearMatrixSelection()">Clear</button>
                </div>
                <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                    <select id="bulkHeaderSelect" class="form-select" style="width: 210px; font-size: 12px; padding: 4px 8px;"></select>
                    <select id="bulkFooterSelect" class="form-select" style="width: 210px; font-size: 12px; padding: 4px 8px;"></select>
                    <button type="button" class="btn btn-primary" style="font-size: 12px; padding: 5px 12px;" onclick="applyBulkMatrixAssignment()">Apply to Selected</button>
                </div>
            </div>

            <!-- Unified Slot Matrix Table -->
            <div style="overflow-x: auto;">
                <table class="hf-matrix-table">
                    <thead>
                        <tr>
                            <th style="width: 36px; text-align: center;">
                                <input type="checkbox" id="matrixSelectAllChk" title="Select all visible targets" onchange="toggleSelectAllMatrixRows(this.checked)" />
                            </th>
                            <th style="width: 26%;">Target Page or System Route</th>
                            <th style="width: 31%;">Header Slot (Active &amp; Selector)</th>
                            <th style="width: 31%;">Footer Slot (Active &amp; Selector)</th>
                            <th style="width: 12%;">Body Layout</th>
                        </tr>
                    </thead>
                    <tbody id="unifiedMatrixTbody"></tbody>
                </table>
            </div>
        </div>
    </div>
    "##
    .to_string()
}

fn render_theme_layouts_workspace() -> String {
    r##"
    <div id="tabPanelLayouts">
        <!-- Filter & Quick Create Bar -->
        <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px; margin-bottom: 18px; flex-wrap: wrap;">
            <div class="ts-seg-group" role="tablist" aria-label="Filter Theme Layouts">
                <button type="button" class="ts-seg-btn active" id="layoutFilterAll" onclick="setLayoutFilter('all')">All Layouts <span id="layoutCountAll" style="font-family:var(--font-mono);font-size:11px;">0</span></button>
                <button type="button" class="ts-seg-btn" id="layoutFilterSinglePost" onclick="setLayoutFilter('single_post')">Single Post <span id="layoutCountSinglePost" style="font-family:var(--font-mono);font-size:11px;">0</span></button>
                <button type="button" class="ts-seg-btn" id="layoutFilterSinglePage" onclick="setLayoutFilter('single_page')">Single Page <span id="layoutCountSinglePage" style="font-family:var(--font-mono);font-size:11px;">0</span></button>
                <button type="button" class="ts-seg-btn" id="layoutFilterArchive" onclick="setLayoutFilter('archive')">Category &amp; Archive <span id="layoutCountArchive" style="font-family:var(--font-mono);font-size:11px;">0</span></button>
                <button type="button" class="ts-seg-btn" id="layoutFilter404" onclick="setLayoutFilter('404_page')">404 Recovery <span id="layoutCount404" style="font-family:var(--font-mono);font-size:11px;">0</span></button>
            </div>

            <div style="display: flex; gap: 6px; flex-wrap: wrap;">
                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openCreateModal('single_post')">+ Single Post</button>
                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openCreateModal('single_page')">+ Single Page</button>
                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openCreateModal('archive')">+ Archive Grid</button>
                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openCreateModal('404_page')">+ 404 Page</button>
            </div>
        </div>

        <!-- Dynamic Token Reference Strip -->
        <div style="background: var(--paper-raised); border: 1px solid var(--rule-strong); border-radius: 8px; padding: 10px 14px; margin-bottom: 20px; display: flex; align-items: center; justify-content: space-between; gap: 12px; flex-wrap: wrap;">
            <div style="display: flex; align-items: center; gap: 8px; flex-wrap: wrap;">
                <span style="font-size: 12px; font-weight: 600; color: var(--ink);">Available Dynamic Tokens:</span>
                <code class="hf-chip-token" onclick="copyTextToClipboard('{{post.title}}')" style="cursor:pointer;" title="Click to copy">{{post.title}}</code>
                <code class="hf-chip-token" onclick="copyTextToClipboard('{{post.content}}')" style="cursor:pointer;" title="Click to copy">{{post.content}}</code>
                <code class="hf-chip-token" onclick="copyTextToClipboard('{{post.excerpt}}')" style="cursor:pointer;" title="Click to copy">{{post.excerpt}}</code>
                <code class="hf-chip-token" onclick="copyTextToClipboard('{{post.date}}')" style="cursor:pointer;" title="Click to copy">{{post.date}}</code>
                <code class="hf-chip-token" onclick="copyTextToClipboard('{{author.name}}')" style="cursor:pointer;" title="Click to copy">{{author.name}}</code>
                <code class="hf-chip-token" onclick="copyTextToClipboard('{{archive.title}}')" style="cursor:pointer;" title="Click to copy">{{archive.title}}</code>
            </div>
            <span style="font-size: 11.5px; color: var(--ink-muted);">Click any token to copy for DayStudio headings, text blocks, or image sources.</span>
        </div>

        <div id="layoutsCardsList" class="hf-grid-2col"></div>
    </div>
    "##
    .to_string()
}

fn render_symbols_workspace() -> String {
    r##"
    <div id="tabPanelSymbols">
        <!-- Symbols Architecture Guide Strip -->
        <div style="background: var(--paper-raised); border: 1px solid var(--rule-strong); border-radius: 8px; padding: 12px 16px; margin-bottom: 20px; display: flex; align-items: center; justify-content: space-between; gap: 16px; flex-wrap: wrap;">
            <div style="display: flex; flex-direction: column; gap: 2px;">
                <strong style="font-size: 13px; color: var(--ink);">Synced Global Symbols &amp; Mega Menu Dropdown Panels</strong>
                <span style="font-size: 12px; color: var(--ink-muted);">Edit a symbol once in DayStudio to update every page that embeds it (<code>SymbolInstance</code>), or bind a symbol below to any Primary Navigation Menu item to render it as an interactive Mega Menu dropdown.</span>
            </div>
            <div style="display: flex; gap: 8px;">
                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 12px;" onclick="installStarterNow('global-cta-symbol', 'Synced Newsletter CTA Symbol')">Install CTA Symbol Starter</button>
                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 12px;" onclick="installStarterNow('mega-menu-product-symbol', 'Product Mega Menu Panel Symbol')">Install Mega Menu Starter</button>
            </div>
        </div>

        <div id="symbolsCardsList" class="hf-grid-2col"></div>
    </div>
    "##
    .to_string()
}

fn render_popups_workspace() -> String {
    r##"
    <div id="tabPanelPopups">
        <!-- Filter & Quick Create Bar -->
        <div style="display: flex; justify-content: space-between; align-items: center; gap: 12px; margin-bottom: 18px; flex-wrap: wrap;">
            <div class="ts-seg-group" role="tablist" aria-label="Filter Popups and Bars">
                <button type="button" class="ts-seg-btn active" id="popupFilterAll" onclick="setPopupFilter('all')">All Popups &amp; Bars <span id="popupCountAll" style="font-family:var(--font-mono);font-size:11px;">0</span></button>
                <button type="button" class="ts-seg-btn" id="popupFilterBars" onclick="setPopupFilter('bars')">Top &amp; Bottom Bars <span id="popupCountBars" style="font-family:var(--font-mono);font-size:11px;">0</span></button>
                <button type="button" class="ts-seg-btn" id="popupFilterModals" onclick="setPopupFilter('modals')">Center Modals <span id="popupCountModals" style="font-family:var(--font-mono);font-size:11px;">0</span></button>
                <button type="button" class="ts-seg-btn" id="popupFilterDrawers" onclick="setPopupFilter('drawers')">Slide-In Drawers <span id="popupCountDrawers" style="font-family:var(--font-mono);font-size:11px;">0</span></button>
            </div>

            <div style="display: flex; gap: 6px; flex-wrap: wrap;">
                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openCreatePopupWithVariant('top_bar')">+ Top Bar</button>
                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openCreatePopupWithVariant('bottom_bar')">+ Bottom Bar</button>
                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openCreatePopupWithVariant('slide_in_right')">+ Slide-In Drawer</button>
                <button type="button" class="btn btn-secondary" style="font-size: 12px; padding: 5px 10px;" onclick="openCreatePopupWithVariant('modal')">+ Center Modal</button>
            </div>
        </div>

        <div id="popupsCardsList" class="hf-grid-2col"></div>
    </div>
    "##
    .to_string()
}

fn render_shared_modals_html(default_type: &str) -> String {
    format!(
        r##"
        <!-- MODAL 1: CREATE NEW TEMPLATE -->
        <div id="hfCreateModal" class="hf-modal-backdrop" onclick="if(event.target===this)closeCreateModal()">
            <div class="hf-modal-card">
                <div class="hf-modal-header">
                    <h3 id="hfCreateModalTitle" style="font-size: 16px; font-weight: 600; margin: 0; color: var(--ink);">Create Theme Component</h3>
                    <button type="button" onclick="closeCreateModal()" style="background:none;border:none;font-size:18px;cursor:pointer;color:var(--ink-muted);">&times;</button>
                </div>
                <div class="hf-modal-body">
                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px;">
                        <div>
                            <label class="form-label">Component Type</label>
                            <select id="createTplType" class="form-select" data-default-type="{default_type}" onchange="onCreateTypeChange()">
                                <option value="header">Header (Site Top Navigation)</option>
                                <option value="footer">Footer (Site Bottom Section)</option>
                                <option value="single_post">Single Post Layout (Blog Articles)</option>
                                <option value="single_page">Single Page Layout (Static Pages)</option>
                                <option value="archive">Archive Layout (Categories / Blog Index)</option>
                                <option value="404_page">404 Error Page Layout</option>
                                <option value="section_symbol">Synced Global Symbol (Reusable Section)</option>
                                <option value="popup">Popup / Announcement Bar (Site-wide Overlay)</option>
                            </select>
                        </div>
                        <div>
                            <label class="form-label">Starting Blueprint</label>
                            <select id="createTplStarter" class="form-select" onchange="onCreateStarterChange()">
                                <option value="blank">Blank Canvas (Clean Root AST)</option>
                            </select>
                        </div>
                    </div>

                    <div>
                        <label class="form-label">Component Name</label>
                        <input type="text" id="createTplTitle" class="form-input" placeholder="e.g. Primary Site Header, Product Landing Header..." />
                    </div>

                    <div id="createAssignmentRow" style="display: grid; grid-template-columns: 1fr 120px; gap: 14px;">
                        <div>
                            <label class="form-label">Initial Display Assignment</label>
                            <select id="createTplTarget" class="form-select" onchange="onCreateTargetChange()">
                                <option value="entire_site">Entire Site (Global Default)</option>
                                <option value="front_page">Front Page Only</option>
                                <option value="all_pages">All Static Pages</option>
                                <option value="all_posts">All Blog Posts</option>
                                <option value="archive">All Archives &amp; Categories</option>
                                <option value="not_found">404 Not Found Page</option>
                                <option value="page">Specific Page Only...</option>
                                <option value="unassigned">Unassigned (Save as Draft / Assign Later)</option>
                            </select>
                        </div>
                        <div>
                            <label class="form-label">Rule Priority</label>
                            <input type="number" id="createTplPriority" class="form-input" value="10" min="1" max="999" />
                        </div>
                    </div>

                    <div id="createTplPageWrap" style="display: none;">
                        <label class="form-label">Select Target Page</label>
                        <select id="createTplPageId" class="form-select"></select>
                    </div>

                    <div id="createPopupBehaviors" style="display: none; background: var(--paper); border: 1px solid var(--rule); border-radius: 8px; padding: 14px; flex-direction: column; gap: 10px;">
                        <strong style="font-size: 12.5px; color: var(--ink);">Popup Placement, Trigger &amp; Cooldown</strong>
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px;">
                            <div>
                                <label class="form-label" style="font-size: 11.5px;">Placement Variant</label>
                                <select id="createTplPopupVariant" class="form-select">
                                    <option value="modal">Center Modal Dialog</option>
                                    <option value="top_bar">Top Announcement Bar</option>
                                    <option value="bottom_bar">Bottom Sticky Bar</option>
                                    <option value="slide_in_right">Slide-In Right Drawer</option>
                                    <option value="slide_in_left">Slide-In Left Drawer</option>
                                </select>
                            </div>
                            <div>
                                <label class="form-label" style="font-size: 11.5px;">Trigger Event</label>
                                <select id="createTplPopupTrigger" class="form-select">
                                    <option value="on_load">On Page Load (with Delay)</option>
                                    <option value="on_scroll">On Scroll Depth (%)</option>
                                    <option value="exit_intent">On Exit Intent (Mouse Leave)</option>
                                </select>
                            </div>
                        </div>
                        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px;">
                            <div>
                                <label class="form-label" style="font-size: 11.5px;">Delay (seconds)</label>
                                <input type="number" id="createTplPopupDelaySec" class="form-input" value="0" min="0" max="120" />
                            </div>
                            <div>
                                <label class="form-label" style="font-size: 11.5px;">Scroll Depth (%)</label>
                                <input type="number" id="createTplPopupScrollPct" class="form-input" value="50" min="5" max="100" />
                            </div>
                            <div>
                                <label class="form-label" style="font-size: 11.5px;">Dismiss Cooldown (days)</label>
                                <input type="number" id="createTplPopupDismissDays" class="form-input" value="7" min="0" max="365" />
                            </div>
                        </div>
                    </div>
                </div>
                <div class="hf-modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="closeCreateModal()">Cancel</button>
                    <button type="button" class="btn btn-secondary" onclick="submitCreateTemplate(false)">Create Component</button>
                    <button type="button" class="btn btn-primary" onclick="submitCreateTemplate(true)">Create &amp; Open in DayStudio &nearr;</button>
                </div>
            </div>
        </div>

        <!-- MODAL 2: DISPLAY CONDITIONS & PAGE PICKER -->
        <div id="hfConditionsModal" class="hf-modal-backdrop" onclick="if(event.target===this)closeConditionsModal()">
            <div class="hf-modal-card" style="max-width: 720px;">
                <div class="hf-modal-header">
                    <div>
                        <h3 id="hfCondModalTitle" style="font-size: 16px; font-weight: 600; margin: 0; color: var(--ink);">Display Conditions</h3>
                        <p style="font-size: 12px; color: var(--ink-muted); margin: 2px 0 0 0;">Configure exact Include and Exclude rules. Exclude rules always override Include rules.</p>
                    </div>
                    <button type="button" onclick="closeConditionsModal()" style="background:none;border:none;font-size:18px;cursor:pointer;color:var(--ink-muted);">&times;</button>
                </div>
                <div class="hf-modal-body">
                    <div id="hfCondRowsContainer" style="display: flex; flex-direction: column; gap: 10px;"></div>
                    <div>
                        <button type="button" class="btn btn-secondary" style="font-size: 12px;" onclick="addConditionModalRow()">+ Add Condition Rule</button>
                    </div>
                </div>
                <div class="hf-modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="closeConditionsModal()">Cancel</button>
                    <button type="button" class="btn btn-primary" onclick="saveConditionsModal()">Save Display Conditions</button>
                </div>
            </div>
        </div>

        <!-- MODAL 3: TEMPLATE SETTINGS & BEHAVIOR -->
        <div id="hfSettingsModal" class="hf-modal-backdrop" onclick="if(event.target===this)closeSettingsModal()">
            <div class="hf-modal-card">
                <div class="hf-modal-header">
                    <h3 id="hfSettingsModalTitle" style="font-size: 16px; font-weight: 600; margin: 0; color: var(--ink);">Component Settings &amp; Behavior</h3>
                    <button type="button" onclick="closeSettingsModal()" style="background:none;border:none;font-size:18px;cursor:pointer;color:var(--ink-muted);">&times;</button>
                </div>
                <div class="hf-modal-body">
                    <div style="display: grid; grid-template-columns: 1fr 140px 110px; gap: 12px;">
                        <div>
                            <label class="form-label">Component Title</label>
                            <input type="text" id="setTplTitle" class="form-input" />
                        </div>
                        <div>
                            <label class="form-label">Status</label>
                            <select id="setTplStatus" class="form-select">
                                <option value="publish">Published</option>
                                <option value="draft">Draft</option>
                            </select>
                        </div>
                        <div>
                            <label class="form-label">Priority</label>
                            <input type="number" id="setTplPriority" class="form-input" value="10" />
                        </div>
                    </div>

                    <div id="setHeaderBehaviors" style="background: var(--paper); border: 1px solid var(--rule); border-radius: 8px; padding: 14px; display: flex; flex-direction: column; gap: 10px;">
                        <strong style="font-size: 12.5px; color: var(--ink);">Header Scroll &amp; Positioning Behavior</strong>
                        <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer;">
                            <input type="checkbox" id="setTplSticky" />
                            <span><strong>Sticky Header</strong> &mdash; Pin header to top of viewport while scrolling (<code>position: sticky; top: 0</code>)</span>
                        </label>
                        <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer;">
                            <input type="checkbox" id="setTplTransparent" />
                            <span><strong>Transparent Overlay Header</strong> &mdash; Float over hero banner with backdrop blur</span>
                        </label>
                        <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer;">
                            <input type="checkbox" id="setTplScrollShrink" />
                            <span><strong>Shrink on Scroll</strong> &mdash; Compact vertical padding after scrolling past top fold</span>
                        </label>
                        <div style="display: flex; align-items: center; gap: 10px; margin-top: 4px;">
                            <label class="form-label" style="margin: 0; font-size: 12px;">Custom Z-Index:</label>
                            <input type="number" id="setTplZIndex" class="form-input" placeholder="999" style="width: 110px;" />
                        </div>
                    </div>

                    <div id="setFooterBehaviors" style="background: var(--paper); border: 1px solid var(--rule); border-radius: 8px; padding: 14px; display: none; flex-direction: column; gap: 10px;">
                        <strong style="font-size: 12.5px; color: var(--ink);">Footer Positioning Behavior</strong>
                        <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer;">
                            <input type="checkbox" id="setTplStickyFooter" />
                            <span><strong>Sticky Bottom Footer</strong> &mdash; Keep footer anchored to bottom edge</span>
                        </label>
                        <label style="display: flex; align-items: center; gap: 8px; font-size: 13px; cursor: pointer;">
                            <input type="checkbox" id="setTplRevealScroll" />
                            <span><strong>Parallax Reveal on Scroll</strong> &mdash; Uncover footer smoothly from beneath page body</span>
                        </label>
                    </div>

                    <div id="setPopupBehaviors" style="background: var(--paper); border: 1px solid var(--rule); border-radius: 8px; padding: 14px; display: none; flex-direction: column; gap: 10px;">
                        <strong style="font-size: 12.5px; color: var(--ink);">Popup / Announcement Bar Trigger &amp; Frequency Rules</strong>
                        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 10px;">
                            <div>
                                <label class="form-label" style="font-size: 11.5px;">Placement Variant</label>
                                <select id="setTplPopupVariant" class="form-select">
                                    <option value="modal">Center Modal Dialog</option>
                                    <option value="top_bar">Top Announcement Bar</option>
                                    <option value="bottom_bar">Bottom Sticky Bar</option>
                                    <option value="slide_in_right">Slide-In Right Drawer</option>
                                    <option value="slide_in_left">Slide-In Left Drawer</option>
                                </select>
                            </div>
                            <div>
                                <label class="form-label" style="font-size: 11.5px;">Trigger Event</label>
                                <select id="setTplPopupTrigger" class="form-select">
                                    <option value="on_load">On Page Load (with Delay)</option>
                                    <option value="on_scroll">On Scroll Depth (%)</option>
                                    <option value="exit_intent">On Exit Intent (Mouse Leave)</option>
                                </select>
                            </div>
                        </div>
                        <div style="display: grid; grid-template-columns: 1fr 1fr 1fr; gap: 10px;">
                            <div>
                                <label class="form-label" style="font-size: 11.5px;">Delay (seconds)</label>
                                <input type="number" id="setTplPopupDelaySec" class="form-input" value="0" min="0" max="120" />
                            </div>
                            <div>
                                <label class="form-label" style="font-size: 11.5px;">Scroll Depth (%)</label>
                                <input type="number" id="setTplPopupScrollPct" class="form-input" value="50" min="5" max="100" />
                            </div>
                            <div>
                                <label class="form-label" style="font-size: 11.5px;">Dismiss Cooldown (days)</label>
                                <input type="number" id="setTplPopupDismissDays" class="form-input" value="7" min="0" max="365" />
                            </div>
                        </div>
                    </div>

                    <div>
                        <label class="form-label">Scoped Custom CSS</label>
                        <textarea id="setTplCustomCss" class="form-textarea" rows="4" style="font-family: var(--font-mono); font-size: 12px;" placeholder="/* Scoped CSS injected alongside this component */"></textarea>
                    </div>
                </div>
                <div class="hf-modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="closeSettingsModal()">Cancel</button>
                    <button type="button" class="btn btn-primary" onclick="saveSettingsModal()">Save Settings</button>
                </div>
            </div>
        </div>

        <!-- MODAL 4: LIVE STANDALONE PREVIEW -->
        <div id="hfPreviewModal" class="hf-modal-backdrop" onclick="if(event.target===this)closePreviewModal()">
            <div class="hf-modal-card" style="max-width: 1060px; height: 82vh;">
                <div class="hf-modal-header">
                    <div style="display: flex; align-items: center; gap: 12px;">
                        <h3 id="hfPreviewModalTitle" style="font-size: 15px; font-weight: 600; margin: 0; color: var(--ink);">Live Component Preview</h3>
                        <div style="display: flex; gap: 4px;">
                            <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 3px 8px;" onclick="setPreviewViewport('100%')">Desktop</button>
                            <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 3px 8px;" onclick="setPreviewViewport('768px')">Tablet</button>
                            <button type="button" class="btn btn-secondary" style="font-size: 11px; padding: 3px 8px;" onclick="setPreviewViewport('390px')">Mobile</button>
                        </div>
                    </div>
                    <div style="display: flex; gap: 8px; align-items: center;">
                        <a id="hfPreviewOpenStudioBtn" href="/d-admin/builder" class="btn btn-primary" style="font-size: 12px; padding: 5px 12px;">Open in DayStudio &nearr;</a>
                        <button type="button" onclick="closePreviewModal()" style="background:none;border:none;font-size:18px;cursor:pointer;color:var(--ink-muted);">&times;</button>
                    </div>
                </div>
                <div style="flex: 1; background: var(--paper); display: flex; justify-content: center; overflow: auto; padding: 16px;">
                    <iframe id="hfPreviewIframe" src="about:blank" style="width: 100%; height: 100%; border: 1px solid var(--rule-strong); border-radius: 8px; background: #fff; transition: width 0.2s ease;"></iframe>
                </div>
            </div>
        </div>

        <!-- MODAL 5: 1-CLICK STARTER CATALOG -->
        <div id="hfStartersModal" class="hf-modal-backdrop" onclick="if(event.target===this)closeStarterCatalogModal()">
            <div class="hf-modal-card" style="max-width: 780px;">
                <div class="hf-modal-header">
                    <div>
                        <h3 style="font-size: 16px; font-weight: 600; margin: 0; color: var(--ink);">Starter Blueprint Catalog</h3>
                        <p style="font-size: 12px; color: var(--ink-muted); margin: 2px 0 0 0;">Install pre-designed DayStudio AST layouts directly into SQLite and customize them visually.</p>
                    </div>
                    <button type="button" onclick="closeStarterCatalogModal()" style="background:none;border:none;font-size:18px;cursor:pointer;color:var(--ink-muted);">&times;</button>
                </div>
                <div class="hf-modal-body">
                    <div id="hfStartersGrid" style="display: grid; grid-template-columns: repeat(auto-fill, minmax(320px, 1fr)); gap: 14px;"></div>
                </div>
                <div class="hf-modal-footer">
                    <button type="button" class="btn btn-secondary" onclick="closeStarterCatalogModal()">Close</button>
                </div>
            </div>
        </div>
        "##,
        default_type = escape_html(default_type),
    )
}
