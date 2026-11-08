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
    pub fn get_option(&self, name: &str) -> Result<Option<String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare_cached("SELECT option_value FROM options WHERE option_name = ?1")?;
        let mut rows = stmt.query_map(params![name], |r| r.get(0))?;
        match rows.next() {
            Some(res) => Ok(Some(res?)),
            None => Ok(None),
        }
    }

    pub fn set_option(&self, name: &str, value: &str) -> Result<()> {
        {
            let conn = self.conn.lock().unwrap();
            conn.execute(
                r#"INSERT INTO options (option_name, option_value) VALUES (?1, ?2)
                   ON CONFLICT(option_name) DO UPDATE SET option_value = excluded.option_value;"#,
                params![name, value],
            )?;
        }
        self.invalidate_hot_cache("option_updated");
        Ok(())
    }

    pub fn get_settings(&self) -> Result<DaypressSettingsResponse> {
        let (pending_comments_count, active_plugin_slugs, opts): (
            i64,
            Vec<String>,
            std::collections::HashMap<String, String>,
        ) = {
            let conn = self.conn.lock().unwrap();
            let _ = Self::ensure_builtin_plugins_seeded_with_conn(&conn);
            let pending = conn
                .query_row(
                    "SELECT COUNT(*) FROM comments WHERE comment_approved = '0' OR comment_approved = 'hold' OR comment_approved = 'pending'",
                    [],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            let slugs = conn
                .prepare_cached(
                    "SELECT slug FROM daypress_plugins WHERE is_active = 1 ORDER BY slug ASC",
                )
                .and_then(|mut stmt| {
                    let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
                    Ok(rows.filter_map(|r| r.ok()).collect::<Vec<String>>())
                })
                .unwrap_or_default();
            let options_map = conn
                .prepare_cached("SELECT option_name, option_value FROM options")
                .and_then(|mut stmt| {
                    let rows = stmt
                        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))?;
                    Ok(rows
                        .filter_map(|r| r.ok())
                        .collect::<std::collections::HashMap<String, String>>())
                })
                .unwrap_or_default();
            (pending, slugs, options_map)
        };

        let opt = |key: &str| -> Option<String> { opts.get(key).cloned() };
        let cpt_definitions = self.list_post_type_definitions().unwrap_or_default();
        let custom_post_types: Vec<String> =
            cpt_definitions.iter().map(|d| d.slug.clone()).collect();

        Ok(DaypressSettingsResponse {
            title: opt("blogname").unwrap_or_else(|| "WordPress Reference".to_string()),
            description: opt("blogdescription").unwrap_or_default(),
            url: opt("siteurl").unwrap_or_else(|| "http://localhost:8090".to_string()),
            email: opt("admin_email").unwrap_or_else(|| "admin@example.com".to_string()),
            timezone: opt("timezone_string")
                .filter(|s| !s.trim().is_empty())
                .or_else(|| {
                    opt("gmt_offset")
                        .filter(|s| !s.trim().is_empty() && s.trim() != "0")
                        .map(|off| {
                            if off.starts_with('+')
                                || off.starts_with('-')
                                || off.starts_with("UTC")
                            {
                                if off.starts_with("UTC") {
                                    off
                                } else {
                                    format!("UTC{off}")
                                }
                            } else {
                                format!("UTC+{off}")
                            }
                        })
                })
                .unwrap_or_else(|| "UTC".to_string()),
            date_format: opt("date_format").unwrap_or_else(|| "F j, Y".to_string()),
            time_format: opt("time_format").unwrap_or_else(|| "g:i a".to_string()),
            start_of_week: opt("start_of_week")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1),
            language: opt("WPLANG")
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "en_US".to_string()),
            use_smilies: opt("use_smilies")
                .map(|v| v == "1" || v == "true")
                .unwrap_or(true),
            default_category: opt("default_category")
                .and_then(|s| s.parse().ok())
                .unwrap_or(1),
            default_post_format: opt("default_post_format")
                .unwrap_or_else(|| "standard".to_string()),
            posts_per_page: opt("posts_per_page")
                .and_then(|s| s.parse().ok())
                .unwrap_or(10),
            show_on_front: opt("show_on_front").unwrap_or_else(|| "posts".to_string()),
            page_on_front: opt("page_on_front")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            page_for_posts: opt("page_for_posts")
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            active_theme: opt("daypress_active_theme")
                .filter(|s| !s.trim().is_empty())
                .or_else(|| opt("stylesheet").filter(|s| !s.trim().is_empty()))
                .or_else(|| opt("daypress_imported_theme").filter(|s| !s.trim().is_empty()))
                .unwrap_or_else(|| "Daypress Daylight".to_string()),
            custom_css: crate::legacy::rewrite_daystudio_style_prefixes(
                &opt("custom_css")
                    .or_else(|| opt("daypress_custom_css"))
                    .unwrap_or_default(),
            ),
            brand_color: opt("daypress_brand_color")
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "#a8401b".to_string()),
            typography_scale: opt("daypress_typography_scale")
                .filter(|s| !s.trim().is_empty())
                .unwrap_or_else(|| "serif".to_string()),
            nav_menu_items: opt("daypress_nav_menu_items")
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            widgets_json: opt("daypress_widgets").unwrap_or_default(),
            active_plugins: active_plugin_slugs,
            pending_comments_count,
            daystudio_kit_css: crate::legacy::rewrite_daystudio_style_prefixes(
                &opt("daypress_elementor_kit_css")
                    .filter(|s| !s.trim().is_empty())
                    .or_else(|| opt("elementor_active_kit_css"))
                    .unwrap_or_default(),
            ),
            daystudio_header_html: crate::legacy::rewrite_daystudio_style_prefixes(
                &opt("daypress_elementor_header_html").unwrap_or_default(),
            ),
            daystudio_footer_html: crate::legacy::rewrite_daystudio_style_prefixes(
                &opt("daypress_elementor_footer_html").unwrap_or_default(),
            ),
            daystudio_popups_html: opt("daypress_elementor_popups_html").unwrap_or_default(),
            blog_public: opt("blog_public")
                .map(|v| !matches!(v.trim(), "0" | "false"))
                .unwrap_or(true),
            privacy_policy_page_id: opt("wp_page_for_privacy_policy")
                .and_then(|v| v.trim().parse().ok())
                .unwrap_or(0),
            site_logo_url: opt("daypress_site_logo").unwrap_or_default(),
            site_favicon_url: opt("daypress_site_favicon").unwrap_or_default(),
            custom_post_types,
            cpt_definitions,
            custom_taxonomies: self.list_custom_taxonomy_definitions().unwrap_or_default(),
            theme_assets_html: opt("daypress_theme_assets")
                .map(|raw| {
                    if let Ok(manifest) =
                        serde_json::from_str::<crate::theme::ThemeAssetsManifest>(&raw)
                    {
                        manifest.compile_to_html()
                    } else {
                        raw
                    }
                })
                .unwrap_or_default(),
            update_available: crate::updater::cached_available_update_version(self),
            privacy_settings: self.get_privacy_settings().unwrap_or_default(),
        })
    }

    pub fn get_registered_post_types(&self) -> Result<Vec<String>> {
        let defs = self.list_post_type_definitions()?;
        Ok(defs.into_iter().map(|d| d.slug).collect())
    }

    pub fn vacuum(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch("VACUUM;")?;
        Ok(())
    }

    pub fn integrity_check(&self) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let res: String = conn.query_row("PRAGMA integrity_check;", [], |r| r.get(0))?;
        Ok(res)
    }

    pub fn get_stats(&self) -> Result<DbStats> {
        let conn = self.conn.lock().unwrap();
        let total_posts: i64 = conn.query_row(
            "SELECT count(*) FROM posts WHERE post_type = 'post'",
            [],
            |r| r.get(0),
        )?;
        let total_pages: i64 = conn.query_row(
            "SELECT count(*) FROM posts WHERE post_type = 'page'",
            [],
            |r| r.get(0),
        )?;
        let total_comments: i64 =
            conn.query_row("SELECT count(*) FROM comments", [], |r| r.get(0))?;
        let total_options: i64 =
            conn.query_row("SELECT count(*) FROM options", [], |r| r.get(0))?;
        let total_revisions: i64 = conn
            .query_row(
                "SELECT count(*) FROM posts WHERE post_type = 'revision'",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let page_size: i64 = conn.query_row("PRAGMA page_size;", [], |r| r.get(0))?;
        let page_count: i64 = conn.query_row("PRAGMA page_count;", [], |r| r.get(0))?;
        let journal_mode: String = conn.query_row("PRAGMA journal_mode;", [], |r| r.get(0))?;

        // WAL mode keeps recent writes in `<db>-wal` until a checkpoint, so count both files.
        let wal_path = format!("{}-wal", self.db_path.to_string_lossy());
        let file_size_bytes = fs::metadata(&self.db_path).map(|m| m.len()).unwrap_or(0)
            + fs::metadata(&wal_path).map(|m| m.len()).unwrap_or(0);

        Ok(DbStats {
            db_path: self.db_path.to_string_lossy().to_string(),
            file_size_bytes,
            total_posts,
            total_pages,
            total_comments,
            total_options,
            total_revisions,
            page_size,
            page_count,
            journal_mode,
        })
    }
}

