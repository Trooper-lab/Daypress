//! Admin CSS styles: page header, rule-led cards, the metric strip and editorial tables.

pub fn get_styles_components() -> &'static str {
    r#"
		.breadcrumbs {
			display: flex;
			align-items: center;
			gap: 8px;
			font: 500 12px/16px var(--font-mono);
		}

		.crumb-root {
			color: var(--ink-muted);
			text-decoration: none;
		}

		.crumb-sep {
			color: var(--ink-muted);
		}

		.crumb-active {
			color: var(--ink);
		}

		.topbar-actions {
			display: flex;
			align-items: center;
			gap: 12px;
		}

		.latency-chip {
			font: 500 12px/16px var(--font-mono);
			color: var(--ink-muted);
			display: flex;
			align-items: center;
			gap: 6px;
		}

		.latency-dot {
			width: 6px;
			height: 6px;
			border-radius: var(--radius-pill);
			background: var(--moss);
		}

		.btn-new-post {
			background: var(--oxide);
			color: var(--on-oxide) !important;
			text-decoration: none;
			font: 600 14px/20px var(--font-sans);
			padding: 8px 16px;
			border-radius: var(--radius-md);
			display: inline-flex;
			align-items: center;
			gap: 6px;
			transition: background 0.12s;
		}

		.btn-new-post:hover {
			background: var(--ink);
			color: var(--paper) !important;
		}

		#day-canvas {
			padding: 0 36px 64px;
			flex: 1;
		}

		#day-canvas > :first-child:not(.page-header) {
			margin-top: 32px;
		}

		/* ==================== PAGE HEADER ==================== */
		/* Set like the desk's masthead: serif title, mono dek, actions right, a strong rule below. */
		.page-header {
			margin: 0 -36px 32px;
			padding: 14px 36px;
			min-height: 64px;
			border-bottom: 1px solid var(--rule-strong);
			display: flex;
			justify-content: space-between;
			align-items: center;
			gap: 24px;
			flex-wrap: wrap;
		}

		.page-header > div:first-child {
			display: flex;
			align-items: baseline;
			flex-wrap: wrap;
			column-gap: 16px;
			row-gap: 2px;
			min-width: 0;
		}

		.page-title {
			font: 450 28px/1.18 var(--font-serif);
			letter-spacing: -0.01em;
			font-variation-settings: 'opsz' 24;
			color: var(--ink);
			margin: 0;
		}

		.page-subtitle {
			font: 400 12.5px/18px var(--font-mono);
			color: var(--ink-muted);
			margin: 0;
		}

		.page-subtitle code {
			font-size: 1em;
			background: none;
			padding: 0;
			color: var(--ink);
		}

		.day-kicker {
			font: 700 12px/16px var(--font-sans);
			letter-spacing: 0.14em;
			text-transform: uppercase;
			color: var(--oxide);
		}

		.day-meta {
			font: 500 12px/16px var(--font-mono);
			letter-spacing: 0.02em;
			color: var(--ink-muted);
		}

		/* ==================== CARDS ==================== */
		/* Grouped by rules, not boxes: a strong rule on top, hairlines inside. */
		.day-card {
			background: transparent;
			border: 0;
			border-top: 1px solid var(--rule-strong);
			border-radius: 0;
			box-shadow: none;
			margin-bottom: 40px;
		}

		.day-card-header {
			padding: 14px 0 12px;
			border-bottom: 1px solid var(--rule);
			display: flex;
			justify-content: space-between;
			align-items: baseline;
			gap: 16px;
		}

		.day-card-title {
			font: 450 21px/1.2 var(--font-serif);
			letter-spacing: -0.01em;
			font-variation-settings: 'opsz' 24;
			color: var(--ink);
			margin: 0;
		}

		.day-card-body {
			padding: 18px 0 4px;
		}

		/* ==================== METRIC STRIP ==================== */
		.kpi-grid {
			display: grid;
			grid-template-columns: repeat(4, 1fr);
			border-top: 1px solid var(--rule-strong);
			border-bottom: 1px solid var(--rule-strong);
			margin-bottom: 40px;
		}

		.kpi-card {
			background: transparent;
			border: 0;
			border-left: 1px solid var(--rule);
			border-radius: 0;
			padding: 26px 24px 22px;
			min-width: 0;
		}

		.kpi-card:first-child {
			border-left: 0;
			padding-left: 0;
		}

		.kpi-label {
			font: 700 12px/16px var(--font-sans);
			letter-spacing: 0.14em;
			text-transform: uppercase;
			color: var(--oxide);
			margin-bottom: 10px;
		}

		.kpi-value {
			font: 300 56px/1 var(--font-serif);
			letter-spacing: -0.03em;
			font-variation-settings: 'opsz' 72;
			color: var(--ink);
			font-variant-numeric: lining-nums tabular-nums;
		}

		.kpi-value span {
			font: 400 22px/1 var(--font-serif) !important;
			letter-spacing: 0;
			margin-left: 2px;
		}

		.kpi-subtext {
			font: 500 12px/16px var(--font-mono);
			color: var(--ink-muted);
			margin-top: 14px;
		}

		.kpi-subtext a {
			color: inherit;
			text-decoration-color: var(--oxide);
			text-underline-offset: 3px;
		}

		/* ==================== TABLES ==================== */
		.day-table-wrapper {
			background: transparent;
			border: 0;
			border-radius: 0;
			overflow-x: auto;
		}

		table.day-table {
			width: 100%;
			border-collapse: collapse;
			text-align: left;
		}

		table.day-table th {
			background: transparent;
			padding: 10px 14px;
			font: 600 11px/16px var(--font-sans);
			text-transform: uppercase;
			letter-spacing: 0.12em;
			color: var(--ink-muted);
			border-bottom: 1px solid var(--rule-strong);
		}

		table.day-table td {
			padding: 14px;
			border-bottom: 1px solid var(--rule);
			font-size: 14.5px;
			vertical-align: middle;
		}

		table.day-table th:first-child,
		table.day-table td:first-child {
			padding-left: 0;
		}

		table.day-table tr:hover td {
			background: var(--paper-raised);
		}

		.table-title-cell strong {
			font: 450 17.5px/1.3 var(--font-serif);
			font-variation-settings: 'opsz' 18;
			color: var(--ink);
		}

		.table-title-cell strong a {
			color: inherit;
			text-decoration: none;
		}

		.table-title-cell strong a:hover {
			text-decoration: underline;
			text-decoration-color: var(--oxide);
			text-decoration-thickness: 2px;
			text-underline-offset: 4px;
		}

		.table-row-slug {
			font: 400 12px/16px var(--font-mono);
			color: var(--ink-muted);
			margin-top: 3px;
		}

		.table-actions {
			display: flex;
			gap: 14px;
			margin-top: 6px;
			font-size: 13px;
		}

		.table-actions a {
			text-decoration: none;
			color: var(--press);
			font-weight: 500;
		}

		.table-actions a:hover {
			text-decoration: underline;
			text-underline-offset: 3px;
		}

		.table-actions a.trash-action {
			color: var(--oxide);
		}

		/* ==================== QUICK SEARCH ==================== */
		#day-qs-backdrop {
			display: none;
			position: fixed;
			inset: 0;
			background: #16171a73;
			z-index: 100000;
			align-items: flex-start;
			justify-content: center;
			padding-top: 12vh;
		}

		.day-qs-dialog {
			width: min(640px, 92vw);
			background: var(--paper-raised);
			border: 1px solid var(--rule-strong);
			border-radius: var(--radius-md);
			overflow: hidden;
		}

		#day-qs-input {
			width: 100%;
			border: 0;
			border-bottom: 1px solid var(--rule-strong);
			padding: 16px 18px;
			font: 400 19px/1.4 var(--font-serif);
			background: transparent;
			color: var(--ink);
			outline: none;
		}

		#day-qs-results {
			max-height: 52vh;
			overflow-y: auto;
			padding: 6px 0;
		}

		.day-qs-item {
			display: flex;
			justify-content: space-between;
			gap: 12px;
			padding: 9px 18px;
			text-decoration: none;
			color: var(--ink);
			font-size: 14.5px;
		}

		.day-qs-item.is-active {
			background: var(--selection);
		}

		.day-qs-kind,
		.day-qs-hint,
		.day-qs-empty {
			font: 500 11.5px/16px var(--font-mono);
			color: var(--ink-muted);
		}

		.day-qs-empty {
			padding: 14px 18px;
		}

		.day-qs-hint {
			padding: 8px 18px;
			border-top: 1px solid var(--rule);
		}
"#
}
