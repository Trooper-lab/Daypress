use super::defaults::*;
use super::kind::NodeKind;
use crate::schema::items::{
    AccordionItem, IconListItem, NavMenuItem, PriceFeatureItem, SharePlatformItem, SocialIconItem,
    TabItem, TestimonialSlide, ToggleItem,
};
use crate::schema::node::normalize::normalize_node_kind_object;
use crate::schema::styles::StyleProperties;
use serde::de::{self, Deserializer};
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub(crate) enum NodeKindRepr {
    Root,
    Section,
    Container,
    Column,
    Grid {
        #[serde(default = "default_grid_columns")]
        columns: u8,
        #[serde(default)]
        gap: u32,
    },
    Flex {
        #[serde(default = "default_direction")]
        direction: String,
        #[serde(default = "default_justify")]
        justify: String,
        #[serde(default = "default_align")]
        align: String,
        #[serde(default)]
        wrap: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        basis: Option<String>,
    },
    Heading {
        #[serde(default)]
        text: String,
        #[serde(default = "default_heading_level")]
        level: u8,
    },
    Text {
        #[serde(default, alias = "content", alias = "text")]
        content_html: String,
    },
    Image {
        #[serde(default, alias = "url")]
        src: String,
        #[serde(default)]
        alt: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        width: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        height: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        link: Option<String>,
    },
    Button {
        #[serde(default)]
        text: String,
        #[serde(default = "default_button_url")]
        url: String,
        #[serde(default = "default_target")]
        target: String,
    },
    Spacer {
        #[serde(default = "default_spacer_height")]
        height: u32,
    },
    Divider {
        #[serde(default = "default_divider_style")]
        style: String,
    },
    Icon {
        #[serde(default, alias = "icon")]
        name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        size: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color: Option<String>,
    },
    Form {
        #[serde(default = "default_form_id")]
        form_id: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        button_text: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        endpoint: Option<String>,
    },
    NativeForm {
        #[serde(default)]
        title: String,
        #[serde(default)]
        button_text: String,
        #[serde(default)]
        endpoint: String,
    },
    PriceCard {
        #[serde(default)]
        title: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subtitle: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        currency: Option<String>,
        #[serde(default)]
        price: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        original_price: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        period: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        badge: Option<String>,
        #[serde(default)]
        featured: bool,
        #[serde(default)]
        features: Vec<PriceFeatureItem>,
        #[serde(default = "default_button_text")]
        button_text: String,
        #[serde(default = "default_button_url")]
        button_url: String,
    },
    PriceTable {
        #[serde(default)]
        title: String,
        #[serde(default)]
        price: String,
        #[serde(default)]
        period: String,
        #[serde(default)]
        features: Vec<String>,
        #[serde(default = "default_button_text")]
        button_text: String,
        #[serde(default = "default_button_url")]
        button_url: String,
    },
    CallToAction {
        #[serde(default)]
        title: String,
        #[serde(default)]
        description: String,
        #[serde(default = "default_button_text")]
        button_text: String,
        #[serde(default = "default_button_url")]
        button_url: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        skin: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ribbon: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        image_url: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        secondary_button_text: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        secondary_button_url: Option<String>,
    },
    Testimonial {
        #[serde(default)]
        content: String,
        #[serde(default)]
        name: String,
        #[serde(default)]
        job: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        image_url: Option<String>,
    },
    TestimonialCarousel {
        #[serde(default)]
        slides: Vec<TestimonialSlide>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        skin: Option<String>,
        #[serde(default = "default_true")]
        autoplay: bool,
        #[serde(default = "default_true")]
        loop_slides: bool,
    },
    Countdown {
        #[serde(default)]
        target_date: String,
        #[serde(default = "default_countdown_mode")]
        mode: String,
        #[serde(default = "default_true")]
        show_days: bool,
        #[serde(default = "default_true")]
        show_hours: bool,
        #[serde(default = "default_true")]
        show_minutes: bool,
        #[serde(default = "default_true")]
        show_seconds: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expiry_message: Option<String>,
    },
    FlipBox {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        front_icon: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        front_image: Option<String>,
        #[serde(default)]
        front_title: String,
        #[serde(default)]
        front_description: String,
        #[serde(default)]
        back_title: String,
        #[serde(default)]
        back_description: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        button_text: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        button_url: Option<String>,
        #[serde(default = "default_flip_direction")]
        direction: String,
    },
    AnimatedHeadline {
        #[serde(default)]
        before_text: String,
        #[serde(default)]
        words: Vec<String>,
        #[serde(default)]
        after_text: String,
        #[serde(default = "default_animation_style")]
        animation_style: String,
    },
    Rating {
        #[serde(default = "default_rating")]
        rating: f32,
        #[serde(default = "default_max_rating")]
        max_rating: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    StarRating {
        #[serde(default = "default_rating")]
        rating: f32,
        #[serde(default = "default_max_rating")]
        max_rating: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        star_color: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        star_size: Option<u32>,
    },
    ProgressBar {
        #[serde(default)]
        title: String,
        #[serde(default)]
        percent: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        display_text: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        track_color: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        bar_color: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        height: Option<u32>,
    },
    #[serde(alias = "Notice")]
    Alert {
        #[serde(default = "default_alert_type")]
        alert_type: String,
        #[serde(default)]
        title: String,
        #[serde(default)]
        description: String,
        #[serde(default = "default_true")]
        dismissible: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        icon: Option<String>,
    },
    Counter {
        #[serde(
            default = "default_counter_number",
            deserialize_with = "deserialize_counter_number"
        )]
        number: i64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prefix: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        suffix: Option<String>,
        #[serde(default, alias = "label", skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    Benchmark {
        #[serde(default, alias = "text")]
        label: String,
        #[serde(default)]
        value: String,
        #[serde(default, alias = "subtitle")]
        comparison: String,
    },
    SocialIcons {
        #[serde(default)]
        items: Vec<SocialIconItem>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        skin: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        color_type: Option<String>,
    },
    ShareButtons {
        #[serde(default)]
        platforms: Vec<SharePlatformItem>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        skin: Option<String>,
        #[serde(default = "default_true")]
        official_colors: bool,
    },
    NavMenu {
        #[serde(default)]
        menu_slug: String,
        #[serde(default = "default_nav_layout")]
        layout: String,
        #[serde(default = "default_nav_align")]
        align: String,
        #[serde(default = "default_true")]
        mobile_hamburger: bool,
        #[serde(default)]
        items: Vec<NavMenuItem>,
    },
    MegaMenuContainer {
        #[serde(default)]
        trigger: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        width: Option<String>,
    },
    PopupModal {
        #[serde(default)]
        modal_id: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        trigger_selector: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        overlay_bg: Option<String>,
        #[serde(default = "default_true")]
        close_button: bool,
    },
    #[serde(alias = "PostGrid", alias = "PortfolioGrid")]
    LoopGrid {
        #[serde(default = "default_post_type")]
        post_type: String,
        #[serde(default = "default_posts_per_page")]
        posts_per_page: u32,
        #[serde(default = "default_grid_columns")]
        columns: u8,
        #[serde(default = "default_true")]
        show_thumbnail: bool,
        #[serde(default = "default_true")]
        show_excerpt: bool,
        #[serde(default = "default_true")]
        show_meta: bool,
        #[serde(default = "default_true")]
        show_pagination: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        taxonomy: Option<String>,
    },
    TableOfContents {
        #[serde(default = "default_toc_title")]
        title: String,
        #[serde(default = "default_toc_min_depth")]
        min_depth: u8,
        #[serde(default = "default_toc_max_depth")]
        max_depth: u8,
        #[serde(default = "default_true")]
        collapsible: bool,
    },
    Breadcrumbs {
        #[serde(default = "default_breadcrumb_separator")]
        separator: String,
        #[serde(default = "default_breadcrumb_home")]
        home_text: String,
    },
    SearchForm {
        #[serde(default = "default_search_placeholder")]
        placeholder: String,
        #[serde(default = "default_search_button_text")]
        button_text: String,
        #[serde(default = "default_search_endpoint")]
        endpoint: String,
    },
    LoginForm {
        #[serde(default = "default_login_user_label")]
        label_username: String,
        #[serde(default = "default_login_pass_label")]
        label_password: String,
        #[serde(default = "default_login_redirect")]
        redirect_url: String,
        #[serde(default = "default_true")]
        remember_me: bool,
    },
    Accordion {
        #[serde(default)]
        items: Vec<AccordionItem>,
    },
    Tabs {
        #[serde(default)]
        items: Vec<TabItem>,
    },
    Toggle {
        #[serde(default)]
        items: Vec<ToggleItem>,
    },
    Video {
        #[serde(default)]
        url: String,
        #[serde(
            default = "default_aspect",
            alias = "aspect",
            deserialize_with = "deserialize_aspect_ratio"
        )]
        aspect_ratio: String,
        #[serde(default)]
        autoplay: bool,
        #[serde(default)]
        muted: bool,
        #[serde(default, alias = "loop")]
        loop_video: bool,
    },
    Map {
        #[serde(default)]
        address: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zoom: Option<u8>,
    },
    IconBox {
        #[serde(default)]
        icon: String,
        #[serde(default)]
        title: String,
        #[serde(default)]
        description: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        link: Option<String>,
    },
    ImageBox {
        #[serde(default, alias = "url")]
        image_url: String,
        #[serde(default)]
        title: String,
        #[serde(default)]
        description: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        link: Option<String>,
    },
    IconList {
        #[serde(default)]
        items: Vec<IconListItem>,
    },
    Audio {
        #[serde(default)]
        url: String,
    },
    MenuAnchor {
        #[serde(default)]
        anchor_id: String,
    },
    CustomHtml {
        #[serde(default)]
        raw: String,
    },
    ShortcodeFallback {
        #[serde(default)]
        tag: String,
        #[serde(default)]
        attributes: HashMap<String, String>,
        #[serde(default)]
        content: String,
    },
}

impl Serialize for NodeKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let repr: NodeKindRepr = self.clone().into();
        repr.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for NodeKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = Value::deserialize(deserializer)?;
        let obj = val
            .as_object()
            .ok_or_else(|| de::Error::custom("NodeKind must be a JSON object"))?;
        let normalized = normalize_node_kind_object(obj, &StyleProperties::default());
        let repr: NodeKindRepr = serde_json::from_value(normalized).map_err(de::Error::custom)?;
        Ok(repr.into())
    }
}
