//! Right Webflow-Class Inspector Dock, Selector Bar and Pseudo-State Switcher CSS (`styles/inspector_base.rs`).

pub const CSS_INSPECTOR_BASE: &str = r#"
/* -------------------------------------------------------------
   RIGHT WEBFLOW-CLASS INSPECTOR DOCK (300px)
   ------------------------------------------------------------- */
#neostudio-inspector {
    width: var(--neo-inspector-w);
    min-width: var(--neo-inspector-w);
    background: var(--neo-surface);
    border-left: 1px solid var(--neo-rule);
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    z-index: var(--neo-z-dock);
    user-select: none;
}

/* Tab Bar (Style vs Settings) */
.inspector-tab-nav {
    height: 32px;
    background: var(--neo-surface);
    border-bottom: 1px solid var(--neo-rule);
    display: flex;
    padding: 0 4px;
}

.insp-tab-btn {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--neo-muted);
    font-size: var(--neo-text-sub);
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
    color: var(--neo-ink);
}

.insp-tab-btn.active {
    color: var(--neo-ink);
}

.insp-tab-btn.active::after {
    content: "";
    position: absolute;
    bottom: -1px;
    left: 8px;
    right: 8px;
    height: 2px;
    background: var(--neo-signal);
    border-radius: 1px;
}

/* -------------------------------------------------------------
   SELECTOR & INHERITANCE BAR
   ------------------------------------------------------------- */
.inspector-selector-bar {
    padding: 10px 12px;
    border-bottom: 1px solid var(--neo-rule);
    background: var(--neo-surface-inset);
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
    font-family: var(--neo-font-mono);
    font-size: 11px;
    color: var(--neo-ink);
    font-weight: 600;
}

.target-id {
    color: var(--neo-muted);
    font-size: 10px;
}

.classes-wrapper {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px;
}

.class-pill {
    background: var(--neo-token-tint);
    border: 1px solid var(--neo-token-border);
    color: var(--neo-token);
    font-family: var(--neo-font-mono);
    font-size: 10px;
    padding: 2px 6px;
    border-radius: var(--neo-radius-sm);
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
    background: var(--neo-surface);
    border: 1px dashed var(--neo-rule);
    color: var(--neo-muted);
    font-size: 10px;
    padding: 2px 6px;
    border-radius: var(--neo-radius-sm);
    cursor: pointer;
    font-family: var(--neo-font-mono);
}

.btn-add-class:hover {
    color: var(--neo-ink);
    border-color: var(--neo-rule-focus);
}

.add-class-input {
    background: var(--neo-surface);
    border: 1px solid var(--neo-signal);
    color: var(--neo-ink);
    font-family: var(--neo-font-mono);
    font-size: 10.5px;
    padding: 2px 6px;
    border-radius: var(--neo-radius-sm);
    width: 90px;
    outline: none;
}

/* -------------------------------------------------------------
   PSEUDO-STATE SWITCHER (Default, Hover, Focus, Active)
   ------------------------------------------------------------- */
.state-switcher {
    display: flex;
    background: var(--neo-surface);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-sm);
    padding: 2px;
    gap: 2px;
}

.state-btn {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--neo-muted);
    font-size: 10.5px;
    font-weight: 500;
    padding: 3px 0;
    border-radius: 2px;
    cursor: pointer;
    text-align: center;
    transition: all 0.12s ease;
}

.state-btn:hover {
    color: var(--neo-ink);
    background: var(--neo-surface-hover);
}

.state-btn.active {
    background: var(--neo-surface-active);
    color: var(--neo-ink);
    font-weight: 600;
    box-shadow: 0 1px 2px rgba(0,0,0,0.3);
}

.state-btn[data-state="hover"].active {
    color: var(--neo-token);
}

/* -------------------------------------------------------------
   BREAKPOINT INHERITANCE INDICATORS
   ------------------------------------------------------------- */
.breakpoint-indicator-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 10px;
    color: var(--neo-muted-dark);
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
    background: var(--neo-rule);
}

.bp-dot.active-bp {
    background: var(--neo-signal);
    box-shadow: 0 0 4px var(--neo-signal);
}

.bp-dot.has-override {
    background: var(--neo-token);
}

/* Empty State */
.inspector-empty-state {
    padding: 48px 24px;
    text-align: center;
    color: var(--neo-muted);
    font-size: var(--neo-text-sub);
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
}

.inspector-empty-icon {
    font-size: 28px;
    color: var(--neo-rule-focus);
}
"#;
