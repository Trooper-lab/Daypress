//! Universal Theme Template Persistence, Elementor Library Ingestion & Display Conditions.

pub use crate::importer::menus::{
    build_universal_theme_footer_html, build_universal_theme_header_html,
    render_universal_nav_items_recursive,
};
use crate::theme_builder::models::{ConditionType, RuleTarget, TemplateSettings};
use rusqlite::{params, OptionalExtension, Result, Transaction};

/// Ensures template and condition tables exist in SQLite.
pub fn ensure_template_tables_exist(tx: &Transaction) -> Result<()> {
    tx.execute_batch(
        "CREATE TABLE IF NOT EXISTS wp_neopress_templates (id INTEGER PRIMARY KEY AUTOINCREMENT, title TEXT NOT NULL, template_type TEXT NOT NULL, status TEXT NOT NULL DEFAULT 'publish', layout_json TEXT NOT NULL DEFAULT '{}', rendered_html TEXT DEFAULT NULL, settings_json TEXT NOT NULL DEFAULT '{}', priority INTEGER NOT NULL DEFAULT 10, is_active INTEGER NOT NULL DEFAULT 1, created_at TEXT NOT NULL DEFAULT (datetime('now')), updated_at TEXT NOT NULL DEFAULT (datetime('now')));
        CREATE TABLE IF NOT EXISTS wp_neopress_template_conditions (id INTEGER PRIMARY KEY AUTOINCREMENT, template_id INTEGER NOT NULL REFERENCES wp_neopress_templates(id) ON DELETE CASCADE, condition_type TEXT NOT NULL DEFAULT 'include', rule_target TEXT NOT NULL, target_id TEXT NOT NULL DEFAULT '', priority INTEGER NOT NULL DEFAULT 10, created_at TEXT NOT NULL DEFAULT (datetime('now')));"
    )
}

/// Inserts or updates a header or footer template in `wp_neopress_templates`
/// and assigns the 'Entire Site' inclusion condition (priority 100).
pub fn upsert_theme_template(
    tx: &Transaction,
    title: &str,
    template_type: &str,
    content_html: &str,
    layout: Option<&neopress_ast::NeopressLayout>,
    priority: i32,
) -> Result<i64> {
    ensure_template_tables_exist(tx)?;
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let existing_id: Option<i64> = tx
        .query_row(
            "SELECT id FROM wp_neopress_templates WHERE template_type = ?1 AND is_active = 1 ORDER BY priority DESC LIMIT 1",
            params![template_type],
            |r| r.get(0),
        )
        .optional()?;

    let layout_json = match layout {
        Some(l) => serde_json::to_string(l).unwrap_or_else(|_| "{}".to_string()),
        None => {
            let custom_layout = neopress_ast::NeopressLayout {
                version: 1,
                root: neopress_ast::NeopressNode {
                    id: format!("tpl_{}_root", template_type),
                    kind: neopress_ast::NodeKind::CustomHtml { raw: content_html.to_string() },
                    children: Vec::new(),
                    styles: neopress_ast::StyleProperties::default(),
                },
                global_styles: None,
            };
            serde_json::to_string(&custom_layout).unwrap_or_else(|_| "{}".to_string())
        }
    };

    let template_id = match existing_id {
        Some(id) => {
            tx.execute(
                "UPDATE wp_neopress_templates SET title = ?1, layout_json = ?2, rendered_html = ?3, priority = ?4, updated_at = ?5 WHERE id = ?6",
                params![title, layout_json, content_html, priority, now, id],
            )?;
            id
        }
        None => {
            tx.execute(
                "INSERT INTO wp_neopress_templates (title, template_type, status, layout_json, rendered_html, settings_json, priority, is_active, created_at, updated_at) VALUES (?1, ?2, 'publish', ?3, ?4, '{}', ?5, 1, ?6, ?6)",
                params![title, template_type, layout_json, content_html, priority, now],
            )?;
            let id = tx.last_insert_rowid();
            let _ = tx.execute(
                "INSERT INTO wp_neopress_template_conditions (template_id, condition_type, rule_target, target_id, priority, created_at) VALUES (?1, 'include', 'entire_site', '', ?2, ?3)",
                params![id, priority, now],
            );
            id
        }
    };

    let opt_name = if template_type == "header" { "neopress_elementor_header_html" } else { "neopress_elementor_footer_html" };
    let _ = tx.execute(
        "INSERT INTO wp_options (option_name, option_value) VALUES (?1, ?2) ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
        params![opt_name, content_html],
    );
    Ok(template_id)
}

