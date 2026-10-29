//! Public SSR Theme Stylesheets (`templates/public/styles/mod.rs`).
//!
//! Subdivided into modular, single-responsibility stylesheets strictly <= 400 lines:
//! - `base.rs`: CSS variables, reset, body, header, footer, admin bar.
//! - `elements.rs`: Canonical Daypress AST Element System (.dp-*).
//! - `elementor.rs`: Elementor compatibility, boxed containers, form grids, modals.
//! - `widgets.rs`: Pro Block widgets (PriceCard, Countdown, FlipBox, ProgressBar, etc.).

pub mod base;
pub mod elementor;
pub mod elements;
pub mod widgets;

pub use widgets::CSS_WIDGETS;

/// Assembles the unified public SSR theme stylesheet.
pub fn render_public_theme_css(brand_color: &str) -> String {
    format!(
        "{}\n{}\n{}\n{}",
        base::render_base_css(brand_color),
        elements::CSS_ELEMENTS,
        elementor::CSS_ELEMENTOR,
        widgets::CSS_WIDGETS,
    )
}

/// Convenience alias for compiling public theme CSS.
pub fn render_public_css(brand_color: &str) -> String {
    render_public_theme_css(brand_color)
}
