//! Ingestion and parent remapping of WXR comments.

use crate::importer::models::ParsedWxrComment;
use rusqlite::{params, Result, Transaction};
use std::collections::HashMap;

/// Pass 1: Inserts comments with comment_parent = 0 to prevent broken foreign keys,
/// tracks old_id -> new_id mappings, and synchronizes wp_posts.comment_count.
pub fn ingest_post_comments(
    tx: &Transaction,
    post_id: i64,
    comments: &[ParsedWxrComment],
    now: &str,
    old_to_new_comment_id: &mut HashMap<i64, i64>,
    pending_comment_parent_remaps: &mut Vec<(i64, i64)>,
) -> Result<usize> {
    let mut count = 0;
    for c in comments {
        let c_date = if c.date.is_empty() { now } else { &c.date };
        let _ = tx.execute(
            r#"INSERT INTO wp_comments (comment_post_ID, comment_author, comment_author_email, comment_author_url, comment_date, comment_date_gmt, comment_content, comment_approved, comment_type, comment_parent)
               VALUES (?1, ?2, ?3, ?4, ?5, ?5, ?6, ?7, ?8, 0)"#,
            params![post_id, c.author, c.email, c.url, c_date, c.content, c.approved, c.comment_type],
        );
        let new_cid = tx.last_insert_rowid();
        if let Some(old_cid) = c.id {
            old_to_new_comment_id.insert(old_cid, new_cid);
        }
        if c.parent > 0 {
            pending_comment_parent_remaps.push((new_cid, c.parent));
        }
        count += 1;
    }

    if count > 0 {
        let _ = tx.execute(
            "UPDATE wp_posts SET comment_count = (SELECT COUNT(*) FROM wp_comments WHERE comment_post_ID = ?1 AND (comment_approved = '1' OR comment_approved = 'approved')) WHERE ID = ?1",
            params![post_id],
        );
    }

    Ok(count)
}

/// Pass 2: Remaps comment_parent IDs using the old_to_new_comment_id map.
pub fn remap_comment_parents(
    tx: &Transaction,
    pending_comment_parent_remaps: &[(i64, i64)],
    old_to_new_comment_id: &HashMap<i64, i64>,
) -> Result<()> {
    for &(child_cid, old_parent_cid) in pending_comment_parent_remaps {
        if let Some(&new_parent_cid) = old_to_new_comment_id.get(&old_parent_cid) {
            let _ = tx.execute(
                "UPDATE wp_comments SET comment_parent = ?1 WHERE comment_ID = ?2",
                params![new_parent_cid, child_cid],
            );
        }
    }
    Ok(())
}
