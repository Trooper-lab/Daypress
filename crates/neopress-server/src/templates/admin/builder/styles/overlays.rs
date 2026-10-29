//! Selection chrome, live margin/padding box-model overlays, drop indicators and X-Ray mode (`styles/overlays.rs`).

pub const CSS_OVERLAYS: &str = r#"
/* -------------------------------------------------------------
   ON-CANVAS SELECTION CHROME & ACTION CHIPS
   ------------------------------------------------------------- */
#neostudio-selection-chrome {
    position: absolute;
    pointer-events: none;
    z-index: var(--neo-z-overlays);
    transition: all 0.08s ease-out;
}

.selection-outline {
    position: absolute;
    inset: 0;
    border: 2px solid var(--neo-signal);
    border-radius: 1px;
    box-shadow: 0 0 0 1px rgba(13, 15, 18, 0.6), 0 0 10px rgba(37, 99, 235, 0.25);
    pointer-events: none;
}

.selection-badge {
    position: absolute;
    top: -24px;
    left: -2px;
    height: 22px;
    background: var(--neo-signal);
    color: #ffffff;
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 6px;
    border-radius: var(--neo-radius-sm) var(--neo-radius-sm) 0 0;
    font-family: var(--neo-font-mono);
    font-size: 10.5px;
    font-weight: 600;
    pointer-events: auto;
    user-select: none;
    box-shadow: 0 -2px 6px rgba(0, 0, 0, 0.3);
}

.badge-kind {
    color: #ffffff;
}

.badge-id {
    color: rgba(255, 255, 255, 0.7);
    font-size: 9.5px;
}

.badge-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    margin-left: 4px;
    border-left: 1px solid rgba(255, 255, 255, 0.25);
    padding-left: 4px;
}

.badge-action-btn {
    background: transparent;
    border: none;
    color: rgba(255, 255, 255, 0.85);
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    border-radius: 2px;
    font-size: 10px;
    transition: background 0.12s ease, color 0.12s ease;
}

.badge-action-btn:hover {
    background: rgba(255, 255, 255, 0.25);
    color: #ffffff;
}

/* -------------------------------------------------------------
   LIVE MARGIN & PADDING BOX-MODEL OVERLAYS
   ------------------------------------------------------------- */
#neostudio-box-overlays {
    position: absolute;
    pointer-events: none;
    z-index: var(--neo-z-overlays);
}

.overlay-margin {
    position: absolute;
    background: var(--neo-token-tint);
    border: 1px dashed var(--neo-token-border);
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
}

.overlay-padding {
    position: absolute;
    background: var(--neo-signal-tint);
    border: 1px dashed var(--neo-signal-border);
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
}

.overlay-chip {
    background: var(--neo-chassis);
    border: 1px solid var(--neo-rule);
    border-radius: 2px;
    padding: 1px 4px;
    font-family: var(--neo-font-mono);
    font-size: 9px;
    color: var(--neo-ink);
    font-variant-numeric: tabular-nums;
}

/* -------------------------------------------------------------
   DROP INSERTION INDICATOR GUIDE
   ------------------------------------------------------------- */
.neostudio-drop-guide {
    position: fixed;
    height: 3px;
    background: var(--neo-signal);
    box-shadow: 0 0 10px var(--neo-signal);
    pointer-events: none;
    z-index: 9999;
    display: none;
    border-radius: 2px;
}

/* -------------------------------------------------------------
   X-RAY / WIREFRAME OUTLINE MODE
   ------------------------------------------------------------- */
.neostudio-xray * {
    outline: 1px dashed var(--neo-cyan) !important;
    outline-offset: -1px;
}

.neostudio-xray .np-section,
.neostudio-xray .np-container,
.neostudio-xray .np-column,
.neostudio-xray .elementor-column {
    background-image: repeating-linear-gradient(
        45deg,
        rgba(6, 182, 212, 0.04),
        rgba(6, 182, 212, 0.04) 10px,
        transparent 10px,
        transparent 20px
    ) !important;
}

.neostudio-xray img,
.neostudio-xray video {
    opacity: 0.3 !important;
}

/* -------------------------------------------------------------
   PSEUDO-STATE FORCED HOVER SIMULATION
   ------------------------------------------------------------- */
.neostudio-force-hover {
    outline: 2px dashed var(--neo-token) !important;
    outline-offset: 1px;
}
"#;
