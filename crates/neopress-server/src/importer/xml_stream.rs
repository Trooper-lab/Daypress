//! quick-xml streaming event loop and token extraction for WXR imports.

use crate::errors::AppError;
use crate::importer::models::{ParsedWxrComment, ParsedWxrItem, ParsedWxrTerm};
use crate::models::ImportSiteSettings;
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use std::io::{BufRead, Cursor};

#[allow(clippy::too_many_arguments)]
fn process_xml_text(
    text: &str,
    tag_stack: &[String],
    site_settings: &mut ImportSiteSettings,
    source_site_url: &mut Option<(String, u8)>,
    site_language: &mut Option<String>,
    site_favicon_url: &mut Option<String>,
    current_term: &mut Option<ParsedWxrTerm>,
    current_item: &mut Option<ParsedWxrItem>,
    current_comment: &mut Option<ParsedWxrComment>,
    current_meta_key: &mut String,
    current_meta_value: &mut String,
    current_category_domain: &str,
    current_category_nicename: &str,
) {
    if text.is_empty() {
        return;
    }

    let current_tag = tag_stack.last().map(|s| s.as_str()).unwrap_or("");
    let parent_tag = if tag_stack.len() >= 2 {
        tag_stack[tag_stack.len() - 2].as_str()
    } else {
        ""
    };

    if parent_tag == "channel" {
        match current_tag {
            "title" => site_settings.title = Some(text.to_string()),
            "description" => site_settings.description = Some(text.to_string()),
            "wp:base_blog_url" => {
                if text.starts_with("http://") || text.starts_with("https://") {
                    *source_site_url = Some((text.trim_end_matches('/').to_string(), 3));
                }
            }
            "link" => {
                let current_prio = source_site_url.as_ref().map(|(_, p)| *p).unwrap_or(0);
                if current_prio < 2 && (text.starts_with("http://") || text.starts_with("https://"))
                {
                    *source_site_url = Some((text.trim_end_matches('/').to_string(), 2));
                }
            }
            "wp:base_site_url" => {
                let current_prio = source_site_url.as_ref().map(|(_, p)| *p).unwrap_or(0);
                if current_prio < 1 && (text.starts_with("http://") || text.starts_with("https://"))
                {
                    *source_site_url = Some((text.trim_end_matches('/').to_string(), 1));
                }
            }
            "language" => {
                *site_language = Some(text.to_string());
            }
            _ => (),
        }
    } else if parent_tag == "image" && current_tag == "url" {
        *site_favicon_url = Some(text.to_string());
    }

    if let Some(term) = current_term.as_mut() {
        match current_tag {
            "wp:cat_name" | "wp:tag_name" | "wp:term_name" => {
                term.name = text.to_string();
            }
            "wp:category_nicename" | "wp:tag_slug" | "wp:term_slug" => {
                term.slug = text.to_string();
            }
            "wp:category_description" | "wp:term_description" => {
                term.description = text.to_string();
            }
            "wp:term_taxonomy" => {
                term.taxonomy = text.to_string();
            }
            _ => (),
        }
    }

    if let Some(item) = current_item.as_mut() {
        if parent_tag == "wp:comment" || tag_stack.iter().any(|s| s == "wp:comment") {
            if let Some(comm) = current_comment.as_mut() {
                match current_tag {
                    "wp:comment_id" => comm.id = text.parse().ok(),
                    "wp:comment_author" => comm.author = text.to_string(),
                    "wp:comment_author_email" => comm.email = text.to_string(),
                    "wp:comment_author_url" => comm.url = text.to_string(),
                    "wp:comment_date" => comm.date = text.to_string(),
                    "wp:comment_content" => comm.content.push_str(text),
                    "wp:comment_approved" => comm.approved = text.to_string(),
                    "wp:comment_type" => comm.comment_type = text.to_string(),
                    "wp:comment_parent" => comm.parent = text.parse().unwrap_or(0),
                    _ => (),
                }
            }
        } else if parent_tag == "wp:postmeta" || tag_stack.iter().any(|s| s == "wp:postmeta") {
            match current_tag {
                "wp:meta_key" => *current_meta_key = text.to_string(),
                "wp:meta_value" => current_meta_value.push_str(text),
                _ => (),
            }
        } else {
            match current_tag {
                "title" => item.title = text.to_string(),
                "wp:post_name" => item.slug = text.to_string(),
                "content:encoded" => item.content.push_str(text),
                "excerpt:encoded" => item.excerpt.push_str(text),
                "wp:post_date" | "wp:post_date_gmt" => {
                    if item.post_date.is_empty() {
                        item.post_date = text.to_string();
                    }
                }
                "wp:status" => item.post_status = text.to_string(),
                "wp:post_type" => item.post_type = text.to_string(),
                "wp:post_mime_type" => item.post_mime_type = text.to_string(),
                "wp:post_id" => item.id = text.parse().ok(),
                "wp:attachment_url" => item.attachment_url = text.to_string(),
                "guid" => item.guid = text.to_string(),
                "wp:post_parent" => item.parent = text.parse().unwrap_or(0),
                "wp:menu_order" => item.menu_order = text.parse().unwrap_or(0),
                "wp:is_sticky" => item.is_sticky = text.trim() == "1",
                "category" => {
                    if current_category_domain == "category" || current_category_domain.is_empty() {
                        let slug = if !current_category_nicename.is_empty() {
                            current_category_nicename.to_string()
                        } else {
                            generated_slugify::slugify(text)
                        };
                        item.categories.push(slug);
                    } else if current_category_domain == "post_tag" {
                        let slug = if !current_category_nicename.is_empty() {
                            current_category_nicename.to_string()
                        } else {
                            generated_slugify::slugify(text)
                        };
                        item.tags.push(slug);
                    } else if current_category_domain == "nav_menu" {
                        let slug = if !current_category_nicename.is_empty() {
                            current_category_nicename.to_string()
                        } else {
                            generated_slugify::slugify(text)
                        };
                        item.nav_menu_slug = Some(slug);
                    } else if current_category_domain == "elementor_library_type" {
                        let lib_type = if !current_category_nicename.is_empty() {
                            current_category_nicename.to_string()
                        } else {
                            text.to_string()
                        };
                        item.elementor_library_type = Some(lib_type);
                    } else if !current_category_domain.is_empty() {
                        let slug = if !current_category_nicename.is_empty() {
                            current_category_nicename.to_string()
                        } else {
                            generated_slugify::slugify(text)
                        };
                        item.custom_terms.push((
                            current_category_domain.to_string(),
                            slug,
                            text.to_string(),
                        ));
                    }
                }
                _ => (),
            }
        }
    }
}

