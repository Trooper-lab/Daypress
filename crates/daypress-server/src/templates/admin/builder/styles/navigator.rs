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
.nav-tree-item[draggable="true"] {
    cursor: grab;
}

.nav-tree-item[draggable="true"]:active {
    cursor: grabbing;
}

.nav-tree-item.is-dragging {
    opacity: 0.35;
    background: var(--day-surface-inset);
}

.nav-tree-item.day-drop-before {
    position: relative;
}

.nav-tree-item.day-drop-before::before {
    content: "";
    position: absolute;
    top: -1px;
    left: 4px;
    right: 4px;
    height: 2px;
    background: var(--day-signal);
    border-radius: 1px;
    z-index: 10;
    box-shadow: 0 0 4px var(--day-signal);
}

.nav-tree-item.day-drop-after {
    position: relative;
}

.nav-tree-item.day-drop-after::after {
    content: "";
    position: absolute;
    bottom: -1px;
    left: 4px;
    right: 4px;
    height: 2px;
    background: var(--day-signal);
    border-radius: 1px;
    z-index: 10;
    box-shadow: 0 0 4px var(--day-signal);
}

.nav-tree-item.day-drop-inside {
    background: rgba(37, 99, 235, 0.12) !important;
    outline: 1px dashed var(--day-signal);
    outline-offset: -1px;
}
"#;
