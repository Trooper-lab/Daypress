//! Multilingual (`i18n`) Languages, Translation Groups, `hreflang` Resolution, and
//! WPML / Polylang Migration Converters (`adapters/database/i18n_repo.rs`).

use super::Database;
use crate::i18n::preset_language_info;
use crate::models::{DaypressLanguage, DaypressPostResponse, HreflangLink};
use rusqlite::{params, Connection, OptionalExtension, Result};
use std::collections::HashMap;

impl Database {
    pub(crate) fn list_languages_with_conn(conn: &Connection) -> Vec<DaypressLanguage> {
        let mut out = Vec::new();
        if let Ok(mut stmt) = conn.prepare(
            "SELECT code, locale, name, flag_emoji, is_default, is_rtl, sort_order
             FROM daypress_languages
             ORDER BY is_default DESC, sort_order ASC, code ASC",
        ) {
            if let Ok(rows) = stmt.query_map([], |r| {
                let name: String = r.get(2)?;
                Ok(DaypressLanguage {
                    code: r.get(0)?,
                    locale: r.get(1)?,
                    native_name: name.clone(),
                    name,
                    flag_emoji: r.get(3)?,
                    is_default: r.get::<_, i64>(4).unwrap_or(0) != 0,
                    is_rtl: r.get::<_, i64>(5).unwrap_or(0) != 0,
                    is_active: true,
                    sort_order: r.get(6).unwrap_or(0),
                })
            }) {
                for row in rows.flatten() {
                    out.push(row);
                }
            }
        }
        if out.is_empty() {
            out.push(DaypressLanguage {
                code: "en".to_string(),
                locale: "en_US".to_string(),
                name: "English".to_string(),
                native_name: "English".to_string(),
                flag_emoji: "🇺🇸".to_string(),
                is_default: true,
                is_rtl: false,
                is_active: true,
                sort_order: 0,
            });
        }
        out
    }

    pub fn list_languages(&self) -> Result<Vec<DaypressLanguage>> {
        let conn = self.conn.lock().unwrap();
        Ok(Self::list_languages_with_conn(&conn))
    }

    pub(crate) fn get_default_language_code_with_conn(conn: &Connection) -> String {
        conn.query_row(
            "SELECT code FROM daypress_languages WHERE is_default = 1 ORDER BY sort_order ASC LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .ok()
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| "en".to_string())
    }

    pub fn get_default_language_code(&self) -> String {
        let Ok(conn) = self.conn.lock() else {
            return "en".to_string();
        };
        Self::get_default_language_code_with_conn(&conn)
    }

    pub(crate) fn prefix_default_language_enabled_with_conn(conn: &Connection) -> bool {
        conn.query_row(
            "SELECT option_value FROM options WHERE option_name = 'daypress_i18n_prefix_default' LIMIT 1",
            [],
            |r| r.get::<_, String>(0),
        )
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    }

    pub fn prefix_default_language_enabled(&self) -> bool {
        self.get_option("daypress_i18n_prefix_default")
            .ok()
            .flatten()
            .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
    }

    pub fn set_prefix_default_language(&self, enabled: bool) -> Result<()> {
        self.set_option(
            "daypress_i18n_prefix_default",
            if enabled { "1" } else { "0" },
        )?;
        self.invalidate_hot_cache("i18n_prefix_default");
        Ok(())
    }

    pub fn is_registered_language_code(&self, code: &str) -> bool {
        let clean = code.trim().to_ascii_lowercase();
        if clean.is_empty() || clean.len() > 8 {
            return false;
        }
        let Ok(conn) = self.conn.lock() else {
            return false;
        };
        conn.query_row(
            "SELECT 1 FROM daypress_languages WHERE LOWER(code) = ?1 LIMIT 1",
            params![clean],
            |_| Ok(true),
        )
        .unwrap_or(false)
    }

