//! SEO Meta, OpenGraph, Canonical, and Rich Schema.org JSON-LD Head Component.

use crate::adapters::database::search_fts::{estimate_reading_time_minutes, strip_html_for_fts};
use crate::models::{DaypressPostResponse, DaypressSettingsResponse};
use crate::templates::utils::{clean_post_content, escape_html};
use serde_json::{json, Value};

/// Strips HTML tags cleanly and truncates on UTF-8 character boundaries.
pub fn plain_text_excerpt(html: &str, max_chars: usize) -> String {
    let cleaned = clean_post_content(html);
    let mut out = String::with_capacity(cleaned.len());
    let mut in_tag = false;
    for ch in cleaned.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => {
                in_tag = false;
                out.push(' ');
            }
            _ if !in_tag => out.push(ch),
            _ => {}
        }
    }
    let normalized = out.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.chars().count() > max_chars {
        let prefix: String = normalized.chars().take(max_chars).collect();
        format!("{}...", escape_html(&prefix))
    } else {
        escape_html(&normalized)
    }
}

pub struct HeadMetaOutput {
    pub page_title: String,
    pub meta_desc: String,
    pub robots_meta: &'static str,
    pub canonical_tag: String,
    pub og_image_tags: String,
    pub schema_type: String,
    pub schema_json_ld: String,
    /// RSS autodiscovery `<link rel="alternate">` for `/feed/`.
    pub feed_links: String,
}

