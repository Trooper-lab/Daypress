use serde::{Deserialize, Serialize};

/// Accordion item containing an expandable title and HTML body content.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AccordionItem {
    #[serde(default)]
    pub title: String,
    #[serde(default, alias = "content")]
    pub content_html: String,
}

/// Tab item containing a tab label and tab pane HTML content.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TabItem {
    #[serde(default)]
    pub title: String,
    #[serde(default, alias = "content")]
    pub content_html: String,
}

/// Toggle item containing a collapsible title and body content.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ToggleItem {
    #[serde(default)]
    pub title: String,
    #[serde(default, alias = "content")]
    pub content_html: String,
}

/// Icon list item for bulleted icon feature lists.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct IconListItem {
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}

/// Social icon item for platform profiles.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SocialIconItem {
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub link: String,
}

/// Share platform item for social share buttons.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SharePlatformItem {
    #[serde(default)]
    pub platform: String, // "twitter" / "x", "facebook", "linkedin", "whatsapp", "email", etc.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url_template: Option<String>,
}

/// Pricing feature item with inclusion toggle and optional check/cross icon.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct PriceFeatureItem {
    #[serde(default)]
    pub text: String,
    #[serde(default = "default_true")]
    pub included: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

/// Testimonial slide for testimonial carousels.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TestimonialSlide {
    #[serde(default, alias = "content")]
    pub quote: String,
    #[serde(default)]
    pub author: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub company: Option<String>,
    #[serde(default, alias = "image", skip_serializing_if = "Option::is_none")]
    pub avatar_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating: Option<f32>,
}

/// Navigation menu item for WordPress/Daypress navigation menus.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct NavMenuItem {
    #[serde(default)]
    pub id: i64,
    #[serde(default, alias = "label")]
    pub title: String,
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub target: String,
    #[serde(default)]
    pub parent: i64,
    #[serde(default)]
    pub order: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<NavMenuItem>,
}

fn default_true() -> bool {
    true
}
