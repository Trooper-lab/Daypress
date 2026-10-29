//! Admin CSS styles: brand tokens, fonts, the desk sidebar and the main column.
//!
//! Tokens follow the Daypress design system (Daylight theme): paper and ink do most of the
//! work, oxide is the brand accent, press blue is the product's working colour.

/// The brand typefaces, served from `/static/fonts/` (SIL OFL, see `static/fonts/OFL.txt`).
pub const DESK_FONT_FACES: &str = r#"
		@font-face { font-family: "Newsreader"; src: url("/static/fonts/Newsreader-Variable.woff2") format("woff2"); font-weight: 200 800; font-style: normal; font-display: swap; }
		@font-face { font-family: "Newsreader"; src: url("/static/fonts/Newsreader-Italic-Variable.woff2") format("woff2"); font-weight: 200 800; font-style: italic; font-display: swap; }
		@font-face { font-family: "Schibsted Grotesk"; src: url("/static/fonts/SchibstedGrotesk-Variable.woff2") format("woff2"); font-weight: 400 900; font-style: normal; font-display: swap; }
		@font-face { font-family: "IBM Plex Mono"; src: url("/static/fonts/IBMPlexMono-Regular.woff2") format("woff2"); font-weight: 400; font-style: normal; font-display: swap; }
		@font-face { font-family: "IBM Plex Mono"; src: url("/static/fonts/IBMPlexMono-Medium.woff2") format("woff2"); font-weight: 500; font-style: normal; font-display: swap; }
"#;

/// Tokens and element defaults shared by the desk and the login screens.
pub const DESK_FOUNDATION: &str = r#"
		:root {
			/* Daylight */
			--paper: #f3eee4;
			--paper-raised: #fbf8f2;
			--paper-sunk: #e7e0d1;
			--ink: #16171a;
			--ink-muted: #5a5852;
			--rule: #16171a2e;
			--rule-strong: #16171a;
			--oxide: #a8401b;
			--on-oxide: #fff8f0;
			--press: #2a3bd0;
			--on-press: #ffffff;
			--moss: #3d5a39;
			--ochre: #d9a42a;
			--on-ochre: #16171a;
			--focus: var(--press);
			--selection: #2a3bd02e;

			/* Washes behind status rows and notices; the accent itself carries the meaning. */
			--oxide-wash: color-mix(in srgb, var(--oxide) 9%, var(--paper-raised));
			--moss-wash: color-mix(in srgb, var(--moss) 10%, var(--paper-raised));
			--ochre-wash: color-mix(in srgb, var(--ochre) 22%, var(--paper-raised));
			--press-wash: color-mix(in srgb, var(--press) 8%, var(--paper-raised));

			/* The desk sidebar is set in the Night edition. */
			--desk: #1a1c1f;
			--desk-raised: #26282c;
			--desk-ink: #efe9de;
			--desk-muted: #a7a297;
			--desk-rule: #efe9de2e;
			--desk-oxide: #e8744a;

			--font-serif: "Newsreader", "Iowan Old Style", Georgia, serif;
			--font-sans: "Schibsted Grotesk", "Helvetica Neue", Arial, sans-serif;
			--font-mono: "IBM Plex Mono", ui-monospace, Menlo, monospace;

			--radius-none: 0px;
			--radius-sm: 2px;
			--radius-md: 6px;
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
			--primary: var(--press);
			--primary-hover: var(--ink);

			--sidebar-w: 232px;
			--topbar-h: 60px;
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
			font-size: 15px;
			line-height: 1.55;
			-webkit-font-smoothing: antialiased;
		}

		::selection { background: var(--selection); }

		:where(a, button, input, select, textarea, summary, [tabindex]):focus-visible {
			outline: 2px solid var(--focus);
			outline-offset: 2px;
		}

		a { color: var(--press); }

		code, kbd, pre, samp { font-family: var(--font-mono); }

		#day-canvas :where(h1, h2, h3) {
			font-family: var(--font-serif);
			font-weight: 450 !important;
			letter-spacing: -0.01em;
			font-variation-settings: 'opsz' 24;
		}

		@media (prefers-reduced-motion: reduce) {
			*, *::before, *::after { transition: none !important; animation: none !important; }
		}
"#;

