//! Dynamic Placeholder AST Hydration & SSR Injection Pipeline.

use super::conditions::resolve_template_for_type;
use super::models::{
    BreadcrumbItem, RequestContext, ResolvedThemePartials, TemplateType,
};
use super::repository::ThemeTemplateRepository;
use crate::adapters::database::Database;
use crate::models::{WpPostResponse, WpSettingsResponse};
use neopress_ast::{
    compile_layout_css, render_layout_to_html, sanitize_ssr_html,
    NavMenuItem, NeopressLayout, NeopressNode, NodeKind,
};
use std::collections::HashMap;

pub struct HydrationContext<'a> {
    pub post: Option<&'a WpPostResponse>,
    pub settings: &'a WpSettingsResponse,
    pub breadcrumbs: Vec<BreadcrumbItem>,
    pub nav_menus: HashMap<String, Vec<NavMenuItem>>,
}

pub struct SsrInjector;

impl SsrInjector {
    /// Resolves active Header, Footer, and layout templates, hydrates dynamic placeholders,
    /// and compiles scoped responsive stylesheets.
    pub fn resolve_and_hydrate_partials(
        db: &Database,
        req_ctx: &RequestContext,
        settings: &WpSettingsResponse,
    ) -> ResolvedThemePartials {
        Self::resolve_and_hydrate_partials_with_post(db, req_ctx, settings, None)
    }

    /// Resolves active Header, Footer, and layout templates with an optional active post,
    /// hydrates dynamic placeholders, and compiles scoped responsive stylesheets.
    pub fn resolve_and_hydrate_partials_with_post(
        db: &Database,
        req_ctx: &RequestContext,
        settings: &WpSettingsResponse,
        post: Option<&WpPostResponse>,
    ) -> ResolvedThemePartials {
        let repo = ThemeTemplateRepository::from_database(db);
        let mut partials = ResolvedThemePartials::default();
        let hyd_ctx = Self::create_hydration_context(req_ctx, settings, post);

        // 1. Resolve Header Template
        if let Ok(headers) = repo.find_active_by_type(TemplateType::Header) {
            if !headers.is_empty() {
                partials.has_theme_builder_header = true;
                if let Some(tpl) = resolve_template_for_type(&headers, TemplateType::Header, req_ctx) {
                    let mut layout = tpl.layout.clone();
                    Self::hydrate_layout(&mut layout, &hyd_ctx);
                    let raw_html = render_layout_to_html(&layout);
                    let sanitized_html = sanitize_ssr_html(&raw_html);
                    let css = compile_layout_css(&layout);

                    partials.header_html = Some(sanitized_html);
                    partials.header_css = if css.trim().is_empty() { None } else { Some(css) };
                    partials.header_sticky = tpl.settings.sticky;
                    partials.header_transparent = tpl.settings.transparent;
                }
            }
        }

        // 2. Resolve Footer Template
        if let Ok(footers) = repo.find_active_by_type(TemplateType::Footer) {
            if !footers.is_empty() {
                partials.has_theme_builder_footer = true;
                if let Some(tpl) = resolve_template_for_type(&footers, TemplateType::Footer, req_ctx) {
                    let mut layout = tpl.layout.clone();
                    Self::hydrate_layout(&mut layout, &hyd_ctx);
                    let raw_html = render_layout_to_html(&layout);
                    let sanitized_html = sanitize_ssr_html(&raw_html);
                    let css = compile_layout_css(&layout);

                    partials.footer_html = Some(sanitized_html);
                    partials.footer_css = if css.trim().is_empty() { None } else { Some(css) };
                }
            }
        }

        partials
    }

    /// Recursively traverses the AST layout and hydrates dynamic nodes in-place.
    pub fn hydrate_layout(layout: &mut NeopressLayout, ctx: &HydrationContext) {
        Self::hydrate_node(&mut layout.root, ctx);
    }

