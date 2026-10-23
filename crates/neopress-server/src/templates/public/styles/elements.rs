//! Canonical Neopress AST Elements Stylesheet (`templates/public/styles/elements.rs`).
//!
//! Provides styles for AST sections, containers, flex, grid, columns,
//! typography headings, body text, buttons, imageboxes, iconboxes, counters,
//! accordions, and post/page cards.

pub const CSS_ELEMENTS: &str = r#"
		/* ==========================================================================
		   Canonical Neopress AST Element System (.np-*)
		   ========================================================================== */
		.np-section {
			width: 100%;
			padding: 64px 24px;
			position: relative;
			border-bottom: 1px solid var(--border-subtle);
		}

		.np-container {
			max-width: 1200px;
			margin: 0 auto;
			width: 100%;
		}

		.np-grid {
			display: grid;
			gap: 24px;
			width: 100%;
		}

		.np-flex {
			display: flex;
			gap: 20px;
		}

		.np-section:has(> .np-flex + .np-flex),
		.np-container:has(> .np-flex + .np-flex) {
			display: flex;
			flex-direction: row;
			gap: 28px;
			align-items: flex-start;
		}

		.np-section > .np-flex + .np-flex,
		.np-section > .np-flex:has(+ .np-flex),
		.np-container > .np-flex + .np-flex,
		.np-container > .np-flex:has(+ .np-flex) {
			flex: 1 1 0%;
			min-width: 0;
		}

		.np-column {
			display: flex;
			flex-direction: column;
			gap: 16px;
			min-width: 0;
		}

		.np-heading {
			color: #ffffff;
			font-weight: 800;
			letter-spacing: -0.025em;
			line-height: 1.2;
			margin-bottom: 12px;
		}

		h1.np-heading { font-size: 44px; }
		h2.np-heading { font-size: 32px; }
		h3.np-heading { font-size: 22px; }
		h4.np-heading { font-size: 18px; }

		.np-text {
			color: #cbd5e1;
			font-size: 16px;
			line-height: 1.75;
		}

		.np-text p {
			margin-bottom: 16px;
		}

		.np-text p:last-child {
			margin-bottom: 0;
		}

		.np-image {
			max-width: 100%;
			height: auto;
			border-radius: 10px;
			display: block;
		}

		.np-button, .btn-primary-neon {
			display: inline-flex;
			align-items: center;
			justify-content: center;
			gap: 8px;
			background: #2563eb;
			color: #ffffff;
			font-size: 14px;
			font-weight: 600;
			padding: 11px 22px;
			border-radius: 8px;
			border: 1px solid rgba(255,255,255,0.1);
			cursor: pointer;
			transition: all 0.15s ease;
			width: fit-content;
		}

		.np-button:hover, .btn-primary-neon:hover {
			background: #1d4ed8;
			transform: translateY(-1px);
		}

		.np-btn-sm {
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

		.np-divider {
			border: none;
			border-top: 1px solid var(--border);
			margin: 24px 0;
			width: 100%;
		}

		.np-iconbox, .np-imagebox {
			display: flex;
			flex-direction: column;
			gap: 10px;
			text-decoration: none;
			color: inherit;
			font-size: 22px;
		}

		.np-iconbox-icon {
			color: var(--np-color-primary, var(--e-global-color-primary, #e23212));
			display: flex;
			align-items: center;
			justify-content: flex-start;
			font-size: 44px;
			margin-bottom: 4px;
		}

		.np-imagebox-img {
			width: 100%;
			overflow: hidden;
			border-radius: 8px;
		}

		.np-imagebox-img img {
			width: 100%;
			max-width: 100%;
			height: auto;
			display: block;
			object-fit: cover;
			border-radius: 8px;
		}

		.np-iconbox-title, .np-imagebox-title {
			font-size: inherit;
			font-family: inherit;
			font-weight: 700;
			color: inherit;
			line-height: 1.15;
			margin: 0;
		}

		.np-iconbox-desc, .np-imagebox-desc {
			font-size: 15px;
			font-family: var(--np-font-body, inherit);
			font-weight: 400;
			color: inherit;
			opacity: 0.9;
			line-height: 1.6;
			margin: 0;
		}

		.np-counter {
			background: rgba(15, 23, 42, 0.75);
			border: 1px solid var(--border);
			border-radius: 12px;
			padding: 24px;
			text-align: center;
		}

		.np-counter-val {
			font-size: 38px;
			font-weight: 900;
			color: #38bdf8;
			font-family: var(--font-mono);
			line-height: 1.1;
			margin-bottom: 6px;
		}

		.np-counter-title {
			font-size: 13px;
			font-weight: 600;
			color: var(--text-muted);
			text-transform: uppercase;
			letter-spacing: 0.05em;
		}

		.np-accordion {
			display: flex;
			flex-direction: column;
			gap: 10px;
		}

		.np-accordion-item {
			background: var(--bg-card);
			border: 1px solid var(--border);
			border-radius: 10px;
			overflow: hidden;
		}

		.np-accordion-item summary {
			padding: 16px 20px;
			font-weight: 700;
			font-size: 15px;
			color: #ffffff;
			cursor: pointer;
			list-style: none;
		}

		.np-accordion-body {
			padding: 0 20px 18px 20px;
			color: var(--text-muted);
			font-size: 14.5px;
			line-height: 1.65;
		}

		/* Neopress Post & Page Card Elements */
		.np-card {
			background: var(--bg-card);
			border: 1px solid var(--border);
			border-radius: 12px;
			overflow: hidden;
			display: flex;
			flex-direction: column;
			transition: transform 0.15s ease, border-color 0.15s ease;
		}

		.np-card:hover {
			border-color: #3b82f6;
			transform: translateY(-2px);
		}

		.np-card-media {
			width: 100%;
			height: 200px;
			overflow: hidden;
			background: #0f172a;
			border-bottom: 1px solid var(--border);
		}

		.np-card-media img {
			width: 100%;
			height: 100%;
			object-fit: cover;
			display: block;
		}

		.np-card-body {
			padding: 24px;
			display: flex;
			flex-direction: column;
			flex: 1;
		}

		.np-card-meta {
			font-size: 12px;
			color: var(--text-dim);
			display: flex;
			align-items: center;
			flex-wrap: wrap;
			gap: 8px;
			margin-bottom: 10px;
		}

		.np-badge {
			background: rgba(37, 99, 235, 0.18);
			color: #60a5fa;
			padding: 2px 8px;
			border-radius: 4px;
			font-weight: 700;
			font-size: 11px;
			text-transform: uppercase;
		}

		.np-card-title {
			font-size: 20px;
			font-weight: 700;
			color: #ffffff;
			line-height: 1.35;
			margin-bottom: 10px;
		}

		.np-card-excerpt {
			font-size: 14px;
			color: var(--text-muted);
			line-height: 1.6;
			margin-bottom: 18px;
			flex: 1;
		}

		.np-card-link {
			font-size: 13px;
			font-weight: 600;
			color: #60a5fa;
		}

		/* Dropdown & Recursive Nav Menus (.theme-nav, .neo-nav) */
		.theme-nav ul, .neo-nav ul { list-style: none; margin: 0; padding: 0; }
		.theme-nav ul.nav-menu, .neo-nav ul.menu { display: flex; align-items: center; gap: 16px; }
		.theme-nav li.menu-item, .neo-nav li.menu-item { position: relative; }
		.theme-nav li.menu-item a, .neo-nav li.menu-item a {
			display: block; text-decoration: none; color: inherit; transition: color 0.15s ease;
		}
		.theme-nav li.menu-item-has-children > a::after, .neo-nav li.menu-item-has-children > a::after {
			content: ' ▾'; font-size: 10px; opacity: 0.7; margin-left: 4px;
		}
		.theme-nav ul.sub-menu li.menu-item-has-children > a::after, .neo-nav ul.sub-menu li.menu-item-has-children > a::after {
			content: ' ▸'; float: right; margin-left: 8px;
		}
		.theme-nav ul.sub-menu, .neo-nav ul.sub-menu {
			display: none; position: absolute; top: 100%; left: 0; min-width: 190px;
			background: var(--bg-card, #111827); border: 1px solid var(--border, #1e293b);
			border-radius: 8px; box-shadow: 0 10px 25px rgba(0, 0, 0, 0.4); padding: 6px 0; z-index: 1000;
		}
		.theme-nav ul.sub-menu ul.sub-menu, .neo-nav ul.sub-menu ul.sub-menu { top: 0; left: 100%; }
		.theme-nav li.menu-item:hover > ul.sub-menu, .theme-nav li.menu-item:focus-within > ul.sub-menu,
		.neo-nav li.menu-item:hover > ul.sub-menu, .neo-nav li.menu-item:focus-within > ul.sub-menu {
			display: block;
		}
		.theme-nav ul.sub-menu li.menu-item a, .neo-nav ul.sub-menu li.menu-item a {
			padding: 8px 16px; color: var(--text-muted, #cbd5e1); font-size: 13.5px; white-space: nowrap; text-align: left;
		}
		.theme-nav ul.sub-menu li.menu-item a:hover, .theme-nav ul.sub-menu li.menu-item a:focus,
		.neo-nav ul.sub-menu li.menu-item a:hover, .neo-nav ul.sub-menu li.menu-item a:focus {
			background: rgba(255, 255, 255, 0.06); color: #60a5fa;
		}
		@media (max-width: 768px) {
			.theme-header .header-inner { flex-wrap: wrap; gap: 16px; padding: 12px 0 !important; }
			.theme-nav { width: 100%; }
			.theme-nav ul.nav-menu { flex-wrap: wrap; gap: 12px !important; }
			.theme-nav ul.sub-menu, .neo-nav ul.sub-menu {
				position: static; min-width: 100%; box-shadow: none; border: none;
				background: rgba(255, 255, 255, 0.04); border-radius: 4px; padding: 4px 0 4px 12px; margin-top: 4px;
			}
			.theme-nav ul.sub-menu ul.sub-menu, .neo-nav ul.sub-menu ul.sub-menu {
				position: static; left: 0; top: 0; padding-left: 12px;
			}
		}

		.np-card.sticky {
			border: 2px solid #3b82f6 !important;
			box-shadow: 0 0 12px rgba(59, 130, 246, 0.25);
		}
"#;
