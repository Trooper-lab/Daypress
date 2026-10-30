//! Milestone 5: In-Process Rust Multi-Site Integration Stress-Test Suite
//! (`crates/daypress-server/tests/multi_site_stress_tests.rs`)
//!
//! Validates relational integrity, AST transpilation, preset CSS compilation,
//! and public SSR HTML output across all 4 target WordPress site archetypes:
//! - Archetype 1: WordPress Theme Unit Test (Gutenberg, 3+ level menus, sticky posts, comments)
//! - Archetype 2: Gutenberg Full Site Editing (Twenty Twenty-Four, theme.json presets)
//! - Archetype 3: Elementor Starter Site (Astra kit, containers, internal link remapping)
//! - Archetype 4: Custom PHP + ACF + CPT (NORTHWIND theme, dynamic CPT archives, custom taxonomies)

mod multi_site_stress;

#[tokio::test]
async fn test_archetype1_theme_unit_test_relational_and_ssr() {
    multi_site_stress::archetype1_theme_unit_test::run_test().await;
}

#[tokio::test]
async fn test_archetype2_gutenberg_fse_presets_and_templates() {
    multi_site_stress::archetype2_gutenberg_fse::run_test().await;
}

#[tokio::test]
async fn test_archetype3_daystudio_starter_kit_and_remapping() {
    multi_site_stress::archetype3_daystudio_starter::run_test().await;
}

#[tokio::test]
async fn test_archetype4_custom_acf_cpt_routes_and_styles() {
    multi_site_stress::archetype4_custom_acf::run_test().await;
}
