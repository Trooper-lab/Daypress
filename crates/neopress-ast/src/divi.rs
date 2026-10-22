use crate::elementor::normalize_video_embed_url;
use crate::schema::{
    AccordionItem, BackgroundConfig, BorderConfig, NeopressLayout, NeopressNode, NodeKind,
    SocialIconItem, Spacing, StyleProperties, TabItem, ToggleItem, TypographyConfig,
};
use regex::Regex;
use std::collections::HashMap;

/// A parsed Divi shortcode element.
#[derive(Debug, Clone)]
pub struct DiviShortcode {
    pub tag: String,
    pub attributes: HashMap<String, String>,
    pub inner_text: String,
    pub children: Vec<DiviShortcode>,
}

/// Parses raw Divi shortcode markup into a canonical NeopressLayout AST.
pub fn transpile_divi_content(content: &str) -> Result<NeopressLayout, String> {
    let tokens = tokenize_divi(content)?;
    let tree = parse_divi_tree(tokens)?;

    let mut id_counter: usize = 0;
    let root_children: Vec<NeopressNode> = tree
        .iter()
        .map(|node| map_divi_node(node, &mut id_counter))
        .collect();

    Ok(NeopressLayout {
        version: 1,
        root: NeopressNode {
            id: "divi_root".to_string(),
            kind: NodeKind::Section,
            styles: StyleProperties::default(),
            children: root_children,
        },
        global_styles: None,
    })
}

#[derive(Debug, PartialEq)]
enum Token {
    OpenTag {
        name: String,
        attributes: HashMap<String, String>,
    },
    CloseTag {
        name: String,
    },
    Text(String),
}

fn is_divi_leaf_tag(tag: &str) -> bool {
    matches!(
        tag,
        "et_pb_divider"
            | "et_pb_button"
            | "et_pb_image"
            | "et_pb_space"
            | "et_pb_video"
            | "et_pb_map"
            | "et_pb_fullwidth_image"
            | "et_pb_contact_field"
    )
}

fn is_divi_container_tag(tag: &str) -> bool {
    matches!(
        tag,
        "et_pb_section"
            | "et_pb_fullwidth_section"
            | "et_pb_specialty_section"
            | "et_pb_row"
            | "et_pb_row_inner"
            | "et_pb_column"
            | "et_pb_column_inner"
            | "et_pb_accordion"
            | "et_pb_tabs"
            | "et_pb_counters"
            | "et_pb_pricing_tables"
            | "et_pb_contact_form"
            | "et_pb_social_media_follow"
    )
}

fn tokenize_divi(input: &str) -> Result<Vec<Token>, String> {
    let tag_re = Regex::new(r"\[(/?)(et_pb_[a-z0-9_]+)([^\]]*)\]").unwrap();
    let mut tokens = Vec::new();
    let mut last_idx = 0;

    for caps in tag_re.captures_iter(input) {
        let full_match = caps.get(0).unwrap();
        let match_start = full_match.start();
        let match_end = full_match.end();

        if match_start > last_idx {
            let text = &input[last_idx..match_start];
            if !text.trim().is_empty() {
                tokens.push(Token::Text(text.to_string()));
            }
        }

        let is_close = caps.get(1).map(|m| m.as_str() == "/").unwrap_or(false);
        let tag_name = caps.get(2).unwrap().as_str().to_string();

        if is_close {
            tokens.push(Token::CloseTag { name: tag_name });
        } else {
            let raw_attrs = caps.get(3).map(|m| m.as_str()).unwrap_or("");
            let is_self_closing = raw_attrs.trim_end().ends_with('/');
            let cleaned_attrs = raw_attrs.trim_end().trim_end_matches('/');
            let attributes = parse_attributes(cleaned_attrs);
            tokens.push(Token::OpenTag {
                name: tag_name.clone(),
                attributes,
            });
            if is_self_closing {
                tokens.push(Token::CloseTag { name: tag_name });
            }
        }

        last_idx = match_end;
    }

    if last_idx < input.len() {
        let text = &input[last_idx..];
        if !text.trim().is_empty() {
            tokens.push(Token::Text(text.to_string()));
        }
    }

    Ok(tokens)
}

