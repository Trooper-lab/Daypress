//! URL normalization and boundary-safe uploads remapping utilities.

/// Extracts host and optional port from a URL (e.g. "example.com" or "localhost:8085").
pub fn extract_url_host(url: &str) -> Option<&str> {
    let after_scheme = url.split("://").nth(1)?;
    let host_end = after_scheme
        .find(['/', '?', '#'])
        .unwrap_or(after_scheme.len());
    let host = &after_scheme[..host_end];
    if host.is_empty() {
        None
    } else {
        Some(host)
    }
}

/// Checks if a prefix slice before `/wp-content/uploads/` indicates an absolute or external URL.
pub fn is_external_url_prefix(prefix: &str) -> bool {
    prefix.contains("://")
        || prefix.contains(":\\/\\/")
        || prefix.starts_with("//")
        || prefix.starts_with("\\/\\/")
}

/// Remaps root-relative occurrences of `/wp-content/uploads/` to `/uploads/` (and escaped `\/uploads\/`)
/// in HTML, JSON, CSS, and markdown content, preserving external third-party WordPress URLs.
pub fn remap_root_relative_uploads(content: &str) -> String {
    if !content.contains("/wp-content/uploads/") && !content.contains("\\/wp-content\\/uploads\\/")
    {
        return content.to_string();
    }

    let mut result = String::with_capacity(content.len());
    let mut cursor = 0;

    while cursor < content.len() {
        let remaining = &content[cursor..];
        let idx1 = remaining.find("/wp-content/uploads/");
        let idx2 = remaining.find("\\/wp-content\\/uploads\\/");

        let (match_offset, target_len, replacement) = match (idx1, idx2) {
            (Some(i1), Some(i2)) => {
                if i1 <= i2 {
                    (i1, "/wp-content/uploads/".len(), "/uploads/")
                } else {
                    (i2, "\\/wp-content\\/uploads\\/".len(), "\\/uploads\\/")
                }
            }
            (Some(i1), None) => (i1, "/wp-content/uploads/".len(), "/uploads/"),
            (None, Some(i2)) => (i2, "\\/wp-content\\/uploads\\/".len(), "\\/uploads\\/"),
            (None, None) => {
                result.push_str(remaining);
                break;
            }
        };

        let abs_match = cursor + match_offset;
        result.push_str(&content[cursor..abs_match]);

        let delimiter_pos = content[..abs_match]
            .rfind(|c: char| {
                matches!(
                    c,
                    '"' | '\''
                        | '`'
                        | '<'
                        | '>'
                        | '='
                        | '('
                        | ')'
                        | '['
                        | ']'
                        | '{'
                        | '}'
                        | ' '
                        | '\t'
                        | '\r'
                        | '\n'
                        | ','
                        | ';'
                )
            })
            .map(|i| i + 1)
            .unwrap_or(0);
        let prefix = &content[delimiter_pos..abs_match];

        if is_external_url_prefix(prefix) {
            result.push_str(&content[abs_match..abs_match + target_len]);
        } else {
            result.push_str(replacement);
        }

        cursor = abs_match + target_len;
    }

    result
}

