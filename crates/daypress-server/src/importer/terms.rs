//! Taxonomies and terms ingestion.

use crate::importer::models::ParsedWxrTerm;
use rusqlite::{params, Result, Transaction};

pub fn ingest_terms(tx: &Transaction, terms: Vec<ParsedWxrTerm>) -> Result<(usize, usize)> {
    let mut cat_count = 0;
    let mut tag_count = 0;

    for term in terms {
        let slug = if term.slug.is_empty() {
            generated_slugify::slugify(&term.name)
        } else {
            term.slug
        };

        let term_id: i64 = match tx.query_row(
            "SELECT term_id FROM terms WHERE slug = ?1",
            params![slug],
            |r| r.get(0),
        ) {
            Ok(id) => {
                let _ = tx.execute(
                    "UPDATE terms SET name = ?1 WHERE term_id = ?2",
                    params![term.name, id],
                );
                id
            }
            Err(_) => {
                tx.execute(
                    "INSERT INTO terms (name, slug) VALUES (?1, ?2)",
                    params![term.name, slug],
                )?;
                tx.last_insert_rowid()
            }
        };

        let taxonomy = if term.taxonomy.is_empty() {
            "category"
        } else {
            &term.taxonomy
        };

        let tt_exists: Option<i64> = tx
            .query_row(
                "SELECT term_taxonomy_id FROM term_taxonomy WHERE term_id = ?1 AND taxonomy = ?2",
                params![term_id, taxonomy],
                |r| r.get(0),
            )
            .ok();

        if tt_exists.is_none() {
            let _ = tx.execute(
                "INSERT INTO term_taxonomy (term_id, taxonomy, description) VALUES (?1, ?2, ?3)",
                params![term_id, taxonomy, term.description],
            );
        }

        if taxonomy == "post_tag" {
            tag_count += 1;
        } else {
            cat_count += 1;
        }
    }

    Ok((cat_count, tag_count))
}

/// Persists post-to-term relationships for categories, tags, and custom taxonomies.
pub fn ingest_post_terms(
    tx: &Transaction,
    post_id: i64,
    categories: &[String],
    tags: &[String],
    custom_terms: &[(String, String, String)],
) -> Result<()> {
    let link_term = |taxonomy: &str, slug: &str, name: &str| -> Result<()> {
        let clean_slug = slug.trim();
        if clean_slug.is_empty() {
            return Ok(());
        }
        let clean_name = if name.trim().is_empty() {
            clean_slug
        } else {
            name.trim()
        };

        let term_id: i64 = match tx.query_row(
            "SELECT term_id FROM terms WHERE slug = ?1 LIMIT 1",
            params![clean_slug],
            |r| r.get(0),
        ) {
            Ok(id) => id,
            Err(_) => {
                tx.execute(
                    "INSERT INTO terms (name, slug) VALUES (?1, ?2)",
                    params![clean_name, clean_slug],
                )?;
                tx.last_insert_rowid()
            }
        };

        let tt_id: i64 = match tx.query_row(
            "SELECT term_taxonomy_id FROM term_taxonomy WHERE term_id = ?1 AND taxonomy = ?2 LIMIT 1",
            params![term_id, taxonomy],
            |r| r.get(0),
        ) {
            Ok(id) => id,
            Err(_) => {
                tx.execute(
                    "INSERT INTO term_taxonomy (term_id, taxonomy, description, parent, count) VALUES (?1, ?2, '', 0, 0)",
                    params![term_id, taxonomy],
                )?;
                tx.last_insert_rowid()
            }
        };

        let _ = tx.execute(
            "INSERT OR IGNORE INTO term_relationships (object_id, term_taxonomy_id, term_order) VALUES (?1, ?2, 0)",
            params![post_id, tt_id],
        );

        Ok(())
    };

    for cat in categories {
        link_term("category", cat, cat)?;
    }
    for tag in tags {
        link_term("post_tag", tag, tag)?;
    }
    for (tax, slug, name) in custom_terms {
        link_term(tax, slug, name)?;
    }

    Ok(())
}
