use crate::divi::context::DiviContext;
use crate::divi::modules::map_divi_node;
use crate::divi::parser::DiviShortcode;
use crate::schema::{NeopressLayout, NeopressNode, NodeKind, StyleProperties};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

static DIVI5_BLOCK_RE: OnceLock<Regex> = OnceLock::new();

/// Parses Divi 5 Gutenberg block comments (`<!-- wp:divi/* -->`) into a canonical NeopressLayout AST.
pub fn transpile_divi5_content_with_context(
    content: &str,
    ctx: &DiviContext,
) -> Result<NeopressLayout, String> {
    let tree = parse_divi5_blocks(content)?;
    let mut id_counter: usize = 0;
    let root_children: Vec<NeopressNode> = tree
        .iter()
        .map(|node| map_divi_node(node, ctx, &mut id_counter))
        .collect();

    Ok(NeopressLayout {
        version: 1,
        root: NeopressNode {
            id: "divi5_root".to_string(),
            kind: NodeKind::Section,
            styles: StyleProperties::default(),
            children: root_children,
        },
        global_styles: None,
    })
}

fn extract_attrs_from_json(val: &serde_json::Value, out: &mut HashMap<String, String>) {
    if let Some(obj) = val.as_object() {
        if let Some(attrs) = obj
            .get("module")
            .and_then(|m| m.get("props"))
            .and_then(|p| p.get("attrs"))
            .and_then(|a| a.as_object())
        {
            for (k, v) in attrs {
                if let Some(s) = v.as_str() {
                    out.insert(k.clone(), s.to_string());
                } else if let Some(n) = v.as_i64() {
                    out.insert(k.clone(), n.to_string());
                } else if let Some(f) = v.as_f64() {
                    out.insert(k.clone(), f.to_string());
                } else if let Some(b) = v.as_bool() {
                    out.insert(
                        k.clone(),
                        if b {
                            "on".to_string()
                        } else {
                            "off".to_string()
                        },
                    );
                }
            }
        }
        for (k, v) in obj {
            if k == "module" {
                continue;
            }
            if let Some(s) = v.as_str() {
                out.insert(k.clone(), s.to_string());
            } else if let Some(n) = v.as_i64() {
                out.insert(k.clone(), n.to_string());
            } else if let Some(f) = v.as_f64() {
                out.insert(k.clone(), f.to_string());
            } else if let Some(b) = v.as_bool() {
                out.insert(
                    k.clone(),
                    if b {
                        "on".to_string()
                    } else {
                        "off".to_string()
                    },
                );
            }
        }
    }
}

pub(crate) fn parse_divi5_blocks(content: &str) -> Result<Vec<DiviShortcode>, String> {
    let block_re = DIVI5_BLOCK_RE.get_or_init(|| {
        Regex::new(r"(?s)<!--\s*(/?wp:divi/([a-z0-9_\-]+))\s*(\{.*?\})?\s*(/?)-->").unwrap()
    });

    let mut root_nodes: Vec<DiviShortcode> = Vec::new();
    let mut stack: Vec<DiviShortcode> = Vec::new();
    let mut last_idx = 0;

    for caps in block_re.captures_iter(content) {
        let full_match = caps.get(0).unwrap();
        let match_start = full_match.start();
        let match_end = full_match.end();

        // Any inner text before the block
        if match_start > last_idx {
            let text = &content[last_idx..match_start];
            if !text.trim().is_empty() {
                if let Some(top) = stack.last_mut() {
                    top.inner_text.push_str(text);
                }
            }
        }

        let full_tag = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let raw_tag = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let is_closing = full_tag.starts_with("/wp:divi/");
        let is_self_closing = caps.get(4).map(|m| m.as_str() == "/").unwrap_or(false);

        let normalized_tag = if raw_tag.starts_with("et_pb_") || raw_tag.starts_with("spb_") {
            raw_tag.to_string()
        } else {
            format!("et_pb_{}", raw_tag)
        };

        if is_closing {
            // Find matching tag in stack
            if let Some(pos) = stack.iter().rposition(|n| n.tag == normalized_tag) {
                while stack.len() > pos + 1 {
                    if let Some(closed) = stack.pop() {
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(closed);
                        } else {
                            root_nodes.push(closed);
                        }
                    }
                }
                if let Some(closed) = stack.pop() {
                    if let Some(parent) = stack.last_mut() {
                        parent.children.push(closed);
                    } else {
                        root_nodes.push(closed);
                    }
                }
            }
        } else {
            let mut attributes = HashMap::new();
            if let Some(json_str) = caps.get(3).map(|m| m.as_str()) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(json_str) {
                    extract_attrs_from_json(&val, &mut attributes);
                }
            }

            let node = DiviShortcode {
                tag: normalized_tag,
                attributes,
                inner_text: String::new(),
                children: Vec::new(),
            };

            if is_self_closing {
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(node);
                } else {
                    root_nodes.push(node);
                }
            } else {
                stack.push(node);
            }
        }

        last_idx = match_end;
    }

    if last_idx < content.len() {
        let text = &content[last_idx..];
        if !text.trim().is_empty() {
            if let Some(top) = stack.last_mut() {
                top.inner_text.push_str(text);
            }
        }
    }

    while let Some(closed) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(closed);
        } else {
            root_nodes.push(closed);
        }
    }

    Ok(root_nodes)
}
