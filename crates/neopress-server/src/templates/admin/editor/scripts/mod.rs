//! Client-side Block & Classic Editor runtime scripts (`templates/admin/editor/scripts/`).

mod acf_blocks;
mod acf_render;
mod actions;
mod autosave;
mod core;
mod seo_studio;
mod toolbar;

pub fn render_editor_scripts(
    site_title: &str,
    initial_id: &str,
    has_visual_layout: bool,
    type_singular: &str,
    post_type: &str,
    known_tags_json: &str,
) -> String {
    format!(
        "{}\n{}\n{}\n{}\n{}\n{}\n{}",
        core::render_core_script(site_title, initial_id, has_visual_layout),
        acf_render::JS_ACF_RENDER,
        acf_blocks::JS_ACF_BLOCKS,
        toolbar::JS_TOOLBAR,
        seo_studio::render_seo_script(type_singular),
        autosave::render_autosave_script(post_type),
        actions::render_actions_script(post_type, type_singular, known_tags_json),
    )
}
