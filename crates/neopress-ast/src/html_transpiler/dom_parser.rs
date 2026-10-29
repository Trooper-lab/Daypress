//! Native Rust HTML5 Tokenizer & DOM Tree Builder (`html_transpiler/dom_parser.rs`).
//!
//! Parses full HTML5 documents or fragments into a clean `DomNode` tree while preserving
//! inline markup (`<em>`, `<strong>`, `<span>`, `<svg>`, `<code>`, `<br>`) and attributes.
//! Zero unsafe byte slicing; all indexing operates on valid UTF-8 `char_indices`.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub enum DomNode {
    Element(DomElement),
    Text(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct DomElement {
    pub tag: String,
    pub attrs: BTreeMap<String, String>,
    pub children: Vec<DomNode>,
    pub self_closing: bool,
}

impl DomElement {
    pub fn id(&self) -> Option<&str> {
        self.attrs
            .get("id")
            .map(String::as_str)
            .filter(|s| !s.is_empty())
    }

    pub fn classes(&self) -> Vec<&str> {
        self.attrs
            .get("class")
            .map(|c| c.split_whitespace().collect())
            .unwrap_or_default()
    }

    pub fn class_str(&self) -> Option<&str> {
        self.attrs
            .get("class")
            .map(String::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
    }

    pub fn attr(&self, name: &str) -> Option<&str> {
        self.attrs.get(name).map(String::as_str)
    }

    /// Serializes inner children back to HTML.
    pub fn inner_html(&self) -> String {
        let mut out = String::new();
        for child in &self.children {
            child.write_html(&mut out);
        }
        out
    }

    /// Serializes this element and its children to HTML.
    pub fn outer_html(&self) -> String {
        let mut out = String::new();
        self.write_html(&mut out);
        out
    }

    /// Extracts concatenated plain text content.
    pub fn text_content(&self) -> String {
        let mut out = String::new();
        self.collect_text(&mut out);
        out
    }

    fn collect_text(&self, out: &mut String) {
        for child in &self.children {
            match child {
                DomNode::Text(t) => out.push_str(t),
                DomNode::Element(el) => el.collect_text(out),
            }
        }
    }

    pub fn write_html(&self, out: &mut String) {
        out.push('<');
        out.push_str(&self.tag);
        for (k, v) in &self.attrs {
            out.push(' ');
            out.push_str(k);
            out.push_str("=\"");
            out.push_str(&v.replace('&', "&amp;").replace('"', "&quot;"));
            out.push('"');
        }
        if is_void_tag(&self.tag) {
            out.push('>');
            return;
        }
        out.push('>');
        for child in &self.children {
            child.write_html(out);
        }
        out.push_str("</");
        out.push_str(&self.tag);
        out.push('>');
    }
}

impl DomNode {
    pub fn write_html(&self, out: &mut String) {
        match self {
            DomNode::Text(t) => out.push_str(t),
            DomNode::Element(el) => el.write_html(out),
        }
    }
}

pub fn is_void_tag(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

pub fn is_inline_tag(tag: &str) -> bool {
    matches!(
        tag,
        "em" | "strong"
            | "b"
            | "i"
            | "u"
            | "s"
            | "span"
            | "small"
            | "mark"
            | "sub"
            | "sup"
            | "code"
            | "kbd"
            | "samp"
            | "var"
            | "cite"
            | "q"
            | "dfn"
            | "abbr"
            | "time"
            | "br"
            | "wbr"
    )
}

#[derive(Debug)]
enum Token {
    StartTag {
        tag: String,
        attrs: BTreeMap<String, String>,
        self_closing: bool,
    },
    EndTag(String),
    Text(String),
}

/// Parses an HTML document or fragment into a synthetic root `DomElement`.
pub fn parse_html_dom(html: &str) -> DomElement {
    let tokens = tokenize_html(html);
    let mut stack: Vec<DomElement> = vec![DomElement {
        tag: "document".to_string(),
        attrs: BTreeMap::new(),
        children: Vec::new(),
        self_closing: false,
    }];

    for token in tokens {
        match token {
            Token::Text(text) => {
                if let Some(top) = stack.last_mut() {
                    top.children.push(DomNode::Text(text));
                }
            }
            Token::StartTag {
                tag,
                attrs,
                self_closing,
            } => {
                // Handle HTML5 implicit auto-closing tags (e.g. <li> followed by <li>, <dt>/<dd>, <p>)
                auto_close_implicit(&mut stack, &tag);

                let el = DomElement {
                    tag: tag.clone(),
                    attrs,
                    children: Vec::new(),
                    self_closing,
                };

                if self_closing || is_void_tag(&tag) {
                    if let Some(top) = stack.last_mut() {
                        top.children.push(DomNode::Element(el));
                    }
                } else {
                    stack.push(el);
                }
            }
            Token::EndTag(end_tag) => {
                if is_void_tag(&end_tag) {
                    continue;
                }
                // Find matching ancestor tag in stack (never pop index 0 "document")
                if let Some(pos) = stack[1..].iter().rposition(|e| e.tag == end_tag) {
                    let target_idx = pos + 1;
                    while stack.len() > target_idx {
                        if let Some(popped) = stack.pop() {
                            if let Some(parent) = stack.last_mut() {
                                parent.children.push(DomNode::Element(popped));
                            }
                        }
                    }
                }
            }
        }
    }

    while stack.len() > 1 {
        if let Some(popped) = stack.pop() {
            if let Some(parent) = stack.last_mut() {
                parent.children.push(DomNode::Element(popped));
            }
        }
    }

    stack.into_iter().next().unwrap_or(DomElement {
        tag: "document".to_string(),
        attrs: BTreeMap::new(),
        children: Vec::new(),
        self_closing: false,
    })
}

fn auto_close_implicit(stack: &mut Vec<DomElement>, new_tag: &str) {
    if stack.len() <= 1 {
        return;
    }
    let current_tag = stack.last().map(|e| e.tag.as_str()).unwrap_or("");
    let should_pop = match new_tag {
        "li" => current_tag == "li",
        "dt" | "dd" => matches!(current_tag, "dt" | "dd"),
        "tr" => matches!(current_tag, "tr" | "td" | "th"),
        "td" | "th" => matches!(current_tag, "td" | "th"),
        "thead" | "tbody" | "tfoot" => matches!(
            current_tag,
            "thead" | "tbody" | "tfoot" | "tr" | "td" | "th"
        ),
        "p" | "div" | "section" | "article" | "header" | "footer" | "aside" | "nav" | "ul"
        | "ol" | "dl" | "blockquote" | "figure" | "table" | "hr" | "h1" | "h2" | "h3" | "h4"
        | "h5" | "h6" => current_tag == "p",
        _ => false,
    };

    if should_pop {
        if let Some(popped) = stack.pop() {
            if let Some(parent) = stack.last_mut() {
                parent.children.push(DomNode::Element(popped));
            }
        }
        // Re-check in case <td> was inside <tr> when a new <tr> arrives
        if matches!(new_tag, "tr" | "thead" | "tbody" | "tfoot") {
            auto_close_implicit(stack, new_tag);
        }
    }
}

fn slice_safe(s: &str, start: usize, end: usize) -> &str {
    s.get(start..end).unwrap_or("")
}

fn tokenize_html(src: &str) -> Vec<Token> {
    let chars: Vec<(usize, char)> = src.char_indices().collect();
    let byte_at = |idx: usize| chars.get(idx).map(|c| c.0).unwrap_or(src.len());
    let mut tokens = Vec::new();
    let mut i = 0usize;
    let mut text_start = 0usize;

    while i < chars.len() {
        if chars[i].1 == '<' {
            // 1. Comment `<!-- ... -->`
            if starts_with_chars(&chars, i, "<!--") {
                if i > text_start {
                    tokens.push(Token::Text(
                        slice_safe(src, byte_at(text_start), byte_at(i)).to_string(),
                    ));
                }
                i += 4;
                while i < chars.len() && !starts_with_chars(&chars, i, "-->") {
                    i += 1;
                }
                if i < chars.len() {
                    i += 3;
                }
                text_start = i;
                continue;
            }

            // 2. Doctype or declaration `<!...>` or `<?...>`
            if matches!(chars.get(i + 1), Some((_, '!' | '?'))) {
                if i > text_start {
                    tokens.push(Token::Text(
                        slice_safe(src, byte_at(text_start), byte_at(i)).to_string(),
                    ));
                }
                while i < chars.len() && chars[i].1 != '>' {
                    i += 1;
                }
                if i < chars.len() {
                    i += 1;
                }
                text_start = i;
                continue;
            }

            // 3. End tag `</tag>`
            if matches!(chars.get(i + 1), Some((_, '/'))) {
                if let Some((tag_name, next_i)) = parse_end_tag(&chars, src, i) {
                    if i > text_start {
                        tokens.push(Token::Text(
                            slice_safe(src, byte_at(text_start), byte_at(i)).to_string(),
                        ));
                    }
                    tokens.push(Token::EndTag(tag_name));
                    i = next_i;
                    text_start = i;
                    continue;
                }
            }

            // 4. Start tag `<tag ...>`
            if matches!(chars.get(i + 1), Some((_, c)) if c.is_ascii_alphabetic()) {
                if let Some((tag, attrs, self_closing, next_i)) = parse_start_tag(&chars, src, i) {
                    if i > text_start {
                        tokens.push(Token::Text(
                            slice_safe(src, byte_at(text_start), byte_at(i)).to_string(),
                        ));
                    }
                    let is_raw_text = matches!(tag.as_str(), "script" | "style") && !self_closing;
                    tokens.push(Token::StartTag {
                        tag: tag.clone(),
                        attrs,
                        self_closing,
                    });
                    i = next_i;

                    if is_raw_text {
                        let raw_start = i;
                        let close_pat = format!("</{}", tag);
                        while i < chars.len() && !starts_with_chars_ci(&chars, i, &close_pat) {
                            i += 1;
                        }
                        if i > raw_start {
                            tokens.push(Token::Text(
                                slice_safe(src, byte_at(raw_start), byte_at(i)).to_string(),
                            ));
                        }
                        if let Some((end_tag, after_end)) = parse_end_tag(&chars, src, i) {
                            tokens.push(Token::EndTag(end_tag));
                            i = after_end;
                        }
                    }
                    text_start = i;
                    continue;
                }
            }
        }
        i += 1;
    }

    if text_start < chars.len() {
        tokens.push(Token::Text(
            slice_safe(src, byte_at(text_start), src.len()).to_string(),
        ));
    }

    tokens
}

fn starts_with_chars(chars: &[(usize, char)], idx: usize, pat: &str) -> bool {
    for (offset, pc) in pat.chars().enumerate() {
        match chars.get(idx + offset) {
            Some(&(_, c)) if c == pc => {}
            _ => return false,
        }
    }
    true
}

fn starts_with_chars_ci(chars: &[(usize, char)], idx: usize, pat: &str) -> bool {
    for (offset, pc) in pat.chars().enumerate() {
        match chars.get(idx + offset) {
            Some(&(_, c)) if c.eq_ignore_ascii_case(&pc) => {}
            _ => return false,
        }
    }
    true
}

fn parse_end_tag(chars: &[(usize, char)], src: &str, start: usize) -> Option<(String, usize)> {
    let byte_at = |idx: usize| chars.get(idx).map(|c| c.0).unwrap_or(src.len());
    let mut j = start + 2; // skip `</`
    let name_start = j;
    while let Some(&(_, c)) = chars.get(j) {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | ':' | '_') {
            j += 1;
        } else {
            break;
        }
    }
    if j == name_start {
        return None;
    }
    let name = slice_safe(src, byte_at(name_start), byte_at(j)).to_ascii_lowercase();
    while j < chars.len() && chars[j].1 != '>' {
        j += 1;
    }
    if j < chars.len() {
        j += 1;
    }
    Some((name, j))
}

#[allow(clippy::type_complexity)]
fn parse_start_tag(
    chars: &[(usize, char)],
    src: &str,
    start: usize,
) -> Option<(String, BTreeMap<String, String>, bool, usize)> {
    let byte_at = |idx: usize| chars.get(idx).map(|c| c.0).unwrap_or(src.len());
    let mut j = start + 1; // skip `<`
    let name_start = j;
    while let Some(&(_, c)) = chars.get(j) {
        if c.is_ascii_alphanumeric() || matches!(c, '-' | ':' | '_') {
            j += 1;
        } else {
            break;
        }
    }
    if j == name_start {
        return None;
    }
    let tag_name = slice_safe(src, byte_at(name_start), byte_at(j)).to_ascii_lowercase();
    let mut attrs = BTreeMap::new();
    let mut self_closing = false;

    while j < chars.len() {
        while let Some(&(_, c)) = chars.get(j) {
            if c.is_whitespace() {
                j += 1;
            } else {
                break;
            }
        }
        if j >= chars.len() {
            break;
        }
        if chars[j].1 == '>' {
            j += 1;
            break;
        }
        if chars[j].1 == '/' && matches!(chars.get(j + 1), Some((_, '>'))) {
            self_closing = true;
            j += 2;
            break;
        }

        // Parse attribute name
        let attr_start = j;
        while let Some(&(_, c)) = chars.get(j) {
            if !c.is_whitespace() && !matches!(c, '=' | '>' | '/' | '"' | '\'') {
                j += 1;
            } else {
                break;
            }
        }
        if j == attr_start {
            j += 1;
            continue;
        }
        let attr_name = slice_safe(src, byte_at(attr_start), byte_at(j)).to_ascii_lowercase();

        while let Some(&(_, c)) = chars.get(j) {
            if c.is_whitespace() {
                j += 1;
            } else {
                break;
            }
        }

        let mut attr_val = String::new();
        if matches!(chars.get(j), Some((_, '='))) {
            j += 1;
            while let Some(&(_, c)) = chars.get(j) {
                if c.is_whitespace() {
                    j += 1;
                } else {
                    break;
                }
            }
            if let Some(&(_, quote @ ('"' | '\''))) = chars.get(j) {
                j += 1;
                let val_start = j;
                while let Some(&(_, c)) = chars.get(j) {
                    if c == quote {
                        break;
                    }
                    j += 1;
                }
                attr_val = slice_safe(src, byte_at(val_start), byte_at(j)).to_string();
                if j < chars.len() {
                    j += 1;
                }
            } else {
                let val_start = j;
                while let Some(&(_, c)) = chars.get(j) {
                    if c.is_whitespace() || matches!(c, '>' | '/') {
                        break;
                    }
                    j += 1;
                }
                attr_val = slice_safe(src, byte_at(val_start), byte_at(j)).to_string();
            }
        }
        attrs.insert(attr_name, attr_val);
    }

    Some((tag_name, attrs, self_closing, j))
}
