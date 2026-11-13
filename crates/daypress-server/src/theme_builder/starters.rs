//! 1-Click Ready-to-Use Starter Theme Templates (Headers & Footers).

#![allow(clippy::field_reassign_with_default)]
use super::models::{
    ConditionType, CreateConditionDto, CreateTemplateRequest, RuleTarget, StarterTemplateInfo,
    TemplateSettings, TemplateType, ThemeTemplateRecord,
};
use super::repository::ThemeTemplateRepository;
use crate::adapters::database::Database;
use crate::errors::AppError;
use axum::{
    body::Bytes,
    extract::{Query, State},
    http::StatusCode,
    response::Json,
};
use daypress_ast::{DaypressLayout, DaypressNode, NavMenuItem, NodeKind, Spacing, StyleProperties};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct InstallStarterPayload {
    #[serde(default)]
    pub starter_id: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
}

pub fn get_starter_catalog() -> Vec<StarterTemplateInfo> {
    vec![
        StarterTemplateInfo {
            id: "agency-header".to_string(),
            title: "Agency Pro Sticky Header".to_string(),
            template_type: TemplateType::Header,
            category: "Agency".to_string(),
            preview_img: "/static/starters/agency-header.svg".to_string(),
            description: "Modern sticky translucent header with site logo, primary navigation menu, and primary CTA button.".to_string(),
        },
        StarterTemplateInfo {
            id: "dark-grid-footer".to_string(),
            title: "Modern Dark Grid Footer".to_string(),
            template_type: TemplateType::Footer,
            category: "Modern".to_string(),
            preview_img: "/static/starters/dark-footer.svg".to_string(),
            description: "High-density 4-column footer featuring company bio, quick links, newsletter signup, and copyright.".to_string(),
        },
        StarterTemplateInfo {
            id: "saas-header".to_string(),
            title: "SaaS Minimal Header".to_string(),
            template_type: TemplateType::Header,
            category: "SaaS".to_string(),
            preview_img: "/static/starters/saas-header.svg".to_string(),
            description: "Crisp hairline layout with product links, changelog pill, and Sign In action.".to_string(),
        },
        StarterTemplateInfo {
            id: "clean-footer".to_string(),
            title: "Clean Minimalist Footer".to_string(),
            template_type: TemplateType::Footer,
            category: "Minimal".to_string(),
            preview_img: "/static/starters/clean-footer.svg".to_string(),
            description: "Single-row streamlined footer with copyright token and social links.".to_string(),
        },
        StarterTemplateInfo {
            id: "global-cta-symbol".to_string(),
            title: "Synced Global CTA Banner".to_string(),
            template_type: TemplateType::SectionSymbol,
            category: "Symbol".to_string(),
            preview_img: "/static/starters/global-cta-symbol.svg".to_string(),
            description: "Reusable synced call-to-action banner with overridable heading, copy, and button link.".to_string(),
        },
        StarterTemplateInfo {
            id: "top-announcement-popup".to_string(),
            title: "Site-Wide Top Announcement Bar".to_string(),
            template_type: TemplateType::Popup,
            category: "Conversion".to_string(),
            preview_img: "/static/starters/top-announcement.svg".to_string(),
            description: "Dismissible top announcement banner with release highlight and action link.".to_string(),
        },
        StarterTemplateInfo {
            id: "editorial-single-post".to_string(),
            title: "Editorial Single Post Layout".to_string(),
            template_type: TemplateType::SinglePost,
            category: "Editorial".to_string(),
            preview_img: "/static/starters/editorial-single.svg".to_string(),
            description: "Dynamic single article template with {{post.title}}, {{post.date}}, {{author.name}}, and {{post.featured_image}} bindings.".to_string(),
        },
        StarterTemplateInfo {
            id: "magazine-archive-grid".to_string(),
            title: "Magazine Archive Loop Grid".to_string(),
            template_type: TemplateType::Archive,
            category: "Archive".to_string(),
            preview_img: "/static/starters/magazine-archive.svg".to_string(),
            description: "Dynamic category/tag/author archive template with {{archive.title}} header and 3-column server-rendered LoopGrid.".to_string(),
        },
        StarterTemplateInfo {
            id: "modern-error-404".to_string(),
            title: "Modern 404 Recovery Page".to_string(),
            template_type: TemplateType::Error404,
            category: "System".to_string(),
            preview_img: "/static/starters/error-404.svg".to_string(),
            description: "Branded 404 Not Found template with site title token and quick recovery navigation.".to_string(),
        },
    ]
}

