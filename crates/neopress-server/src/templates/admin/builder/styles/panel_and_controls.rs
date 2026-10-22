//! Left inspector sidebar (#elementor-panel), palette cards, and form controls.

pub const STYLE_PANEL_AND_CONTROLS: &str = r###":root {
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

"###;