impl crate::ports::output::OptionRepository for Database {
    fn get_option(&self, name: &str) -> DomainResult<Option<String>> {
        Database::get_option(self, name).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn set_option(&self, name: &str, value: &str) -> DomainResult<()> {
        Database::set_option(self, name, value).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn get_settings(&self) -> DomainResult<DaypressSettingsResponse> {
        Database::get_settings(self).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn get_stats(&self) -> DomainResult<DbStats> {
        Database::get_stats(self).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn list_forms(&self) -> DomainResult<Vec<(DaypressForm, i64)>> {
        Database::list_forms(self).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn get_form_by_id(&self, id: i64) -> DomainResult<Option<DaypressForm>> {
        Database::get_form(self, id).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn save_form_record(&self, req: &CreateFormRequest) -> DomainResult<i64> {
        Database::save_form_record(self, req).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn list_form_entries_filtered(
        &self,
        form_id: Option<i64>,
        status: Option<&str>,
        search: Option<&str>,
        limit: i64,
    ) -> DomainResult<Vec<FormEntryItem>> {
        Database::list_form_entries_filtered(self, form_id, status, search, limit)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn get_analytics_dashboard(&self, range: &str) -> DomainResult<AnalyticsSummary> {
        Database::get_analytics_dashboard(self, range)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn list_theme_templates(
        &self,
        template_type: Option<&str>,
    ) -> DomainResult<Vec<serde_json::Value>> {
        use crate::theme_builder::models::{TemplateFilter, TemplateType};
        let parsed_type = template_type
            .map(str::trim)
            .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("all"))
            .and_then(TemplateType::from_str_loose);
        let filter = TemplateFilter {
            template_type: parsed_type,
            status: None,
            is_active: None,
        };
        let records = self
            .theme_templates()
            .list(&filter)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(records
            .into_iter()
            .filter_map(|r| serde_json::to_value(r).ok())
            .collect())
    }

    fn save_theme_template_from_json(
        &self,
        payload: &serde_json::Value,
    ) -> DomainResult<serde_json::Value> {
        use crate::theme_builder::models::{CreateTemplateRequest, UpdateTemplateRequest};
        let repo = self.theme_templates();
        let saved = if let Some(id) = payload
            .get("id")
            .and_then(|v| v.as_i64())
            .filter(|&i| i > 0)
        {
            let req: UpdateTemplateRequest =
                serde_json::from_value(payload.clone()).map_err(|e| {
                    DomainError::Validation(format!("Invalid template update payload: {e}"))
                })?;
            repo.update(id, &req)
                .map_err(|e| DomainError::Storage(e.to_string()))?
                .ok_or_else(|| DomainError::NotFound(format!("Template #{id} not found")))?
        } else {
            let req: CreateTemplateRequest =
                serde_json::from_value(payload.clone()).map_err(|e| {
                    DomainError::Validation(format!("Invalid template create payload: {e}"))
                })?;
            repo.create(&req)
                .map_err(|e| DomainError::Storage(e.to_string()))?
        };
        self.invalidate_hot_cache("mcp_template_saved");
        serde_json::to_value(saved).map_err(|e| DomainError::Internal(e.to_string()))
    }

    fn list_media_folders(&self) -> DomainResult<Vec<MediaFolderDto>> {
        Database::list_media_folders(self).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn create_media_folder(&self, name: &str, parent_id: i64) -> DomainResult<MediaFolderDto> {
        Database::create_media_folder(self, name, parent_id)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn scan_unused_media(&self) -> DomainResult<Vec<DaypressMediaResponse>> {
        Database::scan_unused_media_attachments(self)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn list_post_type_definitions(&self) -> DomainResult<Vec<CustomPostTypeDefinition>> {
        Database::list_post_type_definitions(self).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn save_post_type_definition(
        &self,
        def: &CustomPostTypeDefinition,
    ) -> DomainResult<CustomPostTypeDefinition> {
        Database::save_post_type_definition(self, def)
            .map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn provision_content_type_preset(&self, preset_key: &str) -> DomainResult<serde_json::Value> {
        let (post_type, taxonomies, field_group) =
            Database::provision_content_type_preset(self, preset_key)
                .map_err(|e| DomainError::Validation(e.to_string()))?;
        Ok(serde_json::json!({
            "status": "provisioned",
            "preset": preset_key,
            "post_type": post_type,
            "taxonomies": taxonomies,
            "field_group": field_group
        }))
    }

    fn list_field_group_definitions(&self) -> DomainResult<Vec<FieldGroupDefinition>> {
        Database::list_field_groups(self, None).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn save_field_group_definition(
        &self,
        group: &FieldGroupDefinition,
    ) -> DomainResult<FieldGroupDefinition> {
        Database::save_field_group(self, group).map_err(|e| DomainError::Storage(e.to_string()))
    }

    fn search_fts_json(
        &self,
        query: &str,
        post_type: Option<&str>,
        limit: usize,
    ) -> DomainResult<serde_json::Value> {
        let resp = Database::search_fts(self, query, post_type, limit)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        serde_json::to_value(resp).map_err(|e| DomainError::Internal(e.to_string()))
    }

    fn audit_post_json(&self, payload: &serde_json::Value) -> DomainResult<serde_json::Value> {
        let req: crate::adapters::database::search_fts::AuditRequestPayload =
            serde_json::from_value(payload.clone()).unwrap_or_default();
        let report = Database::audit_post_wcag_links_seo(self, &req);
        serde_json::to_value(report).map_err(|e| DomainError::Internal(e.to_string()))
    }

    fn list_languages_json(&self) -> DomainResult<serde_json::Value> {
        let languages =
            Database::list_languages(self).map_err(|e| DomainError::Storage(e.to_string()))?;
        let default_lang_code = Database::get_default_language_code(self);
        let prefix_default = Database::prefix_default_language_enabled(self);
        Ok(serde_json::json!({
            "languages": languages,
            "default_language": default_lang_code,
            "prefix_default": prefix_default,
        }))
    }

    fn link_translations_json(
        &self,
        payload: &serde_json::Value,
    ) -> DomainResult<serde_json::Value> {
        if let Some(code) = payload.get("upsert_language_code").and_then(|v| v.as_str()) {
            let (preset_locale, preset_name, preset_flag, preset_rtl) =
                crate::i18n::preset_language_info(code);
            let lang = DaypressLanguage {
                code: code.to_ascii_lowercase(),
                locale: preset_locale.to_string(),
                name: preset_name.to_string(),
                native_name: preset_name.to_string(),
                flag_emoji: preset_flag.to_string(),
                is_default: false,
                is_rtl: preset_rtl,
                is_active: true,
                sort_order: 10,
            };
            let _ = Database::upsert_language(self, &lang);
        }
        if let (Some(src_id), Some(tgt_id), Some(tgt_lang)) = (
            payload.get("source_post_id").and_then(|v| v.as_i64()),
            payload.get("target_post_id").and_then(|v| v.as_i64()),
            payload.get("target_lang").and_then(|v| v.as_str()),
        ) {
            let mut map = std::collections::HashMap::new();
            let src_lang = payload
                .get("source_lang")
                .and_then(|v| v.as_str())
                .map(String::from)
                .unwrap_or_else(|| Database::get_default_language_code(self));
            map.insert(src_lang, src_id);
            map.insert(tgt_lang.to_string(), tgt_id);
            let group_id = Database::link_translation_map(self, "post", None, &map)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let updated_map = Database::get_translation_group_map(self, "post", &group_id)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            return Ok(serde_json::json!({
                "status": "linked",
                "translation_group_id": group_id,
                "translations": updated_map,
            }));
        }
        if let Some(obj) = payload.get("translations").and_then(|v| v.as_object()) {
            let obj_type = payload
                .get("object_type")
                .and_then(|v| v.as_str())
                .unwrap_or("post");
            let mut map = std::collections::HashMap::new();
            for (lang, val) in obj {
                if let Some(oid) = val.as_i64() {
                    map.insert(lang.clone(), oid);
                }
            }
            let group_id = Database::link_translation_map(self, obj_type, None, &map)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            let updated_map = Database::get_translation_group_map(self, obj_type, &group_id)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
            return Ok(serde_json::json!({
                "status": "linked",
                "translation_group_id": group_id,
                "translations": updated_map,
            }));
        }
        Err(DomainError::Validation(
            "Provide `translations` object or `(source_post_id, target_post_id, target_lang)`."
                .into(),
        ))
    }

    fn duplicate_post_json(&self, source_id: i64) -> DomainResult<serde_json::Value> {
        let cloned = Database::duplicate_post(self, source_id)
            .map_err(|e| DomainError::Storage(e.to_string()))?
            .ok_or_else(|| DomainError::NotFound(format!("Post #{source_id} not found")))?;
        serde_json::to_value(cloned).map_err(|e| DomainError::Internal(e.to_string()))
    }

    fn create_preview_link_json(
        &self,
        post_id: i64,
        expires_in_hours: i64,
        label: &str,
    ) -> DomainResult<serde_json::Value> {
        let record =
            Database::create_preview_token(self, post_id, expires_in_hours, Some(label), 1)
                .map_err(|e| DomainError::Storage(e.to_string()))?;
        serde_json::to_value(record).map_err(|e| DomainError::Internal(e.to_string()))
    }

    fn schedule_revision_json(
        &self,
        post_id: i64,
        payload: &serde_json::Value,
    ) -> DomainResult<serde_json::Value> {
        let req: ScheduleRevisionRequest = serde_json::from_value(payload.clone())
            .map_err(|e| DomainError::Validation(format!("Invalid schedule_revision args: {e}")))?;
        let summary = Database::schedule_published_post_revision(self, post_id, &req, None)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        serde_json::to_value(summary).map_err(|e| DomainError::Internal(e.to_string()))
    }

    fn diff_revisions_json(
        &self,
        post_id: i64,
        from_rev_id: Option<i64>,
        to_rev_id: Option<i64>,
    ) -> DomainResult<serde_json::Value> {
        let from_id = match from_rev_id {
            Some(fid) => fid,
            None => {
                let revs = Database::get_post_revisions(self, post_id)
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                revs.first().map(|r| r.id).unwrap_or(post_id)
            }
        };
        let report = Database::diff_post_revisions(self, post_id, from_id, to_rev_id)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        serde_json::to_value(report).map_err(|e| DomainError::Internal(e.to_string()))
    }

    fn list_subscribers_json(
        &self,
        status: Option<&str>,
        list: Option<&str>,
    ) -> DomainResult<serde_json::Value> {
        let subscribers = Database::list_subscribers(self, status, list, None, false)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        let stats = Database::get_newsletter_stats(self)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        Ok(serde_json::json!({
            "subscribers": subscribers,
            "stats": stats,
        }))
    }

    fn create_newsletter_campaign_json(
        &self,
        payload: &serde_json::Value,
    ) -> DomainResult<serde_json::Value> {
        let req: CreateNewsletterCampaignRequest = serde_json::from_value(payload.clone())
            .map_err(|e| DomainError::Validation(format!("Invalid campaign payload: {e}")))?;
        let campaign = Database::create_and_send_campaign(self, &req, "broadcast", None)
            .map_err(|e| DomainError::Validation(e.to_string()))?;
        serde_json::to_value(campaign).map_err(|e| DomainError::Internal(e.to_string()))
    }

    fn manage_shortlinks_json(
        &self,
        payload: &serde_json::Value,
    ) -> DomainResult<serde_json::Value> {
        let action = payload
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("list");
        match action {
            "create" | "save" | "upsert" => {
                let req: CreateShortlinkRequest =
                    serde_json::from_value(payload.clone()).map_err(|e| {
                        DomainError::Validation(format!("Invalid shortlink payload: {e}"))
                    })?;
                let saved = Database::save_shortlink(self, &req)
                    .map_err(|e| DomainError::Validation(e.to_string()))?;
                serde_json::to_value(saved).map_err(|e| DomainError::Internal(e.to_string()))
            }
            "delete" => {
                let id = payload
                    .get("id")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| DomainError::Validation("'id' is required for delete".into()))?;
                let deleted = Database::delete_shortlink(self, id)
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                Ok(serde_json::json!({ "deleted": deleted, "id": id }))
            }
            _ => {
                let links = Database::list_shortlinks(self)
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                Ok(serde_json::json!({ "shortlinks": links }))
            }
        }
    }

    fn manage_webhooks_json(&self, payload: &serde_json::Value) -> DomainResult<serde_json::Value> {
        let action = payload
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("list");
        match action {
            "create" | "save" | "upsert" => {
                let req: CreateWebhookEndpointRequest = serde_json::from_value(payload.clone())
                    .map_err(|e| {
                        DomainError::Validation(format!("Invalid webhook payload: {e}"))
                    })?;
                let saved = Database::save_webhook_endpoint(self, &req)
                    .map_err(|e| DomainError::Validation(e.to_string()))?;
                serde_json::to_value(saved).map_err(|e| DomainError::Internal(e.to_string()))
            }
            "delete" => {
                let id = payload
                    .get("id")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| DomainError::Validation("'id' is required for delete".into()))?;
                let deleted = Database::delete_webhook_endpoint(self, id)
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                Ok(serde_json::json!({ "deleted": deleted, "id": id }))
            }
            "replay" => {
                let delivery_id = payload
                    .get("delivery_id")
                    .or_else(|| payload.get("id"))
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| DomainError::Validation("'delivery_id' is required".into()))?;
                let replayed = Database::replay_webhook_delivery_sync(self, delivery_id)
                    .map_err(|e| DomainError::Validation(e.to_string()))?;
                serde_json::to_value(replayed).map_err(|e| DomainError::Internal(e.to_string()))
            }
            _ => {
                let webhooks = Database::list_webhook_endpoints(self)
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                let deliveries = Database::list_webhook_deliveries(self, None, 25)
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                Ok(serde_json::json!({
                    "webhooks": webhooks,
                    "deliveries": deliveries,
                }))
            }
        }
    }

    fn manage_ab_tests_json(&self, payload: &serde_json::Value) -> DomainResult<serde_json::Value> {
        let action = payload
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("list");
        match action {
            "create" | "save" | "upsert" => {
                let req: CreateAbTestRequest =
                    serde_json::from_value(payload.clone()).map_err(|e| {
                        DomainError::Validation(format!("Invalid ab_test payload: {e}"))
                    })?;
                let saved = Database::save_ab_test(self, &req)
                    .map_err(|e| DomainError::Validation(e.to_string()))?;
                serde_json::to_value(saved).map_err(|e| DomainError::Internal(e.to_string()))
            }
            "promote" => {
                let test_id = payload
                    .get("id")
                    .or_else(|| payload.get("test_id"))
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| {
                        DomainError::Validation("'id' is required for promote".into())
                    })?;
                let winner = payload
                    .get("winner")
                    .and_then(|v| v.as_str())
                    .unwrap_or("B");
                let promoted = Database::promote_ab_test_winner(self, test_id, winner)
                    .map_err(|e| DomainError::Validation(e.to_string()))?;
                serde_json::to_value(promoted).map_err(|e| DomainError::Internal(e.to_string()))
            }
            "delete" => {
                let id = payload
                    .get("id")
                    .and_then(|v| v.as_i64())
                    .ok_or_else(|| DomainError::Validation("'id' is required for delete".into()))?;
                let deleted = Database::delete_ab_test(self, id)
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                Ok(serde_json::json!({ "deleted": deleted, "id": id }))
            }
            _ => {
                let post_id = payload.get("post_id").and_then(|v| v.as_i64());
                let tests = Database::list_ab_tests(self, post_id)
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                Ok(serde_json::json!({ "ab_tests": tests }))
            }
        }
    }

    fn get_web_vitals_json(&self) -> DomainResult<serde_json::Value> {
        let vitals = Database::get_core_web_vitals_summary(self)
            .map_err(|e| DomainError::Storage(e.to_string()))?;
        serde_json::to_value(vitals).map_err(|e| DomainError::Internal(e.to_string()))
    }

    fn manage_wasm_extensions_json(
        &self,
        payload: &serde_json::Value,
    ) -> DomainResult<serde_json::Value> {
        let action = payload
            .get("action")
            .and_then(|v| v.as_str())
            .unwrap_or("list");
        match action {
            "install" | "create" | "save" => {
                let req: InstallWasmExtensionRequest = serde_json::from_value(payload.clone())
                    .map_err(|e| {
                        DomainError::Validation(format!("Invalid wasm extension payload: {e}"))
                    })?;
                let saved = Database::install_or_update_wasm_extension(self, &req)
                    .map_err(|e| DomainError::Validation(e.to_string()))?;
                serde_json::to_value(saved).map_err(|e| DomainError::Internal(e.to_string()))
            }
            "execute" => {
                let slug = payload
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError::Validation("'slug' is required".into()))?;
                let export_name = payload
                    .get("export_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("render_shortcode");
                let mut attrs = std::collections::HashMap::new();
                if let Some(obj) = payload.get("attrs").and_then(|v| v.as_object()) {
                    for (k, v) in obj {
                        if let Some(s) = v.as_str() {
                            attrs.insert(k.clone(), s.to_string());
                        }
                    }
                }
                let content = payload
                    .get("content")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let out =
                    Database::execute_wasm_extension(self, slug, export_name, &attrs, content)
                        .map_err(|e| DomainError::Validation(e.to_string()))?;
                serde_json::to_value(out).map_err(|e| DomainError::Internal(e.to_string()))
            }
            "delete" => {
                let slug = payload
                    .get("slug")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| DomainError::Validation("'slug' is required".into()))?;
                let deleted = Database::delete_wasm_extension(self, slug)
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                Ok(serde_json::json!({ "deleted": deleted, "slug": slug }))
            }
            _ => {
                let exts = Database::list_wasm_extensions(self)
                    .map_err(|e| DomainError::Storage(e.to_string()))?;
                Ok(serde_json::json!({ "wasm_extensions": exts }))
            }
        }
    }
}
