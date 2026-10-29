//! Layer-aware Inspector Control Kit CSS: grid rows, color wells, sliders, preset chips,
//! override/inherited label states and canvas state previews (`styles/inspector_controls.rs`).

pub const CSS_INSPECTOR_CONTROLS: &str = r#"
/* -------------------------------------------------------------
   CONTROL ROW GRIDS
   ------------------------------------------------------------- */
.cad-row.ns-row {
    display: grid;
    grid-template-columns: 64px minmax(0, 1fr);
    align-items: center;
    gap: 8px;
}
.cad-row.ns-pair {
    display: grid;
    grid-template-columns: 46px minmax(0, 1fr) 46px minmax(0, 1fr);
    align-items: center;
    gap: 6px;
}
.ns-ctl {
    display: flex;
    align-items: center;
    gap: 4px;
    min-width: 0;
}
.ns-ctl > .cad-input,
.ns-ctl > .cad-select,
.ns-ctl > .ns-unit-wrap,
.ns-ctl > .ns-font-control {
    width: 100%;
    min-width: 0;
}
/* Composite Length Input + Unit Selector */
.ns-unit-wrap {
    display: flex;
    align-items: center;
    height: var(--day-control-h);
    background: var(--day-surface-inset);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-sm);
    overflow: hidden;
    transition: border-color 0.15s ease;
}
.ns-unit-wrap:focus-within {
    border-color: var(--day-signal);
}
.ns-unit-wrap > .cad-input {
    border: none !important;
    background: transparent !important;
    flex: 1;
    min-width: 0;
    height: 100%;
    padding: 0 4px 0 7px;
    font-size: 11px;
}
.ns-unit-wrap > .ns-unit-badge {
    appearance: none;
    -webkit-appearance: none;
    border: none;
    border-left: 1px solid rgba(255, 255, 255, 0.06);
    background: rgba(255, 255, 255, 0.025);
    color: var(--day-muted);
    font-family: var(--day-font-mono);
    font-size: 9.5px;
    font-weight: 500;
    height: 100%;
    padding: 0 5px;
    min-width: 24px;
    text-align: center;
    cursor: pointer;
    outline: none;
    flex-shrink: 0;
    transition: color 0.12s ease, background 0.12s ease;
}
.ns-unit-wrap > .ns-unit-badge:hover,
.ns-unit-wrap > .ns-unit-badge:focus {
    color: var(--day-ink);
    background: rgba(37, 99, 235, 0.16);
}
.ns-unit-wrap > .ns-unit-badge.is-keyword {
    color: var(--day-token);
}
.ns-unit-wrap > .ns-unit-badge option {
    background: var(--day-surface);
    color: var(--day-ink);
    font-family: var(--day-font-mono);
    font-size: 11px;
}
.ns-chip.active {
    border-color: var(--day-signal);
    background: var(--day-signal-tint);
    color: var(--day-signal-light);
    font-weight: 600;
}
.cad-label.is-scrubbable {
    cursor: ew-resize;
}
.ns-ctl > .cad-segmented {
    flex: 1;
    min-width: 0;
}
.ns-ctl > .cad-segmented .seg-btn {
    flex: 1;
    min-width: 0;
    padding: 0 3px;
    white-space: nowrap;
    overflow: hidden;
}
.cad-input::placeholder {
    color: var(--day-muted-dark);
    opacity: 1;
}
.cad-select.is-unset {
    color: var(--day-muted-dark);
}
.seg-btn.ghost {
    color: var(--day-muted-light);
    box-shadow: inset 0 0 0 1px var(--day-rule-focus);
}

/* Labels: blue = set on this layer (click to reset), amber = inherited */
.cad-label {
    overflow: hidden;
    text-overflow: ellipsis;
    border-radius: 2px;
    transition: color 0.12s ease, background 0.12s ease;
}
.cad-label.has-override {
    color: var(--day-signal-light);
    background: var(--day-signal-tint);
    padding: 1px 4px;
    margin-left: -4px;
    cursor: pointer;
}
.cad-label.has-override:hover {
    background: var(--day-crimson-tint);
    color: var(--day-crimson);
    text-decoration: line-through;
}
.cad-label.is-inherited {
    color: var(--day-token);
}

