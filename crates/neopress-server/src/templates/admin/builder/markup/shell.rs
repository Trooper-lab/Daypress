//! 3-Pane Zero-Scroll CAD Dock Shell markup (`markup/shell.rs`).

pub fn render_shell() -> &'static str {
    r#"<div id="neostudio-shell">
    <!-- Top Studio Bar (40px) -->
    <header id="neostudio-topbar">
        <div class="topbar-left">
            <a href="/n-admin/" class="topbar-brand" title="Exit to Neopress Dashboard">
                <span class="brand-glyph"><i class="fa-solid fa-cube"></i></span>
                <span class="brand-name">NEOSTUDIO</span>
            </a>
            <div class="topbar-divider"></div>
            <div class="topbar-switcher" id="topbar-template-switcher">
                <button class="switcher-btn" id="btn-switch-template" title="Switch Page or Template Context">
                    <span class="template-type-badge" id="badge-template-type">__POST_TYPE__</span>
                    <span class="template-title" id="lbl-template-title">__POST_TITLE__</span>
                    <i class="fa-solid fa-chevron-down switcher-caret"></i>
                </button>
                <div class="switcher-dropdown" id="topbar-switcher-dropdown">
                    <div class="switcher-search-box">
                        <input type="search" class="cad-input" id="switcher-search-input" placeholder="Filter pages, posts, templates...">
                    </div>
                    <div class="switcher-scroll-list" id="switcher-items-container"></div>
                    <div class="switcher-footer">
                        <button class="cad-btn secondary" id="switcher-btn-new-page" style="flex:1; height:24px; font-size:11px;">
                            <i class="fa-solid fa-file-circle-plus"></i> New Page
                        </button>
                        <button class="cad-btn primary" id="switcher-btn-new-template" style="flex:1; height:24px; font-size:11px;">
                            <i class="fa-solid fa-shapes"></i> New Template
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <div class="topbar-center">
            <!-- Breakpoints Group -->
            <div class="breakpoint-switcher" role="radiogroup" aria-label="Responsive Breakpoints">
                <button class="bp-btn active" data-bp="desktop" title="Desktop Viewport (1200px+)">
                    <i class="fa-solid fa-desktop"></i>
                    <span>Desktop</span>
                </button>
                <button class="bp-btn" data-bp="tablet" title="Tablet Viewport (768px)">
                    <i class="fa-solid fa-tablet-screen-button"></i>
                    <span>Tablet</span>
                </button>
                <button class="bp-btn" data-bp="mobile" title="Mobile Viewport (375px)">
                    <i class="fa-solid fa-mobile-screen-button"></i>
                    <span>Mobile</span>
                </button>
            </div>

            <!-- Viewport Live Dimension Readout -->
            <div class="dimension-readout" id="lbl-dimension-readout" title="Canvas Dimensions">
                1200 <span class="dim-unit">px</span> &times; 100%
            </div>

            <!-- Canvas Zoom Engine -->
            <div class="zoom-controller">
                <button class="zoom-btn" id="btn-zoom-out" title="Zoom Out (Ctrl+-)"><i class="fa-solid fa-minus"></i></button>
                <button class="zoom-pill" id="btn-zoom-menu" title="Zoom Presets">
                    <span id="lbl-zoom-level">100%</span>
                </button>
                <button class="zoom-btn" id="btn-zoom-in" title="Zoom In (Ctrl++)"><i class="fa-solid fa-plus"></i></button>
                <button class="zoom-btn" id="btn-zoom-reset" title="Reset Zoom (Ctrl+0)"><i class="fa-solid fa-compress"></i></button>
            </div>
        </div>

        <div class="topbar-right">
            <!-- X-Ray Wireframe Mode Toggle -->
            <button class="topbar-tool-btn" id="btn-toggle-xray" title="Toggle Structural X-Ray Mode (Ctrl+Shift+X)">
                <i class="fa-solid fa-border-all"></i>
                <span>X-Ray</span>
            </button>

            <!-- Command Palette Spotlight -->
            <button class="topbar-tool-btn" id="btn-open-palette" title="Command Palette (Cmd+K / Ctrl+K)">
                <i class="fa-solid fa-magnifying-glass"></i>
                <kbd class="kbd-hint">⌘K</kbd>
            </button>

            <!-- Undo / Redo -->
            <div class="history-quick-tools">
                <button class="history-btn disabled" id="btn-undo" title="Undo Mutation (Ctrl+Z)">
                    <i class="fa-solid fa-arrow-rotate-left"></i>
                </button>
                <button class="history-btn disabled" id="btn-redo" title="Redo Mutation (Ctrl+Shift+Z)">
                    <i class="fa-solid fa-arrow-rotate-right"></i>
                </button>
            </div>

            <!-- Raw JSON AST -->
            <button class="topbar-icon-btn" id="btn-open-ast-modal" title="View Canonical Neopress JSON AST">
                <i class="fa-solid fa-code"></i>
            </button>

            <!-- View Live Frontend Page -->
            <a href="/__POST_SLUG__/" target="_blank" class="topbar-tool-btn" id="btn-view-live" title="View Live Page on Frontend (Opens in New Tab)" style="text-decoration: none;">
                <i class="fa-solid fa-arrow-up-right-from-square"></i>
                <span>View Live</span>
            </a>

            <!-- SQLite Dual-Write Save & Publish Indicator -->
            <div class="save-status-group">
                <div class="save-status-pill saved" id="status-save-pill" title="Persistence State">
                    <span class="status-dot"></span>
                    <span class="status-text" id="lbl-save-status">Saved</span>
                </div>
                <button class="btn-publish-action" id="btn-primary-publish" title="Save Layout to SQLite (Ctrl+S)">
                    <i class="fa-solid fa-cloud-arrow-up"></i>
                    <span id="btn-save-text">Publish</span>
                </button>
            </div>
        </div>
    </header>

    <!-- Main Workstation Dock -->
    <div id="neostudio-dock">
        <!-- Left Studio Rail (48px) -->
        <nav id="neostudio-rail">
            <div class="rail-top">
                <button class="rail-tab-btn active" data-drawer="elements" id="rail-btn-elements" title="Add Elements (Q or /)">
                    <i class="fa-solid fa-plus"></i>
                </button>
                <button class="rail-tab-btn" data-drawer="navigator" id="rail-btn-navigator" title="Layers Tree Navigator (Ctrl+I)">
                    <i class="fa-solid fa-layer-group"></i>
                </button>
                <button class="rail-tab-btn" data-drawer="templates" id="rail-btn-templates" title="Theme Templates (Ctrl+T)">
                    <i class="fa-solid fa-shapes"></i>
                </button>
                <button class="rail-tab-btn" data-drawer="tokens" id="rail-btn-tokens" title="Global Tokens & Styles (Ctrl+G)">
                    <i class="fa-solid fa-sliders"></i>
                </button>
                <button class="rail-tab-btn" data-drawer="css" id="rail-btn-css" title="Custom CSS & Element Classes (Ctrl+E)">
                    <i class="fa-solid fa-file-code"></i>
                </button>
                <button class="rail-tab-btn" data-drawer="history" id="rail-btn-history" title="Audit History (Ctrl+H)">
                    <i class="fa-solid fa-clock-rotate-left"></i>
                </button>
            </div>
            <div class="rail-bottom">
                <button class="rail-tab-btn" id="rail-btn-settings" title="Page & SEO Settings (Ctrl+,)">
                    <i class="fa-solid fa-gear"></i>
                </button>
                <button class="rail-tab-btn" id="rail-btn-help" title="Keyboard Shortcuts (?)">
                    <i class="fa-solid fa-circle-question"></i>
                </button>
            </div>
        </nav>

        <!-- Left Drawer (280px) -->
        <aside id="neostudio-drawer" class="drawer-open">
            <div class="drawer-panel active" id="drawer-panel-container">
                <!-- Dynamically populated: Elements, Navigator, Templates, Tokens, History -->
            </div>
        </aside>

        <!-- Center Responsive Artboard Canvas -->
        <div id="neostudio-canvas-viewport">
            <div id="neostudio-zoom-wrapper">
                <!-- Viewport Drag Resize Handles outside clipped artboard -->
                <div class="artboard-handle artboard-handle-left" id="handle-artboard-left" title="Drag to Resize Viewport Width">
                    <span class="handle-grip"></span>
                </div>
                <div id="neostudio-artboard" class="artboard-desktop">
                    <!-- SSR Public Site Header Preview -->
                    <div id="neostudio-ssr-header">__PREVIEW_HEADER_HTML__</div>

                    <!-- Interactive Editable Canvas -->
                    <main id="neostudio-canvas" class="np-visual-canvas"></main>

                    <!-- Bottom Add Section Preset Dropzone -->
                    <div class="neostudio-add-section-area" id="neostudio-bottom-dropzone">
                        <div class="add-section-prompt">
                            <button class="btn-add-section-action" id="btn-open-presets" title="Add Section Preset">
                                <i class="fa-solid fa-plus"></i>
                                <span>Add Section</span>
                            </button>
                            <button class="btn-add-section-action" id="btn-open-starters-prompt" title="Insert Starter Template">
                                <i class="fa-solid fa-shapes"></i>
                                <span>Template Library</span>
                            </button>
                        </div>
                        <div class="preset-selector" id="preset-selector">
                            <div class="preset-title">Select Column Layout</div>
                            <div class="preset-grid">
                                <div class="preset-card" data-cols="1" title="1 Column (100%)">
                                    <div class="preset-col" style="flex: 1;"></div>
                                </div>
                                <div class="preset-card" data-cols="2" title="2 Columns (50% / 50%)">
                                    <div class="preset-col" style="flex: 1;"></div>
                                    <div class="preset-col" style="flex: 1;"></div>
                                </div>
                                <div class="preset-card" data-cols="3" title="3 Columns (33% / 33% / 33%)">
                                    <div class="preset-col" style="flex: 1;"></div>
                                    <div class="preset-col" style="flex: 1;"></div>
                                    <div class="preset-col" style="flex: 1;"></div>
                                </div>
                                <div class="preset-card" data-cols="4" title="4 Columns (25% each)">
                                    <div class="preset-col" style="flex: 1;"></div>
                                    <div class="preset-col" style="flex: 1;"></div>
                                    <div class="preset-col" style="flex: 1;"></div>
                                    <div class="preset-col" style="flex: 1;"></div>
                                </div>
                                <div class="preset-card" data-cols="33-66" title="2 Columns (33% / 66%)">
                                    <div class="preset-col" style="flex: 1;"></div>
                                    <div class="preset-col" style="flex: 2;"></div>
                                </div>
                                <div class="preset-card" data-cols="66-33" title="2 Columns (66% / 33%)">
                                    <div class="preset-col" style="flex: 2;"></div>
                                    <div class="preset-col" style="flex: 1;"></div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <!-- SSR Public Site Footer Preview -->
                    <div id="neostudio-ssr-footer">__PREVIEW_FOOTER_HTML__</div>
                </div>
                <div class="artboard-handle artboard-handle-right" id="handle-artboard-right" title="Drag to Resize Viewport Width">
                    <span class="handle-grip"></span>
                </div>
            </div>

            <!-- On-Canvas Selection Chrome & Action Chips -->
            <div id="neostudio-selection-chrome" style="display: none;">
                <div class="selection-outline"></div>
                <div class="selection-badge" id="neo-selection-badge">
                    <span class="badge-kind" id="sel-badge-kind">Element</span>
                    <span class="badge-id" id="sel-badge-id">#node</span>
                    <div class="badge-actions">
                        <button class="badge-action-btn" id="act-drag-node" title="Drag to Reorder" draggable="true"><i class="fa-solid fa-grip-vertical"></i></button>
                        <button class="badge-action-btn" id="act-parent-node" title="Select Parent Container"><i class="fa-solid fa-arrow-up"></i></button>
                        <button class="badge-action-btn" id="act-dup-node" title="Duplicate Node (Cmd+D)"><i class="fa-regular fa-copy"></i></button>
                        <button class="badge-action-btn" id="act-del-node" title="Delete Node (Del)"><i class="fa-regular fa-trash-can"></i></button>
                    </div>
                </div>
            </div>

            <!-- Live Margin & Padding Box-Model Overlays -->
            <div id="neostudio-box-overlays" style="display: none;">
                <div class="overlay-margin" id="overlay-margin-top"><span class="overlay-chip">0</span></div>
                <div class="overlay-margin" id="overlay-margin-right"><span class="overlay-chip">0</span></div>
                <div class="overlay-margin" id="overlay-margin-bottom"><span class="overlay-chip">0</span></div>
                <div class="overlay-margin" id="overlay-margin-left"><span class="overlay-chip">0</span></div>
                <div class="overlay-padding" id="overlay-padding-top"><span class="overlay-chip">0</span></div>
                <div class="overlay-padding" id="overlay-padding-right"><span class="overlay-chip">0</span></div>
                <div class="overlay-padding" id="overlay-padding-bottom"><span class="overlay-chip">0</span></div>
                <div class="overlay-padding" id="overlay-padding-left"><span class="overlay-chip">0</span></div>
            </div>

            <!-- Drop Insertion Indicator Line -->
            <div id="neostudio-drop-guide" class="neostudio-drop-guide"></div>
        </div>

        <!-- Right Webflow-Class Inspector (300px) -->
        <aside id="neostudio-inspector">
            <!-- Dynamically populated: Tab bar, Selector/State bar, Box-Model, Style panels, Settings -->
        </aside>
    </div>

    <!-- Bottom Breadcrumb Bar (28px) -->
    <div id="neostudio-bottom-bar">
        <div class="bottom-bar-left">
            <span class="bc-label">Hierarchy:</span>
            <div class="breadcrumb-trail" id="neostudio-breadcrumbs">
                <span class="bc-item" data-id="root">Body</span>
            </div>
        </div>
        <div class="bottom-bar-right">
            <span class="hud-stat" id="hud-stat-dimensions">1200 &times; 800 px</span>
            <span class="hud-stat-sep">|</span>
            <span class="hud-stat" id="hud-stat-zoom">100%</span>
            <span class="hud-stat-sep">|</span>
            <span class="hud-stat" id="hud-stat-xray">X-Ray: OFF</span>
        </div>
    </div>
</div>"#
}
