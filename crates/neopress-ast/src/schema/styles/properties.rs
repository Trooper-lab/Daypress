use super::background::BackgroundConfig;
use super::border::BorderConfig;
use super::spacing::Spacing;
use super::typography::TypographyConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Comprehensive style specification supporting desktop, tablet, mobile, hover,
/// CSS Grid, Flexbox, Box-Model, transforms, and third-party fallback settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct StyleProperties {
    // -------------------------------------------------------------
    // Nested Canonical AST Style Objects
    // -------------------------------------------------------------
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding: Option<Spacing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin: Option<Spacing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub typography: Option<TypographyConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background: Option<BackgroundConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border: Option<BorderConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_css: Option<String>,

    // -------------------------------------------------------------
    // Responsive & Pseudo-State Overrides (R1 / R4)
    // -------------------------------------------------------------
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tablet: Option<Box<StyleProperties>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mobile: Option<Box<StyleProperties>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hover: Option<Box<StyleProperties>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub focus: Option<Box<StyleProperties>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active: Option<Box<StyleProperties>>,

    // -------------------------------------------------------------
    // CSS Grid Layout Controls (R1 / R4)
    // -------------------------------------------------------------
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_template_columns: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_template_rows: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_gap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_column: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_row: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grid_auto_flow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub justify_items: Option<String>,

    // -------------------------------------------------------------
    // Positioning & Box-Model Offsets (R1 / R4)
    // -------------------------------------------------------------
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<String>, // "static", "relative", "absolute", "fixed", "sticky"
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bottom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub z_index: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub overflow: Option<String>,

    // -------------------------------------------------------------
    // Sizing & Box Sizing
    // -------------------------------------------------------------
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect_ratio: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object_fit: Option<String>,

    // -------------------------------------------------------------
    // Flexbox Layout Controls
    // -------------------------------------------------------------
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flex_direction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flex_wrap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub justify_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align_items: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align_self: Option<String>,

    // -------------------------------------------------------------
    // Flat Visual Builder Style Fields
    // -------------------------------------------------------------
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_image: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_gradient: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_position: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_repeat: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_attachment: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_family: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_transform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_decoration: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_shadow: Option<String>,

    // Flat Spacing & Borders
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_top: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_right: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_bottom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub padding_left: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin_top: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin_right: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin_bottom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub margin_left: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_style: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_top: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_right: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_bottom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_left: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub box_shadow: Option<String>,

    // Transforms & Visual Effects
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opacity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transform: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transition: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub backdrop_filter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,

    // Responsive Visibility Toggles
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_desktop: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_tablet: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hide_mobile: Option<bool>,

    // Utility & Classes
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub css_classes: Option<String>,
    /// Legacy NeoStudio class list key (pre-`css_classes`); merged into `css_classes` at render time.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custom_classes: Option<String>,

    // Lossless Raw Settings Fallback (R4)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub raw_settings: Option<HashMap<String, serde_json::Value>>,
}
