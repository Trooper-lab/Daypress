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
    ) -> DomainResult<Vec<WpPostResponse>>;

    fn find_post_by_id(&self, id: i64) -> DomainResult<Option<WpPostResponse>>;
    fn find_post_by_slug(&self, slug: &str) -> DomainResult<Option<WpPostResponse>>;
    fn create_post(&self, req: &CreatePostRequest) -> DomainResult<WpPostResponse>;
    fn update_post(&self, id: i64, req: &UpdatePostRequest)
        -> DomainResult<Option<WpPostResponse>>;
    fn duplicate_post(&self, source_id: i64) -> DomainResult<Option<WpPostResponse>>;
    fn trash_post(&self, id: i64) -> DomainResult<Option<WpPostResponse>>;
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
    fn list_terms(&self, taxonomy: &str) -> DomainResult<Vec<WpCategoryResponse>>;
    fn find_term_by_id(&self, taxonomy: &str, id: i64) -> DomainResult<Option<WpCategoryResponse>>;
    fn create_term(
        &self,
        taxonomy: &str,
        req: &CreateTermRequest,
    ) -> DomainResult<WpCategoryResponse>;
    fn update_term(
        &self,
        taxonomy: &str,
        id: i64,
        req: &UpdateTermRequest,
    ) -> DomainResult<Option<WpCategoryResponse>>;
    fn delete_term(&self, taxonomy: &str, id: i64) -> DomainResult<bool>;
}

/// Output port for WordPress Options (`options`) and Site Settings.
pub trait OptionRepository: Send + Sync {
    fn get_option(&self, name: &str) -> DomainResult<Option<String>>;
    fn set_option(&self, name: &str, value: &str) -> DomainResult<()>;
    fn get_settings(&self) -> DomainResult<WpSettingsResponse>;
    fn get_stats(&self) -> DomainResult<DbStats>;
}

/// Output port for WordPress Comments (`comments`).
pub trait CommentRepository: Send + Sync {
    fn list_comments(
        &self,
        post_id: Option<i64>,
        status: Option<&str>,
        limit: i64,
    ) -> DomainResult<Vec<WpCommentAdminItem>>;
    fn find_comment_by_id(&self, id: i64) -> DomainResult<Option<WpCommentAdminItem>>;
    fn create_comment(&self, req: &CreateCommentRequest) -> DomainResult<WpCommentResponse>;
    fn update_comment(
        &self,
        id: i64,
        req: &UpdateCommentRequest,
    ) -> DomainResult<Option<WpCommentAdminItem>>;
    fn delete_comment(&self, id: i64) -> DomainResult<bool>;
}

/// Output port for 1-Click WordPress Bundle Migration & WXR Import/Export.
pub trait MigrationRepository: Send + Sync {
    fn import_bundle(&self, req: &ImportBundleRequest) -> DomainResult<ImportBundleResponse>;
    fn import_wxr_xml(&self, xml_bytes: &[u8]) -> DomainResult<WxrImportResponse>;
    fn export_wxr_xml(&self) -> DomainResult<String>;
}
