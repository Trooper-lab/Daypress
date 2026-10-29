//! Theme Builder REST API Data Transfer Objects (DTOs).

use super::models::{
    ConditionType, RuleTarget, TemplateSettings, TemplateType, ThemeTemplateRecord,
};
use daypress_ast::DaypressLayout;
use serde::{Deserialize, Serialize};

fn default_condition_priority() -> i32 {
    10
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConditionDto {
    #[serde(default)]
    pub id: Option<i64>,
    pub condition_type: ConditionType,
    pub rule_target: RuleTarget,
    #[serde(default)]
    pub target_id: String,
    #[serde(default = "default_condition_priority")]
    pub priority: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateRequest {
    pub title: String,
    pub template_type: TemplateType,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub layout: Option<DaypressLayout>,
    #[serde(default)]
    pub layout_json: Option<serde_json::Value>,
    #[serde(default)]
    pub settings: Option<TemplateSettings>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub conditions: Option<Vec<CreateConditionDto>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTemplateRequest {
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub template_type: Option<TemplateType>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub layout: Option<DaypressLayout>,
    #[serde(default)]
    pub layout_json: Option<serde_json::Value>,
    #[serde(default)]
    pub rendered_html: Option<String>,
    #[serde(default)]
    pub settings: Option<TemplateSettings>,
    #[serde(default)]
    pub priority: Option<i32>,
    #[serde(default)]
    pub is_active: Option<bool>,
    #[serde(default)]
    pub conditions: Option<Vec<CreateConditionDto>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateListItem {
    pub id: i64,
    pub title: String,
    pub template_type: TemplateType,
    pub status: String,
    pub priority: i32,
    pub is_active: bool,
    pub condition_count: usize,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TemplateFilter {
    pub template_type: Option<TemplateType>,
    pub status: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetConditionsPayload {
    pub conditions: Vec<CreateConditionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedTemplatesResponse {
    pub header: Option<ThemeTemplateRecord>,
    pub footer: Option<ThemeTemplateRecord>,
    pub single: Option<ThemeTemplateRecord>,
    pub archive: Option<ThemeTemplateRecord>,
    pub error_404: Option<ThemeTemplateRecord>,
    pub popups: Vec<ThemeTemplateRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarterTemplateInfo {
    pub id: String,
    pub title: String,
    pub template_type: TemplateType,
    pub category: String,
    pub preview_img: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedSlotStatus {
    /// Mode configured on this page/route: "inherit", "template:<id>", "none", "fallback"
    pub mode: String,
    /// Winning template ID if a Theme Builder template won
    pub template_id: Option<i64>,
    /// Winning template title or status label
    pub display_name: String,
    /// Human-readable reason why this resolved
    pub reason: String,
    /// Badge style hint: "custom", "global", "hidden", "fallback"
    pub badge_kind: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageAssignmentRow {
    pub target_kind: String,
    pub post_id: i64,
    pub route_key: String,
    pub title: String,
    pub slug: String,
    pub permalink: String,
    pub status: String,
    pub is_front_page: bool,
    pub in_main_menu: bool,
    pub page_template: String,
    pub header: ResolvedSlotStatus,
    pub footer: ResolvedSlotStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchedPageChip {
    pub post_id: i64,
    pub title: String,
    pub permalink: String,
    pub is_route: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateCoverageSummary {
    pub template_id: i64,
    pub matched_pages_count: usize,
    pub matched_pages: Vec<MatchedPageChip>,
    pub conditions_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderFooterMatrixResponse {
    pub templates: Vec<ThemeTemplateRecord>,
    pub coverage: std::collections::HashMap<i64, TemplateCoverageSummary>,
    pub routes: Vec<PageAssignmentRow>,
    pub pages: Vec<PageAssignmentRow>,
    pub posts: Vec<PageAssignmentRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AssignPageTemplateRequest {
    #[serde(default)]
    pub post_id: Option<i64>,
    #[serde(default)]
    pub route_key: Option<String>,
    #[serde(default)]
    pub header: Option<String>,
    #[serde(default)]
    pub footer: Option<String>,
}