/// Normalizes various Elementor template type strings into Neopress canonical template types.
pub fn normalize_template_type(raw_type: &str) -> &'static str {
    match raw_type.trim().to_lowercase().as_str() {
        "header" => "header",
        "footer" => "footer",
        "single-post" | "single_post" | "single" | "post" => "single_post",
        "single-page" | "single_page" | "page" => "single_page",
        "archive" | "category" | "tag" | "blog" | "search-results" => "archive",
        "404_page" | "404-page" | "404" | "error404" | "not_found" => "404_page",
        "popup" | "modal" => "popup",
        "section" | "container" | "section_symbol" | "symbol" => "section_symbol",
        _ => "header",
    }
}

/// Safely extracts a vector of condition rule strings from PHP serialized or JSON metadata.
pub fn extract_condition_strings(raw: &str) -> Vec<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() { return Vec::new(); }
    if trimmed.starts_with("a:") {
        if let Some(val) = neopress_ast::parse_php_serialized_to_json(trimmed) {
            let items: Vec<String> = match val {
                serde_json::Value::Array(a) => a.into_iter().filter_map(|v| v.as_str().map(String::from)).collect(),
                serde_json::Value::Object(m) => m.into_iter().filter_map(|(_, v)| v.as_str().map(String::from)).collect(),
                _ => Vec::new(),
            };
            if !items.is_empty() { return items; }
        }
    }
    if trimmed.starts_with('[') {
        if let Ok(strings) = serde_json::from_str::<Vec<String>>(trimmed) { return strings; }
    }
    trimmed.split(',').map(|s| s.trim().trim_matches('"').to_string()).filter(|s| !s.is_empty()).collect()
}

/// Parses an Elementor condition rule string into typed (ConditionType, RuleTarget, target_id, priority).
pub fn parse_elementor_condition_rule(rule_str: &str) -> Option<(ConditionType, RuleTarget, String, i32)> {
    let trimmed = rule_str.trim();
    if trimmed.is_empty() { return None; }
    let parts: Vec<&str> = trimmed
        .split('/')
        .map(|s| s.trim().trim_matches('"'))
        .collect();
    if parts.is_empty() { return None; }

    let (c_type, offset) = match parts[0].to_lowercase().as_str() {
        "exclude" | "ex" => (ConditionType::Exclude, 1),
        "include" | "in" => (ConditionType::Include, 1),
        _ => (ConditionType::Include, 0),
    };
    let rem = &parts[offset..];
    let cat = rem.first().copied().unwrap_or("general").to_lowercase();
    let sub = rem.get(1).copied().unwrap_or("").to_lowercase();
    let id_part = rem.get(2).copied().unwrap_or("");

    let (target, target_id, priority) = match cat.as_str() {
        "general" | "entire_site" | "all" | "site" | "" => (RuleTarget::EntireSite, String::new(), 10),
        "singular" => match sub.as_str() {
            "front_page" | "home" => (RuleTarget::FrontPage, String::new(), 40),
            "not_found404" | "404" | "not_found" => (RuleTarget::NotFound, String::new(), 40),
            "posts" | "all_posts" => (RuleTarget::AllPosts, String::new(), 20),
            "pages" | "all_pages" => (RuleTarget::AllPages, String::new(), 20),
            "post" => (!id_part.is_empty()).then(|| (RuleTarget::Post, id_part.to_string(), 40)).unwrap_or((RuleTarget::AllPosts, String::new(), 20)),
            "page" => (!id_part.is_empty()).then(|| (RuleTarget::Page, id_part.to_string(), 40)).unwrap_or((RuleTarget::AllPages, String::new(), 20)),
            "by_author" | "author" => (RuleTarget::Author, id_part.to_string(), 30),
            "in_category" | "category" => (RuleTarget::Category, id_part.to_string(), 30),
            "in_tag" | "tag" => (RuleTarget::Tag, id_part.to_string(), 30),
            _ => (RuleTarget::AllPosts, String::new(), 20),
        },
        "archive" => match sub.as_str() {
            "category" | "categories" => (RuleTarget::Category, id_part.to_string(), 30),
            "tag" | "tags" => (RuleTarget::Tag, id_part.to_string(), 30),
            "author" => (RuleTarget::Author, id_part.to_string(), 30),
            _ => (RuleTarget::Archive, String::new(), 20),
        },
        "not_found" | "not_found404" | "404" => (RuleTarget::NotFound, String::new(), 40),
        "front_page" | "home" => (RuleTarget::FrontPage, String::new(), 40),
        _ => (RuleTarget::EntireSite, String::new(), 10),
    };
    Some((c_type, target, target_id, priority))
}

