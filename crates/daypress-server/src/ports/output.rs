//! Output Ports (Driven / Repository Traits).
//!
//! Defines what the Daypress application needs from external persistence or infrastructure
//! without specifying *how* it happens (Rusqlite, PostgreSQL, or an in-memory test mock).

use crate::domain::errors::DomainResult;
use crate::domain::models::*;

/// Output port for WordPress Posts, Pages, Revisions, and Visual Builder AST Layouts.
pub trait PostRepository: Send + Sync {
    fn list_posts(
        &self,
        post_type: &str,
        status: Option<&str>,
        search: Option<&str>,
        limit: i64,
    ) -> DomainResult<Vec<DaypressPostResponse>>;

    fn find_post_by_id(&self, id: i64) -> DomainResult<Option<DaypressPostResponse>>;
    fn find_post_by_slug(&self, slug: &str) -> DomainResult<Option<DaypressPostResponse>>;
    fn create_post(&self, req: &CreatePostRequest) -> DomainResult<DaypressPostResponse>;
    fn update_post(
        &self,
        id: i64,
        req: &UpdatePostRequest,
    ) -> DomainResult<Option<DaypressPostResponse>>;
    fn duplicate_post(&self, source_id: i64) -> DomainResult<Option<DaypressPostResponse>>;
    fn trash_post(&self, id: i64) -> DomainResult<Option<DaypressPostResponse>>;
    fn delete_post(&self, id: i64) -> DomainResult<bool>;

    fn get_post_layout(&self, post_id: i64) -> DomainResult<Option<String>>;
    fn save_post_layout(
        &self,
        post_id: i64,
        layout_json: &str,
        compiled_html: Option<&str>,
    ) -> DomainResult<()>;
    fn delete_post_layout(&self, post_id: i64) -> DomainResult<()>;
}

/// Output port for WordPress Taxonomies (`category`, `post_tag`, `nav_menu`).
pub trait TaxonomyRepository: Send + Sync {
    fn list_terms(&self, taxonomy: &str) -> DomainResult<Vec<DaypressCategoryResponse>>;
    fn find_term_by_id(
        &self,
        taxonomy: &str,
        id: i64,
    ) -> DomainResult<Option<DaypressCategoryResponse>>;
    fn create_term(
        &self,
        taxonomy: &str,
        req: &CreateTermRequest,
    ) -> DomainResult<DaypressCategoryResponse>;
    fn update_term(
        &self,
        taxonomy: &str,
        id: i64,
        req: &UpdateTermRequest,
    ) -> DomainResult<Option<DaypressCategoryResponse>>;
    fn delete_term(&self, taxonomy: &str, id: i64) -> DomainResult<bool>;
}

/// Output port for WordPress Options (`options`), Site Settings, FormStudio, and Analytics.
pub trait OptionRepository: Send + Sync {
    fn get_option(&self, name: &str) -> DomainResult<Option<String>>;
    fn set_option(&self, name: &str, value: &str) -> DomainResult<()>;
    fn get_settings(&self) -> DomainResult<DaypressSettingsResponse>;
    fn get_stats(&self) -> DomainResult<DbStats>;

