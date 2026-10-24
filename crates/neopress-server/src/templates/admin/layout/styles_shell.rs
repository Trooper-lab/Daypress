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
			--sidebar-w: 200px;
			--topbar-h: 32px;
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
			padding-top: 32px;
		}

		/* ==================== UNIFIED WP ADMIN TOP BAR & SIDEBAR ==================== */
		#wpadminbar {
			position: fixed;
			top: 0;
			left: 0;
			right: 0;
			height: 32px;
			background: #141923;
			color: #e2e8f0;
			z-index: 99999;
			font-family: var(--font-sans);
			font-size: 13px;
			user-select: none;
		}

		#wpadminbar .ab-top-menu-wrap {
			display: flex;
			align-items: center;
			justify-content: space-between;
			height: 100%;
			padding: 0 4px;
		}

		#wpadminbar .ab-top-menu {
			display: flex;
			align-items: center;
			list-style: none;
			height: 100%;
			margin: 0;
			padding: 0;
		}

		#wpadminbar .ab-top-menu > li {
			position: relative;
			height: 32px;
			display: flex;
			align-items: center;
		}

		#wpadminbar .ab-item {
			display: inline-flex;
			align-items: center;
			gap: 6px;
			height: 32px;
			padding: 0 10px;
			color: #e2e8f0;
			text-decoration: none;
			font-size: 13px;
			line-height: 32px;
			transition: background 0.12s, color 0.12s;
		}

		#wpadminbar .ab-top-menu > li:hover > .ab-item {
			background: #1e2636;
			color: #60a5fa;
		}

		#wpadminbar .ab-avatar {
			width: 18px;
			height: 18px;
			border-radius: 3px;
			background: #3858e9;
			color: #fff;
			font-size: 10px;
			font-weight: 700;
			display: inline-flex;
			align-items: center;
			justify-content: center;
		}

		#wpadminbar .ab-sub-wrapper {
			display: none;
			position: absolute;
			top: 32px;
			left: 0;
			min-width: 180px;
			background: #1e2636;
			box-shadow: 0 6px 16px rgba(0, 0, 0, 0.45);
			padding: 4px 0;
			z-index: 100000;
		}

		#wpadminbar .ab-sub-wrapper.ab-sub-right {
			left: auto;
			right: 0;
		}

		#wpadminbar .menupop:hover > .ab-sub-wrapper {
			display: block;
		}

		#wpadminbar .ab-submenu {
			list-style: none;
			margin: 0;
			padding: 0;
		}

		#wpadminbar .ab-submenu .ab-item {
			display: block;
			height: auto;
			line-height: 1.4;
			padding: 6px 14px;
			color: #cbd5e1;
			font-size: 12.5px;
		}

		#wpadminbar .ab-submenu .ab-item:hover {
			background: #141923;
			color: #60a5fa;
		}

		/* ==================== UNIFIED SIDEBAR ==================== */
		#neo-sidebar {
			width: var(--sidebar-w);
			background: #141923;
			color: #f8fafc;
			position: fixed;
			top: 32px;
			bottom: 0;
			left: 0;
			display: flex;
			flex-direction: column;
			z-index: 100;
			overflow-y: auto;
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
			padding: 14px 14px 4px 14px;
		}

		.nav-item {
			display: flex;
			align-items: center;
			gap: 10px;
			padding: 9px 14px;
			margin: 0;
			border-radius: 0;
			text-decoration: none;
			color: #cbd5e1;
			font-size: 13.5px;
			font-weight: 500;
			position: relative;
			transition: background 0.12s ease, color 0.12s ease;
		}

		.nav-item:hover {
			background: #1e2636;
			color: #60a5fa;
		}

		.nav-item.active {
			background: #3858e9;
			color: #ffffff;
			font-weight: 600;
		}

		.nav-item.active::after {
			content: "";
			position: absolute;
			right: 0;
			top: 50%;
			transform: translateY(-50%);
			width: 0;
			height: 0;
			border-top: 8px solid transparent;
			border-bottom: 8px solid transparent;
			border-right: 8px solid var(--bg-app);
		}

		.nav-icon {
			width: 17px;
			height: 17px;
			flex-shrink: 0;
			opacity: 0.85;
		}

		.nav-item:hover .nav-icon {
			opacity: 1;
			color: #60a5fa;
		}

		.nav-item.active .nav-icon {
			opacity: 1;
			color: #ffffff;
		}

		.nav-badge {
			margin-left: auto;
			font-size: 10px;
			font-weight: 600;
			background: #3858e9;
			padding: 1px 6px;
			border-radius: 10px;
			color: #ffffff;
		}

		/* Submenu hierarchy: inline underneath when selected (.open), side-preview flyout on hover when not selected */
		.nav-group {
			position: relative;
			margin-bottom: 0;
		}

		.nav-submenu {
			display: none;
			flex-direction: column;
			gap: 1px;
		}

		.nav-group.open > .nav-submenu {
			display: flex;
			position: static;
			background: #0b0e14;
			padding: 6px 0;
			margin: 0;
			border-left: none;
		}

		.nav-group:not(.open):hover > .nav-submenu {
			display: flex;
			position: fixed;
			left: var(--sidebar-w);
			top: 120px;
			min-width: 185px;
			padding: 6px 0;
			margin: 0;
			background: #1e2636;
			box-shadow: 6px 8px 20px rgba(0, 0, 0, 0.45);
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
			padding: 5px 14px 5px 18px;
			font-size: 13px;
			color: #94a3b8;
			text-decoration: none;
			white-space: nowrap;
			transition: color 0.12s ease, background 0.12s ease;
		}

		.nav-subitem:hover {
			color: #60a5fa;
			background: rgba(255, 255, 255, 0.04);
		}

		.nav-subitem.active {
			color: #ffffff;
			font-weight: 700;
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
			top: 32px;
			z-index: 50;
			display: flex;
			align-items: center;
			justify-content: space-between;
			padding: 0 32px;
		}
"#
}
