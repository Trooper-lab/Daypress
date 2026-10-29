//! Domain Models, Enums, and REST DTOs for the Theme Template Builder.

use daypress_ast::DaypressLayout;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Type of site template managed by the Theme Builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateType {
    Header,
    Footer,
    SinglePost,
    SinglePage,
    Archive,
    #[serde(rename = "404_page", alias = "error404", alias = "not_found")]
    Error404,
    Popup,
    SectionSymbol,
}

impl TemplateType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Header => "header",
            Self::Footer => "footer",
            Self::SinglePost => "single_post",
            Self::SinglePage => "single_page",
            Self::Archive => "archive",
            Self::Error404 => "404_page",
            Self::Popup => "popup",
            Self::SectionSymbol => "section_symbol",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "header" => Some(Self::Header),
            "footer" => Some(Self::Footer),
            "single_post" | "single-post" | "single" | "post" => Some(Self::SinglePost),
            "single_page" | "single-page" | "page" => Some(Self::SinglePage),
            "archive" | "category" | "tag" | "blog" => Some(Self::Archive),
            "404_page" | "404-page" | "404" | "error404" | "not_found" => Some(Self::Error404),
            "popup" | "modal" => Some(Self::Popup),
            "section_symbol" | "section-symbol" | "symbol" | "section" => Some(Self::SectionSymbol),
            _ => None,
        }
    }
}

/// Action to take when a display condition matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    Include,
    Exclude,
}

impl ConditionType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Include => "include",
            Self::Exclude => "exclude",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "include" | "in" => Some(Self::Include),
            "exclude" | "ex" => Some(Self::Exclude),
            _ => None,
        }
    }

    pub fn is_include(&self) -> bool {
        matches!(self, Self::Include)
    }

    pub fn is_exclude(&self) -> bool {
        matches!(self, Self::Exclude)
    }
}

/// Target scope for a display condition rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuleTarget {
    EntireSite,
    FrontPage,
    AllPosts,
    AllPages,
    #[serde(alias = "single_post", alias = "post_id")]
    Post,
    #[serde(alias = "single_page", alias = "page_id")]
    Page,
    Category,
    Tag,
    Author,
    Archive,
    #[serde(rename = "not_found", alias = "error_404", alias = "404")]
    NotFound,
}

