//! ACF (Advanced Custom Fields) & CPT UI Schema Ingestion and Postmeta Relational Remapper.
//!
//! Translates WordPress `acf-field-group`, `acf-field`, `acf-post-type`, `acf-taxonomy`
//! posts and `cptui_post_types` / `cptui_taxonomies` options into canonical Daypress
//! `daypress_post_types`, `daypress_taxonomies`, and `daypress_field_groups` rows,
//! and remaps relational/media IDs + flattened repeater rows after Pass 1 post ingestion.

use crate::adapters::database::content_model::{
    default_icon_for_cpt, humanize_slug, pluralize_label, sanitize_model_slug,
};
use crate::importer::models::ParsedWxrItem;
use crate::models::{
    CustomPostTypeDefinition, CustomTaxonomyDefinition, FieldGroupDefinition,
    FieldGroupLocationRule, ImportPost, StructuredFieldSchema,
};
use rusqlite::{params, Result, Transaction};
use serde_json::Value;
use std::collections::HashMap;

/// Parses either a JSON string or a PHP-serialized string into a `serde_json::Value`.
pub fn parse_serialized_or_json(raw: &str) -> Option<Value> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
    {
        if let Ok(val) = serde_json::from_str::<Value>(trimmed) {
            return Some(val);
        }
    }
    daypress_ast::parse_php_serialized_to_json(trimmed)
}

#[derive(Clone, Debug)]
struct RawAcfItem {
    id: i64,
    parent: i64,
    post_type: String,
    title: String,
    slug: String,
    excerpt: String,
    content: String,
    menu_order: i64,
}

pub fn ingest_acf_and_cpt_from_wxr(tx: &Transaction, items: &[ParsedWxrItem]) -> Result<usize> {
    let raw_items: Vec<RawAcfItem> = items
        .iter()
        .filter(|i| {
            matches!(
                i.post_type.as_str(),
                "acf-field-group" | "acf-field" | "acf-post-type" | "acf-taxonomy"
            )
        })
        .map(|i| RawAcfItem {
            id: i.id.unwrap_or(0),
            parent: i.parent,
            post_type: i.post_type.clone(),
            title: i.title.clone(),
            slug: i.slug.clone(),
            excerpt: i.excerpt.clone(),
            content: i.content.clone(),
            menu_order: i.menu_order,
        })
        .collect();

    ingest_raw_acf_items(tx, &raw_items)
}

pub fn ingest_acf_and_cpt_from_bundle(
    tx: &Transaction,
    posts: &[ImportPost],
    plugin_options: &HashMap<String, Value>,
) -> Result<Vec<String>> {
    let mut converged = Vec::new();

    let raw_items: Vec<RawAcfItem> = posts
        .iter()
        .filter(|p| {
            matches!(
                p.post_type.as_str(),
                "acf-field-group" | "acf-field" | "acf-post-type" | "acf-taxonomy"
            )
        })
        .map(|p| RawAcfItem {
            id: p.id.unwrap_or(0),
            parent: p.parent.unwrap_or(0),
            post_type: p.post_type.clone(),
            title: p.title.clone(),
            slug: p.slug.clone(),
            excerpt: p.excerpt.clone(),
            content: p.content.clone(),
            menu_order: p.menu_order.unwrap_or(0),
        })
        .collect();

    let acf_count = ingest_raw_acf_items(tx, &raw_items)?;
    if acf_count > 0 {
        converged.push(format!(
            "Advanced Custom Fields ({acf_count} schemas) -> Daypress Content Model"
        ));
    }

    let cptui_count = ingest_cptui_options(tx, plugin_options)?;
    if cptui_count > 0 {
        converged.push(format!(
            "Custom Post Type UI ({cptui_count} definitions) -> Daypress Content Model"
        ));
    }

    Ok(converged)
}

