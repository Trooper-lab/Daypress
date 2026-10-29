//! Minimal CSS tokenizer used by the NeoStudio artboard scoper (`templates/admin/builder/css_tokens.rs`).
//!
//! Splits stylesheets into top-level statements/blocks and selector lists into selectors while
//! respecting comments, strings and brackets. All slicing goes through char boundaries.

pub(crate) enum Chunk<'a> {
    Text(&'a str),
    Statement(&'a str),
    Block { prelude: &'a str, body: &'a str },
}

/// Char index just past a comment or string starting at `k`, if one starts there.
pub(crate) fn skip_opaque(chars: &[(usize, char)], k: usize) -> Option<usize> {
    let c = chars.get(k)?.1;
    match c {
        '/' if matches!(chars.get(k + 1), Some((_, '*'))) => {
            let mut j = k + 2;
            while j < chars.len() {
                if chars[j].1 == '*' && matches!(chars.get(j + 1), Some((_, '/'))) {
                    return Some(j + 2);
                }
                j += 1;
            }
            Some(chars.len())
        }
        '"' | '\'' => {
            let mut j = k + 1;
            while let Some(&(_, ch)) = chars.get(j) {
                if ch == '\\' {
                    j += 2;
                } else if ch == c {
                    return Some(j + 1);
                } else {
                    j += 1;
                }
            }
            Some(chars.len())
        }
        _ => None,
    }
}

/// Sub-slice between two char boundaries taken from `char_indices` (never panics).
pub(crate) fn between(src: &str, from: usize, to: usize) -> &str {
    src.get(from..to).unwrap_or("")
}

/// Splits a stylesheet (or a block body) into top-level statements and blocks.
pub(crate) fn split_chunks(src: &str) -> Vec<Chunk<'_>> {
    let chars: Vec<(usize, char)> = src.char_indices().collect();
    let at = |j: usize| chars.get(j).map(|c| c.0).unwrap_or(src.len());
    let slice = |a: usize, b: usize| between(src, at(a), at(b));
    let mut out = Vec::new();
    let (mut start, mut k) = (0usize, 0usize);
    while k < chars.len() {
        if let Some(next) = skip_opaque(&chars, k) {
            k = next;
            continue;
        }
        match chars[k].1 {
            '{' => {
                let mut depth = 1usize;
                let mut j = k + 1;
                while j < chars.len() {
                    if let Some(next) = skip_opaque(&chars, j) {
                        j = next;
                        continue;
                    }
                    match chars[j].1 {
                        '{' => depth += 1,
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                out.push(Chunk::Block {
                    prelude: slice(start, k),
                    body: slice(k + 1, j),
                });
                k = j + 1;
                start = k;
            }
            ';' => {
                out.push(Chunk::Statement(slice(start, k + 1)));
                k += 1;
                start = k;
            }
            '}' => {
                out.push(Chunk::Text(slice(start, k + 1)));
                k += 1;
                start = k;
            }
            _ => k += 1,
        }
    }
    if at(start) < src.len() {
        out.push(Chunk::Text(between(src, at(start), src.len())));
    }
    out
}

/// Splits `s` into leading whitespace/comments, the core text and trailing whitespace.
pub(crate) fn split_padding(s: &str) -> (&str, &str, &str) {
    let mut rest = s;
    loop {
        let t = rest.trim_start();
        let after_comment = t
            .strip_prefix("/*")
            .and_then(|c| c.split_once("*/").map(|(_, after)| after));
        match after_comment {
            Some(after) => rest = after,
            None => {
                rest = t;
                break;
            }
        }
    }
    let lead = s.strip_suffix(rest).unwrap_or("");
    let core = rest.trim_end();
    (lead, core, rest.strip_prefix(core).unwrap_or(""))
}

/// Lower-cased at-rule name (`media`, `import`, ...) when `prelude` is an at-rule.
pub(crate) fn at_keyword(prelude: &str) -> Option<String> {
    let (_, core, _) = split_padding(prelude);
    let name = core.strip_prefix('@')?;
    let name = name
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .next()
        .unwrap_or("");
    Some(name.to_ascii_lowercase())
}

/// Splits on commas outside parentheses, brackets and strings.
pub(crate) fn split_top_level(s: &str, sep: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let (mut depth, mut quote, mut start) = (0i32, None::<char>, 0usize);
    for (i, c) in s.char_indices() {
        match (quote, c) {
            (Some(q), _) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"') | (None, '\'') => quote = Some(c),
            (None, '(') | (None, '[') => depth += 1,
            (None, ')') | (None, ']') => depth -= 1,
            (None, _) if c == sep && depth == 0 => {
                parts.push(between(s, start, i));
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    parts.push(between(s, start, s.len()));
    parts
}

/// Splits a selector into its first compound selector and the remainder (combinator onwards).
pub(crate) fn split_first_compound(sel: &str) -> (&str, &str) {
    let mut depth = 0i32;
    for (i, c) in sel.char_indices() {
        match c {
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            c if depth == 0 && (c.is_whitespace() || matches!(c, '>' | '+' | '~')) => {
                return (between(sel, 0, i), between(sel, i, sel.len()));
            }
            _ => {}
        }
    }
    (sel, "")
}
