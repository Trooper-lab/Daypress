//! Public SSR Theme Stylesheet (`templates/public/styles.rs`).
//!
//! Renders the base CSS variables and responsive stylesheet for public pages.

/// Renders the core public theme CSS interpolated with the site's active `brand_color`.
pub fn render_public_theme_css(brand_color: &str) -> String {
    format!(
        r##"
		:root {{
			--bg-base: #07090e;
			--bg-panel: #0b0f19;
			--bg-card: #111827;
			--bg-card-hover: #1e293b;
			--border: #1e293b;
			--border-subtle: #162032;
			--border-focus: {brand_color};
			--text-main: #f8fafc;
			--text-muted: #94a3b8;
			--text-dim: #64748b;
			--accent: {brand_color};
			--accent-hover: #3b82f6;
			--accent-glow: rgba(37, 99, 235, 0.25);
			--e-global-color-primary: #2563eb;
			--e-global-color-secondary: #0f172a;
			--e-global-color-text: #e2e8f0;
			--e-global-color-accent: #ff4f00;
			--font-sans: 'Inter', -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
			--font-mono: 'JetBrains Mono', monospace;
		}}

		*, *::before, *::after {{
			box-sizing: border-box;
			margin: 0;
			padding: 0;
		}}

		body {{
			background-color: var(--bg-base);
			color: var(--text-main);
			font-family: var(--font-sans);
			font-size: 15px;
			line-height: 1.65;
			-webkit-font-smoothing: antialiased;
			min-height: 100vh;
			display: flex;
			flex-direction: column;
			overflow-x: hidden;
		}}

		a {{
			color: inherit;
			text-decoration: none;
			transition: all 0.15s ease;
		}}

		/* Global Header */
		header.neo-header {{
			position: sticky;
			top: 0;
			z-index: 1000;
			background: rgba(11, 15, 25, 0.9);
			backdrop-filter: blur(12px);
			-webkit-backdrop-filter: blur(12px);
			border-bottom: 1px solid var(--border);
			min-height: 68px;
			display: flex;
			align-items: center;
		}}

		.header-container {{
			max-width: 1240px;
			margin: 0 auto;
			padding: 10px 24px;
			width: 100%;
			display: flex;
			align-items: center;
			justify-content: space-between;
			gap: 20px;
		}}

		.logo-brand {{
			display: inline-flex;
			align-items: center;
			gap: 10px;
			font-weight: 800;
			font-size: 17px;
			letter-spacing: -0.02em;
			color: #ffffff;
			white-space: nowrap;
		}}

		.logo-brand .badge-neo {{
			background: linear-gradient(135deg, #2563eb 0%, #ff4f00 100%);
			color: #ffffff;
			font-size: 10px;
			font-weight: 800;
			padding: 3px 7px;
			border-radius: 4px;
			letter-spacing: 0.05em;
			text-transform: uppercase;
		}}

		nav.neo-nav {{
			display: flex;
			align-items: center;
			flex-wrap: wrap;
			gap: 18px;
		}}

		.nav-link {{
			font-size: 13.5px;
			font-weight: 500;
			color: var(--text-muted);
			transition: color 0.15s ease;
		}}

		.nav-link:hover {{
			color: #ffffff;
		}}

		.header-actions {{
			display: flex;
			align-items: center;
			gap: 12px;
		}}

		.header-search {{
			display: flex;
			align-items: center;
			background: var(--bg-card);
			border: 1px solid var(--border);
			border-radius: 6px;
			padding: 4px 10px;
		}}

		.header-search input {{
			background: transparent;
			border: none;
			color: #ffffff;
			font-size: 12.5px;
			width: 140px;
			outline: none;
		}}

		.btn-login {{
			font-size: 13px;
			font-weight: 600;
			color: var(--text-muted);
			padding: 6px 12px;
			border-radius: 6px;
			border: 1px solid var(--border);
		}}

		.btn-login:hover {{
			color: #ffffff;
			border-color: #475569;
		}}

		main.neo-main {{
			flex: 1;
		}}

		/* ==========================================================================
		   Canonical Neopress AST Element System (.np-*)
		   ========================================================================== */
		.np-section {{
			width: 100%;
			padding: 64px 24px;
			position: relative;
			border-bottom: 1px solid var(--border-subtle);
		}}

		.np-container {{
			max-width: 1200px;
			margin: 0 auto;
			width: 100%;
		}}

		.np-grid {{
			display: grid;
			gap: 24px;
			width: 100%;
		}}

		.np-flex {{
			display: flex;
			gap: 20px;
		}}

		.np-section:has(> .np-flex + .np-flex),
		.np-container:has(> .np-flex + .np-flex) {{
			display: flex;
			flex-direction: row;
			gap: 28px;
			align-items: flex-start;
		}}

		.np-section > .np-flex + .np-flex,
		.np-section > .np-flex:has(+ .np-flex),
		.np-container > .np-flex + .np-flex,
		.np-container > .np-flex:has(+ .np-flex) {{
			flex: 1 1 0%;
			min-width: 0;
		}}

		.np-column {{
			display: flex;
			flex-direction: column;
			gap: 16px;
			min-width: 0;
		}}

		.np-heading {{
			color: #ffffff;
			font-weight: 800;
			letter-spacing: -0.025em;
			line-height: 1.2;
			margin-bottom: 12px;
		}}

		h1.np-heading {{ font-size: 44px; }}
		h2.np-heading {{ font-size: 32px; }}
		h3.np-heading {{ font-size: 22px; }}
		h4.np-heading {{ font-size: 18px; }}

		.np-text {{
			color: #cbd5e1;
			font-size: 16px;
			line-height: 1.75;
		}}

		.np-text p {{
			margin-bottom: 16px;
		}}

		.np-text p:last-child {{
			margin-bottom: 0;
		}}

		.np-image {{
			max-width: 100%;
			height: auto;
			border-radius: 10px;
			display: block;
		}}

		.np-button, .btn-primary-neon {{
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
		}}

		.np-button:hover, .btn-primary-neon:hover {{
			background: #1d4ed8;
			transform: translateY(-1px);
		}}

		.np-btn-sm {{
			padding: 6px 14px;
			font-size: 12.5px;
		}}

		.btn-secondary-edge {{
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
		}}

		.np-divider {{
			border: none;
			border-top: 1px solid var(--border);
			margin: 24px 0;
			width: 100%;
		}}

		.np-iconbox, .np-imagebox {{
			background: var(--bg-card);
			border: 1px solid var(--border);
			border-radius: 12px;
			padding: 28px;
			display: flex;
			flex-direction: column;
			gap: 10px;
		}}

		.np-iconbox-icon {{
			width: 44px;
			height: 44px;
			border-radius: 10px;
			background: rgba(37, 99, 235, 0.15);
			color: #60a5fa;
			display: flex;
			align-items: center;
			justify-content: center;
			font-size: 20px;
			margin-bottom: 4px;
		}}

		.np-iconbox-title, .np-imagebox-title {{
			font-size: 19px;
			font-weight: 700;
			color: #ffffff;
		}}

		.np-iconbox-desc, .np-imagebox-desc {{
			font-size: 14.5px;
			color: var(--text-muted);
			line-height: 1.6;
		}}

		.np-counter {{
			background: rgba(15, 23, 42, 0.75);
			border: 1px solid var(--border);
			border-radius: 12px;
			padding: 24px;
			text-align: center;
		}}

		.np-counter-val {{
			font-size: 38px;
			font-weight: 900;
			color: #38bdf8;
			font-family: var(--font-mono);
			line-height: 1.1;
			margin-bottom: 6px;
		}}

		.np-counter-title {{
			font-size: 13px;
			font-weight: 600;
			color: var(--text-muted);
			text-transform: uppercase;
			letter-spacing: 0.05em;
		}}

		.np-accordion {{
			display: flex;
			flex-direction: column;
			gap: 10px;
		}}

		.np-accordion-item {{
			background: var(--bg-card);
			border: 1px solid var(--border);
			border-radius: 10px;
			overflow: hidden;
		}}

		.np-accordion-item summary {{
			padding: 16px 20px;
			font-weight: 700;
			font-size: 15px;
			color: #ffffff;
			cursor: pointer;
			list-style: none;
		}}

		.np-accordion-body {{
			padding: 0 20px 18px 20px;
			color: var(--text-muted);
			font-size: 14.5px;
			line-height: 1.65;
		}}

		.np-price-table {{
			background: linear-gradient(180deg, #111827 0%, #0f172a 100%);
			border: 1px solid #334155;
			border-radius: 14px;
			padding: 32px;
			display: flex;
			flex-direction: column;
			gap: 20px;
		}}

		.np-price-title {{
			font-size: 20px;
			font-weight: 800;
			color: #ffffff;
		}}

		.np-price-amount {{
			display: flex;
			align-items: baseline;
			gap: 6px;
		}}

		.np-price-val {{
			font-size: 44px;
			font-weight: 900;
			color: #ffffff;
			font-family: var(--font-mono);
		}}

		.np-price-period {{
			font-size: 14px;
			color: var(--text-muted);
		}}

		.np-price-features {{
			list-style: none;
			display: flex;
			flex-direction: column;
			gap: 10px;
		}}

		.np-price-feature {{
			font-size: 14.5px;
			color: #cbd5e1;
			display: flex;
			align-items: center;
			gap: 8px;
		}}

		.np-price-feature::before {{
			content: "✓";
			color: #10b981;
			font-weight: 800;
		}}

		.np-progressbar {{
			background: var(--bg-card);
			border: 1px solid var(--border);
			border-radius: 10px;
			padding: 18px 20px;
		}}

		.np-progress-title {{
			display: flex;
			justify-content: space-between;
			font-size: 13.5px;
			font-weight: 700;
			color: #ffffff;
			margin-bottom: 10px;
		}}

		.np-progress-track {{
			height: 10px;
			background: #090d16;
			border-radius: 999px;
			overflow: hidden;
		}}

		.np-progress-fill {{
			height: 100%;
			background: linear-gradient(90deg, #2563eb, #38bdf8);
			border-radius: 999px;
		}}

		/* Neopress Post & Page Card Elements */
		.np-card {{
			background: var(--bg-card);
			border: 1px solid var(--border);
			border-radius: 12px;
			overflow: hidden;
			display: flex;
			flex-direction: column;
			transition: transform 0.15s ease, border-color 0.15s ease;
		}}

		.np-card:hover {{
			border-color: #3b82f6;
			transform: translateY(-2px);
		}}

		.np-card-media {{
			width: 100%;
			height: 200px;
			overflow: hidden;
			background: #0f172a;
			border-bottom: 1px solid var(--border);
		}}

		.np-card-media img {{
			width: 100%;
			height: 100%;
			object-fit: cover;
			display: block;
		}}

		.np-card-body {{
			padding: 24px;
			display: flex;
			flex-direction: column;
			flex: 1;
		}}

		.np-card-meta {{
			font-size: 12px;
			color: var(--text-dim);
			display: flex;
			align-items: center;
			flex-wrap: wrap;
			gap: 8px;
			margin-bottom: 10px;
		}}

		.np-badge {{
			background: rgba(37, 99, 235, 0.18);
			color: #60a5fa;
			padding: 2px 8px;
			border-radius: 4px;
			font-weight: 700;
			font-size: 11px;
			text-transform: uppercase;
		}}

		.np-card-title {{
			font-size: 20px;
			font-weight: 700;
			color: #ffffff;
			line-height: 1.35;
			margin-bottom: 10px;
		}}

		.np-card-excerpt {{
			font-size: 14px;
			color: var(--text-muted);
			line-height: 1.6;
			margin-bottom: 18px;
			flex: 1;
		}}

		.np-card-link {{
			font-size: 13px;
			font-weight: 600;
			color: #60a5fa;
		}}

		/* Global Footer */
		footer.neo-footer {{
			background: #04060b;
			border-top: 1px solid var(--border);
			padding: 56px 0 36px 0;
			color: var(--text-muted);
			font-size: 13.5px;
		}}

		.footer-grid {{
			max-width: 1200px;
			margin: 0 auto;
			padding: 0 24px;
			display: grid;
			grid-template-columns: 2fr 1.5fr 1fr;
			gap: 40px;
			margin-bottom: 40px;
		}}

		.footer-brand p {{
			color: var(--text-dim);
			margin-top: 12px;
			max-width: 420px;
			line-height: 1.6;
		}}

		.footer-col-title {{
			color: #ffffff;
			font-size: 12px;
			font-weight: 700;
			text-transform: uppercase;
			letter-spacing: 0.06em;
			margin-bottom: 16px;
		}}

		.footer-links {{
			display: flex;
			flex-direction: column;
			gap: 8px;
		}}

		.footer-links a {{
			color: var(--text-muted);
		}}

		.footer-links a:hover {{
			color: #ffffff;
		}}

		.footer-bottom {{
			max-width: 1200px;
			margin: 0 auto;
			padding: 24px 24px 0 24px;
			border-top: 1px solid var(--border-subtle);
			display: flex;
			justify-content: space-between;
			align-items: center;
			font-size: 12.5px;
			color: var(--text-dim);
		}}

		.engine-chip {{
			display: inline-flex;
			align-items: center;
			gap: 6px;
			background: #0c1220;
			border: 1px solid #1e293b;
			padding: 3px 10px;
			border-radius: 4px;
			color: #38bdf8;
			font-family: var(--font-mono);
			font-size: 11px;
		}}

		/* WordPress Live Admin Bar */
		#wpadminbar {{
			direction: ltr;
			color: #c3c4c7;
			font-size: 13px;
			font-weight: 400;
			font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
			line-height: 32px;
			height: 32px;
			position: fixed;
			top: 0;
			left: 0;
			width: 100%;
			z-index: 999999;
			background: #1d2327;
			box-shadow: 0 1px 4px rgba(0,0,0,0.5);
		}}
		body.admin-bar {{
			margin-top: 32px !important;
		}}
		body.admin-bar header.neo-header {{
			top: 32px !important;
		}}
		#wpadminbar .quicklinks {{
			display: flex;
			justify-content: space-between;
			align-items: center;
			height: 32px;
			max-width: 100%;
			padding: 0 12px;
		}}
		#wpadminbar ul {{
			list-style: none;
			margin: 0;
			padding: 0;
			display: flex;
			align-items: center;
			height: 32px;
		}}
		#wpadminbar li {{
			position: relative;
			display: inline-block;
			height: 32px;
		}}
		#wpadminbar .ab-item {{
			display: flex;
			align-items: center;
			gap: 6px;
			padding: 0 10px;
			color: #c3c4c7;
			font-size: 13px;
			height: 32px;
			line-height: 32px;
		}}
		#wpadminbar li:hover > .ab-item {{
			background: #2c3338;
			color: #72aee6;
		}}
		#wpadminbar .ab-btn-builder {{
			background: rgba(224, 38, 96, 0.2);
			color: #f472b6 !important;
			font-weight: 600;
			border-radius: 4px;
			margin: 0 4px;
			height: 24px;
			line-height: 24px;
		}}
		#wpadminbar .ab-sub-wrapper {{
			display: none;
			position: absolute;
			top: 32px;
			left: 0;
			min-width: 190px;
			background: #2c3338;
			box-shadow: 0 4px 14px rgba(0,0,0,0.6);
			border-radius: 0 0 6px 6px;
			z-index: 1000000;
			padding: 4px 0;
		}}
		#wpadminbar li:hover > .ab-sub-wrapper {{
			display: block;
		}}
		#wpadminbar .ab-sub-wrapper ul {{
			display: flex;
			flex-direction: column;
			height: auto;
		}}
		#wpadminbar .ab-sub-wrapper li {{
			width: 100%;
			height: 28px;
		}}
		#wpadminbar .ab-sub-wrapper .ab-item {{
			height: 28px;
			line-height: 28px;
			padding: 0 14px;
			font-size: 12px;
		}}
		#wpadminbar .ab-top-secondary {{
			margin-left: auto;
		}}
		#wpadminbar #wp-admin-bar-my-account .ab-sub-wrapper {{
			right: 0;
			left: auto;
		}}

		/* ==========================================================================
		   Elementor Theme Builder, Flexbox Container, Menu, Form & Modal System
		   ========================================================================== */
		html.elementor-theme-active body {{
			background-color: #ffffff;
			color: var(--e-global-color-text, #1F1C17);
		}}

		.np-elementor-header {{
			width: 100%;
			background: #ffffff;
			position: relative;
		}}

		.np-elementor-footer {{
			width: 100%;
			position: relative;
		}}

		.np-elementor-footer section.np-section {{
			padding-left: max(24px, calc((100% - 960px) / 2)) !important;
			padding-right: max(24px, calc((100% - 960px) / 2)) !important;
			border-bottom: none;
		}}

		.np-visual-canvas .np-container,
		.np-elementor-header .np-container,
		.np-elementor-footer .np-container,
		.np-popup-modal .np-container {{
			max-width: 100%;
			margin: 0;
			position: relative;
			gap: var(--e-widgets-space, 20px);
			background-size: cover;
			background-position: center;
			background-repeat: no-repeat;
		}}

		.np-container.np-container-boxed {{
			max-width: 100%;
			width: 100%;
		}}

		.np-container-boxed > .np-container-inner {{
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
		}}

		.np-container[style*="flex-direction: row"] > .np-container:not([style*="width:"]),
		.np-container[style*="flex-direction: row"] > .np-container-inner > .np-container:not([style*="width:"]) {{
			flex: 1 1 0%;
			min-width: 0;
		}}

		html.elementor-theme-active .np-image {{
			border-radius: 0;
		}}

		html.elementor-theme-active .np-imagebox,
		html.elementor-theme-active .np-iconbox {{
			background: transparent;
			border: none;
			border-radius: 0;
			padding: 0;
			align-items: center;
		}}

		html.elementor-theme-active .np-imagebox-title,
		html.elementor-theme-active .np-iconbox-title {{
			color: var(--e-global-color-secondary, #1F1C17);
			font-family: var(--e-global-typography-primary-font-family, 'Fira Sans', sans-serif);
		}}

		html.elementor-theme-active .np-imagebox-desc,
		html.elementor-theme-active .np-iconbox-desc {{
			color: var(--e-global-color-text, #1F1C17);
		}}

		html.elementor-theme-active .np-divider {{
			border-top-color: rgba(31, 28, 23, 0.15);
		}}

		html.elementor-theme-active .np-accordion-item {{
			background: #ffffff;
			border: 1px solid #d4d4d8;
			border-radius: 4px;
		}}

		html.elementor-theme-active .np-accordion-item summary {{
			color: var(--e-global-color-secondary, #3A3633);
			font-family: var(--e-global-typography-primary-font-family, 'Fira Sans', sans-serif);
		}}

		html.elementor-theme-active .np-accordion-body {{
			color: var(--e-global-color-text, #1F1C17);
			font-family: var(--e-global-typography-text-font-family, 'Fira Sans', sans-serif);
		}}

		.np-nav-menu .np-nav-menu-list,
		.np-nav-menu .np-nav-list {{
			list-style: none;
			margin: 0;
			padding: 0;
			display: flex;
			flex-wrap: wrap;
			gap: 24px;
			align-items: center;
		}}

		.np-nav-menu.np-nav-align-right .np-nav-menu-list {{
			justify-content: flex-end;
		}}

		.np-nav-menu.np-nav-align-center .np-nav-menu-list {{
			justify-content: center;
		}}

		.np-nav-menu.np-nav-layout-vertical .np-nav-menu-list,
		.np-nav-menu.np-nav-layout-vertical .np-nav-list {{
			flex-direction: column;
			align-items: flex-start;
			gap: 8px;
		}}

		.np-nav-menu .np-nav-menu-item {{
			display: inline-flex;
			align-items: center;
		}}

		.np-elementor-header .np-nav-menu.np-nav-layout-horizontal .np-nav-menu-item + .np-nav-menu-item::before {{
			content: "";
			display: inline-block;
			width: 1px;
			height: 15px;
			background: currentColor;
			opacity: 0.65;
			margin-right: 24px;
		}}

		.np-nav-menu .np-nav-menu-link,
		.np-nav-menu .np-nav-link {{
			font-family: inherit;
			font-size: inherit;
			font-weight: inherit;
			color: inherit;
			text-decoration: none;
			transition: color 0.15s ease, opacity 0.15s ease;
		}}

		.np-nav-menu .np-nav-menu-link:hover,
		.np-nav-menu .np-nav-link:hover {{
			color: var(--e-global-color-primary, #9CC13C);
		}}

		.np-iconlist,
		.np-icon-list {{
			list-style: none;
			margin: 0;
			padding: 0;
			display: flex;
			flex-direction: column;
			gap: 8px;
		}}

		.np-iconlist-item,
		.np-icon-list-item {{
			display: flex;
			align-items: center;
			gap: 10px;
			line-height: 1.65;
		}}

		.np-iconlist-icon,
		.np-icon-list-item i {{
			color: var(--e-global-color-primary, #9CC13C);
			font-size: 16px;
			width: 18px;
			text-align: center;
			flex-shrink: 0;
			display: inline-flex;
			align-items: center;
			justify-content: center;
		}}

		.np-iconlist-item a,
		.np-icon-list-link {{
			display: inline-flex;
			align-items: center;
			gap: 10px;
			color: inherit;
			text-decoration: none;
		}}

		.np-iconlist-item a:hover,
		.np-icon-list-link:hover {{
			color: var(--e-global-color-primary, #9CC13C);
		}}

		.np-social-icons {{
			display: flex;
			align-items: center;
			justify-content: center;
			gap: 36px;
			flex-wrap: wrap;
		}}

		.np-social-icon {{
			display: inline-flex;
			align-items: center;
			justify-content: center;
			width: 38px;
			height: 38px;
			border-radius: 50%;
			background: var(--e-global-color-secondary, #1F1C17);
			color: #ffffff !important;
			font-size: 17px;
			transition: opacity 0.15s ease, transform 0.15s ease;
		}}

		.np-social-icon:hover {{
			opacity: 0.85;
			transform: translateY(-1px);
		}}

		.np-elementor-form-grid {{
			display: flex;
			flex-wrap: wrap;
			gap: 16px;
			width: 100%;
		}}

		.np-elementor-form-grid .np-field-group {{
			display: flex;
			flex-direction: column;
			gap: 6px;
		}}

		.np-elementor-form-grid .np-field-w100 {{
			width: 100%;
		}}

		.np-elementor-form-grid .np-field-w50 {{
			width: calc(50% - 8px);
		}}

		.np-elementor-form-grid .np-field-w33 {{
			width: calc(33.333% - 11px);
		}}

		.np-elementor-form-grid .np-field-w25 {{
			width: calc(25% - 12px);
		}}

		.np-elementor-form-grid .np-field-label {{
			font-family: var(--e-global-typography-primary-font-family, 'Fira Sans', sans-serif);
			font-weight: 500;
			font-size: 15px;
			color: var(--e-global-color-secondary, #1F1C17);
		}}

		.np-elementor-form-grid .np-input,
		.np-elementor-form-grid .np-textarea,
		.np-elementor-form-grid .np-form-input {{
			width: 100%;
			padding: 12px 16px;
			border: 1px solid #d4d4d8;
			border-radius: 4px;
			background: #ffffff;
			color: #1F1C17;
			font-family: inherit;
			font-size: 15px;
		}}

		.np-elementor-form-grid .np-field-subgroup,
		.np-elementor-form-grid .np-Subgroup {{
			display: flex;
			flex-direction: column;
			gap: 8px;
			padding: 6px 0;
		}}

		.np-elementor-form-grid .np-field-subgroup.np-Subgroup-inline,
		.np-elementor-form-grid .np-Subgroup.np-Subgroup-inline {{
			flex-direction: row;
			flex-wrap: wrap;
			gap: 18px;
		}}

		.np-elementor-form-grid .np-option-label,
		.np-elementor-form-grid .np-checkbox-label {{
			display: inline-flex;
			align-items: center;
			gap: 8px;
			font-size: 15px;
			cursor: pointer;
		}}

		.np-popup-modal, .np-lightbox-modal {{
			position: fixed;
			inset: 0;
			z-index: 99999;
			display: none;
			align-items: center;
			justify-content: center;
			padding: 24px;
		}}

		.np-popup-modal.active, .np-lightbox-modal.active {{
			display: flex;
		}}

		.np-popup-backdrop, .np-lightbox-backdrop {{
			position: absolute;
			inset: 0;
			background: rgba(0, 0, 0, 0.72);
			backdrop-filter: blur(3px);
		}}

		.np-popup-dialog {{
			position: relative;
			z-index: 1;
			background: #ffffff;
			border-radius: 8px;
			max-width: 680px;
			width: 100%;
			max-height: 90vh;
			overflow-y: auto;
			box-shadow: 0 24px 60px rgba(0, 0, 0, 0.35);
			padding: 32px;
		}}

		.np-popup-close, .np-lightbox-close {{
			position: absolute;
			top: 12px;
			right: 16px;
			z-index: 2;
			background: transparent;
			border: none;
			font-size: 28px;
			line-height: 1;
			color: #1F1C17;
			cursor: pointer;
		}}

		.np-lightbox-close {{
			color: #ffffff;
			font-size: 36px;
			top: 20px;
			right: 28px;
		}}

		.np-lightbox-content {{
			position: relative;
			z-index: 1;
			max-width: 92vw;
			max-height: 88vh;
		}}

		.np-lightbox-content img {{
			max-width: 92vw;
			max-height: 88vh;
			object-fit: contain;
			border-radius: 4px;
			box-shadow: 0 20px 50px rgba(0,0,0,0.5);
			display: block;
		}}

		@media (min-width: 1025px) {{
			.np-hide-desktop {{
				display: none !important;
			}}
		}}

		@media (min-width: 768px) and (max-width: 1024px) {{
			.np-hide-tablet {{
				display: none !important;
			}}
		}}

		@media (max-width: 767px) {{
			.np-hide-mobile {{
				display: none !important;
			}}
			.np-elementor-form-grid .np-field-w50 {{
				width: 100%;
			}}
			.np-container[style*="flex-direction: row"],
			.np-container[style*="flex-direction: row"] > .np-container-inner {{
				flex-wrap: wrap;
			}}
			.np-container[style*="flex-direction: row"] > .np-container,
			.np-container[style*="flex-direction: row"] > .np-container-inner > .np-container {{
				width: 100% !important;
				flex: 1 1 100% !important;
			}}
		}}

		@media (max-width: 900px) {{
			.footer-grid, .np-grid {{
				grid-template-columns: 1fr !important;
			}}
			nav.neo-nav {{
				display: none;
			}}
		}}
"##,
        brand_color = brand_color
    )
}
