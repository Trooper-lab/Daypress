use crate::schema::items::{
    AccordionItem, IconListItem, NavMenuItem, PriceFeatureItem, SharePlatformItem, SocialIconItem,
    TabItem, TestimonialSlide, ToggleItem,
};
use std::collections::HashMap;

/// Strongly-typed widget and container variants conforming to Neopress core,
/// Elementor v2/v3/Pro, Divi 4/5, and the NeoStudio Visual Builder.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    // Structural & Layout
    Root,
    Section,
    Container,
    Column,
    Grid {
        columns: u8,
        gap: u32,
    },
    Flex {
        direction: String,
        justify: String,
        align: String,
        wrap: bool,
        basis: Option<String>,
    },

    // Basic Typography & Content
    Heading {
        text: String,
        level: u8,
    },
    Text {
        content_html: String,
    },
    Image {
        src: String,
        alt: String,
        width: Option<u32>,
        height: Option<u32>,
        link: Option<String>,
    },
    Button {
        text: String,
        url: String,
        target: String,
    },
    Spacer {
        height: u32,
    },
    Divider {
        style: String,
    },
    Icon {
        name: String,
        size: Option<u32>,
        color: Option<String>,
    },

    // Forms
    Form {
        form_id: i64,
        title: Option<String>,
        button_text: Option<String>,
        endpoint: Option<String>,
    },
    NativeForm {
        title: String,
        button_text: String,
        endpoint: String,
    },

    // Commercial & Marketing Pro Blocks (R2)
    PriceCard {
        title: String,
        subtitle: Option<String>,
        currency: Option<String>,
        price: String,
        original_price: Option<String>,
        period: Option<String>,
        badge: Option<String>,
        featured: bool,
        features: Vec<PriceFeatureItem>,
        button_text: String,
        button_url: String,
    },
    PriceTable {
        title: String,
        price: String,
        period: String,
        features: Vec<String>,
        button_text: String,
        button_url: String,
    },
    CallToAction {
        title: String,
        description: String,
        button_text: String,
        button_url: String,
        skin: Option<String>,
        ribbon: Option<String>,
        image_url: Option<String>,
        secondary_button_text: Option<String>,
        secondary_button_url: Option<String>,
    },
    Testimonial {
        content: String,
        name: String,
        job: String,
        image_url: Option<String>,
    },
    TestimonialCarousel {
        slides: Vec<TestimonialSlide>,
        skin: Option<String>,
        autoplay: bool,
        loop_slides: bool,
    },
    Countdown {
        target_date: String,
        mode: String, // "due_date" or "evergreen"
        show_days: bool,
        show_hours: bool,
        show_minutes: bool,
        show_seconds: bool,
        expiry_message: Option<String>,
    },
    FlipBox {
        front_icon: Option<String>,
        front_image: Option<String>,
        front_title: String,
        front_description: String,
        back_title: String,
        back_description: String,
        button_text: Option<String>,
        button_url: Option<String>,
        direction: String, // "flip_left", "flip_right", "flip_up", "flip_down", "push", "slide"
    },
    AnimatedHeadline {
        before_text: String,
        words: Vec<String>,
        after_text: String,
        animation_style: String, // "typing", "clip", "rotate", "highlight"
    },
    Rating {
        rating: f32,
        max_rating: u8,
        title: Option<String>,
    },
    StarRating {
        rating: f32,
        max_rating: u8,
        title: Option<String>,
        star_color: Option<String>,
        star_size: Option<u32>,
    },
    ProgressBar {
        title: String,
        percent: u8,
        display_text: Option<String>,
        track_color: Option<String>,
        bar_color: Option<String>,
        height: Option<u32>,
    },
    Alert {
        alert_type: String,
        title: String,
        description: String,
        dismissible: bool,
        icon: Option<String>,
    },
    Counter {
        number: i64,
        prefix: Option<String>,
        suffix: Option<String>,
        title: Option<String>,
    },
    Benchmark {
        label: String,
        value: String,
        comparison: String,
    },
    SocialIcons {
        items: Vec<SocialIconItem>,
        skin: Option<String>,
        color_type: Option<String>,
    },
    ShareButtons {
        platforms: Vec<SharePlatformItem>,
        skin: Option<String>,
        official_colors: bool,
    },

    // Dynamic & Theme Pro Blocks (R2 / R3)
    NavMenu {
        menu_slug: String,
        layout: String, // "horizontal", "vertical", "dropdown"
        align: String,  // "left", "center", "right", "justify"
        mobile_hamburger: bool,
        items: Vec<NavMenuItem>,
    },
    MegaMenuContainer {
        trigger: String,
        width: Option<String>,
    },
    PopupModal {
        modal_id: String,
        trigger_selector: Option<String>,
        overlay_bg: Option<String>,
        close_button: bool,
    },
    LoopGrid {
        post_type: String,
        posts_per_page: u32,
        columns: u8,
        show_thumbnail: bool,
        show_excerpt: bool,
        show_meta: bool,
        show_pagination: bool,
        taxonomy: Option<String>,
    },
    TableOfContents {
        title: String,
        min_depth: u8,
        max_depth: u8,
        collapsible: bool,
    },
    Breadcrumbs {
        separator: String,
        home_text: String,
    },
    SearchForm {
        placeholder: String,
        button_text: String,
        endpoint: String,
    },
    LoginForm {
        label_username: String,
        label_password: String,
        redirect_url: String,
        remember_me: bool,
    },

    // Media & Interactive Widgets
    Accordion {
        items: Vec<AccordionItem>,
    },
    Tabs {
        items: Vec<TabItem>,
    },
    Toggle {
        items: Vec<ToggleItem>,
    },
    Video {
        url: String,
        aspect_ratio: String,
        autoplay: bool,
        muted: bool,
        loop_video: bool,
    },
    Map {
        address: String,
        zoom: Option<u8>,
    },
    IconBox {
        icon: String,
        title: String,
        description: String,
        link: Option<String>,
    },
    ImageBox {
        image_url: String,
        title: String,
        description: String,
        link: Option<String>,
    },
    IconList {
        items: Vec<IconListItem>,
    },
    Audio {
        url: String,
    },
    MenuAnchor {
        anchor_id: String,
    },

    // Fallbacks & Raw HTML
    CustomHtml {
        raw: String,
    },
    ShortcodeFallback {
        tag: String,
        attributes: HashMap<String, String>,
        content: String,
    },
}
