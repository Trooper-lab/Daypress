//! Admin CSS styles: badges, buttons, form controls, the toast and the classic editor canvas.

pub fn get_styles_forms() -> &'static str {
    r#"
		/* Status badges: pale semantic tint pills, sentence case, >= 12px. */
		.badge {
			display: inline-flex;
			align-items: center;
			gap: 5px;
			padding: 2px 8px;
			border-radius: var(--radius-md);
			border: 1px solid var(--rule);
			font: 500 12px/16px var(--font-sans);
			letter-spacing: 0;
			background: var(--paper-sunk);
			color: var(--ink);
		}

		.badge-published {
			background: var(--moss-wash);
			border-color: transparent;
			color: var(--moss);
		}

		.badge-draft {
			background: var(--ochre-wash);
			border-color: transparent;
			color: var(--ochre);
		}

		.badge-scheduled,
		.badge-future {
			background: var(--press-wash);
			border-color: transparent;
			color: var(--press);
		}

		/* Buttons: WordPress button metrics (32px height, Admin Blue primary, subtle secondary). */
		.btn {
			display: inline-flex;
			align-items: center;
			gap: 6px;
			padding: 6px 12px;
			min-height: 32px;
			font: 500 13px/18px var(--font-sans);
			border-radius: var(--radius-md);
			border: 1px solid transparent;
			cursor: pointer;
			text-decoration: none;
			transition: background-color 0.12s, color 0.12s, border-color 0.12s;
		}

		.btn:disabled,
		.btn[aria-disabled="true"] {
			opacity: 0.45;
			cursor: not-allowed;
		}

		.btn-primary {
			background: var(--oxide);
			border-color: var(--oxide);
			color: var(--on-oxide) !important;
		}

		.btn-primary:hover {
			background: var(--primary-hover);
			border-color: var(--primary-hover);
			color: var(--on-oxide) !important;
		}

		.btn-secondary {
			background: var(--paper-raised);
			border-color: var(--rule-strong);
			color: var(--oxide);
		}

		.btn-secondary:hover {
			background: var(--oxide-wash);
			border-color: var(--oxide);
			color: var(--primary-hover);
		}

		.btn-danger {
			background: var(--paper-raised);
			border-color: var(--danger);
			color: var(--danger);
		}

		.btn-danger:hover {
			background: var(--danger);
			color: #ffffff;
		}

		/* Form controls: clean white inputs with subtle border and Admin Blue focus ring. */
		.form-group {
			margin-bottom: 18px;
		}

		.form-label {
			display: block;
			font: 600 13.5px/18px var(--font-sans);
			color: var(--ink);
			margin-bottom: 6px;
		}

		.form-hint {
			font-size: 12.5px;
			color: var(--ink-muted);
			margin-top: 5px;
		}

		.form-input, .form-select, .form-textarea,
		#day-canvas :where(input[type="text"], input[type="email"], input[type="url"], input[type="password"], input[type="number"], input[type="search"], input[type="date"], input[type="datetime-local"], input[type="time"], input[type="tel"], select, textarea) {
			width: 100%;
			padding: 6px 10px;
			font: 400 13.5px/1.4 var(--font-sans);
			border: 1px solid var(--rule-strong);
			border-radius: var(--radius-md);
			background: var(--paper-raised);
			color: var(--ink);
			transition: border-color 0.12s, box-shadow 0.12s;
		}

		.form-select,
		#day-canvas select {
			min-height: 34px;
			padding: 5px 28px 5px 10px;
			line-height: 1.4;
			vertical-align: middle;
		}

		.form-input:focus, .form-select:focus, .form-textarea:focus,
		#day-canvas :where(input, select, textarea):focus {
			border-color: var(--focus);
			outline: 1px solid var(--focus);
			outline-offset: 0;
		}

		#day-canvas :where(input[type="checkbox"], input[type="radio"]) {
			accent-color: var(--oxide);
		}

		/* WordPress table.form-table horizontal label-left + control-right form layout */
		table.form-table {
			width: 100%;
			border-collapse: collapse;
			margin-top: 8px;
			margin-bottom: 16px;
			clear: both;
		}

		table.form-table th {
			vertical-align: top;
			text-align: left;
			padding: 16px 12px 16px 0;
			width: 210px;
			line-height: 1.4;
			font: 600 13.5px/1.4 var(--font-sans);
			color: var(--ink);
		}

		table.form-table td {
			margin-bottom: 9px;
			padding: 12px 10px;
			line-height: 1.4;
			vertical-align: middle;
			font: 400 13.5px/1.4 var(--font-sans);
			color: var(--ink);
		}

		table.form-table td p.description,
		table.form-table td .form-hint {
			margin: 5px 0 0;
			font-size: 13px;
			line-height: 1.45;
			color: var(--ink-muted);
		}

		table.form-table input.regular-text,
		table.form-table select.regular-text {
			max-width: 380px;
		}

		table.form-table input.small-text {
			width: 80px;
			max-width: 100px;
		}

		@media (max-width: 782px) {
			table.form-table,
			table.form-table tbody,
			table.form-table tr,
			table.form-table th,
			table.form-table td {
				display: block;
				width: 100%;
			}
			table.form-table th {
				padding: 12px 0 4px;
			}
			table.form-table td {
				padding: 0 0 12px;
			}
			table.form-table input.regular-text,
			table.form-table select.regular-text {
				max-width: 100%;
			}
		}

		/* Toast */
		#day-toast {
			position: fixed;
			bottom: 24px;
			right: 24px;
			padding: 10px 16px;
			background: var(--ink);
			color: #ffffff;
			border-radius: var(--radius-md);
			font: 500 13.5px/18px var(--font-sans);
			display: none;
			align-items: center;
			gap: 8px;
			z-index: 9999;
		}

		#day-toast.is-error {
			background: var(--danger);
			color: #ffffff;
		}

		/* ==================== WORDPRESS POST/PAGE EDITOR (#titlediv, #postdivrich, .wp-switch-editor) ==================== */
		.wp-title-input {
			width: 100% !important;
			height: 40px !important;
			padding: 3px 10px !important;
			font: 400 20px/1.4 var(--font-sans) !important;
			background: var(--paper-raised) !important;
			color: var(--ink) !important;
			border: 1px solid var(--rule-strong) !important;
			border-radius: var(--radius-sm) !important;
			box-shadow: inset 0 1px 2px rgba(0, 0, 0, 0.04);
		}

		#edit-slug-box {
			display: flex;
			align-items: center;
			flex-wrap: wrap;
			gap: 6px;
			min-height: 24px;
			margin: 8px 0 12px;
			padding: 0 2px;
			font: 400 13px/1.5 var(--font-sans);
			color: var(--ink-muted);
		}

		.wp-slug-input {
			width: 190px !important;
			height: 24px !important;
			padding: 0 6px !important;
			font: 400 12.5px/22px var(--font-sans) !important;
			background: var(--paper-raised) !important;
			color: var(--ink) !important;
			border: 1px solid var(--rule-strong) !important;
			border-radius: 2px !important;
		}

		#wp-content-editor-tools {
			display: flex;
			justify-content: space-between;
			align-items: flex-end;
			gap: 8px;
			padding-top: 4px;
		}

		#wp-content-media-buttons {
			padding-bottom: 4px;
			display: flex;
			align-items: center;
			gap: 6px;
		}

		.wp-editor-tabs {
			display: inline-flex;
			align-items: flex-end;
			gap: 3px;
		}

		.wp-switch-editor {
			float: left;
			box-sizing: content-box;
			position: relative;
			top: 1px;
			background: var(--paper);
			color: var(--ink-muted);
			cursor: pointer;
			font: 400 13px/19px var(--font-sans);
			height: 20px;
			margin: 0;
			padding: 4px 12px;
			border: 1px solid var(--rule-strong);
			border-bottom-color: var(--rule-strong);
			border-radius: 2px 2px 0 0;
		}

		.wp-switch-editor:hover {
			color: var(--ink);
		}

		.wp-switch-editor.is-active {
			background: var(--paper-sunk);
			color: var(--ink);
			border-bottom-color: var(--paper-sunk);
			font-weight: 600;
			z-index: 2;
		}

		#wp-content-editor-container {
			border: 1px solid var(--rule-strong);
			background: var(--paper-raised);
		}

		.wp-editor-toolbar {
			background: var(--paper-sunk);
			border-bottom: 1px solid var(--rule-strong);
			padding: 5px 8px;
			display: flex;
			flex-wrap: wrap;
			align-items: center;
			gap: 2px;
		}

		.wp-mce-btn {
			display: inline-flex;
			align-items: center;
			justify-content: center;
			min-width: 26px;
			height: 26px;
			padding: 0 6px;
			border: 1px solid transparent;
			border-radius: 2px;
			background: transparent;
			color: #50575e;
			font: 500 12.5px/1 var(--font-sans);
			cursor: pointer;
		}

		.wp-mce-btn:hover {
			border-color: var(--rule-strong);
			background: var(--paper-raised);
			color: var(--ink);
		}

		.wp-mce-sep {
			width: 1px;
			height: 18px;
			background: var(--rule-strong);
			margin: 0 4px;
		}

		#post-status-info {
			background: var(--paper-sunk);
			border: 1px solid var(--rule-strong);
			border-top: 0;
			padding: 4px 10px;
			font: 400 12px/18px var(--font-sans);
			color: var(--ink-muted);
			display: flex;
			justify-content: space-between;
			align-items: center;
		}

		/* Classic editor canvas: clean sans-serif editing surface inside #wp-content-editor-container */
		.day-visual-canvas {
			min-height: 420px;
			max-height: 800px;
			overflow-y: auto;
			padding: 16px 20px;
			background: var(--paper-raised);
			border: 0;
			border-radius: 0;
			font: 400 15px/1.65 var(--font-sans);
			color: var(--ink);
			outline: none;
		}

		.day-visual-canvas:focus {
			outline: none;
		}

		.day-visual-canvas[placeholder]:empty:before {
			content: attr(placeholder);
			color: var(--ink-muted);
			pointer-events: none;
		}

		.day-visual-canvas h1 { font: 600 28px/1.2 var(--font-sans); letter-spacing: -0.015em; margin: 1.1em 0 0.4em; }
		.day-visual-canvas h2 { font: 600 22px/1.25 var(--font-sans); letter-spacing: -0.01em; margin: 1.1em 0 0.4em; }
		.day-visual-canvas h3 { font: 600 18px/1.3 var(--font-sans); margin: 1em 0 0.35em; }
		.day-visual-canvas h4 { font: 600 15px/1.35 var(--font-sans); margin: 1em 0 0.3em; }
		.day-visual-canvas p { margin: 0 0 1em 0; }
		.day-visual-canvas blockquote { margin: 16px 0; padding: 10px 16px; border-left: 3px solid var(--press); background: var(--paper-sunk); font: 400 15px/1.55 var(--font-sans); }
		.day-visual-canvas img { max-width: 100%; height: auto; margin: 14px 0; display: block; }
		.day-visual-canvas code { background: var(--paper-sunk); padding: 1px 5px; border-radius: var(--radius-sm); font-family: var(--font-mono); font-size: 0.85em; }
		.day-visual-canvas pre { background: var(--paper-sunk); color: var(--ink); padding: 14px; border-radius: var(--radius-sm); overflow-x: auto; font: 400 13px/20px var(--font-mono); margin: 14px 0; }
		.day-visual-canvas ul, .day-visual-canvas ol { padding-left: 24px; margin: 0 0 1em 0; }
		.day-visual-canvas li { margin-bottom: 4px; }
		.day-visual-canvas table { width: 100%; border-collapse: collapse; margin: 14px 0; font: 400 14px/1.5 var(--font-sans); }
		.day-visual-canvas th, .day-visual-canvas td { border-bottom: 1px solid var(--rule); padding: 8px 12px; text-align: left; }
		.day-visual-canvas th { border-bottom-color: var(--rule-strong); font-weight: 600; }
		.day-visual-canvas a { color: var(--press); text-decoration: underline; text-underline-offset: 3px; }

		.day-html-textarea {
			width: 100%;
			min-height: 420px;
			max-height: 800px;
			font: 400 13.5px/20px var(--font-mono);
			background: var(--paper-raised);
			color: var(--ink);
			padding: 14px 16px;
			border-radius: 0;
			border: 0 !important;
			resize: vertical;
			outline: none !important;
			tab-size: 4;
			box-sizing: border-box;
		}
"#
}