fn ingest_raw_acf_items(tx: &Transaction, items: &[RawAcfItem]) -> Result<usize> {
    if items.is_empty() {
        return Ok(0);
    }
    let mut total_ingested = 0usize;

    // 1. ACF Post Types (`acf-post-type`)
    for item in items.iter().filter(|i| i.post_type == "acf-post-type") {
        let cfg = parse_serialized_or_json(&item.content).unwrap_or_else(|| serde_json::json!({}));
        let raw_slug = cfg
            .get("post_type")
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| {
                if !item.excerpt.trim().is_empty() {
                    item.excerpt.trim()
                } else {
                    item.slug.trim()
                }
            });
        let slug = sanitize_model_slug(raw_slug);
        if slug.is_empty() {
            continue;
        }
        let label_plural = if item.title.trim().is_empty() {
            pluralize_label(&humanize_slug(&slug))
        } else {
            item.title.trim().to_string()
        };
        let label_singular = cfg
            .get("labels")
            .and_then(|l| l.get("singular_name"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(String::from)
            .unwrap_or_else(|| humanize_slug(&slug));

        let hierarchical = cfg
            .get("hierarchical")
            .and_then(|v| v.as_bool().or_else(|| v.as_i64().map(|n| n != 0)))
            .unwrap_or(false);
        let has_archive = cfg
            .get("has_archive")
            .and_then(|v| v.as_bool().or_else(|| v.as_i64().map(|n| n != 0)))
            .unwrap_or(true);
        let taxonomies: Vec<String> = cfg
            .get("taxonomies")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(sanitize_model_slug))
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let def = CustomPostTypeDefinition {
            slug: slug.clone(),
            label_singular,
            label_plural,
            description: cfg
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            icon: default_icon_for_cpt(&slug).to_string(),
            menu_position: 25,
            public: true,
            has_archive,
            hierarchical,
            rewrite_slug: slug,
            supports: vec![
                "title".into(),
                "editor".into(),
                "thumbnail".into(),
                "excerpt".into(),
                "custom-fields".into(),
            ],
            taxonomies,
            show_in_rest: true,
            preset_key: None,
            post_count: 0,
        };
        upsert_cpt_tx(tx, &def)?;
        total_ingested += 1;
    }

    // 2. ACF Taxonomies (`acf-taxonomy`)
    for item in items.iter().filter(|i| i.post_type == "acf-taxonomy") {
        let cfg = parse_serialized_or_json(&item.content).unwrap_or_else(|| serde_json::json!({}));
        let raw_slug = cfg
            .get("taxonomy")
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| {
                if !item.excerpt.trim().is_empty() {
                    item.excerpt.trim()
                } else {
                    item.slug.trim()
                }
            });
        let slug = sanitize_model_slug(raw_slug);
        if slug.is_empty() {
            continue;
        }
        let label_plural = if item.title.trim().is_empty() {
            pluralize_label(&humanize_slug(&slug))
        } else {
            item.title.trim().to_string()
        };
        let label_singular = cfg
            .get("labels")
            .and_then(|l| l.get("singular_name"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.trim().is_empty())
            .map(String::from)
            .unwrap_or_else(|| humanize_slug(&slug));

        let post_types: Vec<String> = cfg
            .get("object_type")
            .or_else(|| cfg.get("post_types"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|x| x.as_str().map(sanitize_model_slug))
                    .filter(|s| !s.is_empty())
                    .collect()
            })
            .unwrap_or_default();

        let hierarchical = cfg
            .get("hierarchical")
            .and_then(|v| v.as_bool().or_else(|| v.as_i64().map(|n| n != 0)))
            .unwrap_or(true);

        let tax_def = CustomTaxonomyDefinition {
            slug: slug.clone(),
            label_singular,
            label_plural,
            description: String::new(),
            hierarchical,
            public: true,
            rewrite_slug: slug,
            post_types,
            show_in_rest: true,
            term_count: 0,
        };
        upsert_taxonomy_tx(tx, &tax_def)?;
        total_ingested += 1;
    }

    // 3. ACF Field Groups (`acf-field-group`) & Fields (`acf-field`)
    let mut fields_by_parent: HashMap<i64, Vec<&RawAcfItem>> = HashMap::new();
    for f in items.iter().filter(|i| i.post_type == "acf-field") {
        fields_by_parent.entry(f.parent).or_default().push(f);
    }
    for list in fields_by_parent.values_mut() {
        list.sort_by_key(|f| (f.menu_order, f.id));
    }

    for group_item in items.iter().filter(|i| i.post_type == "acf-field-group") {
        let cfg =
            parse_serialized_or_json(&group_item.content).unwrap_or_else(|| serde_json::json!({}));
        let group_key = if group_item.slug.starts_with("group_") {
            group_item.slug.clone()
        } else {
            format!("group_{}", sanitize_model_slug(&group_item.title))
        };
        let slug = if !group_item.excerpt.trim().is_empty() {
            sanitize_model_slug(&group_item.excerpt)
        } else {
            sanitize_model_slug(&group_item.title)
        };

        let mut location_rules = Vec::new();
        if let Some(loc_outer) = cfg.get("location").and_then(|v| v.as_array()) {
            for or_group in loc_outer {
                if let Some(and_group) = or_group.as_array() {
                    for rule in and_group {
                        let param = rule
                            .get("param")
                            .and_then(|v| v.as_str())
                            .unwrap_or("post_type")
                            .to_string();
                        let operator = rule
                            .get("operator")
                            .and_then(|v| v.as_str())
                            .unwrap_or("==")
                            .to_string();
                        let value = rule
                            .get("value")
                            .and_then(|v| v.as_str())
                            .unwrap_or("post")
                            .to_string();
                        location_rules.push(FieldGroupLocationRule {
                            param,
                            operator,
                            value,
                        });
                    }
                }
            }
        }

        let top_fields = fields_by_parent
            .get(&group_item.id)
            .cloned()
            .unwrap_or_default();
        let mut parsed_fields = Vec::new();
        for f_item in top_fields {
            parsed_fields.push(convert_acf_field_item(f_item, &fields_by_parent));
        }

        // Also support inline `"fields"` array inside `acf-field-group` JSON content
        if parsed_fields.is_empty() {
            if let Some(inline_fields) = cfg.get("fields").and_then(|v| v.as_array()) {
                for inf in inline_fields {
                    if let Ok(schema) = serde_json::from_value::<StructuredFieldSchema>(inf.clone())
                    {
                        parsed_fields.push(schema);
                    }
                }
            }
        }

        let group_def = FieldGroupDefinition {
            id: 0,
            group_key,
            title: if group_item.title.trim().is_empty() {
                "Imported Field Group".to_string()
            } else {
                group_item.title.trim().to_string()
            },
            slug,
            description: cfg
                .get("description")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            location_rules,
            fields: parsed_fields,
            position: cfg
                .get("position")
                .and_then(|v| v.as_str())
                .unwrap_or("normal")
                .to_string(),
            menu_order: group_item.menu_order,
            is_active: true,
        };
        upsert_field_group_tx(tx, &group_def)?;
        total_ingested += 1;
    }

    Ok(total_ingested)
}