fn parse_attributes(raw: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    let attr_re = Regex::new(r#"([a-zA-Z0-9_\-]+)=(?:["']([^"']*)["']|([^\s>]+))"#).unwrap();

    for caps in attr_re.captures_iter(raw) {
        let key = caps.get(1).unwrap().as_str().to_string();
        let val = caps
            .get(2)
            .or_else(|| caps.get(3))
            .map(|m| m.as_str())
            .unwrap_or("")
            .to_string();
        attrs.insert(key, val);
    }

    attrs
}

fn pop_top_to_parent(stack: &mut Vec<DiviShortcode>, root_nodes: &mut Vec<DiviShortcode>) {
    if let Some(closed) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(closed);
        } else {
            root_nodes.push(closed);
        }
    }
}

fn parse_divi_tree(tokens: Vec<Token>) -> Result<Vec<DiviShortcode>, String> {
    let mut root_nodes: Vec<DiviShortcode> = Vec::new();
    let mut stack: Vec<DiviShortcode> = Vec::new();

    for token in tokens {
        match token {
            Token::OpenTag { name, attributes } => {
                // Auto-close any open leaf tag or non-container module before opening a new sibling tag
                while let Some(top) = stack.last() {
                    if is_divi_leaf_tag(&top.tag)
                        || (!is_divi_container_tag(&top.tag) && top.tag == name)
                    {
                        pop_top_to_parent(&mut stack, &mut root_nodes);
                    } else {
                        break;
                    }
                }

                let node = DiviShortcode {
                    tag: name,
                    attributes,
                    inner_text: String::new(),
                    children: Vec::new(),
                };
                stack.push(node);
            }
            Token::CloseTag { name } => {
                // First, auto-close any unclosed leaf tags sitting on top of the stack
                while let Some(top) = stack.last() {
                    if top.tag != name && is_divi_leaf_tag(&top.tag) {
                        pop_top_to_parent(&mut stack, &mut root_nodes);
                    } else {
                        break;
                    }
                }

                if let Some(pos) = stack.iter().rposition(|n| n.tag == name) {
                    // Close any unclosed inner tags between the end of stack and `pos` cleanly
                    while stack.len() > pos + 1 {
                        pop_top_to_parent(&mut stack, &mut root_nodes);
                    }
                    pop_top_to_parent(&mut stack, &mut root_nodes);
                }
            }
            Token::Text(t) => {
                // If top of stack is an unclosed leaf tag that does not take inner text (like divider or space),
                // close it first so text goes to the parent or next element.
                if let Some(top) = stack.last() {
                    if matches!(
                        top.tag.as_str(),
                        "et_pb_divider" | "et_pb_space" | "et_pb_image" | "et_pb_fullwidth_image"
                    ) {
                        pop_top_to_parent(&mut stack, &mut root_nodes);
                    }
                }
                if let Some(top) = stack.last_mut() {
                    top.inner_text.push_str(&t);
                }
            }
        }
    }

    // Drain any remaining unclosed tags into their parents or root
    while !stack.is_empty() {
        pop_top_to_parent(&mut stack, &mut root_nodes);
    }

    Ok(root_nodes)
}