/// Parses raw condition string into a list of (ConditionType, RuleTarget, String target_id).
pub fn parse_elementor_conditions(raw: &str) -> Vec<(ConditionType, RuleTarget, String)> {
    extract_condition_strings(raw)
        .into_iter()
        .filter_map(|s| parse_elementor_condition_rule(&s))
        .map(|(c, r, id, _)| (c, r, id))
        .collect()
}

/// Resolves conditions with sensible defaults if raw conditions are missing.
pub fn resolve_elementor_conditions(
    raw_conditions: Option<&str>,
    tpl_type: &str,
) -> Vec<(ConditionType, RuleTarget, String, i32)> {
    let mut conds = Vec::new();
    if let Some(raw) = raw_conditions {
        for rule_str in extract_condition_strings(raw) {
            if let Some(c) = parse_elementor_condition_rule(&rule_str) {
                conds.push(c);
            }
        }
    }
    if conds.is_empty() {
        let (rule_target, priority) = match tpl_type {
            "header" | "footer" => (RuleTarget::EntireSite, 10),
            "single_post" => (RuleTarget::AllPosts, 20),
            "single_page" => (RuleTarget::AllPages, 20),
            "archive" => (RuleTarget::Archive, 20),
            "404_page" => (RuleTarget::NotFound, 40),
            _ => (RuleTarget::EntireSite, 10),
        };
        conds.push((ConditionType::Include, rule_target, String::new(), priority));
    }
    conds
}