fn convert_acf_field_item(
    item: &RawAcfItem,
    fields_by_parent: &HashMap<i64, Vec<&RawAcfItem>>,
) -> StructuredFieldSchema {
    let cfg = parse_serialized_or_json(&item.content).unwrap_or_else(|| serde_json::json!({}));
    let raw_type = cfg
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("text")
        .to_ascii_lowercase();

    let field_type = match raw_type.as_str() {
        "image" | "file" | "gallery" | "media" => "media",
        "wysiwyg" => "wysiwyg",
        "textarea" => "textarea",
        "number" | "range" => "number",
        "date_picker" | "date_time_picker" | "date" => "date",
        "select" | "radio" | "button_group" => "select",
        "checkbox" => "checkbox",
        "true_false" | "boolean" => "true_false",
        "url" | "link" => "url",
        "repeater" | "flexible_content" => "repeater",
        "post_object" | "relationship" | "page_link" => "post_object",
        _ => "text",
    }
    .to_string();

    let name = if !item.excerpt.trim().is_empty() {
        sanitize_model_slug(&item.excerpt)
    } else if let Some(n) = cfg.get("name").and_then(|v| v.as_str()) {
        sanitize_model_slug(n)
    } else {
        sanitize_model_slug(&item.title)
    };

    let key = if item.slug.starts_with("field_") {
        item.slug.clone()
    } else {
        format!("field_{name}")
    };

    let mut choices = Vec::new();
    if let Some(ch_obj) = cfg.get("choices").and_then(|v| v.as_object()) {
        for (k, v) in ch_obj {
            if let Some(vs) = v.as_str() {
                if k == vs {
                    choices.push(vs.to_string());
                } else {
                    choices.push(format!("{k} : {vs}"));
                }
            }
        }
    } else if let Some(ch_arr) = cfg.get("choices").and_then(|v| v.as_array()) {
        for v in ch_arr {
            if let Some(vs) = v.as_str() {
                choices.push(vs.to_string());
            }
        }
    }

    let target_post_type = cfg.get("post_type").and_then(|v| {
        if let Some(s) = v.as_str() {
            Some(s.to_string()).filter(|x| !x.is_empty())
        } else if let Some(arr) = v.as_array() {
            arr.first()
                .and_then(|x| x.as_str())
                .map(String::from)
                .filter(|x| !x.is_empty())
        } else {
            None
        }
    });

    let mut sub_fields = Vec::new();
    if item.id > 0 {
        if let Some(children) = fields_by_parent.get(&item.id) {
            for child in children {
                sub_fields.push(convert_acf_field_item(child, fields_by_parent));
            }
        }
    }

    StructuredFieldSchema {
        key,
        name: name.clone(),
        label: if item.title.trim().is_empty() {
            humanize_slug(&name)
        } else {
            item.title.trim().to_string()
        },
        field_type: field_type.clone(),
        instructions: cfg
            .get("instructions")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        required: cfg
            .get("required")
            .and_then(|v| v.as_bool().or_else(|| v.as_i64().map(|n| n != 0)))
            .unwrap_or(false),
        default_value: cfg
            .get("default_value")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        placeholder: cfg
            .get("placeholder")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        choices,
        target_post_type,
        sub_fields,
        show_in_collection_card: matches!(field_type.as_str(), "text" | "select" | "date"),
    }
}

