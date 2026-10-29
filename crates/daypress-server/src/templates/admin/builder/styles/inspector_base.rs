//! Right Webflow-Class Inspector Dock, Selector Bar and Pseudo-State Switcher CSS (`styles/inspector_base.rs`).

pub const CSS_INSPECTOR_BASE: &str = r#"
/* -------------------------------------------------------------
   RIGHT WEBFLOW-CLASS INSPECTOR DOCK (300px)
   ------------------------------------------------------------- */
#daystudio-inspector {
    width: var(--day-inspector-w);
    min-width: var(--day-inspector-w);
    background: var(--day-surface);
    border-left: 1px solid var(--day-rule);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    z-index: var(--day-z-dock);
    user-select: none;
}

/* Tab Bar (Style vs Settings) */
.inspector-tab-nav {
    height: 32px;
    background: var(--day-surface);
    border-bottom: 1px solid var(--day-rule);
    display: flex;
    padding: 0 4px;
}

.insp-tab-btn {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--day-muted);
    font-size: var(--day-text-sub);
    font-weight: 600;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 6px;
    position: relative;
    transition: color 0.12s ease;
}

.insp-tab-btn:hover {
    color: var(--day-ink);
}

.insp-tab-btn.active {
    color: var(--day-ink);
}

.insp-tab-btn.active::after {
    content: "";
    position: absolute;
    bottom: -1px;
    left: 8px;
    right: 8px;
    height: 2px;
    background: var(--day-signal);
    border-radius: 1px;
}

/* -------------------------------------------------------------
   SELECTOR & INHERITANCE BAR
   ------------------------------------------------------------- */
.inspector-selector-bar {
    padding: 10px 12px;
    border-bottom: 1px solid var(--day-rule);
    background: var(--day-surface-inset);
    display: flex;
    flex-direction: column;
    gap: 8px;
}

.selector-target-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.target-badge {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: var(--day-font-mono);
    font-size: 11px;
    color: var(--day-ink);
    font-weight: 600;
}

.target-id {
    color: var(--day-muted);
    font-size: 10px;
}

.classes-wrapper {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px;
}

.class-pill {
    background: var(--day-token-tint);
    border: 1px solid var(--day-token-border);
    color: var(--day-token);
    font-family: var(--day-font-mono);
    font-size: 10px;
    padding: 2px 6px;
    border-radius: var(--day-radius-sm);
    display: flex;
    align-items: center;
    gap: 4px;
}

.class-pill-remove {
    cursor: pointer;
    font-size: 9px;
    opacity: 0.7;
}

.class-pill-remove:hover {
    opacity: 1;
}

.btn-add-class {
    background: var(--day-surface);
    border: 1px dashed var(--day-rule);
    color: var(--day-muted);
    font-size: 10px;
    padding: 2px 6px;
    border-radius: var(--day-radius-sm);
    cursor: pointer;
    font-family: var(--day-font-mono);
}

.btn-add-class:hover {
    color: var(--day-ink);
    border-color: var(--day-rule-focus);
}

.add-class-input {
    background: var(--day-surface);
    border: 1px solid var(--day-signal);
    color: var(--day-ink);
    font-family: var(--day-font-mono);
    font-size: 10.5px;
    padding: 2px 6px;
    border-radius: var(--day-radius-sm);
    width: 90px;
    outline: none;
}

/* -------------------------------------------------------------
   PSEUDO-STATE SWITCHER (Default, Hover, Focus, Active)
   ------------------------------------------------------------- */
.state-switcher {
    display: flex;
    background: var(--day-surface);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-sm);
    padding: 2px;
    gap: 2px;
}

.state-btn {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--day-muted);
    font-size: 10.5px;
    font-weight: 500;
    padding: 3px 0;
    border-radius: 2px;
    cursor: pointer;
    text-align: center;
    transition: all 0.12s ease;
}

.state-btn:hover {
    color: var(--day-ink);
    background: var(--day-surface-hover);
}

.state-btn.active {
    background: var(--day-surface-active);
    color: var(--day-ink);
    font-weight: 600;
    box-shadow: 0 1px 2px rgba(0,0,0,0.3);
}

.state-btn[data-state="hover"].active {
    color: var(--day-token);
}

/* -------------------------------------------------------------
   BREAKPOINT INHERITANCE INDICATORS
   ------------------------------------------------------------- */
.breakpoint-indicator-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 10px;
    color: var(--day-muted-dark);
}

.bp-cascade-dots {
    display: flex;
    align-items: center;
    gap: 4px;
}

.bp-dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--day-rule);
}

.bp-dot.active-bp {
    background: var(--day-signal);
    box-shadow: 0 0 4px var(--day-signal);
}

.bp-dot.has-override {
    background: var(--day-token);
}

/* Empty State */
.inspector-empty-state {
    padding: 48px 24px;
    text-align: center;
    color: var(--day-muted);
    font-size: var(--day-text-sub);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
}

.inspector-empty-icon {
    font-size: 28px;
    color: var(--day-rule-focus);
}
"#;
