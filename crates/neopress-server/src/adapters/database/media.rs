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
    pub fn create_attachment(
        &self,
        title: &str,
        file_name: &str,
        mime_type: &str,
        file_size: u64,
        url: &str,
    ) -> Result<WpMediaResponse> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        conn.execute(
            r#"INSERT INTO wp_posts (post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type, post_mime_type)
               VALUES (1, ?1, ?1, '', ?2, '', 'inherit', ?3, ?1, ?1, ?4, 'attachment', ?5)"#,
            params![now, title, file_name, url, mime_type],
        )?;

        let id = conn.last_insert_rowid();

        let _ = conn.execute(
            "INSERT INTO wp_postmeta (post_id, meta_key, meta_value) VALUES (?1, '_filesize', ?2)",
            params![id, file_size.to_string()],
        );

        let _ = conn.execute(
            "INSERT INTO wp_postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_attached_file', ?2)",
            params![id, file_name],
        );

        let base_url = Self::get_base_url_with_conn(&conn);
        let disk_file_path = Path::new("uploads").join(file_name);
        let media_details =
            crate::media::process_image_metadata(&disk_file_path, file_name, &base_url)
                .unwrap_or_default();

        if let Ok(json_details) = serde_json::to_string(&media_details) {
            let _ = conn.execute(
                "INSERT INTO wp_postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_attachment_metadata', ?2)",
                params![id, json_details],
            );
        }

        // Spawn background non-blocking thumbnail & WebP generation
        crate::media::spawn_background_image_processing(disk_file_path);

        let media_type = if mime_type.starts_with("image") {
            "image".to_string()
        } else {
            "file".to_string()
        };

        Ok(WpMediaResponse {
            id,
            date: now.replace(' ', "T"),
            slug: file_name.to_string(),
            title: WpRenderedText {
                rendered: title.to_string(),
                protected: false,
            },
            source_url: url.to_string(),
            mime_type: mime_type.to_string(),
            file_size,
            media_type,
            alt_text: String::new(),
            media_details,
            caption: empty_rendered_text(),
            description: empty_rendered_text(),
            post: None,
        })
    }

    /// Shared SELECT for attachment rows (column order consumed by [`Self::map_media_row`]).
    const MEDIA_SELECT_SQL: &'static str = r#"SELECT p.ID, p.post_date, p.post_name, p.post_title, p.guid, p.post_mime_type,
                      COALESCE(m_sz.meta_value, '0'), COALESCE(m_meta.meta_value, ''), COALESCE(m_alt.meta_value, ''),
                      COALESCE(p.post_excerpt, ''), COALESCE(p.post_content, ''), COALESCE(p.post_parent, 0)
               FROM wp_posts p
               LEFT JOIN wp_postmeta m_sz ON m_sz.meta_id = (SELECT meta_id FROM wp_postmeta WHERE post_id = p.ID AND meta_key = '_filesize' LIMIT 1)
               LEFT JOIN wp_postmeta m_meta ON m_meta.meta_id = (SELECT meta_id FROM wp_postmeta WHERE post_id = p.ID AND meta_key = '_wp_attachment_metadata' LIMIT 1)
               LEFT JOIN wp_postmeta m_alt ON m_alt.meta_id = (SELECT meta_id FROM wp_postmeta WHERE post_id = p.ID AND meta_key = '_wp_attachment_image_alt' LIMIT 1)
               WHERE p.post_type = 'attachment'"#;

    fn map_media_row(row: &rusqlite::Row<'_>) -> Result<WpMediaResponse> {
        let id: i64 = row.get(0)?;
        let date: String = row.get(1)?;
        let slug: String = row.get(2)?;
        let title: String = row.get(3)?;
        let url: String = row.get(4)?;
        let mime: String = row.get(5)?;
        let size_str: String = row.get(6)?;
        let file_size: u64 = size_str.trim().parse().unwrap_or(0);
        let meta_json: String = row.get(7)?;
        let alt_text: String = row.get(8)?;
        let caption: String = row.get(9)?;
        let description: String = row.get(10)?;
        let parent: i64 = row.get(11)?;

        let media_details: MediaDetails = if !meta_json.is_empty() {
            serde_json::from_str(&meta_json).unwrap_or_default()
        } else {
            MediaDetails::default()
        };

        let media_type = if mime.starts_with("image") {
            "image".to_string()
        } else {
            "file".to_string()
        };

        Ok(WpMediaResponse {
            id,
            date: date.replace(' ', "T"),
            slug,
            title: WpRenderedText {
                rendered: title,
                protected: false,
            },
            source_url: url,
            mime_type: mime,
            file_size,
            media_type,
            alt_text,
            media_details,
            caption: WpRenderedText {
                rendered: caption,
                protected: false,
            },
            description: WpRenderedText {
                rendered: description,
                protected: false,
            },
            post: if parent > 0 { Some(parent) } else { None },
        })
    }

    pub fn get_media_attachments(&self) -> Result<Vec<WpMediaResponse>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!(
            "{} ORDER BY p.post_date DESC, p.ID DESC",
            Self::MEDIA_SELECT_SQL
        );
        let mut stmt = conn.prepare(&sql)?;
        let rows = stmt.query_map([], Self::map_media_row)?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_media_by_id(&self, id: i64) -> Result<Option<WpMediaResponse>> {
        let conn = self.conn.lock().unwrap();
        let sql = format!("{} AND p.ID = ?1", Self::MEDIA_SELECT_SQL);
        let mut stmt = conn.prepare(&sql)?;
        let mut rows = stmt.query_map(params![id], Self::map_media_row)?;
        match rows.next() {
            Some(r) => Ok(Some(r?)),
            None => Ok(None),
        }
    }

    pub fn update_media_attachment(
        &self,
        id: i64,
        req: &UpdateMediaRequest,
    ) -> Result<Option<WpMediaResponse>> {
        {
            let conn = self.conn.lock().unwrap();
            let exists: Option<i64> = conn
                .query_row(
                    "SELECT ID FROM wp_posts WHERE ID = ?1 AND post_type = 'attachment'",
                    params![id],
                    |r| r.get(0),
                )
                .ok();
            if exists.is_none() {
                return Ok(None);
            }

            if let Some(ref t_val) = req.title {
                let title_str = t_val
                    .as_str()
                    .map(|s| s.to_string())
                    .or_else(|| {
                        t_val
                            .get("raw")
                            .or_else(|| t_val.get("rendered"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_default();
                conn.execute(
                    "UPDATE wp_posts SET post_title = ?1 WHERE ID = ?2",
                    params![title_str, id],
                )?;
            }

            if let Some(ref alt) = req.alt_text {
                let _ = conn.execute(
                    "DELETE FROM wp_postmeta WHERE post_id = ?1 AND meta_key = '_wp_attachment_image_alt'",
                    params![id],
                );
                let _ = conn.execute(
                    "INSERT INTO wp_postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_attachment_image_alt', ?2)",
                    params![id, alt],
                );
            }

            if let Some(ref cap_val) = req.caption {
                if let Some(cap) = cap_val.as_str().or_else(|| {
                    cap_val
                        .get("raw")
                        .or_else(|| cap_val.get("rendered"))
                        .and_then(|v| v.as_str())
                }) {
                    conn.execute(
                        "UPDATE wp_posts SET post_excerpt = ?1 WHERE ID = ?2",
                        params![cap, id],
                    )?;
                }
            }

            if let Some(ref desc_val) = req.description {
                if let Some(desc) = desc_val.as_str().or_else(|| {
                    desc_val
                        .get("raw")
                        .or_else(|| desc_val.get("rendered"))
                        .and_then(|v| v.as_str())
                }) {
                    conn.execute(
                        "UPDATE wp_posts SET post_content = ?1 WHERE ID = ?2",
                        params![desc, id],
                    )?;
                }
            }
        }
        self.get_media_by_id(id)
    }

    pub fn delete_media_attachment(&self, id: i64) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let guid: Option<String> = conn
            .query_row(
                "SELECT guid FROM wp_posts WHERE ID = ?1 AND post_type = 'attachment'",
                params![id],
                |r| r.get(0),
            )
            .ok();

        let _ = conn.execute("DELETE FROM wp_postmeta WHERE post_id = ?1", params![id]);
        let rows = conn.execute(
            "DELETE FROM wp_posts WHERE ID = ?1 AND post_type = 'attachment'",
            params![id],
        )?;
        if rows > 0 {
            // Like WP core's wp_delete_attachment(): drop featured-image references to it.
            let _ = conn.execute(
                "DELETE FROM wp_postmeta WHERE meta_key = '_thumbnail_id' AND meta_value = ?1",
                params![id.to_string()],
            );
            Ok(guid)
        } else {
            Ok(None)
        }
    }
}