    pub fn upsert_language(&self, req: &DaypressLanguage) -> Result<Vec<DaypressLanguage>> {
        let code: String = req
            .code
            .trim()
            .to_ascii_lowercase()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
            .take(8)
            .collect();
        if code.is_empty() {
            return self.list_languages();
        }
        let (preset_locale, preset_name, preset_flag, preset_rtl) = preset_language_info(&code);
        let locale = if req.locale.trim().is_empty() {
            preset_locale.to_string()
        } else {
            req.locale.trim().to_string()
        };
        let name = if req.name.trim().is_empty() {
            preset_name.to_string()
        } else {
            req.name.trim().to_string()
        };
        let flag = if req.flag_emoji.trim().is_empty() {
            preset_flag.to_string()
        } else {
            req.flag_emoji.trim().to_string()
        };
        let is_rtl = if req.is_rtl || preset_rtl { 1 } else { 0 };

        {
            let conn = self.conn.lock().unwrap();
            if req.is_default {
                let _ = conn.execute("UPDATE daypress_languages SET is_default = 0", []);
            }
            conn.execute(
                r#"INSERT INTO daypress_languages (code, locale, name, flag_emoji, is_default, is_rtl, sort_order)
                   VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                   ON CONFLICT(code) DO UPDATE SET
                       locale = excluded.locale,
                       name = excluded.name,
                       flag_emoji = excluded.flag_emoji,
                       is_default = CASE WHEN excluded.is_default = 1 THEN 1 ELSE daypress_languages.is_default END,
                       is_rtl = excluded.is_rtl,
                       sort_order = excluded.sort_order"#,
                params![
                    code,
                    locale,
                    name,
                    flag,
                    if req.is_default { 1 } else { 0 },
                    is_rtl,
                    req.sort_order
                ],
            )?;
        }
        self.invalidate_hot_cache("upsert_language");
        self.list_languages()
    }

    pub fn set_default_language(&self, code: &str) -> Result<()> {
        let clean = code.trim().to_ascii_lowercase();
        let conn = self.conn.lock().unwrap();
        let exists: bool = conn
            .query_row(
                "SELECT 1 FROM daypress_languages WHERE code = ?1",
                params![clean],
                |_| Ok(true),
            )
            .unwrap_or(false);
        if exists {
            conn.execute("UPDATE daypress_languages SET is_default = 0", [])?;
            conn.execute(
                "UPDATE daypress_languages SET is_default = 1 WHERE code = ?1",
                params![clean],
            )?;
        }
        drop(conn);
        self.invalidate_hot_cache("set_default_language");
        Ok(())
    }

    pub fn delete_language(&self, code: &str) -> Result<bool> {
        let clean = code.trim().to_ascii_lowercase();
        let conn = self.conn.lock().unwrap();
        let is_def: bool = conn
            .query_row(
                "SELECT is_default FROM daypress_languages WHERE code = ?1",
                params![clean],
                |r| Ok(r.get::<_, i64>(0).unwrap_or(0) != 0),
            )
            .unwrap_or(false);
        if is_def {
            return Ok(false);
        }
        let deleted = conn.execute(
            "DELETE FROM daypress_languages WHERE code = ?1",
            params![clean],
        )?;
        drop(conn);
        self.invalidate_hot_cache("delete_language");
        Ok(deleted > 0)
    }

    pub(crate) fn ensure_language_registered_with_conn(
        conn: &Connection,
        raw_code: &str,
    ) -> String {
        let code: String = raw_code
            .trim()
            .to_ascii_lowercase()
            .split(['_', '-'])
            .next()
            .unwrap_or("en")
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .take(8)
            .collect();
        let code = if code.is_empty() {
            "en".to_string()
        } else {
            code
        };
        let (locale, name, flag, is_rtl) = preset_language_info(&code);
        let _ = conn.execute(
            r#"INSERT OR IGNORE INTO daypress_languages (code, locale, name, flag_emoji, is_default, is_rtl, sort_order)
               VALUES (?1, ?2, ?3, ?4, 0, ?5, 10)"#,
            params![code, locale, name, flag, if is_rtl { 1 } else { 0 }],
        );
        code
    }

