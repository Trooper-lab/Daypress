//! Input Ports (Driving Use-Case Traits).
//!
//! Defines the entry points through which primary adapters (HTTP REST API, SSR Admin,
//! and Claude / MCP AI tools) interact with the core application layer.

use crate::domain::errors::DomainResult;
use crate::domain::models::*;

/// Input port for managing WordPress content (posts, pages, visual builder AST layouts, taxonomies).
pub trait ContentUseCase: Send + Sync {
    fn list_content(
        &self,
        post_type: &str,
        status: Option<&str>,
        search: Option<&str>,
        limit: i64,
    ) -> DomainResult<Vec<DaypressPostResponse>>;

    fn get_content_by_id(&self, id: i64) -> DomainResult<DaypressPostResponse>;
    fn get_content_by_slug(&self, slug: &str) -> DomainResult<DaypressPostResponse>;
    fn publish_content(&self, req: CreatePostRequest) -> DomainResult<DaypressPostResponse>;
    fn edit_content(&self, id: i64, req: UpdatePostRequest) -> DomainResult<DaypressPostResponse>;
    fn remove_content(&self, id: i64, force: bool) -> DomainResult<bool>;

    fn get_layout_ast(&self, post_id: i64) -> DomainResult<Option<String>>;
    fn update_layout_ast(&self, post_id: i64, layout_json: &str) -> DomainResult<()>;

    fn list_taxonomies(&self, taxonomy: &str) -> DomainResult<Vec<DaypressCategoryResponse>>;
    fn create_taxonomy_term(
        &self,
        taxonomy: &str,
        req: CreateTermRequest,
    ) -> DomainResult<DaypressCategoryResponse>;
}

/// Input port for 1-Click WordPress Migration & WXR XML interchange.
pub trait MigrationUseCase: Send + Sync {
    fn migrate_wordpress_bundle(
        &self,
        bundle: ImportBundleRequest,
    ) -> DomainResult<ImportBundleResponse>;
    fn migrate_wxr_xml(&self, xml_bytes: &[u8]) -> DomainResult<WxrImportResponse>;
    fn export_site_wxr(&self) -> DomainResult<String>;
}

/// Input port for querying and updating WordPress site settings & options.
pub trait SettingsUseCase: Send + Sync {
    fn get_site_settings(&self) -> DomainResult<DaypressSettingsResponse>;
    fn update_site_option(&self, name: &str, value: &str) -> DomainResult<()>;
    fn get_site_option(&self, name: &str) -> DomainResult<Option<String>>;
    fn get_system_stats(&self) -> DomainResult<DbStats>;
}
