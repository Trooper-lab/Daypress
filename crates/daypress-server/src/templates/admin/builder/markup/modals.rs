//! Modals and Dialogs markup (`markup/modals.rs`).

pub fn render_modals() -> &'static str {
    r###"<!-- Command Palette Spotlight (Cmd+K / Ctrl+K) -->
<div class="daystudio-modal-overlay" id="modal-finder">
    <div class="daystudio-modal modal-spotlight">
        <div class="spotlight-header">
            <i class="fa-solid fa-magnifying-glass spotlight-icon"></i>
            <input type="text" id="finder-input" placeholder="Type a command, block name, or shortcut..." autocomplete="off">
            <span class="spotlight-badge">ESC to close</span>
        </div>
        <div id="finder-results" class="spotlight-results"></div>
    </div>
</div>

<!-- 250+ Vector Icon Library Modal -->
<div class="daystudio-modal-overlay" id="icon-library-modal">
    <div class="daystudio-modal icon-modal-layout">
        <div class="daystudio-modal-header">
            <div class="modal-title">
                <i class="fa-solid fa-icons"></i>
                <span>Vector Icon Library</span>
            </div>
            <button class="modal-close-btn" id="btn-close-icon-library">&times;</button>
        </div>
        <div class="daystudio-modal-body">
            <div class="icon-filter-bar">
                <div class="icon-category-tabs" id="icon-cat-tabs">
                    <button class="icon-tab active" data-cat="all">All Icons</button>
                    <button class="icon-tab" data-cat="tech">Tech & Web</button>
                    <button class="icon-tab" data-cat="business">Business</button>
                    <button class="icon-tab" data-cat="social">Social & Comms</button>
                    <button class="icon-tab" data-cat="arrows">Arrows & UI</button>
                    <button class="icon-tab" data-cat="brands">Brands</button>
                </div>
                <input type="search" class="cad-input icon-search-input" id="icon-search" placeholder="Filter icons...">
            </div>
            <div class="icon-grid" id="icon-grid"></div>
        </div>
        <div class="daystudio-modal-footer">
            <button class="cad-btn secondary" id="btn-cancel-icon">Cancel</button>
            <button class="cad-btn primary" id="btn-insert-icon">
                <i class="fa-solid fa-check"></i>
                <span>Insert Icon</span>
            </button>
        </div>
    </div>
</div>

<!-- Page & SEO Settings Modal -->
<div class="daystudio-modal-overlay" id="settings-modal">
    <div class="daystudio-modal" style="width: 520px; max-width: 90vw;">
        <div class="daystudio-modal-header">
            <div class="modal-title">
                <i class="fa-solid fa-gear"></i>
                <span>Page & Document Settings</span>
            </div>
            <button class="modal-close-btn" id="btn-close-settings">&times;</button>
        </div>
        <div class="daystudio-modal-body" style="display: flex; flex-direction: column; gap: 14px;">
            <div class="cad-form-row">
                <label class="cad-label">Page Title</label>
                <input type="text" class="cad-input" id="set-page-title" value="__POST_TITLE__">
            </div>
            <div class="cad-form-row">
                <label class="cad-label">Status</label>
                <select class="cad-select" id="set-page-status">
                    <option value="publish" selected>Published</option>
                    <option value="draft">Draft</option>
                    <option value="private">Private</option>
                </select>
            </div>
            <div class="cad-form-row">
                <label class="cad-label">Layout Template</label>
                <select class="cad-select" id="set-page-template">
                    <option value="default" selected>Full Width (Theme Header + Footer)</option>
                    <option value="canvas">Blank Canvas (No Header / No Footer)</option>
                </select>
            </div>
            <div class="cad-form-row">
                <label class="cad-label">Canvas Background</label>
                <div class="color-picker-control">
                    <input type="color" class="color-swatch-box" id="set-canvas-bg-swatch" value="#ffffff">
                    <input type="text" class="cad-input font-mono" id="set-canvas-bg-text" value="#ffffff">
                </div>
            </div>
        </div>
        <div class="daystudio-modal-footer">
            <button class="cad-btn secondary" id="btn-cancel-settings">Cancel</button>
            <button class="cad-btn primary" id="btn-save-settings">Apply Settings</button>
        </div>
    </div>
</div>

