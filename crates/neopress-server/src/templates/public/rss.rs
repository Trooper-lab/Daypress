//! RSS 2.0 feed document for `/feed/`, `/category/{slug}/feed/` and `/tag/{slug}/feed/`.
//!
//! Every text value is XML-escaped after dropping characters XML 1.0 forbids; post HTML goes
//! into `content:encoded` as CDATA with any `]]>` split so it cannot end the section early.

use crate::models::WpPostResponse;
use crate::templates::utils::clean_wp_content;
use chrono::{NaiveDateTime, Utc};
use std::sync::LazyLock;

/// Channel-level values of a feed.
pub struct FeedChannel<'a> {
    /// `<title>`: the site name, plus ` - Term` for category and tag feeds.
    pub title: &'a str,
    /// `<link>`: the site's home URL (no trailing slash).
    pub site_url: &'a str,
    /// Absolute URL of the feed itself (`atom:link rel="self"`).
    pub self_url: &'a str,
    /// `<description>`: the tagline.
    pub description: &'a str,
    /// WordPress locale such as `en_US`, written as `en-US`.
    pub language: &'a str,
}

/// Words kept in an item description built from the post content (WordPress default).
const EXCERPT_WORDS: usize = 55;

/// Drops characters XML 1.0 does not allow: control characters other than tab, LF and CR,
/// and the non-characters U+FFFE / U+FFFF.
pub fn strip_invalid_xml_chars(s: &str) -> String {
    s.chars().filter(|&c| is_xml_char(c)).collect()
}

fn is_xml_char(c: char) -> bool {
    matches!(c, '\t' | '\n' | '\r') || (c >= ' ' && c != '\u{FFFE}' && c != '\u{FFFF}')
}

