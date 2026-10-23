use regex::Regex;

/// WordPress `wpautop()` port:
/// Converts double line-breaks into `<p>...</p>` paragraphs and optional single line-breaks
/// into `<br />\n`, while preserving block-level HTML elements (`<div>`, `<section>`, `<pre>`,
/// `<script>`, `<style>`, `<table>`, `<ul>`, `<ol>`, `<blockquote>`, `<h1>`..`<h6>`).
pub fn wpautop(pee: &str, br: bool) -> String {
    if pee.trim().is_empty() {
        return String::new();
    }

    let normalized = pee.replace("\r\n", "\n").replace('\r', "\n");
    let re_double_nl = Regex::new(r"\n\s*\n+").expect("valid paragraph split regex");

    let block_prefixes = [
        "<div",
        "</div",
        "<section",
        "</section",
        "<pre",
        "</pre",
        "<script",
        "</script",
        "<style",
        "</style",
        "<table",
        "</table",
        "<ul",
        "</ul",
        "<ol",
        "</ol",
        "<blockquote",
        "</blockquote",
        "<h1",
        "<h2",
        "<h3",
        "<h4",
        "<h5",
        "<h6",
        "<p>",
        "<p ",
        "<header",
        "<footer",
        "<nav",
        "<article",
        "<aside",
        "<figure",
        "<form",
        "<hr",
        "<!--",
    ];

    let mut paragraphs = Vec::new();
    for chunk in re_double_nl.split(normalized.trim()) {
        let trimmed = chunk.trim();
        if trimmed.is_empty() {
            continue;
        }

        let lower = trimmed.to_lowercase();
        let is_block = block_prefixes.iter().any(|p| lower.starts_with(p));

        if is_block {
            paragraphs.push(trimmed.to_string());
        } else if br {
            let with_br = trimmed.replace('\n', "<br />\n");
            paragraphs.push(format!("<p>{}</p>", with_br));
        } else {
            paragraphs.push(format!("<p>{}</p>", trimmed));
        }
    }

    paragraphs.join("\n")
}
