//! Top Studio Bar (40px) CSS (`styles/topbar.rs`).

pub const CSS_TOPBAR: &str = r#"
/* -------------------------------------------------------------
   TOP STUDIO BAR (40px)
   ------------------------------------------------------------- */
#neostudio-topbar {
    height: 40px !important;
    min-height: 40px !important;
    max-height: 44px !important;
    overflow: hidden !important;
    flex-shrink: 0 !important;
    background: var(--neo-chassis);
    border-bottom: 1px solid var(--neo-rule);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    z-index: var(--neo-z-dock);
    user-select: none;
    box-sizing: border-box !important;
}

.topbar-left, .topbar-center, .topbar-right {
    display: flex;
    align-items: center;
    gap: 8px;
}

.topbar-brand {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--neo-ink);
    text-decoration: none;
    font-weight: 700;
    font-size: var(--neo-text-sub);
    letter-spacing: 0.08em;
}

.brand-glyph {
    color: var(--neo-signal);
    font-size: 14px;
}

.topbar-divider {
    width: 1px;
    height: 18px;
    background: var(--neo-rule);
    margin: 0 4px;
}

.switcher-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background: var(--neo-surface);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-sm);
    padding: 2px 8px;
    height: var(--neo-control-sm-h);
    color: var(--neo-ink);
    font-size: var(--neo-text-sub);
    cursor: pointer;
    transition: border-color 0.15s ease;
}

.switcher-btn:hover {
    border-color: var(--neo-signal);
}

.template-type-badge {
    background: var(--neo-violet-tint);
    color: var(--neo-violet);
    font-size: var(--neo-text-micro);
    font-weight: 600;
    padding: 1px 5px;
    border-radius: 2px;
    text-transform: uppercase;
}

.template-title {
    max-width: 140px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.switcher-caret { font-size: 9px; color: var(--neo-muted); }
.topbar-switcher { position: relative; }
.switcher-dropdown {
    /* Portaled to <body> and positioned against the switcher button by topbar_switcher.rs */
    display: none;
    position: fixed;
    top: 46px;
    left: 0;
    width: 360px;
    max-height: calc(100vh - 60px);
    background: var(--neo-chassis);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-md);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.75);
    z-index: var(--neo-z-palette);
    flex-direction: column;
    overflow: hidden;
    font-family: var(--neo-font-ui);
}
.switcher-dropdown.open { display: flex; }
.switcher-search-box { padding: 8px; border-bottom: 1px solid var(--neo-rule); background: var(--neo-surface); flex-shrink: 0; }
.switcher-search-box .cad-input { width: 100%; }
.switcher-scroll-list { flex: 1; min-height: 0; max-height: 420px; overflow-y: auto; padding: 6px; }
.switcher-group-label {
    font-size: 10px;
    font-weight: 700;
    color: var(--neo-muted);
    padding: 6px 8px 4px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}
.switcher-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 8px;
    border-radius: var(--neo-radius-sm);
    cursor: pointer;
    color: var(--neo-ink);
    font-size: 12px;
    gap: 8px;
}
.switcher-item-title { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.switcher-item .template-type-badge { flex-shrink: 0; }
.switcher-item:hover, .switcher-item.kbd-active { background: var(--neo-surface-hover); }
.switcher-item.active { background: rgba(37, 99, 235, 0.16); box-shadow: inset 0 0 0 1px var(--neo-signal); }
.switcher-empty { padding: 12px 10px; font-size: 11px; color: var(--neo-muted); text-align: center; }
.switcher-empty.is-error { color: var(--neo-crimson); }
.switcher-footer { display: flex; gap: 6px; padding: 8px; border-top: 1px solid var(--neo-rule); background: var(--neo-surface); flex-shrink: 0; }

/* Breakpoint Switcher */
.breakpoint-switcher {
    display: flex;
    background: var(--neo-surface-inset);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-sm);
    padding: 2px;
    gap: 2px;
}

.bp-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: none;
    border-radius: 2px;
    padding: 2px 8px;
    height: var(--neo-control-sm-h);
    color: var(--neo-muted);
    font-size: var(--neo-text-sub);
    cursor: pointer;
    transition: all 0.12s ease;
}

.bp-btn:hover {
    color: var(--neo-ink);
    background: var(--neo-surface-hover);
}

.bp-btn.active {
    color: var(--neo-ink);
    background: var(--neo-surface);
    box-shadow: 0 1px 2px rgba(0,0,0,0.4);
}

.bp-btn.active i {
    color: var(--neo-signal);
}

/* Live Dimension Readout */
.dimension-readout {
    font-family: var(--neo-font-mono);
    font-size: var(--neo-text-sub);
    font-variant-numeric: tabular-nums;
    color: var(--neo-muted);
    background: var(--neo-surface-inset);
    border: 1px solid var(--neo-rule-subtle);
    padding: 2px 8px;
    border-radius: var(--neo-radius-sm);
    height: var(--neo-control-sm-h);
    display: flex;
    align-items: center;
}

.dim-unit {
    color: var(--neo-muted-dark);
    margin: 0 2px;
}