/// Decodes minimal HTML numeric/named entities used to obfuscate URL schemes (`javascript:`).
fn decode_entities_for_scheme_probe(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            let rest = &raw[i..];
            if let Some(semi) = rest.find(';').filter(|&p| p <= 12) {
                let inner = &rest[1..semi];
                if let Some(hex) = inner.strip_prefix("#x").or_else(|| inner.strip_prefix("#X")) {
                    if let Ok(cp) = u32::from_str_radix(hex, 16) {
                        if let Some(ch) = char::from_u32(cp) {
                            out.push(ch);
                            i += semi + 1;
                            continue;
                        }
                    }
                } else if let Some(dec) = inner.strip_prefix('#') {
                    if let Ok(cp) = dec.parse::<u32>() {
                        if let Some(ch) = char::from_u32(cp) {
                            out.push(ch);
                            i += semi + 1;
                            continue;
                        }
                    }
                } else if inner.eq_ignore_ascii_case("colon") {
                    out.push(':');
                    i += semi + 1;
                    continue;
                } else if inner.eq_ignore_ascii_case("tab")
                    || inner.eq_ignore_ascii_case("newline")
                {
                    i += semi + 1;
                    continue;
                }
            }
        }
        let ch = raw[i..].chars().next().unwrap_or(' ');
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn is_url_scheme_allowed(raw: &str) -> bool {
    let cleaned: String = raw.chars().filter(|c| !c.is_control()).collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return true;
    }

    let decoded = decode_entities_for_scheme_probe(trimmed);
    let compact: String = decoded
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect::<String>()
        .to_ascii_lowercase();

    // Also inspect un-decoded compact form in case malformed numeric entities without `;`
    // (`&#106avascript:`) are parsed by browsers in href/src attributes.
    let raw_compact: String = trimmed
        .chars()
        .filter(|c| !c.is_whitespace() && !c.is_control())
        .collect::<String>()
        .to_ascii_lowercase();

    for candidate in [&compact, &raw_compact] {
        if candidate.starts_with("javascript:")
            || candidate.starts_with("vbscript:")
            || candidate.starts_with("data:text/html")
            || candidate.starts_with("data:image/svg")
        {
            return false;
        }
        if let Some(delim_idx) = candidate.find([':', '/', '?', '#']) {
            if candidate.as_bytes()[delim_idx] == b':' {
                let scheme = &candidate[..delim_idx];
                let after = &candidate[delim_idx + 1..];
                match scheme {
                    "http" | "https" | "mailto" | "tel" => {}
                    "data" => {
                        let ok_raster = after.starts_with("image/png;")
                            || after.starts_with("image/png,")
                            || after.starts_with("image/jpeg;")
                            || after.starts_with("image/jpeg,")
                            || after.starts_with("image/jpg;")
                            || after.starts_with("image/jpg,")
                            || after.starts_with("image/webp;")
                            || after.starts_with("image/webp,")
                            || after.starts_with("image/gif;")
                            || after.starts_with("image/gif,")
                            || after.starts_with("image/avif;")
                            || after.starts_with("image/avif,");
                        if !ok_raster {
                            return false;
                        }
                    }
                    _ => return false,
                }
            }
        }
    }
    true
}

/// Validates a link `href` / `action` URL against an allowlist of safe protocols
/// (`http`, `https`, `mailto`, `tel`, benign raster `data:image/*`, and relative paths),
/// returning `"#"` when a dangerous scheme (`javascript:`, `vbscript:`, `data:text/html`, etc.) is detected.
pub fn sanitize_safe_url(raw: &str) -> String {
    let cleaned: String = raw.chars().filter(|c| !c.is_control()).collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if is_url_scheme_allowed(trimmed) {
        trimmed.to_string()
    } else {
        "#".to_string()
    }
}

/// Validates a media/embed `src` URL, returning an empty string for dangerous schemes
/// so `<img src="">` / `<iframe src="">` never execute `javascript:` or load self.
pub fn sanitize_media_url(raw: &str) -> String {
    let cleaned: String = raw.chars().filter(|c| !c.is_control()).collect();
    let trimmed = cleaned.trim();
    if trimmed.is_empty() || !is_url_scheme_allowed(trimmed) {
        String::new()
    } else {
        trimmed.to_string()
    }
}

/// Restricts class suffixes / modifier tokens interpolated into HTML `class="..."` attributes.
pub fn sanitize_css_class_token(raw: &str) -> String {
    raw.chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | ' '))
        .collect::<String>()
        .trim()
        .to_string()
}