/// Ingests an Elementor Library template and its display conditions into SQLite.
#[allow(clippy::too_many_arguments)]
pub fn ingest_elementor_library_template(
    tx: &Transaction,
    title: &str,
    raw_type: &str,
    layout: &neopress_ast::NeopressLayout,
    rendered_html: &str,
    raw_conditions: Option<&str>,
    page_settings_raw: Option<&str>,
    priority: i32,
    old_to_new_post_id: &std::collections::HashMap<i64, i64>,
) -> Result<i64> {
    ensure_template_tables_exist(tx)?;
    let tpl_type = normalize_template_type(raw_type);
    let conditions = resolve_elementor_conditions(raw_conditions, tpl_type);
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let mut settings = TemplateSettings::default();
    if let Some(ps_raw) = page_settings_raw {
        let settings_val = if ps_raw.trim().starts_with("a:") {
            neopress_ast::parse_php_serialized_to_json(ps_raw)
        } else {
            serde_json::from_str::<serde_json::Value>(ps_raw).ok()
        };
        if let Some(val) = settings_val {
            if let Some(sticky) = val.get("sticky").and_then(|v| v.as_str()) {
                settings.sticky = sticky == "yes" || sticky == "top";
            }
            if let Some(transparent) = val.get("transparent").and_then(|v| v.as_str()) {
                settings.transparent = transparent == "yes";
            }
            if let Some(css) = val.get("custom_css").and_then(|v| v.as_str()) {
                settings.custom_css = Some(css.to_string());
            }
        }
    }
    let settings_json = serde_json::to_string(&settings).unwrap_or_else(|_| "{}".to_string());
    let layout_json = serde_json::to_string(layout).unwrap_or_else(|_| "{}".to_string());

    let existing_id: Option<i64> = tx
        .query_row(
            "SELECT id FROM wp_neopress_templates WHERE title = ?1 AND template_type = ?2 LIMIT 1",
            params![title, tpl_type],
            |r| r.get(0),
        )
        .optional()?;

    let template_id = match existing_id {
        Some(id) => {
            tx.execute(
                "UPDATE wp_neopress_templates SET layout_json = ?1, rendered_html = ?2, settings_json = ?3, priority = ?4, updated_at = ?5 WHERE id = ?6",
                params![layout_json, rendered_html, settings_json, priority, now, id],
            )?;
            id
        }
        None => {
            tx.execute(
                "INSERT INTO wp_neopress_templates (title, template_type, status, layout_json, rendered_html, settings_json, priority, is_active, created_at, updated_at) VALUES (?1, ?2, 'publish', ?3, ?4, ?5, ?6, 1, ?7, ?7)",
                params![title, tpl_type, layout_json, rendered_html, settings_json, priority, now],
            )?;
            tx.last_insert_rowid()
        }
    };

    let _ = tx.execute("DELETE FROM wp_neopress_template_conditions WHERE template_id = ?1", params![template_id]);
    for (c_type, r_target, target_id, cond_priority) in &conditions {
        let final_target_id = target_id
            .parse::<i64>()
            .ok()
            .and_then(|old_id| old_to_new_post_id.get(&old_id).map(|new_id| new_id.to_string()))
            .unwrap_or_else(|| target_id.clone());

        let _ = tx.execute(
            "INSERT INTO wp_neopress_template_conditions (template_id, condition_type, rule_target, target_id, priority, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![template_id, c_type.as_str(), r_target.as_str(), final_target_id, cond_priority, now],
        );
    }

    let is_entire_site = conditions.iter().any(|(c, r, _, _)| *c == ConditionType::Include && *r == RuleTarget::EntireSite);
    let has_exclusions = conditions.iter().any(|(c, _, _, _)| *c == ConditionType::Exclude);
    if is_entire_site && !has_exclusions {
        let opt_name = if tpl_type == "header" { "neopress_elementor_header_html" } else { "neopress_elementor_footer_html" };
        let _ = tx.execute(
            "INSERT INTO wp_options (option_name, option_value) VALUES (?1, ?2) ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;",
            params![opt_name, rendered_html],
        );
    }

    Ok(template_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_elementor_conditions_php_serialized() {
        let raw = r#"a:2:{i:0;s:15:"include/general";i:1;s:24:"exclude/singular/page/42";}"#;
        let conds = parse_elementor_conditions(raw);
        assert_eq!(conds.len(), 2);
        assert_eq!(conds[0], (ConditionType::Include, RuleTarget::EntireSite, String::new()));
        assert_eq!(conds[1], (ConditionType::Exclude, RuleTarget::Page, "42".to_string()));
    }

    #[test]
    fn test_parse_elementor_conditions_json() {
        let raw = r#"["include/general", "include/singular/front_page"]"#;
        let conds = parse_elementor_conditions(raw);
        assert_eq!(conds.len(), 2);
        assert_eq!(conds[0], (ConditionType::Include, RuleTarget::EntireSite, String::new()));
        assert_eq!(conds[1], (ConditionType::Include, RuleTarget::FrontPage, String::new()));
    }
}