fn ingest_cptui_options(
    tx: &Transaction,
    plugin_options: &HashMap<String, Value>,
) -> Result<usize> {
    let mut count = 0usize;

    if let Some(raw_cpts) = plugin_options.get("cptui_post_types") {
        let parsed = match raw_cpts {
            Value::String(s) => parse_serialized_or_json(s),
            other => Some(other.clone()),
        };
        if let Some(Value::Object(map)) = parsed {
            for (key, val) in map {
                let slug = sanitize_model_slug(
                    val.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(key.as_str()),
                );
                if slug.is_empty() {
                    continue;
                }
                let label_plural = val
                    .get("label")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.trim().is_empty())
                    .map(String::from)
                    .unwrap_or_else(|| pluralize_label(&humanize_slug(&slug)));
                let label_singular = val
                    .get("singular_label")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.trim().is_empty())
                    .map(String::from)
                    .unwrap_or_else(|| humanize_slug(&slug));
                let taxonomies: Vec<String> = val
                    .get("taxonomies")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str().map(sanitize_model_slug))
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();

                let def = CustomPostTypeDefinition {
                    slug: slug.clone(),
                    label_singular,
                    label_plural,
                    description: val
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    icon: default_icon_for_cpt(&slug).to_string(),
                    menu_position: 25,
                    public: true,
                    has_archive: true,
                    hierarchical: false,
                    rewrite_slug: slug,
                    supports: vec![
                        "title".into(),
                        "editor".into(),
                        "thumbnail".into(),
                        "excerpt".into(),
                        "custom-fields".into(),
                    ],
                    taxonomies,
                    show_in_rest: true,
                    preset_key: None,
                    post_count: 0,
                };
                upsert_cpt_tx(tx, &def)?;
                count += 1;
            }
        }
    }

    if let Some(raw_taxes) = plugin_options.get("cptui_taxonomies") {
        let parsed = match raw_taxes {
            Value::String(s) => parse_serialized_or_json(s),
            other => Some(other.clone()),
        };
        if let Some(Value::Object(map)) = parsed {
            for (key, val) in map {
                let slug = sanitize_model_slug(
                    val.get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(key.as_str()),
                );
                if slug.is_empty() {
                    continue;
                }
                let label_plural = val
                    .get("label")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.trim().is_empty())
                    .map(String::from)
                    .unwrap_or_else(|| pluralize_label(&humanize_slug(&slug)));
                let label_singular = val
                    .get("singular_label")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.trim().is_empty())
                    .map(String::from)
                    .unwrap_or_else(|| humanize_slug(&slug));
                let post_types: Vec<String> = val
                    .get("object_types")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str().map(sanitize_model_slug))
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_default();
                let hierarchical = val
                    .get("hierarchical")
                    .and_then(|v| {
                        v.as_bool()
                            .or_else(|| v.as_str().map(|s| s == "true" || s == "1"))
                    })
                    .unwrap_or(true);

                let tax_def = CustomTaxonomyDefinition {
                    slug: slug.clone(),
                    label_singular,
                    label_plural,
                    description: String::new(),
                    hierarchical,
                    public: true,
                    rewrite_slug: slug,
                    post_types,
                    show_in_rest: true,
                    term_count: 0,
                };
                upsert_taxonomy_tx(tx, &tax_def)?;
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Finalizes CPT, Custom Taxonomy, and ACF Field Group registration after Pass 1 post ingestion:
/// 1. Auto-registers any custom `post_type` present in `posts` into `daypress_post_types`.
/// 2. Auto-registers any custom `taxonomy` present in `term_taxonomy` into `daypress_taxonomies`.
/// 3. Auto-synthesizes and saves `daypress_field_groups` for custom post types that have custom postmeta.
/// 4. Remaps `old_post_id -> new_post_id` for `media` and `post_object` structured fields.
pub fn finalize_acf_and_cpt_migration(
    tx: &Transaction,
    old_to_new_post_id: &HashMap<i64, i64>,
) -> Result<()> {
    // 1. Discover and persist any custom post types from `posts`
    let defs = crate::adapters::database::Database::list_post_type_definitions_with_conn(tx)?;
    for def in &defs {
        upsert_cpt_tx(tx, def)?;
    }

    // 2. Auto-synthesize field groups for each CPT if none exist yet
    for def in &defs {
        let existing =
            crate::adapters::database::Database::list_field_groups_with_conn(tx, Some(&def.slug))?;
        if let Some(first) = existing.first() {
            if first.id == 0 {
                upsert_field_group_tx(tx, first)?;
            }
        }
    }

    // 3. Remap `old_post_id -> new_post_id` for `media` and `post_object` fields in `postmeta`
    if !old_to_new_post_id.is_empty() {
        let all_groups =
            crate::adapters::database::Database::list_field_groups_with_conn(tx, None)?;
        let mut relational_field_names = std::collections::HashSet::new();
        for g in all_groups {
            for f in g.fields {
                if matches!(
                    f.field_type.as_str(),
                    "media" | "image" | "file" | "post_object" | "relationship"
                ) {
                    relational_field_names.insert(f.name);
                }
            }
        }

        for field_name in relational_field_names {
            let rows: Vec<(i64, String)> = tx
                .prepare(
                    "SELECT meta_id, COALESCE(meta_value, '') FROM postmeta WHERE meta_key = ?1",
                )
                .and_then(|mut stmt| {
                    let mapped =
                        stmt.query_map(params![&field_name], |r| Ok((r.get(0)?, r.get(1)?)))?;
                    Ok(mapped.filter_map(|r| r.ok()).collect())
                })
                .unwrap_or_default();

            for (meta_id, raw_val) in rows {
                if let Ok(old_id) = raw_val.trim().parse::<i64>() {
                    if let Some(&new_id) = old_to_new_post_id.get(&old_id) {
                        if new_id != old_id {
                            let _ = tx.execute(
                                "UPDATE postmeta SET meta_value = ?1 WHERE meta_id = ?2",
                                params![new_id.to_string(), meta_id],
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn upsert_cpt_tx(tx: &Transaction, def: &CustomPostTypeDefinition) -> Result<()> {
    let supports_json = serde_json::to_string(&def.supports).unwrap_or_else(|_| "[]".to_string());
    let taxonomies_json =
        serde_json::to_string(&def.taxonomies).unwrap_or_else(|_| "[]".to_string());
    tx.execute(
        r#"INSERT INTO daypress_post_types (
               slug, label_singular, label_plural, description, icon, menu_position,
               public, has_archive, hierarchical, rewrite_slug, supports_json,
               taxonomies_json, show_in_rest, preset_key, updated_at
           ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, datetime('now'))
           ON CONFLICT(slug) DO UPDATE SET
               label_singular = excluded.label_singular,
               label_plural = excluded.label_plural,
               taxonomies_json = CASE WHEN excluded.taxonomies_json != '[]' THEN excluded.taxonomies_json ELSE daypress_post_types.taxonomies_json END,
               updated_at = datetime('now')"#,
        params![
            &def.slug,
            &def.label_singular,
            &def.label_plural,
            &def.description,
            &def.icon,
            def.menu_position,
            if def.public { 1i64 } else { 0i64 },
            if def.has_archive { 1i64 } else { 0i64 },
            if def.hierarchical { 1i64 } else { 0i64 },
            if def.rewrite_slug.is_empty() {
                &def.slug
            } else {
                &def.rewrite_slug
            },
            &supports_json,
            &taxonomies_json,
            if def.show_in_rest { 1i64 } else { 0i64 },
            &def.preset_key,
        ],
    )?;
    Ok(())
}

fn upsert_taxonomy_tx(tx: &Transaction, def: &CustomTaxonomyDefinition) -> Result<()> {
    let post_types_json =
        serde_json::to_string(&def.post_types).unwrap_or_else(|_| "[]".to_string());
    tx.execute(
        r#"INSERT INTO daypress_taxonomies (
               slug, label_singular, label_plural, description, hierarchical,
               public, rewrite_slug, post_types_json, show_in_rest, updated_at
           ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'))
           ON CONFLICT(slug) DO UPDATE SET
               label_singular = excluded.label_singular,
               label_plural = excluded.label_plural,
               post_types_json = CASE WHEN excluded.post_types_json != '[]' THEN excluded.post_types_json ELSE daypress_taxonomies.post_types_json END,
               updated_at = datetime('now')"#,
        params![
            &def.slug,
            &def.label_singular,
            &def.label_plural,
            &def.description,
            if def.hierarchical { 1i64 } else { 0i64 },
            if def.public { 1i64 } else { 0i64 },
            if def.rewrite_slug.is_empty() {
                &def.slug
            } else {
                &def.rewrite_slug
            },
            &post_types_json,
            if def.show_in_rest { 1i64 } else { 0i64 },
        ],
    )?;
    Ok(())
}

fn upsert_field_group_tx(tx: &Transaction, group: &FieldGroupDefinition) -> Result<()> {
    let loc_json =
        serde_json::to_string(&group.location_rules).unwrap_or_else(|_| "[]".to_string());
    let fields_json = serde_json::to_string(&group.fields).unwrap_or_else(|_| "[]".to_string());
    tx.execute(
        r#"INSERT INTO daypress_field_groups (
               group_key, title, slug, description, location_rules_json,
               fields_json, position, menu_order, is_active, updated_at
           ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'))
           ON CONFLICT(group_key) DO UPDATE SET
               title = excluded.title,
               location_rules_json = excluded.location_rules_json,
               fields_json = excluded.fields_json,
               updated_at = datetime('now')"#,
        params![
            &group.group_key,
            &group.title,
            &group.slug,
            &group.description,
            &loc_json,
            &fields_json,
            &group.position,
            group.menu_order,
            if group.is_active { 1i64 } else { 0i64 },
        ],
    )?;
    Ok(())
}
