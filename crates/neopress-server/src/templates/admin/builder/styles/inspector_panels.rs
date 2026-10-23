//! Collapsible CAD Style Panels, Control Rows, Inputs and Repeaters CSS (`styles/inspector_panels.rs`).

pub const CSS_INSPECTOR_PANELS: &str = r#"
/* -------------------------------------------------------------
   COLLAPSIBLE CAD STYLE PANELS
   ------------------------------------------------------------- */
.cad-panel {
    border-bottom: 1px solid var(--neo-rule);
}

.cad-panel-header {
    height: 30px;
    background: var(--neo-surface);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    cursor: pointer;
    user-select: none;
    transition: background 0.12s ease;
}

.cad-panel-header:hover {
    background: var(--neo-surface-hover);
}

.cad-panel-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--neo-ink);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    gap: 6px;
}

.cad-panel-chevron {
    font-size: 9px;
    color: var(--neo-muted);
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
    background: var(--neo-surface);
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
    min-height: var(--neo-control-h);
    gap: 8px;
}

.cad-label {
    font-size: var(--neo-text-sub);
    color: var(--neo-muted);
    font-weight: 500;
    white-space: nowrap;
}

.cad-label.has-override {
    color: var(--neo-signal-light);
}

.cad-label.has-token {
    color: var(--neo-token);
}

.cad-input, .cad-select {
    height: var(--neo-control-h);
    background: var(--neo-surface-inset);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-sm);
    color: var(--neo-ink);
    font-size: var(--neo-text-sub);
    padding: 0 8px;
    outline: none;
    transition: border-color 0.15s ease;
}

.cad-input:focus, .cad-select:focus {
    border-color: var(--neo-signal);
}

.cad-textarea {
    background: var(--neo-surface-inset);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-sm);
    color: var(--neo-ink);
    font-size: var(--neo-text-sub);
    padding: 6px 8px;
    outline: none;
    resize: vertical;
    min-height: 54px;
    transition: border-color 0.15s ease;
}

.cad-textarea:focus {
    border-color: var(--neo-signal);
}

.font-mono {
    font-family: var(--neo-font-mono) !important;
}

/* Segmented Buttons Group */
.cad-segmented {
    display: flex;
    background: var(--neo-surface-inset);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-sm);
    padding: 1px;
    gap: 1px;
}

.seg-btn {
    background: transparent;
    border: none;
    color: var(--neo-muted);
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
    color: var(--neo-ink);
    background: var(--neo-surface-hover);
}

.seg-btn.active {
    background: var(--neo-surface);
    color: var(--neo-ink);
    font-weight: 600;
    box-shadow: 0 1px 2px rgba(0,0,0,0.3);
}

/* Scrubber Group with Unit */
.cad-scrubber-group {
    display: flex;
    align-items: center;
    background: var(--neo-surface-inset);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-sm);
    height: var(--neo-control-h);
    overflow: hidden;
}

.scrubber-val {
    width: 52px;
    height: 100%;
    border: none;
    background: transparent;
    color: var(--neo-ink);
    font-family: var(--neo-font-mono);
    font-size: 11px;
    text-align: right;
    padding: 0 4px;
    outline: none;
}

.unit-select {
    height: 100%;
    border: none;
    background: transparent;
    color: var(--neo-muted);
    font-family: var(--neo-font-mono);
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
    border: 1px solid var(--neo-rule);
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
    color: var(--neo-muted-dark);
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
    background: var(--neo-surface-inset);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-sm);
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
    color: var(--neo-muted-dark);
    font-size: 10px;
}

.repeater-remove-btn {
    background: transparent;
    border: none;
    color: var(--neo-muted);
    cursor: pointer;
    font-size: 10px;
}

.repeater-remove-btn:hover {
    color: var(--neo-crimson);
}
"#;
