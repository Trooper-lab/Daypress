use serde::de::{self, Deserializer};
use serde::ser::{SerializeMap, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// The canonical JSON layout container representation for Neopress.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NeopressLayout {
    #[serde(default = "default_version")]
    pub version: u32,
    #[serde(default = "default_root_node")]
    pub root: NeopressNode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub global_styles: Option<serde_json::Value>,
}

fn default_version() -> u32 {
    1
}

fn default_root_node() -> NeopressNode {
    NeopressNode {
        id: "sec_root".to_string(),
        kind: NodeKind::Section,
        styles: StyleProperties::default(),
        children: Vec::new(),
    }
}

impl Default for NeopressLayout {
    fn default() -> Self {
        Self {
            version: 1,
            root: default_root_node(),
            global_styles: None,
        }
    }
}

/// A node within the Neopress Abstract Syntax Tree.
#[derive(Debug, Clone, PartialEq)]
pub struct NeopressNode {
    pub id: String,
    pub kind: NodeKind,
    pub styles: StyleProperties,
    pub children: Vec<NeopressNode>,
}

impl Serialize for NeopressNode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let kind_val = serde_json::to_value(&self.kind).map_err(serde::ser::Error::custom)?;
        let kind_obj = kind_val.as_object();

        let has_data = kind_obj.and_then(|o| o.get("data")).is_some();
        let has_children = !self.children.is_empty();

        let mut field_count = 3; // id, type, styles
        if has_data {
            field_count += 1;
        }
        if has_children {
            field_count += 1;
        }

        let mut map = serializer.serialize_map(Some(field_count))?;
        map.serialize_entry("id", &self.id)?;
        if let Some(obj) = kind_obj {
            if let Some(t) = obj.get("type") {
                map.serialize_entry("type", t)?;
            }
            if let Some(d) = obj.get("data") {
                map.serialize_entry("data", d)?;
            }
        }
        map.serialize_entry("styles", &self.styles)?;
        if has_children {
            map.serialize_entry("children", &self.children)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for NeopressNode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = Value::deserialize(deserializer)?;
        let obj = val
            .as_object()
            .ok_or_else(|| de::Error::custom("NeopressNode must be a JSON object"))?;

        let id = obj
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("node")
            .to_string();

        let mut styles: StyleProperties = match obj.get("styles") {
            Some(v) if !v.is_null() => serde_json::from_value(v.clone()).unwrap_or_default(),
            _ => StyleProperties::default(),
        };
        styles.sync_flat_and_nested();

        let children: Vec<NeopressNode> = match obj.get("children") {
            Some(v) if !v.is_null() => serde_json::from_value(v.clone()).unwrap_or_default(),
            _ => Vec::new(),
        };

        let normalized_kind_val = normalize_node_kind_object(obj, &styles);
        let kind: NodeKind =
            serde_json::from_value(normalized_kind_val).map_err(de::Error::custom)?;

        Ok(NeopressNode {
            id,
            kind,
            styles,
            children,
        })
    }
}

fn normalize_node_kind_object(
    obj: &Map<String, Value>,
    styles: &StyleProperties,
) -> Value {
    let node_type = obj
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or("Section");

    let mut out = Map::new();
    out.insert("type".to_string(), Value::String(node_type.to_string()));

    if matches!(node_type, "Root" | "Section" | "Container" | "Column") {
        return Value::Object(out);
    }

    let mut data_map = match obj.get("data") {
        Some(Value::Object(m)) => m.clone(),
        _ => Map::new(),
    };

    if node_type == "Spacer" && !data_map.contains_key("height") {
        if let Some(ref h_str) = styles.height {
            let digits: String = h_str.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(h) = digits.parse::<u64>() {
                data_map.insert("height".to_string(), Value::Number(h.into()));
            }
        }
    }

    if node_type == "Counter" {
        if let Some(Value::String(s)) = data_map.get("number") {
            if s.trim().ends_with('%') && !data_map.contains_key("suffix") {
                data_map.insert("suffix".to_string(), Value::String("%".to_string()));
            }
        }
    }

    out.insert("data".to_string(), Value::Object(data_map));
    Value::Object(out)
}

