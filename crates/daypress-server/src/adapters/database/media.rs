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
    ) -> Result<DaypressMediaResponse> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();

        conn.execute(
            r#"INSERT INTO posts (post_author, post_date, post_date_gmt, post_content, post_title, post_excerpt, post_status, post_name, post_modified, post_modified_gmt, guid, post_type, post_mime_type)
               VALUES (1, ?1, ?1, '', ?2, '', 'inherit', ?3, ?1, ?1, ?4, 'attachment', ?5)"#,
            params![now, title, file_name, url, mime_type],
        )?;

        let id = conn.last_insert_rowid();

        let _ = conn.execute(
            "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_filesize', ?2)",
            params![id, file_size.to_string()],
        );

        let _ = conn.execute(
            "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_attached_file', ?2)",
            params![id, file_name],
        );

        let base_url = Self::get_base_url_with_conn(&conn);
        let disk_file_path = Path::new("uploads").join(file_name);
        let media_details =
            crate::media::process_image_metadata(&disk_file_path, file_name, &base_url)
                .unwrap_or_default();

        if let Ok(json_details) = serde_json::to_string(&media_details) {
            let _ = conn.execute(
                "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_attachment_metadata', ?2)",
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

        Ok(DaypressMediaResponse {
            id,
            date: now.replace(' ', "T"),
            slug: file_name.to_string(),
            title: DaypressRenderedText {
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
            folder_id: None,
            tags: Vec::new(),
            focal_x: 0.5,
            focal_y: 0.5,
        })
    }

    /// Shared SELECT for attachment rows (column order consumed by [`Self::map_media_row`]).
    const MEDIA_SELECT_SQL: &'static str = r#"SELECT p.ID, p.post_date, p.post_name, p.post_title, p.guid, p.post_mime_type,
                      COALESCE(m_sz.meta_value, '0'), COALESCE(m_meta.meta_value, ''), COALESCE(m_alt.meta_value, ''),
                      COALESCE(p.post_excerpt, ''), COALESCE(p.post_content, ''), COALESCE(p.post_parent, 0),
                      COALESCE(m_fld.meta_value, ''), COALESCE(m_tags.meta_value, ''), COALESCE(m_fp.meta_value, '')
               FROM posts p
               LEFT JOIN postmeta m_sz ON m_sz.meta_id = (SELECT meta_id FROM postmeta WHERE post_id = p.ID AND meta_key = '_filesize' LIMIT 1)
               LEFT JOIN postmeta m_meta ON m_meta.meta_id = (SELECT meta_id FROM postmeta WHERE post_id = p.ID AND meta_key = '_wp_attachment_metadata' LIMIT 1)
               LEFT JOIN postmeta m_alt ON m_alt.meta_id = (SELECT meta_id FROM postmeta WHERE post_id = p.ID AND meta_key = '_wp_attachment_image_alt' LIMIT 1)
               LEFT JOIN postmeta m_fld ON m_fld.meta_id = (SELECT meta_id FROM postmeta WHERE post_id = p.ID AND meta_key = '_daypress_media_folder' LIMIT 1)
               LEFT JOIN postmeta m_tags ON m_tags.meta_id = (SELECT meta_id FROM postmeta WHERE post_id = p.ID AND meta_key = '_daypress_media_tags' LIMIT 1)
               LEFT JOIN postmeta m_fp ON m_fp.meta_id = (SELECT meta_id FROM postmeta WHERE post_id = p.ID AND meta_key = '_daypress_focal_point' LIMIT 1)
               WHERE p.post_type = 'attachment'"#;

    fn map_media_row(row: &rusqlite::Row<'_>) -> Result<DaypressMediaResponse> {
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
        let folder_str: String = row.get(12).unwrap_or_default();
        let tags_str: String = row.get(13).unwrap_or_default();
        let focal_str: String = row.get(14).unwrap_or_default();

        let mut media_details: MediaDetails = if !meta_json.is_empty() {
            serde_json::from_str(&meta_json).unwrap_or_default()
        } else {
            MediaDetails::default()
        };

        let (focal_x, focal_y) = if let Some((fx_s, fy_s)) = focal_str.split_once(',') {
            let fx = fx_s
                .trim()
                .parse::<f32>()
                .unwrap_or(media_details.focal_x)
                .clamp(0.0, 1.0);
            let fy = fy_s
                .trim()
                .parse::<f32>()
                .unwrap_or(media_details.focal_y)
                .clamp(0.0, 1.0);
            (fx, fy)
        } else {
            (
                media_details.focal_x.clamp(0.0, 1.0),
                media_details.focal_y.clamp(0.0, 1.0),
            )
        };
        media_details.focal_x = focal_x;
        media_details.focal_y = focal_y;

        let folder_id = folder_str.trim().parse::<i64>().ok().filter(|&fid| fid > 0);
        let tags: Vec<String> = tags_str
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        let media_type = if mime.starts_with("image") {
            "image".to_string()
        } else {
            "file".to_string()
        };

        Ok(DaypressMediaResponse {
            id,
            date: date.replace(' ', "T"),
            slug,
            title: DaypressRenderedText {
                rendered: title,
                protected: false,
            },
            source_url: url,
            mime_type: mime,
            file_size,
            media_type,
            alt_text,
            media_details,
            caption: DaypressRenderedText {
                rendered: caption,
                protected: false,
            },
            description: DaypressRenderedText {
                rendered: description,
                protected: false,
            },
            post: if parent > 0 { Some(parent) } else { None },
            folder_id,
            tags,
            focal_x,
            focal_y,
        })
    }

    pub fn get_media_attachments(&self) -> Result<Vec<DaypressMediaResponse>> {
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

    pub fn get_media_by_id(&self, id: i64) -> Result<Option<DaypressMediaResponse>> {
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
    ) -> Result<Option<DaypressMediaResponse>> {
        let mut regen_focal: Option<(PathBuf, f32, f32)> = None;
        {
            let conn = self.conn.lock().unwrap();
            let exists: Option<i64> = conn
                .query_row(
                    "SELECT ID FROM posts WHERE ID = ?1 AND post_type = 'attachment'",
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
                    "UPDATE posts SET post_title = ?1 WHERE ID = ?2",
                    params![title_str, id],
                )?;
            }

            if let Some(ref alt) = req.alt_text {
                let _ = conn.execute(
                    "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_wp_attachment_image_alt'",
                    params![id],
                );
                let _ = conn.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_attachment_image_alt', ?2)",
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
                        "UPDATE posts SET post_excerpt = ?1 WHERE ID = ?2",
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
                        "UPDATE posts SET post_content = ?1 WHERE ID = ?2",
                        params![desc, id],
                    )?;
                }
            }

            if let Some(folder_id) = req.folder_id {
                let _ = conn.execute(
                    "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_media_folder'",
                    params![id],
                );
                if folder_id > 0 {
                    let _ = conn.execute(
                        "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_media_folder', ?2)",
                        params![id, folder_id.to_string()],
                    );
                }
            }

            if let Some(ref tags_vec) = req.tags {
                let cleaned_tags = tags_vec
                    .iter()
                    .map(|t| t.trim())
                    .filter(|t| !t.is_empty())
                    .collect::<Vec<_>>()
                    .join(",");
                let _ = conn.execute(
                    "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_media_tags'",
                    params![id],
                );
                if !cleaned_tags.is_empty() {
                    let _ = conn.execute(
                        "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_media_tags', ?2)",
                        params![id, cleaned_tags],
                    );
                }
            }

            if req.focal_x.is_some() || req.focal_y.is_some() {
                let existing_meta_json: String = conn
                    .query_row(
                        "SELECT COALESCE(meta_value, '') FROM postmeta WHERE post_id = ?1 AND meta_key = '_wp_attachment_metadata' LIMIT 1",
                        params![id],
                        |r| r.get(0),
                    )
                    .unwrap_or_default();
                let mut media_details: MediaDetails = if !existing_meta_json.is_empty() {
                    serde_json::from_str(&existing_meta_json).unwrap_or_default()
                } else {
                    MediaDetails::default()
                };

                let existing_fp: String = conn
                    .query_row(
                        "SELECT COALESCE(meta_value, '') FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_focal_point' LIMIT 1",
                        params![id],
                        |r| r.get(0),
                    )
                    .unwrap_or_default();
                let (cur_fx, cur_fy) = if let Some((fx_s, fy_s)) = existing_fp.split_once(',') {
                    (
                        fx_s.trim().parse::<f32>().unwrap_or(media_details.focal_x),
                        fy_s.trim().parse::<f32>().unwrap_or(media_details.focal_y),
                    )
                } else {
                    (media_details.focal_x, media_details.focal_y)
                };

                let fx = req.focal_x.unwrap_or(cur_fx).clamp(0.0, 1.0);
                let fy = req.focal_y.unwrap_or(cur_fy).clamp(0.0, 1.0);
                media_details.focal_x = fx;
                media_details.focal_y = fy;

                let fp_val = format!("{:.4},{:.4}", fx, fy);
                let _ = conn.execute(
                    "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_focal_point'",
                    params![id],
                );
                let _ = conn.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_daypress_focal_point', ?2)",
                    params![id, fp_val],
                );

                if let Ok(new_meta_json) = serde_json::to_string(&media_details) {
                    let _ = conn.execute(
                        "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_wp_attachment_metadata'",
                        params![id],
                    );
                    let _ = conn.execute(
                        "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_attachment_metadata', ?2)",
                        params![id, new_meta_json],
                    );
                }

                let attached_file: String = conn
                    .query_row(
                        "SELECT COALESCE(meta_value, '') FROM postmeta WHERE post_id = ?1 AND meta_key = '_wp_attached_file' LIMIT 1",
                        params![id],
                        |r| r.get(0),
                    )
                    .unwrap_or_default();
                if !attached_file.is_empty() {
                    regen_focal = Some((Path::new("uploads").join(attached_file), fx, fy));
                }
            }
        }
        if let Some((path, fx, fy)) = regen_focal {
            crate::media::spawn_background_image_processing_with_focal(path, fx, fy);
        }
        self.get_media_by_id(id)
    }

    pub fn list_media_folders(&self) -> Result<Vec<MediaFolderDto>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            r#"SELECT f.id, f.name, f.slug, f.parent_id, f.created_at,
                      (SELECT COUNT(*) FROM postmeta pm
                       JOIN posts p ON p.ID = pm.post_id
                       WHERE p.post_type = 'attachment'
                         AND pm.meta_key = '_daypress_media_folder'
                         AND pm.meta_value = CAST(f.id AS TEXT)) AS cnt
               FROM daypress_media_folders f
               ORDER BY f.name ASC, f.id ASC"#,
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(MediaFolderDto {
                id: r.get(0)?,
                name: r.get(1)?,
                slug: r.get(2)?,
                parent_id: r.get(3)?,
                created_at: r.get(4)?,
                count: r.get(5)?,
            })
        })?;
        let mut folders = Vec::new();
        for row in rows {
            folders.push(row?);
        }
        Ok(folders)
    }

    pub fn create_media_folder(&self, name: &str, parent_id: i64) -> Result<MediaFolderDto> {
        let conn = self.conn.lock().unwrap();
        let clean_name = if name.trim().is_empty() {
            "New Folder"
        } else {
            name.trim()
        };
        let base_slug: String = clean_name
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        let base_slug = if base_slug.is_empty() {
            "folder".to_string()
        } else {
            base_slug
        };

        let mut slug = base_slug.clone();
        let mut suffix = 2u32;
        loop {
            let exists: Option<i64> = conn
                .query_row(
                    "SELECT id FROM daypress_media_folders WHERE slug = ?1 LIMIT 1",
                    params![slug],
                    |r| r.get(0),
                )
                .ok();
            if exists.is_none() {
                break;
            }
            slug = format!("{}-{}", base_slug, suffix);
            suffix += 1;
        }

        let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO daypress_media_folders (name, slug, parent_id, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![clean_name, slug, parent_id, now],
        )?;
        let id = conn.last_insert_rowid();

        Ok(MediaFolderDto {
            id,
            name: clean_name.to_string(),
            slug,
            parent_id,
            count: 0,
            created_at: now,
        })
    }

    pub fn delete_media_folder(&self, folder_id: i64) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let _ = conn.execute(
            "DELETE FROM postmeta WHERE meta_key = '_daypress_media_folder' AND meta_value = ?1",
            params![folder_id.to_string()],
        );
        let _ = conn.execute(
            "UPDATE daypress_media_folders SET parent_id = 0 WHERE parent_id = ?1",
            params![folder_id],
        );
        let rows = conn.execute(
            "DELETE FROM daypress_media_folders WHERE id = ?1",
            params![folder_id],
        )?;
        Ok(rows > 0)
    }

    /// Replaces an existing media attachment's underlying binary file in-place without changing its ID or URL.
    pub fn replace_media_attachment(
        &self,
        id: i64,
        new_bytes: &[u8],
        new_mime_type: Option<&str>,
    ) -> Result<Option<DaypressMediaResponse>> {
        let (file_name, existing_mime, focal_x, focal_y, base_url) = {
            let conn = self.conn.lock().unwrap();
            let post_row: Option<(String, String, String)> = conn
                .query_row(
                    "SELECT post_name, guid, post_mime_type FROM posts WHERE ID = ?1 AND post_type = 'attachment'",
                    params![id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .ok();
            let (post_name, guid, existing_mime) = match post_row {
                Some(row) => row,
                None => return Ok(None),
            };

            let attached_file: String = conn
                .query_row(
                    "SELECT COALESCE(meta_value, '') FROM postmeta WHERE post_id = ?1 AND meta_key = '_wp_attached_file' LIMIT 1",
                    params![id],
                    |r| r.get(0),
                )
                .unwrap_or_default();

            let derived_name = if !attached_file.trim().is_empty() {
                attached_file.trim().to_string()
            } else if let Some((_, after)) = guid.rsplit_once("/uploads/") {
                after.trim().to_string()
            } else if !post_name.trim().is_empty() {
                post_name.trim().to_string()
            } else {
                format!("attachment_{}.bin", id)
            };

            let fp_str: String = conn
                .query_row(
                    "SELECT COALESCE(meta_value, '') FROM postmeta WHERE post_id = ?1 AND meta_key = '_daypress_focal_point' LIMIT 1",
                    params![id],
                    |r| r.get(0),
                )
                .unwrap_or_default();
            let (fx, fy) = if let Some((fx_s, fy_s)) = fp_str.split_once(',') {
                (
                    fx_s.trim().parse::<f32>().unwrap_or(0.5).clamp(0.0, 1.0),
                    fy_s.trim().parse::<f32>().unwrap_or(0.5).clamp(0.0, 1.0),
                )
            } else {
                (0.5, 0.5)
            };

            let base_url = Self::get_base_url_with_conn(&conn);
            (derived_name, existing_mime, fx, fy, base_url)
        };

        let is_svg = file_name.to_ascii_lowercase().ends_with(".svg")
            || existing_mime.eq_ignore_ascii_case("image/svg+xml")
            || new_mime_type
                .map(|m| m.eq_ignore_ascii_case("image/svg+xml"))
                .unwrap_or(false);

        let final_bytes: Vec<u8> = if is_svg {
            crate::media::sanitize_svg_bytes(new_bytes).map_err(|e| {
                rusqlite::Error::ToSqlConversionFailure(Box::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    e.to_string(),
                )))
            })?
        } else {
            new_bytes.to_vec()
        };

        let uploads_dir = Path::new("uploads");
        let _ = fs::create_dir_all(uploads_dir);
        let disk_file_path = uploads_dir.join(&file_name);
        if let Some(parent) = disk_file_path.parent() {
            let _ = fs::create_dir_all(parent);
            // Remove stale cached variant files ({stem}-*x*.*, {stem}.webp, {stem}.avif)
            if let Some(stem) = Path::new(&file_name).file_stem().and_then(|s| s.to_str()) {
                let webp_variant = format!("{}.webp", stem);
                let avif_variant = format!("{}.avif", stem);
                let crop_prefix = format!("{}-", stem);
                if let Ok(entries) = fs::read_dir(parent) {
                    for entry in entries.flatten() {
                        if let Some(ename) = entry.file_name().to_str() {
                            if ename == file_name {
                                continue;
                            }
                            if ename == webp_variant
                                || ename == avif_variant
                                || ename.starts_with(&crop_prefix)
                            {
                                let _ = fs::remove_file(entry.path());
                            }
                        }
                    }
                }
            }
        }

        let _ = fs::write(&disk_file_path, &final_bytes);
        let file_size = final_bytes.len() as u64;
        let media_details = crate::media::process_image_metadata_with_focal(
            &disk_file_path,
            &file_name,
            &base_url,
            focal_x,
            focal_y,
        )
        .unwrap_or_default();

        {
            let conn = self.conn.lock().unwrap();
            let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let effective_mime = new_mime_type
                .filter(|m| !m.trim().is_empty())
                .unwrap_or(&existing_mime);

            conn.execute(
                "UPDATE posts SET post_modified = ?1, post_modified_gmt = ?1, post_mime_type = ?2 WHERE ID = ?3",
                params![now, effective_mime, id],
            )?;

            let _ = conn.execute(
                "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_filesize'",
                params![id],
            );
            let _ = conn.execute(
                "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_filesize', ?2)",
                params![id, file_size.to_string()],
            );

            if let Ok(json_details) = serde_json::to_string(&media_details) {
                let _ = conn.execute(
                    "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = '_wp_attachment_metadata'",
                    params![id],
                );
                let _ = conn.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, '_wp_attachment_metadata', ?2)",
                    params![id, json_details],
                );
            }
        }

        crate::media::spawn_background_image_processing_with_focal(
            disk_file_path,
            focal_x,
            focal_y,
        );
        self.invalidate_hot_cache("media_replaced");
        self.get_media_by_id(id)
    }

    /// Scans the database for unused media attachments that are not referenced by any
    /// featured image (`_thumbnail_id`), post content/layout AST, template layout AST, or option value.
    pub fn scan_unused_media_attachments(&self) -> Result<Vec<DaypressMediaResponse>> {
        let all_media = self.get_media_attachments()?;
        if all_media.is_empty() {
            return Ok(Vec::new());
        }

        let mut referenced_ids = std::collections::HashSet::<i64>::new();
        let mut text_corpus = Vec::<String>::new();

        {
            let conn = self.conn.lock().unwrap();

            // 1. postmeta where meta_key = '_thumbnail_id'
            if let Ok(mut stmt) = conn.prepare(
                "SELECT COALESCE(meta_value, '') FROM postmeta WHERE meta_key = '_thumbnail_id'",
            ) {
                if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                    for val in rows.flatten() {
                        let trimmed = val.trim();
                        if let Ok(tid) = trimmed.parse::<i64>() {
                            referenced_ids.insert(tid);
                        } else if !trimmed.is_empty() {
                            text_corpus.push(trimmed.to_string());
                        }
                    }
                }
            }

            // 2. posts.post_content and posts.layout_json (for non-trash posts/pages)
            if let Ok(mut stmt) = conn.prepare(
                "SELECT COALESCE(post_content, ''), COALESCE(layout_json, '') FROM posts WHERE post_type != 'attachment' AND post_status != 'trash'",
            ) {
                if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))) {
                    for (content, layout) in rows.flatten() {
                        if !content.is_empty() {
                            text_corpus.push(content);
                        }
                        if !layout.is_empty() {
                            text_corpus.push(layout);
                        }
                    }
                }
            }

            // 3. daypress_templates.layout_json
            if let Ok(mut stmt) = conn.prepare(
                "SELECT COALESCE(layout_json, ''), COALESCE(settings_json, '') FROM daypress_templates",
            ) {
                if let Ok(rows) = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))) {
                    for (layout, settings) in rows.flatten() {
                        if !layout.is_empty() {
                            text_corpus.push(layout);
                        }
                        if !settings.is_empty() {
                            text_corpus.push(settings);
                        }
                    }
                }
            }

            // 4. options.option_value (e.g. site_logo, site_favicon, daystudio_header_html, daystudio_footer_html)
            if let Ok(mut stmt) = conn
                .prepare("SELECT COALESCE(option_value, '') FROM options WHERE option_value != ''")
            {
                if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                    for val in rows.flatten() {
                        let trimmed = val.trim();
                        if let Ok(oid) = trimmed.parse::<i64>() {
                            referenced_ids.insert(oid);
                        }
                        if !trimmed.is_empty() {
                            text_corpus.push(trimmed.to_string());
                        }
                    }
                }
            };
        }

        let unused = all_media
            .into_iter()
            .filter(|m| {
                if referenced_ids.contains(&m.id) {
                    return false;
                }
                let file_basename = m.source_url.rsplit('/').next().unwrap_or("");
                let wp_image_marker = format!("wp-image-{}", m.id);
                let json_id_marker = format!("\"attachment_id\":{}", m.id);

                let is_referenced = text_corpus.iter().any(|haystack| {
                    (!m.source_url.is_empty() && haystack.contains(&m.source_url))
                        || (!m.slug.is_empty() && haystack.contains(&m.slug))
                        || (!file_basename.is_empty() && haystack.contains(file_basename))
                        || haystack.contains(&wp_image_marker)
                        || haystack.contains(&json_id_marker)
                });
                !is_referenced
            })
            .collect();

        Ok(unused)
    }

    pub fn delete_media_attachment(&self, id: i64) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let guid: Option<String> = conn
            .query_row(
                "SELECT guid FROM posts WHERE ID = ?1 AND post_type = 'attachment'",
                params![id],
                |r| r.get(0),
            )
            .ok();

        let _ = conn.execute("DELETE FROM postmeta WHERE post_id = ?1", params![id]);
        let rows = conn.execute(
            "DELETE FROM posts WHERE ID = ?1 AND post_type = 'attachment'",
            params![id],
        )?;
        if rows > 0 {
            // Like WP core's wp_delete_attachment(): drop featured-image references to it.
            let _ = conn.execute(
                "DELETE FROM postmeta WHERE meta_key = '_thumbnail_id' AND meta_value = ?1",
                params![id.to_string()],
            );
            Ok(guid)
        } else {
            Ok(None)
        }
    }
}