<!-- Theme Template Display Conditions Modal -->
<div class="daystudio-modal-overlay" id="modal-conditions">
    <div class="daystudio-modal" style="width: 620px; max-width: 92vw;">
        <div class="daystudio-modal-header">
            <div class="modal-title">
                <i class="fa-solid fa-shapes"></i>
                <span id="lbl-conditions-title">Display Conditions</span>
            </div>
            <button class="modal-close-btn" id="btn-close-conditions">&times;</button>
        </div>
        <div class="daystudio-modal-body">
            <p class="conditions-hint">
                Set priority-evaluated inclusion and exclusion rules for this template in public SSR.
                Specific page/post IDs override broad rules; exclusions override inclusions at equal specificity.
            </p>
            <div id="conditions-list-container" class="conditions-list"></div>
            <button class="cad-btn secondary" id="btn-add-condition-rule" style="margin-top: 12px;">
                <i class="fa-solid fa-plus"></i> Add Condition Rule
            </button>
        </div>
        <div class="daystudio-modal-footer">
            <button class="cad-btn secondary" id="btn-cancel-conditions">Cancel</button>
            <button class="cad-btn primary" id="btn-save-conditions">
                <i class="fa-solid fa-check"></i> Save Conditions
            </button>
        </div>
    </div>
</div>

<!-- Keyboard Shortcuts CAD Cheatsheet -->
<div class="daystudio-modal-overlay" id="modal-shortcuts">
    <div class="daystudio-modal" style="width: 640px; max-width: 90vw;">
        <div class="daystudio-modal-header">
            <div class="modal-title">
                <i class="fa-solid fa-keyboard"></i>
                <span>Keyboard Shortcuts Cheatsheet</span>
            </div>
            <button class="modal-close-btn" id="btn-close-shortcuts">&times;</button>
        </div>
        <div class="daystudio-modal-body" style="padding: 16px;">
            <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 16px;">
                <div class="shortcut-group">
                    <div class="shortcut-group-header">Studio Actions</div>
                    <div class="shortcut-row"><span>Save to SQLite</span><kbd>Ctrl + S</kbd></div>
                    <div class="shortcut-row"><span>Undo Mutation</span><kbd>Ctrl + Z</kbd></div>
                    <div class="shortcut-row"><span>Redo Mutation</span><kbd>Ctrl + Shift + Z</kbd></div>
                    <div class="shortcut-row"><span>Duplicate Element</span><kbd>Ctrl + D</kbd></div>
                    <div class="shortcut-row"><span>Delete Selected</span><kbd>Del / Backspace</kbd></div>
                    <div class="shortcut-row"><span>Toggle X-Ray Mode</span><kbd>Ctrl + Shift + X</kbd></div>
                </div>
                <div class="shortcut-group">
                    <div class="shortcut-group-header">Panels & Navigation</div>
                    <div class="shortcut-row"><span>Command Palette</span><kbd>Cmd + K / Ctrl + K</kbd></div>
                    <div class="shortcut-row"><span>Add Elements Drawer</span><kbd>Q or /</kbd></div>
                    <div class="shortcut-row"><span>Navigator Tree</span><kbd>Ctrl + I</kbd></div>
                    <div class="shortcut-row"><span>Theme Templates</span><kbd>Ctrl + T</kbd></div>
                    <div class="shortcut-row"><span>Design Tokens</span><kbd>Ctrl + G</kbd></div>
                    <div class="shortcut-row"><span>Audit History</span><kbd>Ctrl + H</kbd></div>
                </div>
            </div>
        </div>
    </div>
</div>

<!-- Canonical Daypress JSON AST Modal -->
<div class="daystudio-modal-overlay" id="ast-modal">
    <div class="daystudio-modal" style="width: 780px; max-width: 90vw; height: 580px;">
        <div class="daystudio-modal-header">
            <div class="modal-title">
                <i class="fa-solid fa-code"></i>
                <span>Canonical Daypress Layout JSON AST</span>
            </div>
            <button class="modal-close-btn" id="btn-close-ast">&times;</button>
        </div>
        <div class="daystudio-modal-body" style="display: flex; flex-direction: column;">
            <textarea class="cad-textarea font-mono" id="ast-json-textarea" style="flex: 1;" spellcheck="false"></textarea>
        </div>
        <div class="daystudio-modal-footer">
            <button class="cad-btn secondary" id="btn-copy-ast">
                <i class="fa-regular fa-copy"></i> Copy JSON
            </button>
            <button class="cad-btn primary" id="btn-apply-ast">
                <i class="fa-solid fa-arrows-rotate"></i> Import & Recompile
            </button>
        </div>
    </div>
