//! Public SSR Base & Reset Stylesheet (`templates/public/styles/base.rs`).
//!
//! Renders CSS variables, reset, body, global header, global footer,
//! and WordPress live admin bar.

/// Renders the base CSS rules interpolated with the site's active `brand_color`.
pub fn render_base_css(brand_color: &str) -> String {
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
			--font-sans: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
			--font-mono: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
		}}

		*, *::before, *::after {{
			box-sizing: border-box;
			margin: 0;
			padding: 0;
		}}

		html, body {{
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
		header.day-header {{
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

		.logo-brand .badge-day {{
			background: linear-gradient(135deg, #2563eb 0%, #ff4f00 100%);
			color: #ffffff;
			font-size: 10px;
			font-weight: 800;
			padding: 3px 7px;
			border-radius: 4px;
			letter-spacing: 0.05em;
			text-transform: uppercase;
		}}

		nav.day-nav {{
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

		@media (max-width: 768px) {{
			.header-search {{
				display: none;
			}}
			.header-container {{
				padding: 10px 16px;
				gap: 12px;
			}}
		}}

		main.day-main {{
			flex: 1;
		}}

		/* Global Footer */
		footer.day-footer {{
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
		body.admin-bar header.day-header {{
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
"##,
        brand_color = brand_color
    )
}
