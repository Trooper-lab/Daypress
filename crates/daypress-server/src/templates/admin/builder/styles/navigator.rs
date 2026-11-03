//! Navigator Layers Tree and Theme Templates Drawer Cards CSS (`styles/navigator.rs`).

pub const CSS_NAVIGATOR: &str = r#"
/* -------------------------------------------------------------
   NAVIGATOR TREE DRAWER
   ------------------------------------------------------------- */
.nav-tree-item {
    height: 26px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 4px;
    border-radius: var(--day-radius-sm);
    cursor: pointer;
    user-select: none;
    position: relative;
    font-size: 11.5px;
    transition: background 0.1s ease;
}

.nav-tree-item:hover {
    background: var(--day-surface-hover);
}

.nav-tree-item.active {
    background: var(--day-surface-active);
    color: #ffffff;
}

.nav-tree-item.active::before {
    content: "";
    position: absolute;
    left: 0;
    top: 2px;
    bottom: 2px;
    width: 2px;
    background: var(--day-signal);
    border-radius: 1px;
}

.nav-tree-left {
    display: flex;
    align-items: center;
    gap: 4px;
    overflow: hidden;
    flex: 1;
}

.nav-indent {
    width: 14px;
    height: 100%;
    flex-shrink: 0;
    position: relative;
}

.nav-indent::after {
    content: "";
    position: absolute;
    left: 7px;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--day-rule-subtle);
}

.nav-chevron {
    width: 14px;
    height: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 9px;
    color: var(--day-muted);
    transition: transform 0.15s ease;
    flex-shrink: 0;
}

.nav-chevron.open {
    transform: rotate(90deg);
}

.nav-node-icon {
    font-size: 11px;
    color: var(--day-muted);
    flex-shrink: 0;
    width: 14px;
    text-align: center;
}

.nav-node-label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 11.5px;
}

.nav-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    opacity: 0;
    transition: opacity 0.12s ease;
}

.nav-tree-item:hover .nav-actions,
.nav-tree-item.is-locked .nav-actions,
.nav-tree-item.is-hidden .nav-actions {
    opacity: 1;
}

.nav-act-btn {
    background: transparent;
    border: none;
    color: var(--day-muted);
    width: 18px;
    height: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 10px;
    border-radius: 2px;
}

.nav-act-btn:hover {
    color: var(--day-ink);
    background: var(--day-surface-hover);
}

.nav-tree-item.is-locked .act-lock {
    color: var(--day-token);
}

.nav-tree-item.is-hidden {
    opacity: 0.45;
}

.nav-rename-input {
    background: var(--day-surface-inset);
    border: 1px solid var(--day-signal);
    border-radius: 2px;
    color: var(--day-ink);
    font-size: 11px;
    padding: 1px 4px;
    width: 110px;
    outline: none;
}

/* -------------------------------------------------------------
   THEME TEMPLATES DRAWER CARDS
   ------------------------------------------------------------- */
.template-card {
    background: var(--day-surface-inset);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-sm);
    padding: 10px;
    margin-bottom: 8px;
    transition: all 0.15s ease;
}

.template-card:hover {
    border-color: var(--day-rule-focus);
    background: var(--day-surface);
}

.template-card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 6px;
}

.template-card-title {
    font-size: 12px;
    font-weight: 600;
    color: var(--day-ink);
}

.template-condition-pill {
    font-size: 10px;
    background: var(--day-chassis);
    border: 1px solid var(--day-rule);
    color: var(--day-token);
    padding: 2px 6px;
    border-radius: 2px;
    display: inline-block;
    margin-top: 4px;
}

.template-card-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-top: 8px;
    border-top: 1px solid var(--day-rule-subtle);
    padding-top: 6px;
}

.template-act-btn {
    font-size: 10.5px;
    background: transparent;
    border: 1px solid var(--day-rule);
    color: var(--day-muted);
    padding: 2px 6px;
    border-radius: 2px;
    cursor: pointer;
}

.template-act-btn:hover {
    color: var(--day-ink);
    border-color: var(--day-signal);
}

/* Drag and Drop Reordering */
.nav-tree-item[draggable="true"] { cursor: grab; }
.nav-tree-item[draggable="true"]:active { cursor: grabbing; }
.nav-tree-item.is-dragging { opacity: 0.35; background: var(--day-surface-inset); }
.nav-tree-item.day-drop-before { position: relative; }
.nav-tree-item.day-drop-before::before {
    content: ""; position: absolute; top: -1px; left: 4px; right: 4px; height: 2px;
    background: var(--day-signal); border-radius: 1px; z-index: 10; box-shadow: 0 0 4px var(--day-signal);
}
.nav-tree-item.day-drop-after { position: relative; }
.nav-tree-item.day-drop-after::after {
    content: ""; position: absolute; bottom: -1px; left: 4px; right: 4px; height: 2px;
    background: var(--day-signal); border-radius: 1px; z-index: 10; box-shadow: 0 0 4px var(--day-signal);
}
.nav-tree-item.day-drop-inside {
    background: var(--day-signal-subtle) !important;
    outline: 1px dashed var(--day-signal); outline-offset: -1px;
}