pub fn parse_wxr_stream<R: BufRead>(
    mut reader: R,
) -> Result<
    (
        ImportSiteSettings,
        Option<String>,
        Option<String>,
        Option<String>,
        Vec<ParsedWxrTerm>,
        Vec<ParsedWxrItem>,
    ),
    AppError,
> {
    let mut raw_bytes = Vec::new();
    let _ = reader.read_to_end(&mut raw_bytes);
    raw_bytes.retain(|&b| b >= 0x20 || b == b'\t' || b == b'\n' || b == b'\r');
    let mut xml_reader = Reader::from_reader(Cursor::new(raw_bytes));
    xml_reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut tag_stack: Vec<String> = Vec::new();

    let mut site_settings = ImportSiteSettings::default();
    let mut source_site_tracked = Option::<(String, u8)>::None;
    let mut site_language = Option::<String>::None;
    let mut site_favicon_url = Option::<String>::None;
    let mut current_term = Option::<ParsedWxrTerm>::None;
    let mut current_item = Option::<ParsedWxrItem>::None;
    let mut current_comment = Option::<ParsedWxrComment>::None;
    let mut current_meta_key = String::new();
    let mut current_meta_value = String::new();
    let mut current_category_domain = String::new();
    let mut current_category_nicename = String::new();

    let mut terms_to_insert = Vec::new();
    let mut posts_to_insert = Vec::new();

    loop {
        match xml_reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag_name == "item" {
                    current_item = Some(ParsedWxrItem {
                        post_status: "publish".to_string(),
                        post_type: "post".to_string(),
                        ..Default::default()
                    });
                } else if tag_name == "wp:category" || tag_name == "wp:tag" || tag_name == "wp:term"
                {
                    let tax = if tag_name == "wp:tag" {
                        "post_tag".to_string()
                    } else {
                        "category".to_string()
                    };
                    current_term = Some(ParsedWxrTerm {
                        taxonomy: tax,
                        ..Default::default()
                    });
                } else if tag_name == "wp:comment" {
                    current_comment = Some(ParsedWxrComment {
                        approved: "1".to_string(),
                        comment_type: "comment".to_string(),
                        ..Default::default()
                    });
                } else if tag_name == "wp:postmeta" {
                    current_meta_key.clear();
                    current_meta_value.clear();
                } else if tag_name == "category" {
                    current_category_domain.clear();
                    current_category_nicename.clear();
                    for attr in e.attributes().flatten() {
                        let k = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                        let v = String::from_utf8_lossy(&attr.value).to_string();
                        if k == "domain" {
                            current_category_domain = v;
                        } else if k == "nicename" {
                            current_category_nicename = v;
                        }
                    }
                }
                tag_stack.push(tag_name);
            }
            Ok(Event::Text(e)) => {
                let text = match e.unescape() {
                    Ok(cow) => cow.to_string(),
                    Err(_) => String::from_utf8_lossy(e.as_ref()).to_string(),
                };
                process_xml_text(
                    text.trim(),
                    &tag_stack,
                    &mut site_settings,
                    &mut source_site_tracked,
                    &mut site_language,
                    &mut site_favicon_url,
                    &mut current_term,
                    &mut current_item,
                    &mut current_comment,
                    &mut current_meta_key,
                    &mut current_meta_value,
                    &current_category_domain,
                    &current_category_nicename,
                );
            }
            Ok(Event::CData(e)) => {
                let text = String::from_utf8_lossy(e.as_ref()).to_string();
                process_xml_text(
                    text.trim(),
                    &tag_stack,
                    &mut site_settings,
                    &mut source_site_tracked,
                    &mut site_language,
                    &mut site_favicon_url,
                    &mut current_term,
                    &mut current_item,
                    &mut current_comment,
                    &mut current_meta_key,
                    &mut current_meta_value,
                    &current_category_domain,
                    &current_category_nicename,
                );
            }
            Ok(Event::End(e)) => {
                let tag_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                if tag_name == "item" {
                    if let Some(item) = current_item.take() {
                        posts_to_insert.push(item);
                    }
                } else if tag_name == "wp:category" || tag_name == "wp:tag" || tag_name == "wp:term"
                {
                    if let Some(term) = current_term.take() {
                        if !term.name.is_empty() {
                            terms_to_insert.push(term);
                        }
                    }
                } else if tag_name == "wp:comment" {
                    if let (Some(item), Some(comm)) =
                        (current_item.as_mut(), current_comment.take())
                    {
                        item.comments.push(comm);
                    }
                } else if tag_name == "wp:postmeta" {
                    if let Some(item) = current_item.as_mut() {
                        if !current_meta_key.is_empty() {
                            item.postmeta
                                .insert(current_meta_key.clone(), current_meta_value.clone());
                        }
                    }
                }
                tag_stack.pop();
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(AppError::BadRequest(format!(
                    "WXR XML parsing error: {}",
                    e
                )));
            }
            _ => (),
        }
        buf.clear();
    }

    let source_site_url = source_site_tracked.map(|(url, _)| url);

    Ok((
        site_settings,
        source_site_url,
        site_language,
        site_favicon_url,
        terms_to_insert,
        posts_to_insert,
    ))
}
