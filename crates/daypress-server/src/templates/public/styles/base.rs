//! Public SSR Base & Reset Stylesheet (`templates/public/styles/base.rs`).
//!
//! Renders CSS variables, reset, body, global header, global footer,
//! and WordPress live admin bar across Daypress Daylight (default), Daypress Modern,
//! and Daypress Night themes.

/// Renders the base CSS rules interpolated with the site's active `brand_color`.
pub fn render_base_css(brand_color: &str) -> String {
    format!(
        r##"
		@font-face {{ font-family: "Newsreader"; src: url("/static/fonts/Newsreader-Variable.woff2") format("woff2"); font-weight: 200 800; font-style: normal; font-display: swap; }}
		@font-face {{ font-family: "Newsreader"; src: url("/static/fonts/Newsreader-Italic-Variable.woff2") format("woff2"); font-weight: 200 800; font-style: italic; font-display: swap; }}
		@font-face {{ font-family: "Schibsted Grotesk"; src: url("/static/fonts/SchibstedGrotesk-Variable.woff2") format("woff2"); font-weight: 400 900; font-style: normal; font-display: swap; }}
		@font-face {{ font-family: "IBM Plex Mono"; src: url("/static/fonts/IBMPlexMono-Regular.woff2") format("woff2"); font-weight: 400; font-style: normal; font-display: swap; }}
		@font-face {{ font-family: "IBM Plex Mono"; src: url("/static/fonts/IBMPlexMono-Medium.woff2") format("woff2"); font-weight: 500; font-style: normal; font-display: swap; }}

		:root {{
			--bg-base: #f3eee4;
			--bg-panel: #fbf8f2;
			--bg-card: #fbf8f2;
			--bg-card-hover: #e7e0d1;
			--header-bg: rgba(251, 248, 242, 0.92);
			--footer-bg: #1a1c1f;
			--footer-text: #efe9de;
			--footer-muted: #a7a297;
			--footer-border: rgba(239, 233, 222, 0.16);
			--border: #d4cdc0;
			--border-subtle: #e2dacb;
			--border-focus: {brand_color};
			--text-main: #16171a;
			--text-muted: #5a5852;
			--text-dim: #78756e;
			--accent: {brand_color};
			--accent-hover: #2a3bd0;
			--accent-glow: rgba(168, 64, 27, 0.16);
			--dp-color-heading: #16171a;
			--dp-color-text: #2c2b28;
			--dp-font-heading: "Newsreader", Georgia, "Times New Roman", serif;
			--dp-font-body: "Schibsted Grotesk", system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
			--dp-heading-weight: 500;
			--dp-button-bg: {brand_color};
			--dp-button-color: #fff8f0;
			--dp-button-hover-bg: #16171a;
			--e-global-color-primary: {brand_color};
			--e-global-color-secondary: #16171a;
			--e-global-color-text: #2c2b28;
			--e-global-color-accent: #2a3bd0;
			--font-serif: "Newsreader", Georgia, "Times New Roman", serif;
			--font-sans: system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
			--font-mono: "IBM Plex Mono", ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace;
		}}

		html.daypress-modern {{
			--bg-base: #f8fafc;
			--bg-panel: #ffffff;
			--bg-card: #ffffff;
			--bg-card-hover: #f1f5f9;
			--header-bg: rgba(255, 255, 255, 0.92);
			--footer-bg: #0f172a;
			--footer-text: #f8fafc;
			--footer-muted: #94a3b8;
			--footer-border: #1e293b;
			--border: #e2e8f0;
			--border-subtle: #f1f5f9;
			--text-main: #0f172a;
			--text-muted: #475569;
			--text-dim: #64748b;
			--dp-color-heading: #0f172a;
			--dp-color-text: #334155;
			--dp-font-heading: "Schibsted Grotesk", system-ui, -apple-system, sans-serif;
			--dp-heading-weight: 700;
		}}

		html.dark {{
			--bg-base: #0b0d12;
			--bg-panel: #11151d;
			--bg-card: #151a24;
			--bg-card-hover: #1e2533;
			--header-bg: rgba(17, 21, 29, 0.9);
			--footer-bg: #07090d;
			--footer-text: #f8fafc;
			--footer-muted: #94a3b8;
			--footer-border: #1e293b;
			--border: #232b3a;
			--border-subtle: #1a202c;
			--text-main: #f8fafc;
			--text-muted: #94a3b8;
			--text-dim: #64748b;
			--dp-color-heading: #f8fafc;
			--dp-color-text: #cbd5e1;
			--dp-font-heading: "Newsreader", Georgia, serif;
			--dp-heading-weight: 500;
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
			background: var(--header-bg);
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
			font-family: var(--font-serif);
			font-weight: 600;
			font-size: 20px;
			letter-spacing: -0.02em;
			color: var(--text-main);
			white-space: nowrap;
		}}

		.logo-brand .badge-day {{
			background: var(--accent);
			color: #fff8f0;
			font-family: var(--font-mono);
			font-size: 10px;
			font-weight: 600;
			padding: 3px 7px;
			border-radius: 4px;
			letter-spacing: 0.06em;
			text-transform: uppercase;
		}}

		nav.day-nav {{
			display: flex;
			align-items: center;
			flex-wrap: wrap;
			gap: 18px;
		}}

		.nav-link {{
			font-size: 14px;
			font-weight: 500;
			color: var(--text-muted);
			transition: color 0.15s ease;
		}}

		.nav-link:hover {{
			color: var(--text-main);
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
			padding: 5px 10px;
		}}

		.header-search input {{
			background: transparent;
			border: none;
			color: var(--text-main);
			font-size: 13px;
			width: 140px;
			outline: none;
		}}

		.btn-login {{
			font-size: 13px;
			font-weight: 600;
			color: var(--text-main);
			background: var(--bg-card);
			padding: 6px 13px;
			border-radius: 6px;
			border: 1px solid var(--border);
		}}

		.btn-login:hover {{
			color: #fff8f0;
			background: var(--accent);
			border-color: var(--accent);
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
			background: var(--footer-bg);
			border-top: 1px solid var(--border);
			padding: 56px 0 36px 0;
			color: var(--footer-muted);
			font-size: 13.5px;
		}}

		footer.day-footer .logo-brand {{
			color: var(--footer-text);
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
			color: var(--footer-muted);
			margin-top: 12px;
			max-width: 420px;
			line-height: 1.6;
		}}

		.footer-col-title {{
			color: var(--footer-text);
			font-size: 11.5px;
			font-weight: 700;
			text-transform: uppercase;
			letter-spacing: 0.08em;
			margin-bottom: 16px;
		}}

		.footer-links {{
			display: flex;
			flex-direction: column;
			gap: 8px;
		}}

		.footer-links a {{
			color: var(--footer-muted);
		}}

		.footer-links a:hover {{
			color: var(--footer-text);
		}}

		.footer-bottom {{
			max-width: 1200px;
			margin: 0 auto;
			padding: 24px 24px 0 24px;
			border-top: 1px solid var(--footer-border);
			display: flex;
			justify-content: space-between;
			align-items: center;
			font-size: 12.5px;
			color: var(--footer-muted);
		}}

		.engine-chip {{
			display: inline-flex;
			align-items: center;
			gap: 6px;
			background: rgba(255, 255, 255, 0.05);
			border: 1px solid var(--footer-border);
			padding: 3px 10px;
			border-radius: 4px;
			color: var(--footer-text);
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
