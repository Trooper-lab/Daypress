//! Post queries for the RSS 2.0 feeds (`handlers::rss_feed`).

use super::Database;
use crate::models::WpPostResponse;
use rusqlite::{params, OptionalExtension, Result};

/// A category or tag a feed is limited to.
#[derive(Debug, Clone)]
pub struct FeedTerm {
    pub taxonomy: String,
    pub term_id: i64,
    pub name: String,
    pub slug: String,
}

const FEED_SELECT: &str = "SELECT ID, post_date, post_date_gmt, post_modified, post_modified_gmt, \
    post_title, post_content, post_excerpt, post_name, post_status, post_type, guid, post_author, \
    comment_status, ping_status, COALESCE(post_parent, 0), COALESCE(menu_order, 0), \
    COALESCE(is_sticky, 0) FROM posts";

impl Database {
    /// Looks up a `category` or `post_tag` term by slug.
    pub fn find_feed_term(&self, taxonomy: &str, slug: &str) -> Result<Option<FeedTerm>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.query_row(
            "SELECT t.term_id, t.name, t.slug FROM terms t
             JOIN term_taxonomy tt ON tt.term_id = t.term_id
             WHERE tt.taxonomy = ?1 AND t.slug = ?2
             LIMIT 1",
            params![taxonomy, slug],
            |r| {
                Ok(FeedTerm {
                    taxonomy: taxonomy.to_string(),
                    term_id: r.get(0)?,
                    name: r.get(1)?,
                    slug: r.get(2)?,
                })
            },
        )
        .optional()
    }

    /// The newest published blog posts (newest first, stickiness ignored), optionally only
    /// those in `term`. A post without any category counts as category 1, exactly as the
    /// category archive and the REST API report it.
    pub fn get_feed_posts(
        &self,
        term: Option<&FeedTerm>,
        limit: i64,
    ) -> Result<Vec<WpPostResponse>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let base_url = Self::get_base_url_with_conn(&conn);
        let order = "ORDER BY post_date DESC, ID DESC LIMIT ?";
        let published = "WHERE post_type = 'post' AND post_status = 'publish'";

        let mut posts = Vec::new();
        match term {
            None => {
                let sql = format!("{FEED_SELECT} {published} {order}");
                let mut stmt = conn.prepare_cached(&sql)?;
                for row in stmt.query_map(params![limit], Self::map_post_row)? {
                    posts.push(row?);
                }
            }
            Some(term) => {
                let sql = format!(
                    "{FEED_SELECT} {published} AND (
                        ID IN (SELECT r.object_id FROM term_relationships r
                               JOIN term_taxonomy tt ON tt.term_taxonomy_id = r.term_taxonomy_id
                               WHERE tt.taxonomy = ?1 AND tt.term_id = ?2)
                        OR (?1 = 'category' AND ?2 = 1 AND ID NOT IN (
                               SELECT r.object_id FROM term_relationships r
                               JOIN term_taxonomy tt ON tt.term_taxonomy_id = r.term_taxonomy_id
                               WHERE tt.taxonomy = 'category'))
                    ) {order}"
                );
                let mut stmt = conn.prepare_cached(&sql)?;
                let rows = stmt.query_map(
                    params![term.taxonomy, term.term_id, limit],
                    Self::map_post_row,
                )?;
                for row in rows {
                    posts.push(row?);
                }
            }
        }

        for post in &mut posts {
            Self::hydrate_post_relations(&conn, post, &base_url);
        }
        Ok(posts)
    }
}