    /// Hydrates a single node and recursively processes all child nodes.
    pub fn hydrate_node(node: &mut NeopressNode, ctx: &HydrationContext) {
        // A. Hydrate NavMenu
        if let NodeKind::NavMenu { menu_slug, items, .. } = &mut node.kind {
            if items.is_empty() {
                if let Some(cached) = ctx.nav_menus.get(menu_slug.as_str()) {
                    *items = cached.clone();
                } else if !ctx.settings.nav_menu_items.is_empty() {
                    *items = ctx.settings.nav_menu_items.iter().enumerate().map(|(idx, it)| NavMenuItem {
                        id: it.id,
                        title: it.title.clone(),
                        url: it.url.clone(),
                        target: it.target.clone(),
                        parent: it.parent,
                        order: idx as i64,
                        ..Default::default()
                    }).collect();
                }
            }
        }

        // B. Hydrate Breadcrumbs
        if let NodeKind::Breadcrumbs { separator, home_text } = &node.kind {
            let markup = Self::render_breadcrumbs_html(&ctx.breadcrumbs, separator, home_text);
            node.kind = NodeKind::CustomHtml { raw: markup };
        }

        // C. Dynamic tag detection
        let dynamic_tag: Option<String> = node
            .styles
            .raw_settings
            .as_ref()
            .and_then(|m| m.get("data-dynamic").and_then(|v| v.as_str().map(|s| s.to_string())))
            .or_else(|| {
                let lower_id = node.id.to_lowercase();
                if lower_id.contains("post-title") || lower_id.contains("post_title") {
                    Some("post_title".to_string())
                } else if lower_id.contains("featured-image") || lower_id.contains("featured_image") {
                    Some("featured_image".to_string())
                } else if lower_id.contains("post-content") || lower_id.contains("post_content") {
                    Some("post_content".to_string())
                } else if lower_id.contains("post-date") || lower_id.contains("post_date") {
                    Some("post_date".to_string())
                } else if lower_id.contains("author-box") || lower_id.contains("author_box") {
                    Some("author_box".to_string())
                } else if lower_id.contains("site-logo") || lower_id.contains("site_logo") {
                    Some("site_logo".to_string())
                } else if lower_id.contains("site-title") || lower_id.contains("site_title") {
                    Some("site_title".to_string())
                } else {
                    None
                }
            });

        if let Some(ref tag) = dynamic_tag {
            Self::apply_dynamic_tag(node, tag, ctx);
        }

        // D. General token interpolation on text nodes
        Self::interpolate_text_tokens(node, ctx);

        // Recursively hydrate children
        for child in &mut node.children {
            Self::hydrate_node(child, ctx);
        }
    }

    fn apply_dynamic_tag(node: &mut NeopressNode, tag: &str, ctx: &HydrationContext) {
        match tag {
            "post_title" => {
                let title = ctx.post.map(|p| p.title.rendered.clone()).unwrap_or_else(|| ctx.settings.title.clone());
                match &mut node.kind {
                    NodeKind::Heading { text, .. } => *text = title,
                    NodeKind::Text { content_html } => *content_html = title,
                    NodeKind::CustomHtml { raw } => *raw = title,
                    _ => node.kind = NodeKind::Heading { text: title, level: 1 },
                }
            }
            "featured_image" => {
                let img_url = ctx.post
                    .and_then(|p| p.featured_image_url.as_ref())
                    .filter(|u| !u.is_empty())
                    .cloned()
                    .unwrap_or_else(|| "/static/placeholder.jpg".to_string());
                let alt = ctx.post.map(|p| p.title.rendered.clone()).unwrap_or_default();
                match &mut node.kind {
                    NodeKind::Image { src, alt: a, .. } => {
                        *src = img_url;
                        *a = alt;
                    }
                    _ => {
                        node.kind = NodeKind::Image {
                            src: img_url,
                            alt,
                            width: None,
                            height: None,
                            link: None,
                        };
                    }
                }
            }
            "post_content" => {
                if let Some(post) = ctx.post {
                    node.kind = NodeKind::CustomHtml { raw: post.content.rendered.clone() };
                }
            }
            "post_date" => {
                let date_str = ctx.post
                    .map(|p| p.date.chars().take(10).collect::<String>())
                    .unwrap_or_default();
                match &mut node.kind {
                    NodeKind::Text { content_html } => *content_html = date_str.clone(),
                    NodeKind::Heading { text, .. } => *text = date_str.clone(),
                    _ => node.kind = NodeKind::Text { content_html: date_str },
                }
            }
            "author_box" => {
                let author_name = ctx.post
                    .map(|p| if p.author_name.is_empty() { "admin" } else { &p.author_name })
                    .unwrap_or("admin");
                let escaped_author = html_escape(author_name);
                let card = format!(
                    r#"<div class="np-author-box"><div class="np-author-avatar"><i class="fa-solid fa-user"></i></div><div class="np-author-info"><strong class="np-author-name">{}</strong><span class="np-author-role">Author</span></div></div>"#,
                    escaped_author
                );
                node.kind = NodeKind::CustomHtml { raw: card };
            }
            "site_logo" => {
                let logo_url = if !ctx.settings.site_logo_url.is_empty() {
                    ctx.settings.site_logo_url.clone()
                } else {
                    "/static/favicon.svg".to_string()
                };
                match &mut node.kind {
                    NodeKind::Image { src, .. } => *src = logo_url,
                    _ => {
                        node.kind = NodeKind::Image {
                            src: logo_url,
                            alt: ctx.settings.title.clone(),
                            width: None,
                            height: None,
                            link: Some("/".to_string()),
                        };
                    }
                }
            }
            "site_title" => {
                let title = ctx.settings.title.clone();
                match &mut node.kind {
                    NodeKind::Heading { text, .. } => *text = title,
                    NodeKind::Text { content_html } => *content_html = title,
                    _ => node.kind = NodeKind::Heading { text: title, level: 2 },
                }
            }
            _ => {}
        }
    }

