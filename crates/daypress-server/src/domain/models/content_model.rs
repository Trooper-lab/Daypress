use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomPostTypeDefinition {
    pub slug: String,
    #[serde(default, alias = "singular_label", alias = "singular_name")]
    pub label_singular: String,
    #[serde(default, alias = "plural_label", alias = "plural_name")]
    pub label_plural: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_cpt_icon", alias = "menu_icon")]
    pub icon: String,
    #[serde(default = "default_cpt_menu_pos")]
    pub menu_position: i64,
    #[serde(default = "default_true")]
    pub public: bool,
    #[serde(default = "default_true")]
    pub has_archive: bool,
    #[serde(default)]
    pub hierarchical: bool,
    #[serde(default)]
    pub rewrite_slug: String,
    #[serde(default = "default_cpt_supports")]
    pub supports: Vec<String>,
    #[serde(default)]
    pub taxonomies: Vec<String>,
    #[serde(default = "default_true")]
    pub show_in_rest: bool,
    #[serde(default)]
    pub preset_key: Option<String>,
    #[serde(default)]
    pub post_count: i64,
}

fn default_cpt_icon() -> String {
    "fa-solid fa-folder-open".to_string()
}

fn default_cpt_menu_pos() -> i64 {
    25
}

fn default_true() -> bool {
    true
}

fn default_cpt_supports() -> Vec<String> {
    vec![
        "title".to_string(),
        "editor".to_string(),
        "thumbnail".to_string(),
        "excerpt".to_string(),
        "custom-fields".to_string(),
    ]
}

impl Default for CustomPostTypeDefinition {
    fn default() -> Self {
        Self {
            slug: String::new(),
            label_singular: String::new(),
            label_plural: String::new(),
            description: String::new(),
            icon: default_cpt_icon(),
            menu_position: default_cpt_menu_pos(),
            public: true,
            has_archive: true,
            hierarchical: false,
            rewrite_slug: String::new(),
            supports: default_cpt_supports(),
            taxonomies: Vec::new(),
            show_in_rest: true,
            preset_key: None,
            post_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomTaxonomyDefinition {
    pub slug: String,
    #[serde(default, alias = "singular_label", alias = "singular_name")]
    pub label_singular: String,
    #[serde(default, alias = "plural_label", alias = "plural_name")]
    pub label_plural: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_true")]
    pub hierarchical: bool,
    #[serde(default = "default_true")]
    pub public: bool,
    #[serde(default)]
    pub rewrite_slug: String,
    #[serde(default)]
    pub post_types: Vec<String>,
    #[serde(default = "default_true")]
    pub show_in_rest: bool,
    #[serde(default)]
    pub term_count: i64,
}

impl Default for CustomTaxonomyDefinition {
    fn default() -> Self {
        Self {
            slug: String::new(),
            label_singular: String::new(),
            label_plural: String::new(),
            description: String::new(),
            hierarchical: true,
            public: true,
            rewrite_slug: String::new(),
            post_types: Vec::new(),
            show_in_rest: true,
            term_count: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StructuredFieldSchema {
    #[serde(default)]
    pub key: String,
    pub name: String,
    pub label: String,
    #[serde(default = "default_field_type", alias = "type")]
    pub field_type: String,
    #[serde(default)]
    pub instructions: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default_value: String,
    #[serde(default)]
    pub placeholder: String,
    #[serde(default)]
    pub choices: Vec<String>,
    #[serde(default)]
    pub target_post_type: Option<String>,
    #[serde(default)]
    pub sub_fields: Vec<StructuredFieldSchema>,
    #[serde(default)]
    pub show_in_collection_card: bool,
}

fn default_field_type() -> String {
    "text".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldGroupLocationRule {
    #[serde(default = "default_location_param")]
    pub param: String,
    #[serde(default = "default_location_operator")]
    pub operator: String,
    pub value: String,
}

fn default_location_param() -> String {
    "post_type".to_string()
}

fn default_location_operator() -> String {
    "==".to_string()
}

impl Default for FieldGroupLocationRule {
    fn default() -> Self {
        Self {
            param: default_location_param(),
            operator: default_location_operator(),
            value: "post".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldGroupDefinition {
    #[serde(default)]
    pub id: i64,
    #[serde(default)]
    pub group_key: String,
    pub title: String,
    #[serde(default)]
    pub slug: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub location_rules: Vec<FieldGroupLocationRule>,
    #[serde(default)]
    pub fields: Vec<StructuredFieldSchema>,
    #[serde(default = "default_group_position")]
    pub position: String,
    #[serde(default)]
    pub menu_order: i64,
    #[serde(default = "default_true")]
    pub is_active: bool,
}

fn default_group_position() -> String {
    "normal".to_string()
}

impl Default for FieldGroupDefinition {
    fn default() -> Self {
        Self {
            id: 0,
            group_key: String::new(),
            title: String::new(),
            slug: String::new(),
            description: String::new(),
            location_rules: Vec::new(),
            fields: Vec::new(),
            position: default_group_position(),
            menu_order: 0,
            is_active: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionPresetRequest {
    pub preset_key: String,
}
