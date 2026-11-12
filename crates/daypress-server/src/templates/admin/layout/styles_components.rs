//! Admin CSS styles: page header, rule-led cards, the metric strip and editorial tables.

pub fn get_styles_components() -> &'static str {
    r#"
		.breadcrumbs {
			display: flex;
			align-items: center;
			gap: 8px;
			font: 500 12.5px/16px var(--font-sans);
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
			gap: 10px;
			flex-wrap: wrap;
		}

		.latency-chip {
			font: 500 12px/16px var(--font-sans);
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
			font: 500 13px/18px var(--font-sans);
			padding: 6px 12px;
			border-radius: var(--radius-md);
			display: inline-flex;
			align-items: center;
			gap: 6px;
			transition: background 0.12s;
		}

		.btn-new-post:hover {
			background: var(--primary-hover);
			color: var(--on-oxide) !important;
		}

		#day-canvas {
			padding: 0 20px 32px;
			flex: 1;
		}

		#day-canvas > :first-child:not(.page-header) {
			margin-top: 14px;
		}

		/* ==================== PAGE HEADER (WordPress .wrap h1.wp-heading-inline + .page-title-action) ==================== */
		.page-header {
			margin: 0 0 14px;
			padding: 10px 0 4px;
			min-height: 40px;
			border-bottom: 0;
			display: flex;
			justify-content: space-between;
			align-items: center;
			gap: 12px;
			flex-wrap: wrap;
		}

		.page-header > div:first-child {
			display: flex;
			align-items: center;
			flex-wrap: wrap;
			column-gap: 10px;
			row-gap: 4px;
			min-width: 0;
		}

		.page-title {
			font: 400 23px/1.3 var(--font-sans);
			letter-spacing: -0.01em;
			color: var(--ink);
			margin: 0;
			display: inline-block;
		}

		.page-title-action {
			display: inline-flex;
			align-items: center;
			gap: 4px;
			padding: 4px 10px;
			font: 500 13px/1.4 var(--font-sans);
			color: var(--oxide) !important;
			background: var(--paper-raised);
			border: 1px solid var(--oxide);
			border-radius: 3px;
			text-decoration: none;
			cursor: pointer;
			transition: background-color 0.1s, color 0.1s, border-color 0.1s;
		}

		.page-title-action:hover {
			background: var(--oxide-wash);
			border-color: var(--primary-hover);
			color: var(--primary-hover) !important;
		}

		.page-subtitle {
			width: 100%;
			font: 400 13px/18px var(--font-sans);
			color: var(--ink-muted);
			margin: 2px 0 0;
		}

		.page-subtitle code {
			font-size: 0.95em;
			background: var(--paper-sunk);
			padding: 1px 4px;
			border-radius: 3px;
			color: var(--ink);
		}

		.day-kicker {
			font: 600 12.5px/16px var(--font-sans);
			letter-spacing: 0;
			text-transform: none;
			color: var(--oxide);
		}

		.day-meta {
			font: 500 12.5px/16px var(--font-sans);
			color: var(--ink-muted);
		}

		/* ==================== WORDPRESS SUBSUBSUB & TABLENAV ==================== */
		.wp-subsubsub-row {
			display: flex;
			justify-content: space-between;
			align-items: center;
			gap: 12px;
			flex-wrap: wrap;
			margin: 4px 0 10px;
		}

		.subsubsub {
			list-style: none;
			margin: 0;
			padding: 0;
			display: flex;
			flex-wrap: wrap;
			align-items: center;
			gap: 4px;
			font: 400 13px/1.6 var(--font-sans);
			color: var(--ink-muted);
		}

		.subsubsub a,
		.subsubsub .status-tab {
			color: var(--press);
			text-decoration: none;
			padding: 1px 2px;
			border: 0 !important;
			background: transparent !important;
			font: 400 13px/1.6 var(--font-sans);
			cursor: pointer;
		}

		.subsubsub a:hover,
		.subsubsub .status-tab:hover {
			color: var(--press-hover, #135e96);
			text-decoration: underline;
		}

		.subsubsub a.current,
		.subsubsub .status-tab.active,
		.subsubsub .status-tab.current {
			color: var(--ink) !important;
			font-weight: 600 !important;
		}

		.subsubsub .count {
			color: var(--ink-muted);
			font-weight: 400;
		}

		.subsubsub-sep {
			color: var(--rule-strong);
			margin: 0 2px;
			user-select: none;
		}

		.tablenav {
			display: flex;
			justify-content: space-between;
			align-items: center;
			gap: 10px;
			flex-wrap: wrap;
			margin: 6px 0 10px;
			min-height: 32px;
		}

		.tablenav-actions {
			display: flex;
			align-items: center;
			gap: 6px;
			flex-wrap: wrap;
		}

		.tablenav-pages {
			font: 400 13px/1.5 var(--font-sans);
			color: var(--ink-muted);
			display: flex;
			align-items: center;
			gap: 8px;
			margin-left: auto;
		}

		.wp-filter {
			background: var(--paper-raised);
			border: 1px solid var(--rule-strong);
			border-radius: var(--radius-sm);
			padding: 10px 14px;
			margin-bottom: 16px;
			display: flex;
			justify-content: space-between;
			align-items: center;
			gap: 12px;
			flex-wrap: wrap;
			box-shadow: 0 1px 1px rgba(0, 0, 0, 0.04);
		}

		/* ==================== CARDS (WordPress .postbox & .postbox-header) ==================== */
		.day-card {
			background: var(--paper-raised);
			border: 1px solid var(--rule-strong);
			border-radius: 0;
			box-shadow: 0 1px 1px rgba(0, 0, 0, 0.04);
			margin-bottom: 20px;
		}

		.day-card-header {
			padding: 8px 12px;
			min-height: 36px;
			border-bottom: 1px solid var(--rule-strong);
			display: flex;
			justify-content: space-between;
			align-items: center;
			gap: 8px;
		}

		.day-card[data-dash-widget] > .day-card-header,
		.day-card[data-metabox-id] > .day-card-header {
			padding: 0 12px;
			cursor: move;
			user-select: none;
		}

		.day-card.dp-dash-collapsed > .day-card-header,
		.day-card.dp-metabox-collapsed > .day-card-header {
			border-bottom: 0;
		}

		.day-card-title {
			font: 600 14px/1.4 var(--font-sans);
			letter-spacing: 0;
			color: var(--ink);
			margin: 0;
		}

		.handle-actions {
			display: inline-flex;
			align-items: center;
			gap: 0;
			margin-left: auto;
			margin-right: -6px;
			flex-shrink: 0;
		}

		.handle-order-higher,
		.handle-order-lower,
		.handlediv {
			width: 28px;
			height: 28px;
			padding: 0;
			margin: 0;
			border: 0;
			background: transparent;
			color: #787c82;
			cursor: pointer;
			display: inline-flex;
			align-items: center;
			justify-content: center;
			border-radius: 2px;
			transition: color 0.1s ease;
		}

		.handle-order-higher:hover,
		.handle-order-lower:hover,
		.handlediv:hover,
		.handle-order-higher:focus-visible,
		.handle-order-lower:focus-visible,
		.handlediv:focus-visible {
			color: var(--ink);
			background: transparent;
		}

		.handle-order-higher svg,
		.handle-order-lower svg,
		.handlediv svg {
			width: 18px;
			height: 18px;
			display: block;
			pointer-events: none;
		}

		.handlediv[aria-expanded="false"] svg {
			transform: rotate(180deg);
		}

		.dp-sortable-placeholder {
			border: 1px dashed var(--rule-strong);
			background: rgba(0, 0, 0, 0.02);
			min-height: 52px;
			margin-bottom: 20px;
		}

		.day-card-body {
			padding: 12px 14px;
		}

		/* ==================== METRIC STRIP (WordPress At a Glance) ==================== */
		.kpi-grid {
			display: grid;
			grid-template-columns: repeat(4, 1fr);
			background: var(--paper-raised);
			border: 1px solid var(--rule);
			border-radius: var(--radius-md);
			margin-bottom: 20px;
		}

		.kpi-card {
			background: transparent;
			border: 0;
			border-left: 1px solid var(--rule);
			border-radius: 0;
			padding: 16px 20px;
			min-width: 0;
		}

		.kpi-card:first-child {
			border-left: 0;
		}

		.kpi-label {
			font: 500 13px/18px var(--font-sans);
			letter-spacing: 0;
			text-transform: none;
			color: var(--ink-muted);
			margin-bottom: 6px;
		}

		.kpi-value {
			font: 600 26px/1.2 var(--font-sans);
			letter-spacing: -0.02em;
			color: var(--ink);
			font-variant-numeric: tabular-nums;
		}

		.kpi-value span {
			font: 500 14px/1 var(--font-sans) !important;
			letter-spacing: 0;
			margin-left: 2px;
			color: var(--ink-muted);
		}

		.kpi-subtext {
			font: 400 12.5px/16px var(--font-sans);
			color: var(--ink-muted);
			margin-top: 6px;
		}

		.kpi-subtext a {
			color: var(--press);
			text-decoration: none;
		}

		.kpi-subtext a:hover {
			text-decoration: underline;
		}

		/* ==================== TABLES (WordPress .wp-list-table.widefat.striped) ==================== */
		.day-table-wrapper {
			background: var(--paper-raised);
			border: 1px solid var(--rule-strong);
			border-radius: 0;
			box-shadow: 0 1px 1px rgba(0, 0, 0, 0.04);
			overflow-x: auto;
		}

		table.day-table {
			width: 100%;
			border-collapse: collapse;
			text-align: left;
		}

		table.day-table thead th,
		table.day-table tfoot th {
			background: var(--paper-raised);
			padding: 9px 10px;
			font: 600 13px/18px var(--font-sans);
			text-transform: none;
			letter-spacing: 0;
			color: var(--ink);
		}

		table.day-table thead th {
			border-bottom: 1px solid var(--rule-strong);
		}

		table.day-table tfoot th {
			border-top: 1px solid var(--rule-strong);
			border-bottom: 0;
		}

		table.day-table td {
			padding: 10px 10px;
			border-bottom: 1px solid var(--rule);
			font-size: 13px;
			line-height: 1.5;
			vertical-align: top;
		}

		table.day-table tbody tr:last-child td {
			border-bottom: 0;
		}

		table.day-table th:first-child,
		table.day-table td:first-child {
			padding-left: 12px;
		}

		table.day-table tbody tr:nth-child(odd) td {
			background: #f6f7f7;
		}

		table.day-table tbody tr:hover td {
			background: var(--oxide-wash);
		}

		.table-title-cell strong {
			font: 600 14px/1.35 var(--font-sans);
			color: var(--press);
		}

		.table-title-cell strong a {
			color: var(--press) !important;
			text-decoration: none;
		}

		.table-title-cell strong a:hover {
			color: var(--press-hover, #135e96) !important;
			text-decoration: underline;
		}

		.table-row-slug {
			font: 400 12px/16px var(--font-sans);
			color: var(--ink-muted);
			margin-top: 2px;
		}

		.table-actions {
			display: flex;
			flex-wrap: wrap;
			align-items: center;
			gap: 6px;
			margin-top: 4px;
			font-size: 12.5px;
			line-height: 1.4;
			color: var(--rule-strong);
		}

		.table-actions a,
		.table-actions button.row-action-btn {
			text-decoration: none;
			color: var(--press);
			font-weight: 400;
			background: none;
			border: 0;
			padding: 0;
			font: inherit;
			cursor: pointer;
		}

		.table-actions a:hover,
		.table-actions button.row-action-btn:hover {
			color: var(--press-hover, #135e96);
			text-decoration: underline;
		}

		.table-actions a.trash-action,
		.table-actions button.trash-action {
			color: var(--danger);
		}

		/* ==================== QUICK SEARCH ==================== */
		#day-qs-backdrop {
			display: none;
			position: fixed;
			inset: 0;
			background: rgba(29, 35, 39, 0.55);
			z-index: 100000;
			align-items: flex-start;
			justify-content: center;
			padding-top: 12vh;
		}

		.day-qs-dialog {
			width: min(600px, 92vw);
			background: var(--paper-raised);
			border: 1px solid var(--rule);
			border-radius: var(--radius-md);
			overflow: hidden;
		}

		#day-qs-input {
			width: 100%;
			border: 0;
			border-bottom: 1px solid var(--rule);
			padding: 14px 16px;
			font: 400 15px/1.4 var(--font-sans);
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
			padding: 8px 16px;
			text-decoration: none;
			color: var(--ink);
			font-size: 13.5px;
		}

		.day-qs-item.is-active {
			background: var(--selection);
		}

		.day-qs-kind,
		.day-qs-hint,
		.day-qs-empty {
			font: 500 12px/16px var(--font-sans);
			color: var(--ink-muted);
		}

		.day-qs-empty {
			padding: 14px 16px;
		}

		.day-qs-hint {
			padding: 8px 16px;
			border-top: 1px solid var(--rule);
		}

		/* ==================== RESPONSIVE GRIDS & TABLES ==================== */
		@media (max-width: 960px) {
			#day-canvas {
				padding: 0 16px 40px;
			}
			.kpi-grid {
				grid-template-columns: repeat(2, 1fr);
			}
			.kpi-card:nth-child(3) {
				border-left: 0;
			}
			.kpi-card:nth-child(n+3) {
				border-top: 1px solid var(--rule);
			}
			#dashColumnsGrid {
				grid-template-columns: 1fr !important;
			}
		}

		@media (max-width: 600px) {
			.kpi-grid {
				grid-template-columns: 1fr;
			}
			.kpi-card {
				border-left: 0 !important;
				border-top: 1px solid var(--rule);
			}
			.kpi-card:first-child {
				border-top: 0;
			}
			.page-header {
				padding: 14px 0 6px;
				margin-bottom: 14px;
			}
			.page-title {
				font-size: 20px;
			}
			table.day-table .col-author,
			table.day-table .col-category,
			table.day-table .col-date {
				display: none;
			}
		}
"#
}
