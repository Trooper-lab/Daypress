//! Application Services (Use-Case Orchestrator).
//!
//! Coordinates pure business logic by implementing input ports (`ContentUseCase`,
//! `MigrationUseCase`, `SettingsUseCase`) and delegating persistence to output ports
//! via `Arc<dyn ...Repository>`. Zero `axum` or `rusqlite` imports.

use crate::domain::errors::{DomainError, DomainResult};
use crate::domain::models::*;
use crate::ports::input::{ContentUseCase, MigrationUseCase, SettingsUseCase};
use crate::ports::output::{
    CommentRepository, MigrationRepository, OptionRepository, PostRepository, TaxonomyRepository,
};
use std::sync::Arc;

/// Core CMS Application Orchestrator wired with trait-object output ports.
#[derive(Clone)]
pub struct CmsService {
    posts: Arc<dyn PostRepository>,
    taxonomies: Arc<dyn TaxonomyRepository>,
    options: Arc<dyn OptionRepository>,
    comments: Arc<dyn CommentRepository>,
    migrations: Arc<dyn MigrationRepository>,
}

impl CmsService {
    pub fn new(
        posts: Arc<dyn PostRepository>,
        taxonomies: Arc<dyn TaxonomyRepository>,
        options: Arc<dyn OptionRepository>,
        comments: Arc<dyn CommentRepository>,
        migrations: Arc<dyn MigrationRepository>,
    ) -> Self {
        Self {
            posts,
            taxonomies,
            options,
            comments,
            migrations,
        }
    }

    /// Convenience constructor for any unified storage adapter that implements all output ports.
    pub fn from_adapter<R>(adapter: R) -> Self
    where
        R: PostRepository
            + TaxonomyRepository
            + OptionRepository
            + CommentRepository
            + MigrationRepository
            + Clone
            + 'static,
    {
        Self {
            posts: Arc::new(adapter.clone()),
            taxonomies: Arc::new(adapter.clone()),
            options: Arc::new(adapter.clone()),
            comments: Arc::new(adapter.clone()),
            migrations: Arc::new(adapter),
        }
    }

    pub fn posts_repo(&self) -> &Arc<dyn PostRepository> {
        &self.posts
    }

    pub fn taxonomies_repo(&self) -> &Arc<dyn TaxonomyRepository> {
        &self.taxonomies
    }

    pub fn options_repo(&self) -> &Arc<dyn OptionRepository> {
        &self.options
    }

    pub fn comments_repo(&self) -> &Arc<dyn CommentRepository> {
        &self.comments
    }

    pub fn migrations_repo(&self) -> &Arc<dyn MigrationRepository> {
        &self.migrations
    }
}

impl ContentUseCase for CmsService {
    fn list_content(
        &self,
        post_type: &str,
        status: Option<&str>,
        search: Option<&str>,
        limit: i64,
    ) -> DomainResult<Vec<WpPostResponse>> {
        let clamped_limit = limit.clamp(1, 500);
        self.posts
            .list_posts(post_type, status, search, clamped_limit)
    }

    fn get_content_by_id(&self, id: i64) -> DomainResult<WpPostResponse> {
        self.posts
            .find_post_by_id(id)?
            .ok_or_else(|| DomainError::NotFound(format!("Post #{id} not found")))
    }

    fn get_content_by_slug(&self, slug: &str) -> DomainResult<WpPostResponse> {
        let normalized = slug.trim();
        if normalized.is_empty() {
            return Err(DomainError::Validation("Post slug cannot be empty".into()));
        }
        self.posts.find_post_by_slug(normalized)?.ok_or_else(|| {
            DomainError::NotFound(format!("Post with slug '{normalized}' not found"))
        })
    }

