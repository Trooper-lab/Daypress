use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaypressCategoryResponse {
    pub id: i64,
    pub count: i64,
    pub description: String,
    pub link: String,
    pub name: String,
    pub slug: String,
    pub taxonomy: String,
    pub parent: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CreateTermRequest {
    pub name: String,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parent: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateTermRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub slug: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub parent: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaxonomyInfo {
    pub slug: String,
    pub name: String,
    pub hierarchical: bool,
    pub terms: Vec<DaypressCategoryResponse>,
    pub assigned_term_ids: Vec<i64>,
}