    fn list_forms(&self) -> DomainResult<Vec<(DaypressForm, i64)>> {
        Ok(vec![])
    }
    fn get_form_by_id(&self, _id: i64) -> DomainResult<Option<DaypressForm>> {
        Ok(None)
    }
    fn save_form_record(&self, _req: &CreateFormRequest) -> DomainResult<i64> {
        Err(crate::domain::errors::DomainError::NotFound("mock".into()))
    }
    fn list_form_entries_filtered(
        &self,
        _form_id: Option<i64>,
        _status: Option<&str>,
        _search: Option<&str>,
        _limit: i64,
    ) -> DomainResult<Vec<FormEntryItem>> {
        Ok(vec![])
    }
    fn get_analytics_dashboard(&self, _range: &str) -> DomainResult<AnalyticsSummary> {
        Ok(AnalyticsSummary::default())
    }
    fn list_theme_templates(
        &self,
        _template_type: Option<&str>,
    ) -> DomainResult<Vec<serde_json::Value>> {
        Ok(vec![])
    }
    fn save_theme_template_from_json(
        &self,
        _payload: &serde_json::Value,
    ) -> DomainResult<serde_json::Value> {
        Err(crate::domain::errors::DomainError::NotFound("mock".into()))
    }
    fn list_media_folders(&self) -> DomainResult<Vec<MediaFolderDto>> {
        Ok(vec![])
    }
    fn create_media_folder(&self, _name: &str, _parent_id: i64) -> DomainResult<MediaFolderDto> {
        Err(crate::domain::errors::DomainError::NotFound("mock".into()))
    }
    fn scan_unused_media(&self) -> DomainResult<Vec<DaypressMediaResponse>> {
        Ok(vec![])
    }
    fn list_post_type_definitions(&self) -> DomainResult<Vec<CustomPostTypeDefinition>> {
        Ok(vec![])
    }
    fn save_post_type_definition(
        &self,
        _def: &CustomPostTypeDefinition,
    ) -> DomainResult<CustomPostTypeDefinition> {
        Err(crate::domain::errors::DomainError::NotFound("mock".into()))
    }
    fn provision_content_type_preset(&self, _preset_key: &str) -> DomainResult<serde_json::Value> {
        Err(crate::domain::errors::DomainError::NotFound("mock".into()))
    }
    fn list_field_group_definitions(&self) -> DomainResult<Vec<FieldGroupDefinition>> {
        Ok(vec![])
    }
    fn save_field_group_definition(
        &self,
        _group: &FieldGroupDefinition,
    ) -> DomainResult<FieldGroupDefinition> {
        Err(crate::domain::errors::DomainError::NotFound("mock".into()))
    }
    fn search_fts_json(
        &self,
        _query: &str,
        _post_type: Option<&str>,
        _limit: usize,
    ) -> DomainResult<serde_json::Value> {
        Ok(serde_json::json!({ "results": [], "total": 0 }))
    }
    fn audit_post_json(&self, _payload: &serde_json::Value) -> DomainResult<serde_json::Value> {
        Ok(serde_json::json!({ "score": 100, "issues": [] }))
    }
    fn list_languages_json(&self) -> DomainResult<serde_json::Value> {
        Ok(serde_json::json!({ "languages": [] }))
    }
    fn link_translations_json(
        &self,
        _payload: &serde_json::Value,
    ) -> DomainResult<serde_json::Value> {
        Err(crate::domain::errors::DomainError::NotFound("mock".into()))
    }
    fn duplicate_post_json(&self, _source_id: i64) -> DomainResult<serde_json::Value> {
        Err(crate::domain::errors::DomainError::NotFound("mock".into()))
    }
    fn create_preview_link_json(
        &self,
        _post_id: i64,
        _expires_in_hours: i64,
        _label: &str,
    ) -> DomainResult<serde_json::Value> {
        Err(crate::domain::errors::DomainError::NotFound("mock".into()))
    }
    fn schedule_revision_json(
        &self,
        _post_id: i64,
        _payload: &serde_json::Value,
    ) -> DomainResult<serde_json::Value> {
        Err(crate::domain::errors::DomainError::NotFound("mock".into()))
    }
    fn diff_revisions_json(
        &self,
        _post_id: i64,
        _from_rev_id: Option<i64>,
        _to_rev_id: Option<i64>,
    ) -> DomainResult<serde_json::Value> {
        Err(crate::domain::errors::DomainError::NotFound("mock".into()))
    }
    fn list_subscribers_json(
        &self,
        _status: Option<&str>,
        _list: Option<&str>,
    ) -> DomainResult<serde_json::Value> {
        Ok(serde_json::json!({ "subscribers": [], "stats": {} }))
    }
    fn create_newsletter_campaign_json(
        &self,
        _payload: &serde_json::Value,
    ) -> DomainResult<serde_json::Value> {
        Err(crate::domain::errors::DomainError::NotFound("mock".into()))
    }
    fn manage_shortlinks_json(
        &self,
        _payload: &serde_json::Value,
    ) -> DomainResult<serde_json::Value> {
        Ok(serde_json::json!({ "shortlinks": [] }))
    }
    fn manage_webhooks_json(
        &self,
        _payload: &serde_json::Value,
    ) -> DomainResult<serde_json::Value> {
        Ok(serde_json::json!({ "webhooks": [], "deliveries": [] }))
    }
    fn manage_ab_tests_json(
        &self,
        _payload: &serde_json::Value,
    ) -> DomainResult<serde_json::Value> {
        Ok(serde_json::json!({ "ab_tests": [] }))
    }
    fn get_web_vitals_json(&self) -> DomainResult<serde_json::Value> {
        Ok(serde_json::json!({}))
    }
    fn manage_wasm_extensions_json(
        &self,
        _payload: &serde_json::Value,
    ) -> DomainResult<serde_json::Value> {
        Ok(serde_json::json!({ "wasm_extensions": [] }))
    }
    fn security_audit_json(&self) -> DomainResult<serde_json::Value> {
        Ok(serde_json::json!({ "overall_status": "production_ready", "score": 100 }))
    }
    fn run_benchmark_json(&self, _iterations: usize) -> DomainResult<serde_json::Value> {
        Ok(serde_json::json!({ "all_slas_met": true, "workloads": [] }))
    }
}

/// Output port for WordPress Comments (`comments`).
pub trait CommentRepository: Send + Sync {
    fn list_comments(
        &self,
        post_id: Option<i64>,
        status: Option<&str>,
        limit: i64,
    ) -> DomainResult<Vec<DaypressCommentAdminItem>>;
    fn find_comment_by_id(&self, id: i64) -> DomainResult<Option<DaypressCommentAdminItem>>;
    fn create_comment(&self, req: &CreateCommentRequest) -> DomainResult<DaypressCommentResponse>;
    fn update_comment(
        &self,
        id: i64,
        req: &UpdateCommentRequest,
    ) -> DomainResult<Option<DaypressCommentAdminItem>>;
    fn delete_comment(&self, id: i64) -> DomainResult<bool>;
}

/// Output port for 1-Click WordPress Bundle Migration & WXR Import/Export.
pub trait MigrationRepository: Send + Sync {
    fn import_bundle(&self, req: &ImportBundleRequest) -> DomainResult<ImportBundleResponse>;
    fn import_wxr_xml(&self, xml_bytes: &[u8]) -> DomainResult<WxrImportResponse>;
    fn export_wxr_xml(&self) -> DomainResult<String>;
}
