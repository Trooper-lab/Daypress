pub fn get_styles() -> &'static str {
    r###":root {
            --e-panel-width: 300px;
            --e-bg-panel: #26292c;
            --e-bg-panel-dark: #1e2124;
            --e-bg-panel-light: #33363a;
            --e-border: #33363a;
            --e-border-bold: #404449;
            --e-color-txt: #d5d8dc;
            --e-color-txt-muted: #9da5ae;
            --e-color-accent: #e02660;
            --e-color-accent-hover: #b81c4c;
            --e-color-success: #39b54a;
            --e-color-success-hover: #2e963c;
            --e-color-blue: #0496ff;
            --e-font: 'Roboto', -apple-system, BlinkMacSystemFont, "Segoe UI", Arial, sans-serif;
        }

        * {
            box-sizing: border-box;
            margin: 0;
            padding: 0;
        }

        body, html {
            height: 100%;
            width: 100%;
            overflow: hidden;
            font-family: var(--e-font);
            font-size: 13px;
            color: var(--e-color-txt);
            background: #ffffff;
            user-select: none;
        }

        /* Full App Wrapper */
        #elementor-editor-wrapper {
            display: flex;
            width: 100vw;
            height: 100vh;
            overflow: hidden;
            position: relative;
        }

        /* ------------------------------------------------------------- */
        /* LEFT SIDEBAR: #elementor-panel                                */
        /* ------------------------------------------------------------- */
        #elementor-panel {
            width: var(--e-panel-width);
            min-width: var(--e-panel-width);
            max-width: var(--e-panel-width);
            height: 100vh;
            background-color: var(--e-bg-panel);
            border-right: 1px solid var(--e-border);
            display: flex;
            flex-direction: column;
            position: relative;
            z-index: 1000;
            box-shadow: 2px 0 10px rgba(0, 0, 0, 0.25);
        }

        /* Panel Header */
        #elementor-panel-header {
            height: 44px;
            background-color: var(--e-bg-panel-dark);
            border-bottom: 1px solid var(--e-border);
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 8px;
            color: #ffffff;
        }

        .elementor-header-button {
            width: 36px;
            height: 36px;
            background: transparent;
            border: none;
            color: var(--e-color-txt-muted);
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 16px;
            border-radius: 3px;
            transition: all 0.2s;
        }

        .elementor-header-button:hover {
            color: #ffffff;
            background: rgba(255, 255, 255, 0.08);
        }

        #elementor-panel-header-title {
            font-size: 13px;
            font-weight: 700;
            letter-spacing: 0.5px;
            text-transform: uppercase;
            display: flex;
            align-items: center;
            gap: 6px;
            color: #ffffff;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
            max-width: 180px;
        }

        .e-logo {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            width: 20px;
            height: 20px;
            background: var(--e-color-accent);
            color: #ffffff;
            font-weight: 900;
            font-size: 12px;
            border-radius: 4px;
            flex-shrink: 0;
        }

        /* Panel Navigation Tabs */
        .elementor-panel-navigation {
            display: flex;
            height: 40px;
            background-color: var(--e-bg-panel-dark);
            border-bottom: 1px solid var(--e-border);
        }

        .elementor-panel-navigation-tab {
            flex: 1;
            background: transparent;
            border: none;
            border-bottom: 3px solid transparent;
            color: var(--e-color-txt-muted);
            font-size: 11px;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 6px;
            transition: all 0.2s;
        }

        .elementor-panel-navigation-tab:hover {
            color: #ffffff;
            background: rgba(255, 255, 255, 0.04);
        }

        .elementor-panel-navigation-tab.active {
            color: #ffffff;
            border-bottom-color: var(--e-color-accent);
        }

        /* Panel Content Area */
        #elementor-panel-content-wrapper {
            flex: 1;
            overflow-y: auto;
            overflow-x: hidden;
            display: flex;
            flex-direction: column;
        }

        #elementor-panel-content-wrapper::-webkit-scrollbar {
            width: 6px;
        }

        #elementor-panel-content-wrapper::-webkit-scrollbar-track {
            background: var(--e-bg-panel-dark);
        }

        #elementor-panel-content-wrapper::-webkit-scrollbar-thumb {
            background: #404449;
            border-radius: 3px;
        }

        /* Search input */
        .elementor-panel-search {
            padding: 12px 14px;
            background: var(--e-bg-panel-dark);
            border-bottom: 1px solid var(--e-border);
            position: relative;
        }

        .elementor-panel-search i {
            position: absolute;
            left: 24px;
            top: 50%;
            transform: translateY(-50%);
            color: var(--e-color-txt-muted);
            font-size: 13px;
        }

        .elementor-panel-search input {
            width: 100%;
            height: 34px;
            background: #191b1d;
            border: 1px solid var(--e-border);
            border-radius: 4px;
            color: #ffffff;
            padding: 0 12px 0 34px;
            font-size: 12px;
            outline: none;
            transition: border-color 0.2s;
        }

        .elementor-panel-search input:focus {
            border-color: var(--e-color-accent);
        }

        /* Accordion Categories */
        .elementor-panel-category {
            border-bottom: 1px solid var(--e-border);
        }

        .elementor-category-header {
            padding: 10px 14px;
            display: flex;
            align-items: center;
            justify-content: space-between;
            background: var(--e-bg-panel-dark);
            cursor: pointer;
            font-size: 11px;
            font-weight: 700;
            letter-spacing: 0.5px;
            text-transform: uppercase;
            color: #ffffff;
        }

        .elementor-category-header i {
            font-size: 10px;
            transition: transform 0.2s;
        }

        .elementor-panel-category.collapsed .elementor-category-header i {
            transform: rotate(-90deg);
        }

        .elementor-panel-category.collapsed .elementor-category-items {
            display: none;
        }

        .elementor-category-items {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 8px;
            padding: 12px;
            background: var(--e-bg-panel);
        }

        /* Widget Card (Draggable from palette) */
        .elementor-element-widget {
            background: #202225;
            border: 1px solid #33363a;
            border-radius: 4px;
            padding: 14px 6px;
            text-align: center;
            cursor: grab;
            transition: all 0.2s;
            user-select: none;
        }

        .elementor-element-widget:hover {
            background: #2c2f33;
            border-color: #555b62;
            transform: translateY(-1px);
            box-shadow: 0 4px 10px rgba(0, 0, 0, 0.3);
        }

        .elementor-element-widget:active {
            cursor: grabbing;
        }

        .elementor-element-widget .widget-icon {
            font-size: 24px;
            color: #a4afb7;
            margin-bottom: 8px;
            display: flex;
            align-items: center;
            justify-content: center;
            height: 28px;
            pointer-events: none;
        }

        .elementor-element-widget:hover .widget-icon {
            color: var(--e-color-accent);
        }

        .elementor-element-widget .widget-title {
            font-size: 11px;
            font-weight: 500;
            color: var(--e-color-txt);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            pointer-events: none;
        }

        /* Controls / Inspector Section */
        .elementor-controls-wrapper {
            padding: 0;
            display: flex;
            flex-direction: column;
        }

        .elementor-control-section {
            border-bottom: 1px solid var(--e-border);
        }

        .elementor-control-section-header {
            padding: 10px 14px;
            background: var(--e-bg-panel-dark);
            cursor: pointer;
            font-size: 11px;
            font-weight: 700;
            letter-spacing: 0.5px;
            text-transform: uppercase;
            color: #ffffff;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }

        .elementor-control-section-header i {
            font-size: 10px;
            transition: transform 0.2s;
        }

        .elementor-control-section.collapsed .elementor-control-section-header i {
            transform: rotate(-90deg);
        }

        .elementor-control-section.collapsed .elementor-control-section-body {
            display: none;
        }

        .elementor-control-section-body {
            padding: 14px;
            background: var(--e-bg-panel);
            display: flex;
            flex-direction: column;
            gap: 14px;
        }

        .elementor-control-row {
            display: flex;
            flex-direction: column;
            gap: 6px;
        }

        .elementor-control-label {
            font-size: 11px;
            font-weight: 600;
            color: var(--e-color-txt-muted);
            text-transform: uppercase;
            letter-spacing: 0.4px;
            display: flex;
            align-items: center;
            justify-content: space-between;
        }

        .elementor-input, .elementor-select, .elementor-textarea {
            width: 100%;
            background: #191b1d;
            border: 1px solid var(--e-border);
            border-radius: 4px;
            color: #ffffff;
            padding: 8px 10px;
            font-size: 12px;
            font-family: inherit;
            outline: none;
            transition: border-color 0.2s;
        }

        .elementor-input:focus, .elementor-select:focus, .elementor-textarea:focus {
            border-color: var(--e-color-accent);
        }

        .elementor-textarea {
            resize: vertical;
            min-height: 70px;
        }

        /* Color Picker Field */
        .color-picker-control {
            display: flex;
            align-items: center;
            gap: 8px;
        }

        .color-swatch {
            width: 32px;
            height: 32px;
            border-radius: 4px;
            border: 1px solid #4a4e54;
            cursor: pointer;
            padding: 0;
            background: transparent;
        }

        /* Alignment Button Group */
        .elementor-button-group {
            display: flex;
            border-radius: 4px;
            overflow: hidden;
            border: 1px solid var(--e-border);
        }

        .elementor-group-btn {
            flex: 1;
            height: 32px;
            background: #191b1d;
            border: none;
            border-right: 1px solid var(--e-border);
            color: var(--e-color-txt-muted);
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 12px;
            transition: all 0.2s;
        }

        .elementor-group-btn:last-child {
            border-right: none;
        }

        .elementor-group-btn:hover {
            background: #2a2d31;
            color: #ffffff;
        }

        .elementor-group-btn.active {
            background: var(--e-color-accent);
            color: #ffffff;
        }

        /* Linked 4-Dimension Input */
        .dimensions-control {
            display: grid;
            grid-template-columns: 1fr 1fr 1fr 1fr auto;
            gap: 4px;
            align-items: center;
        }

        .dimension-col {
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 3px;
        }

        .dimension-col input {
            width: 100%;
            height: 30px;
            text-align: center;
            padding: 0 4px;
            font-size: 11px;
            background: #191b1d;
            border: 1px solid var(--e-border);
            border-radius: 3px;
            color: #ffffff;
            outline: none;
        }

        .dimension-col input:focus {
            border-color: var(--e-color-accent);
        }

        .dimension-col span {
            font-size: 9px;
            color: var(--e-color-txt-muted);
            text-transform: uppercase;
        }

        .dimension-link-btn {
            width: 26px;
            height: 30px;
            background: #191b1d;
            border: 1px solid var(--e-border);
            border-radius: 3px;
            color: var(--e-color-txt-muted);
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 11px;
        }

        .dimension-link-btn.linked {
            color: var(--e-color-accent);
            border-color: var(--e-color-accent);
        }

        /* Icon Picker Control */
        .icon-selector-control {
            display: flex;
            align-items: center;
            gap: 10px;
        }

        .icon-preview-box {
            width: 44px;
            height: 44px;
            background: #191b1d;
            border: 1px solid var(--e-border);
            border-radius: 4px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 20px;
            color: #ffffff;
        }

        .btn-icon-library {
            flex: 1;
            height: 36px;
            background: #2a2d31;
            border: 1px solid #404449;
            border-radius: 4px;
            color: #ffffff;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 8px;
            font-size: 12px;
            font-weight: 500;
            transition: all 0.2s;
        }

        .btn-icon-library:hover {
            background: #363a3f;
            border-color: var(--e-color-accent);
        }

        /* Panel Footer Dock */
        #elementor-panel-footer {
            height: 48px;
            background-color: var(--e-bg-panel-dark);
            border-top: 1px solid var(--e-border);
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 8px;
            position: relative;
        }

        .elementor-panel-footer-tools {
            display: flex;
            align-items: center;
            gap: 4px;
        }

        .elementor-panel-footer-tool {
            width: 32px;
            height: 32px;
            background: transparent;
            border: none;
            color: var(--e-color-txt-muted);
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 14px;
            border-radius: 3px;
            transition: all 0.2s;
        }

        .elementor-panel-footer-tool:hover {
            color: #ffffff;
            background: rgba(255, 255, 255, 0.08);
        }

        .elementor-panel-footer-tool.active {
            color: var(--e-color-accent);
            background: rgba(224, 38, 96, 0.15);
        }

        /* Elementor Saver Green Button */
        .elementor-panel-footer-saver {
            display: flex;
            align-items: center;
        }

        #elementor-panel-saver-button-publish {
            height: 32px;
            background: var(--e-color-success);
            color: #ffffff;
            border: none;
            border-radius: 3px 0 0 3px;
            padding: 0 14px;
            font-size: 12px;
            font-weight: 700;
            letter-spacing: 0.5px;
            text-transform: uppercase;
            cursor: pointer;
            display: flex;
            align-items: center;
            gap: 6px;
            transition: background 0.2s;
        }

        #elementor-panel-saver-button-publish:hover {
            background: var(--e-color-success-hover);
        }

        #elementor-panel-saver-options-btn {
            height: 32px;
            background: var(--e-color-success);
            color: #ffffff;
            border: none;
            border-left: 1px solid rgba(0, 0, 0, 0.15);
            border-radius: 0 3px 3px 0;
            padding: 0 8px;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 11px;
            transition: background 0.2s;
        }

        #elementor-panel-saver-options-btn:hover {
            background: var(--e-color-success-hover);
        }

        /* Saver Options Dropup */
        #saver-options-menu {
            position: absolute;
            bottom: 54px;
            right: 8px;
            background: #1e2124;
            border: 1px solid var(--e-border);
            border-radius: 4px;
            box-shadow: 0 6px 20px rgba(0, 0, 0, 0.5);
            display: none;
            flex-direction: column;
            width: 190px;
            z-index: 2000;
            overflow: hidden;
        }

        #saver-options-menu.open {
            display: flex;
        }

        .saver-menu-item {
            padding: 10px 14px;
            color: var(--e-color-txt);
            background: transparent;
            border: none;
            text-align: left;
            cursor: pointer;
            display: flex;
            align-items: center;
            gap: 10px;
            font-size: 12px;
            transition: background 0.2s;
        }

        .saver-menu-item:hover {
            background: rgba(255, 255, 255, 0.08);
            color: #ffffff;
        }

        /* ------------------------------------------------------------- */
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

        /* ------------------------------------------------------------- */
        /* MODALS: Icon Library, Navigator, Page Settings, JSON AST      */
        /* ------------------------------------------------------------- */
        .elementor-modal-overlay {
            position: fixed;
            top: 0;
            left: 0;
            width: 100vw;
            height: 100vh;
            background: rgba(0, 0, 0, 0.65);
            backdrop-filter: blur(3px);
            z-index: 9999;
            display: none;
            align-items: center;
            justify-content: center;
        }

        .elementor-modal-overlay.open {
            display: flex;
        }

        .elementor-modal {
            background: #1e2124;
            color: #ffffff;
            border: 1px solid var(--e-border);
            border-radius: 8px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.7);
            display: flex;
            flex-direction: column;
            overflow: hidden;
            animation: modalFadeIn 0.2s ease-out;
        }

        @keyframes modalFadeIn {
            from { opacity: 0; transform: scale(0.96); }
            to { opacity: 1; transform: scale(1); }
        }

        .elementor-modal-header {
            height: 48px;
            background: #16181b;
            border-bottom: 1px solid var(--e-border);
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 16px;
        }

        .elementor-modal-title {
            font-size: 14px;
            font-weight: 700;
            letter-spacing: 0.4px;
            display: flex;
            align-items: center;
            gap: 8px;
        }

        .elementor-modal-close {
            background: transparent;
            border: none;
            color: var(--e-color-txt-muted);
            font-size: 16px;
            cursor: pointer;
            padding: 4px;
        }

        .elementor-modal-close:hover {
            color: #ffffff;
        }

        .elementor-modal-body {
            padding: 16px;
            overflow-y: auto;
            flex: 1;
        }

        .elementor-modal-footer {
            height: 52px;
            background: #16181b;
            border-top: 1px solid var(--e-border);
            display: flex;
            align-items: center;
            justify-content: flex-end;
            gap: 10px;
            padding: 0 16px;
        }

        /* Icon Library Specific Styles */
        .icon-modal-layout {
            width: 860px;
            max-width: 90vw;
            height: 640px;
            max-height: 90vh;
        }

        .icon-filter-bar {
            display: flex;
            align-items: center;
            gap: 12px;
            margin-bottom: 14px;
        }

        .icon-category-tabs {
            display: flex;
            gap: 4px;
            overflow-x: auto;
            flex: 1;
        }

        .icon-tab {
            background: #282b30;
            border: 1px solid #3c4046;
            color: var(--e-color-txt-muted);
            padding: 6px 12px;
            border-radius: 4px;
            font-size: 11px;
            font-weight: 600;
            cursor: pointer;
            white-space: nowrap;
            transition: all 0.2s;
        }

        .icon-tab:hover {
            color: #ffffff;
            background: #33373d;
        }

        .icon-tab.active {
            background: var(--e-color-accent);
            border-color: var(--e-color-accent);
            color: #ffffff;
        }

        .icon-search-input {
            width: 240px;
            height: 32px;
            background: #141618;
            border: 1px solid var(--e-border);
            border-radius: 4px;
            padding: 0 10px;
            font-size: 12px;
            color: #ffffff;
            outline: none;
        }

        .icon-search-input:focus {
            border-color: var(--e-color-accent);
        }

        .icon-grid {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(88px, 1fr));
            gap: 10px;
            max-height: 460px;
            overflow-y: auto;
            padding: 4px;
        }

        .icon-card {
            background: #25282c;
            border: 1px solid #36393f;
            border-radius: 6px;
            padding: 12px 6px;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            gap: 8px;
            cursor: pointer;
            transition: all 0.2s;
        }

        .icon-card:hover {
            background: #2f3338;
            border-color: #555a62;
            transform: translateY(-2px);
        }

        .icon-card.selected {
            border-color: var(--e-color-accent);
            background: rgba(224, 38, 96, 0.15);
        }

        .icon-card i {
            font-size: 24px;
            color: #ffffff;
        }

        .icon-card span {
            font-size: 10px;
            color: var(--e-color-txt-muted);
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            max-width: 100%;
        }

        /* ------------------------------------------------------------- */
        /* STRUCTURE / LAYERS NAVIGATOR PANEL (#elementor-navigator)      */
        /* ------------------------------------------------------------- */
        #elementor-navigator {
            position: fixed;
            top: 60px;
            right: 24px;
            width: 320px;
            max-height: calc(100vh - 120px);
            background: #1e2124;
            border: 1px solid #3c4046;
            border-radius: 8px;
            box-shadow: 0 16px 40px rgba(0, 0, 0, 0.55);
            z-index: 1500;
            display: none;
            flex-direction: column;
            overflow: hidden;
        }

        #elementor-navigator.open {
            display: flex;
        }

        .navigator-header {
            height: 42px;
            background: #16181b;
            border-bottom: 1px solid var(--e-border);
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 0 12px;
            font-size: 12px;
            font-weight: 700;
            color: #ffffff;
            cursor: move;
        }

        .navigator-header-title {
            display: flex;
            align-items: center;
            gap: 8px;
        }

        .navigator-count-badge {
            font-size: 10px;
            font-weight: 700;
            background: rgba(255, 255, 255, 0.1);
            color: var(--e-color-txt-muted);
            padding: 2px 6px;
            border-radius: 10px;
        }

        .navigator-header-actions {
            display: flex;
            align-items: center;
            gap: 6px;
        }

        .nav-btn {
            background: transparent;
            border: none;
            color: var(--e-color-txt-muted);
            cursor: pointer;
            width: 26px;
            height: 26px;
            border-radius: 4px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 12px;
            transition: all 0.15s;
        }

        .nav-btn:hover {
            background: rgba(255, 255, 255, 0.1);
            color: #ffffff;
        }

        .navigator-filter-bar {
            padding: 8px 10px;
            background: #191b1d;
            border-bottom: 1px solid var(--e-border);
        }

        .navigator-filter-bar input {
            width: 100%;
            height: 28px;
            background: #121315;
            border: 1px solid #33363a;
            border-radius: 4px;
            color: #ffffff;
            font-size: 11px;
            padding: 0 8px;
            outline: none;
        }

        .navigator-filter-bar input:focus {
            border-color: var(--e-color-accent);
        }

        .navigator-tree {
            flex: 1;
            overflow-y: auto;
            padding: 6px 0;
            max-height: 480px;
        }

        .nav-tree-item {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 6px 12px;
            font-size: 12px;
            color: var(--e-color-txt);
            cursor: pointer;
            transition: background 0.15s;
            user-select: none;
            border-left: 3px solid transparent;
        }

        .nav-tree-item:hover {
            background: rgba(255, 255, 255, 0.05);
            color: #ffffff;
        }

        .nav-tree-item.active {
            background: rgba(4, 150, 255, 0.15);
            color: #ffffff;
            border-left-color: var(--e-color-blue);
        }

        .nav-tree-item.is-dragging {
            opacity: 0.4;
            background: #2f3338;
        }

        .nav-tree-item.drag-over-item {
            border-top: 2px solid var(--e-color-blue);
        }

        .nav-item-left {
            display: flex;
            align-items: center;
            gap: 8px;
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
            flex: 1;
        }

        .nav-caret {
            font-size: 10px;
            color: #64748b;
            width: 14px;
            text-align: center;
            transition: transform 0.2s;
        }

        .nav-caret.collapsed {
            transform: rotate(-90deg);
        }

        .nav-item-icon {
            font-size: 12px;
            color: #889098;
            width: 16px;
            text-align: center;
        }

        .nav-item-title {
            overflow: hidden;
            text-overflow: ellipsis;
            white-space: nowrap;
            font-size: 11.5px;
            font-weight: 500;
        }

        .nav-item-actions {
            display: flex;
            align-items: center;
            gap: 4px;
            opacity: 0;
            transition: opacity 0.15s;
        }

        .nav-tree-item:hover .nav-item-actions,
        .nav-tree-item.active .nav-item-actions {
            opacity: 1;
        }

        .nav-action-btn {
            background: transparent;
            border: none;
            color: var(--e-color-txt-muted);
            cursor: pointer;
            width: 20px;
            height: 20px;
            border-radius: 3px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 10.5px;
            transition: all 0.15s;
        }

        .nav-action-btn:hover {
            background: rgba(255, 255, 255, 0.15);
            color: #ffffff;
        }

        .nav-action-btn.del:hover {
            color: #f87171;
        }

        /* Toast notification */
        #elementor-toast {
            position: fixed;
            bottom: 24px;
            right: 24px;
            background: #1e2124;
            border: 1px solid #33363a;
            color: #ffffff;
            padding: 12px 20px;
            border-radius: 6px;
            box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4);
            display: none;
            align-items: center;
            gap: 10px;
            z-index: 10000;
            font-size: 13px;
            font-weight: 500;
        }

        #elementor-toast.show {
            display: flex;
            animation: toastIn 0.25s ease-out;
        }

                @keyframes toastIn {
            from { opacity: 0; transform: translateY(10px); }
            to { opacity: 1; transform: translateY(0); }
        }

        /* ============================================================== */
        /* HAMBURGER MENU & SITE SETTINGS STYLING (Elementor 1:1 Parity) */
        /* ============================================================== */
        .elementor-panel-menu {
            display: flex;
            flex-direction: column;
            padding: 8px 0;
            overflow-y: auto;
            background: #26292c;
            height: calc(100vh - 104px);
        }

        .elementor-panel-menu-group {
            margin-bottom: 8px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.05);
            padding-bottom: 8px;
        }

        .elementor-panel-menu-group:last-child {
            border-bottom: none;
        }

        .elementor-panel-menu-group-title {
            padding: 12px 18px 6px;
            font-size: 11px;
            font-weight: 700;
            color: #818a91;
            text-transform: uppercase;
            letter-spacing: 0.6px;
        }

        .elementor-panel-menu-items {
            display: flex;
            flex-direction: column;
        }

        .elementor-panel-menu-item {
            display: flex;
            align-items: center;
            gap: 14px;
            padding: 12px 18px;
            color: #d5dade;
            text-decoration: none !important;
            cursor: pointer;
            transition: all 0.15s ease-in-out;
            border-left: 3px solid transparent;
            font-size: 13px;
            font-weight: 500;
            background: transparent;
        }

        .elementor-panel-menu-item:visited,
        .elementor-panel-menu-item:link,
        .elementor-panel-menu-item:active {
            color: #d5dade !important;
            text-decoration: none !important;
        }

        .elementor-panel-menu-item:hover {
            background-color: #1e2124 !important;
            color: #ffffff !important;
            border-left-color: var(--e-color-accent);
        }

        .elementor-panel-menu-item-icon {
            width: 22px;
            height: 22px;
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 15px;
            color: #9da5ae;
            flex-shrink: 0;
            transition: color 0.15s;
        }

        .elementor-panel-menu-item:hover .elementor-panel-menu-item-icon {
            color: var(--e-color-accent);
        }

        .elementor-panel-menu-item-title {
            flex: 1;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
            color: inherit;
        }

        /* Sub-Panel Header & Breadcrumb */
        .elementor-subpanel-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 10px 14px;
            background: #1e2124;
            border-bottom: 1px solid var(--e-border);
        }

        .elementor-back-btn {
            background: transparent;
            border: none;
            color: #9da5ae;
            font-size: 11px;
            font-weight: 700;
            text-transform: uppercase;
            cursor: pointer;
            display: flex;
            align-items: center;
            gap: 6px;
            padding: 5px 8px;
            border-radius: 3px;
            transition: all 0.2s;
        }

        .elementor-back-btn:hover {
            color: #ffffff;
            background: rgba(255, 255, 255, 0.08);
        }

        .elementor-subpanel-body {
            padding: 16px;
            display: flex;
            flex-direction: column;
            gap: 16px;
            overflow-y: auto;
            background: #26292c;
            max-height: calc(100vh - 110px);
        }

        /* Color Token Row */
        .elementor-color-token-row {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 10px 14px;
            background: #1e2124;
            border: 1px solid var(--e-border);
            border-radius: 6px;
            gap: 12px;
            transition: border-color 0.15s;
        }

        .elementor-color-token-row:hover {
            border-color: #4a4e54;
        }

        .color-token-meta {
            display: flex;
            align-items: center;
            gap: 12px;
            flex: 1;
        }

        .color-token-preview {
            width: 30px;
            height: 30px;
            border-radius: 5px;
            border: 1px solid rgba(255, 255, 255, 0.25);
            cursor: pointer;
            box-shadow: 0 2px 6px rgba(0,0,0,0.35);
            flex-shrink: 0;
        }

        .color-token-info {
            display: flex;
            flex-direction: column;
            gap: 2px;
        }

        .color-token-title {
            font-size: 12px;
            font-weight: 700;
            color: #ffffff;
        }

        .color-token-code {
            font-size: 11px;
            color: #9da5ae;
            font-family: monospace;
        }

        .color-token-input-wrapper input[type="color"] {
            width: 32px;
            height: 32px;
            border: none;
            background: transparent;
            cursor: pointer;
            padding: 0;
            border-radius: 4px;
        }

        /* ============================================================== */
        /* GLOBALS TAB DRAWER STYLES                                      */
        /* ============================================================== */
        .elementor-globals-panel {
            padding: 16px;
            display: flex;
            flex-direction: column;
            gap: 18px;
            overflow-y: auto;
            background: #26292c;
            height: calc(100vh - 104px);
        }

        .globals-section-header {
            display: flex;
            align-items: center;
            justify-content: space-between;
            font-size: 11px;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            color: #818a91;
            margin-bottom: 8px;
            border-bottom: 1px solid rgba(255, 255, 255, 0.06);
            padding-bottom: 6px;
        }

        .globals-manage-pill {
            display: inline-flex;
            align-items: center;
            gap: 4px;
            background: rgba(224, 38, 96, 0.15);
            color: var(--e-color-accent);
            border: 1px solid rgba(224, 38, 96, 0.3);
            padding: 2px 8px;
            border-radius: 12px;
            font-size: 10px;
            font-weight: 700;
            text-transform: uppercase;
            cursor: pointer;
            transition: all 0.2s;
        }

        .globals-manage-pill:hover {
            background: var(--e-color-accent);
            color: #ffffff;
        }

        .globals-colors-grid {
            display: grid;
            grid-template-columns: repeat(2, 1fr);
            gap: 8px;
        }

        .globals-color-chip {
            display: flex;
            align-items: center;
            gap: 10px;
            background: #1e2124;
            border: 1px solid var(--e-border);
            border-radius: 6px;
            padding: 8px 10px;
            cursor: pointer;
            transition: all 0.15s;
        }

        .globals-color-chip:hover {
            border-color: var(--e-color-accent);
            background: #272a2e;
            transform: translateY(-1px);
        }

        .globals-chip-swatch {
            width: 22px;
            height: 22px;
            border-radius: 4px;
            border: 1px solid rgba(255, 255, 255, 0.25);
            flex-shrink: 0;
            box-shadow: 0 2px 4px rgba(0,0,0,0.3);
        }

        .globals-chip-text {
            display: flex;
            flex-direction: column;
            overflow: hidden;
            gap: 1px;
        }

        .globals-chip-name {
            font-size: 11px;
            font-weight: 700;
            color: #ffffff;
            white-space: nowrap;
            overflow: hidden;
            text-overflow: ellipsis;
        }

        .globals-chip-hex {
            font-size: 10px;
            color: #9da5ae;
            font-family: monospace;
        }

        .globals-typography-list {
            display: flex;
            flex-direction: column;
            gap: 6px;
        }

        .globals-type-item {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 9px 12px;
            background: #1e2124;
            border: 1px solid var(--e-border);
            border-radius: 6px;
            font-size: 12px;
            transition: border-color 0.15s;
        }

        .globals-type-item:hover {
            border-color: #4a4e54;
        }

        .globals-type-name {
            font-weight: 700;
            color: #ffffff;
        }

        .globals-type-spec {
            font-size: 11px;
            color: #9da5ae;
        }

        .elementor-nerd-box {
            background: rgba(255, 255, 255, 0.02);
            border: 1px dashed rgba(255, 255, 255, 0.14);
            border-radius: 6px;
            padding: 16px;
            text-align: center;
            display: flex;
            flex-direction: column;
            align-items: center;
            gap: 8px;
        }

        .elementor-nerd-box-icon {
            width: 36px;
            height: 36px;
            border-radius: 50%;
            background: rgba(224, 8, 86, 0.15);
            color: var(--e-color-accent);
            display: flex;
            align-items: center;
            justify-content: center;
            font-size: 16px;
        }

        .elementor-nerd-box-title {
            font-size: 13px;
            font-weight: 700;
            color: #ffffff;
        }

        .elementor-nerd-box-message {
            font-size: 11px;
            line-height: 1.5;
            color: #9da5ae;
        }

        .btn-elementor-action {
            width: 100%;
            padding: 10px 16px;
            background: var(--e-color-accent);
            border: none;
            border-radius: 4px;
            color: #ffffff;
            font-size: 12px;
            font-weight: 700;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            cursor: pointer;
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 8px;
            transition: all 0.2s;
        }

        .btn-elementor-action:hover {
            background: #c41b4e;
            box-shadow: 0 4px 12px rgba(224, 38, 96, 0.3);
        }

        .reusable-blocks-grid {
            display: flex;
            flex-direction: column;
            gap: 8px;
        }

        .reusable-block-card {
            display: flex;
            align-items: center;
            gap: 12px;
            padding: 10px 12px;
            background: #1e2124;
            border: 1px solid var(--e-border);
            border-radius: 6px;
            cursor: grab;
            transition: all 0.2s;
        }

        .reusable-block-card:hover {
            border-color: var(--e-color-accent);
            background: #272a2e;
            transform: translateX(3px);
        }

        .reusable-block-icon {
            width: 34px;
            height: 34px;
            border-radius: 6px;
            background: rgba(224, 38, 96, 0.12);
            display: flex;
            align-items: center;
            justify-content: center;
            color: var(--e-color-accent);
            font-size: 15px;
            flex-shrink: 0;
        }

        .reusable-block-info {
            display: flex;
            flex-direction: column;
            gap: 2px;
        }

        .reusable-block-title {
            font-size: 12px;
            font-weight: 700;
            color: #ffffff;
        }

                .reusable-block-desc {
            font-size: 11px;
            color: #818a91;
        }

        /* Modals & Shortcuts */
        kbd {
            background: #2a2d32;
            border: 1px solid #454950;
            color: #d5dade;
            padding: 2px 7px;
            border-radius: 4px;
            font-size: 11px;
            font-family: inherit;
            box-shadow: 0 1px 2px rgba(0,0,0,0.3);
        }
        .shortcut-row {
            display: flex;
            align-items: center;
            justify-content: space-between;
            padding: 8px 12px;
            background: #191b1d;
            border: 1px solid var(--e-border);
            border-radius: 5px;
            font-size: 12px;
            margin-bottom: 6px;
        }
        .finder-item {
            display: flex;
            align-items: center;
            gap: 12px;
            padding: 10px 14px;
            border-radius: 5px;
            color: #d5dade;
            cursor: pointer;
            transition: all 0.15s;
        }
        .finder-item:hover, .finder-item.active {
            background: var(--e-color-accent);
            color: #ffffff;
        }
        .finder-item-icon {
            width: 22px;
            text-align: center;
            font-size: 14px;
            color: #9da5ae;
        }
        .finder-item:hover .finder-item-icon {
            color: #ffffff;
        }
        .finder-item-title {
            flex: 1;
            font-size: 13px;
            font-weight: 600;
        }
        .finder-item-cat {
            font-size: 10px;
            text-transform: uppercase;
            letter-spacing: 0.5px;
            opacity: 0.7;
        }

        /* Theme toggle support */
        #elementor-editor-wrapper.theme-light {
            --e-bg-panel: #f1f3f5;
            --e-bg-panel-dark: #e9ecef;
            --e-border: #ced4da;
            --e-color-txt: #212529;
            --e-color-txt-muted: #6c757d;
        }
        #elementor-editor-wrapper.theme-light #elementor-panel,
        #elementor-editor-wrapper.theme-light .elementor-panel-menu,
        #elementor-editor-wrapper.theme-light .elementor-globals-panel,
        #elementor-editor-wrapper.theme-light .elementor-subpanel-body {
            background: #f8f9fa !important;
            color: #212529 !important;
        }
        #elementor-editor-wrapper.theme-light .elementor-panel-menu-item,
        #elementor-editor-wrapper.theme-light .globals-color-chip,
        #elementor-editor-wrapper.theme-light .globals-type-item,
        #elementor-editor-wrapper.theme-light .reusable-block-card,
        #elementor-editor-wrapper.theme-light .elementor-color-token-row {
            background: #ffffff !important;
            border-color: #dee2e6 !important;
            color: #212529 !important;
        }
        #elementor-editor-wrapper.theme-light .globals-chip-name,
        #elementor-editor-wrapper.theme-light .globals-type-name,
        #elementor-editor-wrapper.theme-light .reusable-block-title,
        #elementor-editor-wrapper.theme-light .color-token-title {
            color: #212529 !important;
        }

        /* Gutenberg / Content Typography inside Studio Canvas */
        #elementor-canvas pre,
        #elementor-canvas .wp-block-code {
            background: #0f172a;
            color: #e2e8f0;
            padding: 18px 20px;
            border-radius: 8px;
            overflow-x: auto;
            font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
            font-size: 13px;
            line-height: 1.6;
            border: 1px solid #1e293b;
            margin: 12px 0;
        }
        #elementor-canvas blockquote,
        #elementor-canvas .wp-block-quote {
            border-left: 4px solid var(--e-color-accent);
            background: rgba(148, 163, 184, 0.08);
            padding: 14px 20px;
            border-radius: 0 8px 8px 0;
            font-style: italic;
            margin: 12px 0;
        }
        #elementor-canvas .elementor-widget ul,
        #elementor-canvas .elementor-widget ol {
            padding-left: 22px;
            line-height: 1.7;
        }"###
}
