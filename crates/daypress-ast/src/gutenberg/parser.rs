use serde_json::Value;

#[derive(Debug, Clone)]
pub(crate) struct GutenbergBlock {
    pub(crate) name: String,
    pub(crate) attrs: Value,
    pub(crate) inner_html: String,
    pub(crate) children: Vec<GutenbergBlock>,
}

#[derive(Debug, PartialEq)]
pub(crate) enum BlockToken {
    Open { name: String, attrs: Value },
    Close { name: String },
    SelfClosing { name: String, attrs: Value },
    Html(String),
}

fn scan_block_comment(input: &str, start: usize) -> Option<(BlockToken, usize)> {
    let bytes = input.as_bytes();
    if !input[start..].starts_with("<!--") {
        return None;
    }
    let mut i = start + 4;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if i >= bytes.len() {
        return None;
    }

    let is_close = if bytes[i] == b'/' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        true
    } else {
        false
    };

    if !input[i..].starts_with("wp:") {
        return None;
    }
    i += 3;

    let name_start = i;
    while i < bytes.len()
        && (bytes[i].is_ascii_alphanumeric()
            || bytes[i] == b'_'
            || bytes[i] == b'-'
            || bytes[i] == b'/')
    {
        i += 1;
    }
    if i == name_start {
        return None;
    }

    let raw_name = &input[name_start..i];
    let name = if !raw_name.contains('/') {
        format!("core/{}", raw_name)
    } else {
        raw_name.to_string()
    };

    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }

    let mut attrs = Value::Null;
    if !is_close && i < bytes.len() && bytes[i] == b'{' {
        let json_start = i;
        let mut depth = 0usize;
        let mut in_string = false;
        let mut escaped = false;
        let mut json_end = None;

        while i < bytes.len() {
            let b = bytes[i];
            if escaped {
                escaped = false;
            } else if b == b'\\' && in_string {
                escaped = true;
            } else if b == b'"' {
                in_string = !in_string;
            } else if !in_string {
                if b == b'{' {
                    depth += 1;
                } else if b == b'}' {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        json_end = Some(i + 1);
                        i += 1;
                        break;
                    }
                }
            }
            i += 1;
        }

        let j_end = json_end?;
        let json_str = &input[json_start..j_end];
        if let Ok(v) = serde_json::from_str::<Value>(json_str) {
            attrs = v;
        }

        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
    }

    if i >= bytes.len() {
        return None;
    }

    let is_self_closing = if !is_close && bytes[i] == b'/' {
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        true
    } else {
        false
    };

    if !input[i..].starts_with("-->") {
        return None;
    }
    let end = i + 3;

    let token = if is_close {
        BlockToken::Close { name }
    } else if is_self_closing {
        BlockToken::SelfClosing { name, attrs }
    } else {
        BlockToken::Open { name, attrs }
    };

    Some((token, end))
}

pub(crate) fn tokenize_gutenberg(input: &str) -> Vec<BlockToken> {
    let mut tokens = Vec::new();
    let mut i = 0;
    let mut last_html_start = 0;

    while i < input.len() {
        if let Some(rel_pos) = input[i..].find("<!--") {
            let comment_start = i + rel_pos;
            if let Some((token, comment_end)) = scan_block_comment(input, comment_start) {
                if comment_start > last_html_start {
                    let html = &input[last_html_start..comment_start];
                    if !html.trim().is_empty() {
                        tokens.push(BlockToken::Html(html.to_string()));
                    }
                }
                tokens.push(token);
                i = comment_end;
                last_html_start = comment_end;
            } else {
                i = comment_start + 4;
            }
        } else {
            break;
        }
    }

    if last_html_start < input.len() {
        let html = &input[last_html_start..];
        if !html.trim().is_empty() {
            tokens.push(BlockToken::Html(html.to_string()));
        }
    }

    tokens
}

