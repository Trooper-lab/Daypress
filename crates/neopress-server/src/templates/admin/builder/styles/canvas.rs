//! Center Responsive Artboard Canvas, CAD dot grid, zoom wrapper and resize handles (`styles/canvas.rs`).

pub const CSS_CANVAS: &str = r#"
/* -------------------------------------------------------------
   CENTER RESPONSIVE CANVAS VIEWPORT
   ------------------------------------------------------------- */
#neostudio-canvas-viewport {
    flex: 1;
    position: relative;
    overflow: auto;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding: 24px 32px;
    background-color: var(--neo-chassis-subtle);
    background-image: radial-gradient(var(--neo-rule) 1px, transparent 1px);
    background-size: 24px 24px;
    background-position: 0 0;
}

#neostudio-zoom-wrapper {
    position: relative;
    transform-origin: top center;
    transition: transform 0.15s cubic-bezier(0.16, 1, 0.3, 1);
    display: flex;
    justify-content: center;
    align-items: stretch;
    width: 100%;
    max-width: 1200px;
}

#neostudio-artboard {
    position: relative;
    background: #ffffff;
    min-height: 850px;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.7);
    transition: width 0.22s cubic-bezier(0.16, 1, 0.3, 1);
    margin: 0 auto;
    color: #0f172a;
    box-sizing: border-box;
    contain: layout paint !important;
    transform: translateZ(0) !important;
    isolation: isolate !important;
    overflow: clip !important;
}

/* Breakpoint width transitions */
#neostudio-artboard.artboard-desktop {
    width: 1200px;
}

#neostudio-artboard.artboard-tablet {
    width: 768px;
}

#neostudio-artboard.artboard-mobile {
    width: 375px;
}

/* -------------------------------------------------------------
   TACTILE VIEWPORT DRAG RESIZE HANDLES
   ------------------------------------------------------------- */
.artboard-handle {
    position: relative;
    width: 18px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: ew-resize;
    z-index: var(--neo-z-artboard-handles);
    user-select: none;
    transition: background 0.12s ease;
    flex-shrink: 0;
}

.artboard-handle:hover, .artboard-handle.is-dragging {
    background: rgba(37, 99, 235, 0.12);
}

.artboard-handle-left {
    margin-right: -18px;
    z-index: 20;
}

.artboard-handle-right {
    margin-left: -18px;
    z-index: 20;
}

.handle-grip {
    width: 4px;
    height: 48px;
    background: var(--neo-rule);
    border-radius: 2px;
    transition: all 0.15s ease;
}

.artboard-handle:hover .handle-grip,
.artboard-handle.is-dragging .handle-grip {
    background: var(--neo-signal);
    height: 64px;
    box-shadow: 0 0 8px var(--neo-signal);
}

/* -------------------------------------------------------------
   HEADER & FOOTER SSR PREVIEW SHELLS
   ------------------------------------------------------------- */
#neostudio-ssr-header, #neostudio-ssr-footer {
    width: 100%;
    pointer-events: none;
    opacity: 0.95;
    user-select: none;
}

#neostudio-ssr-header {
    border-bottom: 1px dashed var(--neo-rule);
}

#neostudio-ssr-footer {
    border-top: 1px dashed var(--neo-rule);
}

/* -------------------------------------------------------------
   EDITABLE CANVAS ROOT
   ------------------------------------------------------------- */
#neostudio-canvas {
    min-height: 400px;
    position: relative;
    display: flex;
    flex-direction: column;
}

/* -------------------------------------------------------------
   BOTTOM ADD SECTION PRESET DROPZONE
   ------------------------------------------------------------- */
.neostudio-add-section-area {
    margin: 32px 24px;
    border: 2px dashed var(--neo-rule);
    border-radius: var(--neo-radius-md);
    background: var(--neo-surface-inset);
    padding: 24px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    transition: all 0.18s ease;
}

.neostudio-add-section-area:hover {
    border-color: var(--neo-signal);
    background: var(--neo-surface);
}

.add-section-prompt {
    display: flex;
    align-items: center;
    gap: 12px;
}

.btn-add-section-action {
    display: flex;
    align-items: center;
    gap: 8px;
    background: var(--neo-surface);
    border: 1px solid var(--neo-rule);
    color: var(--neo-ink);
    padding: 8px 16px;
    border-radius: var(--neo-radius-sm);
    font-size: var(--neo-text-sub);
    font-weight: 600;
    cursor: pointer;
    transition: all 0.15s ease;
}

.btn-add-section-action:hover {
    background: var(--neo-signal);
    border-color: var(--neo-signal);
    color: #ffffff;
}

/* Structure Presets Grid */
.preset-selector {
    display: none;
    margin-top: 20px;
    width: 100%;
    max-width: 500px;
    animation: fadeIn 0.15s ease;
}

.preset-selector.open {
    display: block;
}

.preset-title {
    font-size: 11px;
    font-weight: 600;
    color: var(--neo-muted);
    text-align: center;
    margin-bottom: 12px;
    text-transform: uppercase;
    letter-spacing: 0.05em;
}

.preset-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 10px;
}

.preset-card {
    height: 48px;
    background: var(--neo-surface);
    border: 1px solid var(--neo-rule);
    border-radius: var(--neo-radius-sm);
    display: flex;
    padding: 6px;
    gap: 4px;
    cursor: pointer;
    transition: all 0.15s ease;
}

.preset-card:hover {
    border-color: var(--neo-signal);
    transform: translateY(-1px);
}

.preset-col {
    background: var(--neo-rule);
    border-radius: 2px;
    transition: background 0.15s ease;
}

.preset-card:hover .preset-col {
    background: var(--neo-signal);
}
"#;
