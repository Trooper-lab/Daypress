//! DayStudio Stylesheet (`templates/public/styles/daystudio.rs`).
//!
//! Provides styles for `html.daystudio-theme-active` overrides,
//! boxed containers, iconlists, form grids, popup/lightbox modals,
//! and responsive media query breakpoints.

pub const CSS_DAYSTUDIO: &str = r#"
		/* ==========================================================================
		   Daystudio Theme Builder, Flexbox Container, Menu, Form & Modal System
		   ========================================================================== */
		html.daystudio-theme-active body {
			background-color: #ffffff;
			color: var(--e-global-color-text, #1F1C17);
		}

		html.daystudio-theme-active .dp-section {
			border-bottom: none;
		}

		html.daystudio-theme-active .dp-heading {
			color: var(--dp-color-heading, var(--e-global-color-secondary, #111827));
			font-family: var(--dp-font-heading, var(--e-global-typography-primary-font-family, inherit));
		}

		html.daystudio-theme-active .dp-text {
			color: var(--dp-color-text, var(--e-global-color-text, #334155));
			font-family: var(--dp-font-body, var(--e-global-typography-text-font-family, inherit));
		}

		html.daystudio-theme-active .dp-button {
			background: var(--e-button-bg, var(--e-global-color-primary, #2563eb));
			color: var(--e-button-color, #ffffff);
			border-radius: var(--e-button-border-radius, 6px);
		}

		html.daystudio-theme-active .dp-price-table {
			background: #ffffff;
			color: var(--dp-color-text, #111827);
			border-color: #e2e8f0;
		}

		html.daystudio-theme-active .dp-price-val,
		html.daystudio-theme-active .dp-price-feature {
			color: inherit;
		}

		.dp-daystudio-header {
			width: 100%;
			background: #ffffff;
			position: relative;
		}

		.dp-daystudio-footer {
			width: 100%;
			position: relative;
		}

		.dp-daystudio-footer section.dp-section {
			padding-left: max(24px, calc((100% - 960px) / 2)) !important;
			padding-right: max(24px, calc((100% - 960px) / 2)) !important;
			border-bottom: none;
		}

		.dp-visual-canvas .dp-container,
		.dp-daystudio-header .dp-container,
		.dp-daystudio-footer .dp-container,
		.dp-popup-modal .dp-container {
			max-width: 100%;
			margin: 0;
			position: relative;
			gap: var(--e-widgets-space, 20px);
			background-size: cover;
			background-position: center;
			background-repeat: no-repeat;
		}

		.dp-container.dp-container-boxed {
			max-width: 100%;
			width: 100%;
		}

		.dp-container-boxed > .dp-container-inner {
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

		.dp-container[style*="flex-direction: row"] > .dp-container:not([style*="width:"]),
		.dp-container[style*="flex-direction: row"] > .dp-container-inner > .dp-container:not([style*="width:"]) {
			flex: 1 1 0%;
			min-width: 0;
		}

		html.daystudio-theme-active .dp-image {
			border-radius: 0;
		}

		html.daystudio-theme-active .dp-imagebox,
		html.daystudio-theme-active .dp-iconbox {
			background: transparent;
			border: none;
			border-radius: 0;
			padding: 0;
		}

		html.daystudio-theme-active .dp-imagebox-title,
		html.daystudio-theme-active .dp-iconbox-title {
			color: inherit;
			font-family: var(--dp-font-heading, var(--e-global-typography-primary-font-family, 'Fira Sans', sans-serif));
		}

		html.daystudio-theme-active .dp-imagebox-desc,
		html.daystudio-theme-active .dp-iconbox-desc {
			color: inherit;
		}

		html.daystudio-theme-active .dp-divider {
			border-top-color: rgba(31, 28, 23, 0.15);
		}

		html.daystudio-theme-active .dp-accordion-item {
			background: #ffffff;
			border: 1px solid #d4d4d8;
			border-radius: 4px;
		}

		html.daystudio-theme-active .dp-accordion-item summary {
			color: var(--e-global-color-secondary, #3A3633);
			font-family: var(--e-global-typography-primary-font-family, 'Fira Sans', sans-serif);
		}

		html.daystudio-theme-active .dp-accordion-body {
			color: var(--e-global-color-text, #1F1C17);
			font-family: var(--e-global-typography-text-font-family, 'Fira Sans', sans-serif);
		}

		.dp-iconlist,
		.dp-icon-list {
			list-style: none;
			margin: 0;
			padding: 0;
			display: flex;
			flex-direction: column;
			gap: 8px;
		}

		.dp-iconlist-item,
		.dp-icon-list-item {
			display: flex;
			align-items: center;
			gap: 10px;
			line-height: 1.65;
		}

		.dp-iconlist-icon,
		.dp-icon-list-item i {
			color: var(--e-global-color-primary, #9CC13C);
			font-size: 16px;
			width: 18px;
			text-align: center;
			flex-shrink: 0;
			display: inline-flex;
			align-items: center;
			justify-content: center;
		}

		.dp-iconlist-item a,
		.dp-icon-list-link {
			display: inline-flex;
			align-items: center;
			gap: 10px;
			color: inherit;
			text-decoration: none;
		}

		.dp-iconlist-item a:hover,
		.dp-icon-list-link:hover {
			color: var(--e-global-color-primary, #9CC13C);
		}

		.dp-daystudio-form-grid {
			display: flex;
			flex-wrap: wrap;
			gap: 16px;
			width: 100%;
		}

		.dp-daystudio-form-grid .dp-field-group {
			display: flex;
			flex-direction: column;
			gap: 6px;
		}

		.dp-daystudio-form-grid .dp-field-w100 {
			width: 100%;
		}

		.dp-daystudio-form-grid .dp-field-w50 {
			width: calc(50% - 8px);
		}

		.dp-daystudio-form-grid .dp-field-w33 {
			width: calc(33.333% - 11px);
		}

		.dp-daystudio-form-grid .dp-field-w25 {
			width: calc(25% - 12px);
		}

		.dp-daystudio-form-grid .dp-field-label {
			font-family: var(--e-global-typography-primary-font-family, 'Fira Sans', sans-serif);
			font-weight: 500;
			font-size: 15px;
			color: var(--e-global-color-secondary, #1F1C17);
		}

		.dp-daystudio-form-grid .dp-input,
		.dp-daystudio-form-grid .dp-textarea,
		.dp-daystudio-form-grid .dp-form-input {
			width: 100%;
			padding: 12px 16px;
			border: 1px solid #d4d4d8;
			border-radius: 4px;
			background: #ffffff;
			color: #1F1C17;
			font-family: inherit;
			font-size: 15px;
		}

		.dp-daystudio-form-grid .dp-field-subgroup,
		.dp-daystudio-form-grid .dp-Subgroup {
			display: flex;
			flex-direction: column;
			gap: 8px;
			padding: 6px 0;
		}

		.dp-daystudio-form-grid .dp-field-subgroup.dp-Subgroup-inline,
		.dp-daystudio-form-grid .dp-Subgroup.dp-Subgroup-inline {
			flex-direction: row;
			flex-wrap: wrap;
			gap: 18px;
		}

		.dp-daystudio-form-grid .dp-option-label,
		.dp-daystudio-form-grid .dp-checkbox-label {
			display: inline-flex;
			align-items: center;
			gap: 8px;
			font-size: 15px;
			cursor: pointer;
		}

		.dp-popup-modal, .dp-lightbox-modal {
			position: fixed;
			inset: 0;
			z-index: 99999;
			display: none;
			align-items: center;
			justify-content: center;
			padding: 24px;
		}

		.dp-popup-modal.active, .dp-lightbox-modal.active {
			display: flex;
		}

		.dp-popup-backdrop, .dp-lightbox-backdrop {
			position: absolute;
			inset: 0;
			background: rgba(0, 0, 0, 0.72);
			backdrop-filter: blur(3px);
		}

		.dp-popup-dialog {
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

		.dp-popup-close, .dp-lightbox-close {
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

		.dp-lightbox-close {
			color: #ffffff;
			font-size: 36px;
			top: 20px;
			right: 28px;
		}

		.dp-lightbox-content {
			position: relative;
			z-index: 1;
			max-width: 92vw;
			max-height: 88vh;
		}

		.dp-lightbox-content img {
			max-width: 92vw;
			max-height: 88vh;
			object-fit: contain;
			border-radius: 4px;
			box-shadow: 0 20px 50px rgba(0,0,0,0.5);
			display: block;
		}

		@media (min-width: 1025px) {
			.dp-hide-desktop {
				display: none !important;
			}
		}

		@media (min-width: 768px) and (max-width: 1024px) {
			.dp-hide-tablet {
				display: none !important;
			}
		}

		@media (max-width: 767px) {
			.dp-hide-mobile {
				display: none !important;
			}
			.dp-daystudio-form-grid .dp-field-w50 {
				width: 100%;
			}
			.dp-container[style*="flex-direction: row"],
			.dp-container[style*="flex-direction: row"] > .dp-container-inner {
				flex-wrap: wrap;
			}
			.dp-container[style*="flex-direction: row"] > .dp-container,
			.dp-container[style*="flex-direction: row"] > .dp-container-inner > .dp-container {
				width: 100% !important;
				flex: 1 1 100% !important;
			}
		}

		@media (max-width: 900px) {
			.footer-grid, .dp-grid {
				grid-template-columns: 1fr !important;
			}
			nav.day-nav {
				display: none;
			}
		}
"#;
