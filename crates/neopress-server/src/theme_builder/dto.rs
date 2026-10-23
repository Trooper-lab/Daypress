//! Theme Builder REST API Data Transfer Objects (DTOs).

use super::models::{ConditionType, RuleTarget, TemplateSettings, TemplateType, ThemeTemplateRecord};
use neopress_ast::NeopressLayout;
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
    pub layout: Option<NeopressLayout>,
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
    pub layout: Option<NeopressLayout>,
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
