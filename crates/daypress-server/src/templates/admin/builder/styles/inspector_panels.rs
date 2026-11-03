//! Collapsible CAD Style Panels, Control Rows, Inputs and Repeaters CSS (`styles/inspector_panels.rs`).

pub const CSS_INSPECTOR_PANELS: &str = r#"
/* -------------------------------------------------------------
   COLLAPSIBLE CAD STYLE PANELS
   ------------------------------------------------------------- */
.cad-panel {
    border-bottom: 1px solid var(--day-rule);
}

.cad-panel-header {
    height: 30px;
    background: var(--day-surface);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    cursor: pointer;
    user-select: none;
    transition: background 0.12s ease;
}

.cad-panel-header:hover {
    background: var(--day-surface-hover);
}

.cad-panel-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--day-ink);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    gap: 6px;
}

.cad-panel-chevron {
    font-size: 9px;
    color: var(--day-muted);
    transition: transform 0.15s ease;
}

.cad-panel.open .cad-panel-chevron {
    transform: rotate(90deg);
}

.cad-panel-body {
    display: none;
    padding: 10px 12px;
    flex-direction: column;
    gap: 8px;
    background: var(--day-surface);
}

.cad-panel.open .cad-panel-body {
    display: flex;
}

/* -------------------------------------------------------------
   HIGH-DENSITY CAD CONTROL ROWS (26px)
   ------------------------------------------------------------- */
.cad-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: var(--day-control-h);
    gap: 8px;
}

.cad-label {
    font-size: var(--day-text-sub);
    color: var(--day-muted);
    font-weight: 500;
    white-space: nowrap;
}

.cad-label.has-override {
    color: var(--day-signal-light);
}

.cad-label.has-token {
    color: var(--day-token);
}

.cad-input, .cad-select {
    height: var(--day-control-h);
    background: var(--day-surface-inset);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-sm);
    color: var(--day-ink);
    font-size: var(--day-text-sub);
    padding: 0 8px;
    outline: none;
    transition: border-color 0.15s ease;
}

.cad-input:focus, .cad-select:focus {
    border-color: var(--day-signal);
}

.cad-textarea {
    background: var(--day-surface-inset);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-sm);
    color: var(--day-ink);
    font-size: var(--day-text-sub);
    padding: 6px 8px;
    outline: none;
    resize: vertical;
    min-height: 54px;
    transition: border-color 0.15s ease;
}

.cad-textarea:focus {
    border-color: var(--day-signal);
}

.font-mono {
    font-family: var(--day-font-mono) !important;
}

/* Segmented Buttons Group */
.cad-segmented {
    display: flex;
    background: var(--day-surface-inset);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-sm);
    padding: 1px;
    gap: 1px;
}

.seg-btn {
    background: transparent;
    border: none;
    color: var(--day-muted);
    height: 22px;
    padding: 0 6px;
    font-size: 10.5px;
    border-radius: 2px;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.12s ease;
}

.seg-btn:hover {
    color: var(--day-ink);
    background: var(--day-surface-hover);
}

.seg-btn.active {
    background: var(--day-surface);
    color: var(--day-ink);
    font-weight: 600;
    box-shadow: 0 1px 2px rgba(0,0,0,0.3);
}

/* Scrubber Group with Unit */
.cad-scrubber-group {
    display: flex;
    align-items: center;
    background: var(--day-surface-inset);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-sm);
    height: var(--day-control-h);
    overflow: hidden;
}

.scrubber-val {
    width: 52px;
    height: 100%;
    border: none;
    background: transparent;
    color: var(--day-ink);
    font-family: var(--day-font-mono);
    font-size: 11px;
    text-align: right;
    padding: 0 4px;
    outline: none;
}

.unit-select {
    height: 100%;
    border: none;
    background: transparent;
    color: var(--day-muted);
    font-family: var(--day-font-mono);
    font-size: 10px;
    padding: 0 4px;
    outline: none;
    cursor: pointer;
}

/* Color Picker Well */
.color-picker-control {
    display: flex;
    align-items: center;
    gap: 6px;
}

.color-swatch-box {
    width: 22px;
    height: 22px;
    border: 1px solid var(--day-rule);
    border-radius: 2px;
    padding: 0;
    cursor: pointer;
    background: none;
}

/* 4-Corner Border Radius Matrix */
.quad-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 4px;
}

.quad-item {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
}

.quad-label {
    font-size: 8.5px;
    color: var(--day-muted-dark);
}

.quad-input {
    width: 100%;
    text-align: center;
}

/* Repeater Item Cards */
.repeater-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 6px;
}

.repeater-item {
    background: var(--day-surface-inset);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-sm);
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 6px;
}

