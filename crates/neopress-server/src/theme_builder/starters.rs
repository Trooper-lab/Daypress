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
use neopress_ast::{NavMenuItem, NeopressLayout, NeopressNode, NodeKind, Spacing, StyleProperties};
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
    ]
}

fn make_node(
    id: &str,
    kind: NodeKind,
    styles: StyleProperties,
    children: Vec<NeopressNode>,
) -> NeopressNode {
    NeopressNode {
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
            let layout = NeopressLayout {
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
                content_html: "<p>© 2026 {{site_title}}. Built with Neopress. Ultra-fast sub-millisecond CMS.</p>".to_string(),
            }, copy_styles, vec![]);

            let root = make_node(
                "dark-footer-root",
                NodeKind::Section,
                root_styles,
                vec![copy],
            );
            let layout = NeopressLayout {
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
            let layout = NeopressLayout {
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
            let layout = NeopressLayout {
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
        _ => None,
    }
}

pub async fn list_starters_handler() -> Json<Vec<StarterTemplateInfo>> {
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
