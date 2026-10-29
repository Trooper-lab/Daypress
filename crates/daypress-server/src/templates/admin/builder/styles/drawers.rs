//! Left Studio Rail (48px) and Add Elements Drawer (280px) CSS (`styles/drawers.rs`).

pub const CSS_DRAWERS: &str = r#"
/* -------------------------------------------------------------
   LEFT STUDIO RAIL (48px)
   ------------------------------------------------------------- */
#daystudio-rail {
    width: var(--day-rail-w);
    min-width: var(--day-rail-w);
    background: var(--day-chassis);
    border-right: 1px solid var(--day-rule);
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    z-index: var(--day-z-dock);
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
    color: var(--day-muted);
    border: none;
    background: transparent;
    cursor: pointer;
    position: relative;
    font-size: 14px;
    transition: all 0.15s ease;
}

.rail-tab-btn:hover {
    color: var(--day-ink);
    background: var(--day-chassis-hover);
}

.rail-tab-btn.active {
    color: var(--day-ink);
    background: var(--day-surface);
}

.rail-tab-btn.active::before {
    content: "";
    position: absolute;
    left: 0;
    top: 6px;
    bottom: 6px;
    width: 3px;
    background: var(--day-signal);
    border-radius: 0 2px 2px 0;
}

/* -------------------------------------------------------------
   LEFT COLLAPSIBLE DRAWER (280px)
   ------------------------------------------------------------- */
#daystudio-drawer {
    width: var(--day-drawer-w);
    min-width: var(--day-drawer-w);
    background: var(--day-surface);
    border-right: 1px solid var(--day-rule);
    overflow: hidden;
    display: flex;
    flex-direction: column;
    z-index: calc(var(--day-z-dock) - 1);
    transition: width 0.18s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.15s ease;
}

#daystudio-drawer.drawer-closed {
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
    border-bottom: 1px solid var(--day-rule);
    background: var(--day-surface);
    user-select: none;
}

.drawer-title {
    font-size: 11.5px;
    font-weight: 700;
    letter-spacing: 0.05em;
    color: var(--day-ink);
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
    color: var(--day-muted);
    width: 22px;
    height: 22px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: var(--day-radius-sm);
    cursor: pointer;
    font-size: 11px;
}

.drawer-action-btn:hover {
    color: var(--day-ink);
    background: var(--day-surface-hover);
}

.drawer-search-well {
    padding: 8px 12px;
    border-bottom: 1px solid var(--day-rule);
    background: var(--day-surface-inset);
}

.drawer-search-input {
    width: 100%;
    height: var(--day-control-h);
    background: var(--day-surface);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-sm);
    padding: 0 8px;
    font-size: var(--day-text-sub);
    color: var(--day-ink);
    outline: none;
    transition: border-color 0.15s ease;
}

.drawer-search-input:focus {
    border-color: var(--day-signal);
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
    color: var(--day-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    display: flex;
    align-items: center;
    gap: 6px;
}

.cat-count {
    font-family: var(--day-font-mono);
    font-size: 10px;
    color: var(--day-muted-dark);
}

.cat-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 6px;
}

.widget-card {
    height: 52px;
    background: var(--day-surface);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-sm);
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
    border-color: var(--day-signal);
    background: var(--day-surface-hover);
    transform: translateY(-1px);
}

.widget-card:active {
    cursor: grabbing;
}

.widget-icon {
    font-size: 14px;
    color: var(--day-ink);
}

.widget-title {
    font-size: 10.5px;
    color: var(--day-muted);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100px;
}

.widget-card:hover .widget-title {
    color: var(--day-ink);
}
"#;