fn map_divi_node(divi: &DiviShortcode, counter: &mut usize) -> NeopressNode {
    *counter += 1;
    let short_name = divi.tag.strip_prefix("et_pb_").unwrap_or(&divi.tag);
    let id = format!("divi_{}_{}", short_name, *counter);
    let mut styles = extract_divi_styles(&divi.attributes);

    let mut children: Vec<NeopressNode> = divi
        .children
        .iter()
        .map(|c| map_divi_node(c, counter))
        .collect();

    let kind = match divi.tag.as_str() {
        "et_pb_section" | "et_pb_fullwidth_section" | "et_pb_specialty_section" => {
            NodeKind::Section
        }
        "et_pb_row" | "et_pb_row_inner" => NodeKind::Container,
        "et_pb_column" | "et_pb_column_inner" => {
            let col_type = divi
                .attributes
                .get("type")
                .map(|s| s.as_str())
                .unwrap_or("4_4");
            let basis = match col_type {
                "4_4" => "100%",
                "1_2" => "50%",
                "1_3" => "33.33%",
                "2_3" => "66.66%",
                "1_4" => "25%",
                "3_4" => "75%",
                "1_5" => "20%",
                "2_5" => "40%",
                "3_5" => "60%",
                "4_5" => "80%",
                "1_6" => "16.66%",
                other => other,
            };
            NodeKind::Flex {
                direction: "column".to_string(),
                justify: "flex-start".to_string(),
                align: "stretch".to_string(),
                wrap: false,
                basis: Some(basis.to_string()),
            }
        }
        "et_pb_text" => {
            let content_html = if !divi.inner_text.trim().is_empty() {
                divi.inner_text.clone()
            } else {
                divi.attributes.get("content").cloned().unwrap_or_default()
            };
            NodeKind::Text { content_html }
        }
        "et_pb_heading" | "et_pb_title" | "et_pb_post_title" => {
            let text = divi
                .attributes
                .get("title")
                .cloned()
                .unwrap_or_else(|| divi.inner_text.clone());
            let level = divi
                .attributes
                .get("title_level")
                .or_else(|| divi.attributes.get("header_level"))
                .and_then(|s| s.strip_prefix('h'))
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(2);
            NodeKind::Heading { text, level }
        }
        "et_pb_image" | "et_pb_fullwidth_image" => {
            let src = divi.attributes.get("src").cloned().unwrap_or_default();
            let alt = divi
                .attributes
                .get("alt")
                .or_else(|| divi.attributes.get("title_text"))
                .cloned()
                .unwrap_or_default();
            let link = divi
                .attributes
                .get("url")
                .cloned()
                .filter(|s| !s.is_empty());
            NodeKind::Image {
                src,
                alt,
                width: None,
                height: None,
                link,
            }
        }
        "et_pb_button" => {
            let text = divi
                .attributes
                .get("button_text")
                .cloned()
                .unwrap_or_else(|| {
                    if !divi.inner_text.trim().is_empty() {
                        divi.inner_text.trim().to_string()
                    } else {
                        "Click Here".to_string()
                    }
                });
            let url = divi
                .attributes
                .get("button_url")
                .cloned()
                .unwrap_or_else(|| "#".to_string());
            let is_external = divi
                .attributes
                .get("url_new_window")
                .map(|s| s == "on")
                .unwrap_or(false);
            let target = if is_external { "_blank" } else { "_self" }.to_string();
            NodeKind::Button { text, url, target }
        }
        "et_pb_blurb" => {
            let title = divi
                .attributes
                .get("title")
                .cloned()
                .unwrap_or_else(|| "Feature".to_string());
            let description = divi.inner_text.trim().to_string();
            let link = divi
                .attributes
                .get("url")
                .cloned()
                .filter(|s| !s.is_empty());
            let use_icon = divi
                .attributes
                .get("use_icon")
                .map(|s| s == "on")
                .unwrap_or(false);

            if !use_icon {
                if let Some(img_url) = divi.attributes.get("image").filter(|s| !s.is_empty()) {
                    return NeopressNode {
                        id,
                        kind: NodeKind::ImageBox {
                            image_url: img_url.clone(),
                            title,
                            description,
                            link,
                        },
                        styles,
                        children: Vec::new(),
                    };
                }
            }

            let icon = divi
                .attributes
                .get("font_icon")
                .map(|ic| map_divi_icon(ic))
                .unwrap_or_else(|| "fa-solid fa-bolt".to_string());

            NodeKind::IconBox {
                icon,
                title,
                description,
                link,
            }
        }
        "et_pb_cta" | "et_pb_fullwidth_header" => {
            let title = divi.attributes.get("title").cloned().unwrap_or_default();
            let subhead = divi.attributes.get("subhead").cloned().unwrap_or_default();
            let description = if !divi.inner_text.trim().is_empty() {
                divi.inner_text.trim().to_string()
            } else {
                subhead
            };
            let button_text = divi
                .attributes
                .get("button_text")
                .or_else(|| divi.attributes.get("button_one_text"))
                .cloned()
                .unwrap_or_else(|| "Get Started".to_string());
            let button_url = divi
                .attributes
                .get("button_url")
                .or_else(|| divi.attributes.get("button_one_url"))
                .cloned()
                .unwrap_or_else(|| "#".to_string());

            NodeKind::CallToAction {
                title,
                description,
                button_text,
                button_url,
            }
        }
        "et_pb_divider" => {
            let style = divi
                .attributes
                .get("divider_style")
                .cloned()
                .unwrap_or_else(|| "solid".to_string());
            NodeKind::Divider { style }
        }
        "et_pb_space" => {
            let height = divi
                .attributes
                .get("height")
                .and_then(|h| {
                    let digits: String = h.chars().take_while(|c| c.is_ascii_digit()).collect();
                    digits.parse::<u32>().ok()
                })
                .unwrap_or(40);
            NodeKind::Spacer { height }
        }
        "et_pb_code" | "et_pb_fullwidth_code" => NodeKind::CustomHtml {
            raw: divi.inner_text.clone(),
        },
        "et_pb_counters" => NodeKind::Container,
        "et_pb_counter" => {
            let percent = divi
                .attributes
                .get("percent")
                .and_then(|p| p.trim_end_matches('%').parse::<u8>().ok())
                .unwrap_or(100);
            let title = if !divi.inner_text.trim().is_empty() {
                divi.inner_text.trim().to_string()
            } else {
                divi.attributes
                    .get("title")
                    .cloned()
                    .unwrap_or_else(|| "Progress".to_string())
            };
            NodeKind::ProgressBar {
                title,
                percent,
                display_text: Some(format!("{}%", percent)),
            }
        }
        "et_pb_number_counter" | "et_pb_circle_counter" => {
            let number = divi
                .attributes
                .get("number")
                .or_else(|| divi.attributes.get("percent"))
                .and_then(|p| p.trim_end_matches('%').parse::<i64>().ok())
                .unwrap_or(100);
            let percent_sign = divi
                .attributes
                .get("percent_sign")
                .map(|s| s == "on")
                .unwrap_or(true);
            let suffix = if percent_sign {
                Some("%".to_string())
            } else {
                None
            };
            let title = divi.attributes.get("title").cloned().or_else(|| {
                if !divi.inner_text.trim().is_empty() {
                    Some(divi.inner_text.trim().to_string())
                } else {
                    None
                }
            });
            NodeKind::Counter {
                number,
                prefix: None,
                suffix,
                title,
            }
        }
        "et_pb_accordion" => {
            let items: Vec<AccordionItem> = divi
                .children
                .iter()
                .filter(|c| {
                    c.tag == "et_pb_accordion_item"
                        || c.tag == "et_pb_toggle"
                        || !c.inner_text.is_empty()
                })
                .map(|c| AccordionItem {
                    title: c
                        .attributes
                        .get("title")
                        .cloned()
                        .unwrap_or_else(|| "Accordion Item".to_string()),
                    content_html: c.inner_text.trim().to_string(),
                })
                .collect();
            children.clear();
            NodeKind::Accordion { items }
        }
        "et_pb_toggle" => {
            let title = divi
                .attributes
                .get("title")
                .cloned()
                .unwrap_or_else(|| "Toggle".to_string());
            let content_html = divi.inner_text.trim().to_string();
            NodeKind::Toggle {
                items: vec![ToggleItem {
                    title,
                    content_html,
                }],
            }
        }
        "et_pb_tabs" => {
            let items: Vec<TabItem> = divi
                .children
                .iter()
                .filter(|c| c.tag == "et_pb_tab" || !c.inner_text.is_empty())
                .map(|c| TabItem {
                    title: c
                        .attributes
                        .get("title")
                        .cloned()
                        .unwrap_or_else(|| "Tab".to_string()),
                    content_html: c.inner_text.trim().to_string(),
                })
                .collect();
            children.clear();
            NodeKind::Tabs { items }
        }
        "et_pb_testimonial" => {
            let content = divi.inner_text.trim().to_string();
            let name = divi
                .attributes
                .get("author")
                .cloned()
                .unwrap_or_else(|| "Client".to_string());
            let job_title = divi.attributes.get("job_title").cloned().unwrap_or_default();
            let company = divi
                .attributes
                .get("company_name")
                .cloned()
                .unwrap_or_default();
            let job = match (job_title.is_empty(), company.is_empty()) {
                (false, false) => format!("{}, {}", job_title, company),
                (false, true) => job_title,
                (true, false) => company,
                (true, true) => String::new(),
            };
            let image_url = divi
                .attributes
                .get("portrait_url")
                .cloned()
                .filter(|s| !s.is_empty());
            NodeKind::Testimonial {
                content,
                name,
                job,
                image_url,
            }
        }
        "et_pb_pricing_tables" => {
            let cols = divi.children.len().clamp(1, 4) as u8;
            NodeKind::Grid {
                columns: cols,
                gap: 24,
            }
        }
        "et_pb_pricing_table" => {
            let title = divi
                .attributes
                .get("title")
                .cloned()
                .unwrap_or_else(|| "Plan".to_string());
            let currency = divi
                .attributes
                .get("currency")
                .cloned()
                .unwrap_or_else(|| "$".to_string());
            let sum = divi
                .attributes
                .get("sum")
                .cloned()
                .unwrap_or_else(|| "29".to_string());
            let price = if sum.starts_with('$') || sum.starts_with('€') || sum.starts_with('£') {
                sum
            } else {
                format!("{}{}", currency, sum)
            };
            let period = divi
                .attributes
                .get("per")
                .map(|p| format!("/ {}", p))
                .unwrap_or_else(|| "/ month".to_string());

            let mut features: Vec<String> = divi
                .inner_text
                .lines()
                .map(|l| l.trim().trim_start_matches('+').trim_start_matches('-').trim())
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect();
            for child in &divi.children {
                if !child.inner_text.trim().is_empty() {
                    features.push(child.inner_text.trim().to_string());
                }
            }

            let button_text = divi
                .attributes
                .get("button_text")
                .cloned()
                .unwrap_or_else(|| "Get Started".to_string());
            let button_url = divi
                .attributes
                .get("button_url")
                .cloned()
                .unwrap_or_else(|| "#".to_string());

            children.clear();
            NodeKind::PriceTable {
                title,
                price,
                period,
                features,
                button_text,
                button_url,
            }
        }
        "et_pb_contact_form" => {
            let title = divi.attributes.get("title").cloned();
            let button_text = divi.attributes.get("submit_button_text").cloned();
            children.clear();
            NodeKind::Form {
                form_id: 1,
                title,
                button_text,
                endpoint: None,
            }
        }
        "et_pb_video" => {
            let raw_url = divi
                .attributes
                .get("src")
                .cloned()
                .unwrap_or_default();
            let url = normalize_video_embed_url(&raw_url);
            NodeKind::Video {
                url,
                aspect_ratio: "16/9".to_string(),
                autoplay: false,
                muted: false,
                loop_video: false,
            }
        }
        "et_pb_map" => {
            let address = divi
                .attributes
                .get("address")
                .cloned()
                .unwrap_or_else(|| "New York, NY".to_string());
            let zoom = divi
                .attributes
                .get("zoom_level")
                .and_then(|z| z.parse::<u8>().ok());
            children.clear();
            NodeKind::Map { address, zoom }
        }
        "et_pb_social_media_follow" => {
            let items: Vec<SocialIconItem> = divi
                .children
                .iter()
                .map(|c| {
                    let net = c
                        .attributes
                        .get("social_network")
                        .map(|s| s.as_str())
                        .unwrap_or("facebook");
                    let icon = match net {
                        "twitter" | "x" => "fa-brands fa-x-twitter",
                        "linkedin" => "fa-brands fa-linkedin",
                        "instagram" => "fa-brands fa-instagram",
                        "youtube" => "fa-brands fa-youtube",
                        "github" => "fa-brands fa-github",
                        _ => "fa-brands fa-facebook",
                    }
                    .to_string();
                    let link = c
                        .attributes
                        .get("url")
                        .cloned()
                        .unwrap_or_else(|| "#".to_string());
                    SocialIconItem { icon, link }
                })
                .collect();
            children.clear();
            NodeKind::SocialIcons { items }
        }
        "et_pb_gallery" => NodeKind::CustomHtml {
            raw: "<div class=\"np-gallery\"></div>".to_string(),
        },
        "et_pb_blog" => NodeKind::CustomHtml {
            raw: "<div class=\"np-posts-grid\"></div>".to_string(),
        },
        // Unrecognized or third-party Divi child modules
        _ => NodeKind::ShortcodeFallback {
            tag: divi.tag.clone(),
            attributes: divi.attributes.clone(),
            content: divi.inner_text.clone(),
        },
    };

    styles.sync_flat_and_nested();

    NeopressNode {
        id,
        kind,
        styles,
        children,
    }
}

