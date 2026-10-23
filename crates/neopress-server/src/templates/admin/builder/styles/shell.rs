//! 3-Pane Zero-Scroll Dock Shell and Bottom Breadcrumb Bar CSS (`styles/shell.rs`).

pub const CSS_SHELL: &str = r#"
#neostudio-shell {
    width: 100vw;
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--neo-chassis);
}

/* -------------------------------------------------------------
   MAIN WORKSTATION DOCK
   ------------------------------------------------------------- */
#neostudio-dock {
    flex: 1;
    display: flex;
    overflow: hidden;
    position: relative;
    background: var(--neo-chassis);
}

/* Chrome isolation reset preventing theme css leaks into CAD dock shell */
#neostudio-shell *:not(#neostudio-artboard):not(#neostudio-artboard *) {
    box-sizing: border-box !important;
}

/* -------------------------------------------------------------
   BOTTOM BREADCRUMB BAR (28px)
   ------------------------------------------------------------- */
#neostudio-bottom-bar {
    height: 28px !important;
    min-height: 28px !important;
    max-height: 32px !important;
    overflow: hidden !important;
    flex-shrink: 0 !important;
    background: var(--neo-chassis);
    border-top: 1px solid var(--neo-rule);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    font-size: var(--neo-text-sub);
    color: var(--neo-muted);
    z-index: var(--neo-z-dock);
    user-select: none;
    box-sizing: border-box !important;
}

.bottom-bar-left, .bottom-bar-right {
    display: flex;
    align-items: center;
    gap: 8px;
}

.bc-label {
    font-size: 10px;
    color: var(--neo-muted-dark);
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.breadcrumb-trail {
    display: flex;
    align-items: center;
    gap: 4px;
    font-family: var(--neo-font-mono);
    font-size: 11px;
}

.bc-item {
    color: var(--neo-muted);
    cursor: pointer;
    padding: 1px 4px;
    border-radius: 2px;
    transition: all 0.12s ease;
}

.bc-item:hover {
    color: var(--neo-ink);
    background: var(--neo-surface);
}

.bc-item.active {
    color: var(--neo-signal-light);
    font-weight: 600;
}

.bc-sep {
    color: var(--neo-rule);
    font-size: 9px;
}

.hud-stat {
    font-family: var(--neo-font-mono);
    font-size: 10.5px;
    color: var(--neo-muted);
    font-variant-numeric: tabular-nums;
}

.hud-stat-sep {
    color: var(--neo-rule);
    font-size: 10px;
}
"#;
