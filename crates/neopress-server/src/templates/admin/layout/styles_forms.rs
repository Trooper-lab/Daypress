//! Admin CSS styles: Badges, Buttons, Form Inputs, Toast, and Visual Editor Canvases.

pub fn get_styles_forms() -> &'static str {
    r#"
		/* Status Badges */
		.badge {
			display: inline-flex;
			align-items: center;
			gap: 5px;
			padding: 2px 8px;
			border-radius: 4px;
			font-size: 11.5px;
			font-weight: 600;
		}

		.badge-published {
			background: #ecfdf5;
			color: #059669;
			border: 1px solid #a7f3d0;
		}

		.badge-draft {
			background: #fffbeb;
			color: #d97706;
			border: 1px solid #fde68a;
		}

		/* Buttons */
		.btn {
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
		}

		.btn-primary {
			background: #2563eb;
			color: #ffffff !important;
		}

		.btn-primary:hover {
			background: #1d4ed8;
		}

		.btn-secondary {
			background: #ffffff;
			border-color: var(--border-subtle);
			color: var(--text-main);
		}

		.btn-secondary:hover {
			background: #f8fafc;
			border-color: var(--border-hover);
		}

		.btn-danger {
			background: #fef2f2;
			border-color: #fecaca;
			color: #ef4444;
		}

		.btn-danger:hover {
			background: #fee2e2;
		}

		/* Form Inputs */
		.form-group {
			margin-bottom: 20px;
		}

		.form-label {
			display: block;
			font-size: 13px;
			font-weight: 600;
			color: var(--text-main);
			margin-bottom: 6px;
		}

		.form-hint {
			font-size: 12px;
			color: var(--text-muted);
			margin-top: 4px;
		}

		.form-input, .form-select, .form-textarea {
			width: 100%;
			padding: 8px 12px;
			font-size: 13.5px;
			font-family: var(--font-sans);
			border: 1px solid var(--border-subtle);
			border-radius: 6px;
			background: #ffffff;
			color: var(--text-main);
			transition: border-color 0.15s, box-shadow 0.15s;
		}

		.form-input:focus, .form-select:focus, .form-textarea:focus {
			outline: none;
			border-color: var(--primary);
			box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.12);
		}

		/* Toast Notification */
		#neo-toast {
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
		}

		/* Visual Live Canvas & Source Editor */
		.neo-visual-canvas {
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
		}

		.neo-visual-canvas:focus {
			border-color: var(--primary);
			box-shadow: 0 0 0 3px rgba(37, 99, 235, 0.12);
		}

		.neo-visual-canvas[placeholder]:empty:before {
			content: attr(placeholder);
			color: #94a3b8;
			pointer-events: none;
		}

		.neo-visual-canvas h1 { font-size: 26px; font-weight: 700; margin: 1.2em 0 0.5em 0; color: #0f172a; line-height: 1.3; }
		.neo-visual-canvas h2 { font-size: 21px; font-weight: 700; margin: 1.2em 0 0.5em 0; color: #0f172a; line-height: 1.35; border-bottom: 1px solid #f1f5f9; padding-bottom: 6px; }
		.neo-visual-canvas h3 { font-size: 18px; font-weight: 600; margin: 1em 0 0.4em 0; color: #1e293b; }
		.neo-visual-canvas h4 { font-size: 16px; font-weight: 600; margin: 0.9em 0 0.3em 0; color: #334155; }
		.neo-visual-canvas p { margin: 0 0 1em 0; }
		.neo-visual-canvas blockquote { border-left: 4px solid #2563eb; background: #f8fafc; margin: 16px 0; padding: 12px 18px; border-radius: 0 8px 8px 0; font-style: italic; color: #475569; }
		.neo-visual-canvas img { max-width: 100%; height: auto; border-radius: 8px; margin: 14px 0; box-shadow: 0 4px 6px -1px rgba(0,0,0,0.08); display: block; }
		.neo-visual-canvas code { background: #f1f5f9; padding: 2px 6px; border-radius: 4px; font-family: var(--font-mono); font-size: 0.9em; color: #db2777; }
		.neo-visual-canvas pre { background: #0f172a; color: #f8fafc; padding: 16px; border-radius: 8px; overflow-x: auto; font-family: var(--font-mono); margin: 16px 0; }
		.neo-visual-canvas ul, .neo-visual-canvas ol { padding-left: 24px; margin: 0 0 1em 0; }
		.neo-visual-canvas li { margin-bottom: 4px; }
		.neo-visual-canvas table { width: 100%; border-collapse: collapse; margin: 16px 0; font-size: 14px; }
		.neo-visual-canvas th, .neo-visual-canvas td { border: 1px solid #cbd5e1; padding: 8px 12px; text-align: left; }
		.neo-visual-canvas th { background: #f1f5f9; font-weight: 600; }
		.neo-visual-canvas a { color: #2563eb; text-decoration: underline; }

		.neo-html-textarea {
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
		}
		.neo-html-textarea:focus {
			border-color: #38bdf8;
			box-shadow: 0 0 0 3px rgba(56, 189, 248, 0.2);
		}
"#
}
