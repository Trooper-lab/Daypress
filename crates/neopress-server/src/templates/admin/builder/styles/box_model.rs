//! Interactive Trapezoidal Box-Model Widget CSS (`styles/box_model.rs`).

pub const CSS_BOX_MODEL: &str = r#"
/* -------------------------------------------------------------
   INTERACTIVE TRAPEZOIDAL BOX-MODEL WIDGET
   ------------------------------------------------------------- */
.neo-box-model-container {
    padding: 12px;
    border-bottom: 1px solid var(--neo-rule);
    background: var(--neo-surface-inset);
}

.neo-box-model-widget {
    background: var(--neo-surface);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-md);
    padding: 10px;
    user-select: none;
    font-family: var(--neo-font-mono);
    position: relative;
}

.neo-box-margin-outer {
    position: relative;
    background: rgba(245, 158, 11, 0.06);
    border: 1px dashed rgba(245, 158, 11, 0.35);
    border-radius: var(--neo-radius-sm);
    padding: 24px 34px;
    display: flex;
    justify-content: center;
    align-items: center;
    transition: background 0.15s ease;
}

.neo-box-margin-outer:hover {
    background: rgba(245, 158, 11, 0.10);
}

.neo-box-label {
    position: absolute;
    top: 4px;
    left: 6px;
    font-size: 8.5px;
    font-weight: 700;
    color: var(--neo-muted);
    letter-spacing: 0.5px;
    text-transform: uppercase;
}

.neo-box-facet {
    position: absolute;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: ew-resize;
    font-size: 10.5px;
    font-weight: 600;
    color: var(--neo-ink);
    padding: 1px 4px;
    border-radius: 2px;
    transition: background 0.12s ease, color 0.12s ease;
}

.neo-box-facet:hover {
    color: var(--neo-signal-light);
    background: rgba(37, 99, 235, 0.18);
}

.neo-box-facet.facet-top    { top: 3px; left: 50%; transform: translateX(-50%); height: 18px; min-width: 32px; }
.neo-box-facet.facet-bottom { bottom: 3px; left: 50%; transform: translateX(-50%); height: 18px; min-width: 32px; }
.neo-box-facet.facet-left   { left: 4px; top: 50%; transform: translateY(-50%); width: 28px; height: 18px; }
.neo-box-facet.facet-right  { right: 4px; top: 50%; transform: translateY(-50%); width: 28px; height: 18px; }

.neo-box-padding-inner {
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

.neo-box-padding-inner:hover {
    background: rgba(37, 99, 235, 0.12);
}

.neo-box-content-core {
    background: var(--neo-chassis);
    border: 1px solid var(--neo-rule);
    border-radius: 2px;
    padding: 6px 12px;
    font-size: 10px;
    color: var(--neo-muted);
    white-space: nowrap;
    text-align: center;
    font-variant-numeric: tabular-nums;
}

.neo-box-model-inline-input {
    width: 36px;
    height: 18px;
    font-size: 11px;
    font-family: var(--neo-font-mono);
    text-align: center;
    background: var(--neo-surface);
    color: var(--neo-ink);
    border: 1px solid var(--neo-signal);
    border-radius: 2px;
    outline: none;
    padding: 0;
}

/* Box-Model Widget Footer Tools */
.neo-box-tools {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-top: 8px;
    padding-top: 6px;
    border-top: 1px solid var(--neo-rule-subtle);
}

.neo-unit-toggle {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 10.5px;
    color: var(--neo-muted);
}

.btn-unit-cycle {
    background: var(--neo-surface-inset);
    border: 1px solid var(--neo-rule);
    color: var(--neo-ink);
    font-family: var(--neo-font-mono);
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 2px;
    cursor: pointer;
}

.btn-unit-cycle:hover {
    border-color: var(--neo-signal);
}

.btn-link-quads {
    background: transparent;
    border: 1px solid var(--neo-rule);
    color: var(--neo-muted);
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
    color: var(--neo-ink);
    border-color: var(--neo-rule-focus);
}

.btn-link-quads.is-linked {
    background: var(--neo-surface-active);
    color: var(--neo-signal-light);
    border-color: var(--neo-signal);
}
"#;