/// Escapes text for an XML element or attribute value.
pub fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars().filter(|&c| is_xml_char(c)) {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// Wraps HTML in a CDATA section, splitting `]]>` so the markup cannot close it.
pub fn cdata(s: &str) -> String {
    format!(
        "<![CDATA[{}]]>",
        strip_invalid_xml_chars(s).replace("]]>", "]]]]><![CDATA[>")
    )
}

static SCRIPT_STYLE_RE: LazyLock<Option<regex::Regex>> = LazyLock::new(|| {
    regex::Regex::new(r"(?is)<script\b.*?</script\s*>|<style\b.*?</style\s*>").ok()
});

static ROOT_RELATIVE_URL_RE: LazyLock<Option<regex::Regex>> =
    LazyLock::new(|| regex::Regex::new(r#"(\s(?:src|href)=["'])/([^/])"#).ok());

/// Plain text of an HTML fragment: scripts, styles and tags removed, common entities decoded,
/// whitespace collapsed.
pub fn html_to_text(html: &str) -> String {
    let cleaned = clean_wp_content(html);
    let without_code = match SCRIPT_STYLE_RE.as_ref() {
        Some(re) => re.replace_all(&cleaned, " ").into_owned(),
        None => cleaned,
    };
    let mut text = String::with_capacity(without_code.len());
    let mut chars = without_code.chars().peekable();
    let mut in_tag = false;
    while let Some(c) = chars.next() {
        if in_tag {
            if c == '>' {
                in_tag = false;
                text.push(' ');
            }
        } else if c == '<'
            && chars
                .peek()
                .is_some_and(|n| n.is_ascii_alphabetic() || matches!(n, '/' | '!' | '?'))
        {
            in_tag = true;
        } else {
            text.push(c);
        }
    }
    decode_entities(&text)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Decodes numeric entities and the named ones that commonly appear in post titles.
fn decode_entities(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(pos) = rest.find('&') {
        out.push_str(&rest[..pos]);
        let tail = &rest[pos..];
        let decoded = tail
            .char_indices()
            .take(12)
            .find(|&(_, c)| c == ';')
            .and_then(|(end, _)| decode_entity(&tail[1..end]).map(|ch| (ch, end + 1)));
        match decoded {
            Some((ch, len)) => {
                out.push(ch);
                rest = &tail[len..];
            }
            None => {
                out.push('&');
                rest = &tail[1..];
            }
        }
    }
    out.push_str(rest);
    out
}

fn decode_entity(name: &str) -> Option<char> {
    if let Some(num) = name.strip_prefix('#') {
        let code = match num.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => num.parse::<u32>().ok()?,
        };
        return char::from_u32(code);
    }
    Some(match name {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => ' ',
        "hellip" => '\u{2026}',
        "ndash" => '\u{2013}',
        "mdash" => '\u{2014}',
        "lsquo" => '\u{2018}',
        "rsquo" => '\u{2019}',
        "ldquo" => '\u{201C}',
        "rdquo" => '\u{201D}',
        "laquo" => '\u{00AB}',
        "raquo" => '\u{00BB}',
        "copy" => '\u{00A9}',
        "reg" => '\u{00AE}',
        "trade" => '\u{2122}',
        _ => return None,
    })
}

/// The first `max_words` words of `text`, with ` […]` when it was cut (`wp_trim_words`).
fn trim_words(text: &str, max_words: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= max_words {
        return words.join(" ");
    }
    format!("{} [\u{2026}]", words[..max_words].join(" "))
}

/// Makes `src="/..."` and `href="/..."` absolute, so feed readers can load images and links.
fn absolutize_urls(html: &str, site_url: &str) -> String {
    match ROOT_RELATIVE_URL_RE.as_ref() {
        Some(re) => re
            .replace_all(html, |caps: &regex::Captures| {
                format!("{}{}/{}", &caps[1], site_url, &caps[2])
            })
            .into_owned(),
        None => html.to_string(),
    }
}

fn parse_post_date(raw: &str) -> Option<NaiveDateTime> {
    let s = raw.trim().replace('T', " ");
    if s.starts_with("0000") {
        return None;
    }
    NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").ok()
}

/// RFC 2822 form of a UTC timestamp, zero-padded like WordPress (`Tue, 02 Jan 2001 10:00:00 +0000`).
fn format_rfc2822(d: &NaiveDateTime) -> String {
    d.format("%a, %d %b %Y %H:%M:%S +0000").to_string()
}

/// RFC 2822 date of a stored UTC timestamp (falls back to `fallback` when it is unset).
fn rfc2822(primary: &str, fallback: &str) -> Option<String> {
    parse_post_date(primary)
        .or_else(|| parse_post_date(fallback))
        .map(|d| format_rfc2822(&d))
}

fn render_item(post: &WpPostResponse, site_url: &str) -> String {
    let mut xml = String::from("\t<item>\n");
    xml.push_str(&format!(
        "\t\t<title>{}</title>\n\t\t<link>{}</link>\n",
        xml_escape(&html_to_text(&post.title.rendered)),
        xml_escape(&post.link)
    ));
    if !post.author_name.trim().is_empty() {
        xml.push_str(&format!(
            "\t\t<dc:creator>{}</dc:creator>\n",
            xml_escape(post.author_name.trim())
        ));
    }
    if let Some(date) = rfc2822(&post.date_gmt, &post.date) {
        xml.push_str(&format!("\t\t<pubDate>{}</pubDate>\n", date));
    }
    for name in post.category_names.iter().chain(post.tag_names.iter()) {
        xml.push_str(&format!(
            "\t\t<category>{}</category>\n",
            xml_escape(&html_to_text(name))
        ));
    }
    let guid = post.guid.rendered.trim();
    if guid.is_empty() {
        xml.push_str(&format!(
            "\t\t<guid isPermaLink=\"true\">{}</guid>\n",
            xml_escape(&post.link)
        ));
    } else {
        xml.push_str(&format!(
            "\t\t<guid isPermaLink=\"false\">{}</guid>\n",
            xml_escape(guid)
        ));
    }
    let manual_excerpt = html_to_text(&post.excerpt.rendered);
    let content_text = html_to_text(&post.content.rendered);
    let description = if manual_excerpt.is_empty() {
        trim_words(&content_text, EXCERPT_WORDS)
    } else {
        manual_excerpt
    };
    xml.push_str(&format!(
        "\t\t<description>{}</description>\n",
        xml_escape(&description)
    ));
    let content_html = clean_wp_content(&post.content.rendered);
    if !content_html.trim().is_empty() {
        xml.push_str(&format!(
            "\t\t<content:encoded>{}</content:encoded>\n",
            cdata(&absolutize_urls(&content_html, site_url))
        ));
    }
    xml.push_str("\t</item>\n");
    xml
}

/// Renders the complete RSS 2.0 document for `posts` (already newest first).
pub fn render_rss_feed(channel: &FeedChannel, posts: &[WpPostResponse]) -> String {
    let last_build = posts
        .iter()
        .filter_map(|p| {
            parse_post_date(&p.modified_gmt)
                .or_else(|| parse_post_date(&p.date_gmt))
                .or_else(|| parse_post_date(&p.date))
        })
        .max()
        .unwrap_or_else(|| Utc::now().naive_utc());

    let mut xml = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    xml.push_str(
        "<rss version=\"2.0\"\n\txmlns:content=\"http://purl.org/rss/1.0/modules/content/\"\n\txmlns:dc=\"http://purl.org/dc/elements/1.1/\"\n\txmlns:atom=\"http://www.w3.org/2005/Atom\">\n<channel>\n",
    );
    xml.push_str(&format!(
        "\t<title>{}</title>\n\t<atom:link href=\"{}\" rel=\"self\" type=\"application/rss+xml\" />\n\t<link>{}</link>\n\t<description>{}</description>\n\t<lastBuildDate>{}</lastBuildDate>\n",
        xml_escape(&html_to_text(channel.title)),
        xml_escape(channel.self_url),
        xml_escape(channel.site_url),
        xml_escape(&html_to_text(channel.description)),
        format_rfc2822(&last_build)
    ));
    let language = channel.language.trim().replace('_', "-");
    if !language.is_empty() {
        xml.push_str(&format!(
            "\t<language>{}</language>\n",
            xml_escape(&language)
        ));
    }
    xml.push_str("\t<generator>Neopress</generator>\n");
    for post in posts {
        xml.push_str(&render_item(post, channel.site_url));
    }
    xml.push_str("</channel>\n</rss>\n");
    xml
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_markup_and_drops_invalid_xml_chars() {
        assert_eq!(
            xml_escape("Fish & Chips <b>\"5\" 'x'\u{0}\u{1F}\u{FFFF}\t"),
            "Fish &amp; Chips &lt;b&gt;&quot;5&quot; &#39;x&#39;\t"
        );
        assert_eq!(cdata("a]]>b"), "<![CDATA[a]]]]><![CDATA[>b]]>");
    }

    #[test]
    fn html_to_text_strips_tags_and_decodes_entities() {
        assert_eq!(
            html_to_text("<!-- wp:paragraph --><p>Tom &amp; Jerry&#8217;s <em>day</em></p><script>x()</script> 5 < 6"),
            "Tom & Jerry\u{2019}s day 5 < 6"
        );
        assert_eq!(
            html_to_text("AT&T &unknown; &#xZZ;"),
            "AT&T &unknown; &#xZZ;"
        );
    }

    #[test]
    fn trims_long_content_and_absolutizes_urls() {
        let long = (1..=60)
            .map(|n| n.to_string())
            .collect::<Vec<_>>()
            .join(" ");
        assert!(trim_words(&long, 55).ends_with("55 [\u{2026}]"));
        assert_eq!(
            absolutize_urls(
                r#"<img src="/uploads/a.png"><a href="//cdn.test/x">"#,
                "https://example.com"
            ),
            r#"<img src="https://example.com/uploads/a.png"><a href="//cdn.test/x">"#
        );
    }
}