/* Canvas Zoom Engine */
.zoom-controller {
    display: flex;
    align-items: center;
    background: var(--neo-surface-inset);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-sm);
    height: var(--neo-control-sm-h);
}

.zoom-btn {
    background: transparent;
    border: none;
    color: var(--neo-muted);
    width: 22px;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 10px;
}

.zoom-btn:hover {
    color: var(--neo-ink);
    background: var(--neo-surface-hover);
}

.zoom-pill {
    background: transparent;
    border: none;
    color: var(--neo-ink);
    font-family: var(--neo-font-mono);
    font-size: 11px;
    font-variant-numeric: tabular-nums;
    padding: 0 4px;
    cursor: pointer;
}

/* Topbar Tools & Buttons */
.topbar-tool-btn, .topbar-icon-btn {
    display: flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: 1px solid transparent;
    border-radius: var(--neo-radius-sm);
    height: var(--neo-control-sm-h);
    padding: 0 8px;
    color: var(--neo-muted);
    font-size: var(--neo-text-sub);
    cursor: pointer;
    transition: all 0.12s ease;
}

.topbar-icon-btn {
    width: 28px;
    padding: 0;
    justify-content: center;
}

.topbar-tool-btn:hover, .topbar-icon-btn:hover {
    color: var(--neo-ink);
    background: var(--neo-surface);
    border-color: var(--neo-rule);
}

.topbar-tool-btn.active, .topbar-icon-btn.active {
    color: var(--neo-signal-light);
    background: var(--neo-surface);
    border-color: var(--neo-signal);
}

.topbar-tool-btn:disabled, .topbar-icon-btn:disabled {
    opacity: 0.35;
    cursor: not-allowed;
}

.hotkey-badge {
    font-family: var(--neo-font-mono);
    font-size: 10px;
    color: var(--neo-muted-dark);
    background: var(--neo-surface-inset);
    padding: 1px 4px;
    border-radius: 2px;
    border: 1px solid var(--neo-rule-subtle);
}

/* Spotlight Trigger */
.spotlight-trigger {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--neo-surface-inset);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-sm);
    padding: 2px 10px;
    height: var(--neo-control-sm-h);
    color: var(--neo-muted);
    font-size: var(--neo-text-sub);
    cursor: pointer;
    transition: all 0.15s ease;
}

.spotlight-trigger:hover {
    border-color: var(--neo-signal);
    color: var(--neo-ink);
}

.spotlight-hint {
    color: var(--neo-muted-dark);
    font-size: 11px;
}

/* Save Status Pill & History Quick Tools */
.history-quick-tools {
    display: flex;
    align-items: center;
    gap: 2px;
    background: var(--neo-surface-inset);
    border: 1px solid var(--neo-rule-subtle);
    border-radius: var(--neo-radius-sm);
    padding: 2px;
    height: var(--neo-control-sm-h);
}

.history-btn {
    background: transparent;
    border: none;
    color: var(--neo-muted);
    width: 24px;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    font-size: 11px;
    border-radius: 2px;
    padding: 0;
}

.history-btn:hover:not(.disabled) {
    color: var(--neo-ink);
    background: var(--neo-surface-hover);
}

.history-btn.disabled {
    opacity: 0.35;
    cursor: not-allowed;
}

.kbd-hint {
    font-family: var(--neo-font-mono);
    font-size: 10px;
    color: var(--neo-muted-dark);
    background: var(--neo-surface-inset);
    padding: 1px 4px;
    border-radius: 2px;
}

.save-status-group {
    display: flex;
    align-items: center;
    gap: 8px;
}

.save-status-pill {
    display: flex;
    align-items: center;
    gap: 6px;
    font-family: var(--neo-font-mono);
    font-size: 11px;
    padding: 2px 8px;
    border-radius: var(--neo-radius-sm);
    background: var(--neo-surface-inset);
    border: 1px solid var(--neo-rule-subtle);
}

.status-dot { width: 6px; height: 6px; border-radius: 50%; }
.save-status-pill.saved { color: var(--neo-emerald); }
.save-status-pill.saved .status-dot { background: var(--neo-emerald); box-shadow: 0 0 6px var(--neo-emerald); }

.save-status-pill.unsaved {
    color: var(--neo-token);
}
.save-status-pill.unsaved .status-dot { background: var(--neo-token); box-shadow: 0 0 6px var(--neo-token); }
.save-status-pill.saving { color: var(--neo-signal-light); }
.save-status-pill.saving .status-dot { background: var(--neo-signal-light); animation: pulseSave 0.8s infinite alternate; }

@keyframes pulseSave {
    0% { transform: scale(0.8); opacity: 0.5; }
    100% { transform: scale(1.3); opacity: 1; }
}

.btn-publish-action {
    display: flex; align-items: center; gap: 6px;
    background: var(--neo-signal); border: 1px solid var(--neo-signal-hover);
    border-radius: var(--neo-radius-sm); padding: 0 12px; height: var(--neo-control-h);
    color: #ffffff; font-size: var(--neo-text-sub); font-weight: 600;
    cursor: pointer; transition: background 0.15s ease;
}
.btn-publish-action:hover { background: var(--neo-signal-hover); }
"#;
