//! Interactive Trapezoidal Box-Model Widget CSS (`styles/box_model.rs`).

pub const CSS_BOX_MODEL: &str = r#"
/* -------------------------------------------------------------
   INTERACTIVE TRAPEZOIDAL BOX-MODEL WIDGET
   ------------------------------------------------------------- */
.day-box-model-container {
    padding: 12px;
    border-bottom: 1px solid var(--day-rule);
    background: var(--day-surface-inset);
}

.day-box-model-widget {
    background: var(--day-surface);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-md);
    padding: 10px;
    user-select: none;
    font-family: var(--day-font-mono);
    position: relative;
}

.day-box-margin-outer {
    position: relative;
    background: rgba(245, 158, 11, 0.06);
    border: 1px dashed rgba(245, 158, 11, 0.35);
    border-radius: var(--day-radius-sm);
    padding: 24px 34px;
    display: flex;
    justify-content: center;
    align-items: center;
    transition: background 0.15s ease;
}

.day-box-margin-outer:hover {
    background: rgba(245, 158, 11, 0.10);
}

.day-box-label {
    position: absolute;
    top: 4px;
    left: 6px;
    font-size: 8.5px;
    font-weight: 700;
    color: var(--day-muted);
    letter-spacing: 0.5px;
    text-transform: uppercase;
}

.day-box-facet {
    position: absolute;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: ew-resize;
    font-size: 10.5px;
    font-weight: 600;
    color: var(--day-ink);
    padding: 1px 4px;
    border-radius: 2px;
    transition: background 0.12s ease, color 0.12s ease;
}

.day-box-facet:hover {
    color: var(--day-signal-light);
    background: rgba(37, 99, 235, 0.18);
}

.day-box-facet.facet-top    { top: 3px; left: 50%; transform: translateX(-50%); height: 18px; min-width: 32px; }
.day-box-facet.facet-bottom { bottom: 3px; left: 50%; transform: translateX(-50%); height: 18px; min-width: 32px; }
.day-box-facet.facet-left   { left: 4px; top: 50%; transform: translateY(-50%); width: 28px; height: 18px; }
.day-box-facet.facet-right  { right: 4px; top: 50%; transform: translateY(-50%); width: 28px; height: 18px; }

.day-box-padding-inner {
    position: relative;
    background: rgba(37, 99, 235, 0.07);
    border: 1px solid rgba(37, 99, 235, 0.35);
    border-radius: 2px;
    padding: 20px 30px;
    display: flex;
    justify-content: center;
    align-items: center;
    transition: background 0.15s ease;
}

.day-box-padding-inner:hover {
    background: rgba(37, 99, 235, 0.12);
}

.day-box-content-core {
    background: var(--day-chassis);
    border: 1px solid var(--day-rule);
    border-radius: 2px;
    padding: 6px 12px;
    font-size: 10px;
    color: var(--day-muted);
    white-space: nowrap;
    text-align: center;
    font-variant-numeric: tabular-nums;
}

.day-box-model-inline-input {
    width: 36px;
    height: 18px;
    font-size: 11px;
    font-family: var(--day-font-mono);
    text-align: center;
    background: var(--day-surface);
    color: var(--day-ink);
    border: 1px solid var(--day-signal);
    border-radius: 2px;
    outline: none;
    padding: 0;
}

/* Box-Model Widget Footer Tools */
.day-box-tools {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 8px;
    padding-top: 6px;
    border-top: 1px solid var(--day-rule-subtle);
}

.day-unit-toggle {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 10.5px;
    color: var(--day-muted);
}

.btn-unit-cycle {
    background: var(--day-surface-inset);
    border: 1px solid var(--day-rule);
    color: var(--day-ink);
    font-family: var(--day-font-mono);
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 2px;
    cursor: pointer;
}

.btn-unit-cycle:hover {
    border-color: var(--day-signal);
}

.btn-link-quads {
    background: transparent;
    border: 1px solid var(--day-rule);
    color: var(--day-muted);
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 2px;
    cursor: pointer;
    font-size: 10px;
    transition: all 0.12s ease;
}

.btn-link-quads:hover {
    color: var(--day-ink);
    border-color: var(--day-rule-focus);
}

.btn-link-quads.is-linked {
    background: var(--day-surface-active);
    color: var(--day-signal-light);
    border-color: var(--day-signal);
}
"#;