/// The desk layout: sidebar and main column.
pub fn get_styles_shell() -> &'static str {
    r#"
		#day-app {
			display: flex;
			min-height: 100vh;
		}

		/* ==================== DESK SIDEBAR ==================== */
		#day-sidebar {
			width: var(--sidebar-w);
			background: var(--desk);
			color: var(--desk-ink);
			position: fixed;
			top: 0;
			bottom: 0;
			left: 0;
			display: flex;
			flex-direction: column;
			z-index: 100;
			overflow-y: auto;
		}

		.desk-brand {
			padding: 22px 20px 16px;
		}

		.desk-wordmark {
			display: inline-flex;
			align-items: center;
			gap: 10px;
			color: var(--desk-ink);
			text-decoration: none;
			font: 430 25px/1 var(--font-serif);
			letter-spacing: -0.02em;
			font-variation-settings: 'opsz' 48;
		}

		.desk-wordmark .day-dot {
			width: 9px;
			height: 9px;
			border-radius: var(--radius-pill);
			background: var(--desk-oxide);
			flex-shrink: 0;
		}

		.desk-site {
			display: block;
			margin-top: 10px;
			font: 500 12px/16px var(--font-mono);
			color: var(--desk-muted);
			text-decoration: none;
			white-space: nowrap;
			overflow: hidden;
			text-overflow: ellipsis;
		}

		.desk-site:hover { color: var(--desk-ink); }

		.desk-search {
			margin: 0 16px 8px;
			display: flex;
			justify-content: space-between;
			align-items: center;
			padding: 7px 10px;
			background: transparent;
			border: 1px solid var(--desk-rule);
			border-radius: var(--radius-sm);
			color: var(--desk-muted);
			font: 500 13px/18px var(--font-sans);
			cursor: pointer;
			transition: border-color .12s, color .12s;
		}

		.desk-search kbd {
			font: 500 11px/16px var(--font-mono);
			color: inherit;
		}

		.desk-search:hover {
			border-color: var(--desk-muted);
			color: var(--desk-ink);
		}

		.sidebar-nav {
			flex: 1;
			padding: 6px 0 16px;
		}

		.nav-section-title {
			font: 700 11px/16px var(--font-sans);
			letter-spacing: 0.14em;
			text-transform: uppercase;
			color: var(--desk-muted);
			padding: 16px 20px 6px;
		}

		.nav-item {
			display: flex;
			align-items: center;
			gap: 10px;
			padding: 9px 20px;
			text-decoration: none;
			color: var(--desk-ink);
			font: 500 14.5px/20px var(--font-sans);
			position: relative;
			transition: background 0.12s ease;
		}

		.nav-item:hover {
			background: var(--desk-raised);
		}

		.nav-item.active {
			background: var(--desk-raised);
			font-weight: 600;
		}

		.nav-item.active::before {
			content: "";
			position: absolute;
			left: 0;
			top: 0;
			bottom: 0;
			width: 3px;
			background: var(--desk-oxide);
		}

		/* The desk names things; it doesn't decorate them. */
		.nav-icon {
			display: none;
		}

		.nav-badge {
			margin-left: auto;
			font: 500 11px/16px var(--font-mono);
			color: var(--desk-muted);
		}

		/* Submenus: inline under the open section, a flyout on hover for the others. */
		.nav-group {
			position: relative;
		}

		.nav-submenu {
			display: none;
			flex-direction: column;
		}

		.nav-group.open > .nav-submenu {
			display: flex;
			position: static;
			padding: 2px 0 10px;
		}

		.nav-group:not(.open):hover > .nav-submenu {
			display: flex;
			position: fixed;
			left: var(--sidebar-w);
			top: 120px;
			min-width: 200px;
			padding: 6px 0;
			background: var(--desk-raised);
			border: 1px solid var(--desk-rule);
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
			padding: 6px 20px 6px 32px;
			font: 400 13.5px/18px var(--font-sans);
			color: var(--desk-muted);
			text-decoration: none;
			white-space: nowrap;
			transition: color 0.12s ease;
		}

		.nav-group:not(.open) .nav-subitem {
			padding-left: 16px;
		}

		.nav-subitem:hover {
			color: var(--desk-ink);
		}

		.nav-subitem.active {
			color: var(--desk-ink);
			font-weight: 600;
		}

		.desk-footer {
			border-top: 1px solid var(--desk-rule);
			padding: 14px 20px 18px;
		}

		.desk-account {
			display: flex;
			justify-content: space-between;
			align-items: baseline;
			gap: 8px;
			margin-bottom: 8px;
			font: 500 13.5px/18px var(--font-sans);
		}

		.desk-account a {
			color: var(--desk-ink);
			text-decoration: none;
			overflow: hidden;
			text-overflow: ellipsis;
			white-space: nowrap;
		}

		.desk-account a.desk-logout {
			color: var(--desk-muted);
			font-weight: 400;
			flex-shrink: 0;
		}

		.desk-account a:hover {
			text-decoration: underline;
			text-decoration-color: var(--desk-oxide);
			text-underline-offset: 4px;
		}

		.desk-version {
			font: 500 11.5px/16px var(--font-mono);
			color: var(--desk-muted);
		}

		/* ==================== MAIN COLUMN ==================== */
		#day-main {
			margin-left: var(--sidebar-w);
			flex: 1;
			min-width: 0;
			display: flex;
			flex-direction: column;
			background: var(--paper);
		}

		#day-topbar {
			min-height: var(--topbar-h);
			background: var(--paper);
			border-bottom: 1px solid var(--rule-strong);
			position: sticky;
			top: 0;
			z-index: 50;
			display: flex;
			align-items: center;
			justify-content: space-between;
			padding: 0 36px;
		}
"#
}