    pub(crate) fn link_post_translation_with_conn(
        conn: &Connection,
        post_id: i64,
        lang_code: &str,
        group_id: Option<&str>,
    ) -> Result<String> {
        let code = Self::ensure_language_registered_with_conn(conn, lang_code);
        let existing_grp: Option<String> = conn
            .query_row(
                "SELECT translation_group_id FROM daypress_translations WHERE object_type = 'post' AND object_id = ?1",
                params![post_id],
                |r| r.get(0),
            )
            .optional()?;
        let grp = group_id
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .or(existing_grp)
            .unwrap_or_else(|| format!("trgrp_post_{}", post_id));

        // Remove any other post in the same translation group claiming the same lang_code
        let _ = conn.execute(
            "DELETE FROM daypress_translations WHERE object_type = 'post' AND translation_group_id = ?1 AND lang_code = ?2 AND object_id != ?3",
            params![grp, code, post_id],
        );

        conn.execute(
            r#"INSERT INTO daypress_translations (object_type, object_id, lang_code, translation_group_id)
               VALUES ('post', ?1, ?2, ?3)
               ON CONFLICT(object_type, object_id) DO UPDATE SET
                   lang_code = excluded.lang_code,
                   translation_group_id = excluded.translation_group_id"#,
            params![post_id, code, grp],
        )?;
        Ok(grp)
    }

