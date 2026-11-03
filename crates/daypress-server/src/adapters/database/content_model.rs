//! Content Model Repository (`adapters/database/content_model.rs`)
//!
//! Manages Custom Post Types (`daypress_post_types`), Custom Taxonomies (`daypress_taxonomies`),
//! and Structured Field Groups (`daypress_field_groups` — ACF Parity), including 1-click
//! Starter Presets and automatic schema synthesis from imported WordPress postmeta.

use super::Database;
use crate::models::{
    CustomPostTypeDefinition, CustomTaxonomyDefinition, FieldGroupDefinition,
    FieldGroupLocationRule, StructuredFieldSchema,
};
use rusqlite::{params, Connection, Result};

const INTERNAL_POST_TYPES: &[&str] = &[
    "post",
    "page",
    "attachment",
    "revision",
    "nav_menu_item",
    "wp_global_styles",
    "custom_css",
    "customize_changeset",
    "oembed_cache",
    "acf-field-group",
    "acf-field",
    "acf-post-type",
    "acf-taxonomy",
    "wpcf7_contact_form",
    "wpforms",
    "elementor_library",
    "wp_block",
    "wp_template",
    "wp_template_part",
    "wp_navigation",
];

pub fn humanize_slug(slug: &str) -> String {
    slug.replace(['_', '-'], " ")
        .split_whitespace()
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn pluralize_label(singular: &str) -> String {
    if singular.eq_ignore_ascii_case("portfolio")
        || singular.eq_ignore_ascii_case("media")
        || singular.eq_ignore_ascii_case("software")
        || singular.eq_ignore_ascii_case("press")
        || singular.ends_with('s')
        || singular.ends_with("en")
    {
        singular.to_string()
    } else if let Some(stem) = singular.strip_suffix('y') {
        if !singular.ends_with("ay")
            && !singular.ends_with("ey")
            && !singular.ends_with("oy")
            && !singular.ends_with("uy")
        {
            format!("{stem}ies")
        } else {
            format!("{singular}s")
        }
    } else {
        format!("{singular}s")
    }
}

pub fn sanitize_model_slug(raw: &str) -> String {
    let mut out = String::new();
    for ch in raw.trim().chars() {
        if ch.is_ascii_alphanumeric() {
            out.push(ch.to_ascii_lowercase());
        } else if (ch == '_' || ch == '-' || ch == ' ') && !out.ends_with('_') {
            out.push('_');
        }
    }
    out.trim_matches('_').chars().take(32).collect()
}

pub fn default_icon_for_cpt(slug: &str) -> &'static str {
    match slug {
        "case_study" | "case_studies" | "cases" => "fa-solid fa-briefcase",
        "team" | "team_member" | "kandidaten" => "fa-solid fa-user-group",
        "doc" | "docs" | "documentation" | "knowledge_base" => "fa-solid fa-book-open",
        "job" | "jobs" | "job_listing" | "vacatures" => "fa-solid fa-id-badge",
        "event" | "events" | "trainingen" => "fa-solid fa-calendar-days",
        "portfolio" | "project" | "projects" => "fa-solid fa-layer-group",
        "testimonial" | "testimonials" | "review" | "reviews" => "fa-solid fa-quote-left",
        _ => "fa-solid fa-folder-open",
    }
}

impl Database {
    /// Lists all registered and auto-discovered Custom Post Type definitions with live post counts.
    pub fn list_post_type_definitions(&self) -> Result<Vec<CustomPostTypeDefinition>> {
        let conn = self.conn.lock().unwrap();
        Self::list_post_type_definitions_with_conn(&conn)
    }

    pub(crate) fn list_post_type_definitions_with_conn(
        conn: &Connection,
    ) -> Result<Vec<CustomPostTypeDefinition>> {
        let mut defs: Vec<CustomPostTypeDefinition> = Vec::new();
        let mut seen_slugs = std::collections::HashSet::new();

        if let Ok(mut stmt) = conn.prepare(
            "SELECT slug, label_singular, label_plural, description, icon, menu_position,
                    public, has_archive, hierarchical, rewrite_slug, supports_json,
                    taxonomies_json, show_in_rest, preset_key
             FROM daypress_post_types
             ORDER BY menu_position ASC, slug ASC",
        ) {
            if let Ok(rows) = stmt.query_map([], |r| {
                let slug: String = r.get(0)?;
                let supports_raw: String = r.get(10)?;
                let tax_raw: String = r.get(11)?;
                let rewrite_raw: String = r.get(9)?;
                Ok(CustomPostTypeDefinition {
                    rewrite_slug: if rewrite_raw.trim().is_empty() {
                        slug.clone()
                    } else {
                        rewrite_raw
                    },
                    slug,
                    label_singular: r.get(1)?,
                    label_plural: r.get(2)?,
                    description: r.get(3)?,
                    icon: r.get(4)?,
                    menu_position: r.get(5)?,
                    public: r.get::<_, i64>(6)? != 0,
                    has_archive: r.get::<_, i64>(7)? != 0,
                    hierarchical: r.get::<_, i64>(8)? != 0,
                    supports: serde_json::from_str(&supports_raw).unwrap_or_else(|_| {
                        vec![
                            "title".to_string(),
                            "editor".to_string(),
                            "thumbnail".to_string(),
                            "excerpt".to_string(),
                            "custom-fields".to_string(),
                        ]
                    }),
                    taxonomies: serde_json::from_str(&tax_raw).unwrap_or_default(),
                    show_in_rest: r.get::<_, i64>(12)? != 0,
                    preset_key: r.get(13)?,
                    post_count: 0,
                })
            }) {
                for row in rows.flatten() {
                    seen_slugs.insert(row.slug.clone());
                    defs.push(row);
                }
            }
        }

        // Also auto-discover any custom post types stored in `posts` table
        if let Ok(mut stmt) = conn.prepare(
            "SELECT DISTINCT post_type FROM posts
             WHERE post_type IS NOT NULL AND post_type != ''
             ORDER BY post_type ASC",
        ) {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                for ptype in rows.flatten() {
                    if INTERNAL_POST_TYPES.contains(&ptype.as_str()) || seen_slugs.contains(&ptype)
                    {
                        continue;
                    }
                    seen_slugs.insert(ptype.clone());
                    let singular = humanize_slug(&ptype);
                    let plural = pluralize_label(&singular);
                    defs.push(CustomPostTypeDefinition {
                        slug: ptype.clone(),
                        label_singular: singular,
                        label_plural: plural,
                        description: format!("Auto-discovered custom post type ({ptype})."),
                        icon: default_icon_for_cpt(&ptype).to_string(),
                        menu_position: 25,
                        public: true,
                        has_archive: true,
                        hierarchical: false,
                        rewrite_slug: ptype,
                        supports: vec![
                            "title".to_string(),
                            "editor".to_string(),
                            "thumbnail".to_string(),
                            "excerpt".to_string(),
                            "custom-fields".to_string(),
                        ],
                        taxonomies: Vec::new(),
                        show_in_rest: true,
                        preset_key: None,
                        post_count: 0,
                    });
                }
            }
        }

        // Populate live post counts and inferred taxonomies
        for def in &mut defs {
            def.post_count = conn
                .query_row(
                    "SELECT COUNT(*) FROM posts WHERE post_type = ?1 AND post_status != 'trash' AND post_status != 'auto-draft'",
                    params![&def.slug],
                    |r| r.get(0),
                )
                .unwrap_or(0);

            if let Ok(mut tstmt) = conn
                .prepare("SELECT slug, post_types_json FROM daypress_taxonomies ORDER BY slug ASC")
            {
                if let Ok(trows) =
                    tstmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                {
                    for (tslug, ptypes_json) in trows.flatten() {
                        let ptypes: Vec<String> =
                            serde_json::from_str(&ptypes_json).unwrap_or_default();
                        if ptypes.contains(&def.slug) && !def.taxonomies.contains(&tslug) {
                            def.taxonomies.push(tslug);
                        }
                    }
                }
            }
        }

