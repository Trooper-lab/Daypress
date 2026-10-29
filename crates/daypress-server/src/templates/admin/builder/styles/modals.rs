//! Modals, Command Palette Spotlight, Icon Picker and Toast CSS (`styles/modals.rs`).

pub const CSS_MODALS: &str = r#"
/* -------------------------------------------------------------
   MODALS BASE OVERLAY & CONTAINER
   ------------------------------------------------------------- */
.daystudio-modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.72);
    backdrop-filter: blur(4px);
    display: none;
    align-items: center;
    justify-content: center;
    z-index: var(--day-z-modal);
}

.daystudio-modal-overlay.open {
    display: flex;
    animation: fadeIn 0.12s ease;
}

@keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
}

.daystudio-modal {
    background: var(--day-surface);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-md);
    box-shadow: 0 16px 48px rgba(0, 0, 0, 0.8);
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.daystudio-modal-header {
    height: 42px;
    padding: 0 16px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    border-bottom: 1px solid var(--day-rule);
    background: var(--day-surface);
}

.modal-title {
    font-size: 13px;
    font-weight: 600;
    color: var(--day-ink);
    display: flex;
    align-items: center;
    gap: 8px;
}

.modal-close-btn { background: transparent; border: none; color: var(--day-muted); font-size: 18px; cursor: pointer; line-height: 1; }
.modal-close-btn:hover { color: var(--day-ink); }
.daystudio-modal-body { padding: 16px; overflow-y: auto; color: var(--day-ink); }

.daystudio-modal-footer {
    height: 48px;
    padding: 0 16px;
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 8px;
    border-top: 1px solid var(--day-rule);
    background: var(--day-surface);
}

/* CAD Buttons */
.cad-btn {
    height: var(--day-control-h);
    padding: 0 12px;
    border-radius: var(--day-radius-sm);
    font-size: var(--day-text-sub);
    font-weight: 600;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    transition: all 0.15s ease;
}

.cad-btn.secondary {
    background: var(--day-surface-inset);
    border: 1px solid var(--day-rule);
    color: var(--day-ink);
}

.cad-btn.secondary:hover {
    background: var(--day-surface-hover);
    border-color: var(--day-rule-focus);
}

.cad-btn.primary {
    background: var(--day-signal);
    border: 1px solid var(--day-signal-hover);
    color: var(--day-ink-contrast);
}

.cad-btn.primary:hover {
    background: var(--day-signal-hover);
}

/* -------------------------------------------------------------
   COMMAND PALETTE SPOTLIGHT (Cmd+K)
   ------------------------------------------------------------- */
.modal-spotlight {
    width: 580px;
    max-width: 90vw;
    border-color: var(--day-rule-focus);
    box-shadow: 0 20px 60px rgba(0, 0, 0, 0.9);
}

.spotlight-header {
    display: flex;
    align-items: center;
    padding: 12px 16px;
    border-bottom: 1px solid var(--day-rule);
    gap: 12px;
}

.spotlight-icon {
    color: var(--day-signal);
    font-size: 16px;
}

#finder-input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--day-ink);
    font-size: 13.5px;
    outline: none;
}

.spotlight-badge {
    font-size: 10px;
    color: var(--day-muted);
    background: var(--day-chassis);
    border: 1px solid var(--day-rule);
    padding: 2px 6px;
    border-radius: 3px;
    font-family: var(--day-font-mono);
}

.spotlight-results {
    max-height: 360px;
    overflow-y: auto;
    padding: 6px;
}

.spotlight-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 10px;
    border-radius: var(--day-radius-sm);
    cursor: pointer;
    transition: all 0.1s ease;
}

.spotlight-item:hover, .spotlight-item.active {
    background: var(--day-surface-hover);
}

.spotlight-item.active {
    border-left: 2px solid var(--day-signal);
}

.spotlight-item-left {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 12px;
    color: var(--day-ink);
}

