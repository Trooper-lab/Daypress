use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpUserResponse {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub description: String,
    pub link: String,
    pub slug: String,
    pub avatar_urls: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WpUserDetail {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub url: String,
    pub registered: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserListItem {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub url: String,
    pub registered: String,
    pub role: String,
    pub post_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub username: String,
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UpdateUserRequest {
    pub display_name: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    pub email: Option<String>,
    pub url: Option<String>,
    pub password: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub first_name: Option<String>,
    #[serde(default)]
    pub last_name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}
