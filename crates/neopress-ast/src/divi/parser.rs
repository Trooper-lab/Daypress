use crate::divi::context::apply_divi_global_colors_info;
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

#[derive(Debug, PartialEq)]
pub(crate) enum Token {
    OpenTag {
        name: String,
        attributes: HashMap<String, String>,
    },
    CloseTag {
        name: String,
    },
    Text(String),
}

pub(crate) fn is_divi_leaf_tag(tag: &str) -> bool {
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
            | "et_pb_icon"
            | "et_pb_menu"
            | "et_pb_fullwidth_menu"
            | "dvppl_cf7_styler"
            | "spb_kw_calculator_module"
            | "spb_kitwerk_calculator"
            | "et_pb_circle_counter"
            | "et_pb_number_counter"
            | "et_pb_countdown_timer"
            | "et_pb_audio"
            | "et_pb_search"
            | "et_pb_login"
            | "et_pb_code"
    )
}

pub(crate) fn is_divi_container_tag(tag: &str) -> bool {
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
            | "et_pb_bar_counters"
            | "et_pb_pricing_tables"
            | "et_pb_contact_form"
            | "et_pb_social_media_follow"
            | "et_pb_slider"
            | "et_pb_fullwidth_slider"
            | "et_pb_post_slider"
            | "et_pb_slide"
            | "et_pb_signup"
    )
}

pub(crate) fn tokenize_divi(input: &str) -> Result<Vec<Token>, String> {
    let tag_re = Regex::new(r"\[(/?)((?:et_pb_|spb_|dvppl_)[a-z0-9_]+)([^\]]*)\]").unwrap();
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
            let mut attributes = parse_attributes(cleaned_attrs);
            apply_divi_global_colors_info(&mut attributes);
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

pub fn parse_attributes(raw: &str) -> HashMap<String, String> {
    let mut attrs = HashMap::new();
    let attr_re = Regex::new(r#"([a-zA-Z0-9_\-]+)=(?:"([^"]*)"|'([^']*)'|([^\s>]+))"#).unwrap();

    for caps in attr_re.captures_iter(raw) {
        let key = caps.get(1).unwrap().as_str().to_string();
        let val = caps
            .get(2)
            .or_else(|| caps.get(3))
            .or_else(|| caps.get(4))
            .map(|m| m.as_str())
            .unwrap_or("")
            .to_string();
        attrs.insert(key, val);
    }

    attrs
}

pub(crate) fn pop_top_to_parent(
    stack: &mut Vec<DiviShortcode>,
    root_nodes: &mut Vec<DiviShortcode>,
) {
    if let Some(closed) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(closed);
        } else {
            root_nodes.push(closed);
        }
    }
}

pub(crate) fn parse_divi_tree(tokens: Vec<Token>) -> Result<Vec<DiviShortcode>, String> {
    let mut root_nodes: Vec<DiviShortcode> = Vec::new();
    let mut stack: Vec<DiviShortcode> = Vec::new();

    for token in tokens {
        match token {
            Token::OpenTag { name, attributes } => {
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
                while let Some(top) = stack.last() {
                    if top.tag != name && is_divi_leaf_tag(&top.tag) {
                        pop_top_to_parent(&mut stack, &mut root_nodes);
                    } else {
                        break;
                    }
                }

                if let Some(pos) = stack.iter().rposition(|n| n.tag == name) {
                    while stack.len() > pos + 1 {
                        pop_top_to_parent(&mut stack, &mut root_nodes);
                    }
                    pop_top_to_parent(&mut stack, &mut root_nodes);
                }
            }
            Token::Text(t) => {
                if let Some(top) = stack.last() {
                    if matches!(
                        top.tag.as_str(),
                        "et_pb_divider"
                            | "et_pb_space"
                            | "et_pb_image"
                            | "et_pb_fullwidth_image"
                            | "et_pb_icon"
                            | "et_pb_menu"
                            | "dvppl_cf7_styler"
                            | "spb_kw_calculator_module"
                            | "spb_kitwerk_calculator"
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

    while !stack.is_empty() {
        pop_top_to_parent(&mut stack, &mut root_nodes);
    }

    Ok(root_nodes)
}
