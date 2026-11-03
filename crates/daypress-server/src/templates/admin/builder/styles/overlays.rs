//! Selection chrome, live margin/padding box-model overlays, drop indicators and X-Ray mode (`styles/overlays.rs`).

pub const CSS_OVERLAYS: &str = r#"
/* -------------------------------------------------------------
   ON-CANVAS SELECTION CHROME & ACTION CHIPS
   ------------------------------------------------------------- */
#daystudio-selection-chrome {
    position: absolute;
    pointer-events: none;
    z-index: var(--day-z-overlays);
    transition: all 0.08s ease-out;
}

.selection-outline {
    position: absolute;
    inset: 0;
    border: 2px solid var(--day-signal);
    border-radius: 1px;
    box-shadow: 0 0 0 1px rgba(13, 15, 18, 0.6), 0 0 10px var(--day-signal-tint);
    pointer-events: none;
}

/* Bidirectional Navigator <-> Canvas Hover & Click Pulse Highlight */
#daystudio-canvas .dp-section {
    outline: 1px dashed rgba(230, 184, 74, 0.35);
    outline-offset: -1px;
    min-height: 48px;
}

#daystudio-canvas .dp-container {
    outline: 1px dashed rgba(147, 161, 255, 0.38);
    outline-offset: -2px;
    min-height: 36px;
}

#daystudio-canvas .dp-column {
    outline: 1px dashed rgba(94, 201, 152, 0.35);
    outline-offset: -2px;
    min-height: 32px;
}

.daystudio-node.is-hovered,
.daystudio-node.is-nav-hovered {
    outline: 2px dashed var(--day-token) !important;
    outline-offset: -1px !important;
}

.daystudio-node.node-section.is-hovered,
.daystudio-node.node-section.is-selected {
    outline: 2px solid var(--day-token) !important;
    outline-offset: -2px !important;
}

.daystudio-node.node-container.is-hovered,
.daystudio-node.node-container.is-selected {
    outline: 2px solid var(--day-signal) !important;
    outline-offset: -2px !important;
}

.daystudio-node.node-column.is-hovered,
.daystudio-node.node-column.is-selected {
    outline: 2px solid var(--day-emerald) !important;
    outline-offset: -2px !important;
}

#daystudio-selection-chrome[data-sel-type="section"] .selection-outline {
    border-color: var(--day-token);
}
#daystudio-selection-chrome[data-sel-type="section"] .selection-badge {
    background: var(--day-token);
    color: #111214;
}
#daystudio-selection-chrome[data-sel-type="section"] .badge-kind,
#daystudio-selection-chrome[data-sel-type="section"] .badge-id,
#daystudio-selection-chrome[data-sel-type="section"] .badge-action-btn {
    color: #111214;
}

#daystudio-selection-chrome[data-sel-type="column"] .selection-outline {
    border-color: var(--day-emerald);
}
#daystudio-selection-chrome[data-sel-type="column"] .selection-badge {
    background: var(--day-emerald);
    color: #111214;
}

#daystudio-selection-chrome.badge-inside .selection-badge {
    top: 2px;
    left: 2px;
    border-radius: var(--day-radius-sm);
}

@keyframes dayNodePulse {
    0%   { box-shadow: 0 0 0 0 rgba(230, 184, 74, 0.65); }
    70%  { box-shadow: 0 0 0 10px rgba(230, 184, 74, 0); }
    100% { box-shadow: 0 0 0 0 rgba(230, 184, 74, 0); }
}

.daystudio-node.is-flash-highlight {
    animation: dayNodePulse 0.75s cubic-bezier(0.16, 1, 0.3, 1);
}

/* Dynamic Field Binding Visual Indicator & Live Preview Badge */
.daystudio-node.is-dynamic-bound {
    position: relative;
    outline: 1.5px dashed rgba(230, 184, 74, 0.75);
    outline-offset: 2px;
    background-image: linear-gradient(90deg, rgba(230, 184, 74, 0.06), transparent 60%);
}

.daystudio-node.is-dynamic-bound::after {
    content: attr(data-dynamic-badge);
    position: absolute;
    top: -21px;
    right: 0;
    background: rgba(18, 16, 11, 0.94);
    color: #E6B84A;
    border: 1px solid rgba(230, 184, 74, 0.55);
    border-radius: 4px;
    padding: 1px 7px;
    font-family: var(--day-font-mono);
    font-size: 9.5px;
    font-weight: 600;
    line-height: 16px;
    letter-spacing: 0.01em;
    white-space: nowrap;
    pointer-events: none;
    z-index: 20;
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.35);
}

.selection-badge {
    position: absolute;
    top: -24px;
    left: -2px;
    height: 22px;
    background: var(--day-signal);
    color: var(--day-ink-contrast);
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 0 6px;
    border-radius: var(--day-radius-sm) var(--day-radius-sm) 0 0;
    font-family: var(--day-font-mono);
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
#daystudio-box-overlays {
    position: absolute;
    pointer-events: none;
    z-index: var(--day-z-overlays);
}

.overlay-margin {
    position: absolute;
    background: var(--day-token-tint);
    border: 1px dashed var(--day-token-border);
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
}

.overlay-padding {
    position: absolute;
    background: var(--day-signal-tint);
    border: 1px dashed var(--day-signal-border);
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
}

.overlay-chip {
    background: var(--day-chassis);
    border: 1px solid var(--day-rule);
    border-radius: 2px;
    padding: 1px 4px;
    font-family: var(--day-font-mono);
    font-size: 9px;
    color: var(--day-ink);
    font-variant-numeric: tabular-nums;
}

/* -------------------------------------------------------------
   DROP INSERTION INDICATOR GUIDE
   ------------------------------------------------------------- */
.daystudio-drop-guide {
    position: fixed;
    height: 3px;
    background: var(--day-signal);
    box-shadow: 0 0 10px var(--day-signal);
    pointer-events: none;
    z-index: 9999;
    display: none;
    border-radius: 2px;
}

/* -------------------------------------------------------------
   X-RAY / WIREFRAME OUTLINE MODE
   ------------------------------------------------------------- */
.daystudio-xray * {
    outline: 1px dashed var(--day-cyan) !important;
    outline-offset: -1px;
}

.daystudio-xray .dp-section,
.daystudio-xray .dp-container,
.daystudio-xray .dp-column,
.daystudio-xray .daystudio-column {
    background-image: repeating-linear-gradient(
        45deg,
        rgba(6, 182, 212, 0.04),
        rgba(6, 182, 212, 0.04) 10px,
        transparent 10px,
        transparent 20px
    ) !important;
}

.daystudio-xray img,
.daystudio-xray video {
    opacity: 0.3 !important;
}

/* -------------------------------------------------------------
   PSEUDO-STATE FORCED HOVER SIMULATION & BREAKPOINT VISIBILITY
   ------------------------------------------------------------- */
.daystudio-force-hover {
    outline: 2px dashed var(--day-token) !important;
    outline-offset: 1px;
}

.daystudio-hidden-bp {
    display: none !important;
}

.daystudio-xray .daystudio-hidden-bp {
    display: block !important;
    opacity: 0.35 !important;
    outline: 1px dashed #ef4444 !important;
}
"#;
