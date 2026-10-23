//! Left Studio Rail (48px) and Add Elements Drawer (280px) CSS (`styles/drawers.rs`).

pub const CSS_DRAWERS: &str = r#"
/* -------------------------------------------------------------
   LEFT STUDIO RAIL (48px)
   ------------------------------------------------------------- */
#neostudio-rail {
    width: var(--neo-rail-w);
    min-width: var(--neo-rail-w);
    background: var(--neo-chassis);
    border-right: 1px solid var(--neo-rule);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    z-index: var(--neo-z-dock);
    user-select: none;
}

.rail-top, .rail-bottom {
    display: flex;
    flex-direction: column;
    align-items: center;
}

.rail-tab-btn {
    width: 48px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--neo-muted);
    border: none;
    background: transparent;
    cursor: pointer;
    position: relative;
    font-size: 14px;
    transition: all 0.15s ease;
}

.rail-tab-btn:hover {
    color: var(--neo-ink);
    background: var(--neo-chassis-hover);
}

.rail-tab-btn.active {
    color: var(--neo-ink);
    background: var(--neo-surface);
}

.rail-tab-btn.active::before {
    content: "";
    position: absolute;
    left: 0;
    top: 6px;
    bottom: 6px;
    width: 3px;
    background: var(--neo-signal);
    border-radius: 0 2px 2px 0;
}

/* -------------------------------------------------------------
   LEFT COLLAPSIBLE DRAWER (280px)
   ------------------------------------------------------------- */
#neostudio-drawer {
    width: var(--neo-drawer-w);
    min-width: var(--neo-drawer-w);
    background: var(--neo-surface);
    border-right: 1px solid var(--neo-rule);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    z-index: calc(var(--neo-z-dock) - 1);
    transition: width 0.18s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.15s ease;
}

#neostudio-drawer.drawer-closed {
    width: 0;
    min-width: 0;
    opacity: 0;
    border-right: none;
    pointer-events: none;
}

.drawer-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
}

.drawer-header {
    height: 38px;
    padding: 0 12px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--neo-rule);
    background: var(--neo-surface);
    user-select: none;
}

.drawer-title {
    font-size: 11.5px;
    font-weight: 700;
    letter-spacing: 0.05em;
    color: var(--neo-ink);
    text-transform: uppercase;
}

.drawer-header-actions {
    display: flex;
    align-items: center;
    gap: 4px;
}

.drawer-action-btn {
    background: transparent;
    border: none;
    color: var(--neo-muted);
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--neo-radius-sm);
    cursor: pointer;
    font-size: 11px;
}

.drawer-action-btn:hover {
    color: var(--neo-ink);
    background: var(--neo-surface-hover);
}

.drawer-search-well {
    padding: 8px 12px;
    border-bottom: 1px solid var(--neo-rule);
    background: var(--neo-surface-inset);
}

.drawer-search-input {
    width: 100%;
    height: var(--neo-control-h);
    background: var(--neo-surface);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-sm);
    padding: 0 8px;
    font-size: var(--neo-text-sub);
    color: var(--neo-ink);
    outline: none;
    transition: border-color 0.15s ease;
}

.drawer-search-input:focus {
    border-color: var(--neo-signal);
}

.drawer-scroll-body {
    flex: 1;
    overflow-y: auto;
    padding: 8px 12px;
}

/* -------------------------------------------------------------
   ADD ELEMENTS DRAWER: CATEGORIZED GRID & WIDGET CARDS
   ------------------------------------------------------------- */
.cat-accordion {
    margin-bottom: 12px;
}

.cat-header {
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    cursor: pointer;
    user-select: none;
    margin-bottom: 6px;
}

.cat-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--neo-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    gap: 6px;
}

.cat-count {
    font-family: var(--neo-font-mono);
    font-size: 10px;
    color: var(--neo-muted-dark);
}

.cat-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 6px;
}

.widget-card {
    height: 52px;
    background: var(--neo-surface);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-sm);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    cursor: grab;
    user-select: none;
    transition: all 0.15s ease;
}

.widget-card:hover {
    border-color: var(--neo-signal);
    background: var(--neo-surface-hover);
    transform: translateY(-1px);
}

.widget-card:active {
    cursor: grabbing;
}

.widget-icon {
    font-size: 14px;
    color: var(--neo-ink);
}

.widget-title {
    font-size: 10.5px;
    color: var(--neo-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100px;
}

.widget-card:hover .widget-title {
    color: var(--neo-ink);
}
"#;