    fn publish_content(&self, mut req: CreatePostRequest) -> DomainResult<WpPostResponse> {
        if req.title.trim().is_empty() && req.content.trim().is_empty() {
            return Err(DomainError::Validation(
                "A post must include a non-empty title or content".into(),
            ));
        }
        if req.post_type.trim().is_empty() {
            req.post_type = "post".to_string();
        }
        if req.status.trim().is_empty() {
            req.status = "publish".to_string();
        }
        if req.slug.as_deref().map(str::trim).unwrap_or("").is_empty() {
            req.slug = Some(generated_slugify::slugify(&req.title));
        }
        self.posts.create_post(&req)
    }

    fn edit_content(&self, id: i64, req: UpdatePostRequest) -> DomainResult<WpPostResponse> {
        self.posts
            .update_post(id, &req)?
            .ok_or_else(|| DomainError::NotFound(format!("Post #{id} not found")))
    }

    fn remove_content(&self, id: i64, force: bool) -> DomainResult<bool> {
        if force {
            self.posts.delete_post(id)
        } else {
            Ok(self.posts.trash_post(id)?.is_some())
        }
    }

    fn get_layout_ast(&self, post_id: i64) -> DomainResult<Option<String>> {
        self.posts.get_post_layout(post_id)
    }

    fn update_layout_ast(&self, post_id: i64, layout_json: &str) -> DomainResult<()> {
        let layout: neopress_ast::NeopressLayout =
            serde_json::from_str(layout_json).map_err(|e| {
                DomainError::Validation(format!("Invalid NeopressLayout AST JSON: {e}"))
            })?;
        if let Some(ref gs_val) = layout.global_styles {
            if let Ok(kit) = serde_json::from_value::<neopress_ast::ElementorKit>(gs_val.clone()) {
                let kit_css = kit.compile_to_css();
                let _ = self.options.set_option("elementor_kit_css", &kit_css);
            }
        }
        let compiled_html = neopress_ast::render_layout_to_html(&layout);
        let canonical_json = serde_json::to_string(&layout)
            .map_err(|e| DomainError::Internal(format!("Failed to serialize AST layout: {e}")))?;
        self.posts
            .save_post_layout(post_id, &canonical_json, Some(&compiled_html))
    }

    fn list_taxonomies(&self, taxonomy: &str) -> DomainResult<Vec<WpCategoryResponse>> {
        self.taxonomies.list_terms(taxonomy)
    }

    fn create_taxonomy_term(
        &self,
        taxonomy: &str,
        req: CreateTermRequest,
    ) -> DomainResult<WpCategoryResponse> {
        if req.name.trim().is_empty() {
            return Err(DomainError::Validation("Term name is required".into()));
        }
        self.taxonomies.create_term(taxonomy, &req)
    }
}

impl MigrationUseCase for CmsService {
    fn migrate_wordpress_bundle(
        &self,
        bundle: ImportBundleRequest,
    ) -> DomainResult<ImportBundleResponse> {
        self.migrations.import_bundle(&bundle)
    }

    fn migrate_wxr_xml(&self, xml_bytes: &[u8]) -> DomainResult<WxrImportResponse> {
        if xml_bytes.is_empty() {
            return Err(DomainError::Validation("WXR XML payload is empty".into()));
        }
        self.migrations.import_wxr_xml(xml_bytes)
    }

    fn export_site_wxr(&self) -> DomainResult<String> {
        self.migrations.export_wxr_xml()
    }
}

impl SettingsUseCase for CmsService {
    fn get_site_settings(&self) -> DomainResult<WpSettingsResponse> {
        self.options.get_settings()
    }

    fn update_site_option(&self, name: &str, value: &str) -> DomainResult<()> {
        let key = name.trim();
        if key.is_empty() {
            return Err(DomainError::Validation(
                "Option name cannot be empty".into(),
            ));
        }
        self.options.set_option(key, value)
    }

    fn get_site_option(&self, name: &str) -> DomainResult<Option<String>> {
        self.options.get_option(name.trim())
    }