/// Sanitizes a CSS property value or stylesheet fragment against `<style>` breakout (`</style>`)
/// and dangerous legacy/script URL patterns (`expression(`, `javascript:`, `vbscript:`).
pub fn sanitize_css_value(raw: &str) -> String {
    let cleaned: String = raw
        .chars()
        .filter(|c| !c.is_control() || *c == '\n' || *c == '\t')
        .collect();
    let without_quotes = cleaned.replace('"', "'");
    // Strip CSS comments before checking for obfuscated `ex/* */pression(` / `java/* */script:`
    let mut no_comments = String::with_capacity(without_quotes.len());
    let mut chars = without_quotes.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            let mut prev = ' ';
            for n in chars.by_ref() {
                if prev == '*' && n == '/' {
                    break;
                }
                prev = n;
            }
            continue;
        }
        no_comments.push(c);
    }
    let compact: String = no_comments
        .chars()
        .filter(|c| !c.is_whitespace() && *c != '\\')
        .collect::<String>()
        .to_ascii_lowercase();
    if compact.contains("</style")
        || compact.contains("</")
        || compact.contains("expression(")
        || compact.contains("javascript:")
        || compact.contains("vbscript:")
        || compact.contains("-moz-binding")
        || compact.contains("behavior:")
        || compact.contains("data:text/html")
        || compact.contains("data:image/svg")
    {
        return String::new();
    }
    without_quotes.trim().to_string()
}

fn escape_html_text_and_attr(raw: &str) -> String {
    raw.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn parse_tag_attrs(attr_str: &str) -> Vec<(String, String)> {
    let mut attrs = Vec::new();
    let bytes = attr_str.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        while i < bytes.len() && (bytes[i].is_ascii_whitespace() || bytes[i] == b'/') {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let name_start = i;
        while i < bytes.len()
            && !bytes[i].is_ascii_whitespace()
            && bytes[i] != b'='
            && bytes[i] != b'>'
            && bytes[i] != b'/'
        {
            i += 1;
        }
        let name = attr_str[name_start..i].to_ascii_lowercase();
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let mut val = String::new();
        if i < bytes.len() && bytes[i] == b'=' {
            i += 1;
            while i < bytes.len() && bytes[i].is_ascii_whitespace() {
                i += 1;
            }
            if i < bytes.len() && (bytes[i] == b'"' || bytes[i] == b'\'') {
                let q = bytes[i];
                i += 1;
                let v_start = i;
                while i < bytes.len() && bytes[i] != q {
                    i += 1;
                }
                val = attr_str[v_start..i].to_string();
                if i < bytes.len() {
                    i += 1;
                }
            } else {
                let v_start = i;
                while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                    i += 1;
                }
                val = attr_str[v_start..i].to_string();
            }
        }
        if !name.is_empty() {
            attrs.push((name, val));
        }
    }
    attrs
}