.spotlight-cat-tag {
    font-size: 10px;
    color: var(--day-muted);
    background: var(--day-surface-inset);
    padding: 2px 6px;
    border-radius: 2px;
}

/* -------------------------------------------------------------
   ICON LIBRARY MODAL
   ------------------------------------------------------------- */
.icon-modal-layout {
    width: 760px;
    max-width: 92vw;
    height: 560px;
}

.icon-filter-bar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 12px;
}

.icon-category-tabs {
    display: flex;
    gap: 4px;
}

.icon-tab {
    background: var(--day-surface-inset);
    border: 1px solid var(--day-rule);
    color: var(--day-muted);
    padding: 4px 10px;
    border-radius: var(--day-radius-sm);
    font-size: 11px;
    cursor: pointer;
}

.icon-tab.active {
    background: var(--day-surface-active);
    color: var(--day-ink);
    border-color: var(--day-signal);
}

.icon-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(72px, 1fr));
    gap: 8px;
    max-height: 380px;
    overflow-y: auto;
    padding: 4px;
}

.icon-card {
    height: 64px;
    background: var(--day-surface-inset);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-sm);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 6px;
    cursor: pointer;
    transition: all 0.12s ease;
}

.icon-card:hover {
    border-color: var(--day-signal);
    background: var(--day-surface-hover);
}

.icon-card.selected {
    border-color: var(--day-signal);
    background: var(--day-surface-active);
    box-shadow: 0 0 8px rgba(37, 99, 235, 0.4);
}

.icon-card i {
    font-size: 18px;
    color: var(--day-ink);
}

.icon-card-name {
    font-size: 9.5px;
    color: var(--day-muted);
    text-align: center;
    max-width: 60px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

/* -------------------------------------------------------------
   TOAST NOTIFICATION
   ------------------------------------------------------------- */
.daystudio-toast {
    position: fixed;
    bottom: 36px;
    right: 24px;
    background: var(--day-surface);
    border: 1px solid var(--day-emerald);
    color: var(--day-ink);
    padding: 8px 16px;
    border-radius: var(--day-radius-sm);
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--day-text-sub);
    z-index: var(--day-z-toast);
    transform: translateY(60px);
    opacity: 0;
    transition: all 0.2s cubic-bezier(0.16, 1, 0.3, 1);
    pointer-events: none;
}

.daystudio-toast.show {
    transform: translateY(0);
    opacity: 1;
}

.toast-icon {
    color: var(--day-emerald);
    font-size: 14px;
}

/* -------------------------------------------------------------
   CREATE TEMPLATE & CONFIRMATION MODALS
   ------------------------------------------------------------- */
.cad-form-error {
    color: var(--day-crimson);
    font-size: 11px;
    padding: 6px 10px;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: var(--day-radius-sm);
    line-height: 1.4;
}
.starter-chips-container {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    max-height: 170px;
    overflow-y: auto;
    padding: 2px;
}
.starter-chip-card {
    background: var(--day-surface-inset);
    border: 1px solid var(--day-rule);
    border-radius: var(--day-radius-sm);
    padding: 8px 10px;
    cursor: pointer;
    transition: all 0.12s ease;
    display: flex;
    flex-direction: column;
    gap: 4px;
}
.starter-chip-card:hover {
    border-color: var(--day-rule-focus);
    background: var(--day-surface-hover);
}
.starter-chip-card.selected {
    border-color: var(--day-signal);
    background: var(--day-surface-active);
    box-shadow: 0 0 6px rgba(37, 99, 235, 0.3);
}
.starter-chip-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
}
.starter-chip-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--day-ink);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}
.starter-chip-badge {
    font-size: 9px;
    padding: 1px 5px;
    border-radius: 2px;
    background: var(--day-chassis);
    color: var(--day-muted);
    font-family: var(--day-font-mono);
}
.starter-chip-desc {
    font-size: 10px;
    color: var(--day-muted);
    line-height: 1.3;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
}
"#;
