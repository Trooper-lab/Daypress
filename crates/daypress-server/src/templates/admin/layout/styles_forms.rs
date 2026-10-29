//! Admin CSS styles: badges, buttons, form controls, the toast and the classic editor canvas.

pub fn get_styles_forms() -> &'static str {
    r#"
		/* Status badges always carry their word. */
		.badge {
			display: inline-flex;
			align-items: center;
			gap: 5px;
			padding: 2px 8px;
			border-radius: var(--radius-sm);
			border: 1px solid transparent;
			font: 600 11.5px/16px var(--font-sans);
			letter-spacing: 0.02em;
			background: transparent;
			color: var(--ink);
			border-color: var(--rule-strong);
		}

		.badge-published {
			background: var(--moss);
			border-color: var(--moss);
			color: var(--paper);
		}

		.badge-draft {
			background: var(--ochre);
			border-color: var(--ochre);
			color: var(--on-ochre);
		}

		.badge-scheduled,
		.badge-future {
			border-color: var(--press);
			color: var(--press);
		}

		/* Buttons: one oxide primary per view, ink outline beside it. Hover inverts to ink. */
		.btn {
			display: inline-flex;
			align-items: center;
			gap: 6px;
			padding: 8px 16px;
			font: 600 14px/20px var(--font-sans);
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
			background: var(--ink);
			border-color: var(--ink);
			color: var(--paper) !important;
		}

		.btn-secondary {
			background: transparent;
			border-color: var(--rule-strong);
			color: var(--ink);
		}

		.btn-secondary:hover {
			background: var(--ink);
			border-color: var(--ink);
			color: var(--paper);
		}

		.btn-danger {
			background: transparent;
			border-color: var(--oxide);
			color: var(--oxide);
		}

		.btn-danger:hover {
			background: var(--oxide);
			color: var(--on-oxide);
		}

		/* Form controls: sunk wells with a strong border, square-ish corners. */
		.form-group {
			margin-bottom: 20px;
		}

		.form-label {
			display: block;
			font: 600 14px/20px var(--font-sans);
			color: var(--ink);
			margin-bottom: 6px;
		}

		.form-hint {
			font-size: 13px;
			color: var(--ink-muted);
			margin-top: 5px;
		}

		.form-input, .form-select, .form-textarea,
		#day-canvas :where(input[type="text"], input[type="email"], input[type="url"], input[type="password"], input[type="number"], input[type="search"], input[type="date"], input[type="datetime-local"], input[type="time"], input[type="tel"], select, textarea) {
			width: 100%;
			padding: 8px 12px;
			font: 400 14.5px/1.45 var(--font-sans);
			border: 1px solid var(--rule-strong);
			border-radius: var(--radius-sm);
			background: var(--paper-raised);
			color: var(--ink);
			transition: border-color 0.12s, box-shadow 0.12s;
		}

		.form-input:focus, .form-select:focus, .form-textarea:focus,
		#day-canvas :where(input, select, textarea):focus {
			outline: 2px solid var(--focus);
			outline-offset: 2px;
		}

		#day-canvas :where(input[type="checkbox"], input[type="radio"]) {
			accent-color: var(--ink);
		}

		/* Toast */
		#day-toast {
			position: fixed;
			bottom: 24px;
			right: 24px;
			padding: 12px 18px;
			background: var(--ink);
			color: var(--paper);
			border-radius: var(--radius-md);
			font: 500 14px/20px var(--font-sans);
			display: none;
			align-items: center;
			gap: 8px;
			z-index: 9999;
		}

		#day-toast.is-error {
			background: var(--oxide);
			color: var(--on-oxide);
		}

		/* Classic editor canvas: a raised sheet set like the published page. */
		.day-visual-canvas {
			min-height: 480px;
			max-height: 800px;
			overflow-y: auto;
			padding: 32px 40px;
			background: var(--paper-raised);
			border: 1px solid var(--rule);
			border-radius: 0;
			font: 400 19px/1.62 var(--font-serif);
			font-variation-settings: 'opsz' 14;
			color: var(--ink);
			outline: none;
			transition: border-color 0.12s;
		}

		.day-visual-canvas:focus {
			border-color: var(--press);
		}

		.day-visual-canvas[placeholder]:empty:before {
			content: attr(placeholder);
			color: var(--ink-muted);
			font-style: italic;
			pointer-events: none;
		}

		.day-visual-canvas h1 { font: 400 40px/1.1 var(--font-serif); letter-spacing: -0.015em; margin: 1.1em 0 0.4em; }
		.day-visual-canvas h2 { font: 400 30px/1.15 var(--font-serif); letter-spacing: -0.01em; margin: 1.1em 0 0.4em; }
		.day-visual-canvas h3 { font: 500 24px/1.2 var(--font-serif); margin: 1em 0 0.35em; }
		.day-visual-canvas h4 { font: 600 17px/1.3 var(--font-sans); margin: 1em 0 0.3em; }
		.day-visual-canvas p { margin: 0 0 1em 0; }
		.day-visual-canvas blockquote { margin: 24px 0; padding: 16px 0; border-top: 3px solid var(--rule-strong); border-bottom: 1px solid var(--rule-strong); font: italic 300 26px/1.25 var(--font-serif); }
		.day-visual-canvas img { max-width: 100%; height: auto; margin: 16px 0; display: block; }
		.day-visual-canvas code { background: var(--paper-sunk); padding: 1px 5px; border-radius: var(--radius-sm); font-family: var(--font-mono); font-size: 0.78em; }
		.day-visual-canvas pre { background: var(--paper-sunk); color: var(--ink); padding: 16px; border-radius: var(--radius-sm); overflow-x: auto; font: 400 14px/22px var(--font-mono); margin: 16px 0; }
		.day-visual-canvas ul, .day-visual-canvas ol { padding-left: 24px; margin: 0 0 1em 0; }
		.day-visual-canvas li { margin-bottom: 4px; }
		.day-visual-canvas table { width: 100%; border-collapse: collapse; margin: 16px 0; font: 400 15px/1.5 var(--font-sans); }
		.day-visual-canvas th, .day-visual-canvas td { border-bottom: 1px solid var(--rule); padding: 8px 12px; text-align: left; }
		.day-visual-canvas th { border-bottom-color: var(--rule-strong); font-weight: 600; }
		.day-visual-canvas a { color: var(--ink); text-decoration: underline; text-decoration-color: var(--oxide); text-decoration-thickness: 2px; text-underline-offset: 4px; }

		.day-html-textarea {
			width: 100%;
			min-height: 480px;
			max-height: 800px;
			font: 400 14px/22px var(--font-mono);
			background: var(--paper-sunk);
			color: var(--ink);
			padding: 20px;
			border-radius: var(--radius-sm);
			border: 1px solid var(--rule-strong);
			resize: vertical;
			outline: none;
			tab-size: 4;
			box-sizing: border-box;
		}

		.day-html-textarea:focus {
			outline: 2px solid var(--focus);
			outline-offset: 2px;
		}
"#
}
