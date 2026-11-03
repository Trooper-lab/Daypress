//! Dynamic Taxonomy Discovery (`adapters/database/taxonomies_discovery.rs`).

use super::Database;
use crate::models::TaxonomyInfo;
use rusqlite::params;

impl Database {
    /// Discovers all taxonomies registered in SQLite for the given post_type,
    /// along with all available terms and currently assigned terms for the existing post.
    pub fn get_taxonomies_for_post_type(
        &self,
        post_type: &str,
        existing_post_id: Option<i64>,
    ) -> rusqlite::Result<Vec<TaxonomyInfo>> {
        let conn = self.conn.lock().unwrap();

        // 1. Gather distinct taxonomy slugs from term_taxonomy + daypress_taxonomies
        let mut all_tax_slugs = Vec::new();
        {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT taxonomy FROM term_taxonomy WHERE taxonomy NOT IN ('nav_menu', 'link_category', 'post_format') ORDER BY taxonomy ASC",
            )?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            for row in rows.flatten() {
                all_tax_slugs.push(row);
            }
            if let Ok(mut reg_stmt) =
                conn.prepare("SELECT slug FROM daypress_taxonomies ORDER BY slug ASC")
            {
                if let Ok(reg_rows) = reg_stmt.query_map([], |r| r.get::<_, String>(0)) {
                    for row in reg_rows.flatten() {
                        if !all_tax_slugs.contains(&row) {
                            all_tax_slugs.push(row);
                        }
                    }
                }
            }
        }

        // 2. Determine which taxonomies apply to this post_type
        let mut relevant_slugs: Vec<String> = Vec::new();

        // Check explicit bindings in daypress_post_types and daypress_taxonomies first
        if let Ok(tax_json) = conn.query_row(
            "SELECT taxonomies_json FROM daypress_post_types WHERE slug = ?1",
            params![post_type],
            |r| r.get::<_, String>(0),
        ) {
            if let Ok(bound) = serde_json::from_str::<Vec<String>>(&tax_json) {
                for b in bound {
                    if !b.trim().is_empty() && !relevant_slugs.contains(&b) {
                        relevant_slugs.push(b);
                    }
                }
            }
        }
        if let Ok(mut tstmt) =
            conn.prepare("SELECT slug, post_types_json FROM daypress_taxonomies ORDER BY slug ASC")
        {
            if let Ok(trows) =
                tstmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            {
                for (tslug, pt_json) in trows.flatten() {
                    let pts: Vec<String> = serde_json::from_str(&pt_json).unwrap_or_default();
                    if pts.contains(&post_type.to_string()) && !relevant_slugs.contains(&tslug) {
                        relevant_slugs.push(tslug);
                    }
                }
            }
        }