fn map_divi_icon(raw_icon: &str) -> String {
    if raw_icon.starts_with("fa-") || raw_icon.contains("fa-solid") || raw_icon.contains("fa-brands")
    {
        return raw_icon.to_string();
    }
    "fa-solid fa-bolt".to_string()
}

fn parse_divi_quad(raw: &str) -> Option<Spacing> {
    // Divi custom_padding / custom_margin is formatted as "top|right|bottom|left|false|false"
    let parts: Vec<&str> = raw.split('|').collect();
    let clean_part = |idx: usize| -> Option<String> {
        parts.get(idx).and_then(|s| {
            let trimmed = s.trim();
            if trimmed.is_empty()
                || matches!(trimmed, "true" | "false" | "on" | "off")
            {
                None
            } else {
                Some(trimmed.to_string())
            }
        })
    };

    let top = clean_part(0);
    let right = clean_part(1);
    let bottom = clean_part(2);
    let left = clean_part(3);

    if top.is_some() || right.is_some() || bottom.is_some() || left.is_some() {
        Some(Spacing {
            top,
            right,
            bottom,
            left,
        })
    } else {
        None
    }
}

fn extract_divi_styles(attrs: &HashMap<String, String>) -> StyleProperties {
    let mut styles = StyleProperties::default();

    let bg_color = attrs
        .get("background_color")
        .filter(|s| !s.is_empty())
        .cloned();
    let bg_img = attrs
        .get("background_image")
        .filter(|s| !s.is_empty())
        .cloned();

    if bg_color.is_some() || bg_img.is_some() {
        styles.background = Some(BackgroundConfig {
            color: bg_color,
            image_url: bg_img,
            position: None,
            size: None,
        });
    }

    if let Some(pad) = attrs.get("custom_padding") {
        styles.padding = parse_divi_quad(pad);
    }

    if let Some(mar) = attrs.get("custom_margin") {
        styles.margin = parse_divi_quad(mar);
    }

    if let Some(radii) = attrs.get("border_radii") {
        // Divi format: "on|8px|8px|8px|8px"
        let parts: Vec<&str> = radii.split('|').collect();
        if parts.len() >= 5 {
            let t = parts[1].trim();
            let r = parts[2].trim();
            let b = parts[3].trim();
            let l = parts[4].trim();
            if !t.is_empty() {
                let rad = if t == r && r == b && b == l {
                    t.to_string()
                } else {
                    format!("{} {} {} {}", t, r, b, l)
                };
                styles.border = Some(BorderConfig {
                    radius: Some(rad),
                    width: None,
                    color: None,
                    style: None,
                });
            }
        }
    }

    let text_align = attrs
        .get("text_orientation")
        .or_else(|| attrs.get("module_alignment"))
        .filter(|s| !s.is_empty())
        .map(|s| {
            if s == "justified" {
                "justify".to_string()
            } else {
                s.clone()
            }
        });

    let mut text_color = attrs
        .get("header_text_color")
        .or_else(|| attrs.get("body_text_color"))
        .or_else(|| attrs.get("text_color"))
        .filter(|s| !s.is_empty())
        .cloned();

    if text_color.is_none()
        && attrs
            .get("background_layout")
            .map(|s| s == "dark")
            .unwrap_or(false)
    {
        text_color = Some("#ffffff".to_string());
    }

    let font_size = attrs
        .get("header_font_size")
        .or_else(|| attrs.get("body_font_size"))
        .filter(|s| !s.is_empty())
        .cloned();

    if text_align.is_some() || text_color.is_some() || font_size.is_some() {
        styles.typography = Some(TypographyConfig {
            font_size,
            font_weight: None,
            color: text_color,
            text_align,
            line_height: None,
        });
    }

    styles.sync_flat_and_nested();
    styles
}