        Ok(defs)
    }

    pub fn get_post_type_definition(&self, slug: &str) -> Result<Option<CustomPostTypeDefinition>> {
        let defs = self.list_post_type_definitions()?;
        Ok(defs
            .into_iter()
            .find(|d| d.slug == slug || d.rewrite_slug == slug))
    }

    pub fn save_post_type_definition(
        &self,
        def: &CustomPostTypeDefinition,
    ) -> Result<CustomPostTypeDefinition> {
        let slug = sanitize_model_slug(&def.slug);
        if slug.is_empty() || INTERNAL_POST_TYPES.contains(&slug.as_str()) {
            return Err(rusqlite::Error::InvalidParameterName(format!(
                "Invalid or reserved post type slug: {}",
                def.slug
            )));
        }
        let label_singular = if def.label_singular.trim().is_empty() {
            humanize_slug(&slug)
        } else {
            def.label_singular.trim().to_string()
        };
        let label_plural = if def.label_plural.trim().is_empty() {
            pluralize_label(&label_singular)
        } else {
            def.label_plural.trim().to_string()
        };
        let icon = if def.icon.trim().is_empty() {
            default_icon_for_cpt(&slug).to_string()
        } else {
            def.icon.trim().to_string()
        };
        let rewrite_slug = if def.rewrite_slug.trim().is_empty() {
            slug.clone()
        } else {
            sanitize_model_slug(&def.rewrite_slug)
        };
        let supports = if def.supports.is_empty() {
            vec![
                "title".to_string(),
                "editor".to_string(),
                "thumbnail".to_string(),
                "excerpt".to_string(),
                "custom-fields".to_string(),
            ]
        } else {
            def.supports.clone()
        };
        let supports_json = serde_json::to_string(&supports).unwrap_or_else(|_| "[]".to_string());
        let taxonomies_json =
            serde_json::to_string(&def.taxonomies).unwrap_or_else(|_| "[]".to_string());

        {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                r#"INSERT INTO daypress_post_types (
                       slug, label_singular, label_plural, description, icon, menu_position,
                       public, has_archive, hierarchical, rewrite_slug, supports_json,
                       taxonomies_json, show_in_rest, preset_key, updated_at
                   ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, datetime('now'))
                   ON CONFLICT(slug) DO UPDATE SET
                       label_singular = excluded.label_singular,
                       label_plural = excluded.label_plural,
                       description = excluded.description,
                       icon = excluded.icon,
                       menu_position = excluded.menu_position,
                       public = excluded.public,
                       has_archive = excluded.has_archive,
                       hierarchical = excluded.hierarchical,
                       rewrite_slug = excluded.rewrite_slug,
                       supports_json = excluded.supports_json,
                       taxonomies_json = excluded.taxonomies_json,
                       show_in_rest = excluded.show_in_rest,
                       preset_key = COALESCE(excluded.preset_key, daypress_post_types.preset_key),
                       updated_at = datetime('now')"#,
                params![
                    &slug,
                    &label_singular,
                    &label_plural,
                    def.description.trim(),
                    &icon,
                    def.menu_position,
                    if def.public { 1i64 } else { 0i64 },
                    if def.has_archive { 1i64 } else { 0i64 },
                    if def.hierarchical { 1i64 } else { 0i64 },
                    &rewrite_slug,
                    &supports_json,
                    &taxonomies_json,
                    if def.show_in_rest { 1i64 } else { 0i64 },
                    &def.preset_key,
                ],
            )?;
        }
        self.invalidate_hot_cache("cpt_definition_saved");
        self.get_post_type_definition(&slug)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn delete_post_type_definition(&self, slug: &str) -> Result<bool> {
        let affected = {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "DELETE FROM daypress_post_types WHERE slug = ?1",
                params![slug.trim()],
            )?
        };
        if affected > 0 {
            self.invalidate_hot_cache("cpt_definition_deleted");
        }
        Ok(affected > 0)
    }

    /// Lists all registered and auto-discovered Custom Taxonomy definitions with term counts.
    pub fn list_custom_taxonomy_definitions(&self) -> Result<Vec<CustomTaxonomyDefinition>> {
        let conn = self.conn.lock().unwrap();
        let mut defs: Vec<CustomTaxonomyDefinition> = Vec::new();
        let mut seen = std::collections::HashSet::new();

        if let Ok(mut stmt) = conn.prepare(
            "SELECT slug, label_singular, label_plural, description, hierarchical,
                    public, rewrite_slug, post_types_json, show_in_rest
             FROM daypress_taxonomies
             ORDER BY slug ASC",
        ) {
            if let Ok(rows) = stmt.query_map([], |r| {
                let slug: String = r.get(0)?;
                let rewrite_raw: String = r.get(6)?;
                let pt_raw: String = r.get(7)?;
                Ok(CustomTaxonomyDefinition {
                    rewrite_slug: if rewrite_raw.trim().is_empty() {
                        slug.clone()
                    } else {
                        rewrite_raw
                    },
                    slug,
                    label_singular: r.get(1)?,
                    label_plural: r.get(2)?,
                    description: r.get(3)?,
                    hierarchical: r.get::<_, i64>(4)? != 0,
                    public: r.get::<_, i64>(5)? != 0,
                    post_types: serde_json::from_str(&pt_raw).unwrap_or_default(),
                    show_in_rest: r.get::<_, i64>(8)? != 0,
                    term_count: 0,
                })
            }) {
                for row in rows.flatten() {
                    seen.insert(row.slug.clone());
                    defs.push(row);
                }
            }
        }

        // Auto-discover custom taxonomies from `term_taxonomy`
        if let Ok(mut stmt) = conn.prepare(
            "SELECT DISTINCT taxonomy FROM term_taxonomy
             WHERE taxonomy NOT IN ('category', 'post_tag', 'nav_menu', 'link_category', 'post_format')
               AND taxonomy IS NOT NULL AND taxonomy != ''
             ORDER BY taxonomy ASC",
        ) {
            if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                for tax_slug in rows.flatten() {
                    if seen.contains(&tax_slug) {
                        continue;
                    }
                    seen.insert(tax_slug.clone());
                    let singular = humanize_slug(&tax_slug);
                    let plural = pluralize_label(&singular);
                    let hierarchical =
                        !(tax_slug.ends_with("_tag") || tax_slug.ends_with("_tags"));

                    // Discover linked post_types from term_relationships
                    let mut post_types = Vec::new();
                    if let Ok(mut pt_stmt) = conn.prepare(
                        "SELECT DISTINCT p.post_type FROM term_taxonomy tt
                         JOIN term_relationships r ON tt.term_taxonomy_id = r.term_taxonomy_id
                         JOIN posts p ON r.object_id = p.ID
                         WHERE tt.taxonomy = ?1",
                    ) {
                        if let Ok(pt_rows) = pt_stmt.query_map(params![&tax_slug], |r| r.get(0)) {
                            for pt in pt_rows.flatten() {
                                post_types.push(pt);
                            }
                        }
                    }

                    defs.push(CustomTaxonomyDefinition {
                        slug: tax_slug.clone(),
                        label_singular: singular,
                        label_plural: plural,
                        description: format!("Custom taxonomy ({tax_slug})."),
                        hierarchical,
                        public: true,
                        rewrite_slug: tax_slug,
                        post_types,
                        show_in_rest: true,
                        term_count: 0,
                    });
                }
            }
        }

        for def in &mut defs {
            def.term_count = conn
                .query_row(
                    "SELECT COUNT(*) FROM term_taxonomy WHERE taxonomy = ?1",
                    params![&def.slug],
                    |r| r.get(0),
                )
                .unwrap_or(0);
        }

        Ok(defs)
    }

    pub fn save_custom_taxonomy_definition(
        &self,
        def: &CustomTaxonomyDefinition,
    ) -> Result<CustomTaxonomyDefinition> {
        let slug = sanitize_model_slug(&def.slug);
        if slug.is_empty()
            || matches!(
                slug.as_str(),
                "category" | "post_tag" | "nav_menu" | "link_category" | "post_format"
            )
        {
            return Err(rusqlite::Error::InvalidParameterName(format!(
                "Invalid or reserved taxonomy slug: {}",
                def.slug
            )));
        }
        let label_singular = if def.label_singular.trim().is_empty() {
            humanize_slug(&slug)
        } else {
            def.label_singular.trim().to_string()
        };
        let label_plural = if def.label_plural.trim().is_empty() {
            pluralize_label(&label_singular)
        } else {
            def.label_plural.trim().to_string()
        };
        let rewrite_slug = if def.rewrite_slug.trim().is_empty() {
            slug.clone()
        } else {
            sanitize_model_slug(&def.rewrite_slug)
        };
        let post_types_json =
            serde_json::to_string(&def.post_types).unwrap_or_else(|_| "[]".to_string());

        {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                r#"INSERT INTO daypress_taxonomies (
                       slug, label_singular, label_plural, description, hierarchical,
                       public, rewrite_slug, post_types_json, show_in_rest, updated_at
                   ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'))
                   ON CONFLICT(slug) DO UPDATE SET
                       label_singular = excluded.label_singular,
                       label_plural = excluded.label_plural,
                       description = excluded.description,
                       hierarchical = excluded.hierarchical,
                       public = excluded.public,
                       rewrite_slug = excluded.rewrite_slug,
                       post_types_json = excluded.post_types_json,
                       show_in_rest = excluded.show_in_rest,
                       updated_at = datetime('now')"#,
                params![
                    &slug,
                    &label_singular,
                    &label_plural,
                    def.description.trim(),
                    if def.hierarchical { 1i64 } else { 0i64 },
                    if def.public { 1i64 } else { 0i64 },
                    &rewrite_slug,
                    &post_types_json,
                    if def.show_in_rest { 1i64 } else { 0i64 },
                ],
            )?;
        }
        self.invalidate_hot_cache("custom_taxonomy_saved");
        let all = self.list_custom_taxonomy_definitions()?;
        all.into_iter()
            .find(|t| t.slug == slug)
            .ok_or(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn delete_custom_taxonomy_definition(&self, slug: &str) -> Result<bool> {
        let affected = {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "DELETE FROM daypress_taxonomies WHERE slug = ?1",
                params![slug.trim()],
            )?
        };
        if affected > 0 {
            self.invalidate_hot_cache("custom_taxonomy_deleted");
        }
        Ok(affected > 0)
    }

    /// Lists Structured Field Groups (`daypress_field_groups`).
    /// When `post_type_filter` is `Some(pt)`, returns only groups matching `pt`
    /// and auto-synthesizes a schema if posts of `pt` have custom postmeta not yet backed by a group.
    pub fn list_field_groups(
        &self,
        post_type_filter: Option<&str>,
    ) -> Result<Vec<FieldGroupDefinition>> {
        let conn = self.conn.lock().unwrap();
        Self::list_field_groups_with_conn(&conn, post_type_filter)
    }

    pub(crate) fn list_field_groups_with_conn(
        conn: &Connection,
        post_type_filter: Option<&str>,
    ) -> Result<Vec<FieldGroupDefinition>> {
        let mut groups: Vec<FieldGroupDefinition> = Vec::new();

        if let Ok(mut stmt) = conn.prepare(
            "SELECT id, group_key, title, slug, description, location_rules_json,
                    fields_json, position, menu_order, is_active
             FROM daypress_field_groups
             ORDER BY menu_order ASC, id ASC",
        ) {
            if let Ok(rows) = stmt.query_map([], |r| {
                let loc_raw: String = r.get(5)?;
                let fields_raw: String = r.get(6)?;
                Ok(FieldGroupDefinition {
                    id: r.get(0)?,
                    group_key: r.get(1)?,
                    title: r.get(2)?,
                    slug: r.get(3)?,
                    description: r.get(4)?,
                    location_rules: serde_json::from_str(&loc_raw).unwrap_or_default(),
                    fields: serde_json::from_str(&fields_raw).unwrap_or_default(),
                    position: r.get(7)?,
                    menu_order: r.get(8)?,
                    is_active: r.get::<_, i64>(9)? != 0,
                })
            }) {
                for row in rows.flatten() {
                    groups.push(row);
                }
            }
        }

        if let Some(pt) = post_type_filter {
            let matching: Vec<FieldGroupDefinition> = groups
                .into_iter()
                .filter(|g| g.is_active && Self::field_group_matches_post_type(g, pt))
                .collect();

            if !matching.is_empty() {
                return Ok(matching);
            }

            // Auto-synthesize a field group if this CPT has known or discovered custom meta fields
            if let Some(synthesized) = Self::synthesize_field_group_for_post_type(conn, pt) {
                return Ok(vec![synthesized]);
            }

            return Ok(Vec::new());
        }

        Ok(groups)
    }

    pub fn field_group_matches_post_type(group: &FieldGroupDefinition, post_type: &str) -> bool {
        if group.location_rules.is_empty() {
            return true;
        }
        for rule in &group.location_rules {
            if rule.param == "post_type" {
                if rule.operator == "==" && (rule.value == post_type || rule.value == "all") {
                    return true;
                }
                if rule.operator == "!=" && rule.value != post_type {
                    return true;
                }
            }
        }
        false
    }

    pub(crate) fn synthesize_field_group_for_post_type(
        conn: &Connection,
        post_type: &str,
    ) -> Option<FieldGroupDefinition> {
        // 1. Discover custom postmeta keys used on posts of this post_type
        let mut discovered_keys: Vec<(String, String)> = Vec::new();
        if let Ok(mut stmt) = conn.prepare(
            "SELECT pm.meta_key, COALESCE(MAX(pm.meta_value), '')
             FROM postmeta pm
             JOIN posts p ON pm.post_id = p.ID
             WHERE p.post_type = ?1
               AND pm.meta_key NOT LIKE '\\_%' ESCAPE '\\'
               AND pm.meta_key NOT IN ('_thumbnail_id', '_wp_page_template', '_elementor_data', '_daypress_layout', '_edit_lock', '_edit_last')
             GROUP BY pm.meta_key
             ORDER BY pm.meta_key ASC",
        ) {
            if let Ok(rows) = stmt.query_map(params![post_type], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
            }) {
                for (k, v) in rows.flatten() {
                    if !k.starts_with('_') && !v.starts_with("field_") {
                        discovered_keys.push((k, v));
                    }
                }
            }
        }

        // Detect ACF flattened repeater sub-keys (e.g. `programma_0_titel`) so we collapse them into a single `repeater` field
        let mut repeater_subfields: std::collections::BTreeMap<String, Vec<String>> =
            std::collections::BTreeMap::new();
        for (k, _) in &discovered_keys {
            let parts: Vec<&str> = k.splitn(3, '_').collect();
            if parts.len() == 3 && parts[1].chars().all(|c| c.is_ascii_digit()) {
                let rep_name = parts[0].to_string();
                let sub_name = parts[2].to_string();
                let entry = repeater_subfields.entry(rep_name).or_default();
                if !entry.contains(&sub_name) {
                    entry.push(sub_name);
                }
            }
        }

        let mut fields: Vec<StructuredFieldSchema> = Vec::new();
        let mut added_names = std::collections::HashSet::new();

        // Standard baseline schemas for legacy imported CPTs when present
        let legacy_defaults: Vec<(&str, &str, &str, bool)> = match post_type {
            "trainingen" => vec![
                ("samenvatting", "Summary", "textarea", false),
                ("duur", "Duration", "text", true),
                ("investering", "Investment / Price", "text", true),
                ("locatie", "Location", "text", true),
                ("niveau", "Difficulty Level", "text", true),
                ("certificaat", "Certificate", "text", false),
                ("doelgroep", "Target Audience", "textarea", false),
                ("resultaat", "Learning Outcomes", "textarea", false),
            ],
            "vacatures" => vec![
                ("samenvatting", "Summary", "textarea", false),
                ("locatie", "Location", "text", true),
                ("uren", "Hours per Week", "text", true),
                ("salaris", "Salary Range", "text", true),
                ("opleidingsniveau", "Education Level", "text", false),
                ("dienstverband", "Contract Type", "text", true),
                ("ervaring", "Experience", "text", false),
                ("wat_vragen_wij", "Requirements", "textarea", false),
                ("wat_bieden_wij", "Benefits", "textarea", false),
            ],
            "cases" => vec![
                ("samenvatting", "Summary", "textarea", false),
                ("klant", "Client / Organization", "text", true),
                ("sector", "Industry / Sector", "text", true),
                ("doorlooptijd", "Timeline", "text", true),
                ("uitdaging", "Challenge", "textarea", false),
                ("oplossing", "Solution", "textarea", false),
                ("resultaat", "Measurable Results", "textarea", false),
                ("quote", "Client Quote", "textarea", false),
                ("quote_auteur", "Quote Author", "text", false),
            ],
            "kandidaten" => vec![
                ("samenvatting", "Profile Summary", "textarea", false),
                ("functie", "Role / Expertise", "text", true),
                ("beschikbaarheid", "Availability", "text", true),
                ("regio", "Region", "text", true),
                ("ervaring_jaren", "Years of Experience", "text", true),
                ("opleiding", "Education", "text", false),
                ("vaardigheden", "Core Skills", "textarea", false),
            ],
            _ => Vec::new(),
        };

        for (fname, flabel, ftype, show_card) in legacy_defaults {
            added_names.insert(fname.to_string());
            fields.push(StructuredFieldSchema {
                key: format!("field_{post_type}_{fname}"),
                name: fname.to_string(),
                label: flabel.to_string(),
                field_type: ftype.to_string(),
                instructions: String::new(),
                required: false,
                default_value: String::new(),
                placeholder: String::new(),
                choices: Vec::new(),
                target_post_type: None,
                sub_fields: Vec::new(),
                show_in_collection_card: show_card,
            });
        }

        for (rep_name, subs) in &repeater_subfields {
            if added_names.contains(rep_name) {
                continue;
            }
            added_names.insert(rep_name.clone());
            let sub_fields = subs
                .iter()
                .map(|s| StructuredFieldSchema {
                    key: format!("field_{rep_name}_{s}"),
                    name: s.clone(),
                    label: humanize_slug(s),
                    field_type: "text".to_string(),
                    ..Default::default()
                })
                .collect();
            fields.push(StructuredFieldSchema {
                key: format!("field_{post_type}_{rep_name}"),
                name: rep_name.clone(),
                label: humanize_slug(rep_name),
                field_type: "repeater".to_string(),
                sub_fields,
                ..Default::default()
            });
        }

        for (k, sample_val) in discovered_keys {
            // Skip flattened repeater sub-rows
            let parts: Vec<&str> = k.splitn(3, '_').collect();
            if parts.len() == 3 && parts[1].chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            if added_names.contains(&k) {
                continue;
            }
            added_names.insert(k.clone());

            let inferred_type = if sample_val.starts_with('[') && sample_val.ends_with(']') {
                "repeater"
            } else if sample_val.starts_with("http://") || sample_val.starts_with("https://") {
                "url"
            } else if sample_val.chars().count() > 140 || sample_val.contains('\n') {
                "textarea"
            } else {
                "text"
            };

            let show_in_card = fields.iter().filter(|f| f.show_in_collection_card).count() < 3
                && inferred_type == "text";

            fields.push(StructuredFieldSchema {
                key: format!("field_{post_type}_{k}"),
                name: k.clone(),
                label: humanize_slug(&k),
                field_type: inferred_type.to_string(),
                show_in_collection_card: show_in_card,
                ..Default::default()
            });
        }

        if fields.is_empty() {
            return None;
        }

        let pt_label = humanize_slug(post_type);
        Some(FieldGroupDefinition {
            id: 0,
            group_key: format!("group_auto_{post_type}"),
            title: format!("{pt_label} Structured Fields"),
            slug: format!("{post_type}_fields"),
            description: format!("Structured field schema for {pt_label}."),
            location_rules: vec![FieldGroupLocationRule {
                param: "post_type".to_string(),
                operator: "==".to_string(),
                value: post_type.to_string(),
            }],
            fields,
            position: "normal".to_string(),
            menu_order: 0,
            is_active: true,
        })
    }

    pub fn save_field_group(&self, group: &FieldGroupDefinition) -> Result<FieldGroupDefinition> {
        let slug = if group.slug.trim().is_empty() {
            sanitize_model_slug(&group.title)
        } else {
            sanitize_model_slug(&group.slug)
        };
        let group_key = if group.group_key.trim().is_empty() {
            format!("group_{slug}")
        } else {
            group.group_key.trim().to_string()
        };
        let title = if group.title.trim().is_empty() {
            humanize_slug(&slug)
        } else {
            group.title.trim().to_string()
        };

        // Ensure every field has a stable key and normalized name
        let mut normalized_fields = group.fields.clone();
        for f in &mut normalized_fields {
            f.name = sanitize_model_slug(&f.name);
            if f.label.trim().is_empty() {
                f.label = humanize_slug(&f.name);
            }
            if f.key.trim().is_empty() {
                f.key = format!("field_{}_{}", slug, f.name);
            }
            for sf in &mut f.sub_fields {
                sf.name = sanitize_model_slug(&sf.name);
                if sf.label.trim().is_empty() {
                    sf.label = humanize_slug(&sf.name);
                }
                if sf.key.trim().is_empty() {
                    sf.key = format!("field_{}_{}_{}", slug, f.name, sf.name);
                }
            }
        }

        let loc_json =
            serde_json::to_string(&group.location_rules).unwrap_or_else(|_| "[]".to_string());
        let fields_json =
            serde_json::to_string(&normalized_fields).unwrap_or_else(|_| "[]".to_string());
        let position = if group.position.trim().is_empty() {
            "normal".to_string()
        } else {
            group.position.trim().to_string()
        };

        {
            let conn = self.conn.lock().unwrap();
            if group.id > 0 {
                conn.execute(
                    r#"UPDATE daypress_field_groups SET
                           group_key = ?1, title = ?2, slug = ?3, description = ?4,
                           location_rules_json = ?5, fields_json = ?6, position = ?7,
                           menu_order = ?8, is_active = ?9, updated_at = datetime('now')
                       WHERE id = ?10"#,
                    params![
                        &group_key,
                        &title,
                        &slug,
                        group.description.trim(),
                        &loc_json,
                        &fields_json,
                        &position,
                        group.menu_order,
                        if group.is_active { 1i64 } else { 0i64 },
                        group.id,
                    ],
                )?;
            } else {
                conn.execute(
                    r#"INSERT INTO daypress_field_groups (
                           group_key, title, slug, description, location_rules_json,
                           fields_json, position, menu_order, is_active, updated_at
                       ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, datetime('now'))
                       ON CONFLICT(group_key) DO UPDATE SET
                           title = excluded.title,
                           slug = excluded.slug,
                           description = excluded.description,
                           location_rules_json = excluded.location_rules_json,
                           fields_json = excluded.fields_json,
                           position = excluded.position,
                           menu_order = excluded.menu_order,
                           is_active = excluded.is_active,
                           updated_at = datetime('now')"#,
                    params![
                        &group_key,
                        &title,
                        &slug,
                        group.description.trim(),
                        &loc_json,
                        &fields_json,
                        &position,
                        group.menu_order,
                        if group.is_active { 1i64 } else { 0i64 },
                    ],
                )?;
            }
        }
        self.invalidate_hot_cache("field_group_saved");
        let groups = self.list_field_groups(None)?;
        groups
            .into_iter()
            .find(|g| g.group_key == group_key)
            .ok_or(rusqlite::Error::QueryReturnedNoRows)
    }

    pub fn delete_field_group(&self, id_or_key: &str) -> Result<bool> {
        let affected = {
            let conn = self.conn.lock().unwrap();
            if let Ok(id) = id_or_key.trim().parse::<i64>() {
                conn.execute(
                    "DELETE FROM daypress_field_groups WHERE id = ?1",
                    params![id],
                )?
            } else {
                conn.execute(
                    "DELETE FROM daypress_field_groups WHERE group_key = ?1 OR slug = ?1",
                    params![id_or_key.trim()],
                )?
            }
        };
        if affected > 0 {
            self.invalidate_hot_cache("field_group_deleted");
        }
        Ok(affected > 0)
    }

    /// Resolves structured fields (`post.acf`) for a post, unpacking ACF flattened repeater
    /// rows (`{field}_{idx}_{sub}`), resolving media attachment IDs to URLs, and resolving
    /// relational `post_object` IDs to `{ id, title, slug, link, post_type }` summaries.
    pub(crate) fn resolve_post_structured_fields_with_conn(
        conn: &Connection,
        post_type: &str,
        raw_meta: &serde_json::Map<String, serde_json::Value>,
        base_url: &str,
    ) -> serde_json::Value {
        let mut acf_map = serde_json::Map::new();
        for (k, v) in raw_meta {
            if !k.starts_with('_') {
                acf_map.insert(k.clone(), v.clone());
            }
        }

        // Collapse ACF flattened repeaters `{rep}_{idx}_{sub}` -> `acf_map[rep] = [{ sub: val, ... }]`
        let mut repeater_buckets: std::collections::BTreeMap<
            String,
            std::collections::BTreeMap<usize, serde_json::Map<String, serde_json::Value>>,
        > = std::collections::BTreeMap::new();

        for (k, v) in raw_meta {
            if k.starts_with('_') {
                continue;
            }
            let parts: Vec<&str> = k.splitn(3, '_').collect();
            if parts.len() == 3 {
                if let Ok(row_idx) = parts[1].parse::<usize>() {
                    if row_idx < 250 {
                        let rep_name = parts[0].to_string();
                        let sub_name = parts[2].to_string();
                        repeater_buckets
                            .entry(rep_name)
                            .or_default()
                            .entry(row_idx)
                            .or_default()
                            .insert(sub_name, v.clone());
                    }
                }
            }
        }

        for (rep_name, rows_map) in repeater_buckets {
            // Only overwrite if `rep_name` is either missing or an integer count in `raw_meta`
            let should_replace = match acf_map.get(&rep_name) {
                None => true,
                Some(serde_json::Value::Number(_)) => true,
                Some(serde_json::Value::String(s)) => s.parse::<usize>().is_ok(),
                _ => false,
            };
            if should_replace {
                let arr: Vec<serde_json::Value> = rows_map
                    .into_values()
                    .map(serde_json::Value::Object)
                    .collect();
                acf_map.insert(rep_name, serde_json::Value::Array(arr));
            }
        }

        // Apply schema-aware media URL and relational post resolution if a field group exists
        if let Ok(groups) = Self::list_field_groups_with_conn(conn, Some(post_type)) {
            let base_trimmed = base_url.trim_end_matches('/');
            for group in groups {
                for field in group.fields {
                    match field.field_type.as_str() {
                        "media" | "image" | "file" => {
                            if let Some(val) = acf_map.get(&field.name).cloned() {
                                let att_id = val.as_i64().or_else(|| {
                                    val.as_str().and_then(|s| s.trim().parse::<i64>().ok())
                                });
                                if let Some(aid) = att_id.filter(|&id| id > 0) {
                                    if let Ok(guid) = conn.query_row(
                                        "SELECT guid FROM posts WHERE ID = ?1",
                                        params![aid],
                                        |r| r.get::<_, String>(0),
                                    ) {
                                        if !guid.is_empty() {
                                            acf_map.insert(
                                                field.name.clone(),
                                                serde_json::Value::String(guid.clone()),
                                            );
                                            acf_map.insert(
                                                format!("{}_id", field.name),
                                                serde_json::Value::Number(aid.into()),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        "post_object" | "relationship" => {
                            if let Some(val) = acf_map.get(&field.name).cloned() {
                                let target_id = val.as_i64().or_else(|| {
                                    val.as_str().and_then(|s| s.trim().parse::<i64>().ok())
                                });
                                if let Some(tid) = target_id.filter(|&id| id > 0) {
                                    if let Ok((ttitle, tslug, ttype)) = conn.query_row(
                                        "SELECT post_title, post_name, post_type FROM posts WHERE ID = ?1",
                                        params![tid],
                                        |r| {
                                            Ok((
                                                r.get::<_, String>(0)?,
                                                r.get::<_, String>(1)?,
                                                r.get::<_, String>(2)?,
                                            ))
                                        },
                                    ) {
                                        let tlink = if matches!(ttype.as_str(), "post" | "page") {
                                            format!("{}/{}/", base_trimmed, tslug)
                                        } else {
                                            format!("{}/{}/{}/", base_trimmed, ttype, tslug)
                                        };
                                        acf_map.insert(
                                            format!("{}_resolved", field.name),
                                            serde_json::json!({
                                                "id": tid,
                                                "title": ttitle,
                                                "slug": tslug,
                                                "post_type": ttype,
                                                "link": tlink,
                                            }),
                                        );
                                        acf_map.insert(
                                            format!("{}_title", field.name),
                                            serde_json::Value::String(ttitle),
                                        );
                                        acf_map.insert(
                                            format!("{}_link", field.name),
                                            serde_json::Value::String(tlink),
                                        );
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }
        }

        serde_json::Value::Object(acf_map)
    }

    /// Provisions one of the 7 canonical Daypress 0.6.0 Starter Content Type Blueprints
    /// (CPT + linked Custom Taxonomy + Structured Field Group).
    pub fn provision_content_type_preset(
        &self,
        preset_key: &str,
    ) -> Result<(
        CustomPostTypeDefinition,
        Vec<CustomTaxonomyDefinition>,
        FieldGroupDefinition,
    )> {
        let key = preset_key.trim().to_ascii_lowercase();
        let (cpt_def, tax_def, group_def) = match key.as_str() {
            "case_studies" | "case_study" => (
                CustomPostTypeDefinition {
                    slug: "case_study".to_string(),
                    label_singular: "Case Study".to_string(),
                    label_plural: "Case Studies".to_string(),
                    description:
                        "Client success stories, measurable outcomes, and architecture breakdowns."
                            .to_string(),
                    icon: "fa-solid fa-briefcase".to_string(),
                    menu_position: 21,
                    public: true,
                    has_archive: true,
                    hierarchical: false,
                    rewrite_slug: "case_study".to_string(),
                    supports: vec![
                        "title".into(),
                        "editor".into(),
                        "thumbnail".into(),
                        "excerpt".into(),
                        "custom-fields".into(),
                    ],
                    taxonomies: vec!["case_industry".to_string()],
                    show_in_rest: true,
                    preset_key: Some("case_studies".to_string()),
                    post_count: 0,
                },
                CustomTaxonomyDefinition {
                    slug: "case_industry".to_string(),
                    label_singular: "Industry".to_string(),
                    label_plural: "Industries".to_string(),
                    description: "Industry verticals for Case Studies.".to_string(),
                    hierarchical: true,
                    public: true,
                    rewrite_slug: "case_industry".to_string(),
                    post_types: vec!["case_study".to_string()],
                    show_in_rest: true,
                    term_count: 0,
                },
                FieldGroupDefinition {
                    id: 0,
                    group_key: "group_preset_case_study".to_string(),
                    title: "Case Study Specifications".to_string(),
                    slug: "case_study_specs".to_string(),
                    description: "Client metrics, challenge/solution summary, and key outcomes."
                        .to_string(),
                    location_rules: vec![FieldGroupLocationRule {
                        param: "post_type".to_string(),
                        operator: "==".to_string(),
                        value: "case_study".to_string(),
                    }],
                    fields: vec![
                        StructuredFieldSchema {
                            key: "field_cs_client_name".into(),
                            name: "client_name".into(),
                            label: "Client Name".into(),
                            field_type: "text".into(),
                            placeholder: "e.g. Acme Corp".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_cs_headline_metric".into(),
                            name: "headline_metric".into(),
                            label: "Headline Metric".into(),
                            field_type: "text".into(),
                            placeholder: "e.g. +340% Conversion Lift".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_cs_timeline".into(),
                            name: "timeline".into(),
                            label: "Project Timeline".into(),
                            field_type: "text".into(),
                            placeholder: "e.g. 6 Weeks".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_cs_challenge".into(),
                            name: "challenge".into(),
                            label: "The Challenge".into(),
                            field_type: "textarea".into(),
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_cs_solution".into(),
                            name: "solution".into(),
                            label: "The Solution".into(),
                            field_type: "wysiwyg".into(),
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_cs_live_url".into(),
                            name: "project_url".into(),
                            label: "Live Project URL".into(),
                            field_type: "url".into(),
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_cs_metrics".into(),
                            name: "metrics".into(),
                            label: "Key Performance Metrics".into(),
                            field_type: "repeater".into(),
                            sub_fields: vec![
                                StructuredFieldSchema {
                                    key: "field_cs_metric_lbl".into(),
                                    name: "label".into(),
                                    label: "Label".into(),
                                    field_type: "text".into(),
                                    ..Default::default()
                                },
                                StructuredFieldSchema {
                                    key: "field_cs_metric_val".into(),
                                    name: "value".into(),
                                    label: "Value".into(),
                                    field_type: "text".into(),
                                    ..Default::default()
                                },
                            ],
                            ..Default::default()
                        },
                    ],
                    position: "normal".to_string(),
                    menu_order: 0,
                    is_active: true,
                },
            ),
            "team" | "team_member" => (
                CustomPostTypeDefinition {
                    slug: "team_member".to_string(),
                    label_singular: "Team Member".to_string(),
                    label_plural: "Team".to_string(),
                    description: "Leadership, engineering, and editorial team profiles."
                        .to_string(),
                    icon: "fa-solid fa-user-group".to_string(),
                    menu_position: 22,
                    public: true,
                    has_archive: true,
                    hierarchical: false,
                    rewrite_slug: "team_member".to_string(),
                    supports: vec![
                        "title".into(),
                        "editor".into(),
                        "thumbnail".into(),
                        "excerpt".into(),
                        "custom-fields".into(),
                    ],
                    taxonomies: vec!["department".to_string()],
                    show_in_rest: true,
                    preset_key: Some("team".to_string()),
                    post_count: 0,
                },
                CustomTaxonomyDefinition {
                    slug: "department".to_string(),
                    label_singular: "Department".to_string(),
                    label_plural: "Departments".to_string(),
                    description: "Organizational departments for Team Members.".to_string(),
                    hierarchical: true,
                    public: true,
                    rewrite_slug: "department".to_string(),
                    post_types: vec!["team_member".to_string()],
                    show_in_rest: true,
                    term_count: 0,
                },
                FieldGroupDefinition {
                    id: 0,
                    group_key: "group_preset_team_member".to_string(),
                    title: "Team Member Profile".to_string(),
                    slug: "team_member_profile".to_string(),
                    description: "Role title, contact links, and headshot.".to_string(),
                    location_rules: vec![FieldGroupLocationRule {
                        param: "post_type".to_string(),
                        operator: "==".to_string(),
                        value: "team_member".to_string(),
                    }],
                    fields: vec![
                        StructuredFieldSchema {
                            key: "field_tm_role".into(),
                            name: "role_title".into(),
                            label: "Role / Job Title".into(),
                            field_type: "text".into(),
                            placeholder: "e.g. Principal Systems Architect".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_tm_location".into(),
                            name: "location".into(),
                            label: "Location".into(),
                            field_type: "text".into(),
                            placeholder: "e.g. Amsterdam, NL".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_tm_headshot".into(),
                            name: "headshot".into(),
                            label: "Portrait Image".into(),
                            field_type: "media".into(),
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_tm_linkedin".into(),
                            name: "linkedin_url".into(),
                            label: "LinkedIn URL".into(),
                            field_type: "url".into(),
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_tm_bio".into(),
                            name: "short_bio".into(),
                            label: "Short Bio".into(),
                            field_type: "textarea".into(),
                            ..Default::default()
                        },
                    ],
                    position: "normal".to_string(),
                    menu_order: 0,
                    is_active: true,
                },
            ),
            "docs" | "doc" => (
                CustomPostTypeDefinition {
                    slug: "doc".to_string(),
                    label_singular: "Doc".to_string(),
                    label_plural: "Docs".to_string(),
                    description: "Technical documentation, API guides, and reference articles."
                        .to_string(),
                    icon: "fa-solid fa-book-open".to_string(),
                    menu_position: 23,
                    public: true,
                    has_archive: true,
                    hierarchical: true,
                    rewrite_slug: "doc".to_string(),
                    supports: vec![
                        "title".into(),
                        "editor".into(),
                        "excerpt".into(),
                        "custom-fields".into(),
                    ],
                    taxonomies: vec!["doc_section".to_string()],
                    show_in_rest: true,
                    preset_key: Some("docs".to_string()),
                    post_count: 0,
                },
                CustomTaxonomyDefinition {
                    slug: "doc_section".to_string(),
                    label_singular: "Doc Section".to_string(),
                    label_plural: "Doc Sections".to_string(),
                    description: "Documentation sections and chapters.".to_string(),
                    hierarchical: true,
                    public: true,
                    rewrite_slug: "doc_section".to_string(),
                    post_types: vec!["doc".to_string()],
                    show_in_rest: true,
                    term_count: 0,
                },
                FieldGroupDefinition {
                    id: 0,
                    group_key: "group_preset_doc".to_string(),
                    title: "Documentation Metadata".to_string(),
                    slug: "doc_metadata".to_string(),
                    description: "Version, difficulty level, and repository links.".to_string(),
                    location_rules: vec![FieldGroupLocationRule {
                        param: "post_type".to_string(),
                        operator: "==".to_string(),
                        value: "doc".to_string(),
                    }],
                    fields: vec![
                        StructuredFieldSchema {
                            key: "field_doc_version".into(),
                            name: "doc_version".into(),
                            label: "Version".into(),
                            field_type: "text".into(),
                            default_value: "v0.6".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_doc_difficulty".into(),
                            name: "difficulty".into(),
                            label: "Difficulty".into(),
                            field_type: "select".into(),
                            choices: vec![
                                "Beginner".into(),
                                "Intermediate".into(),
                                "Advanced".into(),
                            ],
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_doc_reading_time".into(),
                            name: "reading_time".into(),
                            label: "Reading Time ( mins )".into(),
                            field_type: "number".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_doc_github".into(),
                            name: "github_edit_url".into(),
                            label: "GitHub Edit URL".into(),
                            field_type: "url".into(),
                            ..Default::default()
                        },
                    ],
                    position: "normal".to_string(),
                    menu_order: 0,
                    is_active: true,
                },
            ),
            "jobs" | "job_listing" => (
                CustomPostTypeDefinition {
                    slug: "job_listing".to_string(),
                    label_singular: "Job".to_string(),
                    label_plural: "Jobs".to_string(),
                    description: "Open career positions and role specifications.".to_string(),
                    icon: "fa-solid fa-id-badge".to_string(),
                    menu_position: 24,
                    public: true,
                    has_archive: true,
                    hierarchical: false,
                    rewrite_slug: "job_listing".to_string(),
                    supports: vec![
                        "title".into(),
                        "editor".into(),
                        "excerpt".into(),
                        "custom-fields".into(),
                    ],
                    taxonomies: vec!["job_department".to_string()],
                    show_in_rest: true,
                    preset_key: Some("jobs".to_string()),
                    post_count: 0,
                },
                CustomTaxonomyDefinition {
                    slug: "job_department".to_string(),
                    label_singular: "Hiring Team".to_string(),
                    label_plural: "Hiring Teams".to_string(),
                    description: "Departments hiring for open roles.".to_string(),
                    hierarchical: true,
                    public: true,
                    rewrite_slug: "job_department".to_string(),
                    post_types: vec!["job_listing".to_string()],
                    show_in_rest: true,
                    term_count: 0,
                },
                FieldGroupDefinition {
                    id: 0,
                    group_key: "group_preset_job_listing".to_string(),
                    title: "Job Opening Specifications".to_string(),
                    slug: "job_opening_specs".to_string(),
                    description:
                        "Location, employment type, compensation range, and application link."
                            .to_string(),
                    location_rules: vec![FieldGroupLocationRule {
                        param: "post_type".to_string(),
                        operator: "==".to_string(),
                        value: "job_listing".to_string(),
                    }],
                    fields: vec![
                        StructuredFieldSchema {
                            key: "field_job_location".into(),
                            name: "location".into(),
                            label: "Location".into(),
                            field_type: "text".into(),
                            placeholder: "e.g. Remote / Amsterdam".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_job_type".into(),
                            name: "employment_type".into(),
                            label: "Employment Type".into(),
                            field_type: "select".into(),
                            choices: vec![
                                "Full-Time".into(),
                                "Part-Time".into(),
                                "Contract".into(),
                                "Remote".into(),
                            ],
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_job_salary".into(),
                            name: "salary_range".into(),
                            label: "Compensation Range".into(),
                            field_type: "text".into(),
                            placeholder: "e.g. €95k – €130k".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_job_apply_url".into(),
                            name: "apply_url".into(),
                            label: "Application URL".into(),
                            field_type: "url".into(),
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_job_closing".into(),
                            name: "closing_date".into(),
                            label: "Closing Date".into(),
                            field_type: "date".into(),
                            ..Default::default()
                        },
                    ],
                    position: "normal".to_string(),
                    menu_order: 0,
                    is_active: true,
                },
            ),
            "events" | "event" => (
                CustomPostTypeDefinition {
                    slug: "event".to_string(),
                    label_singular: "Event".to_string(),
                    label_plural: "Events".to_string(),
                    description: "Conferences, workshops, webinars, and community meetups."
                        .to_string(),
                    icon: "fa-solid fa-calendar-days".to_string(),
                    menu_position: 25,
                    public: true,
                    has_archive: true,
                    hierarchical: false,
                    rewrite_slug: "event".to_string(),
                    supports: vec![
                        "title".into(),
                        "editor".into(),
                        "thumbnail".into(),
                        "excerpt".into(),
                        "custom-fields".into(),
                    ],
                    taxonomies: vec!["event_type".to_string()],
                    show_in_rest: true,
                    preset_key: Some("events".to_string()),
                    post_count: 0,
                },
                CustomTaxonomyDefinition {
                    slug: "event_type".to_string(),
                    label_singular: "Event Format".to_string(),
                    label_plural: "Event Formats".to_string(),
                    description: "Format or category for Events.".to_string(),
                    hierarchical: true,
                    public: true,
                    rewrite_slug: "event_type".to_string(),
                    post_types: vec!["event".to_string()],
                    show_in_rest: true,
                    term_count: 0,
                },
                FieldGroupDefinition {
                    id: 0,
                    group_key: "group_preset_event".to_string(),
                    title: "Event Schedule & Venue".to_string(),
                    slug: "event_schedule_venue".to_string(),
                    description: "Event date, venue, ticket pricing, and agenda.".to_string(),
                    location_rules: vec![FieldGroupLocationRule {
                        param: "post_type".to_string(),
                        operator: "==".to_string(),
                        value: "event".to_string(),
                    }],
                    fields: vec![
                        StructuredFieldSchema {
                            key: "field_ev_date".into(),
                            name: "event_date".into(),
                            label: "Event Date".into(),
                            field_type: "date".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_ev_city".into(),
                            name: "location_city".into(),
                            label: "City / Venue".into(),
                            field_type: "text".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_ev_price".into(),
                            name: "ticket_price".into(),
                            label: "Ticket Price".into(),
                            field_type: "text".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_ev_reg_url".into(),
                            name: "registration_url".into(),
                            label: "Registration URL".into(),
                            field_type: "url".into(),
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_ev_virtual".into(),
                            name: "is_virtual".into(),
                            label: "Virtual / Livestream Event".into(),
                            field_type: "true_false".into(),
                            ..Default::default()
                        },
                    ],
                    position: "normal".to_string(),
                    menu_order: 0,
                    is_active: true,
                },
            ),
            "portfolio" | "project" => (
                CustomPostTypeDefinition {
                    slug: "portfolio".to_string(),
                    label_singular: "Project".to_string(),
                    label_plural: "Portfolio".to_string(),
                    description: "Creative works, design showcases, and engineering projects."
                        .to_string(),
                    icon: "fa-solid fa-layer-group".to_string(),
                    menu_position: 26,
                    public: true,
                    has_archive: true,
                    hierarchical: false,
                    rewrite_slug: "portfolio".to_string(),
                    supports: vec![
                        "title".into(),
                        "editor".into(),
                        "thumbnail".into(),
                        "excerpt".into(),
                        "custom-fields".into(),
                    ],
                    taxonomies: vec!["project_category".to_string()],
                    show_in_rest: true,
                    preset_key: Some("portfolio".to_string()),
                    post_count: 0,
                },
                CustomTaxonomyDefinition {
                    slug: "project_category".to_string(),
                    label_singular: "Project Discipline".to_string(),
                    label_plural: "Project Disciplines".to_string(),
                    description: "Creative or technical disciplines for Portfolio Projects."
                        .to_string(),
                    hierarchical: true,
                    public: true,
                    rewrite_slug: "project_category".to_string(),
                    post_types: vec!["portfolio".to_string()],
                    show_in_rest: true,
                    term_count: 0,
                },
                FieldGroupDefinition {
                    id: 0,
                    group_key: "group_preset_portfolio".to_string(),
                    title: "Portfolio Project Specs".to_string(),
                    slug: "portfolio_project_specs".to_string(),
                    description: "Client, completion year, deliverables, and live link."
                        .to_string(),
                    location_rules: vec![FieldGroupLocationRule {
                        param: "post_type".to_string(),
                        operator: "==".to_string(),
                        value: "portfolio".to_string(),
                    }],
                    fields: vec![
                        StructuredFieldSchema {
                            key: "field_pf_client".into(),
                            name: "client".into(),
                            label: "Client".into(),
                            field_type: "text".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_pf_year".into(),
                            name: "completion_year".into(),
                            label: "Year".into(),
                            field_type: "text".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_pf_deliverables".into(),
                            name: "deliverables".into(),
                            label: "Deliverables".into(),
                            field_type: "text".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_pf_url".into(),
                            name: "live_url".into(),
                            label: "Live URL".into(),
                            field_type: "url".into(),
                            ..Default::default()
                        },
                    ],
                    position: "normal".to_string(),
                    menu_order: 0,
                    is_active: true,
                },
            ),
            "testimonials" | "testimonial" => (
                CustomPostTypeDefinition {
                    slug: "testimonial".to_string(),
                    label_singular: "Testimonial".to_string(),
                    label_plural: "Testimonials".to_string(),
                    description: "Customer quotes, ratings, and verified endorsements.".to_string(),
                    icon: "fa-solid fa-quote-left".to_string(),
                    menu_position: 27,
                    public: true,
                    has_archive: true,
                    hierarchical: false,
                    rewrite_slug: "testimonial".to_string(),
                    supports: vec![
                        "title".into(),
                        "editor".into(),
                        "thumbnail".into(),
                        "excerpt".into(),
                        "custom-fields".into(),
                    ],
                    taxonomies: vec!["service_line".to_string()],
                    show_in_rest: true,
                    preset_key: Some("testimonials".to_string()),
                    post_count: 0,
                },
                CustomTaxonomyDefinition {
                    slug: "service_line".to_string(),
                    label_singular: "Service Line".to_string(),
                    label_plural: "Service Lines".to_string(),
                    description: "Product or service lines for Testimonials.".to_string(),
                    hierarchical: true,
                    public: true,
                    rewrite_slug: "service_line".to_string(),
                    post_types: vec!["testimonial".to_string()],
                    show_in_rest: true,
                    term_count: 0,
                },
                FieldGroupDefinition {
                    id: 0,
                    group_key: "group_preset_testimonial".to_string(),
                    title: "Client Testimonial Details".to_string(),
                    slug: "testimonial_details".to_string(),
                    description: "Author attribution, company, rating, and related case study."
                        .to_string(),
                    location_rules: vec![FieldGroupLocationRule {
                        param: "post_type".to_string(),
                        operator: "==".to_string(),
                        value: "testimonial".to_string(),
                    }],
                    fields: vec![
                        StructuredFieldSchema {
                            key: "field_ts_author".into(),
                            name: "author_name".into(),
                            label: "Author Name".into(),
                            field_type: "text".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_ts_role".into(),
                            name: "author_role".into(),
                            label: "Author Role".into(),
                            field_type: "text".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_ts_company".into(),
                            name: "company_name".into(),
                            label: "Company".into(),
                            field_type: "text".into(),
                            show_in_collection_card: true,
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_ts_rating".into(),
                            name: "rating".into(),
                            label: "Star Rating (1-5)".into(),
                            field_type: "number".into(),
                            default_value: "5".into(),
                            ..Default::default()
                        },
                        StructuredFieldSchema {
                            key: "field_ts_related".into(),
                            name: "related_case_study".into(),
                            label: "Related Case Study".into(),
                            field_type: "post_object".into(),
                            target_post_type: Some("case_study".into()),
                            ..Default::default()
                        },
                    ],
                    position: "normal".to_string(),
                    menu_order: 0,
                    is_active: true,
                },
            ),
            _ => {
                return Err(rusqlite::Error::InvalidParameterName(format!(
                    "Unknown content type preset: {preset_key}"
                )));
            }
        };

        let saved_tax = self.save_custom_taxonomy_definition(&tax_def)?;
        let saved_cpt = self.save_post_type_definition(&cpt_def)?;
        let saved_group = self.save_field_group(&group_def)?;

        Ok((saved_cpt, vec![saved_tax], saved_group))
    }

    /// Queries published posts for a Filterable Collection Grid (`LoopGrid`), supporting:
    /// - taxonomy & term filtering (`taxonomy:term_slug` or separate `taxonomy` + `term`)
    /// - keyword search across title, excerpt, content, and custom postmeta values
    /// - sorting (`date_desc`, `date_asc`, `title_asc`, `title_desc`, `menu_order`)
    /// - pagination (`page`, `per_page`)
    ///
    /// Also returns the available filter terms `(term_slug, term_name, post_count)` for the active taxonomy.
    #[allow(clippy::too_many_arguments)]
    pub fn query_collection_posts(
        &self,
        post_type: &str,
        taxonomy: Option<&str>,
        term: Option<&str>,
        sort: Option<&str>,
        search: Option<&str>,
        page: usize,
        per_page: usize,
        base_url: &str,
    ) -> Result<(
        Vec<crate::models::DaypressPostResponse>,
        usize,
        String,
        Vec<(String, String, i64)>,
    )> {
        let ptype = if post_type.trim().is_empty() {
            "post"
        } else {
            post_type.trim()
        };

        // Resolve taxonomy and optional default term from `taxonomy` string (which can be `"industry"` or `"industry:fintech"`)
        let (resolved_tax, embedded_term) = match taxonomy.map(str::trim).filter(|s| !s.is_empty())
        {
            Some(raw) => {
                if let Some((tx, tm)) = raw.split_once(':') {
                    (
                        tx.trim().to_string(),
                        Some(tm.trim().to_string()).filter(|s| !s.is_empty()),
                    )
                } else {
                    (raw.to_string(), None)
                }
            }
            None => (String::new(), None),
        };

        let active_term = term
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("all"))
            .map(String::from)
            .or(embedded_term);

        let conn = self.conn.lock().unwrap();

        // Determine the filter bar taxonomy: either `resolved_tax`, or the first taxonomy bound to `ptype`, or `category` for `post`
        let filter_taxonomy = if !resolved_tax.is_empty() {
            resolved_tax
        } else if ptype == "post" {
            "category".to_string()
        } else {
            // Check daypress_post_types or daypress_taxonomies or term_relationships
            let mut found = String::new();
            if let Ok(tax_json) = conn.query_row(
                "SELECT taxonomies_json FROM daypress_post_types WHERE slug = ?1",
                params![ptype],
                |r| r.get::<_, String>(0),
            ) {
                if let Ok(list) = serde_json::from_str::<Vec<String>>(&tax_json) {
                    if let Some(first) = list.into_iter().find(|s| !s.trim().is_empty()) {
                        found = first;
                    }
                }
            }
            if found.is_empty() {
                if let Ok(tx) = conn.query_row(
                    "SELECT DISTINCT tt.taxonomy FROM term_taxonomy tt
                     JOIN term_relationships r ON tt.term_taxonomy_id = r.term_taxonomy_id
                     JOIN posts p ON r.object_id = p.ID
                     WHERE p.post_type = ?1 AND tt.taxonomy NOT IN ('nav_menu', 'link_category', 'post_format')
                     ORDER BY tt.taxonomy ASC LIMIT 1",
                    params![ptype],
                    |r| r.get::<_, String>(0),
                ) {
                    found = tx;
                }
            }
            found
        };

        // Gather available terms for `filter_taxonomy`
        let mut filter_terms: Vec<(String, String, i64)> = Vec::new();
        if !filter_taxonomy.is_empty() {
            if let Ok(mut tstmt) = conn.prepare(
                "SELECT t.slug, t.name, COUNT(DISTINCT p.ID) as cnt
                 FROM term_taxonomy tt
                 JOIN terms t ON tt.term_id = t.term_id
                 LEFT JOIN term_relationships r ON r.term_taxonomy_id = tt.term_taxonomy_id
                 LEFT JOIN posts p ON p.ID = r.object_id AND p.post_type = ?1 AND p.post_status = 'publish'
                 WHERE tt.taxonomy = ?2
                 GROUP BY t.term_id, t.slug, t.name
                 ORDER BY t.name ASC",
            ) {
                if let Ok(rows) = tstmt.query_map(params![ptype, &filter_taxonomy], |r| {
                    Ok((
                        r.get::<_, String>(0)?,
                        r.get::<_, String>(1)?,
                        r.get::<_, i64>(2)?,
                    ))
                }) {
                    for row in rows.flatten() {
                        filter_terms.push(row);
                    }
                }
            }
        }

        // Fetch all published posts of `ptype`
        let order_clause = match sort.unwrap_or("date_desc") {
            "date_asc" => "p.is_sticky DESC, p.post_date ASC, p.ID ASC",
            "title_asc" => "p.post_title COLLATE NOCASE ASC, p.ID DESC",
            "title_desc" => "p.post_title COLLATE NOCASE DESC, p.ID DESC",
            "menu_order" => "p.menu_order ASC, p.post_date DESC, p.ID DESC",
            _ => "p.is_sticky DESC, p.post_date DESC, p.ID DESC",
        };

        let sql = format!(
            "SELECT p.ID, p.post_date, p.post_date_gmt, p.post_modified, p.post_modified_gmt,
                    p.post_title, p.post_content, p.post_excerpt, p.post_name, p.post_status,
                    p.post_type, p.guid, p.post_author, p.comment_status, p.ping_status,
                    p.post_parent, p.menu_order, p.is_sticky
             FROM posts p
             WHERE p.post_type = ?1 AND p.post_status = 'publish'
             ORDER BY {order_clause}"
        );

        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map(params![ptype], Self::map_post_row)?;
        let mut all_posts: Vec<crate::models::DaypressPostResponse> = Vec::new();
        for r in rows {
            let mut post = r?;
            Self::hydrate_post_relations(&conn, &mut post, base_url);
            all_posts.push(post);
        }

        // Filter by active taxonomy term if specified
        if let Some(ref t_filter) = active_term {
            let t_lower = t_filter.to_lowercase();
            all_posts.retain(|post| {
                let matches_term: bool = if !filter_taxonomy.is_empty() {
                    conn.query_row(
                        "SELECT COUNT(*) FROM term_relationships r
                         JOIN term_taxonomy tt ON r.term_taxonomy_id = tt.term_taxonomy_id
                         JOIN terms t ON tt.term_id = t.term_id
                         WHERE r.object_id = ?1 AND tt.taxonomy = ?2
                           AND (LOWER(t.slug) = ?3 OR LOWER(t.name) = ?3 OR CAST(t.term_id AS TEXT) = ?3)",
                        params![post.id, &filter_taxonomy, &t_lower],
                        |r| r.get::<_, i64>(0),
                    )
                    .unwrap_or(0)
                        > 0
                } else {
                    conn.query_row(
                        "SELECT COUNT(*) FROM term_relationships r
                         JOIN term_taxonomy tt ON r.term_taxonomy_id = tt.term_taxonomy_id
                         JOIN terms t ON tt.term_id = t.term_id
                         WHERE r.object_id = ?1
                           AND (LOWER(t.slug) = ?2 OR LOWER(t.name) = ?2 OR CAST(t.term_id AS TEXT) = ?2)",
                        params![post.id, &t_lower],
                        |r| r.get::<_, i64>(0),
                    )
                    .unwrap_or(0)
                        > 0
                };
                matches_term
            });
        }

        // Filter by search query if specified
        if let Some(q) = search.map(str::trim).filter(|s| !s.is_empty()) {
            let q_lower = q.to_lowercase();
            all_posts.retain(|post| {
                if post.title.rendered.to_lowercase().contains(&q_lower)
                    || post.excerpt.rendered.to_lowercase().contains(&q_lower)
                    || post.content.rendered.to_lowercase().contains(&q_lower)
                {
                    return true;
                }
                if let Some(obj) = post.acf.as_object() {
                    for val in obj.values() {
                        if let Some(s) = val.as_str() {
                            if s.to_lowercase().contains(&q_lower) {
                                return true;
                            }
                        }
                    }
                }
                false
            });
        }

        let total = all_posts.len();
        let safe_per_page = per_page.clamp(1, 100);
        let safe_page = page.max(1);
        let offset = (safe_page - 1).saturating_mul(safe_per_page);
        let paged_items: Vec<crate::models::DaypressPostResponse> = all_posts
            .into_iter()
            .skip(offset)
            .take(safe_per_page)
            .collect();

        Ok((paged_items, total, filter_taxonomy, filter_terms))
    }
}
