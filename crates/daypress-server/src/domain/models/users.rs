use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaypressUserResponse {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub description: String,
    pub link: String,
    pub slug: String,
    pub avatar_urls: serde_json::Value,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub locale: String,
    #[serde(default)]
    pub roles: Vec<String>,
}

fn default_true_flag() -> bool {
    true
}

fn default_admin_color() -> String {
    "default".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaypressUserDetail {
    pub id: i64,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub url: String,
    pub registered: String,
    pub role: String,
    #[serde(default)]
    pub first_name: String,
    #[serde(default)]
    pub last_name: String,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub locale: String,
    #[serde(default = "default_true_flag")]
    pub rich_editing: bool,
    #[serde(default = "default_true_flag")]
    pub syntax_highlighting: bool,
    #[serde(default)]
    pub comment_shortcuts: bool,
    #[serde(default = "default_admin_color")]
    pub admin_color: String,
    #[serde(default = "default_true_flag")]
    pub show_admin_bar_front: bool,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub session_ver: i64,
}

impl Default for DaypressUserDetail {
    fn default() -> Self {
        Self {
            id: 1,
            username: "admin".to_string(),
            display_name: "Administrator".to_string(),
            email: "admin@example.com".to_string(),
            url: String::new(),
            registered: "2026-01-01 00:00:00".to_string(),
            role: "administrator".to_string(),
            first_name: String::new(),
            last_name: String::new(),
            nickname: "admin".to_string(),
            description: String::new(),
            locale: String::new(),
            rich_editing: true,
            syntax_highlighting: true,
            comment_shortcuts: false,
            admin_color: "default".to_string(),
            show_admin_bar_front: true,
            avatar_url: String::new(),
            session_ver: 0,
        }
    }
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
    #[serde(default)]
    pub avatar_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub nickname: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
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
    pub nickname: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub locale: Option<String>,
    #[serde(default)]
    pub rich_editing: Option<bool>,
    #[serde(default)]
    pub syntax_highlighting: Option<bool>,
    #[serde(default)]
    pub comment_shortcuts: Option<bool>,
    #[serde(default)]
    pub admin_color: Option<String>,
    #[serde(default)]
    pub show_admin_bar_front: Option<bool>,
    #[serde(default)]
    pub avatar_url: Option<String>,
}