    pub fn link_post_translation(
        &self,
        post_id: i64,
        lang_code: &str,
        group_id: Option<&str>,
    ) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let grp = Self::link_post_translation_with_conn(&conn, post_id, lang_code, group_id)?;
        drop(conn);
        self.invalidate_hot_cache("link_post_translation");
        Ok(grp)
    }

    /// Links a map of `{ "en": 10, "de": 11, "nl": 12 }` into a single shared `translation_group_id`.
    pub fn link_translation_map(
        &self,
        object_type: &str,
        group_id: Option<&str>,
        mappings: &HashMap<String, i64>,
    ) -> Result<String> {
        let obj_type = if object_type.trim().is_empty() {
            "post"
        } else {
            object_type.trim()
        };
        let conn = self.conn.lock().unwrap();
        // Discover if any of the mapped objects already belong to a group
        let mut resolved_grp = group_id
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from);
        if resolved_grp.is_none() {
            for &obj_id in mappings.values() {
                if let Some(existing) = conn
                    .query_row(
                        "SELECT translation_group_id FROM daypress_translations WHERE object_type = ?1 AND object_id = ?2",
                        params![obj_type, obj_id],
                        |r| r.get::<_, String>(0),
                    )
                    .optional()?
                {
                    if !existing.trim().is_empty() {
                        resolved_grp = Some(existing);
                        break;
                    }
                }
            }
        }
        let min_id = mappings.values().copied().min().unwrap_or(1);
        let grp = resolved_grp.unwrap_or_else(|| format!("trgrp_{}_{}", obj_type, min_id));

        for (raw_lang, &obj_id) in mappings {
            if obj_id <= 0 {
                continue;
            }
            let code = Self::ensure_language_registered_with_conn(&conn, raw_lang);
            let _ = conn.execute(
                "DELETE FROM daypress_translations WHERE object_type = ?1 AND translation_group_id = ?2 AND lang_code = ?3 AND object_id != ?4",
                params![obj_type, grp, code, obj_id],
            );
            conn.execute(
                r#"INSERT INTO daypress_translations (object_type, object_id, lang_code, translation_group_id)
                   VALUES (?1, ?2, ?3, ?4)
                   ON CONFLICT(object_type, object_id) DO UPDATE SET
                       lang_code = excluded.lang_code,
                       translation_group_id = excluded.translation_group_id"#,
                params![obj_type, obj_id, code, grp],
            )?;
        }
        drop(conn);
        self.invalidate_hot_cache("link_translation_map");
        Ok(grp)
    }

    pub fn get_translation_group_map(
        &self,
        object_type: &str,
        group_id: &str,
    ) -> Result<HashMap<String, i64>> {
        let obj_type = if object_type.trim().is_empty() {
            "post"
        } else {
            object_type.trim()
        };
        let conn = self.conn.lock().unwrap();
        let mut map = HashMap::new();
        if let Ok(mut stmt) = conn.prepare_cached(
            "SELECT lang_code, object_id FROM daypress_translations WHERE object_type = ?1 AND translation_group_id = ?2",
        ) {
            if let Ok(rows) = stmt.query_map(params![obj_type, group_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            }) {
                for pair in rows.flatten() {
                    map.insert(pair.0, pair.1);
                }
            }
        }
        Ok(map)
    }

    pub(crate) fn get_post_translation_info_with_conn(
        conn: &Connection,
        post_id: i64,
    ) -> (String, String, HashMap<String, i64>) {
        let default_lang = Self::get_default_language_code_with_conn(conn);
        let row: Option<(String, String)> = conn
            .query_row(
                "SELECT lang_code, translation_group_id FROM daypress_translations WHERE object_type = 'post' AND object_id = ?1",
                params![post_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .ok();
        let Some((lang_code, grp_id)) = row else {
            let mut single = HashMap::new();
            single.insert(default_lang.clone(), post_id);
            return (default_lang, String::new(), single);
        };

        let mut map = HashMap::new();
        if let Ok(mut stmt) = conn.prepare_cached(
            "SELECT lang_code, object_id FROM daypress_translations WHERE object_type = 'post' AND translation_group_id = ?1",
        ) {
            if let Ok(rows) = stmt.query_map(params![&grp_id], |r| {
                Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?))
            }) {
                for pair in rows.flatten() {
                    map.insert(pair.0, pair.1);
                }
            }
        }
        if map.is_empty() {
            map.insert(lang_code.clone(), post_id);
        }
        (lang_code, grp_id, map)
    }

    /// Formats the canonical URL for a post given its language code, slug, post type, and whether it's the front page.
    pub(crate) fn format_localized_post_url_with_conn(
        conn: &Connection,
        base_url: &str,
        post_id: i64,
        post_type: &str,
        slug: &str,
        lang_code: &str,
        is_front_page: bool,
    ) -> String {
        let base_trimmed = base_url.trim_end_matches('/');
        let default_lang = Self::get_default_language_code_with_conn(conn);
        let prefix_default = Self::prefix_default_language_enabled_with_conn(conn);
        let needs_lang_prefix = !lang_code.is_empty()
            && (!lang_code.eq_ignore_ascii_case(&default_lang) || prefix_default);

        // Also check if this post is a translated version of the front page
        let is_translated_front_page = if is_front_page {
            true
        } else if post_type == "page" {
            let page_on_front: i64 = conn
                .query_row(
                    "SELECT option_value FROM options WHERE option_name = 'page_on_front' LIMIT 1",
                    [],
                    |r| r.get::<_, String>(0),
                )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            if page_on_front > 0 {
                let (_, front_grp, front_map) =
                    Self::get_post_translation_info_with_conn(conn, page_on_front);
                !front_grp.is_empty() && front_map.values().any(|&id| id == post_id)
            } else {
                false
            }
        } else {
            false
        };

        let lang_seg = if needs_lang_prefix {
            format!("/{}", lang_code.trim_matches('/'))
        } else {
            String::new()
        };

        if is_translated_front_page {
            return format!("{}{}/", base_trimmed, lang_seg);
        }

        if !matches!(
            post_type,
            "post" | "page" | "attachment" | "nav_menu_item" | "revision"
        ) && !post_type.is_empty()
        {
            let rewrite_prefix: String = conn
                .query_row(
                    "SELECT rewrite_slug FROM daypress_post_types WHERE slug = ?1",
                    params![post_type],
                    |r| r.get::<_, String>(0),
                )
                .ok()
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| post_type.to_string());
            format!("{}{}/{}/{}/", base_trimmed, lang_seg, rewrite_prefix, slug)
        } else {
            format!("{}{}/{}/", base_trimmed, lang_seg, slug)
        }
    }

    /// Returns all `<link rel="alternate" hreflang="..." href="..." />` entries for a post.
    pub fn build_post_hreflang_links(&self, post_id: i64) -> Vec<HreflangLink> {
        let Ok(conn) = self.conn.lock() else {
            return Vec::new();
        };
        let base_url = Self::get_base_url_with_conn(&conn);
        let languages = Self::list_languages_with_conn(&conn);
        let default_lang = Self::get_default_language_code_with_conn(&conn);
        let (_, grp_id, trans_map) = Self::get_post_translation_info_with_conn(&conn, post_id);
        if grp_id.is_empty() && languages.len() <= 1 {
            return Vec::new();
        }

        let mut links = Vec::new();
        let mut default_href: Option<String> = None;

        for lang in &languages {
            let Some(&target_post_id) = trans_map.get(&lang.code) else {
                continue;
            };
            let row: Option<(String, String, String)> = conn
                .query_row(
                    "SELECT post_name, post_type, post_status FROM posts WHERE ID = ?1",
                    params![target_post_id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                )
                .ok();
            let Some((slug, ptype, status)) = row else {
                continue;
            };
            if status != "publish" && target_post_id != post_id {
                continue;
            }
            let page_on_front: i64 = conn
                .query_row(
                    "SELECT option_value FROM options WHERE option_name = 'page_on_front' LIMIT 1",
                    [],
                    |r| r.get::<_, String>(0),
                )
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);
            let href = Self::format_localized_post_url_with_conn(
                &conn,
                &base_url,
                target_post_id,
                &ptype,
                &slug,
                &lang.code,
                target_post_id == page_on_front,
            );
            if lang.code == default_lang {
                default_href = Some(href.clone());
            }
            links.push(HreflangLink {
                hreflang: lang.code.clone(),
                href,
                lang_name: lang.name.clone(),
                flag_emoji: lang.flag_emoji.clone(),
                post_id: target_post_id,
            });
        }

        if let Some(x_def) = default_href.or_else(|| links.first().map(|l| l.href.clone())) {
            if !links.is_empty() {
                links.push(HreflangLink {
                    hreflang: "x-default".to_string(),
                    href: x_def,
                    lang_name: "Default".to_string(),
                    flag_emoji: "🌐".to_string(),
                    post_id,
                });
            }
        }

        links
    }

    /// Builds `LanguageSwitchLink` items for the `NodeKind::LanguageSwitcher` AST widget and shortcode.
    pub fn build_language_switcher_links(
        &self,
        post_id: Option<i64>,
        active_lang: &str,
    ) -> Vec<daypress_ast::LanguageSwitchLink> {
        let Ok(conn) = self.conn.lock() else {
            return Vec::new();
        };
        let languages = Self::list_languages_with_conn(&conn);
        let default_lang = Self::get_default_language_code_with_conn(&conn);
        let prefix_default = Self::prefix_default_language_enabled_with_conn(&conn);
        let trans_map = post_id
            .map(|pid| Self::get_post_translation_info_with_conn(&conn, pid).2)
            .unwrap_or_default();

        let page_on_front: i64 = conn
            .query_row(
                "SELECT option_value FROM options WHERE option_name = 'page_on_front' LIMIT 1",
                [],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0);

        let mut out = Vec::with_capacity(languages.len());
        for lang in languages {
            let url = if let Some(&target_id) = trans_map.get(&lang.code) {
                if let Ok((slug, ptype)) = conn.query_row(
                    "SELECT post_name, post_type FROM posts WHERE ID = ?1",
                    params![target_id],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
                ) {
                    Self::format_localized_post_url_with_conn(
                        &conn,
                        "",
                        target_id,
                        &ptype,
                        &slug,
                        &lang.code,
                        target_id == page_on_front,
                    )
                } else if lang.code == default_lang && !prefix_default {
                    "/".to_string()
                } else {
                    format!("/{}/", lang.code)
                }
            } else if lang.code == default_lang && !prefix_default {
                "/".to_string()
            } else {
                format!("/{}/", lang.code)
            };

            out.push(daypress_ast::LanguageSwitchLink {
                code: lang.code.clone(),
                name: lang.name,
                flag: lang.flag_emoji,
                url,
                active: lang.code.eq_ignore_ascii_case(active_lang),
            });
        }
        out
    }

    /// Resolves the localized front page for `/{lang}/` when a static front page is configured
    /// and linked to a translation in `lang_code`.
    pub fn get_localized_front_page(
        &self,
        lang_code: &str,
    ) -> Result<Option<DaypressPostResponse>> {
        let page_on_front: i64 = {
            let conn = self.conn.lock().unwrap();
            conn.query_row(
                "SELECT option_value FROM options WHERE option_name = 'page_on_front' LIMIT 1",
                [],
                |r| r.get::<_, String>(0),
            )
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(0)
        };
        if page_on_front <= 0 {
            return Ok(None);
        }
        let target_id = {
            let conn = self.conn.lock().unwrap();
            let def_lang = Self::get_default_language_code_with_conn(&conn);
            let (_, _, map) = Self::get_post_translation_info_with_conn(&conn, page_on_front);
            if let Some(&tid) = map.get(&lang_code.to_ascii_lowercase()) {
                tid
            } else if lang_code.eq_ignore_ascii_case(&def_lang) {
                page_on_front
            } else {
                return Ok(None);
            }
        };
        self.get_post_by_id(target_id)
    }

    /// Resolves a post by `(lang_code, slug)`. Matches posts explicitly assigned to `lang_code`
    /// (or unassigned posts when `lang_code` is the site's default language).
    pub fn get_post_by_lang_and_slug(
        &self,
        lang_code: &str,
        slug: &str,
    ) -> Result<Option<DaypressPostResponse>> {
        let clean_lang = lang_code.trim().to_ascii_lowercase();
        let clean_slug = slug.trim_matches('/');
        let matched_id: Option<i64> = {
            let conn = self.conn.lock().unwrap();
            let def_lang = Self::get_default_language_code_with_conn(&conn);
            conn.query_row(
                r#"SELECT p.ID
                   FROM posts p
                   LEFT JOIN daypress_translations tr ON tr.object_type = 'post' AND tr.object_id = p.ID
                   WHERE p.post_name = ?1
                     AND p.post_type NOT IN ('nav_menu_item', 'revision', 'elementor_library')
                     AND p.post_status <> 'trash'
                     AND LOWER(COALESCE(NULLIF(tr.lang_code, ''), ?3)) = ?2
                   ORDER BY CASE WHEN p.post_status = 'publish' THEN 0 ELSE 1 END, p.ID DESC
                   LIMIT 1"#,
                params![clean_slug, clean_lang, def_lang],
                |r| r.get(0),
            )
            .ok()
        };
        if let Some(pid) = matched_id {
            return self.get_post_by_id(pid);
        }
        Ok(None)
    }

    /// Creates a new translation draft of `source_post_id` in `target_lang_code`, cloning its AST layout,
    /// content, custom fields, and taxonomies, and linking both in the same `translation_group_id`.
    pub fn create_translation_draft(
        &self,
        source_post_id: i64,
        target_lang_code: &str,
    ) -> Result<DaypressPostResponse> {
        let code = target_lang_code.trim().to_ascii_lowercase();
        let grp_id = {
            let conn = self.conn.lock().unwrap();
            let (src_lang, existing_grp, _) =
                Self::get_post_translation_info_with_conn(&conn, source_post_id);
            let grp = if existing_grp.is_empty() {
                Self::link_post_translation_with_conn(&conn, source_post_id, &src_lang, None)?
            } else {
                existing_grp
            };
            Self::ensure_language_registered_with_conn(&conn, &code);
            grp
        };

        let mut dup = self
            .duplicate_post(source_post_id)?
            .ok_or(rusqlite::Error::QueryReturnedNoRows)?;
        let new_slug = format!("{}-{}", dup.slug.trim_end_matches("-copy"), code);
        let new_title = format!(
            "{} ({})",
            dup.title.rendered.trim_end_matches(" (Copy)"),
            code.to_uppercase()
        );
        {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                "UPDATE posts SET post_title = ?1, post_name = ?2 WHERE ID = ?3",
                params![new_title, new_slug, dup.id],
            )?;
            Self::link_post_translation_with_conn(&conn, dup.id, &code, Some(&grp_id))?;
        }
        self.invalidate_hot_cache("create_translation_draft");
        if let Some(updated) = self.get_post_by_id(dup.id)? {
            dup = updated;
        }
        Ok(dup)
    }

    /// Automatically migrates Polylang (`language` / `post_translations` / `term_translations` taxonomies
    /// & serialized `description` maps) and WPML (`_wpml_language`, `_icl_lang_duplicate_of`, `trid`,
    /// `icl_translations`) during 1-click JSON bundle and WXR imports.
    pub(crate) fn import_polylang_and_wpml_with_conn(
        conn: &Connection,
        post_id_map: &HashMap<i64, i64>,
        plugin_options: &HashMap<String, serde_json::Value>,
    ) {
        // 1. Inspect Polylang options (`polylang`) or WPML options (`icl_sitepress_settings`) if provided
        if let Some(pll_opt) = plugin_options.get("polylang") {
            if let Some(def_lang) = pll_opt.get("default_lang").and_then(|v| v.as_str()) {
                let code = Self::ensure_language_registered_with_conn(conn, def_lang);
                let _ = conn.execute("UPDATE daypress_languages SET is_default = 0", []);
                let _ = conn.execute(
                    "UPDATE daypress_languages SET is_default = 1 WHERE code = ?1",
                    params![code],
                );
            }
        }
        if let Some(wpml_opt) = plugin_options.get("icl_sitepress_settings") {
            if let Some(def_lang) = wpml_opt.get("default_language").and_then(|v| v.as_str()) {
                let code = Self::ensure_language_registered_with_conn(conn, def_lang);
                let _ = conn.execute("UPDATE daypress_languages SET is_default = 0", []);
                let _ = conn.execute(
                    "UPDATE daypress_languages SET is_default = 1 WHERE code = ?1",
                    params![code],
                );
            }
        }

        // 2. Polylang `language` taxonomy terms (including serialized PHP metadata in `term_taxonomy.description`)
        let pll_term_defs: Vec<(String, String, String)> = conn
            .prepare(
                r#"SELECT t.slug, t.name, COALESCE(tt.description, '')
                   FROM term_taxonomy tt
                   JOIN terms t ON tt.term_id = t.term_id
                   WHERE tt.taxonomy IN ('language', 'pll_language')"#,
            )
            .and_then(|mut stmt| {
                let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
                Ok(rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        for (raw_slug, term_name, desc) in pll_term_defs {
            let clean_code = raw_slug.trim_start_matches("pll_");
            let code = Self::ensure_language_registered_with_conn(conn, clean_code);
            if !desc.trim().is_empty() {
                if let Some(serde_json::Value::Object(map)) =
                    daypress_ast::parse_php_serialized_to_json(desc.trim())
                        .or_else(|| serde_json::from_str::<serde_json::Value>(desc.trim()).ok())
                {
                    let loc = map
                        .get("locale")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .trim();
                    let is_rtl = map
                        .get("rtl")
                        .map(|v| v.as_i64() == Some(1) || v.as_bool() == Some(true))
                        .unwrap_or(false);
                    if !loc.is_empty() {
                        let _ = conn.execute(
                            "UPDATE daypress_languages SET locale = ?1, is_rtl = ?2, name = CASE WHEN ?3 != '' THEN ?3 ELSE name END WHERE code = ?4",
                            params![loc, if is_rtl { 1 } else { 0 }, term_name.trim(), code],
                        );
                    }
                }
            }
        }

        let pll_lang_rows: Vec<(i64, String)> = conn
            .prepare(
                r#"SELECT r.object_id, t.slug
                   FROM term_relationships r
                   JOIN term_taxonomy tt ON r.term_taxonomy_id = tt.term_taxonomy_id
                   JOIN terms t ON tt.term_id = t.term_id
                   WHERE tt.taxonomy IN ('language', 'pll_language')"#,
            )
            .and_then(|mut stmt| {
                let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
                Ok(rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        for (post_id, raw_slug) in pll_lang_rows {
            let clean_code = raw_slug.trim_start_matches("pll_");
            let _ = Self::link_post_translation_with_conn(conn, post_id, clean_code, None);
        }

        // 3. Polylang `post_translations` taxonomy terms (PHP serialized map in `term_taxonomy.description`)
        let pll_group_rows: Vec<(String, String)> = conn
            .prepare(
                r#"SELECT t.slug, tt.description
                   FROM term_taxonomy tt
                   JOIN terms t ON tt.term_id = t.term_id
                   WHERE tt.taxonomy = 'post_translations' AND TRIM(tt.description) != ''"#,
            )
            .and_then(|mut stmt| {
                let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
                Ok(rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        for (group_slug, desc) in pll_group_rows {
            let parsed = daypress_ast::parse_php_serialized_to_json(&desc)
                .or_else(|| serde_json::from_str::<serde_json::Value>(&desc).ok());
            if let Some(serde_json::Value::Object(map)) = parsed {
                for (lang_key, val) in map {
                    let old_id = val
                        .as_i64()
                        .or_else(|| val.as_str().and_then(|s| s.parse::<i64>().ok()))
                        .unwrap_or(0);
                    if old_id > 0 {
                        let new_id = post_id_map.get(&old_id).copied().unwrap_or(old_id);
                        let _ = Self::link_post_translation_with_conn(
                            conn,
                            new_id,
                            &lang_key,
                            Some(&group_slug),
                        );
                    }
                }
            }
        }

        // 4. WPML / Polylang postmeta keys (`_wpml_language`, `_wpml_import_language_code`, `_pll_lang`, `_icl_lang_duplicate_of`, `_wpml_trid`, `_wpml_import_translation_group`)
        let meta_rows: Vec<(i64, String, String)> = conn
            .prepare(
                r#"SELECT post_id, meta_key, COALESCE(meta_value, '')
                   FROM postmeta
                   WHERE meta_key IN ('_wpml_language', '_wpml_import_language_code', '_pll_lang', 'wpml_language', '_wpml_trid', 'wpml_trid', '_wpml_import_translation_group', '_icl_lang_duplicate_of', '_polylang_translations')"#,
            )
            .and_then(|mut stmt| {
                let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
                Ok(rows.filter_map(|r| r.ok()).collect())
            })
            .unwrap_or_default();

        let mut post_langs: HashMap<i64, String> = HashMap::new();
        let mut post_trids: HashMap<i64, String> = HashMap::new();
        for (pid, key, val) in meta_rows {
            let trimmed = val.trim();
            if trimmed.is_empty() {
                continue;
            }
            match key.as_str() {
                "_wpml_language" | "_wpml_import_language_code" | "_pll_lang" | "wpml_language" => {
                    post_langs.insert(pid, trimmed.to_string());
                }
                "_wpml_trid" | "wpml_trid" | "_wpml_import_translation_group" => {
                    post_trids.insert(pid, format!("wpml_trid_{}", trimmed));
                }
                "_icl_lang_duplicate_of" => {
                    if let Ok(src_old_id) = trimmed.parse::<i64>() {
                        let src_new_id =
                            post_id_map.get(&src_old_id).copied().unwrap_or(src_old_id);
                        let grp = format!("wpml_dup_{}", src_new_id);
                        post_trids.entry(src_new_id).or_insert_with(|| grp.clone());
                        post_trids.insert(pid, grp);
                    }
                }
                "_polylang_translations" => {
                    let parsed = daypress_ast::parse_php_serialized_to_json(trimmed)
                        .or_else(|| serde_json::from_str::<serde_json::Value>(trimmed).ok());
                    if let Some(serde_json::Value::Object(map)) = parsed {
                        let grp = format!("pll_meta_{}", pid);
                        for (lang_k, v) in map {
                            if let Some(old_id) = v
                                .as_i64()
                                .or_else(|| v.as_str().and_then(|s| s.parse::<i64>().ok()))
                            {
                                let new_id = post_id_map.get(&old_id).copied().unwrap_or(old_id);
                                let _ = Self::link_post_translation_with_conn(
                                    conn,
                                    new_id,
                                    &lang_k,
                                    Some(&grp),
                                );
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        for (pid, lang) in &post_langs {
            let grp = post_trids.get(pid).map(String::as_str);
            let _ = Self::link_post_translation_with_conn(conn, *pid, lang, grp);
        }

        // 5. Explicit `icl_translations` array inside `plugin_options`
        if let Some(serde_json::Value::Array(icl_rows)) = plugin_options.get("icl_translations") {
            for item in icl_rows {
                let elem_type = item
                    .get("element_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("post_page");
                if !elem_type.starts_with("post_") {
                    continue;
                }
                let old_id = item
                    .get("element_id")
                    .and_then(|v| {
                        v.as_i64()
                            .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                    })
                    .unwrap_or(0);
                let lang = item
                    .get("language_code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let trid = item
                    .get("trid")
                    .and_then(|v| {
                        v.as_i64()
                            .map(|n| n.to_string())
                            .or_else(|| v.as_str().map(String::from))
                    })
                    .unwrap_or_default();
                if old_id > 0 && !lang.is_empty() {
                    let new_id = post_id_map.get(&old_id).copied().unwrap_or(old_id);
                    let grp = if trid.is_empty() {
                        None
                    } else {
                        Some(format!("wpml_trid_{}", trid))
                    };
                    let _ =
                        Self::link_post_translation_with_conn(conn, new_id, lang, grp.as_deref());
                }
            }
        }
    }
}