    fn get_system_stats(&self) -> DomainResult<DbStats> {
        self.options.get_stats()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    /// Pure in-memory mock repository proving `CmsService` is 100% decoupled from SQLite and Axum.
    #[derive(Default)]
    struct MockRepo {
        posts: Mutex<Vec<WpPostResponse>>,
    }

    impl PostRepository for MockRepo {
        fn list_posts(
            &self,
            _post_type: &str,
            _status: Option<&str>,
            _search: Option<&str>,
            _limit: i64,
        ) -> DomainResult<Vec<WpPostResponse>> {
            Ok(self.posts.lock().unwrap().clone())
        }

        fn find_post_by_id(&self, id: i64) -> DomainResult<Option<WpPostResponse>> {
            Ok(self
                .posts
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.id == id)
                .cloned())
        }

        fn find_post_by_slug(&self, slug: &str) -> DomainResult<Option<WpPostResponse>> {
            Ok(self
                .posts
                .lock()
                .unwrap()
                .iter()
                .find(|p| p.slug == slug)
                .cloned())
        }

        fn create_post(&self, req: &CreatePostRequest) -> DomainResult<WpPostResponse> {
            let mut guard = self.posts.lock().unwrap();
            let id = (guard.len() as i64) + 1;
            let slug = req
                .slug
                .clone()
                .unwrap_or_else(|| generated_slugify::slugify(&req.title));
            let post = WpPostResponse {
                id,
                date: "2026-10-22T00:00:00".into(),
                date_gmt: "2026-10-22T00:00:00".into(),
                guid: WpGuid {
                    rendered: format!("http://localhost:8090/?p={id}"),
                },
                modified: "2026-10-22T00:00:00".into(),
                modified_gmt: "2026-10-22T00:00:00".into(),
                slug,
                status: req.status.clone(),
                post_type: req.post_type.clone(),
                link: format!("http://localhost:8090/?p={id}"),
                title: WpRenderedText {
                    rendered: req.title.clone(),
                    protected: false,
                },
                content: WpRenderedText {
                    rendered: req.content.clone(),
                    protected: false,
                },
                excerpt: WpRenderedText {
                    rendered: req.excerpt.clone().unwrap_or_default(),
                    protected: false,
                },
                author: 1,
                featured_media: 0,
                comment_status: "open".into(),
                ping_status: "open".into(),
                sticky: false,
                template: String::new(),
                format: "standard".into(),
                meta: serde_json::json!({}),
                categories: vec![1],
                tags: vec![],
                parent: 0,
                menu_order: 0,
                author_name: "admin".into(),
                category_names: vec![],
                tag_names: vec![],
                featured_image_url: None,
                comments: vec![],
                seo_title: String::new(),
                seo_description: String::new(),
                seo_keyword: String::new(),
                seo_canonical: String::new(),
                seo_noindex: false,
                seo_schema_type: "Article".into(),
                seo_og_image: String::new(),
            };
            guard.push(post.clone());
            Ok(post)
        }

        fn update_post(
            &self,
            _id: i64,
            _req: &UpdatePostRequest,
        ) -> DomainResult<Option<WpPostResponse>> {
            Ok(None)
        }

        fn duplicate_post(&self, _source_id: i64) -> DomainResult<Option<WpPostResponse>> {
            Ok(None)
        }

        fn trash_post(&self, _id: i64) -> DomainResult<Option<WpPostResponse>> {
            Ok(None)
        }

        fn delete_post(&self, _id: i64) -> DomainResult<bool> {
            Ok(true)
        }

        fn get_post_layout(&self, _post_id: i64) -> DomainResult<Option<String>> {
            Ok(None)
        }

        fn save_post_layout(
            &self,
            _post_id: i64,
            _layout_json: &str,
            _compiled_html: Option<&str>,
        ) -> DomainResult<()> {
            Ok(())
        }

        fn delete_post_layout(&self, _post_id: i64) -> DomainResult<()> {
            Ok(())
        }
    }

