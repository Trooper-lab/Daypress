#![allow(unused_imports)]
use super::Database;
use crate::auth::AuthUser;
use crate::domain::errors::{DomainError, DomainResult};
use crate::errors::AppError;
use crate::models::*;
use base64::Engine;
use chrono::Utc;
use rusqlite::{params, Connection, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

impl Database {
    // --- Comments Management ---

    pub fn create_comment(&self, req: &CreateCommentRequest) -> Result<DaypressCommentResponse> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        let author_name = req
            .author_name
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("Anonymous");
        let author_email = req.author_email.as_deref().unwrap_or("");
        let author_url = req.author_url.as_deref().unwrap_or("");
        let parent = req.parent.unwrap_or(0);
        let approved = match req.status.as_deref() {
            Some("0") | Some("hold") | Some("pending") => "0",
            Some("spam") => "spam",
            Some("trash") => "trash",
            _ => "1",
        };

        conn.execute(
            r#"INSERT INTO comments (comment_post_ID, comment_author, comment_author_email, comment_author_url, comment_author_IP, comment_date, comment_date_gmt, comment_content, comment_approved, comment_type, comment_parent, user_id)
               VALUES (?1, ?2, ?3, ?4, '127.0.0.1', ?5, ?5, ?6, ?7, 'comment', ?8, 0)"#,
            params![
                req.post,
                author_name,
                author_email,
                author_url,
                now,
                req.content,
                approved,
                parent
            ],
        )?;
        let cid = conn.last_insert_rowid();
        let base_url = Self::get_base_url_with_conn(&conn);

        Ok(DaypressCommentResponse {
            id: cid,
            post: req.post,
            parent,
            author: 0,
            author_name: author_name.to_string(),
            author_url: author_url.to_string(),
            date: now.replace(' ', "T"),
            date_gmt: now.replace(' ', "T"),
            content: DaypressRenderedText {
                rendered: req.content.clone(),
                protected: false,
            },
            link: format!(
                "{}/?p={}#comment-{}",
                base_url.trim_end_matches('/'),
                req.post,
                cid
            ),
            status: match approved {
                "1" => "approved".to_string(),
                "0" => "hold".to_string(),
                other => other.to_string(),
            },
            comment_type: "comment".to_string(),
        })
    }

    pub fn get_comments(
        &self,
        status_filter: Option<&str>,
        limit: i64,
    ) -> Result<Vec<DaypressCommentAdminItem>> {
        let conn = self.conn.lock().unwrap();
        let query = match status_filter {
            Some("approved") | Some("approve") | Some("1") => {
                "SELECT c.comment_ID, c.comment_post_ID, COALESCE(p.post_title, 'Untitled Post'), c.comment_author, c.comment_author_email, c.comment_author_url, c.comment_author_IP, c.comment_content, c.comment_date, c.comment_approved
                 FROM comments c LEFT JOIN posts p ON c.comment_post_ID = p.ID
                 WHERE c.comment_approved = '1' OR c.comment_approved = 'approved' OR c.comment_approved = 'approve'
                 ORDER BY c.comment_ID DESC LIMIT ?1"
            }
            Some("pending") | Some("hold") | Some("0") => {
                "SELECT c.comment_ID, c.comment_post_ID, COALESCE(p.post_title, 'Untitled Post'), c.comment_author, c.comment_author_email, c.comment_author_url, c.comment_author_IP, c.comment_content, c.comment_date, c.comment_approved
                 FROM comments c LEFT JOIN posts p ON c.comment_post_ID = p.ID
                 WHERE c.comment_approved = '0' OR c.comment_approved = 'hold' OR c.comment_approved = 'pending'
                 ORDER BY c.comment_ID DESC LIMIT ?1"
            }
            Some("spam") => {
                "SELECT c.comment_ID, c.comment_post_ID, COALESCE(p.post_title, 'Untitled Post'), c.comment_author, c.comment_author_email, c.comment_author_url, c.comment_author_IP, c.comment_content, c.comment_date, c.comment_approved
                 FROM comments c LEFT JOIN posts p ON c.comment_post_ID = p.ID
                 WHERE c.comment_approved = 'spam'
                 ORDER BY c.comment_ID DESC LIMIT ?1"
            }
            Some("trash") => {
                "SELECT c.comment_ID, c.comment_post_ID, COALESCE(p.post_title, 'Untitled Post'), c.comment_author, c.comment_author_email, c.comment_author_url, c.comment_author_IP, c.comment_content, c.comment_date, c.comment_approved
                 FROM comments c LEFT JOIN posts p ON c.comment_post_ID = p.ID
                 WHERE c.comment_approved = 'trash'
                 ORDER BY c.comment_ID DESC LIMIT ?1"
            }
            _ => {
                "SELECT c.comment_ID, c.comment_post_ID, COALESCE(p.post_title, 'Untitled Post'), c.comment_author, c.comment_author_email, c.comment_author_url, c.comment_author_IP, c.comment_content, c.comment_date, c.comment_approved
                 FROM comments c LEFT JOIN posts p ON c.comment_post_ID = p.ID
                 ORDER BY c.comment_ID DESC LIMIT ?1"
            }
        };

        let mut stmt = conn.prepare(query)?;
        let rows = stmt.query_map(params![limit], |r| {
            Ok(DaypressCommentAdminItem {
                id: r.get(0)?,
                post_id: r.get(1)?,
                post_title: r.get(2)?,
                author: r.get(3)?,
                author_email: r.get(4)?,
                author_url: r.get(5)?,
                author_ip: r.get(6)?,
                content: r.get(7)?,
                date: r.get(8)?,
                status: r.get(9)?,
            })
        })?;

        let mut res = Vec::new();
        for item in rows {
            res.push(item?);
        }
        Ok(res)
    }

    /// Whether `email` already has an approved comment (WordPress "previously approved" rule).
    pub fn has_approved_comment_from(&self, email: &str) -> Result<bool> {
        if email.trim().is_empty() {
            return Ok(false);
        }
        let conn = self.conn.lock().unwrap();
        let n: i64 = conn.query_row(
            "SELECT COUNT(*) FROM comments WHERE LOWER(comment_author_email) = LOWER(?1) AND comment_approved = '1'",
            params![email.trim()],
            |r| r.get(0),
        )?;
        Ok(n > 0)
    }

    pub fn get_comment_by_id(&self, id: i64) -> Result<Option<DaypressCommentAdminItem>> {
        let comments = self.get_comments(None, 5000)?;
        Ok(comments.into_iter().find(|c| c.id == id))
    }

    pub fn update_comment(
        &self,
        id: i64,
        req: &UpdateCommentRequest,
    ) -> Result<Option<DaypressCommentAdminItem>> {
        {
            let conn = self.conn.lock().unwrap();
            let exists: Option<i64> = conn
                .query_row(
                    "SELECT comment_ID FROM comments WHERE comment_ID = ?1",
                    params![id],
                    |r| r.get(0),
                )
                .ok();
            if exists.is_none() {
                return Ok(None);
            }

            if let Some(ref st) = req.status {
                let norm = match st.trim() {
                    "approved" | "approve" | "1" => "1",
                    "hold" | "pending" | "unapproved" | "0" => "0",
                    other => other,
                };
                conn.execute(
                    "UPDATE comments SET comment_approved = ?1 WHERE comment_ID = ?2",
                    params![norm, id],
                )?;
            }

            if let Some(ref c_val) = req.content {
                if let Some(c_str) = c_val.as_str().or_else(|| {
                    c_val
                        .get("raw")
                        .or_else(|| c_val.get("rendered"))
                        .and_then(|v| v.as_str())
                }) {
                    conn.execute(
                        "UPDATE comments SET comment_content = ?1 WHERE comment_ID = ?2",
                        params![c_str, id],
                    )?;
                }
            }

            if let Some(ref an) = req.author_name {
                conn.execute(
                    "UPDATE comments SET comment_author = ?1 WHERE comment_ID = ?2",
                    params![an, id],
                )?;
            }

            if let Some(ref ae) = req.author_email {
                conn.execute(
                    "UPDATE comments SET comment_author_email = ?1 WHERE comment_ID = ?2",
                    params![ae, id],
                )?;
            }

            if let Some(ref au) = req.author_url {
                conn.execute(
                    "UPDATE comments SET comment_author_url = ?1 WHERE comment_ID = ?2",
                    params![au, id],
                )?;
            }
        }
        self.get_comment_by_id(id)
    }

    pub fn get_comment_counts(&self) -> Result<CommentCounts> {
        let conn = self.conn.lock().unwrap();
        let mut counts = CommentCounts::default();
        let mut stmt = conn
            .prepare("SELECT comment_approved, COUNT(*) FROM comments GROUP BY comment_approved")?;
        let rows = stmt.query_map([], |r| {
            let status: String = r.get(0)?;
            let count: i64 = r.get(1)?;
            Ok((status, count))
        })?;

        for r in rows {
            let (status, count) = r?;
            counts.all += count;
            match status.as_str() {
                "1" | "approved" | "approve" => counts.approved += count,
                "0" | "hold" | "pending" => counts.pending += count,
                "spam" => counts.spam += count,
                "trash" => counts.trash += count,
                _ => {}
            }
        }
        Ok(counts)
    }

    pub fn update_comment_status(&self, id: i64, status: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let norm = match status.trim() {
            "approved" | "approve" | "1" => "1",
            "hold" | "pending" | "unapproved" | "0" => "0",
            other => other,
        };
        let rows = conn.execute(
            "UPDATE comments SET comment_approved = ?1 WHERE comment_ID = ?2",
            params![norm, id],
        )?;
        Ok(rows > 0)
    }

    pub fn delete_comment(&self, id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let rows = conn.execute("DELETE FROM comments WHERE comment_ID = ?1", params![id])?;
        Ok(rows > 0)
    }
}

impl crate::ports::output::CommentRepository for Database {
    fn list_comments(
        &self,
        _post_id: Option<i64>,
        status: Option<&str>,
        limit: i64,
    ) -> DomainResult<Vec<DaypressCommentAdminItem>> {
        self.get_comments(status, limit)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn find_comment_by_id(&self, id: i64) -> DomainResult<Option<DaypressCommentAdminItem>> {
        self.get_comment_by_id(id)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn create_comment(&self, req: &CreateCommentRequest) -> DomainResult<DaypressCommentResponse> {
        Database::create_comment(self, req).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn update_comment(
        &self,
        id: i64,
        req: &UpdateCommentRequest,
    ) -> DomainResult<Option<DaypressCommentAdminItem>> {
        Database::update_comment(self, id, req).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn delete_comment(&self, id: i64) -> DomainResult<bool> {
        Database::delete_comment(self, id).map_err(|e| DomainError::Storage(e.to_string()))
    }
}