/// Strict allowlist HTML sanitizer for untrusted user HTML (posts/pages/layouts from Contributors
/// and Authors without `unfiltered_html`, accordion/tab/widget HTML bodies, and bio/excerpt fields).
/// Drops `<script>`, `<style>`, `<svg>`, `<iframe>`, `<object>`, `<embed>`, `<form>`, `<meta>`,
/// `<base>`, HTML comments, all `on*` event attributes, and dangerous URL schemes.
pub fn sanitize_untrusted_html(raw: &str) -> String {
    const STRIP_SUBTREE_TAGS: &[&str] = &[
        "script", "style", "svg", "math", "iframe", "object", "embed", "applet", "template",
        "noscript", "noembed", "xmp", "textarea", "title",
    ];
    const ALLOWED_TAGS: &[&str] = &[
        "p", "br", "hr", "div", "span", "h1", "h2", "h3", "h4", "h5", "h6", "strong", "b", "em",
        "i", "u", "s", "del", "ins", "mark", "small", "sub", "sup", "code", "pre", "blockquote",
        "ul", "ol", "li", "dl", "dt", "dd", "table", "thead", "tbody", "tfoot", "tr", "th", "td",
        "figure", "figcaption", "caption", "details", "summary", "a", "img", "audio", "video",
        "source",
    ];
    const VOID_TAGS: &[&str] = &["br", "hr", "img", "source"];

    let mut out = String::with_capacity(raw.len());
    let mut i = 0usize;
    let bytes = raw.as_bytes();
    let mut skip_tag: Option<String> = None;

    while i < bytes.len() {
        if bytes[i] != b'<' {
            let next_lt = raw[i..].find('<').map(|off| i + off).unwrap_or(raw.len());
            if skip_tag.is_none() {
                out.push_str(&raw[i..next_lt]);
            }
            i = next_lt;
            continue;
        }

        // Comment or DOCTYPE / processing instruction -> strip safely
        if raw[i..].starts_with("<!--") {
            if let Some(end) = raw[i + 4..].find("-->") {
                i += 4 + end + 3;
            } else {
                break;
            }
            continue;
        }
        if raw[i..].starts_with("<!") || raw[i..].starts_with("<?") {
            if let Some(end) = raw[i..].find('>') {
                i += end + 1;
            } else {
                break;
            }
            continue;
        }

        // Find end of tag respecting quoted attributes
        let mut j = i + 1;
        let mut quote: Option<u8> = None;
        while j < bytes.len() {
            match quote {
                Some(q) => {
                    if bytes[j] == q {
                        quote = None;
                    }
                }
                None => match bytes[j] {
                    b'"' | b'\'' => quote = Some(bytes[j]),
                    b'>' => break,
                    b'<' => break,
                    _ => {}
                },
            }
            j += 1;
        }
        if j >= bytes.len() || bytes[j] != b'>' {
            if skip_tag.is_none() {
                out.push_str("&lt;");
            }
            i += 1;
            continue;
        }

        let inner = raw[i + 1..j].trim();
        i = j + 1;
        if inner.is_empty() {
            continue;
        }

        let is_closing = inner.starts_with('/');
        let tag_body = inner.trim_start_matches('/').trim();
        let name_end = tag_body
            .find(|c: char| c.is_ascii_whitespace() || c == '/')
            .unwrap_or(tag_body.len());
        let tag_name = tag_body[..name_end].to_ascii_lowercase();
        let attr_slice = &tag_body[name_end..];

        if let Some(ref skipping) = skip_tag {
            if is_closing && &tag_name == skipping {
                skip_tag = None;
            }
            continue;
        }

        if !is_closing && STRIP_SUBTREE_TAGS.contains(&tag_name.as_str()) {
            let self_closing = attr_slice.trim_end().ends_with('/');
            if !self_closing {
                skip_tag = Some(tag_name);
            }
            continue;
        }

        if !ALLOWED_TAGS.contains(&tag_name.as_str()) {
            continue;
        }

        if is_closing {
            if !VOID_TAGS.contains(&tag_name.as_str()) {
                out.push_str("</");
                out.push_str(&tag_name);
                out.push('>');
            }
            continue;
        }

        // Re-serialize opening tag with strictly allowlisted attributes
        out.push('<');
        out.push_str(&tag_name);
        let parsed_attrs = parse_tag_attrs(attr_slice);
        let mut has_target_blank = false;
        let mut emitted_rel = false;
        for (k, v) in parsed_attrs {
            if k.starts_with("on") {
                continue;
            }
            match k.as_str() {
                "href" => {
                    if tag_name == "a" {
                        let safe = sanitize_safe_url(&v);
                        out.push_str(&format!(" href=\"{}\"", escape_html_text_and_attr(&safe)));
                    }
                }
                "src" | "poster" => {
                    if matches!(tag_name.as_str(), "img" | "audio" | "video" | "source") {
                        let safe = sanitize_media_url(&v);
                        if !safe.is_empty() {
                            out.push_str(&format!(
                                " {}=\"{}\"",
                                k,
                                escape_html_text_and_attr(&safe)
                            ));
                        }
                    }
                }
                "style" => {
                    let safe_css = sanitize_css_value(&v);
                    if !safe_css.is_empty() {
                        out.push_str(&format!(
                            " style=\"{}\"",
                            escape_html_text_and_attr(&safe_css)
                        ));
                    }
                }
                "target" if tag_name == "a" => {
                    let t = match v.trim() {
                        "_blank" => {
                            has_target_blank = true;
                            "_blank"
                        }
                        "_self" => "_self",
                        "_parent" => "_parent",
                        "_top" => "_top",
                        _ => continue,
                    };
                    out.push_str(&format!(" target=\"{}\"", t));
                }
                "rel" if tag_name == "a" => {
                    emitted_rel = true;
                    out.push_str(&format!(" rel=\"{}\"", escape_html_text_and_attr(&v)));
                }
                "class" | "id" | "title" | "alt" | "width" | "height" | "colspan" | "rowspan"
                | "loading" | "decoding" | "start" | "type" => {
                    out.push_str(&format!(" {}=\"{}\"", k, escape_html_text_and_attr(&v)));
                }
                "controls" | "open" => {
                    out.push(' ');
                    out.push_str(&k);
                }
                _ => {}
            }
        }
        if has_target_blank && !emitted_rel {
            out.push_str(" rel=\"noopener noreferrer\"");
        }
        if VOID_TAGS.contains(&tag_name.as_str()) {
            out.push_str(" />");
        } else {
            out.push('>');
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remap_preserves_external_wp_uploads() {
        let input = r#"<img src="/wp-content/uploads/local.jpg"><a href="https://other.org/wp-content/uploads/doc.pdf">Doc</a>"#;
        let expected = r#"<img src="/uploads/local.jpg"><a href="https://other.org/wp-content/uploads/doc.pdf">Doc</a>"#;
        assert_eq!(remap_root_relative_uploads(input), expected);
    }

    #[test]
    fn test_remap_escaped_json_preserves_external() {
        let input = r#"{"local":"\/wp-content\/uploads\/1.jpg","ext":"https:\/\/cdn.org\/wp-content\/uploads\/2.jpg"}"#;
        let expected =
            r#"{"local":"\/uploads\/1.jpg","ext":"https:\/\/cdn.org\/wp-content\/uploads\/2.jpg"}"#;
        assert_eq!(remap_root_relative_uploads(input), expected);
    }

    #[test]
    fn test_sanitize_urls_css_and_untrusted_html() {
        assert_eq!(sanitize_safe_url("javascript:alert(1)"), "#");
        assert_eq!(sanitize_safe_url("&#106;avascript:alert(1)"), "#");
        assert_eq!(sanitize_safe_url("vbscript:msgbox(1)"), "#");
        assert_eq!(sanitize_safe_url("data:text/html,<script>1</script>"), "#");
        assert_eq!(
            sanitize_safe_url("https://example.com/a?x=1&y=2"),
            "https://example.com/a?x=1&y=2"
        );
        assert_eq!(sanitize_media_url("javascript:alert(1)"), "");
        assert_eq!(
            sanitize_media_url("data:image/svg+xml,<svg onload=alert(1)>"),
            ""
        );
        assert!(
            sanitize_media_url("data:image/png;base64,iVBORw0KGgo=").starts_with("data:image/png;")
        );

        let css = sanitize_css_value("red</style><script>alert(1)</script>");
        assert!(!css.to_ascii_lowercase().contains("</style"));

        let html = sanitize_untrusted_html(
            r#"<p>Hello</p><script>evil()</script><img src=x onerror=alert(1)><svg onload=alert(2)></svg><a href="javascript:alert(3)" target="_blank">Click</a>"#,
        );
        assert!(html.contains("<p>Hello</p>"));
        assert!(!html.contains("<script"));
        assert!(!html.contains("onerror"));
        assert!(!html.contains("<svg"));
        assert!(!html.contains("javascript:"));
        assert!(html.contains(r##"href="#""##));
        assert!(html.contains(r#"rel="noopener noreferrer""#));
    }
}