    fn interpolate_text_tokens(node: &mut NeopressNode, ctx: &HydrationContext) {
        match &mut node.kind {
            NodeKind::Heading { text, .. } => {
                *text = Self::interpolate_tokens(text, ctx);
            }
            NodeKind::Text { content_html } => {
                *content_html = Self::interpolate_tokens(content_html, ctx);
            }
            NodeKind::CustomHtml { raw } => {
                *raw = Self::interpolate_tokens(raw, ctx);
            }
            _ => {}
        }
    }

    pub fn interpolate_tokens(text: &str, ctx: &HydrationContext) -> String {
        if !text.contains("{{") {
            return text.to_string();
        }
        let post_title = ctx.post.map(|p| p.title.rendered.as_str()).unwrap_or("");
        let post_date: String = ctx.post.map(|p| p.date.chars().take(10).collect()).unwrap_or_default();
        let post_author = ctx.post.map(|p| if p.author_name.is_empty() { "admin" } else { p.author_name.as_str() }).unwrap_or("admin");

        text.replace("{{post_title}}", post_title)
            .replace("{{post_date}}", &post_date)
            .replace("{{post_author}}", post_author)
            .replace("{{site_title}}", &ctx.settings.title)
            .replace("{{site_description}}", &ctx.settings.description)
            .replace("{{current_year}}", "2026")
    }

    pub fn render_breadcrumbs_html(items: &[BreadcrumbItem], separator: &str, home_text: &str) -> String {
        let mut list_html = String::new();
        let mut all_items = Vec::new();
        all_items.push(BreadcrumbItem {
            title: if home_text.is_empty() { "Home".to_string() } else { home_text.to_string() },
            url: "/".to_string(),
            is_current: items.is_empty(),
        });
        all_items.extend_from_slice(items);

        let sep_escaped = html_escape(if separator.is_empty() { "/" } else { separator });

        for (idx, item) in all_items.iter().enumerate() {
            if idx > 0 {
                list_html.push_str(&format!(
                    r#"<li class="np-breadcrumb-separator" aria-hidden="true">{}</li>"#,
                    sep_escaped
                ));
            }
            let title_escaped = html_escape(&item.title);
            let url_escaped = html_escape(&item.url);
            if item.is_current {
                list_html.push_str(&format!(
                    r#"<li class="np-breadcrumb-item active" aria-current="page"><span>{}</span></li>"#,
                    title_escaped
                ));
            } else {
                list_html.push_str(&format!(
                    r#"<li class="np-breadcrumb-item"><a href="{}">{}</a></li>"#,
                    url_escaped,
                    title_escaped
                ));
            }
        }

        format!(
            r#"<nav class="np-breadcrumbs" aria-label="Breadcrumb"><ol class="np-breadcrumb-list">{}</ol></nav>"#,
            list_html
        )
    }

    fn create_hydration_context<'a>(
        req_ctx: &RequestContext,
        settings: &'a WpSettingsResponse,
        post: Option<&'a WpPostResponse>,
    ) -> HydrationContext<'a> {
        let mut nav_menus = HashMap::new();
        if !settings.nav_menu_items.is_empty() {
            let primary_items: Vec<NavMenuItem> = settings.nav_menu_items.iter().enumerate().map(|(idx, it)| NavMenuItem {
                id: it.id,
                title: it.title.clone(),
                url: it.url.clone(),
                target: it.target.clone(),
                parent: it.parent,
                order: idx as i64,
                ..Default::default()
            }).collect();
            nav_menus.insert("primary".to_string(), primary_items.clone());
            nav_menus.insert("main-menu".to_string(), primary_items);
        }

        let breadcrumbs = req_ctx.build_breadcrumbs();

        HydrationContext {
            post,
            settings,
            breadcrumbs,
            nav_menus,
        }
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}
