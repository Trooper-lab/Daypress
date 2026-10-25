use crate::elementor::normalize_video_embed_url;
use crate::gutenberg::parser::GutenbergBlock;
use crate::gutenberg::styles::{extract_attr_from_html, strip_html_tags};
use crate::schema::NodeKind;

pub(crate) fn map_gutenberg_embed(block: &GutenbergBlock) -> NodeKind {
    let provider = block
        .attrs
        .get("providerNameSlug")
        .and_then(|v| v.as_str())
        .or_else(|| {
            block
                .name
                .strip_prefix("core-embed/")
                .or_else(|| block.name.strip_prefix("core/"))
        })
        .unwrap_or("");

    let raw_url = block
        .attrs
        .get("url")
        .or_else(|| block.attrs.get("src"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| extract_attr_from_html(&block.inner_html, "src"))
        .or_else(|| extract_attr_from_html(&block.inner_html, "href"))
        .unwrap_or_else(|| strip_html_tags(&block.inner_html));

    let clean_url = raw_url.trim();

    match provider {
        "youtube" | "vimeo" | "video" => {
            let url = normalize_video_embed_url(clean_url);
            NodeKind::Video {
                url,
                aspect_ratio: "16/9".to_string(),
                autoplay: false,
                muted: false,
                loop_video: false,
            }
        }
        "spotify" => {
            let embed_url = if clean_url.contains("/embed/") {
                clean_url.to_string()
            } else {
                clean_url.replace("open.spotify.com/", "open.spotify.com/embed/")
            };
            NodeKind::CustomHtml {
                raw: format!(
                    r#"<div class="wp-block-embed wp-block-embed-spotify is-type-rich is-provider-spotify wp-has-aspect-ratio"><iframe style="border-radius:12px" src="{}" width="100%" height="352" frameBorder="0" allowfullscreen="" allow="autoplay; clipboard-write; encrypted-media; fullscreen; picture-in-picture" loading="lazy"></iframe></div>"#,
                    embed_url
                ),
            }
        }
        "twitter" => NodeKind::CustomHtml {
            raw: format!(
                r#"<div class="wp-block-embed wp-block-embed-twitter is-type-rich is-provider-twitter"><blockquote class="twitter-tweet"><a href="{}"></a></blockquote><script async src="https://platform.twitter.com/widgets.js" charset="utf-8"></script></div>"#,
                clean_url
            ),
        },
        _ => {
            if clean_url.contains("youtube.com")
                || clean_url.contains("youtu.be")
                || clean_url.contains("vimeo.com")
            {
                let url = normalize_video_embed_url(clean_url);
                NodeKind::Video {
                    url,
                    aspect_ratio: "16/9".to_string(),
                    autoplay: false,
                    muted: false,
                    loop_video: false,
                }
            } else if !block.inner_html.trim().is_empty() {
                NodeKind::CustomHtml {
                    raw: block.inner_html.trim().to_string(),
                }
            } else {
                NodeKind::CustomHtml {
                    raw: format!(
                        r#"<div class="wp-block-embed wp-has-aspect-ratio"><iframe src="{}" loading="lazy"></iframe></div>"#,
                        clean_url
                    ),
                }
            }
        }
    }
}