/* Color well */
.ns-color {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
}
.ns-color .cad-input {
    flex: 1;
    min-width: 0;
}
.color-swatch-box.is-empty {
    opacity: 0.55;
}
.ns-clear {
    background: transparent;
    border: 1px solid var(--day-rule);
    color: var(--day-muted);
    width: 22px;
    height: 22px;
    border-radius: 2px;
    cursor: pointer;
    flex-shrink: 0;
    font-size: 12px;
    line-height: 1;
}
.ns-clear:hover {
    color: var(--day-crimson);
    border-color: var(--day-crimson);
}

/* Slider + numeric */
.ns-range {
    display: flex;
    align-items: center;
    gap: 6px;
    width: 100%;
}
.ns-range input[type="range"] {
    flex: 1;
    min-width: 0;
    accent-color: var(--day-signal);
}
.ns-range .cad-input {
    width: 54px;
    flex-shrink: 0;
}

.ns-textarea {
    width: 100%;
    min-height: 84px;
    font-size: 11px;
}

/* Preset chips */
.cad-row.ns-presets {
    display: flex;
    flex-wrap: wrap;
    justify-content: flex-start;
    gap: 4px;
    min-height: 0;
}
.ns-chip {
    background: var(--day-surface-inset);
    border: 1px solid var(--day-rule);
    color: var(--day-muted-light);
    font-size: 10px;
    height: 20px;
    min-width: 24px;
    padding: 0 6px;
    border-radius: 2px;
    cursor: pointer;
}
.ns-chip:hover {
    border-color: var(--day-signal);
    color: var(--day-ink);
}
.ns-hint {
    font-size: 10px;
    color: var(--day-muted-dark);
    line-height: 1.4;
}

/* Panel header dot when the panel has values on the current layer */
.panel-set-dot {
    display: none;
    width: 5px;
    height: 5px;
    border-radius: 50%;
    background: var(--day-signal-light);
    margin-left: 4px;
}
.cad-panel.has-values .panel-set-dot {
    display: inline-block;
}

/* Selector bar: layer info, reset, state dots */
.layer-info {
    font-size: 10px;
    color: var(--day-muted);
    letter-spacing: 0.02em;
}
.layer-info.is-override {
    color: var(--day-token);
    font-weight: 600;
}
.bp-dot[data-bp] {
    cursor: pointer;
}
.btn-layer-reset {
    background: transparent;
    border: 1px solid var(--day-rule);
    color: var(--day-muted);
    font-size: 9.5px;
    padding: 1px 6px;
    border-radius: 2px;
    cursor: pointer;
}
.btn-layer-reset:hover {
    color: var(--day-crimson);
    border-color: var(--day-crimson);
}
.state-dot {
    display: inline-block;
    width: 4px;
    height: 4px;
    border-radius: 50%;
    background: var(--day-token);
    margin-left: 3px;
    vertical-align: middle;
}

/* Box model value states */
.day-box-facet.is-set { color: var(--day-ink-bright); }
.day-box-facet.is-inherited { color: var(--day-token); }
.day-box-facet.is-computed { color: var(--day-muted-dark); font-weight: 500; }

/* -------------------------------------------------------------
   CANVAS PREVIEW STATES
   ------------------------------------------------------------- */
.daystudio-force-focus,
.daystudio-force-active {
    outline: 2px dashed var(--day-token) !important;
    outline-offset: 1px;
}
.daystudio-hidden-bp {
    opacity: 0.35 !important;
    background-image: repeating-linear-gradient(45deg, rgba(239, 68, 68, 0.18) 0 6px, transparent 6px 12px) !important;
    outline: 1px dashed var(--day-crimson) !important;
}
"#;
