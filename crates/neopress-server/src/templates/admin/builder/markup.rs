use super::styles::get_styles;
use super::scripts::get_scripts;

pub fn get_markup(
    post_id: i64,
    post_title: &str,
    post_slug: &str,
    post_type: &str,
    site_title: &str,
    preview_nav_html: &str,
    layout_json_raw: &str,
) -> String {
    let styles = get_styles();
    let scripts = get_scripts();

    let template = r###"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Elementor Visual Builder &ndash; __POST_TITLE__ &ndash; __SITE_TITLE__</title>
    <!-- Google Fonts: Roboto & Inter -->
    <link rel="preconnect" href="https://fonts.googleapis.com">
    <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin>
    <link href="https://fonts.googleapis.com/css2?family=Roboto:wght@300;400;500;700&family=Inter:wght@400;500;600;700&display=swap" rel="stylesheet">
    <!-- FontAwesome 6 Free Vector Icons (Extensive Icon Library) -->
    <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.5.1/css/all.min.css" integrity="sha512-DTOQO9RWCH3ppGqcWaEA1BIZOC6xxalwEsw9c2QQeAIftl+Vegovlnee1c9QX4TctnWMn13TZye+giMm8e2LwA==" crossorigin="anonymous" referrerpolicy="no-referrer" />
    <style>__STYLES__</style>
</head>
<body>

<div id="elementor-editor-wrapper">

    <!-- ============================================================== -->
    <!-- LEFT PANEL: #elementor-panel                                   -->
    <!-- ============================================================== -->
    <aside id="elementor-panel">
        <!-- Header -->
        <header id="elementor-panel-header">
            <button id="elementor-panel-header-menu-button" class="elementor-header-button" title="Menu">
                <i class="fa-solid fa-bars"></i>
            </button>
            <div id="elementor-panel-header-title">
                <span class="e-logo">E</span> ELEMENTOR
            </div>
            <button id="elementor-panel-header-add-button" class="elementor-header-button" title="Widgets Panel">
                <i class="fa-solid fa-table-cells"></i>
            </button>
        </header>

        <!-- Navigation Tabs -->
        <nav class="elementor-panel-navigation" id="panel-nav">
            <!-- Dynamically populated: Elements/Global vs Content/Style/Advanced -->
        </nav>

        <!-- Content Area -->
        <main id="elementor-panel-content-wrapper">
            <!-- Dynamically populated: Widget drawer vs Element Controls -->
        </main>

        <!-- Footer Dock -->
        <footer id="elementor-panel-footer">
            <div class="elementor-panel-footer-tools">
                <button class="elementor-panel-footer-tool" id="tool-settings" title="Page Settings">
                    <i class="fa-solid fa-gear"></i>
                </button>
                <button class="elementor-panel-footer-tool active" id="tool-navigator" title="Structure / Layers Navigator (Ctrl+I)">
                    <i class="fa-solid fa-layer-group"></i>
                </button>
                <button class="elementor-panel-footer-tool" id="tool-history" title="History">
                    <i class="fa-solid fa-clock-rotate-left"></i>
                </button>
                <button class="elementor-panel-footer-tool" id="tool-responsive" title="Responsive Mode">
                    <i class="fa-solid fa-desktop"></i>
                </button>
                <button class="elementor-panel-footer-tool" id="tool-preview" title="Preview Changes">
                    <i class="fa-regular fa-eye"></i>
                </button>
            </div>

            <div class="elementor-panel-footer-saver">
                <button id="elementor-panel-saver-button-publish" title="Save changes (Ctrl+S)">
                    <span id="btn-save-text">UPDATE</span>
                </button>
                <button id="elementor-panel-saver-options-btn" title="Save Options">
                    <i class="fa-solid fa-caret-up"></i>
                </button>
            </div>

            <!-- Saver dropdown -->
            <div id="saver-options-menu">
                <button class="saver-menu-item" id="opt-save-draft">
                    <i class="fa-regular fa-file"></i> Save Draft
                </button>
                <button class="saver-menu-item" id="opt-save-template">
                    <i class="fa-regular fa-floppy-disk"></i> Save as Template
                </button>
                <button class="saver-menu-item" id="opt-view-ast">
                    <i class="fa-solid fa-code"></i> Raw JSON AST
                </button>
                <button class="saver-menu-item" id="opt-exit">
                    <i class="fa-solid fa-arrow-right-from-bracket"></i> Exit to Dashboard
                </button>
            </div>
        </footer>
    </aside>

    <!-- ============================================================== -->
    <!-- RIGHT PREVIEW CANVAS: #elementor-preview                       -->
    <!-- ============================================================== -->
    <main id="elementor-preview">
        <!-- Floating Responsive Bar (Toggled by tool-responsive) -->
        <div id="elementor-responsive-bar">
            <button class="resp-btn active" data-mode="desktop">
                <i class="fa-solid fa-desktop"></i> Desktop
            </button>
            <button class="resp-btn" data-mode="tablet">
                <i class="fa-solid fa-tablet-screen-button"></i> Tablet (768px)
            </button>
            <button class="resp-btn" data-mode="mobile">
                <i class="fa-solid fa-mobile-screen-button"></i> Mobile (375px)
            </button>
            <span class="resp-dimensions" id="resp-dimensions-text">100% / 1200px</span>
            <button class="btn-close-resp-bar" id="btn-close-resp-bar" title="Exit Responsive Bar">
                <i class="fa-solid fa-xmark"></i>
            </button>
        </div>

        <!-- Floating Layers Pill on Canvas -->
        <button id="btn-floating-navigator" class="floating-layers-btn" title="Toggle Structure / Layers Overview">
            <i class="fa-solid fa-layer-group"></i> Structure / Layers
        </button>

        <!-- Drop Insertion Indicator Line moved to body root -->

        <!-- Responsive Viewport Wrapper -->
        <div id="elementor-preview-responsive-wrapper">
            <!-- Live Website Header Preview -->
            <header class="preview-site-header">
                <a href="/" class="preview-site-brand">
                    <i class="fa-solid fa-bolt" style="color: #ff4f00;"></i> __SITE_TITLE__
                </a>
                <nav class="preview-site-nav">
                    __PREVIEW_NAV_ITEMS__
                </nav>
            </header>

            <!-- Editable Canvas Sections -->
            <div id="elementor-canvas"></div>

            <!-- Bottom "Add New Section" Dashed Area -->
            <div class="elementor-add-section-area" id="elementor-bottom-dropzone">
                <div class="elementor-add-section-buttons">
                    <button class="btn-add-section-action btn-add-section-plus" id="btn-open-presets" title="Add Section">
                        <i class="fa-solid fa-plus"></i>
                    </button>
                    <button class="btn-add-section-action btn-add-section-folder" id="btn-open-kits" title="Add Template / Kit">
                        <i class="fa-solid fa-folder-open"></i>
                    </button>
                </div>

                <!-- Structure Presets -->
                <div class="elementor-select-preset" id="preset-selector">
                    <div class="preset-title">Select your Structure</div>
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

            <!-- Live Website Footer Preview -->
            <footer class="preview-site-footer">
                <p>&copy; 2026 __SITE_TITLE__. Built in Native Rust with Zero JavaScript Overhead.</p>
            </footer>
        </div>
    </main>

</div>

<!-- ============================================================== -->
<!-- STRUCTURE / LAYERS NAVIGATOR PANEL (#elementor-navigator)      -->
<!-- ============================================================== -->
<div id="elementor-navigator">
    <div class="navigator-header" id="navigator-header">
        <div class="navigator-header-title">
            <i class="fa-solid fa-layer-group" style="color: var(--e-color-accent);"></i>
            <span>Structure</span>
            <span class="navigator-count-badge" id="nav-element-count">0</span>
        </div>
        <div class="navigator-header-actions">
            <button class="nav-btn" id="nav-btn-expand-all" title="Expand / Collapse All">
                <i class="fa-solid fa-arrows-up-down"></i>
            </button>
            <button class="nav-btn" id="btn-close-navigator" title="Close Structure Panel">
                <i class="fa-solid fa-xmark"></i>
            </button>
        </div>
    </div>
    <div class="navigator-filter-bar">
        <input type="search" id="nav-search-input" placeholder="Filter layers...">
    </div>
    <div class="navigator-tree" id="navigator-tree"></div>
</div>

<!-- ============================================================== -->
<!-- EXTENSIVE ICON LIBRARY MODAL (250+ VECTOR ICONS, ZERO EMOJIS)  -->
<!-- ============================================================== -->
<div class="elementor-modal-overlay" id="icon-library-modal">
    <div class="elementor-modal icon-modal-layout">
        <div class="elementor-modal-header">
            <div class="elementor-modal-title">
                <i class="fa-solid fa-icons"></i> Icon Library
            </div>
            <button class="elementor-modal-close" id="btn-close-icon-library">&times;</button>
        </div>
        <div class="elementor-modal-body">
            <div class="icon-filter-bar">
                <div class="icon-category-tabs" id="icon-cat-tabs">
                    <button class="icon-tab active" data-cat="all">All Icons</button>
                    <button class="icon-tab" data-cat="tech">Tech & Web</button>
                    <button class="icon-tab" data-cat="business">Business</button>
                    <button class="icon-tab" data-cat="social">Social & Comms</button>
                    <button class="icon-tab" data-cat="arrows">Arrows & UI</button>
                    <button class="icon-tab" data-cat="brands">Brands</button>
                </div>
                <input type="search" class="icon-search-input" id="icon-search" placeholder="Filter by name...">
            </div>
            <div class="icon-grid" id="icon-grid"></div>
        </div>
        <div class="elementor-modal-footer">
            <button class="elementor-group-btn" style="padding: 0 16px; border-radius: 4px;" id="btn-cancel-icon">Cancel</button>
            <button class="btn-icon-library" style="flex: none; padding: 0 24px; background: var(--e-color-accent); border-color: var(--e-color-accent);" id="btn-insert-icon">
                <i class="fa-solid fa-check"></i> Insert Icon
            </button>
        </div>
    </div>
</div>

<!-- ============================================================== -->
<!-- PAGE SETTINGS MODAL                                            -->
<!-- ============================================================== -->
<div class="elementor-modal-overlay" id="settings-modal">
    <div class="elementor-modal" style="width: 520px; max-width: 90vw;">
        <div class="elementor-modal-header">
            <div class="elementor-modal-title">
                <i class="fa-solid fa-gear"></i> Page Settings
            </div>
            <button class="elementor-modal-close" id="btn-close-settings">&times;</button>
        </div>
        <div class="elementor-modal-body" style="display: flex; flex-direction: column; gap: 14px;">
            <div class="elementor-control-row">
                <label class="elementor-control-label">Page Title</label>
                <input type="text" class="elementor-input" id="set-page-title" value="__POST_TITLE__">
            </div>
            <div class="elementor-control-row">
                <label class="elementor-control-label">Status</label>
                <select class="elementor-select" id="set-page-status">
                    <option value="publish" selected>Published</option>
                    <option value="draft">Draft</option>
                    <option value="private">Private</option>
                </select>
            </div>
            <div class="elementor-control-row">
                <label class="elementor-control-label">Page Layout Template</label>
                <select class="elementor-select" id="set-page-template">
                    <option value="default" selected>Elementor Full Width (Header + Footer)</option>
                    <option value="canvas">Elementor Canvas (Blank / No Header / No Footer)</option>
                </select>
            </div>
            <div class="elementor-control-row">
                <label class="elementor-control-label">Canvas Background</label>
                <div class="color-picker-control">
                    <input type="color" class="color-swatch" id="set-canvas-bg-swatch" value="#ffffff">
                    <input type="text" class="elementor-input" id="set-canvas-bg-text" value="#ffffff">
                </div>
            </div>
        </div>
        <div class="elementor-modal-footer">
            <button class="btn-icon-library" style="flex: none; padding: 0 20px; background: var(--e-color-accent); border-color: var(--e-color-accent);" id="btn-save-settings">
                Apply Settings
            </button>
        </div>
    </div>
</div>

<!-- ============================================================== -->
<!-- KEYBOARD SHORTCUTS MODAL                                       -->
<!-- ============================================================== -->
<div class="elementor-modal-overlay" id="modal-shortcuts">
    <div class="elementor-modal" style="width: 640px; max-width: 90vw;">
        <div class="elementor-modal-header">
            <div class="elementor-modal-title">
                <i class="fa-solid fa-keyboard" style="color:var(--e-color-accent);"></i> Keyboard Shortcuts
            </div>
            <button class="elementor-modal-close" id="btn-close-shortcuts">&times;</button>
        </div>
        <div class="elementor-modal-body" style="padding: 16px;">
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 14px;">
                <div class="shortcut-group">
                    <div class="globals-section-header">Actions</div>
                    <div class="shortcut-row"><span>Save Layout</span><kbd>Ctrl + S</kbd></div>
                    <div class="shortcut-row"><span>Undo Mutation</span><kbd>Ctrl + Z</kbd></div>
                    <div class="shortcut-row"><span>Redo Mutation</span><kbd>Ctrl + Shift + Z</kbd></div>
                    <div class="shortcut-row"><span>Duplicate Element</span><kbd>Ctrl + D</kbd></div>
                    <div class="shortcut-row"><span>Delete Selected</span><kbd>Delete</kbd></div>
                </div>
                <div class="shortcut-group">
                    <div class="globals-section-header">Panels & Tools</div>
                    <div class="shortcut-row"><span>Spotlight Finder</span><kbd>Ctrl + E</kbd></div>
                    <div class="shortcut-row"><span>Structure / Layers</span><kbd>Ctrl + I</kbd></div>
                    <div class="shortcut-row"><span>Responsive Bar</span><kbd>Ctrl + Shift + M</kbd></div>
                    <div class="shortcut-row"><span>Open Settings Menu</span><kbd>Ctrl + P</kbd></div>
                    <div class="shortcut-row"><span>Close Modal</span><kbd>Esc</kbd></div>
                </div>
            </div>
        </div>
    </div>
</div>

<!-- ============================================================== -->
<!-- SPOTLIGHT FINDER COMMAND PALETTE (Ctrl+E)                      -->
<!-- ============================================================== -->
<div class="elementor-modal-overlay" id="modal-finder">
    <div class="elementor-modal" style="width: 580px; max-width: 90vw; background: #1e2124; border: 1px solid #3d424a; box-shadow: 0 16px 40px rgba(0,0,0,0.6);">
        <div style="display:flex;align-items:center;padding:12px 16px;border-bottom:1px solid #2d3035;gap:12px;">
            <i class="fa-solid fa-magnifying-glass" style="color:var(--e-color-accent);font-size:16px;"></i>
            <input type="text" id="finder-input" placeholder="Type a command or search widget..." style="flex:1;background:transparent;border:none;color:#fff;font-size:14px;outline:none;" autocomplete="off">
            <span style="font-size:11px;color:#818a91;background:#2a2d31;padding:2px 6px;border-radius:4px;">ESC to exit</span>
        </div>
        <div id="finder-results" style="max-height: 340px; overflow-y: auto; padding: 8px;"></div>
    </div>
</div>

<!-- ============================================================== -->
<!-- RAW HEADLESS JSON AST MODAL                                    -->
<!-- ============================================================== -->
<div class="elementor-modal-overlay" id="ast-modal">
    <div class="elementor-modal" style="width: 760px; max-width: 90vw; height: 560px;">
        <div class="elementor-modal-header">
            <div class="elementor-modal-title">
                <i class="fa-solid fa-code"></i> Canonical Neopress JSON AST
            </div>
            <button class="elementor-modal-close" id="btn-close-ast">&times;</button>
        </div>
        <div class="elementor-modal-body" style="display: flex; flex-direction: column;">
            <textarea class="elementor-textarea" id="ast-json-textarea" style="flex: 1; font-family: monospace; font-size: 11px; line-height: 1.5; background: #141618;" spellcheck="false"></textarea>
        </div>
        <div class="elementor-modal-footer">
            <button class="elementor-group-btn" style="padding: 0 16px; border-radius: 4px;" id="btn-copy-ast">
                <i class="fa-regular fa-copy"></i> Copy JSON
            </button>
            <button class="btn-icon-library" style="flex: none; padding: 0 20px; background: var(--e-color-accent); border-color: var(--e-color-accent);" id="btn-apply-ast">
                Import & Recompile
            </button>
        </div>
    </div>
</div>

<!-- Drop Insertion Indicator Line (Root-level for pixel-perfect fixed alignment) -->
<div id="drop-indicator" class="elementor-drop-indicator"></div>

<!-- TOAST -->
<div id="elementor-toast">
    <i class="fa-solid fa-circle-check" style="color: var(--e-color-success);"></i>
    <span id="toast-text">Changes saved successfully</span>
</div>

<script>__SCRIPTS__</script></body></html>"###;

    template
        .replace("__STYLES__", styles)
        .replace("__SCRIPTS__", scripts)
        .replace("__POST_ID__", &post_id.to_string())
        .replace("__POST_TITLE__", post_title)
        .replace("__POST_SLUG__", post_slug)
        .replace("__POST_TYPE__", post_type)
        .replace("__SITE_TITLE__", site_title)
        .replace("__PREVIEW_NAV_ITEMS__", preview_nav_html)
        .replace("__LAYOUT_JSON_RAW__", layout_json_raw)
}