.repeater-item-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.repeater-drag-handle {
    cursor: grab;
    color: var(--day-muted-dark);
    font-size: 10px;
}

.repeater-remove-btn {
    background: transparent;
    border: none;
    color: var(--day-muted);
    cursor: pointer;
    font-size: 10px;
}

.repeater-remove-btn:hover {
    color: var(--day-crimson);
}

/* Unified DayStudio CAD Buttons */
.cad-btn {
    display: inline-flex; align-items: center; justify-content: center; gap: 6px;
    height: 28px; padding: 0 10px; background: var(--day-surface-elevated);
    border: 1px solid var(--day-rule); border-radius: var(--day-radius-sm);
    color: var(--day-ink); font-family: var(--day-font-ui); font-size: 11px; font-weight: 600;
    cursor: pointer; transition: all 0.12s ease; white-space: nowrap; text-decoration: none;
}
.cad-btn:hover { background: var(--day-surface-hover); border-color: var(--day-muted-dark); color: var(--day-ink); }
.cad-btn.primary { background: var(--day-signal); border-color: var(--day-signal); color: #0A0B0C; }
.cad-btn.primary:hover { background: var(--day-signal-hover); border-color: var(--day-signal-hover); color: #0A0B0C; }
.cad-btn.secondary { background: var(--day-surface-inset); border-color: var(--day-rule); color: var(--day-ink); }
.cad-btn.secondary:hover { border-color: var(--day-signal); color: var(--day-signal); }
.cad-btn.danger { background: rgba(240, 108, 108, 0.08); border-color: rgba(240, 108, 108, 0.3); color: var(--day-crimson); }
.cad-btn.danger:hover { background: var(--day-crimson); border-color: var(--day-crimson); color: #0A0B0C; }
.cad-btn.sm { height: 24px; padding: 0 8px; font-size: 10.5px; }

/* Visual Icon Selector Module & Popover Modal */
.cad-icon-field-wrap { display: flex; align-items: center; gap: 4px; flex: 1; min-width: 0; }
.cad-icon-preview-btn {
    width: 26px; height: 26px; flex-shrink: 0;
    display: inline-flex; align-items: center; justify-content: center;
    background: var(--day-surface-inset); border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-sm); color: var(--day-signal); font-size: 12px;
    cursor: pointer; transition: all 0.12s ease;
}
.cad-icon-preview-btn:hover { border-color: var(--day-signal); background: var(--day-signal-subtle); }
.dp-icon-modal-backdrop {
    position: fixed; inset: 0; z-index: 99999; background: rgba(10, 11, 12, 0.76);
    backdrop-filter: blur(4px); display: flex; align-items: center; justify-content: center; padding: 20px;
}
.dp-icon-modal {
    width: 560px; max-width: 94vw; max-height: 82vh; background: var(--day-surface);
    border: 1px solid var(--day-rule); border-radius: var(--day-radius-lg);
    box-shadow: var(--day-shadow-float); display: flex; flex-direction: column;
    overflow: hidden; color: var(--day-ink);
}
.dp-icon-modal-cats {
    display: flex; gap: 4px; padding: 8px 14px;
    border-bottom: 1px solid var(--day-rule-subtle); background: var(--day-chassis); overflow-x: auto;
}
.dp-icon-cat-pill {
    padding: 4px 9px; border-radius: var(--day-radius-sm); border: 1px solid transparent;
    background: transparent; color: var(--day-muted); font-size: 10.5px; font-weight: 600;
    cursor: pointer; white-space: nowrap;
}
.dp-icon-cat-pill.active, .dp-icon-cat-pill:hover {
    background: var(--day-surface-elevated); border-color: var(--day-rule); color: var(--day-ink);
}
.dp-icon-grid {
    display: grid; grid-template-columns: repeat(auto-fill, minmax(78px, 1fr));
    gap: 6px; padding: 12px 14px; overflow-y: auto; max-height: 360px;
}
.dp-icon-tile {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    gap: 6px; padding: 10px 6px; background: var(--day-surface-inset);
    border: 1px solid var(--day-rule-subtle); border-radius: var(--day-radius-sm);
    color: var(--day-ink); cursor: pointer; transition: all 0.12s ease; text-align: center;
}
.dp-icon-tile i { font-size: 16px; color: var(--day-ink); transition: transform 0.12s ease, color 0.12s ease; }
.dp-icon-tile span { font-size: 9.5px; color: var(--day-muted); overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 100%; }
.dp-icon-tile:hover, .dp-icon-tile.active { border-color: var(--day-signal); background: var(--day-signal-subtle); }
.dp-icon-tile:hover i, .dp-icon-tile.active i { color: var(--day-signal); transform: scale(1.1); }
"#;
