//! Admin CSS styles: brand tokens, fonts, the desk sidebar and the main column.
//!
//! Tokens follow the Daypress design system (Daylight theme): paper and ink do most of the
//! work, oxide is the brand accent, press blue is the product's working colour.

/// The brand typefaces, served from `/static/fonts/` (SIL OFL, see `static/fonts/OFL.txt`).
pub const DESK_FONT_FACES: &str = r#"
		@font-face { font-family: "Newsreader"; src: url("/static/fonts/Newsreader-Variable.woff2") format("woff2"); font-weight: 300 700; font-style: normal; font-display: swap; }
		@font-face { font-family: "Schibsted Grotesk"; src: url("/static/fonts/SchibstedGrotesk-Variable.woff2") format("woff2"); font-weight: 400 900; font-style: normal; font-display: swap; }
		@font-face { font-family: "IBM Plex Mono"; src: url("/static/fonts/IBMPlexMono-Regular.woff2") format("woff2"); font-weight: 400; font-style: normal; font-display: swap; }
		@font-face { font-family: "IBM Plex Mono"; src: url("/static/fonts/IBMPlexMono-Medium.woff2") format("woff2"); font-weight: 500; font-style: normal; font-display: swap; }
"#;

/// Tokens and element defaults shared by the desk and the login screens.
pub const DESK_FOUNDATION: &str = r#"
		:root {
			/* WordPress-authentic neutral cool grey canvas + Daypress signature terracotta orange accent */
			--paper: #f0f0f1;
			--paper-raised: #ffffff;
			--paper-sunk: #f6f7f7;
			--ink: #1d2327;
			--ink-muted: #50575e;
			--rule: #dcdcde;
			--rule-strong: #c3c4c7;
			--oxide: #b8431b;
			--on-oxide: #ffffff;
			--press: #2271b1;
			--press-hover: #135e96;
			--on-press: #ffffff;
			--moss: #1e7e34;
			--ochre: #92400e;
			--on-ochre: #ffffff;
			--danger: #b32d2e;
			--focus: #2271b1;
			--selection: #b8431b24;

			/* Pale semantic washes behind status pills and notices */
			--oxide-wash: #fdf2ee;
			--moss-wash: #edfaef;
			--ochre-wash: #fcf9e8;
			--press-wash: #f0f6fc;

			/* WordPress dark slate chrome (#1d2327) with Daypress warm orange active accent */
			--desk: #1d2327;
			--desk-raised: #2c3338;
			--desk-ink: #f0f0f1;
			--desk-muted: #c3c4c7;
			--desk-rule: #2c3338;
			--desk-oxide: #e8744a;
			--desk-text-active: #e8744a;

			/* Strict sans-serif UI typography (--font-logo reserved exclusively for the daypress wordmark) */
			--font-sans: "Schibsted Grotesk", -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen-Sans, Ubuntu, Cantarell, "Helvetica Neue", sans-serif;
			--font-serif: var(--font-sans);
			--font-logo: "Newsreader", Georgia, serif;
			--font-mono: "IBM Plex Mono", ui-monospace, Menlo, Consolas, monospace;

			--radius-none: 0px;
			--radius-sm: 3px;
			--radius-md: 4px;
			--radius-pill: 999px;

			--space-1: 4px;
			--space-2: 8px;
			--space-3: 12px;
			--space-4: 16px;
			--space-6: 24px;
			--space-8: 32px;
			--space-12: 48px;

			/* Older names still used across desk screens. */
			--bg-app: var(--paper);
			--bg-surface: var(--paper-raised);
			--bg-surface-alt: var(--paper-sunk);
			--border-subtle: var(--rule);
			--border-hover: var(--rule-strong);
			--text-main: var(--ink);
			--text-muted: var(--ink-muted);
			--text-dim: var(--ink-muted);
			--primary: var(--oxide);
			--primary-hover: #8f3212;

			--sidebar-w: 176px;
			--topbar-h: 32px;
		}

		.sr-only {
			position: absolute;
			width: 1px;
			height: 1px;
			padding: 0;
			margin: -1px;
			overflow: hidden;
			clip: rect(0, 0, 0, 0);
			white-space: nowrap;
			border: 0;
		}

		*, *::before, *::after {
			box-sizing: border-box;
			margin: 0;
			padding: 0;
		}

		body {
			font-family: var(--font-sans);
			background: var(--paper);
			color: var(--ink);
			font-size: 13.5px;
			line-height: 1.5;
			font-variant-numeric: tabular-nums;
			-webkit-font-smoothing: antialiased;
		}

		::selection { background: var(--selection); }

		:where(a, button, input, select, textarea, summary, [tabindex]):focus-visible {
			outline: 2px solid var(--focus);
			outline-offset: 1px;
		}

		a { color: var(--press); }

		code, kbd, pre, samp { font-family: var(--font-mono); }

		#day-canvas :where(h1, h2, h3, h4) {
			font-family: var(--font-sans);
			font-weight: 600;
			letter-spacing: -0.01em;
		}

		@media (prefers-reduced-motion: reduce) {
			*, *::before, *::after { transition: none !important; animation: none !important; }
		}
