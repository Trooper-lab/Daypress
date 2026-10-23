//! Elementor Compatibility Stylesheet (`templates/public/styles/elementor.rs`).
//!
//! Provides styles for `html.elementor-theme-active` overrides,
//! boxed containers, iconlists, form grids, popup/lightbox modals,
//! and responsive media query breakpoints.

pub const CSS_ELEMENTOR: &str = r#"
		/* ==========================================================================
		   Elementor Theme Builder, Flexbox Container, Menu, Form & Modal System
		   ========================================================================== */
		html.elementor-theme-active body {
			background-color: #ffffff;
			color: var(--e-global-color-text, #1F1C17);
		}

		html.elementor-theme-active .np-section {
			border-bottom: none;
		}

		html.elementor-theme-active .np-heading {
			color: var(--np-color-heading, var(--e-global-color-secondary, #111827));
			font-family: var(--np-font-heading, var(--e-global-typography-primary-font-family, inherit));
		}

		html.elementor-theme-active .np-text {
			color: var(--np-color-text, var(--e-global-color-text, #334155));
			font-family: var(--np-font-body, var(--e-global-typography-text-font-family, inherit));
		}

		html.elementor-theme-active .np-button {
			background: var(--e-button-bg, var(--e-global-color-primary, #2563eb));
			color: var(--e-button-color, #ffffff);
			border-radius: var(--e-button-border-radius, 6px);
		}

		html.elementor-theme-active .np-price-table {
			background: #ffffff;
			color: var(--np-color-text, #111827);
			border-color: #e2e8f0;
		}

		html.elementor-theme-active .np-price-val,
		html.elementor-theme-active .np-price-feature {
			color: inherit;
		}

		.np-elementor-header {
			width: 100%;
			background: #ffffff;
			position: relative;
		}

		.np-elementor-footer {
			width: 100%;
			position: relative;
		}

		.np-elementor-footer section.np-section {
			padding-left: max(24px, calc((100% - 960px) / 2)) !important;
			padding-right: max(24px, calc((100% - 960px) / 2)) !important;
			border-bottom: none;
		}

		.np-visual-canvas .np-container,
		.np-elementor-header .np-container,
		.np-elementor-footer .np-container,
		.np-popup-modal .np-container {
			max-width: 100%;
			margin: 0;
			position: relative;
			gap: var(--e-widgets-space, 20px);
			background-size: cover;
			background-position: center;
			background-repeat: no-repeat;
		}

		.np-container.np-container-boxed {
			max-width: 100%;
			width: 100%;
		}

		.np-container-boxed > .np-container-inner {
			max-width: var(--e-content-width, 1140px);
			width: 100%;
			margin: 0 auto;
			display: flex;
			flex-direction: inherit;
			justify-content: inherit;
			align-items: inherit;
			gap: inherit;
			flex-wrap: inherit;
			min-height: inherit;
		}

		.np-container[style*="flex-direction: row"] > .np-container:not([style*="width:"]),
		.np-container[style*="flex-direction: row"] > .np-container-inner > .np-container:not([style*="width:"]) {
			flex: 1 1 0%;
			min-width: 0;
		}

		html.elementor-theme-active .np-image {
			border-radius: 0;
		}

		html.elementor-theme-active .np-imagebox,
		html.elementor-theme-active .np-iconbox {
			background: transparent;
			border: none;
			border-radius: 0;
			padding: 0;
		}

		html.elementor-theme-active .np-imagebox-title,
		html.elementor-theme-active .np-iconbox-title {
			color: inherit;
			font-family: var(--np-font-heading, var(--e-global-typography-primary-font-family, 'Fira Sans', sans-serif));
		}

		html.elementor-theme-active .np-imagebox-desc,
		html.elementor-theme-active .np-iconbox-desc {
			color: inherit;
		}

		html.elementor-theme-active .np-divider {
			border-top-color: rgba(31, 28, 23, 0.15);
		}

		html.elementor-theme-active .np-accordion-item {
			background: #ffffff;
			border: 1px solid #d4d4d8;
			border-radius: 4px;
		}

		html.elementor-theme-active .np-accordion-item summary {
			color: var(--e-global-color-secondary, #3A3633);
			font-family: var(--e-global-typography-primary-font-family, 'Fira Sans', sans-serif);
		}

		html.elementor-theme-active .np-accordion-body {
			color: var(--e-global-color-text, #1F1C17);
			font-family: var(--e-global-typography-text-font-family, 'Fira Sans', sans-serif);
		}

		.np-iconlist,
		.np-icon-list {
			list-style: none;
			margin: 0;
			padding: 0;
			display: flex;
			flex-direction: column;
			gap: 8px;
		}

		.np-iconlist-item,
		.np-icon-list-item {
			display: flex;
			align-items: center;
			gap: 10px;
			line-height: 1.65;
		}

		.np-iconlist-icon,
		.np-icon-list-item i {
			color: var(--e-global-color-primary, #9CC13C);
			font-size: 16px;
			width: 18px;
			text-align: center;
			flex-shrink: 0;
			display: inline-flex;
			align-items: center;
			justify-content: center;
		}

		.np-iconlist-item a,
		.np-icon-list-link {
			display: inline-flex;
			align-items: center;
			gap: 10px;
			color: inherit;
			text-decoration: none;
		}

		.np-iconlist-item a:hover,
		.np-icon-list-link:hover {
			color: var(--e-global-color-primary, #9CC13C);
		}

		.np-elementor-form-grid {
			display: flex;
			flex-wrap: wrap;
			gap: 16px;
			width: 100%;
		}

		.np-elementor-form-grid .np-field-group {
			display: flex;
			flex-direction: column;
			gap: 6px;
		}

		.np-elementor-form-grid .np-field-w100 {
			width: 100%;
		}

		.np-elementor-form-grid .np-field-w50 {
			width: calc(50% - 8px);
		}

		.np-elementor-form-grid .np-field-w33 {
			width: calc(33.333% - 11px);
		}

		.np-elementor-form-grid .np-field-w25 {
			width: calc(25% - 12px);
		}

		.np-elementor-form-grid .np-field-label {
			font-family: var(--e-global-typography-primary-font-family, 'Fira Sans', sans-serif);
			font-weight: 500;
			font-size: 15px;
			color: var(--e-global-color-secondary, #1F1C17);
		}

		.np-elementor-form-grid .np-input,
		.np-elementor-form-grid .np-textarea,
		.np-elementor-form-grid .np-form-input {
			width: 100%;
			padding: 12px 16px;
			border: 1px solid #d4d4d8;
			border-radius: 4px;
			background: #ffffff;
			color: #1F1C17;
			font-family: inherit;
			font-size: 15px;
		}

		.np-elementor-form-grid .np-field-subgroup,
		.np-elementor-form-grid .np-Subgroup {
			display: flex;
			flex-direction: column;
			gap: 8px;
			padding: 6px 0;
		}

		.np-elementor-form-grid .np-field-subgroup.np-Subgroup-inline,
		.np-elementor-form-grid .np-Subgroup.np-Subgroup-inline {
			flex-direction: row;
			flex-wrap: wrap;
			gap: 18px;
		}

		.np-elementor-form-grid .np-option-label,
		.np-elementor-form-grid .np-checkbox-label {
			display: inline-flex;
			align-items: center;
			gap: 8px;
			font-size: 15px;
			cursor: pointer;
		}

		.np-popup-modal, .np-lightbox-modal {
			position: fixed;
			inset: 0;
			z-index: 99999;
			display: none;
			align-items: center;
			justify-content: center;
			padding: 24px;
		}

		.np-popup-modal.active, .np-lightbox-modal.active {
			display: flex;
		}

		.np-popup-backdrop, .np-lightbox-backdrop {
			position: absolute;
			inset: 0;
			background: rgba(0, 0, 0, 0.72);
			backdrop-filter: blur(3px);
		}

		.np-popup-dialog {
			position: relative;
			z-index: 1;
			background: #ffffff;
			border-radius: 8px;
			max-width: 680px;
			width: 100%;
			max-height: 90vh;
			overflow-y: auto;
			box-shadow: 0 24px 60px rgba(0, 0, 0, 0.35);
			padding: 32px;
		}

		.np-popup-close, .np-lightbox-close {
			position: absolute;
			top: 12px;
			right: 16px;
			z-index: 2;
			background: transparent;
			border: none;
			font-size: 28px;
			line-height: 1;
			color: #1F1C17;
			cursor: pointer;
		}

		.np-lightbox-close {
			color: #ffffff;
			font-size: 36px;
			top: 20px;
			right: 28px;
		}

		.np-lightbox-content {
			position: relative;
			z-index: 1;
			max-width: 92vw;
			max-height: 88vh;
		}

		.np-lightbox-content img {
			max-width: 92vw;
			max-height: 88vh;
			object-fit: contain;
			border-radius: 4px;
			box-shadow: 0 20px 50px rgba(0,0,0,0.5);
			display: block;
		}

		@media (min-width: 1025px) {
			.np-hide-desktop {
				display: none !important;
			}
		}

		@media (min-width: 768px) and (max-width: 1024px) {
			.np-hide-tablet {
				display: none !important;
			}
		}

		@media (max-width: 767px) {
			.np-hide-mobile {
				display: none !important;
			}
			.np-elementor-form-grid .np-field-w50 {
				width: 100%;
			}
			.np-container[style*="flex-direction: row"],
			.np-container[style*="flex-direction: row"] > .np-container-inner {
				flex-wrap: wrap;
			}
			.np-container[style*="flex-direction: row"] > .np-container,
			.np-container[style*="flex-direction: row"] > .np-container-inner > .np-container {
				width: 100% !important;
				flex: 1 1 100% !important;
			}
		}

		@media (max-width: 900px) {
			.footer-grid, .np-grid {
				grid-template-columns: 1fr !important;
			}
			nav.neo-nav {
				display: none;
			}
		}
"#;
