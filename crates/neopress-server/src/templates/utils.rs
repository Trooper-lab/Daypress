pub fn clean_wp_content(raw: &str) -> String {
    let Ok(re_blocks) = regex::Regex::new(r"<!--\s*/?wp:[^>]*-->") else {
        return raw.trim().to_string();
    };
    let mut s = re_blocks.replace_all(raw, "").into_owned();

    // Expand [caption ...]...[/caption]
    if let Ok(re_caption) = regex::Regex::new(r"(?is)\[caption[^\]]*\](.*?)\[/caption\]") {
        s = re_caption
            .replace_all(&s, r#"<figure class="wp-caption">$1</figure>"#)
            .into_owned();
    }

    // Expand [neopress_form ...] and [contact-form-7 ...] left over after plugin expansion,
    // posting to the form's real submit route (`id="N"` when given, else form 1).
    if let Ok(re_form) = regex::Regex::new(r#"(?i)\[(?:neopress_form|contact-form-7)([^\]]*)\]"#) {
        let form_html = r#"<form class="np-inline-form" onsubmit="event.preventDefault();var f=this;fetch('/wp-json/neopress/v1/forms/__FORM_ID__/submit',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({data:{name:f.querySelector('[name=name]').value,email:f.querySelector('[name=email]').value,message:f.querySelector('[name=message]').value}})}).then(function(r){if(!r.ok)throw r;f.innerHTML='<p style=&quot;color:#10b981;font-weight:600;margin:0;&quot;>✓ Thank you! Your message has been received.</p>';}).catch(function(){alert('Sorry, your message could not be sent. Please try again.');});" style="display:flex;flex-direction:column;gap:12px;max-width:520px;margin:24px 0;padding:20px;background:#111114;border:1px solid #222226;border-radius:8px;"><input type="text" name="name" placeholder="Your Name" required style="padding:10px 12px;border-radius:6px;border:1px solid #27272a;background:#09090b;color:#fff;" /><input type="email" name="email" placeholder="Email Address" required style="padding:10px 12px;border-radius:6px;border:1px solid #27272a;background:#09090b;color:#fff;" /><textarea name="message" rows="4" placeholder="Your Message..." required style="padding:10px 12px;border-radius:6px;border:1px solid #27272a;background:#09090b;color:#fff;"></textarea><button type="submit" class="btn-primary-neon" style="align-self:flex-start;border:none;cursor:pointer;">Send Message</button></form>"#;
        let re_id = regex::Regex::new(r#"id=["']?(\d+)"#).ok();
        s = re_form
            .replace_all(&s, |caps: &regex::Captures| {
                let form_id = re_id
                    .as_ref()
                    .and_then(|re| re.captures(caps.get(1).map_or("", |m| m.as_str())))
                    .and_then(|c| c.get(1))
                    .map_or("1", |m| m.as_str())
                    .to_string();
                form_html.replace("__FORM_ID__", &form_id)
            })
            .into_owned();
    }

    // Expand [audio src="..."]
    if let Ok(re_audio) =
        regex::Regex::new(r#"(?i)\[audio[^\]]*(?:src|mp3)=["']([^"']+)["'][^\]]*\](?:\[/audio\])?"#)
    {
        s = re_audio
            .replace_all(
                &s,
                r#"<audio controls src="$1" style="width:100%;margin:16px 0;"></audio>"#,
            )
            .into_owned();
    }

    // Expand [video src="..."]
    if let Ok(re_video) =
        regex::Regex::new(r#"(?i)\[video[^\]]*(?:src|mp4)=["']([^"']+)["'][^\]]*\](?:\[/video\])?"#)
    {
        s = re_video
            .replace_all(
                &s,
                r#"<video controls src="$1" style="max-width:100%;border-radius:8px;margin:16px 0;"></video>"#,
            )
            .into_owned();
    }

    // Expand [gallery ...]
    if let Ok(re_gallery) = regex::Regex::new(r#"(?i)\[gallery[^\]]*\]"#) {
        s = re_gallery
            .replace_all(&s, r#"<div class="np-gallery"></div>"#)
            .into_owned();
    }

    // Strip any remaining legacy page-builder/plugin wrapper shortcodes
    if let Ok(re_unknown) = regex::Regex::new(
        r#"(?i)\[/?(?:vc_|et_pb_|rev_slider|wpforms|gravityform|embed)[a-z0-9_-]*[^\]]*\]"#,
    ) {
        s = re_unknown.replace_all(&s, "").into_owned();
    }

    s.trim().to_string()
}

pub fn escape_html(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&#038;", "&")
        .replace("&#8211;", "–")
        .replace("&#8212;", "—")
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn escape_json(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            // Keep `</script>` and HTML comments from closing an inline <script> block (JSON-LD).
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
            c if c.is_control() => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

/// Inline tags a visitor's comment may keep; they are restored only in their exact,
/// attribute-free form after the whole comment has been escaped.
const COMMENT_ALLOWED_TAGS: [&str; 7] = ["p", "br", "b", "strong", "i", "em", "code"];

/// Renders untrusted comment text as safe HTML: everything is escaped, a few attribute-free
/// inline tags are restored, and line breaks become `<br>`.
pub fn sanitize_comment_html(raw: &str) -> String {
    let mut out = escape_html(raw.trim());
    for tag in COMMENT_ALLOWED_TAGS {
        for (escaped, plain) in [
            (format!("&lt;{tag}&gt;"), format!("<{tag}>")),
            (format!("&lt;/{tag}&gt;"), format!("</{tag}>")),
            (format!("&lt;{tag} /&gt;"), format!("<{tag} />")),
            (format!("&lt;{tag}/&gt;"), format!("<{tag} />")),
        ] {
            out = out.replace(&escaped, &plain);
        }
    }
    out.replace("\r\n", "\n").replace('\n', "<br>\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_comment_html_neutralises_markup() {
        let out = sanitize_comment_html(
            "Hi <b>there</b>\n<img src=x onerror=alert(1)><script>alert(2)</script><a href=\"javascript:x\">l</a>",
        );
        assert!(out.starts_with("Hi <b>there</b><br>"));
        assert!(!out.contains("<img") && !out.contains("<script") && !out.contains("<a "));
        assert!(out.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_escape_json_cannot_close_script_block() {
        let out = escape_json("</script><img src=x onerror=alert(1)>");
        assert!(!out.contains('<') && !out.contains('>'));
    }

    #[test]
    fn test_escape_html() {
        assert_eq!(
            escape_html("<script>alert('xss');</script>"),
            "&lt;script&gt;alert(&#39;xss&#39;);&lt;/script&gt;"
        );
        assert_eq!(
            escape_html("Tom & Jerry \"fun\""),
            "Tom &amp; Jerry &quot;fun&quot;"
        );
    }

    #[test]
    fn test_clean_wp_content() {
        let raw = "<!-- wp:paragraph -->\n<p>Hello world</p>\n<!-- /wp:paragraph -->";
        assert_eq!(clean_wp_content(raw), "<p>Hello world</p>");
    }

    #[test]
    fn test_shortcodes_in_clean_wp_content() {
        let raw =
            r#"[caption id="1"]<img src="a.jpg" /> Cute cat[/caption] [audio src="song.mp3"]"#;
        let cleaned = clean_wp_content(raw);
        assert!(
            cleaned.contains(r#"<figure class="wp-caption"><img src="a.jpg" /> Cute cat</figure>"#)
        );
        assert!(cleaned.contains(r#"<audio controls src="song.mp3""#));
    }
}
