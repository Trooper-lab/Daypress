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
"#;
