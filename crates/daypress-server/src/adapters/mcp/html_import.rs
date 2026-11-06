//! `daypress_import_html_design`: one call turns an HTML/CSS design into a native DayStudio page,
//! site-wide header and footer theme templates, and FormStudio forms.
//!
//! Re-importing is idempotent: the page is matched by `post_id` or slug, the templates by title
//! ("{site name} header" / "{site name} footer") and the forms by title ("{site name}: {form}").
//! Matching templates and forms are updated in place (form ids, and so their submissions, stay).

use crate::application::CmsService;
use crate::domain::errors::DomainError;
use crate::domain::models::{CreateFormRequest, CreatePostRequest, FormFieldSchema, FormSettings};
use crate::ports::input::{ContentUseCase, SettingsUseCase};
use daypress_ast::{DaypressLayout, ImportedForm};
use serde_json::{json, Value};
use std::collections::HashMap;

pub(super) fn import_html_design(service: &CmsService, args: &Value) -> Result<Value, DomainError> {
    let html = args
        .get("html")
        .and_then(Value::as_str)
        .ok_or_else(|| DomainError::Validation("'html' is required".into()))?;
    if html.trim().is_empty() {
        return Err(DomainError::Validation("'html' cannot be empty".into()));
    }
    let flag = |key: &str, default: bool| args.get(key).and_then(Value::as_bool).unwrap_or(default);
    let extract_header_footer = flag("extract_header_footer", true);
    let extract_forms = flag("extract_forms", true);
    let set_as_front_page = flag("set_as_front_page", false);
    let post_type = args
        .get("post_type")
        .and_then(Value::as_str)
        .unwrap_or("page");

    let mut imported = daypress_ast::transpile_html_document(
        html,
        &daypress_ast::HtmlDesignImportOptions {
            extract_header_footer,
            extract_forms,
        },
    );

    let arg_str = |key: &str| {
        args.get(key)
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
    };
    let resolved_title = arg_str("title")
        .or_else(|| imported.title.clone())
        .unwrap_or_else(|| "Imported Design".to_string());
    let resolved_slug = arg_str("slug").unwrap_or_else(|| {
        if set_as_front_page {
            "home".to_string()
        } else {
            generated_slugify::slugify(&resolved_title)
        }
    });
    let site_name = arg_str("site_name")
        .or_else(|| imported.title.as_deref().map(brand_from_title))
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| resolved_title.clone());

    let target_post_id = if let Some(pid) = args.get("post_id").and_then(Value::as_i64) {
        pid
    } else if let Ok(existing) = service.get_content_by_slug(&resolved_slug) {
        existing.id
    } else {
        let created = service.publish_content(CreatePostRequest {
            title: resolved_title.clone(),
            content: "<p>Importing native design...</p>".to_string(),
            slug: Some(resolved_slug.clone()),
            status: "publish".to_string(),
            post_type: post_type.to_string(),
            ..Default::default()
        })?;
        created.id
    };

    // 1. FormStudio forms, then point the Form nodes at them.
    let saved_forms = save_forms(service, &site_name, &imported.forms)?;
    let ids: HashMap<i64, i64> = saved_forms
        .iter()
        .map(|f| (f.placeholder_id, f.id))
        .collect();
    daypress_ast::replace_form_placeholders(&mut imported.layout.root, &ids);
    for chrome in [&mut imported.header_layout, &mut imported.footer_layout]
        .into_iter()
        .flatten()
    {
        daypress_ast::replace_form_placeholders(&mut chrome.root, &ids);
    }

    // 2. Site header and footer as active, site-wide theme templates.
    let header_template = match imported.header_layout.as_ref() {
        Some(layout) => Some(save_chrome_template(
            service,
            &format!("{site_name} header"),
            "header",
            layout,
        )?),
        None => None,
    };
    let footer_template = match imported.footer_layout.as_ref() {
        Some(layout) => Some(save_chrome_template(
            service,
            &format!("{site_name} footer"),
            "footer",
            layout,
        )?),
        None => None,
    };

    // 3. The page.
    if let Some(ref kit) = imported.kit {
        let kit_css = kit.compile_to_css();
        service.update_site_option("elementor_kit_css", &kit_css)?;
    }
    let layout_json = serde_json::to_string(&imported.layout)
        .map_err(|e| DomainError::Internal(format!("Failed to serialize transpiled AST: {e}")))?;
    service.update_layout_ast(target_post_id, &layout_json)?;

    if set_as_front_page {
        service.update_site_option("show_on_front", "page")?;
        service.update_site_option("page_on_front", &target_post_id.to_string())?;
    }

    Ok(json!({
        "status": "imported",
        "post_id": target_post_id,
        "title": resolved_title,
        "slug": resolved_slug,
        "site_name": site_name,
        "node_count": imported.node_count,
        "token_count": imported.token_count,
        "custom_html_nodes": 0,
        "header_extracted": imported.header_layout.is_some(),
        "footer_extracted": imported.footer_layout.is_some(),
        "header_template": header_template,
        "footer_template": footer_template,
        "forms": saved_forms.iter().map(|f| json!({
            "id": f.id,
            "title": f.title,
            "status": f.status,
            "field_count": f.field_count,
            "shortcode": format!("[daypress_form id=\"{}\"]", f.id),
        })).collect::<Vec<_>>(),
        "set_as_front_page": set_as_front_page
    }))
}

