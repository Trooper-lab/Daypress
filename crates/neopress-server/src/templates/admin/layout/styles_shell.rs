//! Admin CSS styles: Shell, Sidebar, and Topbar.

pub fn get_styles_shell() -> &'static str {
    r#"
		:root {
			--font-sans: -apple-system, BlinkMacSystemFont, "Inter", "Plus Jakarta Sans", "Segoe UI", Roboto, sans-serif;
			--font-mono: "JetBrains Mono", "SF Mono", Consolas, Menlo, monospace;
			--bg-app: #f8fafc;
			--bg-surface: #ffffff;
			--bg-surface-alt: #f1f5f9;
			--border-subtle: #e2e8f0;
			--border-hover: #cbd5e1;
			--text-main: #0f172a;
			--text-muted: #64748b;
			--text-dim: #94a3b8;
			--primary: #2563eb;
			--primary-hover: #1d4ed8;
			--sidebar-w: 240px;
			--topbar-h: 56px;
		}

		*, *::before, *::after {
			box-sizing: border-box;
			margin: 0;
			padding: 0;
		}

		body {
			font-family: var(--font-sans);
			background: var(--bg-app);
			color: var(--text-main);
			font-size: 13.5px;
			line-height: 1.5;
			-webkit-font-smoothing: antialiased;
		}

		#neo-app {
			display: flex;
			min-height: 100vh;
		}

		/* ==================== 2026 SIDEBAR ==================== */
		#neo-sidebar {
			width: var(--sidebar-w);
			background: #090d16;
			border-right: 1px solid #1e293b;
			color: #f8fafc;
			position: fixed;
			top: 0;
			bottom: 0;
			left: 0;
			display: flex;
			flex-direction: column;
			z-index: 100;
			overflow-y: auto;
		}

		.sidebar-header {
			padding: 18px 20px 14px 20px;
			border-bottom: 1px solid rgba(255, 255, 255, 0.06);
		}

		.brand-link {
			display: flex;
			align-items: center;
			gap: 10px;
			text-decoration: none;
			color: #ffffff;
		}

		.brand-icon {
			width: 28px;
			height: 28px;
			border-radius: 7px;
		}

		.brand-name {
			font-size: 16px;
			font-weight: 700;
			letter-spacing: -0.4px;
		}

		.brand-badge {
			font-family: var(--font-mono);
			font-size: 9px;
			font-weight: 700;
			padding: 2px 5px;
			background: rgba(56, 189, 248, 0.15);
			color: #38bdf8;
			border: 1px solid rgba(56, 189, 248, 0.25);
			border-radius: 4px;
			letter-spacing: 0.5px;
		}

		/* Engine Health Pill */
		.sidebar-engine-pill {
			margin: 12px 14px;
			padding: 8px 10px;
			background: rgba(255, 255, 255, 0.03);
			border: 1px solid rgba(255, 255, 255, 0.08);
			border-radius: 7px;
			display: flex;
			align-items: center;
			gap: 8px;
			text-decoration: none;
			transition: background 0.15s;
		}

		.sidebar-engine-pill:hover {
			background: rgba(255, 255, 255, 0.06);
		}

		.pulse-dot {
			width: 7px;
			height: 7px;
			background: #10b981;
			border-radius: 50%;
			box-shadow: 0 0 6px #10b981;
			animation: pulse 2s infinite ease-in-out;
		}

		@keyframes pulse {
			0% { opacity: 0.5; }
			50% { opacity: 1; }
			100% { opacity: 0.5; }
		}

		.engine-meta {
			display: flex;
			flex-direction: column;
		}

		.engine-title {
			font-size: 11px;
			font-weight: 600;
			color: #e2e8f0;
			line-height: 1.2;
		}

		.engine-sub {
			font-size: 10px;
			color: #94a3b8;
			font-family: var(--font-mono);
		}

		/* Nav Section */
		.sidebar-nav {
			flex: 1;
			padding: 8px 0;
		}

		.nav-section-title {
			font-size: 10px;
			font-weight: 700;
			letter-spacing: 0.08em;
			color: #64748b;
			padding: 14px 18px 4px 18px;
		}

		.nav-item {
			display: flex;
			align-items: center;
			gap: 10px;
			padding: 7px 14px;
			margin: 1px 10px;
			border-radius: 6px;
			text-decoration: none;
			color: #94a3b8;
			font-size: 13px;
			font-weight: 500;
			transition: all 0.15s ease;
		}

		.nav-item:hover {
			background: rgba(255, 255, 255, 0.06);
			color: #f8fafc;
		}

		.nav-item.active {
			background: #1e293b;
			color: #ffffff;
			font-weight: 600;
			box-shadow: inset 2px 0 0 #38bdf8;
		}

		.nav-icon {
			width: 17px;
			height: 17px;
			flex-shrink: 0;
			opacity: 0.85;
		}

		.nav-item.active .nav-icon {
			opacity: 1;
			color: #38bdf8;
		}

		.nav-badge {
			margin-left: auto;
			font-size: 10px;
			font-weight: 600;
			background: rgba(255, 255, 255, 0.08);
			padding: 1px 6px;
			border-radius: 10px;
			color: #cbd5e1;
		}

		/* Submenu hierarchy */
		.nav-group {
			position: relative;
			margin-bottom: 2px;
		}

		.nav-submenu {
			display: none;
			padding: 3px 0 5px 12px;
			margin: 2px 10px 4px 24px;
			border-left: 2px solid rgba(255, 255, 255, 0.1);
			flex-direction: column;
			gap: 2px;
		}

		.nav-group.open .nav-submenu,
		.nav-group:hover .nav-submenu {
			display: flex;
		}

		.nav-subitem {
			display: block;
			padding: 4px 10px;
			font-size: 12.5px;
			color: #94a3b8;
			text-decoration: none;
			border-radius: 4px;
			transition: all 0.15s ease;
		}

		.nav-subitem:hover {
			color: #f8fafc;
			background: rgba(255, 255, 255, 0.06);
		}

		.nav-subitem.active {
			color: #38bdf8;
			background: rgba(56, 189, 248, 0.12);
			font-weight: 600;
		}

		/* Sidebar Footer */
		.sidebar-footer {
			padding: 14px 16px;
			border-top: 1px solid rgba(255, 255, 255, 0.06);
			background: rgba(0, 0, 0, 0.2);
			display: flex;
			flex-direction: column;
			gap: 10px;
		}

		.view-site-btn {
			display: flex;
			align-items: center;
			justify-content: space-between;
			padding: 6px 10px;
			background: rgba(255, 255, 255, 0.05);
			border: 1px solid rgba(255, 255, 255, 0.08);
			border-radius: 6px;
			text-decoration: none;
			color: #cbd5e1;
			font-size: 12px;
			font-weight: 500;
			transition: background 0.15s;
		}

		.view-site-btn:hover {
			background: rgba(255, 255, 255, 0.1);
			color: #ffffff;
		}

		.user-profile {
			display: flex;
			align-items: center;
			gap: 10px;
		}

		.user-avatar {
			width: 28px;
			height: 28px;
			border-radius: 50%;
			background: #3b82f6;
			color: #fff;
			font-size: 12px;
			font-weight: 700;
			display: flex;
			align-items: center;
			justify-content: center;
		}

		.user-details {
			display: flex;
			flex-direction: column;
			line-height: 1.2;
		}

		.user-name {
			font-size: 12px;
			font-weight: 600;
			color: #f8fafc;
		}

		.user-role {
			font-size: 10px;
			color: #64748b;
		}

		/* ==================== 2026 MAIN CONTENT ==================== */
		#neo-main {
			margin-left: var(--sidebar-w);
			flex: 1;
			min-width: 0;
			display: flex;
			flex-direction: column;
		}

		#neo-topbar {
			height: var(--topbar-h);
			background: rgba(255, 255, 255, 0.88);
			backdrop-filter: blur(12px);
			border-bottom: 1px solid var(--border-subtle);
			position: sticky;
			top: 0;
			z-index: 50;
			display: flex;
			align-items: center;
			justify-content: space-between;
			padding: 0 32px;
		}
"#
}
