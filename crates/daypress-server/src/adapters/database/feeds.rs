//! Post and comment queries for the RSS 2.0 and Atom 1.0 feeds (`handlers::rss_feed`).

use super::Database;
use crate::models::DaypressPostResponse;
use rusqlite::{params, OptionalExtension, Result};

/// A category or tag a feed is limited to.
#[derive(Debug, Clone)]
pub struct FeedTerm {
    pub taxonomy: String,
    pub term_id: i64,
    pub name: String,
    pub slug: String,
}

/// An approved comment on a published post for `/comments/feed/` or `/{slug}/feed/`.
#[derive(Debug, Clone)]
pub struct FeedComment {
    pub comment_id: i64,
    pub post_id: i64,
    pub post_title: String,
    pub post_slug: String,
    pub post_link: String,
    pub author: String,
    pub date: String,
    pub date_gmt: String,
    pub content: String,
    pub link: String,
    pub guid: String,
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
    ) -> Result<Vec<DaypressPostResponse>> {
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

    /// Returns approved comments on published posts/pages (newest first), optionally restricted
    /// to a single post ID (`/{slug}/feed/`).
    pub fn get_feed_comments(&self, post_id: Option<i64>, limit: i64) -> Result<Vec<FeedComment>> {
        let conn = self.conn.lock().unwrap_or_else(|e| e.into_inner());
        let base_url = Self::get_base_url_with_conn(&conn)
            .trim_end_matches('/')
            .to_string();

        let sql = match post_id {
            Some(_) => {
                "SELECT c.comment_ID, c.comment_post_ID, p.post_title, p.post_name, p.guid,
                        c.comment_author, c.comment_date, c.comment_date_gmt, c.comment_content
                 FROM comments c
                 JOIN posts p ON p.ID = c.comment_post_ID
                 WHERE p.post_status = 'publish'
                   AND (c.comment_approved = '1' OR c.comment_approved = 'approved')
                   AND c.comment_post_ID = ?1
                 ORDER BY c.comment_date_gmt DESC, c.comment_ID DESC
                 LIMIT ?2"
            }
            None => {
                "SELECT c.comment_ID, c.comment_post_ID, p.post_title, p.post_name, p.guid,
                        c.comment_author, c.comment_date, c.comment_date_gmt, c.comment_content
                 FROM comments c
                 JOIN posts p ON p.ID = c.comment_post_ID
                 WHERE p.post_status = 'publish'
                   AND (c.comment_approved = '1' OR c.comment_approved = 'approved')
                 ORDER BY c.comment_date_gmt DESC, c.comment_ID DESC
                 LIMIT ?1"
            }
        };

        let map_row = |r: &rusqlite::Row| -> Result<FeedComment> {
            let cid: i64 = r.get(0)?;
            let pid: i64 = r.get(1)?;
            let ptitle: String = r.get(2)?;
            let pslug: String = r.get(3)?;
            let pguid: String = r.get(4)?;
            let author: String = r.get(5)?;
            let date: String = r.get(6)?;
            let date_gmt: String = r.get(7)?;
            let content: String = r.get(8)?;
            let post_link = if pslug.trim().is_empty() {
                format!("{}/?p={}", base_url, pid)
            } else {
                format!("{}/{}/", base_url, pslug.trim_matches('/'))
            };
            let link = format!("{}#comment-{}", post_link, cid);
            let guid_base = if pguid.trim().is_empty() {
                format!("{}/?p={}", base_url, pid)
            } else {
                pguid
            };
            let guid = format!("{}#comment-{}", guid_base, cid);
            Ok(FeedComment {
                comment_id: cid,
                post_id: pid,
                post_title: ptitle,
                post_slug: pslug,
                post_link,
                author: if author.trim().is_empty() {
                    "Anonymous".to_string()
                } else {
                    author
                },
                date,
                date_gmt,
                content,
                link,
                guid,
            })
        };

        let mut stmt = conn.prepare_cached(sql)?;
        let mut out = Vec::new();
        match post_id {
            Some(pid) => {
                for row in stmt.query_map(params![pid, limit], map_row)? {
                    out.push(row?);
                }
            }
            None => {
                for row in stmt.query_map(params![limit], map_row)? {
                    out.push(row?);
                }
            }
        }
        Ok(out)
    }
}