/// "VeroMetal verwerker | Echt metaal | Grezzo" → "Grezzo": the last segment of a page title.
pub(super) fn brand_from_title(title: &str) -> String {
    let mut segment = title.trim();
    for sep in [" | ", " – ", " — ", " - ", " · ", " :: ", " • "] {
        if let Some(last) = segment.rsplit(sep).next() {
            segment = last.trim();
        }
    }
    segment.to_string()
}

struct SavedForm {
    placeholder_id: i64,
    id: i64,
    title: String,
    status: &'static str,
    field_count: usize,
}

fn save_forms(
    service: &CmsService,
    site_name: &str,
    forms: &[ImportedForm],
) -> Result<Vec<SavedForm>, DomainError> {
    if forms.is_empty() {
        return Ok(Vec::new());
    }
    let repo = service.options_repo();
    let existing = repo.list_forms()?;
    let mut used_titles: Vec<String> = Vec::new();
    let mut saved = Vec::new();
    for form in forms {
        let base = format!("{site_name}: {}", form.label);
        let mut title = base.clone();
        let mut n = 2;
        while used_titles.contains(&title) {
            title = format!("{base} ({n})");
            n += 1;
        }
        used_titles.push(title.clone());

        let fields: Vec<FormFieldSchema> = form
            .fields
            .iter()
            .filter_map(|f| serde_json::to_value(f).ok())
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();
        let previous = existing
            .iter()
            .find(|(f, _)| f.title == title)
            .map(|(f, _)| f);
        // Keep what people set on the form (recipient, webhook, captcha...) and refresh what the
        // design owns: the fields, the button label and the success message.
        let mut settings: FormSettings = previous
            .and_then(|f| serde_json::from_str(&f.settings_json).ok())
            .unwrap_or_default();
        if let Some(label) = form.submit_label.clone() {
            settings.submit_label = label;
        }
        if let Some(msg) = form.success_message.clone() {
            settings.success_message = msg;
        }
        if settings.email_subject_template.trim().is_empty() {
            if let Some(subject) = form.email_subject.clone() {
                settings.email_subject_template = subject;
            }
        }
        let id = repo.save_form_record(&CreateFormRequest {
            id: previous.map(|f| f.id),
            title: title.clone(),
            fields,
            settings,
        })?;
        saved.push(SavedForm {
            placeholder_id: form.placeholder_id,
            id,
            title,
            status: if previous.is_some() {
                "updated"
            } else {
                "created"
            },
            field_count: form.fields.len(),
        });
    }
    Ok(saved)
}

/// Creates or updates (matched by title and type) an active header/footer template shown on the
/// entire site. On update the template keeps its display conditions and settings.
fn save_chrome_template(
    service: &CmsService,
    title: &str,
    template_type: &str,
    layout: &DaypressLayout,
) -> Result<Value, DomainError> {
    let repo = service.options_repo();
    let existing_id = repo
        .list_theme_templates(Some(template_type))?
        .iter()
        .find(|t| t.get("title").and_then(Value::as_str) == Some(title))
        .and_then(|t| t.get("id").and_then(Value::as_i64));
    let layout_value = serde_json::to_value(layout).map_err(|e| {
        DomainError::Internal(format!("Failed to serialize {template_type} layout: {e}"))
    })?;
    let mut payload = json!({
        "title": title,
        "template_type": template_type,
        "status": "publish",
        "is_active": true,
        "layout": layout_value,
    });
    if let Some(id) = existing_id {
        payload["id"] = json!(id);
    } else {
        payload["conditions"] =
            json!([{ "condition_type": "include", "rule_target": "entire_site" }]);
    }
    let saved = repo.save_theme_template_from_json(&payload)?;
    Ok(json!({
        "id": saved.get("id").cloned().unwrap_or(Value::Null),
        "title": title,
        "status": if existing_id.is_some() { "updated" } else { "created" },
    }))
}

#[cfg(test)]
mod tests {
    use super::brand_from_title;

    #[test]
    fn brand_is_the_last_title_segment() {
        assert_eq!(
            brand_from_title("VeroMetal verwerker | Echt metaal, met de hand gemaakt | Grezzo"),
            "Grezzo"
        );
        assert_eq!(brand_from_title("Contact – Acme"), "Acme");
        assert_eq!(brand_from_title("Studio Noord"), "Studio Noord");
    }
}