/// Extracts Question & Answer pairs from HTML (`<details>/<summary>`, `.dp-accordion-item`, or `<h3>?` + `<p>`)
/// and from structured `post.acf` / `post.meta` fields for `FAQPage` Schema.org JSON-LD.
pub fn extract_faq_entities(post: &DaypressPostResponse) -> Vec<Value> {
    let mut entities = Vec::new();

    // 1. Check structured ACF / meta `faq` or `faqs` array
    for source in [&post.acf, &post.meta] {
        if let Some(obj) = source.as_object() {
            for key in ["faq", "faqs", "faq_items", "questions"] {
                if let Some(Value::Array(arr)) = obj.get(key) {
                    for item in arr {
                        if let Some(item_obj) = item.as_object() {
                            let q = item_obj
                                .get("question")
                                .or_else(|| item_obj.get("title"))
                                .or_else(|| item_obj.get("q"))
                                .or_else(|| item_obj.get("label"))
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .trim();
                            let a = item_obj
                                .get("answer")
                                .or_else(|| item_obj.get("content"))
                                .or_else(|| item_obj.get("a"))
                                .or_else(|| item_obj.get("value"))
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .trim();
                            if !q.is_empty() && !a.is_empty() {
                                entities.push(json!({
                                    "@type": "Question",
                                    "name": strip_html_for_fts(q),
                                    "acceptedAnswer": {
                                        "@type": "Answer",
                                        "text": strip_html_for_fts(a)
                                    }
                                }));
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Scan `<details>` / `<summary>` blocks in post HTML
    let html = &post.content.rendered;
    let lower = html.to_ascii_lowercase();
    let mut cursor = 0;
    while let Some(rel_det) = lower[cursor..].find("<details") {
        let det_start = cursor + rel_det;
        let Some(rel_end) = lower[det_start..].find("</details>") else {
            break;
        };
        let det_end = det_start + rel_end;
        let block = &html[det_start..det_end];
        let block_lower = &lower[det_start..det_end];
        if let (Some(s_open), Some(s_close)) =
            (block_lower.find("<summary"), block_lower.find("</summary>"))
        {
            if s_close > s_open {
                let sum_inner = &block[s_open..s_close];
                let q = strip_html_for_fts(sum_inner);
                let ans_raw = &block[(s_close + 10).min(block.len())..];
                let a = strip_html_for_fts(ans_raw);
                if !q.is_empty() && !a.is_empty() {
                    entities.push(json!({
                        "@type": "Question",
                        "name": q,
                        "acceptedAnswer": {
                            "@type": "Answer",
                            "text": a
                        }
                    }));
                }
            }
        }
        cursor = det_end + 10;
    }

    let meta_schema_override = post
        .meta
        .as_object()
        .and_then(|m| {
            m.get("seo_schema_type")
                .or_else(|| m.get("_daypress_seo_schema_type"))
        })
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let is_faq_schema = post.seo_schema_type.eq_ignore_ascii_case("FAQPage")
        || meta_schema_override.eq_ignore_ascii_case("FAQPage");

    // 3. If schema_type is explicitly FAQPage and no <details> were found, scan question headings (`?`) followed by paragraphs
    if entities.is_empty() && is_faq_schema {
        let mut h_cursor = 0;
        while let Some(rel_h) = lower[h_cursor..]
            .find("<h2")
            .or_else(|| lower[h_cursor..].find("<h3"))
            .or_else(|| lower[h_cursor..].find("<h4"))
        {
            let h_start = h_cursor + rel_h;
            let Some(h_close_rel) = lower[h_start..]
                .find("</h2>")
                .or_else(|| lower[h_start..].find("</h3>"))
                .or_else(|| lower[h_start..].find("</h4>"))
            else {
                break;
            };
            let h_end = h_start + h_close_rel;
            let q = strip_html_for_fts(&html[h_start..h_end]);
            let after_h = (h_end + 5).min(html.len());
            if let (Some(p_open), Some(p_close)) =
                (lower[after_h..].find("<p"), lower[after_h..].find("</p>"))
            {
                if p_close > p_open {
                    let a = strip_html_for_fts(&html[after_h + p_open..after_h + p_close]);
                    if !q.is_empty() && !a.is_empty() {
                        entities.push(json!({
                            "@type": "Question",
                            "name": q,
                            "acceptedAnswer": {
                                "@type": "Answer",
                                "text": a
                            }
                        }));
                    }
                }
            }
            h_cursor = after_h;
        }
    }

    if entities.is_empty() && is_faq_schema {
        let q = strip_html_for_fts(&post.title.rendered);
        let a = strip_html_for_fts(&post.content.rendered);
        if !q.is_empty() {
            entities.push(json!({
                "@type": "Question",
                "name": q,
                "acceptedAnswer": {
                    "@type": "Answer",
                    "text": if a.is_empty() { q } else { a }
                }
            }));
        }
    }

    entities
}

#[allow(clippy::too_many_arguments)]
fn build_rich_schema_json_ld(
    schema_type: &str,
    title: &str,
    meta_desc: &str,
    canonical_url: &str,
    og_image_url: &str,
    site_name: &str,
    base_url: &str,
    current_post: Option<&DaypressPostResponse>,
) -> String {
    let home_url = format!("{}/", base_url.trim_end_matches('/'));
    let mut breadcrumb_items = vec![json!({
        "@type": "ListItem",
        "position": 1,
        "name": site_name,
        "item": home_url
    })];

    if let Some(post) = current_post {
        let is_front = canonical_url == home_url;
        if !is_front {
            let mut pos = 2;
            if post.post_type != "post" && post.post_type != "page" && !post.post_type.is_empty() {
                let mut chars = post.post_type.chars();
                let cpt_label = match chars.next() {
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    None => post.post_type.clone(),
                };
                breadcrumb_items.push(json!({
                    "@type": "ListItem",
                    "position": pos,
                    "name": cpt_label,
                    "item": format!("{}/{}/", base_url.trim_end_matches('/'), post.post_type)
                }));
                pos += 1;
            } else if post.post_type == "post" {
                if let Some(first_cat) = post
                    .category_names
                    .first()
                    .filter(|c| !c.eq_ignore_ascii_case("Uncategorized"))
                {
                    let cat_slug = generated_slugify::slugify(first_cat);
                    breadcrumb_items.push(json!({
                        "@type": "ListItem",
                        "position": pos,
                        "name": first_cat,
                        "item": format!("{}/category/{}/", base_url.trim_end_matches('/'), cat_slug)
                    }));
                    pos += 1;
                }
            }
            breadcrumb_items.push(json!({
                "@type": "ListItem",
                "position": pos,
                "name": strip_html_for_fts(&post.title.rendered),
                "item": canonical_url
            }));
        }
    }

    let breadcrumb_node = json!({
        "@type": "BreadcrumbList",
        "itemListElement": breadcrumb_items
    });

    let website_node = json!({
        "@type": "WebSite",
        "name": site_name,
        "url": home_url,
        "potentialAction": {
            "@type": "SearchAction",
            "target": format!("{}/?s={{search_term_string}}", base_url.trim_end_matches('/')),
            "query-input": "required name=search_term_string"
        }
    });

    let mut root = serde_json::Map::new();
    root.insert("@context".to_string(), json!("https://schema.org"));
    root.insert("@type".to_string(), json!(schema_type));
    root.insert("name".to_string(), json!(site_name));
    root.insert("headline".to_string(), json!(strip_html_for_fts(title)));
    root.insert("description".to_string(), json!(meta_desc));
    if !canonical_url.is_empty() {
        root.insert("url".to_string(), json!(canonical_url));
        root.insert(
            "mainEntityOfPage".to_string(),
            json!({ "@type": "WebPage", "@id": canonical_url }),
        );
    }
    if !og_image_url.is_empty() {
        root.insert("image".to_string(), json!(og_image_url));
    }
    root.insert(
        "publisher".to_string(),
        json!({
            "@type": "Organization",
            "name": site_name,
            "url": home_url
        }),
    );

    let mut graph = vec![website_node, breadcrumb_node.clone()];

    if let Some(post) = current_post {
        let plain_body = strip_html_for_fts(&post.content.rendered);
        let (word_count, reading_mins) = estimate_reading_time_minutes(&plain_body);
        let author_name = if post.author_name.trim().is_empty() {
            site_name
        } else {
            post.author_name.trim()
        };
        let author_json = if post.coauthors.len() > 1 {
            let persons: Vec<Value> = post
                .coauthors
                .iter()
                .map(|ca| {
                    let mut obj = serde_json::Map::new();
                    obj.insert("@type".to_string(), json!("Person"));
                    obj.insert("name".to_string(), json!(ca.display_name));
                    if !ca.slug.trim().is_empty() {
                        obj.insert(
                            "url".to_string(),
                            json!(format!(
                                "{}/author/{}/",
                                base_url.trim_end_matches('/'),
                                ca.slug.trim()
                            )),
                        );
                    }
                    Value::Object(obj)
                })
                .collect();
            Value::Array(persons)
        } else {
            json!({
                "@type": "Person",
                "name": author_name
            })
        };
        root.insert("author".to_string(), author_json.clone());
        if !post.lang_code.trim().is_empty() {
            root.insert("inLanguage".to_string(), json!(post.lang_code.trim()));
        }
        if !post.date.trim().is_empty() {
            root.insert("datePublished".to_string(), json!(post.date));
        }
        if !post.modified.trim().is_empty() {
            root.insert("dateModified".to_string(), json!(post.modified));
        }
        root.insert("wordCount".to_string(), json!(word_count));
        root.insert(
            "timeRequired".to_string(),
            json!(format!("PT{}M", reading_mins)),
        );

        let faq_entities = extract_faq_entities(post);
        if !faq_entities.is_empty() {
            root.insert("mainEntity".to_string(), json!(faq_entities));
            graph.push(json!({
                "@type": "FAQPage",
                "name": strip_html_for_fts(&post.title.rendered),
                "mainEntity": faq_entities
            }));
        }

        if schema_type.eq_ignore_ascii_case("LocalBusiness") {
            let meta_lookup = |k: &str, default: &str| -> String {
                post.acf
                    .get(k)
                    .or_else(|| post.meta.get(k))
                    .and_then(Value::as_str)
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or(default)
                    .to_string()
            };
            let telephone = meta_lookup("telephone", &meta_lookup("phone", ""));
            let price_range = meta_lookup("price_range", "$$");
            let street = meta_lookup("street_address", &meta_lookup("address", ""));
            let locality = meta_lookup("city", &meta_lookup("locality", ""));
            let country = meta_lookup("country", "");
            if !telephone.is_empty() {
                root.insert("telephone".to_string(), json!(telephone));
            }
            root.insert("priceRange".to_string(), json!(price_range));
            let addr = json!({
                "@type": "PostalAddress",
                "streetAddress": street,
                "addressLocality": locality,
                "addressCountry": country
            });
            root.insert("address".to_string(), addr.clone());
            graph.push(json!({
                "@type": "LocalBusiness",
                "name": strip_html_for_fts(&post.title.rendered),
                "description": meta_desc,
                "url": canonical_url,
                "image": og_image_url,
                "priceRange": price_range,
                "address": addr
            }));
        } else if schema_type.eq_ignore_ascii_case("Product") {
            let meta_lookup = |k: &str, default: &str| -> String {
                post.acf
                    .get(k)
                    .or_else(|| post.meta.get(k))
                    .and_then(|v| match v {
                        Value::String(s) => Some(s.clone()),
                        Value::Number(n) => Some(n.to_string()),
                        _ => None,
                    })
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| default.to_string())
            };
            let price = meta_lookup("price", "0.00");
            let currency = meta_lookup("currency", &meta_lookup("price_currency", "USD"));
            let sku = meta_lookup("sku", &post.slug);
            let offers = json!({
                "@type": "Offer",
                "url": canonical_url,
                "priceCurrency": currency,
                "price": price,
                "availability": "https://schema.org/InStock"
            });
            root.insert("sku".to_string(), json!(sku));
            root.insert(
                "brand".to_string(),
                json!({ "@type": "Brand", "name": site_name }),
            );
            root.insert("offers".to_string(), offers.clone());
            graph.push(json!({
                "@type": "Product",
                "name": strip_html_for_fts(&post.title.rendered),
                "description": meta_desc,
                "image": og_image_url,
                "sku": sku,
                "brand": { "@type": "Brand", "name": site_name },
                "offers": offers
            }));
        } else if matches!(
            schema_type,
            "Article" | "BlogPosting" | "NewsArticle" | "TechArticle"
        ) {
            graph.push(json!({
                "@type": schema_type,
                "headline": strip_html_for_fts(&post.title.rendered),
                "description": meta_desc,
                "image": og_image_url,
                "datePublished": post.date,
                "dateModified": post.modified,
                "wordCount": word_count,
                "timeRequired": format!("PT{}M", reading_mins),
                "author": author_json,
                "publisher": {
                    "@type": "Organization",
                    "name": site_name
                },
                "mainEntityOfPage": {
                    "@type": "WebPage",
                    "@id": canonical_url
                }
            }));
        }
    }

    root.insert("breadcrumb".to_string(), breadcrumb_node);
    root.insert("@graph".to_string(), json!(graph));

    serde_json::to_string_pretty(&Value::Object(root)).unwrap_or_else(|_| "{}".to_string())
}

pub fn render_head_meta(
    title: &str,
    settings: &DaypressSettingsResponse,
    current_post: Option<&DaypressPostResponse>,
    site_name: &str,
    site_desc: &str,
) -> HeadMetaOutput {
    let base_url = settings.url.trim_end_matches('/');
    let is_front_page = current_post.is_some_and(|p| {
        settings.show_on_front == "page"
            && settings.page_on_front > 0
            && p.id == settings.page_on_front
    });

    let page_title = if let Some(post) = current_post {
        if !post.seo_title.trim().is_empty() {
            post.seo_title.trim().to_string()
        } else if is_front_page && title.trim().eq_ignore_ascii_case("home") {
            if !site_desc.trim().is_empty() {
                format!("{} – {}", site_name, site_desc)
            } else {
                site_name.to_string()
            }
        } else {
            format!("{} – {}", title, site_name)
        }
    } else if title == site_name {
        if !settings.description.trim().is_empty() {
            format!("{} – {}", site_name, site_desc)
        } else {
            site_name.to_string()
        }
    } else {
        format!("{} – {}", title, site_name)
    };

    let meta_desc = if let Some(post) = current_post {
        if !post.seo_description.trim().is_empty() {
            post.seo_description.trim().to_string()
        } else {
            let ex = plain_text_excerpt(&post.content.rendered, 155);
            if !ex.is_empty() {
                ex
            } else {
                site_desc.to_string()
            }
        }
    } else {
        site_desc.to_string()
    };

    let robots_meta =
        if !settings.blog_public || current_post.map(|p| p.seo_noindex).unwrap_or(false) {
            r#"<meta name="robots" content="noindex, follow" />"#
        } else {
            r#"<meta name="robots" content="index, follow" />"#
        };

    let can_url = if let Some(post) = current_post {
        if !post.seo_canonical.trim().is_empty() {
            post.seo_canonical.trim().to_string()
        } else if is_front_page {
            format!("{}/", base_url)
        } else if !post.link.trim().is_empty() {
            post.link.clone()
        } else {
            format!("{}/{}/", base_url, post.slug.trim_matches('/'))
        }
    } else {
        format!("{}/", base_url)
    };

    let canonical_tag = if !can_url.is_empty() {
        format!(
            r#"<link rel="canonical" href="{}" />"#,
            escape_html(&can_url)
        )
    } else {
        String::new()
    };

    // Resolve explicit OpenGraph image or fallback to automatic Rust-rendered 1200x630 PNG card
    let og_image_url = if let Some(post) = current_post {
        if !post.seo_og_image.trim().is_empty() {
            post.seo_og_image.trim().to_string()
        } else if let Some(feat) = post
            .featured_image_url
            .as_deref()
            .filter(|u| !u.trim().is_empty())
        {
            feat.trim().to_string()
        } else if is_front_page || post.id <= 0 {
            format!("{}/og-image/home.png", base_url)
        } else {
            format!("{}/og-image/{}.png", base_url, post.id)
        }
    } else {
        format!("{}/og-image/home.png", base_url)
    };

    let og_image_tags = format!(
        r#"<meta property="og:image" content="{0}" /><meta property="og:image:width" content="1200" /><meta property="og:image:height" content="630" /><meta name="twitter:image" content="{0}" />"#,
        escape_html(&og_image_url)
    );

    let schema_type = if let Some(post) = current_post {
        let meta_schema = post
            .meta
            .as_object()
            .and_then(|m| {
                m.get("seo_schema_type")
                    .or_else(|| m.get("_daypress_seo_schema_type"))
            })
            .and_then(|v| v.as_str())
            .unwrap_or("");
        if !post.seo_schema_type.trim().is_empty() {
            post.seo_schema_type.trim().to_string()
        } else if !meta_schema.trim().is_empty() {
            meta_schema.trim().to_string()
        } else if post.post_type == "post" {
            "BlogPosting".to_string()
        } else {
            "WebPage".to_string()
        }
    } else {
        "WebSite".to_string()
    };

    let schema_json_ld = build_rich_schema_json_ld(
        &schema_type,
        title,
        &meta_desc,
        &can_url,
        &og_image_url,
        site_name,
        base_url,
        current_post,
    );

    // Feed autodiscovery, like WordPress `feed_links()`.
    let mut feed_links = format!(
        r#"<link rel="alternate" type="application/rss+xml" title="{0} &raquo; Feed" href="/feed/" /><link rel="alternate" type="application/atom+xml" title="{0} &raquo; Atom Feed" href="/feed/atom/" /><link rel="alternate" type="application/rss+xml" title="{0} &raquo; Comments Feed" href="/comments/feed/" />"#,
        escape_html(site_name)
    );
    if let Some(post) = current_post {
        if post.status == "publish" && post.comment_status == "open" && !post.slug.trim().is_empty()
        {
            feed_links.push_str(&format!(
                r#"<link rel="alternate" type="application/rss+xml" title="{} &raquo; {} Comments Feed" href="/{}/feed/" />"#,
                escape_html(site_name),
                escape_html(&plain_text_excerpt(&post.title.rendered, 80)),
                escape_html(post.slug.trim_matches('/'))
            ));
        }
    }

    HeadMetaOutput {
        page_title,
        meta_desc,
        robots_meta,
        canonical_tag,
        og_image_tags,
        schema_type,
        schema_json_ld,
        feed_links,
    }
}