/// Strongly-typed widget and container variants conforming to Neopress core and Visual Builder.
#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
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
    Benchmark {
        label: String,
        value: String,
        comparison: String,
    },
    CallToAction {
        title: String,
        description: String,
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
    Counter {
        number: i64,
        prefix: Option<String>,
        suffix: Option<String>,
        title: Option<String>,
    },
    Accordion {
        items: Vec<AccordionItem>,
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
    ProgressBar {
        title: String,
        percent: u8,
        display_text: Option<String>,
    },
    Testimonial {
        content: String,
        name: String,
        job: String,
        image_url: Option<String>,
    },
    Tabs {
        items: Vec<TabItem>,
    },
    Toggle {
        items: Vec<ToggleItem>,
    },
    Alert {
        alert_type: String,
        title: String,
        description: String,
        dismissible: bool,
    },
    SocialIcons {
        items: Vec<SocialIconItem>,
    },
    Rating {
        rating: f32,
        max_rating: u8,
        title: Option<String>,
    },
    Audio {
        url: String,
    },
    MenuAnchor {
        anchor_id: String,
    },
    CustomHtml {
        raw: String,
    },
    ShortcodeFallback {
        tag: String,
        attributes: HashMap<String, String>,
        content: String,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
enum NodeKindRepr {
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
    Benchmark {
        #[serde(default, alias = "text")]
        label: String,
        #[serde(default)]
        value: String,
        #[serde(default, alias = "subtitle")]
        comparison: String,
    },
    CallToAction {
        #[serde(default)]
        title: String,
        #[serde(default)]
        description: String,
        #[serde(default)]
        button_text: String,
        #[serde(default)]
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
        #[serde(default)]
        button_text: String,
        #[serde(default)]
        button_url: String,
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
    Accordion {
        #[serde(default)]
        items: Vec<AccordionItem>,
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
    ProgressBar {
        #[serde(default)]
        title: String,
        #[serde(default)]
        percent: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        display_text: Option<String>,
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
    Tabs {
        #[serde(default)]
        items: Vec<TabItem>,
    },
    Toggle {
        #[serde(default)]
        items: Vec<ToggleItem>,
    },
    Alert {
        #[serde(default = "default_alert_type")]
        alert_type: String,
        #[serde(default)]
        title: String,
        #[serde(default)]
        description: String,
        #[serde(default = "default_true")]
        dismissible: bool,
    },
    SocialIcons {
        #[serde(default)]
        items: Vec<SocialIconItem>,
    },
    Rating {
        #[serde(default = "default_rating")]
        rating: f32,
        #[serde(default = "default_max_rating")]
        max_rating: u8,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        title: Option<String>,
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

impl From<NodeKind> for NodeKindRepr {
    fn from(k: NodeKind) -> Self {
        match k {
            NodeKind::Root => NodeKindRepr::Root,
            NodeKind::Section => NodeKindRepr::Section,
            NodeKind::Container => NodeKindRepr::Container,
            NodeKind::Column => NodeKindRepr::Column,
            NodeKind::Grid { columns, gap } => NodeKindRepr::Grid { columns, gap },
            NodeKind::Flex {
                direction,
                justify,
                align,
                wrap,
                basis,
            } => NodeKindRepr::Flex {
                direction,
                justify,
                align,
                wrap,
                basis,
            },
            NodeKind::Heading { text, level } => NodeKindRepr::Heading { text, level },
            NodeKind::Text { content_html } => NodeKindRepr::Text { content_html },
            NodeKind::Image {
                src,
                alt,
                width,
                height,
                link,
            } => NodeKindRepr::Image {
                src,
                alt,
                width,
                height,
                link,
            },
            NodeKind::Button { text, url, target } => NodeKindRepr::Button { text, url, target },
            NodeKind::Form {
                form_id,
                title,
                button_text,
                endpoint,
            } => NodeKindRepr::Form {
                form_id,
                title,
                button_text,
                endpoint,
            },
            NodeKind::NativeForm {
                title,
                button_text,
                endpoint,
            } => NodeKindRepr::NativeForm {
                title,
                button_text,
                endpoint,
            },
            NodeKind::Benchmark {
                label,
                value,
                comparison,
            } => NodeKindRepr::Benchmark {
                label,
                value,
                comparison,
            },
            NodeKind::CallToAction {
                title,
                description,
                button_text,
                button_url,
            } => NodeKindRepr::CallToAction {
                title,
                description,
                button_text,
                button_url,
            },
            NodeKind::PriceTable {
                title,
                price,
                period,
                features,
                button_text,
                button_url,
            } => NodeKindRepr::PriceTable {
                title,
                price,
                period,
                features,
                button_text,
                button_url,
            },
            NodeKind::Spacer { height } => NodeKindRepr::Spacer { height },
            NodeKind::Divider { style } => NodeKindRepr::Divider { style },
            NodeKind::Icon { name, size, color } => NodeKindRepr::Icon { name, size, color },
            NodeKind::Counter {
                number,
                prefix,
                suffix,
                title,
            } => NodeKindRepr::Counter {
                number,
                prefix,
                suffix,
                title,
            },
            NodeKind::Accordion { items } => NodeKindRepr::Accordion { items },
            NodeKind::Video {
                url,
                aspect_ratio,
                autoplay,
                muted,
                loop_video,
            } => NodeKindRepr::Video {
                url,
                aspect_ratio,
                autoplay,
                muted,
                loop_video,
            },
            NodeKind::Map { address, zoom } => NodeKindRepr::Map { address, zoom },
            NodeKind::IconBox {
                icon,
                title,
                description,
                link,
            } => NodeKindRepr::IconBox {
                icon,
                title,
                description,
                link,
            },
            NodeKind::ImageBox {
                image_url,
                title,
                description,
                link,
            } => NodeKindRepr::ImageBox {
                image_url,
                title,
                description,
                link,
            },
            NodeKind::IconList { items } => NodeKindRepr::IconList { items },
            NodeKind::ProgressBar {
                title,
                percent,
                display_text,
            } => NodeKindRepr::ProgressBar {
                title,
                percent,
                display_text,
            },
            NodeKind::Testimonial {
                content,
                name,
                job,
                image_url,
            } => NodeKindRepr::Testimonial {
                content,
                name,
                job,
                image_url,
            },
            NodeKind::Tabs { items } => NodeKindRepr::Tabs { items },
            NodeKind::Toggle { items } => NodeKindRepr::Toggle { items },
            NodeKind::Alert {
                alert_type,
                title,
                description,
                dismissible,
            } => NodeKindRepr::Alert {
                alert_type,
                title,
                description,
                dismissible,
            },
            NodeKind::SocialIcons { items } => NodeKindRepr::SocialIcons { items },
            NodeKind::Rating {
                rating,
                max_rating,
                title,
            } => NodeKindRepr::Rating {
                rating,
                max_rating,
                title,
            },
            NodeKind::Audio { url } => NodeKindRepr::Audio { url },
            NodeKind::MenuAnchor { anchor_id } => NodeKindRepr::MenuAnchor { anchor_id },
            NodeKind::CustomHtml { raw } => NodeKindRepr::CustomHtml { raw },
            NodeKind::ShortcodeFallback {
                tag,
                attributes,
                content,
            } => NodeKindRepr::ShortcodeFallback {
                tag,
                attributes,
                content,
            },
        }
    }
}

impl From<NodeKindRepr> for NodeKind {
    fn from(r: NodeKindRepr) -> Self {
        match r {
            NodeKindRepr::Root => NodeKind::Root,
            NodeKindRepr::Section => NodeKind::Section,
            NodeKindRepr::Container => NodeKind::Container,
            NodeKindRepr::Column => NodeKind::Column,
            NodeKindRepr::Grid { columns, gap } => NodeKind::Grid { columns, gap },
            NodeKindRepr::Flex {
                direction,
                justify,
                align,
                wrap,
                basis,
            } => NodeKind::Flex {
                direction,
                justify,
                align,
                wrap,
                basis,
            },
            NodeKindRepr::Heading { text, level } => NodeKind::Heading { text, level },
            NodeKindRepr::Text { content_html } => NodeKind::Text { content_html },
            NodeKindRepr::Image {
                src,
                alt,
                width,
                height,
                link,
            } => NodeKind::Image {
                src,
                alt,
                width,
                height,
                link,
            },
            NodeKindRepr::Button { text, url, target } => NodeKind::Button { text, url, target },
            NodeKindRepr::Form {
                form_id,
                title,
                button_text,
                endpoint,
            } => NodeKind::Form {
                form_id,
                title,
                button_text,
                endpoint,
            },
            NodeKindRepr::NativeForm {
                title,
                button_text,
                endpoint,
            } => NodeKind::NativeForm {
                title,
                button_text,
                endpoint,
            },
            NodeKindRepr::Benchmark {
                label,
                value,
                comparison,
            } => NodeKind::Benchmark {
                label,
                value,
                comparison,
            },
            NodeKindRepr::CallToAction {
                title,
                description,
                button_text,
                button_url,
            } => NodeKind::CallToAction {
                title,
                description,
                button_text,
                button_url,
            },
            NodeKindRepr::PriceTable {
                title,
                price,
                period,
                features,
                button_text,
                button_url,
            } => NodeKind::PriceTable {
                title,
                price,
                period,
                features,
                button_text,
                button_url,
            },
            NodeKindRepr::Spacer { height } => NodeKind::Spacer { height },
            NodeKindRepr::Divider { style } => NodeKind::Divider { style },
            NodeKindRepr::Icon { name, size, color } => NodeKind::Icon { name, size, color },
            NodeKindRepr::Counter {
                number,
                prefix,
                suffix,
                title,
            } => NodeKind::Counter {
                number,
                prefix,
                suffix,
                title,
            },
            NodeKindRepr::Accordion { items } => NodeKind::Accordion { items },
            NodeKindRepr::Video {
                url,
                aspect_ratio,
                autoplay,
                muted,
                loop_video,
            } => NodeKind::Video {
                url,
                aspect_ratio,
                autoplay,
                muted,
                loop_video,
            },
            NodeKindRepr::Map { address, zoom } => NodeKind::Map { address, zoom },
            NodeKindRepr::IconBox {
                icon,
                title,
                description,
                link,
            } => NodeKind::IconBox {
                icon,
                title,
                description,
                link,
            },
            NodeKindRepr::ImageBox {
                image_url,
                title,
                description,
                link,
            } => NodeKind::ImageBox {
                image_url,
                title,
                description,
                link,
            },
            NodeKindRepr::IconList { items } => NodeKind::IconList { items },
            NodeKindRepr::ProgressBar {
                title,
                percent,
                display_text,
            } => NodeKind::ProgressBar {
                title,
                percent,
                display_text,
            },
            NodeKindRepr::Testimonial {
                content,
                name,
                job,
                image_url,
            } => NodeKind::Testimonial {
                content,
                name,
                job,
                image_url,
            },
            NodeKindRepr::Tabs { items } => NodeKind::Tabs { items },
            NodeKindRepr::Toggle { items } => NodeKind::Toggle { items },
            NodeKindRepr::Alert {
                alert_type,
                title,
                description,
                dismissible,
            } => NodeKind::Alert {
                alert_type,
                title,
                description,
                dismissible,
            },
            NodeKindRepr::SocialIcons { items } => NodeKind::SocialIcons { items },
            NodeKindRepr::Rating {
                rating,
                max_rating,
                title,
            } => NodeKind::Rating {
                rating,
                max_rating,
                title,
            },
            NodeKindRepr::Audio { url } => NodeKind::Audio { url },
            NodeKindRepr::MenuAnchor { anchor_id } => NodeKind::MenuAnchor { anchor_id },
            NodeKindRepr::CustomHtml { raw } => NodeKind::CustomHtml { raw },
            NodeKindRepr::ShortcodeFallback {
                tag,
                attributes,
                content,
            } => NodeKind::ShortcodeFallback {
                tag,
                attributes,
                content,
            },
        }
    }
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
        let repr: NodeKindRepr =
            serde_json::from_value(normalized).map_err(de::Error::custom)?;
        Ok(repr.into())
    }
}

fn default_grid_columns() -> u8 {
    3
}
fn default_direction() -> String {
    "row".to_string()
}
fn default_justify() -> String {
    "flex-start".to_string()
}
fn default_align() -> String {
    "stretch".to_string()
}
fn default_heading_level() -> u8 {
    2
}
fn default_button_url() -> String {
    "#".to_string()
}
fn default_target() -> String {
    "_self".to_string()
}
fn default_form_id() -> i64 {
    1
}
fn default_spacer_height() -> u32 {
    40
}
fn default_divider_style() -> String {
    "solid".to_string()
}
fn default_counter_number() -> i64 {
    100
}
fn default_aspect() -> String {
    "16/9".to_string()
}
fn default_alert_type() -> String {
    "info".to_string()
}
fn default_true() -> bool {
    true
}
fn default_rating() -> f32 {
    5.0
}
fn default_max_rating() -> u8 {
    5
}

fn deserialize_counter_number<'de, D>(deserializer: D) -> Result<i64, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Value::deserialize(deserializer)?;
    match val {
        Value::Number(n) => Ok(n
            .as_i64()
            .or_else(|| n.as_f64().map(|f| f.round() as i64))
            .unwrap_or(100)),
        Value::String(s) => {
            let trimmed = s.trim();
            let mut numeric = String::new();
            for (idx, ch) in trimmed.chars().enumerate() {
                if (idx == 0 && ch == '-') || ch.is_ascii_digit() {
                    numeric.push(ch);
                } else if ch == ',' {
                    continue;
                } else {
                    break;
                }
            }
            Ok(numeric.parse::<i64>().unwrap_or(100))
        }
        _ => Ok(100),
    }
}

fn deserialize_aspect_ratio<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let val = Option::<String>::deserialize(deserializer)?;
    match val {
        Some(s) if !s.trim().is_empty() => Ok(s.replace(':', "/")),
        _ => Ok(default_aspect()),
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct AccordionItem {
    #[serde(default)]
    pub title: String,
    #[serde(default, alias = "content")]
    pub content_html: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct IconListItem {
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub link: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TabItem {
    #[serde(default)]
    pub title: String,
    #[serde(default, alias = "content")]
    pub content_html: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ToggleItem {
    #[serde(default)]
    pub title: String,
    #[serde(default, alias = "content")]
    pub content_html: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct SocialIconItem {
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub link: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct StyleProperties {
    // Nested canonical AST style properties
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

    // Flat Visual Builder style properties
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<String>,
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
    pub border_top: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub border_bottom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flex_direction: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub justify_content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub align_items: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gap: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_height: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flex: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub box_shadow: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub background_gradient: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct Spacing {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub top: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub right: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bottom: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub left: Option<String>,
}

impl<'de> Deserialize<'de> for Spacing {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = Value::deserialize(deserializer)?;
        match val {
            Value::Object(map) => {
                let get_opt = |k: &str| -> Option<String> {
                    map.get(k)
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                };
                Ok(Spacing {
                    top: get_opt("top"),
                    right: get_opt("right"),
                    bottom: get_opt("bottom"),
                    left: get_opt("left"),
                })
            }
            Value::String(s) => {
                let parts: Vec<&str> = s.split_whitespace().collect();
                match parts.as_slice() {
                    [a] => Ok(Spacing {
                        top: Some((*a).to_string()),
                        right: Some((*a).to_string()),
                        bottom: Some((*a).to_string()),
                        left: Some((*a).to_string()),
                    }),
                    [tb, rl] => Ok(Spacing {
                        top: Some((*tb).to_string()),
                        right: Some((*rl).to_string()),
                        bottom: Some((*tb).to_string()),
                        left: Some((*rl).to_string()),
                    }),
                    [t, rl, b] => Ok(Spacing {
                        top: Some((*t).to_string()),
                        right: Some((*rl).to_string()),
                        bottom: Some((*b).to_string()),
                        left: Some((*rl).to_string()),
                    }),
                    [t, r, b, l, ..] => Ok(Spacing {
                        top: Some((*t).to_string()),
                        right: Some((*r).to_string()),
                        bottom: Some((*b).to_string()),
                        left: Some((*l).to_string()),
                    }),
                    _ => Ok(Spacing::default()),
                }
            }
            _ => Ok(Spacing::default()),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct TypographyConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_size: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub font_weight: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_align: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line_height: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct BackgroundConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub position: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, PartialEq)]
pub struct BorderConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub radius: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub width: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
}

impl<'de> Deserialize<'de> for BorderConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let val = Value::deserialize(deserializer)?;
        match val {
            Value::Object(map) => {
                let get_opt = |k: &str| -> Option<String> {
                    map.get(k)
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string())
                };
                Ok(BorderConfig {
                    radius: get_opt("radius"),
                    width: get_opt("width"),
                    color: get_opt("color"),
                    style: get_opt("style"),
                })
            }
            Value::String(s) => {
                // e.g. "1px solid #e2e8f0"
                let parts: Vec<&str> = s.split_whitespace().collect();
                match parts.as_slice() {
                    [w, st, c, ..] => Ok(BorderConfig {
                        radius: None,
                        width: Some((*w).to_string()),
                        style: Some((*st).to_string()),
                        color: Some((*c).to_string()),
                    }),
                    [st] => Ok(BorderConfig {
                        radius: None,
                        width: None,
                        style: Some((*st).to_string()),
                        color: None,
                    }),
                    _ => Ok(BorderConfig::default()),
                }
            }
            _ => Ok(BorderConfig::default()),
        }
    }
}

impl StyleProperties {
    /// Synchronizes flat Visual Builder style fields and nested canonical AST style structs
    /// so both representations stay in complete lockstep.
    pub fn sync_flat_and_nested(&mut self) {
        // Background color
        if let Some(ref bg) = self.background {
            if self.background_color.is_none() {
                self.background_color = bg.color.clone();
            }
        }
        if let Some(ref bg_col) = self.background_color {
            let bg = self.background.get_or_insert_with(Default::default);
            if bg.color.is_none() {
                bg.color = Some(bg_col.clone());
            }
        }

        // Typography
        if let Some(ref typo) = self.typography {
            if self.text_color.is_none() {
                self.text_color = typo.color.clone();
            }
            if self.font_size.is_none() {
                self.font_size = typo.font_size.clone();
            }
            if self.font_weight.is_none() {
                self.font_weight = typo.font_weight.clone();
            }
            if self.text_align.is_none() {
                self.text_align = typo.text_align.clone();
            }
            if self.line_height.is_none() {
                self.line_height = typo.line_height.clone();
            }
        }
        if self.text_color.is_some()
            || self.font_size.is_some()
            || self.font_weight.is_some()
            || self.text_align.is_some()
            || self.line_height.is_some()
        {
            let typo = self.typography.get_or_insert_with(Default::default);
            if typo.color.is_none() {
                typo.color = self.text_color.clone();
            }
            if typo.font_size.is_none() {
                typo.font_size = self.font_size.clone();
            }
            if typo.font_weight.is_none() {
                typo.font_weight = self.font_weight.clone();
            }
            if typo.text_align.is_none() {
                typo.text_align = self.text_align.clone();
            }
            if typo.line_height.is_none() {
                typo.line_height = self.line_height.clone();
            }
        }

        // Padding
        if let Some(ref pad) = self.padding {
            if self.padding_top.is_none() {
                self.padding_top = pad.top.clone();
            }
            if self.padding_right.is_none() {
                self.padding_right = pad.right.clone();
            }
            if self.padding_bottom.is_none() {
                self.padding_bottom = pad.bottom.clone();
            }
            if self.padding_left.is_none() {
                self.padding_left = pad.left.clone();
            }
        }
        if self.padding_top.is_some()
            || self.padding_right.is_some()
            || self.padding_bottom.is_some()
            || self.padding_left.is_some()
        {
            let pad = self.padding.get_or_insert_with(Default::default);
            if pad.top.is_none() {
                pad.top = self.padding_top.clone();
            }
            if pad.right.is_none() {
                pad.right = self.padding_right.clone();
            }
            if pad.bottom.is_none() {
                pad.bottom = self.padding_bottom.clone();
            }
            if pad.left.is_none() {
                pad.left = self.padding_left.clone();
            }
        }

        // Margin
        if let Some(ref mar) = self.margin {
            if self.margin_top.is_none() {
                self.margin_top = mar.top.clone();
            }
            if self.margin_right.is_none() {
                self.margin_right = mar.right.clone();
            }
            if self.margin_bottom.is_none() {
                self.margin_bottom = mar.bottom.clone();
            }
            if self.margin_left.is_none() {
                self.margin_left = mar.left.clone();
            }
        }
        if self.margin_top.is_some()
            || self.margin_right.is_some()
            || self.margin_bottom.is_some()
            || self.margin_left.is_some()
        {
            let mar = self.margin.get_or_insert_with(Default::default);
            if mar.top.is_none() {
                mar.top = self.margin_top.clone();
            }
            if mar.right.is_none() {
                mar.right = self.margin_right.clone();
            }
            if mar.bottom.is_none() {
                mar.bottom = self.margin_bottom.clone();
            }
            if mar.left.is_none() {
                mar.left = self.margin_left.clone();
            }
        }

        // Border radius
        if let Some(ref bdr) = self.border {
            if self.border_radius.is_none() {
                self.border_radius = bdr.radius.clone();
            }
        }
        if let Some(ref rad) = self.border_radius {
            let bdr = self.border.get_or_insert_with(Default::default);
            if bdr.radius.is_none() {
                bdr.radius = Some(rad.clone());
            }
        }
    }

    /// Compiles style properties into an inline CSS style string for semantic SSR rendering.
    pub fn to_css_style(&self) -> String {
        let mut css = Vec::new();

        // Padding (nested or flat)
        let pad_top = self
            .padding
            .as_ref()
            .and_then(|p| p.top.as_deref())
            .or(self.padding_top.as_deref());
        let pad_right = self
            .padding
            .as_ref()
            .and_then(|p| p.right.as_deref())
            .or(self.padding_right.as_deref());
        let pad_bottom = self
            .padding
            .as_ref()
            .and_then(|p| p.bottom.as_deref())
            .or(self.padding_bottom.as_deref());
        let pad_left = self
            .padding
            .as_ref()
            .and_then(|p| p.left.as_deref())
            .or(self.padding_left.as_deref());

        if let Some(t) = pad_top.filter(|s| !s.is_empty()) {
            css.push(format!("padding-top: {};", t));
        }
        if let Some(r) = pad_right.filter(|s| !s.is_empty()) {
            css.push(format!("padding-right: {};", r));
        }
        if let Some(b) = pad_bottom.filter(|s| !s.is_empty()) {
            css.push(format!("padding-bottom: {};", b));
        }
        if let Some(l) = pad_left.filter(|s| !s.is_empty()) {
            css.push(format!("padding-left: {};", l));
        }

        // Margin (nested or flat)
        let mar_top = self
            .margin
            .as_ref()
            .and_then(|m| m.top.as_deref())
            .or(self.margin_top.as_deref());
        let mar_right = self
            .margin
            .as_ref()
            .and_then(|m| m.right.as_deref())
            .or(self.margin_right.as_deref());
        let mar_bottom = self
            .margin
            .as_ref()
            .and_then(|m| m.bottom.as_deref())
            .or(self.margin_bottom.as_deref());
        let mar_left = self
            .margin
            .as_ref()
            .and_then(|m| m.left.as_deref())
            .or(self.margin_left.as_deref());

        if let Some(t) = mar_top.filter(|s| !s.is_empty()) {
            css.push(format!("margin-top: {};", t));
        }
        if let Some(r) = mar_right.filter(|s| !s.is_empty()) {
            css.push(format!("margin-right: {};", r));
        }
        if let Some(b) = mar_bottom.filter(|s| !s.is_empty()) {
            css.push(format!("margin-bottom: {};", b));
        }
        if let Some(l) = mar_left.filter(|s| !s.is_empty()) {
            css.push(format!("margin-left: {};", l));
        }

        // Typography (nested or flat)
        let font_size = self
            .typography
            .as_ref()
            .and_then(|t| t.font_size.as_deref())
            .or(self.font_size.as_deref());
        let font_weight = self
            .typography
            .as_ref()
            .and_then(|t| t.font_weight.as_deref())
            .or(self.font_weight.as_deref());
        let color = self
            .typography
            .as_ref()
            .and_then(|t| t.color.as_deref())
            .or(self.text_color.as_deref());
        let text_align = self
            .typography
            .as_ref()
            .and_then(|t| t.text_align.as_deref())
            .or(self.text_align.as_deref());
        let line_height = self
            .typography
            .as_ref()
            .and_then(|t| t.line_height.as_deref())
            .or(self.line_height.as_deref());

        if let Some(s) = font_size.filter(|s| !s.is_empty()) {
            css.push(format!("font-size: {};", s));
        }
        if let Some(w) = font_weight.filter(|s| !s.is_empty()) {
            css.push(format!("font-weight: {};", w));
        }
        if let Some(c) = color.filter(|s| !s.is_empty()) {
            css.push(format!("color: {};", c));
        }
        if let Some(a) = text_align.filter(|s| !s.is_empty()) {
            css.push(format!("text-align: {};", a));
        }
        if let Some(lh) = line_height.filter(|s| !s.is_empty()) {
            css.push(format!("line-height: {};", lh));
        }

        // Background (nested or flat)
        let bg_color = self
            .background
            .as_ref()
            .and_then(|b| b.color.as_deref())
            .or(self.background_color.as_deref());

        if let Some(c) = bg_color.filter(|s| !s.is_empty()) {
            css.push(format!("background-color: {};", c));
        }
        if let Some(ref bg) = self.background {
            if let Some(ref img) = bg.image_url {
                if !img.is_empty() {
                    css.push(format!("background-image: url('{}');", img));
                }
            }
            if let Some(ref pos) = bg.position {
                if !pos.is_empty() {
                    css.push(format!("background-position: {};", pos));
                }
            }
            if let Some(ref sz) = bg.size {
                if !sz.is_empty() {
                    css.push(format!("background-size: {};", sz));
                }
            }
        }
        if let Some(ref grad) = self.background_gradient {
            if !grad.is_empty() {
                css.push(format!("background-image: {};", grad));
            }
        }

        // Border (nested or flat)
        let border_radius = self
            .border
            .as_ref()
            .and_then(|b| b.radius.as_deref())
            .or(self.border_radius.as_deref());

        if let Some(rad) = border_radius.filter(|s| !s.is_empty()) {
            css.push(format!("border-radius: {};", rad));
        }
        if let Some(ref bdr) = self.border {
            if let Some(ref w) = bdr.width {
                if !w.is_empty() {
                    css.push(format!("border-width: {};", w));
                }
            }
            if let Some(ref c) = bdr.color {
                if !c.is_empty() {
                    css.push(format!("border-color: {};", c));
                }
            }
            if let Some(ref st) = bdr.style {
                if !st.is_empty() {
                    css.push(format!("border-style: {};", st));
                }
            }
        }
        if let Some(ref bt) = self.border_top {
            if !bt.is_empty() {
                css.push(format!("border-top: {};", bt));
            }
        }
        if let Some(ref bb) = self.border_bottom {
            if !bb.is_empty() {
                css.push(format!("border-bottom: {};", bb));
            }
        }

        // Layout & Flexbox flat properties
        if let Some(ref d) = self.display {
            if !d.is_empty() {
                css.push(format!("display: {};", d));
            }
        }
        if let Some(ref fd) = self.flex_direction {
            if !fd.is_empty() {
                css.push(format!("flex-direction: {};", fd));
            }
        }
        if let Some(ref jc) = self.justify_content {
            if !jc.is_empty() {
                css.push(format!("justify-content: {};", jc));
            }
        }
        if let Some(ref ai) = self.align_items {
            if !ai.is_empty() {
                css.push(format!("align-items: {};", ai));
            }
        }
        if let Some(ref g) = self.gap {
            if !g.is_empty() {
                css.push(format!("gap: {};", g));
            }
        }
        if let Some(ref fl) = self.flex {
            if !fl.is_empty() {
                css.push(format!("flex: {};", fl));
            }
        }
        if let Some(ref w) = self.width {
            if !w.is_empty() {
                css.push(format!("width: {};", w));
            }
        }
        if let Some(ref mw) = self.max_width {
            if !mw.is_empty() {
                css.push(format!("max-width: {};", mw));
            }
        }
        if let Some(ref h) = self.height {
            if !h.is_empty() {
                css.push(format!("height: {};", h));
            }
        }
        if let Some(ref mh) = self.min_height {
            if !mh.is_empty() {
                css.push(format!("min-height: {};", mh));
            }
        }
        if let Some(ref bs) = self.box_shadow {
            if !bs.is_empty() {
                css.push(format!("box-shadow: {};", bs));
            }
        }

        if let Some(ref custom) = self.custom_css {
            if !custom.is_empty() {
                css.push(custom.clone());
            }
        }

        css.join(" ")
    }
}
