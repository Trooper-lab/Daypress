use crate::schema::layout::DaypressLayout;
use serde::{Deserialize, Serialize};

/// Type of site template managed by the Theme Builder.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemplateType {
    Header,
    Footer,
    SinglePost,
    SinglePage,
    Archive,
    Error404,
    Popup,
    SectionSymbol,
}

/// Action to take when a display condition matches.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConditionType {
    Include,
    Exclude,
}

/// Scope target for display conditions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "target")]
pub enum ConditionScope {
    EntireSite,
    FrontPage,
    AllPosts,
    AllPages,
    SpecificPost(i64),
    SpecificPage(i64),
    TaxonomyTerm { taxonomy: String, term_id: i64 },
    Archive,
    Error404,
}

/// A priority-ranked display condition rule assigned to a ThemeTemplate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DisplayCondition {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub template_id: i64,
    pub condition_type: ConditionType,
    pub scope: ConditionScope,
    #[serde(default)]
    pub priority: i32,
}

/// A Theme Builder template record stored in SQLite and hydrated at SSR time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ThemeTemplate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
    pub title: String,
    pub template_type: TemplateType,
    pub layout: DaypressLayout,
    #[serde(default = "default_publish_status")]
    pub status: String, // "publish" or "draft"
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub sticky: bool,
    #[serde(default)]
    pub transparent: bool,
    #[serde(default)]
    pub conditions: Vec<DisplayCondition>,
}

fn default_publish_status() -> String {
    "publish".to_string()
}
