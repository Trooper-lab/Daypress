use crate::models::WpSettingsResponse;

pub fn render_wp_admin_shell(
    current_tab: &str,
    page_title: &str,
    body_content: &str,
    settings: &WpSettingsResponse,
) -> String {
    let site_name = if settings.title.is_empty() {
        "Neopress"
    } else {
        &settings.title
    };

    let is_dash = if current_tab == "dashboard" || current_tab == "updates" {
        "active"
    } else {
        ""
    };
    let is_dash_open = if !is_dash.is_empty() { "open" } else { "" };
    let sub_dash_home = if current_tab == "dashboard" || current_tab == "home" {
        "active"
    } else {
        ""
    };
    let sub_dash_updates = if current_tab == "updates" {
        "active"
    } else {
        ""
    };

    let is_posts = if current_tab == "posts"
        || current_tab == "post-new"
        || current_tab == "categories"
        || current_tab == "tags"
    {
        "active"
    } else {
        ""
    };
    let is_posts_open = if !is_posts.is_empty() { "open" } else { "" };
    let sub_posts_all = if current_tab == "posts" { "active" } else { "" };
    let sub_posts_new = if current_tab == "post-new" {
        "active"
    } else {
        ""
    };
    let sub_posts_cats = if current_tab == "categories" {
        "active"
    } else {
        ""
    };
    let sub_posts_tags = if current_tab == "tags" { "active" } else { "" };

    let is_media = if current_tab == "media" || current_tab == "media-new" {
        "active"
    } else {
        ""
    };
    let is_media_open = if !is_media.is_empty() { "open" } else { "" };
    let sub_media_lib = if current_tab == "media" { "active" } else { "" };
    let sub_media_new = if current_tab == "media-new" {
        "active"
    } else {
        ""
    };

    let is_pages = if current_tab == "pages" || current_tab == "page-new" {
        "active"
    } else {
        ""
    };
    let is_pages_open = if !is_pages.is_empty() { "open" } else { "" };
    let sub_pages_all = if current_tab == "pages" { "active" } else { "" };
    let sub_pages_new = if current_tab == "page-new" {
        "active"
    } else {
        ""
    };

    let is_comments = if current_tab == "comments" {
        "active"
    } else {
        ""
    };

    let is_appearance = if current_tab == "appearance"
        || current_tab == "themes"
        || current_tab == "customize"
        || current_tab == "widgets"
        || current_tab == "menus"
        || current_tab == "theme-editor"
    {
        "active"
    } else {
        ""
    };
    let is_app_open = if !is_appearance.is_empty() {
        "open"
    } else {
        ""
    };
    let sub_app_themes = if current_tab == "themes" || current_tab == "appearance" {
        "active"
    } else {
        ""
    };
    let sub_app_cust = if current_tab == "customize" {
        "active"
    } else {
        ""
    };
    let sub_app_widg = if current_tab == "widgets" {
        "active"
    } else {
        ""
    };
    let sub_app_menus = if current_tab == "menus" { "active" } else { "" };
    let sub_app_editor = if current_tab == "theme-editor" {
        "active"
    } else {
        ""
    };

    let is_plugins = if current_tab == "plugins"
        || current_tab == "plugin-install"
        || current_tab == "plugin-editor"
        || current_tab == "plugin-settings"
    {
        "active"
    } else {
        ""
    };
    let is_plugins_open = if !is_plugins.is_empty() { "open" } else { "" };
    let sub_plug_list = if current_tab == "plugins" || current_tab == "plugin-settings" {
        "active"
    } else {
        ""
    };
    let sub_plug_new = if current_tab == "plugin-install" {
        "active"
    } else {
        ""
    };
    let sub_plug_editor = if current_tab == "plugin-editor" {
        "active"
    } else {
        ""
    };

    let is_seo = if current_tab == "seo" { "active" } else { "" };
    let is_forms = if current_tab == "forms" { "active" } else { "" };
    let is_backups = if current_tab == "backups" { "active" } else { "" };

    let is_users =
        if current_tab == "users" || current_tab == "user-new" || current_tab == "profile" {
            "active"
        } else {
            ""
        };
    let is_users_open = if !is_users.is_empty() { "open" } else { "" };
    let sub_users_all = if current_tab == "users" { "active" } else { "" };
    let sub_users_new = if current_tab == "user-new" {
        "active"
    } else {
        ""
    };
    let sub_users_prof = if current_tab == "profile" {
        "active"
    } else {
        ""
    };

    let is_tools = if current_tab == "tools"
        || current_tab == "import"
        || current_tab == "export"
        || current_tab == "site-health"
        || current_tab == "export-personal-data"
        || current_tab == "erase-personal-data"
    {
        "active"
    } else {
        ""
    };
    let is_tools_open = if !is_tools.is_empty() { "open" } else { "" };
    let sub_tools_avail = if current_tab == "tools" { "active" } else { "" };
    let sub_tools_import = if current_tab == "import" {
        "active"
    } else {
        ""
    };
    let sub_tools_export = if current_tab == "export" {
        "active"
    } else {
        ""
    };
    let sub_tools_health = if current_tab == "site-health" {
        "active"
    } else {
        ""
    };
    let sub_tools_exp_data = if current_tab == "export-personal-data" {
        "active"
    } else {
        ""
    };
    let sub_tools_era_data = if current_tab == "erase-personal-data" {
        "active"
    } else {
        ""
    };

    let is_settings = if current_tab.starts_with("settings")
        || current_tab == "writing"
        || current_tab == "reading"
        || current_tab == "discussion"
        || current_tab == "options-media"
        || current_tab == "permalinks"
        || current_tab == "privacy"
    {
        "active"
    } else {
        ""
    };
    let is_settings_open = if !is_settings.is_empty() { "open" } else { "" };
    let sub_set_gen = if current_tab == "settings" || current_tab == "settings-general" {
        "active"
    } else {
        ""
    };
    let sub_set_writ = if current_tab == "writing" || current_tab == "settings-writing" {
        "active"
    } else {
        ""
    };
    let sub_set_read = if current_tab == "reading" || current_tab == "settings-reading" {
        "active"
    } else {
        ""
    };
    let sub_set_disc = if current_tab == "discussion" || current_tab == "settings-discussion" {
        "active"
    } else {
        ""
    };
    let sub_set_media = if current_tab == "options-media" || current_tab == "settings-media" {
        "active"
    } else {
        ""
    };
    let sub_set_perm = if current_tab == "permalinks"
        || current_tab == "settings-permalink"
        || current_tab == "settings-permalinks"
    {
        "active"
    } else {
        ""
    };
    let sub_set_priv = if current_tab == "privacy" || current_tab == "settings-privacy" {
        "active"
    } else {
        ""
    };

    let is_engine = if current_tab == "engine" {
        "active"
    } else {
        ""
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en-US">
<head>
	<meta charset="UTF-8" />
	<title>{page_title} &bull; {site_name}</title>
	<meta name="viewport" content="width=device-width, initial-scale=1.0">
	<link rel="icon" type="image/svg+xml" href="/static/favicon.svg" />
	<style>
		:root {{
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
		}}

		*, *::before, *::after {{
			box-sizing: border-box;
			margin: 0;
			padding: 0;
		}}

		body {{
			font-family: var(--font-sans);
			background: var(--bg-app);
			color: var(--text-main);
			font-size: 13.5px;
			line-height: 1.5;
			-webkit-font-smoothing: antialiased;
		}}

		#neo-app {{
			display: flex;
			min-height: 100vh;
		}}

		/* ==================== 2026 SIDEBAR ==================== */
		#neo-sidebar {{
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
		}}

		.sidebar-header {{
			padding: 18px 20px 14px 20px;
			border-bottom: 1px solid rgba(255, 255, 255, 0.06);
		}}

		.brand-link {{
			display: flex;
			align-items: center;
			gap: 10px;
			text-decoration: none;
			color: #ffffff;
		}}

		.brand-icon {{
			width: 28px;
			height: 28px;
			border-radius: 7px;
		}}

		.brand-name {{
			font-size: 16px;
			font-weight: 700;
			letter-spacing: -0.4px;
		}}

		.brand-badge {{
			font-family: var(--font-mono);
			font-size: 9px;
			font-weight: 700;
			padding: 2px 5px;
			background: rgba(56, 189, 248, 0.15);
			color: #38bdf8;
			border: 1px solid rgba(56, 189, 248, 0.25);
			border-radius: 4px;
			letter-spacing: 0.5px;
		}}

		/* Engine Health Pill */
		.sidebar-engine-pill {{
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
		}}

		.sidebar-engine-pill:hover {{
			background: rgba(255, 255, 255, 0.06);
		}}

		.pulse-dot {{
			width: 7px;
			height: 7px;
			background: #10b981;
			border-radius: 50%;
			box-shadow: 0 0 6px #10b981;
			animation: pulse 2s infinite ease-in-out;
		}}

		@keyframes pulse {{
			0% {{ opacity: 0.5; }}
			50% {{ opacity: 1; }}
			100% {{ opacity: 0.5; }}
		}}

		.engine-meta {{
			display: flex;
			flex-direction: column;
		}}

		.engine-title {{
			font-size: 11px;
			font-weight: 600;
			color: #e2e8f0;
			line-height: 1.2;
		}}

		.engine-sub {{
			font-size: 10px;
			color: #94a3b8;
			font-family: var(--font-mono);
		}}

		/* Nav Section */
		.sidebar-nav {{
			flex: 1;
			padding: 8px 0;
		}}

		.nav-section-title {{
			font-size: 10px;
			font-weight: 700;
			letter-spacing: 0.08em;
			color: #64748b;
			padding: 14px 18px 4px 18px;
		}}

		.nav-item {{
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
		}}

		.nav-item:hover {{
			background: rgba(255, 255, 255, 0.06);
			color: #f8fafc;
		}}

		.nav-item.active {{
			background: #1e293b;
			color: #ffffff;
			font-weight: 600;
			box-shadow: inset 2px 0 0 #38bdf8;
		}}

		.nav-icon {{
			width: 17px;
			height: 17px;
			flex-shrink: 0;
			opacity: 0.85;
		}}

		.nav-item.active .nav-icon {{
			opacity: 1;
			color: #38bdf8;
		}}

		.nav-badge {{
			margin-left: auto;
			font-size: 10px;
			font-weight: 600;
			background: rgba(255, 255, 255, 0.08);
			padding: 1px 6px;
			border-radius: 10px;
			color: #cbd5e1;
		}}

		/* Submenu hierarchy */
		.nav-group {{
			position: relative;
			margin-bottom: 2px;
		}}

		.nav-submenu {{
			display: none;
			padding: 3px 0 5px 12px;
			margin: 2px 10px 4px 24px;
			border-left: 2px solid rgba(255, 255, 255, 0.1);
			flex-direction: column;
			gap: 2px;
		}}

		.nav-group.open .nav-submenu,
		.nav-group:hover .nav-submenu {{
			display: flex;
		}}

		.nav-subitem {{
			display: block;
			padding: 4px 10px;
			font-size: 12.5px;
			color: #94a3b8;
			text-decoration: none;
			border-radius: 4px;
			transition: all 0.15s ease;
		}}

		.nav-subitem:hover {{
			color: #f8fafc;
			background: rgba(255, 255, 255, 0.06);
		}}

		.nav-subitem.active {{
			color: #38bdf8;
			background: rgba(56, 189, 248, 0.12);
			font-weight: 600;
		}}

		/* Sidebar Footer */
		.sidebar-footer {{
			padding: 14px 16px;
			border-top: 1px solid rgba(255, 255, 255, 0.06);
			background: rgba(0, 0, 0, 0.2);
			display: flex;
			flex-direction: column;
			gap: 10px;
		}}

		.view-site-btn {{
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
		}}

		.view-site-btn:hover {{
			background: rgba(255, 255, 255, 0.1);
			color: #ffffff;
		}}

		.user-profile {{
			display: flex;
			align-items: center;
			gap: 10px;
		}}

		.user-avatar {{
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
		}}

		.user-details {{
			display: flex;
			flex-direction: column;
			line-height: 1.2;
		}}

		.user-name {{
			font-size: 12px;
			font-weight: 600;
			color: #f8fafc;
		}}

		.user-role {{
			font-size: 10px;
			color: #64748b;
		}}

		/* ==================== 2026 MAIN CONTENT ==================== */
		#neo-main {{
			margin-left: var(--sidebar-w);
			flex: 1;
			min-width: 0;
			display: flex;
			flex-direction: column;
		}}

		#neo-topbar {{
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
		}}

		.breadcrumbs {{
			display: flex;
			align-items: center;
			gap: 8px;
			font-size: 13px;
			font-weight: 500;
		}}

		.crumb-root {{
			color: var(--text-muted);
			text-decoration: none;
		}}

		.crumb-sep {{
			color: var(--text-dim);
		}}

		.crumb-active {{
			color: var(--text-main);
			font-weight: 600;
		}}

		.topbar-actions {{
			display: flex;
			align-items: center;
			gap: 12px;
		}}

		.latency-chip {{
			font-family: var(--font-mono);
			font-size: 11px;
			padding: 4px 9px;
			border-radius: 6px;
			background: #ecfdf5;
			color: #059669;
			border: 1px solid #a7f3d0;
			display: flex;
			align-items: center;
			gap: 6px;
		}}

		.latency-dot {{
			width: 6px;
			height: 6px;
			border-radius: 50%;
			background: #10b981;
		}}

		.btn-new-post {{
			background: #0f172a;
			color: #ffffff !important;
			text-decoration: none;
			font-size: 12.5px;
			font-weight: 600;
			padding: 6px 14px;
			border-radius: 6px;
			display: inline-flex;
			align-items: center;
			gap: 6px;
			transition: background 0.15s;
		}}

		.btn-new-post:hover {{
			background: #1e293b;
		}}

		#neo-canvas {{
			padding: 32px 36px 60px 36px;
			flex: 1;
		}}

		/* ==================== 2026 UI COMPONENTS ==================== */
		.page-header {{
			margin-bottom: 24px;
			display: flex;
			justify-content: space-between;
			align-items: flex-end;
		}}

		.page-title {{
			font-size: 22px;
			font-weight: 700;
			letter-spacing: -0.02em;
			color: var(--text-main);
			margin: 0;
		}}

		.page-subtitle {{
			font-size: 13.5px;
			color: var(--text-muted);
			margin-top: 4px;
		}}

		/* Modern Cards */
		.neo-card {{
			background: #ffffff;
			border: 1px solid var(--border-subtle);
			border-radius: 10px;
			box-shadow: 0 1px 3px rgba(0, 0, 0, 0.02);
			overflow: hidden;
			margin-bottom: 24px;
		}}

		.neo-card-header {{
			padding: 16px 20px;
			border-bottom: 1px solid var(--border-subtle);
			display: flex;
			justify-content: space-between;
			align-items: center;
		}}

		.neo-card-title {{
			font-size: 14px;
			font-weight: 600;
			color: var(--text-main);
			margin: 0;
		}}

		.neo-card-body {{
			padding: 20px;
		}}

		/* KPI Metric Grid */
		.kpi-grid {{
			display: grid;
			grid-template-columns: repeat(4, 1fr);
			gap: 16px;
			margin-bottom: 24px;
		}}

		.kpi-card {{
			background: #ffffff;
			border: 1px solid var(--border-subtle);
			border-radius: 10px;
			padding: 18px 20px;
		}}

		.kpi-label {{
			font-size: 11.5px;
			font-weight: 600;
			text-transform: uppercase;
			letter-spacing: 0.04em;
			color: var(--text-muted);
			margin-bottom: 6px;
		}}

		.kpi-value {{
			font-size: 26px;
			font-weight: 700;
			letter-spacing: -0.03em;
			color: var(--text-main);
			font-variant-numeric: tabular-nums;
		}}

		.kpi-subtext {{
			font-size: 12px;
			color: #10b981;
			margin-top: 4px;
			display: flex;
			align-items: center;
			gap: 4px;
			font-weight: 500;
		}}

		/* Modern Data Tables */
		.neo-table-wrapper {{
			background: #ffffff;
			border: 1px solid var(--border-subtle);
			border-radius: 10px;
			overflow: hidden;
		}}

		table.neo-table {{
			width: 100%;
			border-collapse: collapse;
			text-align: left;
		}}

		table.neo-table th {{
			background: #f8fafc;
			padding: 12px 18px;
			font-size: 11.5px;
			font-weight: 600;
			text-transform: uppercase;
			letter-spacing: 0.04em;
			color: var(--text-muted);
			border-bottom: 1px solid var(--border-subtle);
		}}

		table.neo-table td {{
			padding: 14px 18px;
			border-bottom: 1px solid var(--border-subtle);
			font-size: 13.5px;
			vertical-align: middle;
		}}

		table.neo-table tr:last-child td {{
			border-bottom: none;
		}}

		table.neo-table tr:hover td {{
			background: #fafafa;
		}}

		.table-title-cell strong {{
			font-weight: 600;
			color: var(--text-main);
		}}

		.table-row-slug {{
			font-size: 12px;
			color: var(--text-muted);
			margin-top: 2px;
		}}

		.table-actions {{
			display: flex;
			gap: 12px;
			margin-top: 6px;
			font-size: 12px;
		}}

		.table-actions a {{
			text-decoration: none;
			color: var(--primary);
			font-weight: 500;
		}}

		.table-actions a.trash-action {{
			color: #ef4444;
		}}

		/* Status Badges */
		.badge {{
			display: inline-flex;
			align-items: center;
			gap: 5px;
			padding: 2px 8px;
			border-radius: 4px;
			font-size: 11.5px;
			font-weight: 600;
		}}

		.badge-published {{
			background: #ecfdf5;
			color: #059669;
			border: 1px solid #a7f3d0;
		}}

		.badge-draft {{
			background: #fffbeb;
			color: #d97706;
			border: 1px solid #fde68a;
		}}

		/* Buttons */
		.btn {{
			display: inline-flex;
			align-items: center;
			gap: 6px;
			padding: 7px 14px;
			font-size: 13px;
			font-weight: 600;
			border-radius: 6px;
			border: 1px solid transparent;
			cursor: pointer;
			text-decoration: none;
			transition: all 0.15s ease;
		}}

		.btn-primary {{
			background: #2563eb;
			color: #ffffff !important;
		}}

		.btn-primary:hover {{
			background: #1d4ed8;
		}}

		.btn-secondary {{
			background: #ffffff;
			border-color: var(--border-subtle);
			color: var(--text-main);
		}}

		.btn-secondary:hover {{
			background: #f8fafc;
			border-color: var(--border-hover);
		}}

		.btn-danger {{
			background: #fef2f2;
			border-color: #fecaca;
			color: #ef4444;
		}}

		.btn-danger:hover {{
			background: #fee2e2;
		}}

		/* Form Inputs */
		.form-group {{
			margin-bottom: 20px;
		}}

		.form-label {{
			display: block;
			font-size: 13px;
			font-weight: 600;
			color: var(--text-main);
			margin-bottom: 6px;
		}}

		.form-hint {{
			font-size: 12px;
			color: var(--text-muted);
			margin-top: 4px;
		}}

		.form-input, .form-select, .form-textarea {{
			width: 100%;
			padding: 8px 12px;
			font-size: 13.5px;
			font-family: var(--font-sans);
			border: 1px solid var(--border-subtle);
			border-radius: 6px;
			background: #ffffff;
			color: var(--text-main);
			transition: border-color 0.15s, box-shadow 0.15s;
		}}

		.form-input:focus, .form-select:focus, .form-textarea:focus {{
			outline: none;
			border-color: var(--primary);
			box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.12);
		}}

		/* Toast Notification */
		#neo-toast {{
			position: fixed;
			bottom: 24px;
			right: 24px;
			padding: 12px 18px;
			background: #0f172a;
			color: #ffffff;
			border-radius: 8px;
			font-size: 13px;
			font-weight: 500;
			box-shadow: 0 10px 15px -3px rgba(0,0,0,0.1), 0 4px 6px -2px rgba(0,0,0,0.05);
			display: none;
			align-items: center;
			gap: 8px;
			z-index: 9999;
		}}

		/* Visual Live Canvas & Source Editor */
		.neo-visual-canvas {{
			min-height: 480px;
			max-height: 800px;
			overflow-y: auto;
			padding: 24px;
			background: #ffffff;
			border: 1px solid var(--border-subtle);
			border-radius: 8px;
			font-family: var(--font-sans);
			font-size: 15.5px;
			line-height: 1.8;
			color: #1e293b;
			outline: none;
			box-shadow: inset 0 1px 2px rgba(0,0,0,0.02);
			transition: border-color 0.15s, box-shadow 0.15s;
		}}

		.neo-visual-canvas:focus {{
			border-color: var(--primary);
			box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.12);
		}}

		.neo-visual-canvas[placeholder]:empty:before {{
			content: attr(placeholder);
			color: #94a3b8;
			pointer-events: none;
		}}

		.neo-visual-canvas h1 {{ font-size: 26px; font-weight: 700; margin: 1.2em 0 0.5em 0; color: #0f172a; line-height: 1.3; }}
		.neo-visual-canvas h2 {{ font-size: 21px; font-weight: 700; margin: 1.2em 0 0.5em 0; color: #0f172a; line-height: 1.35; border-bottom: 1px solid #f1f5f9; padding-bottom: 6px; }}
		.neo-visual-canvas h3 {{ font-size: 18px; font-weight: 600; margin: 1em 0 0.4em 0; color: #1e293b; }}
		.neo-visual-canvas h4 {{ font-size: 16px; font-weight: 600; margin: 0.9em 0 0.3em 0; color: #334155; }}
		.neo-visual-canvas p {{ margin: 0 0 1em 0; }}
		.neo-visual-canvas blockquote {{ border-left: 4px solid #2563eb; background: #f8fafc; margin: 16px 0; padding: 12px 18px; border-radius: 0 8px 8px 0; font-style: italic; color: #475569; }}
		.neo-visual-canvas img {{ max-width: 100%; height: auto; border-radius: 8px; margin: 14px 0; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.08); display: block; }}
		.neo-visual-canvas code {{ background: #f1f5f9; padding: 2px 6px; border-radius: 4px; font-family: var(--font-mono); font-size: 0.9em; color: #db2777; }}
		.neo-visual-canvas pre {{ background: #0f172a; color: #f8fafc; padding: 16px; border-radius: 8px; overflow-x: auto; font-family: var(--font-mono); margin: 16px 0; }}
		.neo-visual-canvas ul, .neo-visual-canvas ol {{ padding-left: 24px; margin: 0 0 1em 0; }}
		.neo-visual-canvas li {{ margin-bottom: 4px; }}
		.neo-visual-canvas table {{ width: 100%; border-collapse: collapse; margin: 16px 0; font-size: 14px; }}
		.neo-visual-canvas th, .neo-visual-canvas td {{ border: 1px solid #cbd5e1; padding: 8px 12px; text-align: left; }}
		.neo-visual-canvas th {{ background: #f1f5f9; font-weight: 600; }}
		.neo-visual-canvas a {{ color: #2563eb; text-decoration: underline; }}

		.neo-html-textarea {{
			width: 100%;
			min-height: 480px;
			max-height: 800px;
			font-family: var(--font-mono);
			font-size: 13.5px;
			line-height: 1.6;
			background: #0f172a;
			color: #f8fafc;
			padding: 20px;
			border-radius: 8px;
			border: 1px solid #334155;
			resize: vertical;
			outline: none;
			tab-size: 4;
			box-sizing: border-box;
		}}
		.neo-html-textarea:focus {{
			border-color: #38bdf8;
			box-shadow: 0 0 0 3px rgba(56, 189, 248, 0.2);
		}}
	</style>
	<script>
		// Standard WordPress REST API settings & session initialization
		window.wpApiSettings = window.wpApiSettings || {{
			root: '/wp-json/',
			nonce: 'wp_rest_admin',
			versionString: 'wp/v2/'
		}};
	</script>
</head>
<body>
<div id="neo-app">
	<!-- Left 240px Navigation Sidebar -->
	<aside id="neo-sidebar">
		<div class="sidebar-header">
			<a href="/n-admin/" class="brand-link">
				<img src="/static/neopress-icon.svg" class="brand-icon" alt="Neopress Icon" />
				<span class="brand-name">Neopress</span>
				<span class="brand-badge">RUST</span>
			</a>
		</div>

		<a href="/n-admin/neopress-engine.php" class="sidebar-engine-pill">
			<span class="pulse-dot"></span>
			<div class="engine-meta">
				<span class="engine-title">Embedded Rusqlite</span>
				<span class="engine-sub">WAL Mode &bull; &lt;0.15ms</span>
			</div>
		</a>

		<nav class="sidebar-nav">
			<!-- 1. DASHBOARD -->
			<div class="nav-group {is_dash_open}">
				<a href="/n-admin/index.php" class="nav-item {is_dash}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="7" height="7"></rect><rect x="14" y="3" width="7" height="7"></rect><rect x="14" y="14" width="7" height="7"></rect><rect x="3" y="14" width="7" height="7"></rect></svg>
					<span>Dashboard</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/index.php" class="nav-subitem {sub_dash_home}">Home</a>
					<a href="/n-admin/update-core.php" class="nav-subitem {sub_dash_updates}">Updates</a>
				</div>
			</div>

			<!-- 2. POSTS -->
			<div class="nav-group {is_posts_open}">
				<a href="/n-admin/edit.php" class="nav-item {is_posts}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>
					<span>Posts</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/edit.php" class="nav-subitem {sub_posts_all}">All Posts</a>
					<a href="/n-admin/post-new.php" class="nav-subitem {sub_posts_new}">Add New</a>
					<a href="/n-admin/edit-tags.php?taxonomy=category" class="nav-subitem {sub_posts_cats}">Categories</a>
					<a href="/n-admin/edit-tags.php?taxonomy=post_tag" class="nav-subitem {sub_posts_tags}">Tags</a>
				</div>
			</div>

			<!-- 3. MEDIA -->
			<div class="nav-group {is_media_open}">
				<a href="/n-admin/upload.php" class="nav-item {is_media}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>
					<span>Media</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/upload.php" class="nav-subitem {sub_media_lib}">Library</a>
					<a href="/n-admin/media-new.php" class="nav-subitem {sub_media_new}">Add New</a>
				</div>
			</div>

			<!-- 4. PAGES -->
			<div class="nav-group {is_pages_open}">
				<a href="/n-admin/edit.php?post_type=page" class="nav-item {is_pages}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="4" y="4" width="16" height="16" rx="2" ry="2"></rect><rect x="9" y="9" width="6" height="6"></rect><line x1="9" y1="1" x2="9" y2="4"></line><line x1="15" y1="1" x2="15" y2="4"></line><line x1="9" y1="20" x2="9" y2="23"></line><line x1="15" y1="20" x2="15" y2="23"></line></svg>
					<span>Pages</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/edit.php?post_type=page" class="nav-subitem {sub_pages_all}">All Pages</a>
					<a href="/n-admin/post-new.php?post_type=page" class="nav-subitem {sub_pages_new}">Add New</a>
				</div>
			</div>

			<!-- 5. COMMENTS -->
			<div class="nav-group">
				<a href="/n-admin/edit-comments.php" class="nav-item {is_comments}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"></path></svg>
					<span>Comments</span>
				</a>
			</div>

			<!-- 6. APPEARANCE -->
			<div class="nav-group {is_app_open}">
				<a href="/n-admin/themes.php" class="nav-item {is_appearance}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z"></path></svg>
					<span>Appearance</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/themes.php" class="nav-subitem {sub_app_themes}">Themes</a>
					<a href="/n-admin/customize.php" class="nav-subitem {sub_app_cust}">Customize</a>
					<a href="/n-admin/widgets.php" class="nav-subitem {sub_app_widg}">Widgets</a>
					<a href="/n-admin/nav-menus.php" class="nav-subitem {sub_app_menus}">Menus</a>
					<a href="/n-admin/theme-editor.php" class="nav-subitem {sub_app_editor}">Theme File Editor</a>
				</div>
			</div>

			<!-- 7. PLUGINS -->
			<div class="nav-group {is_plugins_open}">
				<a href="/n-admin/plugins.php" class="nav-item {is_plugins}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="8" rx="2" ry="2"></rect><rect x="2" y="14" width="20" height="8" rx="2" ry="2"></rect><line x1="6" y1="6" x2="6.01" y2="6"></line><line x1="6" y1="18" x2="6.01" y2="18"></line></svg>
					<span>Plugins</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/plugins.php" class="nav-subitem {sub_plug_list}">Installed Plugins</a>
					<a href="/n-admin/plugin-install.php" class="nav-subitem {sub_plug_new}">Add New &amp; SDK</a>
					<a href="/n-admin/plugin-editor.php" class="nav-subitem {sub_plug_editor}">Plugin File Editor</a>
				</div>
			</div>

			<!-- 7b. SEO & REDIRECTS -->
			<div class="nav-group">
				<a href="/n-admin/seo.php" class="nav-item {is_seo}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line><line x1="11" y1="8" x2="11" y2="14"></line><line x1="8" y1="11" x2="14" y2="11"></line></svg>
					<span>SEO &amp; Redirects</span>
				</a>
			</div>

			<!-- 7c. FORMS STUDIO -->
			<div class="nav-group">
				<a href="/n-admin/forms.php" class="nav-item {is_forms}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 11l3 3L22 4"></path><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"></path></svg>
					<span>Forms Studio</span>
				</a>
			</div>

			<!-- 7d. SNAPSHOT BACKUPS -->
			<div class="nav-group">
				<a href="/n-admin/backups.php" class="nav-item {is_backups}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><ellipse cx="12" cy="5" rx="9" ry="3"></ellipse><path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3"></path><path d="M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5"></path></svg>
					<span>Backups</span>
				</a>
			</div>

			<!-- 8. USERS -->
			<div class="nav-group {is_users_open}">
				<a href="/n-admin/users.php" class="nav-item {is_users}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M17 21v-2a4 4 0 0 0-4-4H5a4 4 0 0 0-4 4v2"></path><circle cx="9" cy="7" r="4"></circle><path d="M23 21v-2a4 4 0 0 0-3-3.87"></path><path d="M16 3.13a4 4 0 0 1 0 7.75"></path></svg>
					<span>Users</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/users.php" class="nav-subitem {sub_users_all}">All Users</a>
					<a href="/n-admin/user-new.php" class="nav-subitem {sub_users_new}">Add New</a>
					<a href="/n-admin/profile.php" class="nav-subitem {sub_users_prof}">Profile</a>
				</div>
			</div>

			<!-- 9. TOOLS -->
			<div class="nav-group {is_tools_open}">
				<a href="/n-admin/tools.php" class="nav-item {is_tools}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"></path></svg>
					<span>Tools</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/tools.php" class="nav-subitem {sub_tools_avail}">Available Tools</a>
					<a href="/n-admin/import.php" class="nav-subitem {sub_tools_import}">Import</a>
					<a href="/n-admin/export.php" class="nav-subitem {sub_tools_export}">Export</a>
					<a href="/n-admin/site-health.php" class="nav-subitem {sub_tools_health}">Site Health</a>
					<a href="/n-admin/export-personal-data.php" class="nav-subitem {sub_tools_exp_data}">Export Personal Data</a>
					<a href="/n-admin/erase-personal-data.php" class="nav-subitem {sub_tools_era_data}">Erase Personal Data</a>
				</div>
			</div>

			<!-- 10. SETTINGS -->
			<div class="nav-group {is_settings_open}">
				<a href="/n-admin/options-general.php" class="nav-item {is_settings}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
					<span>Settings</span>
				</a>
				<div class="nav-submenu">
					<a href="/n-admin/options-general.php" class="nav-subitem {sub_set_gen}">General</a>
					<a href="/n-admin/options-writing.php" class="nav-subitem {sub_set_writ}">Writing</a>
					<a href="/n-admin/options-reading.php" class="nav-subitem {sub_set_read}">Reading</a>
					<a href="/n-admin/options-discussion.php" class="nav-subitem {sub_set_disc}">Discussion</a>
					<a href="/n-admin/options-media.php" class="nav-subitem {sub_set_media}">Media</a>
					<a href="/n-admin/options-permalink.php" class="nav-subitem {sub_set_perm}">Permalinks</a>
					<a href="/n-admin/options-privacy.php" class="nav-subitem {sub_set_priv}">Privacy</a>
				</div>
			</div>

			<!-- SYSTEM TELEMETRY -->
			<div class="nav-group">
				<a href="/n-admin/neopress-engine.php" class="nav-item {is_engine}">
					<svg class="nav-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"></polygon></svg>
					<span>Engine Telemetry</span>
				</a>
			</div>
		</nav>

		<div class="sidebar-footer">
			<a href="/" target="_blank" class="view-site-btn">
				<span>View Public Site</span>
				<span style="font-size: 13px;">&nearr;</span>
			</a>
			<a href="/n-admin/profile.php" class="user-profile" style="text-decoration: none;">
				<div class="user-avatar">A</div>
				<div class="user-details">
					<span class="user-name">admin</span>
					<span class="user-role">Administrator</span>
				</div>
			</a>
			<div style="margin-top: 10px; padding-top: 10px; border-top: 1px solid rgba(255, 255, 255, 0.08); display: flex; justify-content: space-between; align-items: center;">
				<a href="/n-admin/profile.php" style="color: #94a3b8; font-size: 12px; text-decoration: none;">Edit Profile</a>
				<a href="/wp-login.php?action=logout" style="color: #f87171; font-size: 12px; text-decoration: none; font-weight: 600; display: inline-flex; align-items: center; gap: 4px;">
					<span>Log Out</span>
					<span>&rarr;</span>
				</a>
			</div>
		</div>
	</aside>

	<!-- Main Workspace -->
	<main id="neo-main">
		<header id="neo-topbar">
			<div class="breadcrumbs">
				<a href="/n-admin/" class="crumb-root">Neopress</a>
				<span class="crumb-sep">/</span>
				<span class="crumb-active">{page_title}</span>
			</div>

			<div class="topbar-actions">
				<div class="latency-chip">
					<span class="latency-dot"></span>
					<span>⚡ &lt;0.15ms in-process</span>
				</div>
				<a href="/n-admin/post-new.php" class="btn-new-post">
					<span>+</span>
					<span>New Post</span>
				</a>
				<a href="/wp-login.php?action=logout" title="Log Out" style="background: #f1f5f9; color: #475569; text-decoration: none; font-size: 12px; font-weight: 500; padding: 6px 12px; border-radius: 6px; border: 1px solid #cbd5e1; display: inline-flex; align-items: center; gap: 4px; transition: all 0.15s;">
					<span>Log Out</span>
				</a>
			</div>
		</header>

		<div id="neo-canvas">
			{body_content}
		</div>
	</main>
</div>

<!-- Global Toast -->
<div id="neo-toast"></div>

<script>
	function showToast(message, isError = false) {{
		const toast = document.getElementById('neo-toast');
		toast.textContent = message;
		toast.style.background = isError ? '#b91c1c' : '#0f172a';
		toast.style.display = 'flex';
		setTimeout(() => {{
			toast.style.display = 'none';
		}}, 3500);
	}}
</script>
</body>
</html>
"##,
        page_title = page_title,
        site_name = site_name,
        is_dash = is_dash,
        is_dash_open = is_dash_open,
        sub_dash_home = sub_dash_home,
        sub_dash_updates = sub_dash_updates,
        is_posts = is_posts,
        is_posts_open = is_posts_open,
        sub_posts_all = sub_posts_all,
        sub_posts_new = sub_posts_new,
        sub_posts_cats = sub_posts_cats,
        sub_posts_tags = sub_posts_tags,
        is_media = is_media,
        is_media_open = is_media_open,
        sub_media_lib = sub_media_lib,
        sub_media_new = sub_media_new,
        is_pages = is_pages,
        is_pages_open = is_pages_open,
        sub_pages_all = sub_pages_all,
        sub_pages_new = sub_pages_new,
        is_comments = is_comments,
        is_appearance = is_appearance,
        is_app_open = is_app_open,
        sub_app_themes = sub_app_themes,
        sub_app_cust = sub_app_cust,
        sub_app_widg = sub_app_widg,
        sub_app_menus = sub_app_menus,
        sub_app_editor = sub_app_editor,
        is_plugins = is_plugins,
        is_plugins_open = is_plugins_open,
        sub_plug_list = sub_plug_list,
        sub_plug_new = sub_plug_new,
        sub_plug_editor = sub_plug_editor,
        is_seo = is_seo,
        is_forms = is_forms,
        is_backups = is_backups,
        is_users = is_users,
        is_users_open = is_users_open,
        sub_users_all = sub_users_all,
        sub_users_new = sub_users_new,
        sub_users_prof = sub_users_prof,
        is_tools = is_tools,
        is_tools_open = is_tools_open,
        sub_tools_avail = sub_tools_avail,
        sub_tools_import = sub_tools_import,
        sub_tools_export = sub_tools_export,
        sub_tools_health = sub_tools_health,
        sub_tools_exp_data = sub_tools_exp_data,
        sub_tools_era_data = sub_tools_era_data,
        is_settings = is_settings,
        is_settings_open = is_settings_open,
        sub_set_gen = sub_set_gen,
        sub_set_writ = sub_set_writ,
        sub_set_read = sub_set_read,
        sub_set_disc = sub_set_disc,
        sub_set_media = sub_set_media,
        sub_set_perm = sub_set_perm,
        sub_set_priv = sub_set_priv,
        is_engine = is_engine,
        body_content = body_content
    )
}