</div>

<!-- Create Theme Template Modal (Non-Blocking 2026 CAD Dialog) -->
<div class="daystudio-modal-overlay" id="modal-create-template">
    <div class="daystudio-modal" style="width: 540px; max-width: 92vw;">
        <div class="daystudio-modal-header">
            <div class="modal-title">
                <i class="fa-solid fa-shapes" style="color: var(--day-signal);"></i>
                <span>Create Theme Template</span>
            </div>
            <button class="modal-close-btn" id="btn-close-create-template">&times;</button>
        </div>
        <div class="daystudio-modal-body" style="display: flex; flex-direction: column; gap: 14px;">
            <div id="create-template-error" class="cad-form-error" style="display: none;"></div>
            
            <div class="cad-form-row">
                <label class="cad-label" for="input-template-title">
                    Template Name <span style="color: var(--day-crimson); font-weight: 700;">*</span>
                </label>
                <input type="text" class="cad-input font-mono" id="input-template-title" placeholder="e.g. Custom Header" value="Custom Header" autocomplete="off" required>
            </div>
            
            <div class="cad-form-row">
                <label class="cad-label" for="select-template-type">Template Type</label>
                <select class="cad-select font-mono" id="select-template-type">
                    <option value="header" selected>Header (Sticky, Transparent, Nav)</option>
                    <option value="footer">Footer (Multi-Column, Social, Copyright)</option>
                    <option value="single_post">Single (Blog Posts & Custom Post Types)</option>
                    <option value="archive">Archive (Category, Tag, Blog Index)</option>
                    <option value="404_page">NotFound (Error 404 Page Layout)</option>
                </select>
            </div>
            
            <div class="cad-form-row">
                <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 4px;">
                    <label class="cad-label" for="select-starter-template" style="margin-bottom: 0;">Starter Foundation</label>
                    <span style="font-size: 10px; color: var(--day-muted); font-family: var(--day-font-mono);">1-Click Ready</span>
                </div>
                <select class="cad-select font-mono" id="select-starter-template">
                    <option value="blank" selected>Blank Canvas (Clean Root AST)</option>
                </select>
            </div>
            
            <div class="starter-chips-container" id="starter-chips-list">
                <!-- Dynamically populated starter cards -->
            </div>
        </div>
        <div class="daystudio-modal-footer">
            <button class="cad-btn secondary" id="btn-cancel-create-template">Cancel</button>
            <button class="cad-btn primary" id="btn-submit-create-template">
                <i class="fa-solid fa-plus"></i>
                <span>Create Template</span>
            </button>
        </div>
    </div>
</div>

<!-- Delete Template Confirmation Modal (Non-Blocking CAD Dialog) -->
<div class="daystudio-modal-overlay" id="modal-confirm-delete">
    <div class="daystudio-modal" style="width: 440px; max-width: 90vw;">
        <div class="daystudio-modal-header">
            <div class="modal-title" style="color: var(--day-crimson);">
                <i class="fa-solid fa-triangle-exclamation"></i>
                <span>Delete Theme Template</span>
            </div>
            <button class="modal-close-btn" id="btn-close-confirm-delete">&times;</button>
        </div>
        <div class="daystudio-modal-body">
            <p style="font-size: 12.5px; color: var(--day-ink); margin: 0 0 8px 0;" id="lbl-delete-confirm-msg">
                Permanently delete this template from SQLite?
            </p>
            <p style="font-size: 11px; color: var(--day-muted); margin: 0;">
                All associated public SSR display conditions will also be removed. This action cannot be undone.
            </p>
        </div>
        <div class="daystudio-modal-footer">
            <button class="cad-btn secondary" id="btn-cancel-confirm-delete">Cancel</button>
            <button class="cad-btn primary" id="btn-confirm-delete-action" style="background: var(--day-crimson); border-color: var(--day-crimson);">
                <i class="fa-regular fa-trash-can"></i>
                <span>Delete Template</span>
            </button>
        </div>
    </div>
</div>

<!-- Toast Notification -->
<div id="elementor-toast" class="daystudio-toast">
    <i class="fa-solid fa-circle-check toast-icon"></i>
    <span id="toast-text">Changes saved to SQLite</span>
</div>"###
}