impl RuleTarget {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EntireSite => "entire_site",
            Self::FrontPage => "front_page",
            Self::AllPosts => "all_posts",
            Self::AllPages => "all_pages",
            Self::Post => "post",
            Self::Page => "page",
            Self::Category => "category",
            Self::Tag => "tag",
            Self::Author => "author",
            Self::Archive => "archive",
            Self::NotFound => "not_found",
        }
    }

    pub fn from_str_loose(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "entire_site" | "entire-site" | "site" | "global" => Some(Self::EntireSite),
            "front_page" | "front-page" | "home" => Some(Self::FrontPage),
            "all_posts" | "all-posts" | "posts" => Some(Self::AllPosts),
            "all_pages" | "all-pages" | "pages" => Some(Self::AllPages),
            "post" | "single_post" | "post_id" => Some(Self::Post),
            "page" | "single_page" | "page_id" => Some(Self::Page),
            "category" => Some(Self::Category),
            "tag" => Some(Self::Tag),
            "author" => Some(Self::Author),
            "archive" => Some(Self::Archive),
            "not_found" | "not-found" | "404" | "error_404" => Some(Self::NotFound),
            _ => None,
        }
    }

    pub fn base_specificity_score(&self) -> i32 {
        match self {
            Self::Post | Self::Page | Self::FrontPage | Self::NotFound => 40,
            Self::Category | Self::Tag | Self::Author => 30,
            Self::AllPosts | Self::AllPages | Self::Archive => 20,
            Self::EntireSite => 10,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Granular, typed settings for templates (deserialized from `settings_json`).
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct TemplateSettings {
    #[serde(default)]
    pub sticky: bool,
    #[serde(default)]
    pub transparent: bool,
    #[serde(default)]
    pub scroll_shrink: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shrink_height_px: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z_index: Option<i32>,

    #[serde(default)]
    pub sticky_footer: bool,
    #[serde(default)]
    pub reveal_on_scroll: bool,

    #[serde(default = "default_true")]
    pub comments_enabled: bool,
    #[serde(default = "default_true")]
    pub show_featured_hero: bool,
    #[serde(default)]
    pub full_width_canvas: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub posts_per_page: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pagination_type: Option<String>,

    #[serde(default)]
    pub suggest_recent_posts: bool,
    #[serde(default = "default_true")]
    pub show_search_form: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delay_sec: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scroll_pct: Option<u32>,
    #[serde(default)]
    pub backdrop_blur: bool,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_css: Option<String>,
    #[serde(default, flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

fn default_condition_priority() -> i32 {
    10
}

/// A priority-ranked display condition row.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TemplateCondition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub template_id: i64,
    pub condition_type: ConditionType,
    pub rule_target: RuleTarget,
    #[serde(default)]
    pub target_id: String,
    #[serde(default = "default_condition_priority")]
    pub priority: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
}

/// A fully hydrated Theme Builder template entity.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ThemeTemplateRecord {
    pub id: i64,
    pub title: String,
    pub template_type: TemplateType,
    pub status: String,
    pub layout: DaypressLayout,
    pub rendered_html: Option<String>,
    pub settings: TemplateSettings,
    pub priority: i32,
    pub is_active: bool,
    pub conditions: Vec<TemplateCondition>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TaxonomyTermRef {
    pub taxonomy: String,
    pub term_id: i64,
    #[serde(default)]
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BreadcrumbItem {
    pub title: String,
    pub url: String,
    pub is_current: bool,
}

/// Representation of an incoming request context used by the condition engine.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RequestContext {
    pub path: String,
    pub post_id: Option<i64>,
    pub post_type: Option<String>,
    pub is_front_page: bool,
    pub is_archive: bool,
    pub is_404: bool,
    pub is_search: bool,
    pub taxonomy_terms: Vec<TaxonomyTermRef>,
    pub author_id: Option<i64>,
}

impl RequestContext {
    pub fn for_front_page() -> Self {
        Self {
            path: "/".to_string(),
            is_front_page: true,
            ..Default::default()
        }
    }

    pub fn for_front_page_post(post_id: i64, post_type: &str) -> Self {
        Self {
            path: "/".to_string(),
            post_id: Some(post_id),
            post_type: Some(post_type.to_string()),
            is_front_page: true,
            ..Default::default()
        }
    }

    pub fn for_post(
        post_id: i64,
        post_type: &str,
        is_front: bool,
        terms: Vec<TaxonomyTermRef>,
    ) -> Self {
        Self {
            post_id: Some(post_id),
            post_type: Some(post_type.to_string()),
            is_front_page: is_front,
            taxonomy_terms: terms,
            ..Default::default()
        }
    }

    pub fn for_archive(taxonomy: &str, term_id: i64, slug: &str) -> Self {
        Self {
            is_archive: true,
            taxonomy_terms: vec![TaxonomyTermRef {
                taxonomy: taxonomy.to_string(),
                term_id,
                slug: slug.to_string(),
            }],
            ..Default::default()
        }
    }

    pub fn for_404(path: &str) -> Self {
        Self {
            path: path.to_string(),
            is_404: true,
            ..Default::default()
        }
    }

    pub fn build_breadcrumbs(&self) -> Vec<BreadcrumbItem> {
        let mut items = Vec::new();
        if self.is_front_page {
            return items;
        }
        if self.is_404 {
            items.push(BreadcrumbItem {
                title: "404 Not Found".to_string(),
                url: self.path.clone(),
                is_current: true,
            });
            return items;
        }
        for term in &self.taxonomy_terms {
            items.push(BreadcrumbItem {
                title: if term.slug.is_empty() {
                    format!("Term {}", term.term_id)
                } else {
                    term.slug.clone()
                },
                url: format!("/{}/{}", term.taxonomy, term.slug),
                is_current: false,
            });
        }
        if let Some(pid) = self.post_id {
            items.push(BreadcrumbItem {
                title: format!("Item {}", pid),
                url: self.path.clone(),
                is_current: true,
            });
        }
        items
    }
}

/// Resolved dynamic theme partials ready to be injected into SSR document shell.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResolvedThemePartials {
    pub header_html: Option<String>,
    pub header_css: Option<String>,
    pub header_sticky: bool,
    pub header_transparent: bool,
    pub footer_html: Option<String>,
    pub footer_css: Option<String>,
    pub single_html: Option<String>,
    pub single_css: Option<String>,
    pub archive_html: Option<String>,
    pub archive_css: Option<String>,
    #[serde(default)]
    pub has_theme_builder_header: bool,
    #[serde(default)]
    pub has_theme_builder_footer: bool,
    #[serde(default)]
    pub header_hidden: bool,
    #[serde(default)]
    pub footer_hidden: bool,
    #[serde(default)]
    pub header_force_fallback: bool,
    #[serde(default)]
    pub footer_force_fallback: bool,
}

pub use super::dto::*;