        if post_type == "post" {
            relevant_slugs.push("category".to_string());
            relevant_slugs.push("post_tag".to_string());
            let mut stmt = conn.prepare(
                "SELECT DISTINCT tt.taxonomy FROM term_taxonomy tt
                 JOIN term_relationships r ON tt.term_taxonomy_id = r.term_taxonomy_id
                 JOIN posts p ON r.object_id = p.ID
                 WHERE p.post_type = 'post' AND tt.taxonomy NOT IN ('nav_menu', 'link_category', 'post_format', 'category', 'post_tag')",
            )?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            for row in rows.flatten() {
                if !relevant_slugs.contains(&row) {
                    relevant_slugs.push(row);
                }
            }
        } else if post_type == "page" {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT tt.taxonomy FROM term_taxonomy tt
                 JOIN term_relationships r ON tt.term_taxonomy_id = r.term_taxonomy_id
                 JOIN posts p ON r.object_id = p.ID
                 WHERE p.post_type = 'page' AND tt.taxonomy NOT IN ('nav_menu', 'link_category', 'post_format')",
            )?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            for row in rows.flatten() {
                if !relevant_slugs.contains(&row) {
                    relevant_slugs.push(row);
                }
            }
        } else {
            // Custom post type
            let mut stmt = conn.prepare(
                "SELECT DISTINCT tt.taxonomy FROM term_taxonomy tt
                 JOIN term_relationships r ON tt.term_taxonomy_id = r.term_taxonomy_id
                 JOIN posts p ON r.object_id = p.ID
                 WHERE p.post_type = ?1 AND tt.taxonomy NOT IN ('nav_menu', 'link_category', 'post_format')",
            )?;
            let rows = stmt.query_map(params![post_type], |r| r.get::<_, String>(0))?;
            for row in rows.flatten() {
                if !relevant_slugs.contains(&row) {
                    relevant_slugs.push(row);
                }
            }

            // Naming patterns: e.g. training_category for trainingen
            for slug in &all_tax_slugs {
                if slug == "category" || slug == "post_tag" {
                    continue;
                }
                let pt_stem = post_type.trim_end_matches('s').trim_end_matches("en");
                if (slug.starts_with(pt_stem) || slug.ends_with(pt_stem) || slug.contains(pt_stem))
                    && !relevant_slugs.contains(slug)
                {
                    relevant_slugs.push(slug.clone());
                }
            }

            // Fallback: if no specific taxonomy found yet, include all custom taxonomies
            if relevant_slugs.is_empty() {
                for slug in &all_tax_slugs {
                    if slug != "category" && slug != "post_tag" {
                        relevant_slugs.push(slug.clone());
                    }
                }
            }
        }

        // If existing_post_id is provided, also include any taxonomy where this post has term relationships
        if let Some(pid) = existing_post_id {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT tt.taxonomy FROM term_taxonomy tt
                 JOIN term_relationships r ON tt.term_taxonomy_id = r.term_taxonomy_id
                 WHERE r.object_id = ?1 AND tt.taxonomy NOT IN ('nav_menu', 'link_category', 'post_format')",
            )?;
            let rows = stmt.query_map(params![pid], |r| r.get::<_, String>(0))?;
            for row in rows.flatten() {
                if !relevant_slugs.contains(&row) {
                    relevant_slugs.push(row);
                }
            }
        }

        drop(conn);

        // 3. For each relevant taxonomy, build TaxonomyInfo
        let mut results = Vec::new();
        for slug in relevant_slugs {
            let terms = self.get_terms(&slug)?;
            let assigned_ids = if let Some(pid) = existing_post_id {
                let conn = self.conn.lock().unwrap();
                let mut stmt = conn.prepare(
                    "SELECT tt.term_id FROM term_taxonomy tt
                     JOIN term_relationships r ON tt.term_taxonomy_id = r.term_taxonomy_id
                     WHERE r.object_id = ?1 AND tt.taxonomy = ?2",
                )?;
                let rows = stmt.query_map(params![pid, &slug], |r| r.get::<_, i64>(0))?;
                rows.flatten().collect()
            } else {
                Vec::new()
            };

            let (hierarchical, name) = {
                let conn = self.conn.lock().unwrap();
                if let Ok((lbl_pl, hier)) = conn.query_row(
                    "SELECT label_plural, hierarchical FROM daypress_taxonomies WHERE slug = ?1",
                    params![&slug],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)),
                ) {
                    (
                        hier != 0,
                        if lbl_pl.trim().is_empty() {
                            format_taxonomy_display_name(&slug)
                        } else {
                            lbl_pl
                        },
                    )
                } else {
                    (
                        !(slug == "post_tag" || slug.ends_with("_tag") || slug.ends_with("_tags")),
                        format_taxonomy_display_name(&slug),
                    )
                }
            };

            results.push(TaxonomyInfo {
                slug,
                name,
                hierarchical,
                terms,
                assigned_term_ids: assigned_ids,
            });
        }

        Ok(results)
    }
}

pub fn format_taxonomy_display_name(slug: &str) -> String {
    match slug {
        "category" => "Categories".to_string(),
        "post_tag" => "Tags".to_string(),
        other => {
            let parts: Vec<String> = other
                .replace(['_', '-'], " ")
                .split_whitespace()
                .map(|w| {
                    let mut c = w.chars();
                    match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }
                })
                .collect();
            let mut joined = parts.join(" ");
            if !joined.ends_with('s') && !joined.ends_with("ies") {
                joined.push('s');
            }
            joined
        }
    }
}
