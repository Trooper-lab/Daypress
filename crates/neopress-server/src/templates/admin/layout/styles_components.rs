//! Admin CSS styles: UI Components, Cards, KPI Grids, and Tables.

pub fn get_styles_components() -> &'static str {
    r#"
		.breadcrumbs {
			display: flex;
			align-items: center;
			gap: 8px;
			font-size: 13px;
			font-weight: 500;
		}

		.crumb-root {
			color: var(--text-muted);
			text-decoration: none;
		}

		.crumb-sep {
			color: var(--text-dim);
		}

		.crumb-active {
			color: var(--text-main);
			font-weight: 600;
		}

		.topbar-actions {
			display: flex;
			align-items: center;
			gap: 12px;
		}

		.latency-chip {
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
		}

		.latency-dot {
			width: 6px;
			height: 6px;
			border-radius: 50%;
			background: #10b981;
		}

		.btn-new-post {
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
		}

		.btn-new-post:hover {
			background: #1e293b;
		}

		#neo-canvas {
			padding: 32px 36px 60px 36px;
			flex: 1;
		}

		/* ==================== 2026 UI COMPONENTS ==================== */
		.page-header {
			margin-bottom: 24px;
			display: flex;
			justify-content: space-between;
			align-items: flex-end;
		}

		.page-title {
			font-size: 22px;
			font-weight: 700;
			letter-spacing: -0.02em;
			color: var(--text-main);
			margin: 0;
		}

		.page-subtitle {
			font-size: 13.5px;
			color: var(--text-muted);
			margin-top: 4px;
		}

		/* Modern Cards */
		.neo-card {
			background: #ffffff;
			border: 1px solid var(--border-subtle);
			border-radius: 10px;
			box-shadow: 0 1px 3px rgba(0, 0, 0, 0.02);
			overflow: hidden;
			margin-bottom: 24px;
		}

		.neo-card-header {
			padding: 16px 20px;
			border-bottom: 1px solid var(--border-subtle);
			display: flex;
			justify-content: space-between;
			align-items: center;
		}

		.neo-card-title {
			font-size: 14px;
			font-weight: 600;
			color: var(--text-main);
			margin: 0;
		}

		.neo-card-body {
			padding: 20px;
		}

		/* KPI Metric Grid */
		.kpi-grid {
			display: grid;
			grid-template-columns: repeat(4, 1fr);
			gap: 16px;
			margin-bottom: 24px;
		}

		.kpi-card {
			background: #ffffff;
			border: 1px solid var(--border-subtle);
			border-radius: 10px;
			padding: 18px 20px;
		}

		.kpi-label {
			font-size: 11.5px;
			font-weight: 600;
			text-transform: uppercase;
			letter-spacing: 0.04em;
			color: var(--text-muted);
			margin-bottom: 6px;
		}

		.kpi-value {
			font-size: 26px;
			font-weight: 700;
			letter-spacing: -0.03em;
			color: var(--text-main);
			font-variant-numeric: tabular-nums;
		}

		.kpi-subtext {
			font-size: 12px;
			color: #10b981;
			margin-top: 4px;
			display: flex;
			align-items: center;
			gap: 4px;
			font-weight: 500;
		}

		/* Modern Data Tables */
		.neo-table-wrapper {
			background: #ffffff;
			border: 1px solid var(--border-subtle);
			border-radius: 10px;
			overflow: hidden;
		}

		table.neo-table {
			width: 100%;
			border-collapse: collapse;
			text-align: left;
		}

		table.neo-table th {
			background: #f8fafc;
			padding: 12px 18px;
			font-size: 11.5px;
			font-weight: 600;
			text-transform: uppercase;
			letter-spacing: 0.04em;
			color: var(--text-muted);
			border-bottom: 1px solid var(--border-subtle);
		}

		table.neo-table td {
			padding: 14px 18px;
			border-bottom: 1px solid var(--border-subtle);
			font-size: 13.5px;
			vertical-align: middle;
		}

		table.neo-table tr:last-child td {
			border-bottom: none;
		}

		table.neo-table tr:hover td {
			background: #fafafa;
		}

		.table-title-cell strong {
			font-weight: 600;
			color: var(--text-main);
		}

		.table-row-slug {
			font-size: 12px;
			color: var(--text-muted);
			margin-top: 2px;
		}

		.table-actions {
			display: flex;
			gap: 12px;
			margin-top: 6px;
			font-size: 12px;
		}

		.table-actions a {
			text-decoration: none;
			color: var(--primary);
			font-weight: 500;
		}

		.table-actions a.trash-action {
			color: #ef4444;
		}
"#
}
