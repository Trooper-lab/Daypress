//! Navigator tree (#elementor-navigator), Icon library, Site Settings, and Globals drawer.

pub const STYLE_MODALS_AND_GLOBALS: &str = r###"        /* ------------------------------------------------------------- */
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
        }

        /* ==========================================================================
           Elementor Theme Builder, Flexbox Container, Menu, Form & Responsive Parity
           ========================================================================== */
        #elementor-preview-responsive-wrapper {
            font-family: var(--e-global-typography-text-font-family, 'Fira Sans', 'Roboto', sans-serif);
            font-size: var(--e-global-typography-text-font-size, 15px);
            font-weight: var(--e-global-typography-text-font-weight, 300);
            line-height: 1.65;
            color: var(--e-global-color-text, #1F1C17);
        }

        #elementor-canvas {
            font-family: var(--e-global-typography-text-font-family, 'Fira Sans', 'Roboto', sans-serif);
            font-size: var(--e-global-typography-text-font-size, 15px);
            font-weight: var(--e-global-typography-text-font-weight, 300);
            line-height: 1.65;
            color: var(--e-global-color-text, #1F1C17);
        }

        #elementor-preview-responsive-wrapper p {
            margin-bottom: 1em;
        }
        #elementor-preview-responsive-wrapper p:last-child {
            margin-bottom: 0;
        }

        #elementor-preview-responsive-wrapper a {
            color: var(--e-global-color-primary, #9CC13C);
            text-decoration: none;
        }

        .np-elementor-header {
            width: 100%;
            background: #ffffff;
            position: relative;
        }

        .np-elementor-footer {
            width: 100%;
            position: relative;
        }

        .np-visual-canvas .np-container,
        .np-elementor-header .np-container,
        .np-elementor-footer .np-container {
            max-width: 100%;
            margin: 0;
            position: relative;
            gap: var(--e-widgets-space, 20px);
            background-size: cover;
            background-position: center;
            background-repeat: no-repeat;
        }

        #elementor-canvas .np-container:hover {
            outline: 1px dashed rgba(4, 150, 255, 0.55);
        }

        #elementor-canvas .np-container.active-element {
            outline: 2px solid var(--e-color-blue) !important;
        }

        .np-container.np-container-boxed {
            max-width: 100%;
            width: 100%;
        }

        .np-container-boxed > .np-container-inner {
            max-width: var(--e-content-width, 1140px);
            width: 100%;
            margin: 0 auto;
            display: flex;
            flex-direction: inherit;
            justify-content: inherit;
            align-items: inherit;
            gap: inherit;
            flex-wrap: inherit;
            min-height: inherit;
        }

        .np-container[style*="flex-direction: row"] > .np-container:not([style*="width:"]),
        .np-container[style*="flex-direction: row"] > .np-container-inner > .np-container:not([style*="width:"]) {
            flex: 1 1 0%;
            min-width: 0;
        }

        .np-button {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            gap: 8px;
            padding: 12px 24px;
            background-color: var(--e-global-color-primary, var(--e-color-accent));
            color: #ffffff;
            border-radius: 3px;
            font-family: var(--e-global-typography-accent-font-family, inherit);
            font-weight: var(--e-global-typography-accent-font-weight, 500);
            text-decoration: none;
            line-height: 1.2;
            transition: all 0.2s ease;
        }

        .np-image {
            max-width: 100%;
            height: auto;
            display: inline-block;
        }

        .np-nav-menu .np-nav-menu-list,
        .np-nav-menu .np-nav-list,
        #elementor-canvas .elementor-widget .np-nav-menu ul {
            list-style: none;
            margin: 0;
            padding: 0;
            display: flex;
            flex-wrap: wrap;
            gap: 24px;
            align-items: center;
        }

        .np-nav-menu.np-nav-align-right .np-nav-menu-list {
            justify-content: flex-end;
        }

        .np-nav-menu.np-nav-align-center .np-nav-menu-list {
            justify-content: center;
        }

        .np-nav-menu.np-nav-layout-vertical .np-nav-menu-list,
        .np-nav-menu.np-nav-layout-vertical .np-nav-list,
        #elementor-canvas .elementor-widget .np-nav-menu.np-nav-layout-vertical ul {
            flex-direction: column;
            align-items: flex-start;
            gap: 8px;
        }

        .np-nav-menu .np-nav-menu-item {
            display: inline-flex;
            align-items: center;
        }

        .np-elementor-header .np-nav-menu.np-nav-layout-horizontal .np-nav-menu-item + .np-nav-menu-item::before {
            content: "";
            display: inline-block;
            width: 1px;
            height: 15px;
            background: currentColor;
            opacity: 0.65;
            margin-right: 24px;
        }

        .np-nav-menu .np-nav-menu-link,
        .np-nav-menu .np-nav-link {
            font-family: inherit;
            font-size: inherit;
            font-weight: inherit;
            color: inherit !important;
            text-decoration: none;
        }

        .np-iconlist,
        .np-icon-list,
        #elementor-canvas .elementor-widget ul.np-iconlist {
            list-style: none;
            margin: 0;
            padding: 0;
            display: flex;
            flex-direction: column;
            gap: 8px;
        }

        .np-iconlist-item,
        .np-icon-list-item {
            display: flex;
            align-items: center;
            gap: 10px;
            line-height: 1.65;
        }

        .np-iconlist-icon,
        .np-icon-list-item i {
            color: var(--e-global-color-primary, #9CC13C);
            font-size: 16px;
            width: 18px;
            text-align: center;
            flex-shrink: 0;
            display: inline-flex;
            align-items: center;
            justify-content: center;
        }

        .np-iconlist-item a,
        .np-icon-list-link {
            display: inline-flex;
            align-items: center;
            gap: 10px;
            color: inherit !important;
            text-decoration: none;
        }

        .np-social-icons {
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 36px;
            flex-wrap: wrap;
        }

        .np-social-icon {
            display: inline-flex;
            align-items: center;
            justify-content: center;
            width: 38px;
            height: 38px;
            border-radius: 50%;
            background: var(--e-global-color-secondary, #1F1C17);
            color: #ffffff !important;
            font-size: 17px;
            text-decoration: none;
        }

        .np-elementor-form-grid {
            display: flex;
            flex-wrap: wrap;
            gap: 16px;
            width: 100%;
        }

        .np-elementor-form-grid .np-field-group {
            display: flex;
            flex-direction: column;
            gap: 6px;
        }

        .np-elementor-form-grid .np-field-w100 { width: 100%; }
        .np-elementor-form-grid .np-field-w50 { width: calc(50% - 8px); }
        .np-elementor-form-grid .np-field-w33 { width: calc(33.333% - 11px); }
        .np-elementor-form-grid .np-field-w25 { width: calc(25% - 12px); }

        .np-elementor-form-grid .np-field-label {
            font-family: var(--e-global-typography-primary-font-family, 'Fira Sans', sans-serif);
            font-weight: 500;
            font-size: 15px;
            color: var(--e-global-color-secondary, #1F1C17);
        }

        .np-elementor-form-grid .np-input,
        .np-elementor-form-grid .np-textarea,
        .np-elementor-form-grid .np-form-input {
            width: 100%;
            padding: 12px 16px;
            border: 1px solid #d4d4d8;
            border-radius: 4px;
            background: #ffffff;
            color: #1F1C17;
            font-family: inherit;
            font-size: 15px;
        }

        .np-elementor-form-grid .np-field-subgroup,
        .np-elementor-form-grid .np-Subgroup {
            display: flex;
            flex-direction: column;
            gap: 8px;
            padding: 6px 0;
        }

        .np-elementor-form-grid .np-field-subgroup.np-Subgroup-inline,
        .np-elementor-form-grid .np-Subgroup.np-Subgroup-inline {
            flex-direction: row;
            flex-wrap: wrap;
            gap: 18px;
        }

        .np-elementor-form-grid .np-option-label,
        .np-elementor-form-grid .np-checkbox-label {
            display: inline-flex;
            align-items: center;
            gap: 8px;
            font-size: 15px;
            cursor: pointer;
        }

        /* Responsive Device Switcher Visibility Parity */
        #elementor-preview:not(.mode-tablet):not(.mode-mobile) .np-hide-desktop {
            display: none !important;
        }
        #elementor-preview.mode-tablet .np-hide-tablet {
            display: none !important;
        }
        #elementor-preview.mode-mobile .np-hide-mobile {
            display: none !important;
        }
        #elementor-preview.mode-mobile .np-container[style*="flex-direction: row"],
        #elementor-preview.mode-mobile .np-container[style*="flex-direction: row"] > .np-container-inner {
            flex-wrap: wrap;
        }
        #elementor-preview.mode-mobile .np-container[style*="flex-direction: row"] > .np-container,
        #elementor-preview.mode-mobile .np-container[style*="flex-direction: row"] > .np-container-inner > .np-container {
            width: 100% !important;
            flex: 1 1 100% !important;
        }
        #elementor-preview.mode-mobile .np-elementor-form-grid .np-field-w50 {
            width: 100%;
        }"###;
