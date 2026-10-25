//! Public Front Page / Blog Index Renderer (`templates/public/home.rs`).

use super::shell::{plain_text_excerpt, render_base_layout};
use crate::auth::AuthUser;
use crate::models::{WpPostResponse, WpSettingsResponse};
use crate::templates::utils::escape_html;
use neopress_ast::{NeopressLayout, NeopressNode, NodeKind, StyleProperties};

/// Renders the dynamic Front Page / Blog Index 100% using Neopress AST elements (`neopress_ast`)
/// and live database content (`settings`, `posts`). Zero hardcoded marketing copy.
pub fn render_home_page(
    posts: &[WpPostResponse],
    settings: &WpSettingsResponse,
    current_user: Option<&AuthUser>,
    partials: Option<&crate::theme_builder::models::ResolvedThemePartials>,
) -> String {
    render_home_page_paginated(posts, settings, current_user, partials, "")
}

/// [`render_home_page`] with a pagination nav below "Latest Articles".
pub fn render_home_page_paginated(
    posts: &[WpPostResponse],
    settings: &WpSettingsResponse,
    current_user: Option<&AuthUser>,
    partials: Option<&crate::theme_builder::models::ResolvedThemePartials>,
    pagination_html: &str,
) -> String {
    let site_title = if settings.title.trim().is_empty() {
        "Neopress"
    } else {
        settings.title.trim()
    };
    let site_tagline = if settings.description.trim().is_empty() {
        "Published with Neopress Native Rust CMS"
    } else {
        settings.description.trim()
    };

    // 1. Construct the Hero Section as a genuine NeopressLayout AST and compile it to HTML
    let first_post = posts
        .iter()
        .find(|p| p.post_type == "post")
        .or_else(|| posts.first());
    let cta_url = first_post
        .map(|p| format!("/{}/", p.slug))
        .unwrap_or_else(|| "#posts".to_string());
    let cta_label = first_post
        .map(|p| format!("Read Latest: {}", p.title.rendered))
        .unwrap_or_else(|| "Browse Articles".to_string());
    let hero_img_url = posts
        .iter()
        .find_map(|p| p.featured_image_url.clone())
        .unwrap_or_default();

    let mut hero_left_children = vec![
        NeopressNode {
            id: "np_home_hero_title".to_string(),
            kind: NodeKind::Heading {
                text: escape_html(site_title),
                level: 1,
            },
            styles: StyleProperties::default(),
            children: vec![],
        },
        NeopressNode {
            id: "np_home_hero_desc".to_string(),
            kind: NodeKind::Text {
                content_html: format!("<p>{}</p>", escape_html(site_tagline)),
            },
            styles: StyleProperties::default(),
            children: vec![],
        },
        NeopressNode {
            id: "np_home_hero_cta".to_string(),
            kind: NodeKind::Button {
                text: escape_html(&cta_label),
                url: cta_url,
                target: "_self".to_string(),
            },
            styles: StyleProperties::default(),
            children: vec![],
        },
    ];

    let hero_grid_node = if !hero_img_url.is_empty() {
        NeopressNode {
            id: "np_home_hero_grid".to_string(),
            kind: NodeKind::Grid {
                columns: 2,
                gap: 40,
            },
            styles: StyleProperties::default(),
            children: vec![
                NeopressNode {
                    id: "np_home_hero_col_left".to_string(),
                    kind: NodeKind::Column,
                    styles: StyleProperties::default(),
                    children: hero_left_children,
                },
                NeopressNode {
                    id: "np_home_hero_col_right".to_string(),
                    kind: NodeKind::Column,
                    styles: StyleProperties::default(),
                    children: vec![NeopressNode {
                        id: "np_home_hero_image".to_string(),
                        kind: NodeKind::Image {
                            src: hero_img_url,
                            alt: site_title.to_string(),
                            width: None,
                            height: None,
                            link: None,
                        },
                        styles: StyleProperties::default(),
                        children: vec![],
                    }],
                },
            ],
        }
    } else {
        NeopressNode {
            id: "np_home_hero_col_single".to_string(),
            kind: NodeKind::Column,
            styles: StyleProperties::default(),
            children: std::mem::take(&mut hero_left_children),
        }
    };

    let hero_layout = NeopressLayout {
        version: 1,
        global_styles: None,
        root: NeopressNode {
            id: "np_home_hero_section".to_string(),
            kind: NodeKind::Section,
            styles: StyleProperties::default(),
            children: vec![NeopressNode {
                id: "np_home_hero_container".to_string(),
                kind: NodeKind::Container,
                styles: StyleProperties::default(),
                children: vec![hero_grid_node],
            }],
        },
    };
    let hero_html = neopress_ast::render_layout_to_html(&hero_layout);

    // 2. Render all published posts using Neopress Element Cards (.np-section -> .np-container -> .np-grid -> .np-card)
    let max_posts = settings.posts_per_page.max(1) as usize;
    let mut post_cards_html = String::new();
    for post in posts
        .iter()
        .filter(|p| p.post_type == "post" || p.post_type.is_empty())
        .take(max_posts)
    {
        let excerpt = if !post.excerpt.rendered.trim().is_empty() {
            plain_text_excerpt(&post.excerpt.rendered, 140)
        } else {
            plain_text_excerpt(&post.content.rendered, 140)
        };
        let date_short: String = post.date.chars().take(10).collect();
        let cat_label = post
            .category_names
            .first()
            .map(|c| escape_html(c))
            .unwrap_or_else(|| "Article".to_string());
        let author_label = if post.author_name.trim().is_empty() {
            "admin".to_string()
        } else {
            escape_html(&post.author_name)
        };

        let media_html = if let Some(ref img_url) = post.featured_image_url {
            if !img_url.is_empty() {
                format!(
                    r#"<div class="np-card-media"><img src="{}" alt="{}" loading="lazy" /></div>"#,
                    escape_html(img_url),
                    escape_html(&post.title.rendered)
                )
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let card_cls = if post.sticky {
            "np-card sticky"
        } else {
            "np-card"
        };

        post_cards_html.push_str(&format!(
            r##"
            <a href="/{slug}/" class="{card_cls}">
                {media_html}
                <div class="np-card-body">
                    <div class="np-card-meta">
                        <span class="np-badge">{cat_label}</span>
                        <span>&bull;</span>
                        <span>{date}</span>
                        <span>&bull;</span>
                        <span>{author}</span>
                    </div>
                    <h3 class="np-card-title">{title}</h3>
                    <p class="np-card-excerpt">{excerpt}</p>
                    <div class="np-card-link">Read article &rarr;</div>
                </div>
            </a>
            "##,
            slug = escape_html(&post.slug),
            card_cls = card_cls,
            media_html = media_html,
            cat_label = cat_label,
            date = escape_html(&date_short),
            author = author_label,
            title = post.title.rendered,
            excerpt = excerpt,
        ));
    }

    // 3. Render site navigation / pages directory section using Neopress Elements if menu items exist
    let pages_section_html = if !settings.nav_menu_items.is_empty() {
        let mut page_cards = String::new();
        for item in settings
            .nav_menu_items
            .iter()
            .filter(|i| i.url != "/" && !i.title.eq_ignore_ascii_case("Home"))
        {
            page_cards.push_str(&format!(
                r##"
                <a href="{url}" class="np-iconbox">
                    <div class="np-iconbox-icon"><i class="fa-solid fa-file-lines"></i></div>
                    <h3 class="np-iconbox-title">{title}</h3>
                    <p class="np-iconbox-desc">Explore {title} &rarr;</p>
                </a>
                "##,
                url = escape_html(&item.url),
                title = escape_html(&item.title),
            ));
        }
        if !page_cards.is_empty() {
            format!(
                r##"
                <section id="np-site-pages" class="np-section">
                    <div class="np-container">
                        <h2 class="np-heading" style="margin-bottom: 24px;">Site Pages</h2>
                        <div class="np-grid" style="grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));">
                            {page_cards}
                        </div>
                    </div>
                </section>
                "##,
                page_cards = page_cards
            )
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let home_content = format!(
        r##"
        {hero_html}
        <section id="posts" class="np-section">
            <div class="np-container">
                <h2 class="np-heading" style="margin-bottom: 28px;">Latest Articles</h2>
                <div class="np-grid" style="grid-template-columns: repeat(auto-fit, minmax(340px, 1fr));">
                    {post_cards_html}
                </div>
                {pagination_html}
            </div>
        </section>
        {pages_section_html}
        "##,
        hero_html = hero_html,
        post_cards_html = post_cards_html,
        pagination_html = pagination_html,
        pages_section_html = pages_section_html,
    );

    render_base_layout(
        &settings.title,
        settings,
        &home_content,
        current_user,
        None,
        partials,
    )
}