/* Bidirectional Canvas -> Navigator Hover Highlight */
.nav-tree-item.is-canvas-hovered {
    background: var(--day-surface-hover);
    box-shadow: inset 0 0 0 1px var(--day-token-border);
}

/* Brand-Aligned Element Type Color Coding (Sections vs Containers vs Columns vs Widgets) */
.nav-type-pill {
    font-family: var(--day-font-mono);
    font-size: 8.5px;
    font-weight: 700;
    letter-spacing: 0.04em;
    padding: 1px 4px;
    border-radius: 2px;
    text-transform: uppercase;
    flex-shrink: 0;
    line-height: 1.2;
}

/* 1. Section: Warm Amber / Gold (--day-token) */
.nav-tree-item[data-node-cat="section"] .nav-node-icon { color: var(--day-token); }
.nav-tree-item[data-node-cat="section"] .nav-type-pill {
    background: var(--day-token-tint);
    color: var(--day-token);
    border: 1px solid var(--day-token-border);
}
.nav-tree-item[data-node-cat="section"] {
    border-left: 2px solid rgba(230, 184, 74, 0.55);
}

/* 2. Container / Grid / Flex: Opposite Cool Press Indigo (--day-signal) */
.nav-tree-item[data-node-cat="container"] .nav-node-icon { color: var(--day-signal); }
.nav-tree-item[data-node-cat="container"] .nav-type-pill {
    background: var(--day-signal-tint);
    color: var(--day-signal);
    border: 1px solid var(--day-signal-border);
}
.nav-tree-item[data-node-cat="container"] {
    border-left: 2px solid rgba(147, 161, 255, 0.55);
}

/* 3. Column: Sage Emerald (--day-emerald) */
.nav-tree-item[data-node-cat="column"] .nav-node-icon { color: var(--day-emerald); }
.nav-tree-item[data-node-cat="column"] .nav-type-pill {
    background: rgba(94, 201, 152, 0.14);
    color: var(--day-emerald);
    border: 1px solid rgba(94, 201, 152, 0.35);
}

/* 4. Dynamic & Symbols: Soft Violet (--day-violet) */
.nav-tree-item[data-node-cat="symbol"] .nav-node-icon { color: var(--day-violet); }
.nav-tree-item[data-node-cat="symbol"] .nav-type-pill {
    background: var(--day-violet-tint);
    color: var(--day-violet);
    border: 1px solid rgba(194, 201, 255, 0.35);
}

/* 5. Interactive / CTA Widgets: Warm Oxide Terracotta (--day-oxide) */
.nav-tree-item[data-node-cat="action"] .nav-node-icon { color: var(--day-oxide); }

/* 6. Media & Typography */
.nav-tree-item[data-node-cat="media"] .nav-node-icon { color: #7CC4FA; }
.nav-tree-item[data-node-cat="text"] .nav-node-icon { color: var(--day-ink); }

/* Right-Click CAD Context Menu */
.dp-nav-context-menu {
    position: fixed;
    z-index: 99999;
    min-width: 210px;
    background: var(--day-surface);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-md);
    box-shadow: var(--day-shadow-float);
    padding: 5px;
    display: flex;
    flex-direction: column;
    gap: 1px;
    font-family: var(--day-font-ui);
    user-select: none;
}
.dp-ctx-header {
    padding: 4px 8px 5px;
    font-size: 9.5px;
    font-weight: 700;
    color: var(--day-muted);
    font-family: var(--day-font-mono);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    border-bottom: 1px solid var(--day-rule-subtle);
    margin-bottom: 3px;
    display: flex;
    align-items: center;
    justify-content: space-between;
}
.dp-ctx-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 6px 8px;
    background: transparent;
    border: none;
    border-radius: var(--day-radius-sm);
    color: var(--day-ink);
    font-size: 11.5px;
    font-weight: 500;
    cursor: pointer;
    text-align: left;
    width: 100%;
}
.dp-ctx-item:hover:not(:disabled) {
    background: var(--day-surface-hover);
    color: var(--day-signal-light);
}
.dp-ctx-item:disabled {
    opacity: 0.38;
    cursor: not-allowed;
}
.dp-ctx-item.danger { color: var(--day-crimson); }
.dp-ctx-item.danger:hover:not(:disabled) {
    background: rgba(240, 108, 108, 0.14);
    color: var(--day-crimson);
}
.dp-ctx-left { display: inline-flex; align-items: center; gap: 8px; }
.dp-ctx-left i { width: 14px; text-align: center; font-size: 11px; color: var(--day-muted); }
.dp-ctx-item:hover:not(:disabled) .dp-ctx-left i { color: inherit; }
.dp-ctx-kbd {
    font-family: var(--day-font-mono);
    font-size: 9.5px;
    color: var(--day-muted-dark);
}
.dp-ctx-sep {
    height: 1px;
    background: var(--day-rule-subtle);
    margin: 3px 4px;
}
"#;
