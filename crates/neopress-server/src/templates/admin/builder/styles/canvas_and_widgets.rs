//! Right preview canvas (#elementor-preview), responsive frames, handles, and drop zones.

pub const STYLE_CANVAS_AND_WIDGETS: &str = r###"        /* ------------------------------------------------------------- */
        /* RIGHT PREVIEW CANVAS: #elementor-preview                      */
        /* ------------------------------------------------------------- */
        #elementor-preview {
            flex: 1;
            height: 100vh;
            overflow-y: auto;
            overflow-x: hidden;
            background: #ffffff;
            position: relative;
            display: flex;
            flex-direction: column;
        }

        /* Floating Responsive Bar */
        #elementor-responsive-bar {
            height: 42px;
            background: #1e2124;
            border-bottom: 1px solid var(--e-border);
            display: none;
            align-items: center;
            justify-content: center;
            gap: 14px;
            position: sticky;
            top: 0;
            z-index: 500;
            padding: 0 16px;
            box-shadow: 0 2px 10px rgba(0, 0, 0, 0.2);
        }

        #elementor-responsive-bar.open {
            display: flex;
        }

        .resp-btn {
            background: transparent;
            border: 1px solid transparent;
            color: var(--e-color-txt-muted);
            padding: 5px 12px;
            border-radius: 4px;
            cursor: pointer;
            display: flex;
            align-items: center;
            gap: 6px;
            font-size: 12px;
            font-weight: 500;
            transition: all 0.2s;
        }

        .resp-btn:hover {
            color: #ffffff;
            background: rgba(255, 255, 255, 0.06);
        }

        .resp-btn.active {
            color: #ffffff;
            background: var(--e-color-accent);
        }

        .resp-dimensions {
            font-size: 12px;
            color: var(--e-color-txt-muted);
            background: #141618;
            padding: 4px 10px;
            border-radius: 3px;
            border: 1px solid #2e3236;
        }

        .btn-close-resp-bar {
            background: transparent;
            border: none;
            color: var(--e-color-txt-muted);
            cursor: pointer;
            font-size: 14px;
            margin-left: 10px;
        }

        .btn-close-resp-bar:hover {
            color: #ffffff;
        }

        /* Floating Layers Button on Canvas */
        .floating-layers-btn {
            position: fixed;
            bottom: 24px;
            left: calc(var(--e-panel-width) + 24px);
            background: #1e2124;
            border: 1px solid #3c4046;
            color: #ffffff;
            padding: 8px 16px;
            border-radius: 20px;
            cursor: pointer;
            display: flex;
            align-items: center;
            gap: 8px;
            font-size: 12px;
            font-weight: 600;
            box-shadow: 0 4px 16px rgba(0, 0, 0, 0.35);
            z-index: 400;
            transition: all 0.2s;
        }

        .floating-layers-btn:hover {
            background: var(--e-color-accent);
            border-color: var(--e-color-accent);
            transform: translateY(-2px);
        }

        /* Responsive Wrapper & Canvas Frame */
        #elementor-preview-responsive-wrapper {
            flex: 1;
            width: 100%;
            min-height: 100%;
            display: flex;
            flex-direction: column;
            transition: all 0.3s ease;
            position: relative;
        }

        /* Tablet Viewport */
        #elementor-preview.mode-tablet {
            background: #2b2d31;
            padding: 24px 0;
        }

        #elementor-preview.mode-tablet #elementor-preview-responsive-wrapper {
            width: 768px;
            max-width: 768px;
            margin: 0 auto;
            background: #ffffff;
            border-radius: 8px;
            box-shadow: 0 10px 40px rgba(0, 0, 0, 0.4);
            min-height: 90vh;
        }

        /* Mobile Viewport */
        #elementor-preview.mode-mobile {
            background: #2b2d31;
            padding: 24px 0;
        }

        #elementor-preview.mode-mobile #elementor-preview-responsive-wrapper {
            width: 375px;
            max-width: 375px;
            margin: 0 auto;
            background: #ffffff;
            border-radius: 12px;
            box-shadow: 0 10px 40px rgba(0, 0, 0, 0.4);
            min-height: 85vh;
        }

        /* Canvas Site Navigation Header */
        .preview-site-header {
            background: #0f172a;
            color: #ffffff;
            padding: 16px 32px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            border-bottom: 1px solid rgba(255, 255, 255, 0.1);
        }

        .preview-site-brand {
            font-size: 18px;
            font-weight: 800;
            letter-spacing: -0.5px;
            color: #ffffff;
            display: flex;
            align-items: center;
            gap: 8px;
            text-decoration: none;
        }

        .preview-site-brand span {
            color: #ff4f00;
        }

        .preview-site-nav {
            display: flex;
            align-items: center;
            gap: 20px;
        }

        .preview-site-nav a {
            color: #cbd5e1;
            text-decoration: none;
            font-size: 13px;
            font-weight: 500;
            transition: color 0.2s;
        }

        .preview-site-nav a:hover {
            color: #ffffff;
        }

        /* Canvas Site Footer */
        .preview-site-footer {
            background: #090d16;
            color: #64748b;
            padding: 40px 32px;
            text-align: center;
            font-size: 13px;
            border-top: 1px solid rgba(255, 255, 255, 0.08);
            margin-top: auto;
        }

        /* Canvas Editable Workspace */
        #elementor-canvas {
            min-height: 400px;
            position: relative;
            color: #0f172a;
        }

        /* Canvas Section */
        .elementor-section {
            position: relative;
            transition: outline 0.15s;
        }

        .elementor-section:hover {
            outline: 1px solid var(--e-color-blue);
        }

        .elementor-section.active-element {
            outline: 2px solid var(--e-color-blue) !important;
        }

        .elementor-section.is-dragging {
            opacity: 0.35;
            outline: 2px dashed var(--e-color-blue) !important;
        }

        /* Elementor Top Section Handle Pill */
        .elementor-section-handle {
            position: absolute;
            top: 0;
            left: 50%;
            transform: translate(-50%, -100%);
            height: 24px;
            background: var(--e-color-blue);
            color: #ffffff;
            border-radius: 4px 4px 0 0;
            display: none;
            align-items: center;
            padding: 0 4px;
            gap: 2px;
            z-index: 50;
        }

        .elementor-section:hover > .elementor-section-handle,
        .elementor-section.active-element > .elementor-section-handle {
            display: flex;
        }

        .sec-handle-btn {
            width: 20px;
            height: 20px;
            background: transparent;
            border: none;
            color: #ffffff;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 11px;
            border-radius: 2px;
            transition: background 0.15s;
        }

        .sec-handle-btn:hover {
            background: rgba(0, 0, 0, 0.25);
        }

        .sec-handle-btn.sec-drag {
            cursor: grab;
        }

        .sec-handle-btn.sec-drag:active {
            cursor: grabbing;
        }

        /* Canvas Container / Column */
        .elementor-container {
            max-width: 1200px;
            margin: 0 auto;
            position: relative;
            display: flex;
            flex-wrap: wrap;
            gap: 20px;
            width: 100%;
        }

        .elementor-column {
            position: relative;
            min-height: 50px;
            flex: 1;
            min-width: 0;
        }

        /* Canvas Widget */
        .elementor-widget {
            position: relative;
            cursor: pointer;
            transition: outline 0.15s, opacity 0.2s;
        }

        .elementor-widget:hover {
            outline: 1px solid var(--e-color-blue);
        }

        .elementor-widget.active-element {
            outline: 2px solid var(--e-color-blue) !important;
        }

        .elementor-widget.is-dragging {
            opacity: 0.35;
            outline: 2px dashed var(--e-color-blue) !important;
        }

        /* Widget Corner Badge */
        .elementor-widget-handle {
            position: absolute;
            top: 0;
            right: 0;
            height: 22px;
            background: var(--e-color-blue);
            color: #ffffff;
            border-radius: 0 0 0 4px;
            display: none;
            align-items: center;
            padding: 0 2px;
            gap: 2px;
            z-index: 40;
        }

        .elementor-widget:hover > .elementor-widget-handle,
        .elementor-widget.active-element > .elementor-widget-handle {
            display: flex;
        }

        .widget-handle-btn {
            width: 18px;
            height: 18px;
            background: transparent;
            border: none;
            color: #ffffff;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 10px;
            border-radius: 2px;
            transition: background 0.15s;
        }

        .widget-handle-btn:hover {
            background: rgba(0, 0, 0, 0.25);
        }

        .widget-handle-btn.w-drag {
            cursor: grab;
        }

        .widget-handle-btn.w-drag:active {
            cursor: grabbing;
        }

        /* DRAG AND DROP INSERTION INDICATOR LINE (EXACT ELEMENTOR STYLE - FIXED VIEWPORT ALIGNED) */
        .elementor-drop-indicator {
            position: fixed !important;
            height: 4px;
            background: var(--e-color-blue);
            border-radius: 2px;
            box-shadow: 0 0 12px rgba(4, 150, 255, 0.9), 0 0 2px #0496ff;
            z-index: 999999 !important;
            pointer-events: none !important;
            display: none;
            transition: none !important;
        }

        .elementor-drop-indicator::before,
        .elementor-drop-indicator::after {
            content: '';
            position: absolute;
            top: -4px;
            width: 12px;
            height: 12px;
            border-radius: 50%;
            background: var(--e-color-blue);
            border: 2px solid #ffffff;
            box-shadow: 0 0 6px rgba(4, 150, 255, 0.8);
            box-sizing: border-box;
        }

        .elementor-drop-indicator::before {
            left: -6px;
        }

        .elementor-drop-indicator::after {
            right: -6px;
        }

        .elementor-empty-view * {
            pointer-events: none;
        }

        /* Empty Column Drop Zone */
        .elementor-empty-view {
            border: 2px dashed #cbd5e1;
            border-radius: 6px;
            padding: 28px 16px;
            text-align: center;
            color: #94a3b8;
            font-size: 12px;
            font-weight: 500;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            gap: 6px;
            transition: all 0.2s;
            margin: 8px 0;
            pointer-events: auto;
        }

        .elementor-empty-view:hover, .elementor-empty-view.drag-over {
            border-color: var(--e-color-blue);
            background: rgba(4, 150, 255, 0.08);
            color: var(--e-color-blue);
            transform: scale(1.01);
        }

        .elementor-empty-view i {
            font-size: 20px;
        }

        /* Bottom "Add New Section" Dashed Box */
        .elementor-add-section-area {
            margin: 32px auto 48px;
            max-width: 1140px;
            border: 2px dashed #cbd5e1;
            border-radius: 8px;
            padding: 36px 20px;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            background: #f8fafc;
            transition: all 0.2s;
        }

        .elementor-add-section-area.drag-over {
            border-color: var(--e-color-accent);
            background: rgba(224, 38, 96, 0.06);
        }

        .elementor-add-section-buttons {
            display: flex;
            align-items: center;
            gap: 16px;
        }

        .btn-add-section-action {
            width: 48px;
            height: 48px;
            border-radius: 50%;
            border: none;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 18px;
            color: #ffffff;
            box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
            transition: all 0.2s;
        }

        .btn-add-section-action:hover {
            transform: scale(1.08);
        }

        .btn-add-section-plus {
            background: var(--e-color-accent);
        }

        .btn-add-section-folder {
            background: #515962;
        }

        /* Structure Preset Selector */
        .elementor-select-preset {
            margin-top: 20px;
            display: none;
            flex-direction: column;
            align-items: center;
            gap: 12px;
            width: 100%;
        }

        .elementor-select-preset.open {
            display: flex;
        }

        .preset-title {
            font-size: 13px;
            font-weight: 600;
            color: #64748b;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }

        .preset-grid {
            display: flex;
            flex-wrap: wrap;
            gap: 10px;
            justify-content: center;
            max-width: 600px;
        }

        .preset-card {
            width: 80px;
            height: 54px;
            background: #ffffff;
            border: 1px solid #cbd5e1;
            border-radius: 4px;
            display: flex;
            padding: 4px;
            gap: 3px;
            cursor: pointer;
            transition: all 0.2s;
        }

        .preset-card:hover {
            border-color: var(--e-color-accent);
            box-shadow: 0 3px 8px rgba(224, 38, 96, 0.2);
        }

        .preset-col {
            background: #e2e8f0;
            height: 100%;
            border-radius: 2px;
        }

"###;