    impl TaxonomyRepository for MockRepo {
        fn list_terms(&self, _taxonomy: &str) -> DomainResult<Vec<WpCategoryResponse>> {
            Ok(vec![])
        }
        fn find_term_by_id(
            &self,
            _taxonomy: &str,
            _id: i64,
        ) -> DomainResult<Option<WpCategoryResponse>> {
            Ok(None)
        }
        fn create_term(
            &self,
            _taxonomy: &str,
            _req: &CreateTermRequest,
        ) -> DomainResult<WpCategoryResponse> {
            Err(DomainError::NotFound("mock".into()))
        }
        fn update_term(
            &self,
            _taxonomy: &str,
            _id: i64,
            _req: &UpdateTermRequest,
        ) -> DomainResult<Option<WpCategoryResponse>> {
            Ok(None)
        }
        fn delete_term(&self, _taxonomy: &str, _id: i64) -> DomainResult<bool> {
            Ok(false)
        }
    }

    impl OptionRepository for MockRepo {
        fn get_option(&self, _name: &str) -> DomainResult<Option<String>> {
            Ok(None)
        }
        fn set_option(&self, _name: &str, _value: &str) -> DomainResult<()> {
            Ok(())
        }
        fn get_settings(&self) -> DomainResult<WpSettingsResponse> {
            Ok(WpSettingsResponse::default())
        }
        fn get_stats(&self) -> DomainResult<DbStats> {
            Err(DomainError::NotFound("mock".into()))
        }
    }

    impl CommentRepository for MockRepo {
        fn list_comments(
            &self,
            _post_id: Option<i64>,
            _status: Option<&str>,
            _limit: i64,
        ) -> DomainResult<Vec<WpCommentAdminItem>> {
            Ok(vec![])
        }
        fn find_comment_by_id(&self, _id: i64) -> DomainResult<Option<WpCommentAdminItem>> {
            Ok(None)
        }
        fn create_comment(&self, _req: &CreateCommentRequest) -> DomainResult<WpCommentResponse> {
            Err(DomainError::NotFound("mock".into()))
        }
        fn update_comment(
            &self,
            _id: i64,
            _req: &UpdateCommentRequest,
        ) -> DomainResult<Option<WpCommentAdminItem>> {
            Ok(None)
        }
        fn delete_comment(&self, _id: i64) -> DomainResult<bool> {
            Ok(false)
        }
    }

    impl MigrationRepository for MockRepo {
        fn import_bundle(&self, _req: &ImportBundleRequest) -> DomainResult<ImportBundleResponse> {
            Err(DomainError::NotFound("mock".into()))
        }
        fn import_wxr_xml(&self, _xml_bytes: &[u8]) -> DomainResult<WxrImportResponse> {
            Err(DomainError::NotFound("mock".into()))
        }
        fn export_wxr_xml(&self) -> DomainResult<String> {
            Ok(String::new())
        }
    }

    #[test]
    fn test_hexagonal_cms_service_with_pure_mock_repo() {
        let mock = Arc::new(MockRepo::default());
        let service = CmsService::new(mock.clone(), mock.clone(), mock.clone(), mock.clone(), mock);

        // Rejects empty post
        let empty_res = service.publish_content(CreatePostRequest::default());
        assert!(matches!(empty_res, Err(DomainError::Validation(_))));

        // Publishes valid post and auto-slugifies title
        let created = service
            .publish_content(CreatePostRequest {
                title: "Hexagonal Architecture in Rust".into(),
                content: "<p>Zero SQLite or Axum coupling!</p>".into(),
                ..Default::default()
            })
            .expect("publish should succeed");

        assert_eq!(created.id, 1);
        assert_eq!(created.slug, "hexagonal-architecture-in-rust");

        let fetched = service
            .get_content_by_slug("hexagonal-architecture-in-rust")
            .expect("should find by slug");
        assert_eq!(fetched.title.rendered, "Hexagonal Architecture in Rust");
    }
}
