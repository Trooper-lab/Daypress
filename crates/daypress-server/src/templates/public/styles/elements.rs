//! Canonical Daypress AST Elements Stylesheet (`templates/public/styles/elements.rs`).
//!
//! Provides zero-specificity (`:where(...)`) base styles for AST sections, containers,
//! flex, grid, columns, typography headings, body text, buttons, images, and cards
//! driven by `--dp-*` / `--e-*` CSS custom properties so imported light-mode or custom
//! design systems never get overridden by default dark-mode rules.

pub const CSS_ELEMENTS: &str = r#"
		/* ==========================================================================
		   Canonical Daypress AST Element System (:where(.dp-*))
		   Zero-specificity rules parameterized via --dp-* custom properties
		   ========================================================================== */
		:where(.dp-section) {
			width: 100%;
			padding: var(--dp-section-padding, 64px 24px);
			position: relative;
			border-bottom: var(--dp-section-border, 1px solid var(--border-subtle, transparent));
		}

		:where(.dp-container) {
			max-width: var(--dp-container-max-width, 1200px);
			margin: var(--dp-container-margin, 0 auto);
			width: var(--dp-container-width, 100%);
		}

		:where(.dp-container .dp-container),
		:where(.dp-grid > .dp-container),
		:where(.dp-flex > .dp-container),
		:where(.dp-column > .dp-container) {
			max-width: var(--dp-nested-container-max-width, none);
			margin: var(--dp-nested-container-margin, 0);
			width: var(--dp-nested-container-width, auto);
		}

		:where(.dp-grid) {
			display: grid;
			gap: var(--dp-grid-gap, 24px);
			width: 100%;
		}

		:where(.dp-flex) {
			display: flex;
			gap: var(--dp-flex-gap, 20px);
		}

		:where(.dp-section:has(> .dp-flex + .dp-flex)),
		:where(.dp-container:has(> .dp-flex + .dp-flex)) {
			display: flex;
			flex-direction: row;
			gap: 28px;
			align-items: flex-start;
		}

		:where(.dp-section > .dp-flex + .dp-flex),
		:where(.dp-section > .dp-flex:has(+ .dp-flex)),
		:where(.dp-container > .dp-flex + .dp-flex),
		:where(.dp-container > .dp-flex:has(+ .dp-flex)) {
			flex: 1 1 0%;
			min-width: 0;
		}

		:where(.dp-column) {
			display: flex;
			flex-direction: column;
			gap: var(--dp-column-gap, 16px);
			min-width: 0;
		}

		:where(.dp-heading) {
			color: var(--dp-color-heading, var(--e-global-color-secondary, var(--text-main, inherit)));
			font-family: var(--dp-font-heading, var(--e-global-typography-primary-font-family, inherit));
			font-weight: var(--dp-heading-weight, 800);
			letter-spacing: var(--dp-heading-tracking, -0.025em);
			line-height: var(--dp-heading-line-height, 1.2);
			margin-bottom: var(--dp-heading-margin-bottom, 12px);
		}

		:where(h1.dp-heading) { font-size: var(--dp-h1-size, 44px); }
		:where(h2.dp-heading) { font-size: var(--dp-h2-size, 32px); }
		:where(h3.dp-heading) { font-size: var(--dp-h3-size, 22px); }
		:where(h4.dp-heading) { font-size: var(--dp-h4-size, 18px); }

		:where(.dp-text) {
			color: var(--dp-color-text, var(--e-global-color-text, var(--text-muted, inherit)));
			font-family: var(--dp-font-body, var(--e-global-typography-text-font-family, inherit));
			font-size: var(--dp-text-size, 16px);
			line-height: var(--dp-text-line-height, 1.75);
		}

		:where(.dp-text p) {
			margin-bottom: var(--dp-paragraph-gap, 16px);
		}

		:where(.dp-text p:last-child) {
			margin-bottom: 0;
		}

		:where(.dp-image) {
			max-width: 100%;
			height: auto;
			border-radius: var(--dp-image-radius, 10px);
			display: block;
		}

		:where(.dp-button), .btn-primary-neon {
			display: inline-flex;
			align-items: center;
			justify-content: center;
			gap: 8px;
			background: var(--dp-button-bg, var(--e-button-bg, #2563eb));
			color: var(--dp-button-color, var(--e-button-color, #ffffff));
			font-family: var(--dp-font-accent, var(--e-global-typography-accent-font-family, inherit));
			font-size: var(--dp-button-font-size, 14px);
			font-weight: var(--dp-button-font-weight, 600);
			padding: var(--dp-button-padding, 11px 22px);
			border-radius: var(--dp-button-radius, var(--e-button-border-radius, 8px));
			border: var(--dp-button-border, 1px solid rgba(255,255,255,0.1));
			cursor: pointer;
			transition: all 0.15s ease;
			width: fit-content;
			text-decoration: none;
		}

		:where(.dp-button:hover), .btn-primary-neon:hover {
			background: var(--dp-button-hover-bg, #1d4ed8);
			transform: translateY(-1px);
		}

		.dp-btn-sm {
			padding: 6px 14px;
			font-size: 12.5px;
		}

		.btn-secondary-edge {
			display: inline-flex;
			align-items: center;
			justify-content: center;
			gap: 8px;
			background: var(--bg-card);
			color: #ffffff;
			font-size: 14px;
			font-weight: 600;
			padding: 10px 20px;
			border-radius: 8px;
			border: 1px solid var(--border);
		}

		:where(.dp-divider) {
			border: none;
			border-top: 1px solid var(--dp-divider-color, var(--border, #e2e8f0));
			margin: var(--dp-divider-margin, 24px 0);
			width: 100%;
		}

		.dp-iconbox, .dp-imagebox {
			display: flex;
			flex-direction: column;
			gap: 10px;
			text-decoration: none;
			color: inherit;
			font-size: 22px;
		}

		.dp-iconbox-icon {
			color: var(--dp-color-primary, var(--e-global-color-primary, #e23212));
			display: flex;
			align-items: center;
			justify-content: flex-start;
			font-size: 44px;
			margin-bottom: 4px;
		}

		.dp-imagebox-img {
			width: 100%;
			overflow: hidden;
			border-radius: 8px;
		}

		.dp-imagebox-img img {
			width: 100%;
			max-width: 100%;
			height: auto;
			display: block;
			object-fit: cover;
			border-radius: 8px;
		}

		.dp-iconbox-title, .dp-imagebox-title {
			font-size: inherit;
			font-family: inherit;
			font-weight: 700;
			color: inherit;
			line-height: 1.15;
			margin: 0;
		}

		.dp-iconbox-desc, .dp-imagebox-desc {
			font-size: 15px;
			font-family: var(--dp-font-body, inherit);
			font-weight: 400;
			color: inherit;
			opacity: 0.9;
			line-height: 1.6;
			margin: 0;
		}

		.dp-counter {
			background: rgba(15, 23, 42, 0.75);
			border: 1px solid var(--border);
			border-radius: 12px;
			padding: 24px;
			text-align: center;
		}

		.dp-counter-val {
			font-size: 38px;
			font-weight: 900;
			color: #38bdf8;
			font-family: var(--font-mono);
			line-height: 1.1;
			margin-bottom: 6px;
		}

		.dp-counter-title {
			font-size: 13px;
			font-weight: 600;
			color: var(--text-muted);
			text-transform: uppercase;
			letter-spacing: 0.05em;
		}

		.dp-accordion {
			display: flex;
			flex-direction: column;
			gap: 10px;
		}

		.dp-accordion-item {
			background: var(--bg-card);
			border: 1px solid var(--border);
			border-radius: 10px;
			overflow: hidden;
		}

		.dp-accordion-item summary {
			padding: 16px 20px;
			font-weight: 700;
			font-size: 15px;
			color: var(--text-main, #16171a);
			cursor: pointer;
			list-style: none;
		}

		.dp-accordion-body {
			padding: 0 20px 18px 20px;
			color: var(--text-muted);
			font-size: 14.5px;
			line-height: 1.65;
		}

		/* Daypress Post & Page Card Elements */
		.dp-card {
			background: var(--bg-card);
			border: 1px solid var(--border);
			border-radius: 12px;
			overflow: hidden;
			display: flex;
			flex-direction: column;
			transition: transform 0.15s ease, border-color 0.15s ease;
		}

		.dp-card:hover {
			border-color: var(--accent, #a8401b);
			transform: translateY(-2px);
		}

		.dp-card-media {
			width: 100%;
			height: 200px;
			overflow: hidden;
			background: var(--bg-card-hover, #e7e0d1);
			border-bottom: 1px solid var(--border);
		}

		.dp-card-media img {
			width: 100%;
			height: 100%;
			object-fit: cover;
			display: block;
		}

		.dp-card-body {
			padding: 24px;
			display: flex;
			flex-direction: column;
			flex: 1;
		}

		.dp-card-meta {
			font-size: 12px;
			color: var(--text-dim);
			display: flex;
			align-items: center;
			flex-wrap: wrap;
			gap: 8px;
			margin-bottom: 10px;
		}

		.dp-badge {
			background: color-mix(in srgb, var(--accent, #a8401b) 12%, transparent);
			color: var(--accent, #a8401b);
			padding: 2px 8px;
			border-radius: 4px;
			font-weight: 700;
			font-size: 11px;
			text-transform: uppercase;
		}

		.dp-card-title {
			font-size: 20px;
			font-family: var(--dp-font-heading, inherit);
			font-weight: 600;
			color: var(--text-main, #16171a);
			line-height: 1.35;
			margin-bottom: 10px;
		}

		.dp-card-excerpt {
			font-size: 14px;
			color: var(--text-muted);
			line-height: 1.6;
			margin-bottom: 18px;
			flex: 1;
		}

		.dp-card-link {
			font-size: 13px;
			font-weight: 600;
			color: var(--accent, #a8401b);
		}

		/* Dropdown & Recursive Nav Menus (.theme-nav, .day-nav) */
		.theme-nav ul, .day-nav ul { list-style: none; margin: 0; padding: 0; }
		.theme-nav ul.nav-menu, .day-nav ul.menu { display: flex; align-items: center; gap: 16px; }
		.theme-nav li.menu-item, .day-nav li.menu-item { position: relative; }
		.theme-nav li.menu-item a, .day-nav li.menu-item a {
			display: block; text-decoration: none; color: inherit; transition: color 0.15s ease;
		}
		.theme-nav li.menu-item-has-children > a::after, .day-nav li.menu-item-has-children > a::after {
			content: ' ▾'; font-size: 10px; opacity: 0.7; margin-left: 4px;
		}
		.theme-nav ul.sub-menu li.menu-item-has-children > a::after, .day-nav ul.sub-menu li.menu-item-has-children > a::after {
			content: ' ▸'; float: right; margin-left: 8px;
		}
		.theme-nav ul.sub-menu, .day-nav ul.sub-menu {
			display: none; position: absolute; top: 100%; left: 0; min-width: 190px;
			background: var(--bg-card, #111827); border: 1px solid var(--border, #1e293b);
			border-radius: 8px; box-shadow: 0 10px 25px rgba(0, 0, 0, 0.4); padding: 6px 0; z-index: 1000;
		}
		.theme-nav ul.sub-menu ul.sub-menu, .day-nav ul.sub-menu ul.sub-menu { top: 0; left: 100%; }
		.theme-nav li.menu-item:hover > ul.sub-menu, .theme-nav li.menu-item:focus-within > ul.sub-menu,
		.day-nav li.menu-item:hover > ul.sub-menu, .day-nav li.menu-item:focus-within > ul.sub-menu {
			display: block;
		}
		.theme-nav ul.sub-menu li.menu-item a, .day-nav ul.sub-menu li.menu-item a {
			padding: 8px 16px; color: var(--text-muted, #cbd5e1); font-size: 13.5px; white-space: nowrap; text-align: left;
		}
		.theme-nav ul.sub-menu li.menu-item a:hover, .theme-nav ul.sub-menu li.menu-item a:focus,
		.day-nav ul.sub-menu li.menu-item a:hover, .day-nav ul.sub-menu li.menu-item a:focus {
			background: rgba(255, 255, 255, 0.06); color: #60a5fa;
		}
		@media (max-width: 768px) {
			.theme-header .header-inner { flex-wrap: wrap; gap: 16px; padding: 12px 0 !important; }
			.theme-nav { width: 100%; }
			.theme-nav ul.nav-menu { flex-wrap: wrap; gap: 12px !important; }
			.theme-nav ul.sub-menu, .day-nav ul.sub-menu {
				position: static; min-width: 100%; box-shadow: none; border: none;
				background: rgba(255, 255, 255, 0.04); border-radius: 4px; padding: 4px 0 4px 12px; margin-top: 4px;
			}
			.theme-nav ul.sub-menu ul.sub-menu, .day-nav ul.sub-menu ul.sub-menu {
				position: static; left: 0; top: 0; padding-left: 12px;
			}
		}

		.dp-card.sticky {
			border: 2px solid #3b82f6 !important;
			box-shadow: 0 0 12px rgba(59, 130, 246, 0.25);
		}
"#;