pub(crate) fn parse_gutenberg_tree(tokens: Vec<BlockToken>) -> Vec<GutenbergBlock> {
    let mut root_blocks: Vec<GutenbergBlock> = Vec::new();
    let mut stack: Vec<GutenbergBlock> = Vec::new();

    for token in tokens {
        match token {
            BlockToken::Open { name, attrs } => {
                stack.push(GutenbergBlock {
                    name,
                    attrs,
                    inner_html: String::new(),
                    children: Vec::new(),
                });
            }
            BlockToken::SelfClosing { name, attrs } => {
                let blk = GutenbergBlock {
                    name,
                    attrs,
                    inner_html: String::new(),
                    children: Vec::new(),
                };
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(blk);
                } else {
                    root_blocks.push(blk);
                }
            }
            BlockToken::Close { name } => {
                let target = name.strip_prefix("core/").unwrap_or(&name);
                if let Some(pos) = stack.iter().rposition(|b| {
                    let b_name = b.name.strip_prefix("core/").unwrap_or(&b.name);
                    b_name == target
                }) {
                    while stack.len() > pos + 1 {
                        if let Some(unclosed) = stack.pop() {
                            if let Some(parent) = stack.last_mut() {
                                parent.children.push(unclosed);
                            } else {
                                root_blocks.push(unclosed);
                            }
                        }
                    }
                    if let Some(closed) = stack.pop() {
                        if let Some(parent) = stack.last_mut() {
                            parent.children.push(closed);
                        } else {
                            root_blocks.push(closed);
                        }
                    }
                }
            }
            BlockToken::Html(html) => {
                if let Some(top) = stack.last_mut() {
                    top.inner_html.push_str(&html);
                } else {
                    root_blocks.push(GutenbergBlock {
                        name: "core/freeform".to_string(),
                        attrs: Value::Null,
                        inner_html: html,
                        children: Vec::new(),
                    });
                }
            }
        }
    }

    while let Some(unclosed) = stack.pop() {
        if let Some(parent) = stack.last_mut() {
            parent.children.push(unclosed);
        } else {
            root_blocks.push(unclosed);
        }
    }

    root_blocks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_balanced_brace_nested_json() {
        let input = r#"<!-- wp:columns {"style":{"spacing":{"padding":{"top":"20px","bottom":"20px"}}}} -->
<!-- wp:column {"width":"50%"} -->
<p>Col 1</p>
<!-- /wp:column -->
<!-- /wp:columns -->"#;
        let tokens = tokenize_gutenberg(input);
        assert_eq!(tokens.len(), 5);
        match &tokens[0] {
            BlockToken::Open { name, attrs } => {
                assert_eq!(name, "core/columns");
                assert_eq!(attrs["style"]["spacing"]["padding"]["top"], "20px");
            }
            _ => panic!("Expected Open token with nested attrs"),
        }
        let tree = parse_gutenberg_tree(tokens);
        assert_eq!(tree.len(), 1);
        assert_eq!(tree[0].name, "core/columns");
        assert_eq!(tree[0].children.len(), 1);
        assert_eq!(tree[0].children[0].name, "core/column");
    }

    #[test]
    fn test_braces_in_string_attributes() {
        let input = r#"<!-- wp:paragraph {"placeholder":"Use {name} tag here"} -->
<p>Hello {name}</p>
<!-- /wp:paragraph -->"#;
        let tokens = tokenize_gutenberg(input);
        assert_eq!(tokens.len(), 3);
        match &tokens[0] {
            BlockToken::Open { attrs, .. } => {
                assert_eq!(attrs["placeholder"], "Use {name} tag here");
            }
            _ => panic!("Expected Open token"),
        }
    }

    #[test]
    fn test_zero_raw_wp_comments_in_html() {
        let input = r#"<!-- wp:group {"layout":{"type":"constrained"}} -->
<div class="wp-block-group">
<!-- wp:paragraph -->
<p>Zero comment leak!</p>
<!-- /wp:paragraph -->
</div>
<!-- /wp:group -->"#;
        let tokens = tokenize_gutenberg(input);
        for t in &tokens {
            if let BlockToken::Html(h) = t {
                assert!(!h.contains("<!-- wp:"), "Leaked raw wp comment: {}", h);
                assert!(
                    !h.contains("<!-- /wp:"),
                    "Leaked raw wp close comment: {}",
                    h
                );
            }
        }
    }
}
