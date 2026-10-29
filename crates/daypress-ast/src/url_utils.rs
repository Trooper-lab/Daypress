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
}