fn make_node(
    id: &str,
    kind: NodeKind,
    styles: StyleProperties,
    children: Vec<DaypressNode>,
) -> DaypressNode {
    DaypressNode {
        id: id.to_string(),
        kind,
        styles,
        children,
    }
}

fn make_spacing(top: &str, right: &str, bottom: &str, left: &str) -> Spacing {
    Spacing {
        top: Some(top.to_string()),
        right: Some(right.to_string()),
        bottom: Some(bottom.to_string()),
        left: Some(left.to_string()),
    }
}

pub fn build_starter_template(starter_id: &str) -> Option<CreateTemplateRequest> {
    match starter_id {
        "agency-header" => {
            let mut root_styles = StyleProperties::default();
            root_styles.display = Some("flex".to_string());
            root_styles.justify_content = Some("space-between".to_string());
            root_styles.align_items = Some("center".to_string());
            root_styles.padding = Some(make_spacing("16px", "24px", "16px", "24px"));
            root_styles.background_color = Some("rgba(13, 15, 18, 0.85)".to_string());

            let mut logo_raw = std::collections::HashMap::new();
            logo_raw.insert(
                "data-dynamic".to_string(),
                serde_json::Value::String("site_logo".to_string()),
            );
            let mut logo_styles = StyleProperties::default();
            logo_styles.raw_settings = Some(logo_raw);
            let logo = make_node(
                "header-site-logo",
                NodeKind::Image {
                    src: "/static/favicon.svg".to_string(),
                    alt: "Site Logo".to_string(),
                    width: Some(36),
                    height: Some(36),
                    link: Some("/".to_string()),
                },
                logo_styles,
                vec![],
            );

            let mut nav_styles = StyleProperties::default();
            nav_styles.margin = Some(make_spacing("0", "auto", "0", "auto"));
            let nav = make_node(
                "header-nav-menu",
                NodeKind::NavMenu {
                    menu_slug: "primary".to_string(),
                    layout: "horizontal".to_string(),
                    align: "center".to_string(),
                    mobile_hamburger: true,
                    items: vec![
                        NavMenuItem {
                            title: "Home".to_string(),
                            url: "/".to_string(),
                            order: 0,
                            ..Default::default()
                        },
                        NavMenuItem {
                            title: "About".to_string(),
                            url: "/about".to_string(),
                            order: 1,
                            ..Default::default()
                        },
                        NavMenuItem {
                            title: "Services".to_string(),
                            url: "/services".to_string(),
                            order: 2,
                            ..Default::default()
                        },
                        NavMenuItem {
                            title: "Contact".to_string(),
                            url: "/contact".to_string(),
                            order: 3,
                            ..Default::default()
                        },
                    ],
                },
                nav_styles,
                vec![],
            );

            let mut cta_styles = StyleProperties::default();
            cta_styles.background_color = Some("#2563EB".to_string());
            cta_styles.text_color = Some("#FFFFFF".to_string());
            cta_styles.padding = Some(make_spacing("8px", "16px", "8px", "16px"));
            cta_styles.border_radius = Some("6px".to_string());
            let cta = make_node(
                "header-cta-btn",
                NodeKind::Button {
                    text: "Get Started".to_string(),
                    url: "/contact".to_string(),
                    target: "_self".to_string(),
                },
                cta_styles,
                vec![],
            );

            let root = make_node(
                "agency-header-root",
                NodeKind::Section,
                root_styles,
                vec![logo, nav, cta],
            );
            let layout = DaypressLayout {
                version: 1,
                root,
                global_styles: None,
            };

            Some(CreateTemplateRequest {
                title: "Agency Pro Sticky Header".to_string(),
                template_type: TemplateType::Header,
                status: Some("publish".to_string()),
                layout: Some(layout),
                layout_json: None,
                settings: Some(TemplateSettings {
                    sticky: true,
                    transparent: true,
                    scroll_shrink: true,
                    z_index: Some(999),
                    ..Default::default()
                }),
                priority: Some(10),
                is_active: Some(true),
                conditions: Some(vec![CreateConditionDto {
                    id: None,
                    condition_type: ConditionType::Include,
                    rule_target: RuleTarget::EntireSite,
                    target_id: String::new(),
                    priority: 10,
                }]),
            })
        }
        "dark-grid-footer" => {
            let mut root_styles = StyleProperties::default();
            root_styles.background_color = Some("#0D0F12".to_string());
            root_styles.text_color = Some("#8A93A6".to_string());
            root_styles.padding = Some(make_spacing("48px", "24px", "32px", "24px"));

            let mut copy_styles = StyleProperties::default();
            copy_styles.text_align = Some("center".to_string());
            copy_styles.font_size = Some("13px".to_string());
            let copy = make_node("footer-copy", NodeKind::Text {
                content_html: "<p>© 2026 {{site_title}}. Built with Daypress. Ultra-fast sub-millisecond CMS.</p>".to_string(),
            }, copy_styles, vec![]);

            let root = make_node(
                "dark-footer-root",
                NodeKind::Section,
                root_styles,
                vec![copy],
            );
            let layout = DaypressLayout {
                version: 1,
                root,
                global_styles: None,
            };

            Some(CreateTemplateRequest {
                title: "Modern Dark Grid Footer".to_string(),
                template_type: TemplateType::Footer,
                status: Some("publish".to_string()),
                layout: Some(layout),
                layout_json: None,
                settings: Some(TemplateSettings {
                    sticky_footer: false,
                    ..Default::default()
                }),
                priority: Some(10),
                is_active: Some(true),
                conditions: Some(vec![CreateConditionDto {
                    id: None,
                    condition_type: ConditionType::Include,
                    rule_target: RuleTarget::EntireSite,
                    target_id: String::new(),
                    priority: 10,
                }]),
            })
        }
        "saas-header" => {
            let mut root_styles = StyleProperties::default();
            root_styles.display = Some("flex".to_string());
            root_styles.justify_content = Some("space-between".to_string());
            root_styles.align_items = Some("center".to_string());
            root_styles.padding = Some(make_spacing("12px", "24px", "12px", "24px"));
            root_styles.background_color = Some("#15181E".to_string());

            let title_node = make_node(
                "saas-title",
                NodeKind::Heading {
                    text: "{{site_title}}".to_string(),
                    level: 3,
                },
                StyleProperties::default(),
                vec![],
            );

            let root = make_node(
                "saas-header-root",
                NodeKind::Section,
                root_styles,
                vec![title_node],
            );
            let layout = DaypressLayout {
                version: 1,
                root,
                global_styles: None,
            };

            Some(CreateTemplateRequest {
                title: "SaaS Minimal Header".to_string(),
                template_type: TemplateType::Header,
                status: Some("publish".to_string()),
                layout: Some(layout),
                layout_json: None,
                settings: Some(TemplateSettings::default()),
                priority: Some(10),
                is_active: Some(true),
                conditions: Some(vec![CreateConditionDto {
                    id: None,
                    condition_type: ConditionType::Include,
                    rule_target: RuleTarget::EntireSite,
                    target_id: String::new(),
                    priority: 10,
                }]),
            })
        }
        "clean-footer" => {
            let mut root_styles = StyleProperties::default();
            root_styles.padding = Some(make_spacing("24px", "24px", "24px", "24px"));
            let text = make_node(
                "clean-copy",
                NodeKind::Text {
                    content_html: "<p>© {{current_year}} {{site_title}}. All rights reserved.</p>"
                        .to_string(),
                },
                StyleProperties::default(),
                vec![],
            );

            let root = make_node(
                "clean-footer-root",
                NodeKind::Section,
                root_styles,
                vec![text],
            );
            let layout = DaypressLayout {
                version: 1,
                root,
                global_styles: None,
            };

            Some(CreateTemplateRequest {
                title: "Clean Minimalist Footer".to_string(),
                template_type: TemplateType::Footer,
                status: Some("publish".to_string()),
                layout: Some(layout),
                layout_json: None,
                settings: Some(TemplateSettings::default()),
                priority: Some(10),
                is_active: Some(true),
                conditions: Some(vec![CreateConditionDto {
                    id: None,
                    condition_type: ConditionType::Include,
                    rule_target: RuleTarget::EntireSite,
                    target_id: String::new(),
                    priority: 10,
                }]),
            })
        }
        "global-cta-symbol" => {
            let mut root_styles = StyleProperties::default();
            root_styles.background_color = Some("#111827".to_string());
            root_styles.text_color = Some("#F9FAFB".to_string());
            root_styles.padding = Some(make_spacing("48px", "32px", "48px", "32px"));
            root_styles.border_radius = Some("16px".to_string());
            root_styles.text_align = Some("center".to_string());

            let heading = make_node(
                "cta-heading",
                NodeKind::Heading {
                    text: "Ready to accelerate {{site.title}} to sub-millisecond SSR?".to_string(),
                    level: 2,
                },
                StyleProperties::default(),
                vec![],
            );
            let subtext = make_node(
                "cta-subtext",
                NodeKind::Text {
                    content_html: "<p>Reusable synced global component with per-instance text and link overrides.</p>".to_string(),
                },
                StyleProperties::default(),
                vec![],
            );
            let mut btn_styles = StyleProperties::default();
            btn_styles.background_color = Some("#2563EB".to_string());
            btn_styles.text_color = Some("#FFFFFF".to_string());
            btn_styles.padding = Some(make_spacing("12px", "24px", "12px", "24px"));
            btn_styles.border_radius = Some("8px".to_string());
            let btn = make_node(
                "cta-button",
                NodeKind::Button {
                    text: "Get Started Now".to_string(),
                    url: "/contact".to_string(),
                    target: "_self".to_string(),
                },
                btn_styles,
                vec![],
            );

            let root = make_node(
                "global-cta-symbol-root",
                NodeKind::Section,
                root_styles,
                vec![heading, subtext, btn],
            );
            let layout = DaypressLayout {
                version: 1,
                root,
                global_styles: None,
            };

            Some(CreateTemplateRequest {
                title: "Synced Global CTA Banner".to_string(),
                template_type: TemplateType::SectionSymbol,
                status: Some("publish".to_string()),
                layout: Some(layout),
                layout_json: None,
                settings: Some(TemplateSettings::default()),
                priority: Some(10),
                is_active: Some(true),
                conditions: Some(vec![]),
            })
        }
        "top-announcement-popup" => {
            let mut root_styles = StyleProperties::default();
            root_styles.display = Some("flex".to_string());
            root_styles.justify_content = Some("center".to_string());
            root_styles.align_items = Some("center".to_string());
            root_styles.gap = Some("16px".to_string());
            root_styles.background_color = Some("#1D4ED8".to_string());
            root_styles.text_color = Some("#FFFFFF".to_string());
            root_styles.padding = Some(make_spacing("10px", "24px", "10px", "24px"));

            let msg = make_node(
                "announcement-text",
                NodeKind::Text {
                    content_html: "<span><strong>New in {{site.title}}:</strong> Sub-50µs Rust SSR &amp; Visual Theme Builder are live.</span>".to_string(),
                },
                StyleProperties::default(),
                vec![],
            );
            let btn = make_node(
                "announcement-cta",
                NodeKind::Button {
                    text: "Learn More".to_string(),
                    url: "/about".to_string(),
                    target: "_self".to_string(),
                },
                StyleProperties::default(),
                vec![],
            );

            let root = make_node(
                "top-announcement-root",
                NodeKind::Section,
                root_styles,
                vec![msg, btn],
            );
            let layout = DaypressLayout {
                version: 1,
                root,
                global_styles: None,
            };

            Some(CreateTemplateRequest {
                title: "Site-Wide Top Announcement Bar".to_string(),
                template_type: TemplateType::Popup,
                status: Some("publish".to_string()),
                layout: Some(layout),
                layout_json: None,
                settings: Some(TemplateSettings {
                    popup_variant: Some("announcement_bar_top".to_string()),
                    dismiss_days: Some(7),
                    ..Default::default()
                }),
                priority: Some(10),
                is_active: Some(true),
                conditions: Some(vec![CreateConditionDto {
                    id: None,
                    condition_type: ConditionType::Include,
                    rule_target: RuleTarget::EntireSite,
                    target_id: String::new(),
                    priority: 10,
                }]),
            })
        }
        "editorial-single-post" => {
            let mut root_styles = StyleProperties::default();
            root_styles.padding = Some(make_spacing("48px", "24px", "64px", "24px"));
            root_styles.max_width = Some("860px".to_string());
            root_styles.margin = Some(make_spacing("0", "auto", "0", "auto"));

            let meta_node = make_node(
                "single-post-meta",
                NodeKind::Text {
                    content_html: "<p class=\"dp-post-meta\">Published on {{post.date}} by <strong>{{author.name}}</strong></p>".to_string(),
                },
                StyleProperties::default(),
                vec![],
            );
            let title_node = make_node(
                "single-post-title",
                NodeKind::Heading {
                    text: "{{post.title}}".to_string(),
                    level: 1,
                },
                StyleProperties::default(),
                vec![],
            );
            let featured_img = make_node(
                "single-featured-image",
                NodeKind::Image {
                    src: "{{post.featured_image}}".to_string(),
                    alt: "{{post.title}}".to_string(),
                    width: None,
                    height: None,
                    link: None,
                },
                StyleProperties::default(),
                vec![],
            );
            let content_node = make_node(
                "single-post-content",
                NodeKind::CustomHtml {
                    raw: "<div class=\"dp-entry-content\">{{post.content}}</div>".to_string(),
                },
                StyleProperties::default(),
                vec![],
            );

            let root = make_node(
                "editorial-single-root",
                NodeKind::Section,
                root_styles,
                vec![meta_node, title_node, featured_img, content_node],
            );
            let layout = DaypressLayout {
                version: 1,
                root,
                global_styles: None,
            };

            Some(CreateTemplateRequest {
                title: "Editorial Single Post Layout".to_string(),
                template_type: TemplateType::SinglePost,
                status: Some("publish".to_string()),
                layout: Some(layout),
                layout_json: None,
                settings: Some(TemplateSettings::default()),
                priority: Some(10),
                is_active: Some(true),
                conditions: Some(vec![CreateConditionDto {
                    id: None,
                    condition_type: ConditionType::Include,
                    rule_target: RuleTarget::EntireSite,
                    target_id: String::new(),
                    priority: 10,
                }]),
            })
        }
        "magazine-archive-grid" => {
            let mut root_styles = StyleProperties::default();
            root_styles.padding = Some(make_spacing("48px", "24px", "64px", "24px"));
            root_styles.max_width = Some("1180px".to_string());
            root_styles.margin = Some(make_spacing("0", "auto", "0", "auto"));

            let archive_heading = make_node(
                "archive-header-title",
                NodeKind::Heading {
                    text: "{{archive.title}}".to_string(),
                    level: 1,
                },
                StyleProperties::default(),
                vec![],
            );
            let archive_desc = make_node(
                "archive-header-desc",
                NodeKind::Text {
                    content_html: "<p>{{archive.description}}</p>".to_string(),
                },
                StyleProperties::default(),
                vec![],
            );
            let loop_grid = make_node(
                "archive-loop-grid",
                NodeKind::LoopGrid {
                    post_type: "post".to_string(),
                    posts_per_page: 9,
                    columns: 3,
                    show_thumbnail: true,
                    show_excerpt: true,
                    show_meta: true,
                    show_pagination: true,
                    taxonomy: None,
                    enable_filter_bar: true,
                    enable_sort: true,
                    enable_search: true,
                    default_sort: "date_desc".to_string(),
                    card_skin: "editorial".to_string(),
                    meta_fields: Vec::new(),
                },
                StyleProperties::default(),
                vec![],
            );

            let root = make_node(
                "magazine-archive-root",
                NodeKind::Section,
                root_styles,
                vec![archive_heading, archive_desc, loop_grid],
            );
            let layout = DaypressLayout {
                version: 1,
                root,
                global_styles: None,
            };

            Some(CreateTemplateRequest {
                title: "Magazine Archive Loop Grid".to_string(),
                template_type: TemplateType::Archive,
                status: Some("publish".to_string()),
                layout: Some(layout),
                layout_json: None,
                settings: Some(TemplateSettings::default()),
                priority: Some(10),
                is_active: Some(true),
                conditions: Some(vec![CreateConditionDto {
                    id: None,
                    condition_type: ConditionType::Include,
                    rule_target: RuleTarget::EntireSite,
                    target_id: String::new(),
                    priority: 10,
                }]),
            })
        }
        "modern-error-404" => {
            let mut root_styles = StyleProperties::default();
            root_styles.padding = Some(make_spacing("72px", "24px", "72px", "24px"));
            root_styles.max_width = Some("680px".to_string());
            root_styles.margin = Some(make_spacing("0", "auto", "0", "auto"));
            root_styles.text_align = Some("center".to_string());

            let heading = make_node(
                "error-404-heading",
                NodeKind::Heading {
                    text: "404 — Page Not Found on {{site.title}}".to_string(),
                    level: 1,
                },
                StyleProperties::default(),
                vec![],
            );
            let desc = make_node(
                "error-404-desc",
                NodeKind::Text {
                    content_html:
                        "<p>The page you requested could not be found or may have moved.</p>"
                            .to_string(),
                },
                StyleProperties::default(),
                vec![],
            );
            let btn = make_node(
                "error-404-home-btn",
                NodeKind::Button {
                    text: "Return to Homepage".to_string(),
                    url: "/".to_string(),
                    target: "_self".to_string(),
                },
                StyleProperties::default(),
                vec![],
            );

            let root = make_node(
                "modern-error-404-root",
                NodeKind::Section,
                root_styles,
                vec![heading, desc, btn],
            );
            let layout = DaypressLayout {
                version: 1,
                root,
                global_styles: None,
            };

            Some(CreateTemplateRequest {
                title: "Modern 404 Recovery Page".to_string(),
                template_type: TemplateType::Error404,
                status: Some("publish".to_string()),
                layout: Some(layout),
                layout_json: None,
                settings: Some(TemplateSettings::default()),
                priority: Some(10),
                is_active: Some(true),
                conditions: Some(vec![CreateConditionDto {
                    id: None,
                    condition_type: ConditionType::Include,
                    rule_target: RuleTarget::EntireSite,
                    target_id: String::new(),
                    priority: 10,
                }]),
            })
        }
        "mega-menu-product-symbol" => {
            let mut root_styles = StyleProperties::default();
            root_styles.padding = Some(make_spacing("24px", "28px", "24px", "28px"));

            let heading = make_node(
                "mega-menu-heading",
                NodeKind::Heading {
                    text: "Platform Capabilities & Solutions".to_string(),
                    level: 3,
                },
                StyleProperties::default(),
                vec![],
            );
            let desc = make_node(
                "mega-menu-desc",
                NodeKind::Text {
                    content_html: "<p>Sub-millisecond Rust SSR, visual theme architecture, and 1-click WordPress migration.</p>".to_string(),
                },
                StyleProperties::default(),
                vec![],
            );
            let cta = make_node(
                "mega-menu-cta",
                NodeKind::Button {
                    text: "Explore Documentation".to_string(),
                    url: "/".to_string(),
                    target: "_self".to_string(),
                },
                StyleProperties::default(),
                vec![],
            );

            let root = make_node(
                "mega-menu-symbol-root",
                NodeKind::Section,
                root_styles,
                vec![heading, desc, cta],
            );
            let layout = DaypressLayout {
                version: 1,
                root,
                global_styles: None,
            };

            Some(CreateTemplateRequest {
                title: "Product Mega Menu Panel Symbol".to_string(),
                template_type: TemplateType::SectionSymbol,
                status: Some("publish".to_string()),
                layout: Some(layout),
                layout_json: None,
                settings: Some(TemplateSettings::default()),
                priority: Some(10),
                is_active: Some(true),
                conditions: Some(vec![CreateConditionDto {
                    id: None,
                    condition_type: ConditionType::Exclude,
                    rule_target: RuleTarget::EntireSite,
                    target_id: String::new(),
                    priority: 10,
                }]),
            })
        }
        _ => None,
    }
}

pub async fn list_starters_handler(
    _auth: crate::auth::AdminUser,
) -> Json<Vec<StarterTemplateInfo>> {
    Json(get_starter_catalog())
}

pub async fn install_starter_handler(
    _auth: crate::auth::AdminUser,
    State(db): State<Database>,
    Query(query): Query<InstallStarterPayload>,
    body: Bytes,
) -> Result<(StatusCode, Json<ThemeTemplateRecord>), AppError> {
    let mut starter_id = query.starter_id.or(query.id);
    if starter_id.is_none() && !body.is_empty() {
        if let Ok(parsed) = serde_json::from_slice::<InstallStarterPayload>(&body) {
            starter_id = parsed.starter_id.or(parsed.id);
        }
    }
    let s_id = starter_id.ok_or_else(|| AppError::BadRequest("Missing starter_id".to_string()))?;

    let req = build_starter_template(&s_id)
        .ok_or_else(|| AppError::NotFound(format!("Starter template '{}' not found", s_id)))?;

    let repo = ThemeTemplateRepository::from_database(&db);
    let record = repo.create(&req)?;
    Ok((StatusCode::CREATED, Json(record)))
}