"#;

/// The desk layout: top admin bar, left sidebar, screen-meta tabs, and main column.
pub fn get_styles_shell() -> &'static str {
    r#"
		#day-app {
			display: flex;
			min-height: 100vh;
			padding-top: var(--topbar-h);
		}

		/* ==================== FULL-WIDTH TOP ADMIN BAR (WordPress #wpadminbar) ==================== */
		#day-adminbar {
			position: fixed;
			top: 0;
			left: 0;
			right: 0;
			height: var(--topbar-h);
			min-height: var(--topbar-h);
			background: var(--desk);
			color: var(--desk-muted);
			z-index: 99999;
			display: flex;
			align-items: stretch;
			justify-content: space-between;
			padding: 0;
			font: 400 13px/32px var(--font-sans);
			user-select: none;
		}

		.ab-top-menu,
		.ab-top-secondary {
			list-style: none;
			margin: 0;
			padding: 0;
			display: flex;
			align-items: stretch;
		}

		.ab-top-menu > li,
		.ab-top-secondary > li {
			position: relative;
			display: flex;
			align-items: stretch;
		}

		.ab-item {
			display: inline-flex;
			align-items: center;
			gap: 6px;
			padding: 0 10px;
			height: var(--topbar-h);
			color: var(--desk-ink);
			text-decoration: none;
			font: 400 13px/32px var(--font-sans);
			background: transparent;
			border: 0;
			cursor: pointer;
			white-space: nowrap;
			transition: background-color 0.1s ease, color 0.1s ease;
		}

		.ab-top-menu > li:hover > .ab-item,
		.ab-top-secondary > li:hover > .ab-item,
		.ab-item:hover,
		.ab-item:focus-visible {
			background: var(--desk-raised);
			color: var(--desk-oxide);
		}

		.ab-brand-icon {
			width: 20px;
			height: 20px;
			display: block;
			flex-shrink: 0;
			color: var(--desk-ink);
			transition: color 0.1s ease;
		}

		.ab-top-menu > li:hover > .ab-item .ab-brand-icon,
		.ab-item:hover .ab-brand-icon {
			color: var(--desk-ink);
		}

		.ab-icon {
			width: 15px;
			height: 15px;
			flex-shrink: 0;
			color: var(--desk-muted);
			transition: color 0.1s ease;
		}

		.ab-top-menu > li:hover > .ab-item .ab-icon,
		.ab-top-secondary > li:hover > .ab-item .ab-icon,
		.ab-item:hover .ab-icon {
			color: var(--desk-oxide);
		}

		.ab-kbd {
			font: 500 11px/14px var(--font-mono);
			color: var(--desk-muted);
			background: rgba(128, 128, 128, 0.15);
			padding: 1px 4px;
			border-radius: 2px;
			margin-left: 2px;
		}

		.ab-badge-pill {
			display: inline-flex;
			align-items: center;
			justify-content: center;
			min-width: 17px;
			height: 17px;
			padding: 0 5px;
			border-radius: 999px;
			background: var(--oxide);
			color: #ffffff;
			font: 600 11px/17px var(--font-sans);
		}

		/* Right-hand account item: Howdy, {user} + square avatar */
		#wp-admin-bar-my-account > .ab-item {
			padding: 0 12px;
			gap: 7px;
		}

		#wp-admin-bar-my-account .display-name {
			font-weight: 500;
		}

		.ab-avatar {
			width: 18px;
			height: 18px;
			background: #50575e;
			border: 1px solid #8c8f94;
			border-radius: 2px;
			color: #f0f0f1;
			display: inline-flex;
			align-items: center;
			justify-content: center;
			flex-shrink: 0;
		}

		/* Topbar flyout dropdowns (.ab-sub-wrapper) */
		.ab-sub-wrapper {
			display: none;
			position: absolute;
			top: var(--topbar-h);
			left: 0;
			min-width: 180px;
			background: var(--desk-raised);
			box-shadow: 0 3px 5px rgba(0, 0, 0, 0.2);
			padding: 6px 0;
			z-index: 100000;
		}

		.ab-top-secondary .ab-sub-wrapper {
			left: auto;
			right: 0;
			min-width: 240px;
		}

		.menupop:hover > .ab-sub-wrapper,
		.menupop:focus-within > .ab-sub-wrapper {
			display: block;
		}

		.ab-submenu {
			list-style: none;
			margin: 0;
			padding: 0;
		}

		.ab-submenu li a {
			display: block;
			padding: 5px 14px;
			font: 400 13px/20px var(--font-sans);
			color: var(--desk-muted);
			text-decoration: none;
			white-space: nowrap;
		}

		.ab-submenu li a:hover {
			color: var(--desk-oxide);
			background: rgba(255, 255, 255, 0.04);
		}

		.ab-user-card {
			display: flex;
			align-items: center;
			gap: 12px;
			padding: 10px 14px 12px;
			border-bottom: 1px solid rgba(255, 255, 255, 0.08);
			margin-bottom: 4px;
		}

		.ab-user-avatar-lg {
			width: 46px;
			height: 46px;
			border-radius: 3px;
			background: #50575e;
			border: 1px solid #8c8f94;
			color: #f0f0f1;
			display: flex;
			align-items: center;
			justify-content: center;
			flex-shrink: 0;
		}

		.ab-user-meta {
			min-width: 0;
			line-height: 1.35;
		}

		.ab-user-meta strong {
			display: block;
			color: var(--desk-ink);
			font: 600 13.5px/18px var(--font-sans);
			overflow: hidden;
			text-overflow: ellipsis;
		}

		.ab-user-meta span {
			display: block;
			color: var(--desk-muted);
			font: 400 12px/16px var(--font-sans);
		}

		.adminbar-menu-btn {
			display: none;
			align-items: center;
			justify-content: center;
			gap: 6px;
			padding: 0 10px;
			height: var(--topbar-h);
			background: transparent;
			border: 0;
			color: var(--desk-ink);
			font: 500 13px/32px var(--font-sans);
			cursor: pointer;
		}

		/* ==================== DESK SIDEBAR (WordPress #adminmenuwrap) ==================== */
		#day-sidebar {
			width: var(--sidebar-w);
			background: var(--desk);
			color: var(--desk-ink);
			position: fixed;
			top: var(--topbar-h);
			bottom: 0;
			left: 0;
			display: flex;
			flex-direction: column;
			z-index: 9990;
			overflow-y: auto;
			overflow-x: hidden;
			transition: width 0.12s ease;
		}

		.sidebar-nav {
			flex: 1;
			padding: 8px 0 12px;
		}

		.nav-section-title {
			font: 600 12px/16px var(--font-sans);
			letter-spacing: 0;
			text-transform: none;
			color: var(--desk-muted);
			padding: 10px 12px 4px;
		}

		.nav-item {
			display: flex;
			align-items: center;
			gap: 9px;
			padding: 7px 12px;
			text-decoration: none;
			color: var(--desk-ink);
			font: 400 13.5px/18px var(--font-sans);
			position: relative;
			transition: background 0.1s ease, color 0.1s ease;
		}

		.nav-item:hover {
			background: var(--desk-raised);
			color: var(--desk-oxide);
		}

		.nav-item:hover::before {
			content: "";
			position: absolute;
			left: 0;
			top: 0;
			bottom: 0;
			width: 4px;
			background: var(--desk-oxide);
		}

		.nav-item.active {
			background: var(--oxide);
			color: #ffffff;
			font-weight: 600;
		}

		.nav-item.active::before {
			display: none;
		}

		/* WordPress signature right-edge notch pointing from active menu item into canvas */
		.nav-item.active::after {
			content: "";
			position: absolute;
			right: 0;
			top: 50%;
			transform: translateY(-50%);
			width: 0;
			height: 0;
			border: 7px solid transparent;
			border-right-color: var(--paper);
			pointer-events: none;
		}

		/* Sidebar navigation icons */
		.nav-icon {
			width: 16px;
			height: 16px;
			flex-shrink: 0;
			color: var(--desk-muted);
			opacity: 0.95;
			transition: color 0.1s, opacity 0.1s;
		}

		.nav-item:hover .nav-icon {
			color: var(--desk-oxide);
			opacity: 1;
		}

		.nav-item.active .nav-icon {
			color: #ffffff;
			opacity: 1;
		}

		.nav-badge {
			margin-left: auto;
			font: 600 11px/16px var(--font-sans);
			font-variant-numeric: tabular-nums;
			background: var(--oxide);
			color: #ffffff;
			padding: 0 6px;
			border-radius: 999px;
		}

		.nav-item.active .nav-badge {
			background: rgba(0, 0, 0, 0.25);
			color: #ffffff;
		}

		/* Submenus: inline under the open section, a flyout on hover for the others. */
		.nav-group {
			position: relative;
		}

		.nav-submenu {
			display: none;
			flex-direction: column;
			background: var(--desk-raised);
		}

		.nav-group.open > .nav-submenu {
			display: flex;
			position: static;
			padding: 5px 0 6px;
		}

		.nav-group:not(.open):hover > .nav-submenu {
			display: flex;
			position: fixed;
			left: var(--sidebar-w);
			top: 120px;
			min-width: 180px;
			padding: 6px 0;
			background: var(--desk-raised);
			box-shadow: 2px 3px 6px rgba(0, 0, 0, 0.22);
			z-index: 99999;
		}

		.nav-group:not(.open):hover > .nav-submenu::before {
			content: "";
			position: absolute;
			left: -14px;
			top: 0;
			bottom: 0;
			width: 14px;
		}

		.nav-subitem {
			display: block;
			padding: 5px 12px 5px 38px;
			font: 400 13px/18px var(--font-sans);
			color: var(--desk-muted);
			text-decoration: none;
			white-space: nowrap;
			transition: color 0.1s ease;
		}

		.nav-group:not(.open) .nav-subitem {
			padding-left: 14px;
		}

		.nav-subitem:hover {
			color: var(--desk-oxide);
		}

		.nav-subitem.active {
			color: var(--desk-ink);
			font-weight: 600;
		}

		/* ==================== COLLAPSIBLE SIDEBAR (WordPress #collapse-menu) ==================== */
		.desk-collapse-btn {
			display: flex;
			align-items: center;
			gap: 9px;
			width: 100%;
			margin-top: 6px;
			padding: 8px 12px;
			background: transparent;
			border: 0;
			border-top: 1px solid var(--desk-rule);
			color: var(--desk-muted);
			font: 400 13px/18px var(--font-sans);
			cursor: pointer;
			text-align: left;
			transition: color 0.1s, background 0.1s;
		}
		.desk-collapse-btn:hover {
			color: var(--desk-oxide);
			background: var(--desk-raised);
		}
		.desk-collapse-icon-circle {
			width: 17px;
			height: 17px;
			border-radius: 50%;
			background: rgba(128, 128, 128, 0.22);
			display: inline-flex;
			align-items: center;
			justify-content: center;
			font-size: 11px;
			line-height: 1;
			flex-shrink: 0;
		}
		body.dp-sidebar-collapsed {
			--sidebar-w: 44px;
		}
		body.dp-sidebar-collapsed .nav-item > span:not(.nav-badge),
		body.dp-sidebar-collapsed .desk-collapse-label {
			display: none !important;
		}
		body.dp-sidebar-collapsed .nav-item {
			justify-content: center;
			padding: 9px 0;
		}
		body.dp-sidebar-collapsed .nav-badge {
			position: absolute;
			top: 3px;
			right: 3px;
			font-size: 10px;
			padding: 0 4px;
		}
		body.dp-sidebar-collapsed .nav-group.open > .nav-submenu {
			display: none;
		}
		body.dp-sidebar-collapsed .nav-group:hover > .nav-submenu {
			display: flex;
			position: fixed;
			left: var(--sidebar-w);
			top: 120px;
			min-width: 180px;
			padding: 6px 0;
			background: var(--desk-raised);
			box-shadow: 2px 3px 6px rgba(0, 0, 0, 0.22);
			z-index: 99999;
		}
		body.dp-sidebar-collapsed .desk-collapse-btn {
			justify-content: center;
			padding: 9px 0;
		}

		/* ==================== MAIN COLUMN & TOP-RIGHT SCREEN META LINKS ==================== */
		#day-main {
			margin-left: var(--sidebar-w);
			flex: 1;
			min-width: 0;
			display: flex;
			flex-direction: column;
			background: var(--paper);
			min-height: calc(100vh - var(--topbar-h));
		}

		#day-topbar {
			height: 36px;
			min-height: 36px;
			background: var(--paper-raised);
			border-bottom: 1px solid var(--rule);
			display: flex;
			align-items: center;
			justify-content: space-between;
			padding: 0 20px;
			font: 500 13px/18px var(--font-sans);
			color: var(--ink-muted);
		}

		/* WordPress #screen-meta-links hanging from top-right of #wpbody-content */
		#screen-meta {
			display: none;
			margin: 0 20px -1px 20px;
			background: var(--paper-raised);
			border: 1px solid var(--rule-strong);
			border-top: 0;
			border-radius: 0 0 4px 4px;
			padding: 16px 20px;
			box-shadow: 0 1px 1px rgba(0, 0, 0, 0.04);
		}

		#screen-meta.is-open {
			display: block;
		}

		#screen-meta-links {
			margin: 0 20px 0 0;
			display: flex;
			justify-content: flex-end;
			align-items: flex-start;
			gap: 6px;
			position: relative;
			z-index: 10;
		}

		.screen-meta-toggle {
			background: var(--paper-raised);
			border: 1px solid var(--rule-strong);
			border-top: 0;
			border-radius: 0 0 4px 4px;
			color: var(--ink-muted);
			font: 400 13px/1.7 var(--font-sans);
			padding: 3px 10px 4px 10px;
			cursor: pointer;
			display: inline-flex;
			align-items: center;
			gap: 6px;
			box-shadow: 0 1px 1px rgba(0, 0, 0, 0.04);
			transition: color 0.1s, border-color 0.1s;
		}

		.screen-meta-toggle:hover,
		.screen-meta-toggle[aria-expanded="true"] {
			color: var(--ink);
			border-color: #8c8f94;
		}

		.screen-meta-toggle::after {
			content: "\25BE";
			font-size: 12px;
			line-height: 1;
			color: var(--ink-muted);
		}

		.screen-meta-toggle[aria-expanded="true"]::after {
			content: "\25B4";
		}

		/* WordPress bottom footer (#wpfooter) */
		#wpfooter {
			margin-top: auto;
			padding: 14px 20px;
			display: flex;
			justify-content: space-between;
			align-items: center;
			gap: 12px;
			flex-wrap: wrap;
			color: var(--ink-muted);
			font: 400 13px/18px var(--font-sans);
			border-top: 1px solid var(--rule);
		}

		#wpfooter a {
			color: var(--press);
			text-decoration: none;
		}

		#wpfooter a:hover {
			text-decoration: underline;
		}

		#day-mobile-backdrop {
			display: none;
			position: fixed;
			inset: 0;
			background: rgba(29, 35, 39, 0.45);
			z-index: 9980;
		}

		/* ==================== RESPONSIVE BREAKPOINTS (Tablet <= 960px & Mobile <= 600px) ==================== */
		@media (max-width: 960px) {
			#day-sidebar {
				display: none;
				width: 210px;
				z-index: 9995;
			}
			body.dp-mobile-nav-open #day-sidebar {
				display: flex;
			}
			body.dp-mobile-nav-open #day-mobile-backdrop {
				display: block;
			}
			#day-main {
				margin-left: 0 !important;
			}
			.adminbar-menu-btn {
				display: inline-flex;
			}
			.adminbar-hide-mobile {
				display: none !important;
			}
			#screen-meta-links {
				margin-right: 16px;
			}
		}
"#
}
