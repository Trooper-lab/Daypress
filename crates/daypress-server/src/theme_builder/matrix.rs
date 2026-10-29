//! Header & Footer Page Assignment Matrix, Condition Resolution Summary,
//! and Per-Page / Per-Route Slot Override Persistence (`theme_builder/matrix.rs`).

use super::conditions::{evaluate_condition, resolve_template_for_type, ConditionMatch};
use super::models::*;
use super::repository::ThemeTemplateRepository;
use crate::adapters::database::Database;
use crate::models::{WpPostResponse, WpSettingsResponse};
use chrono::Utc;
use daypress_ast::{DaypressLayout, DaypressNode, NodeKind, StyleProperties};
use rusqlite::{params, OptionalExtension, Result};
use std::collections::{HashMap, HashSet};

/// Synchronizes legacy/migrated `daypress_elementor_header_html` and `daypress_elementor_footer_html`
/// options into `daypress_templates` if no Header or Footer template exists yet.
pub fn ensure_legacy_headers_footers_synced(db: &Database, settings: &WpSettingsResponse) {
    let repo = ThemeTemplateRepository::from_database(db);
    let existing = repo.list(&TemplateFilter::default()).unwrap_or_default();

    let has_header = existing
        .iter()
        .any(|t| t.template_type == TemplateType::Header);
    if !has_header && !settings.elementor_header_html.trim().is_empty() {
        let layout = DaypressLayout {
            version: 1,
            root: DaypressNode {
                id: "migrated_header_root".to_string(),
                kind: NodeKind::CustomHtml {
                    raw: settings.elementor_header_html.clone(),
                },
                styles: StyleProperties::default(),
                children: Vec::new(),
            },
            global_styles: None,
        };
        let _ = repo.create(&CreateTemplateRequest {
            title: "Migrated Theme Header".to_string(),
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
        });
    }

    let has_footer = existing
        .iter()
        .any(|t| t.template_type == TemplateType::Footer);
    if !has_footer && !settings.elementor_footer_html.trim().is_empty() {
        let layout = DaypressLayout {
            version: 1,
            root: DaypressNode {
                id: "migrated_footer_root".to_string(),
                kind: NodeKind::CustomHtml {
                    raw: settings.elementor_footer_html.clone(),
                },
                styles: StyleProperties::default(),
                children: Vec::new(),
            },
            global_styles: None,
        };
        let _ = repo.create(&CreateTemplateRequest {
            title: "Migrated Theme Footer".to_string(),
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
        });
    }
}

/// Reads per-post slot override (`_np_header_override` or `_np_footer_override`) from `WpPostResponse.meta`
/// or directly from `postmeta`.
pub fn get_post_slot_override(
    db: &Database,
    post: Option<&WpPostResponse>,
    slot: TemplateType,
) -> Option<String> {
    let key = match slot {
        TemplateType::Header => "_np_header_override",
        TemplateType::Footer => "_np_footer_override",
        _ => return None,
    };

    if let Some(p) = post {
        if let Some(val) = p.meta.get(key).and_then(|v| v.as_str()) {
            let trimmed = val.trim();
            if !trimmed.is_empty() && trimmed != "inherit" {
                return Some(trimmed.to_string());
            }
        }
        let conn = db.conn.lock().ok()?;
        let val: Option<String> = conn
            .query_row(
                "SELECT meta_value FROM postmeta WHERE post_id = ?1 AND meta_key = ?2 LIMIT 1",
                params![p.id, key],
                |r| r.get(0),
            )
            .optional()
            .ok()
            .flatten();
        if let Some(v) = val {
            let trimmed = v.trim();
            if !trimmed.is_empty() && trimmed != "inherit" {
                return Some(trimmed.to_string());
            }
        }
    }
    None
}

/// Reads route-level slot override from `options` (`_np_header_route_{route_key}`).
pub fn get_route_slot_override(
    db: &Database,
    route_key: &str,
    slot: TemplateType,
) -> Option<String> {
    let slot_str = match slot {
        TemplateType::Header => "header",
        TemplateType::Footer => "footer",
        _ => return None,
    };
    let opt_name = format!("_np_{}_route_{}", slot_str, route_key);
    let conn = db.conn.lock().ok()?;
    let val: Option<String> = conn
        .query_row(
            "SELECT option_value FROM options WHERE option_name = ?1 LIMIT 1",
            params![opt_name],
            |r| r.get(0),
        )
        .optional()
        .ok()
        .flatten();
    val.map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "inherit")
}

/// Maps a request context to its primary route key for checking route-level overrides in SSR.
pub fn context_route_keys(ctx: &RequestContext) -> Vec<&'static str> {
    let mut keys = Vec::new();
    if ctx.is_front_page {
        keys.push("front_page");
    }
    if ctx.is_404 {
        keys.push("not_found");
    } else if ctx.is_archive {
        keys.push("archive");
    } else if ctx.post_type.as_deref() == Some("page") {
        keys.push("all_pages");
    } else if ctx.post_type.as_deref() == Some("post") {
        keys.push("all_posts");
    }
    keys
}

/// Describes which condition rule caused `template` to match `ctx`.
pub fn describe_winning_reason(
    template: &ThemeTemplateRecord,
    ctx: &RequestContext,
) -> (String, String) {
    let mut best_spec = 0;
    let mut best_target = RuleTarget::EntireSite;

    for cond in &template.conditions {
        if cond.condition_type == ConditionType::Include {
            if let ConditionMatch::Matched { specificity } = evaluate_condition(cond, ctx) {
                if specificity >= best_spec {
                    best_spec = specificity;
                    best_target = cond.rule_target;
                }
            }
        }
    }

    if best_spec == 0 {
        return ("Entire Site (Default)".to_string(), "global".to_string());
    }

    match best_target {
        RuleTarget::Page => ("Specific Page Rule".to_string(), "custom".to_string()),
        RuleTarget::Post => ("Specific Post Rule".to_string(), "custom".to_string()),
        RuleTarget::FrontPage => ("Front Page Rule".to_string(), "custom".to_string()),
        RuleTarget::NotFound => ("404 Page Rule".to_string(), "custom".to_string()),
        RuleTarget::Category => ("Category Rule".to_string(), "custom".to_string()),
        RuleTarget::Tag => ("Tag Rule".to_string(), "custom".to_string()),
        RuleTarget::Author => ("Author Rule".to_string(), "custom".to_string()),
        RuleTarget::AllPages => ("All Pages Rule".to_string(), "global".to_string()),
        RuleTarget::AllPosts => ("All Posts Rule".to_string(), "global".to_string()),
        RuleTarget::Archive => ("All Archives Rule".to_string(), "global".to_string()),
        RuleTarget::EntireSite => ("Entire Site Rule".to_string(), "global".to_string()),
    }
}

/// Resolves the effective Header or Footer status for a given Page/Post or System Route.
#[allow(clippy::too_many_arguments)]
pub fn resolve_slot_status(
    db: &Database,
    slot: TemplateType,
    active_templates: &[ThemeTemplateRecord],
    ctx: &RequestContext,
    post: Option<&WpPostResponse>,
    route_key: Option<&str>,
    settings: &WpSettingsResponse,
) -> ResolvedSlotStatus {
    let slot_label = if slot == TemplateType::Header {
        "Header"
    } else {
        "Footer"
    };

    // 1. Check Blank Canvas Page Template
    if let Some(p) = post {
        let is_blank = matches!(
            p.template.as_str(),
            "canvas"
                | "elementor_canvas"
                | "page-template-blank.php"
                | "leeg.php"
                | "page_templates/leeg.php"
        );
        if is_blank {
            return ResolvedSlotStatus {
                mode: "inherit".to_string(),
                template_id: None,
                display_name: format!("Blank Canvas (No {})", slot_label),
                reason: format!("Page Template '{}' hides header & footer", p.template),
                badge_kind: "hidden".to_string(),
            };
        }
    }

    // 2. Check Post-level Override (`_np_header_override` / `_np_footer_override`)
    if let Some(ov) = get_post_slot_override(db, post, slot) {
        if ov == "none" {
            return ResolvedSlotStatus {
                mode: "none".to_string(),
                template_id: None,
                display_name: format!("Hidden (No {})", slot_label),
                reason: "Explicitly hidden on this page".to_string(),
                badge_kind: "hidden".to_string(),
            };
        } else if ov == "fallback" {
            return ResolvedSlotStatus {
                mode: "fallback".to_string(),
                template_id: None,
                display_name: format!("Built-in Theme {}", slot_label),
                reason: "Forced built-in theme fallback on this page".to_string(),
                badge_kind: "fallback".to_string(),
            };
        }
    }

    // 3. Check Route-level Override (`_np_header_route_{rk}`)
    let route_to_check = route_key
        .map(|s| vec![s])
        .unwrap_or_else(|| context_route_keys(ctx));
    for rk in route_to_check {
        if let Some(ov) = get_route_slot_override(db, rk, slot) {
            let is_direct_route = route_key == Some(rk);
            if ov == "none" {
                return ResolvedSlotStatus {
                    mode: if is_direct_route {
                        "none".to_string()
                    } else {
                        "inherit".to_string()
                    },
                    template_id: None,
                    display_name: format!("Hidden (No {})", slot_label),
                    reason: format!("Hidden by '{}' route rule", rk.replace('_', " ")),
                    badge_kind: "hidden".to_string(),
                };
            } else if ov == "fallback" {
                return ResolvedSlotStatus {
                    mode: if is_direct_route {
                        "fallback".to_string()
                    } else {
                        "inherit".to_string()
                    },
                    template_id: None,
                    display_name: format!("Built-in Theme {}", slot_label),
                    reason: format!(
                        "Built-in fallback via '{}' route rule",
                        rk.replace('_', " ")
                    ),
                    badge_kind: "fallback".to_string(),
                };
            }
        }
    }

    // Determine if there is an explicit direct Include rule for this exact page/post or route
    let direct_template_id: Option<i64> = if let Some(p) = post {
        let pid_str = p.id.to_string();
        active_templates
            .iter()
            .filter(|t| t.template_type == slot)
            .find(|t| {
                t.conditions.iter().any(|c| {
                    c.condition_type == ConditionType::Include
                        && matches!(c.rule_target, RuleTarget::Page | RuleTarget::Post)
                        && c.target_id == pid_str
                })
            })
            .map(|t| t.id)
    } else if let Some(rk) = route_key {
        if let Some(rt) = RuleTarget::from_str_loose(rk) {
            active_templates
                .iter()
                .filter(|t| t.template_type == slot)
                .find(|t| {
                    t.conditions.iter().any(|c| {
                        c.condition_type == ConditionType::Include
                            && c.rule_target == rt
                            && c.target_id.is_empty()
                    })
                })
                .map(|t| t.id)
        } else {
            None
        }
    } else {
        None
    };

    // 4. Evaluate Theme Builder templates
    if let Some(winner) = resolve_template_for_type(active_templates, slot, ctx) {
        let (reason, badge_kind) = describe_winning_reason(&winner, ctx);
        let mode = if direct_template_id == Some(winner.id) {
            format!("template:{}", winner.id)
        } else {
            "inherit".to_string()
        };
        return ResolvedSlotStatus {
            mode,
            template_id: Some(winner.id),
            display_name: winner.title,
            reason,
            badge_kind,
        };
    }

    // 5. Fallback if no active template matched
    let has_any_tb_of_slot = active_templates.iter().any(|t| t.template_type == slot);
    let legacy_html = if slot == TemplateType::Header {
        &settings.elementor_header_html
    } else {
        &settings.elementor_footer_html
    };

    if !has_any_tb_of_slot && !legacy_html.trim().is_empty() {
        ResolvedSlotStatus {
            mode: "inherit".to_string(),
            template_id: None,
            display_name: format!("Migrated Theme {}", slot_label),
            reason: "Global Migrated Option".to_string(),
            badge_kind: "global".to_string(),
        }
    } else {
        ResolvedSlotStatus {
            mode: "inherit".to_string(),
            template_id: None,
            display_name: format!("Built-in Theme {}", slot_label),
            reason: if has_any_tb_of_slot {
                "Excluded from custom templates (Theme Fallback)".to_string()
            } else {
                "Default Daypress Theme Shell".to_string()
            },
            badge_kind: "fallback".to_string(),
        }
    }
}

/// Formats a human-readable summary of a template's conditions using page/post titles where available.
pub fn format_conditions_summary(
    conditions: &[TemplateCondition],
    post_titles: &HashMap<i64, String>,
) -> String {
    if conditions.is_empty() {
        return "Entire Site (Default)".to_string();
    }
    if conditions.len() == 1
        && conditions[0].condition_type == ConditionType::Exclude
        && conditions[0].rule_target == RuleTarget::EntireSite
    {
        return "Unassigned (No pages selected)".to_string();
    }
    let mut parts = Vec::new();
    for c in conditions {
        let prefix = match c.condition_type {
            ConditionType::Include => "Include",
            ConditionType::Exclude => "Exclude",
        };
        let target_desc = match c.rule_target {
            RuleTarget::EntireSite => "Entire Site".to_string(),
            RuleTarget::FrontPage => "Front Page".to_string(),
            RuleTarget::AllPages => "All Pages".to_string(),
            RuleTarget::AllPosts => "All Blog Posts".to_string(),
            RuleTarget::Archive => "All Archives".to_string(),
            RuleTarget::NotFound => "404 Error Page".to_string(),
            RuleTarget::Page => {
                if let Ok(pid) = c.target_id.parse::<i64>() {
                    if let Some(title) = post_titles.get(&pid) {
                        format!("Page: {} (#{})", title, pid)
                    } else {
                        format!("Page #{}", pid)
                    }
                } else {
                    "Page".to_string()
                }
            }
            RuleTarget::Post => {
                if let Ok(pid) = c.target_id.parse::<i64>() {
                    if let Some(title) = post_titles.get(&pid) {
                        format!("Post: {} (#{})", title, pid)
                    } else {
                        format!("Post #{}", pid)
                    }
                } else {
                    "Post".to_string()
                }
            }
            RuleTarget::Category => {
                if c.target_id.is_empty() {
                    "Category Archive".to_string()
                } else {
                    format!("Category ({})", c.target_id)
                }
            }
            RuleTarget::Tag => {
                if c.target_id.is_empty() {
                    "Tag Archive".to_string()
                } else {
                    format!("Tag ({})", c.target_id)
                }
            }
            RuleTarget::Author => {
                if c.target_id.is_empty() {
                    "Author Archive".to_string()
                } else {
                    format!("Author #{}", c.target_id)
                }
            }
        };
        parts.push(format!("{}: {}", prefix, target_desc));
    }
    parts.join(" • ")
}

/// Computes the full Header & Footer Matrix (Templates, Coverage Summary, System Routes, Pages, and Posts).
pub fn build_header_footer_matrix(db: &Database) -> HeaderFooterMatrixResponse {
    let settings = db.get_settings().unwrap_or_default();
    ensure_legacy_headers_footers_synced(db, &settings);

    let repo = ThemeTemplateRepository::from_database(db);
    let all_templates = repo.list(&TemplateFilter::default()).unwrap_or_default();
    let active_templates: Vec<ThemeTemplateRecord> = all_templates
        .iter()
        .filter(|t| t.is_active && t.status == "publish")
        .cloned()
        .collect();

    let raw_pages = db
        .get_posts("page", Some("any"), None, 300)
        .unwrap_or_default();
    let raw_posts = db
        .get_posts("post", Some("any"), None, 200)
        .unwrap_or_default();

    let mut post_titles: HashMap<i64, String> = HashMap::new();
    for p in raw_pages.iter().chain(raw_posts.iter()) {
        let t = if p.title.rendered.trim().is_empty() {
            format!("Untitled #{}", p.id)
        } else {
            p.title.rendered.clone()
        };
        post_titles.insert(p.id, t);
    }

    let nav_slugs: HashSet<String> = settings
        .nav_menu_items
        .iter()
        .map(|m| m.url.trim_matches('/').to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    // 1. Build System Routes rows
    let system_routes_spec = vec![
        (
            "front_page",
            "Front Page (Home)",
            "/",
            RequestContext::for_front_page(),
        ),
        (
            "all_pages",
            "All Standard Pages (Default)",
            "/(pages)",
            RequestContext {
                path: "/sample-page/".to_string(),
                post_type: Some("page".to_string()),
                ..Default::default()
            },
        ),
        (
            "all_posts",
            "All Blog Posts (Default)",
            "/(posts)",
            RequestContext {
                path: "/sample-post/".to_string(),
                post_type: Some("post".to_string()),
                ..Default::default()
            },
        ),
        (
            "archive",
            "Category, Tag & Blog Archives",
            "/category/*",
            RequestContext {
                path: "/category/news/".to_string(),
                is_archive: true,
                ..Default::default()
            },
        ),
        (
            "not_found",
            "404 Not Found Error Page",
            "/404",
            RequestContext::for_404("/404"),
        ),
    ];

    let mut routes = Vec::new();
    for (rk, r_title, r_link, r_ctx) in system_routes_spec {
        let header = resolve_slot_status(
            db,
            TemplateType::Header,
            &active_templates,
            &r_ctx,
            None,
            Some(rk),
            &settings,
        );
        let footer = resolve_slot_status(
            db,
            TemplateType::Footer,
            &active_templates,
            &r_ctx,
            None,
            Some(rk),
            &settings,
        );
        routes.push(PageAssignmentRow {
            target_kind: "route".to_string(),
            post_id: 0,
            route_key: rk.to_string(),
            title: r_title.to_string(),
            slug: rk.to_string(),
            permalink: r_link.to_string(),
            status: "publish".to_string(),
            is_front_page: rk == "front_page",
            in_main_menu: false,
            page_template: "system".to_string(),
            header,
            footer,
        });
    }

    // 2. Build Pages rows (ordered with Front Page first, then Main Menu, then Published)
    let mut ordered_pages: Vec<&WpPostResponse> =
        raw_pages.iter().filter(|p| p.status != "trash").collect();
    ordered_pages.sort_by(|a, b| {
        let a_front = a.id == settings.page_on_front;
        let b_front = b.id == settings.page_on_front;
        let a_nav = nav_slugs.contains(&a.slug.to_lowercase());
        let b_nav = nav_slugs.contains(&b.slug.to_lowercase());
        b_front
            .cmp(&a_front)
            .then_with(|| b_nav.cmp(&a_nav))
            .then_with(|| a.title.rendered.cmp(&b.title.rendered))
    });

    let mut pages = Vec::new();
    for p in ordered_pages {
        let is_front = settings.page_on_front > 0 && p.id == settings.page_on_front;
        let ctx = RequestContext::for_post(p.id, "page", is_front, Vec::new());
        let header = resolve_slot_status(
            db,
            TemplateType::Header,
            &active_templates,
            &ctx,
            Some(p),
            None,
            &settings,
        );
        let footer = resolve_slot_status(
            db,
            TemplateType::Footer,
            &active_templates,
            &ctx,
            Some(p),
            None,
            &settings,
        );
        let permalink = if is_front {
            "/".to_string()
        } else {
            format!("/{}/", p.slug.trim_matches('/'))
        };
        let title = if p.title.rendered.trim().is_empty() {
            format!("(Untitled Page #{})", p.id)
        } else {
            p.title.rendered.clone()
        };
        pages.push(PageAssignmentRow {
            target_kind: "page".to_string(),
            post_id: p.id,
            route_key: String::new(),
            title,
            slug: p.slug.clone(),
            permalink,
            status: p.status.clone(),
            is_front_page: is_front,
            in_main_menu: nav_slugs.contains(&p.slug.to_lowercase()),
            page_template: if p.template.is_empty() {
                "default".to_string()
            } else {
                p.template.clone()
            },
            header,
            footer,
        });
    }

    // 3. Build Posts rows
    let mut posts = Vec::new();
    for p in raw_posts.iter().filter(|p| p.status != "trash") {
        let terms: Vec<TaxonomyTermRef> = p
            .categories
            .iter()
            .map(|cid| TaxonomyTermRef {
                taxonomy: "category".to_string(),
                term_id: *cid,
                slug: String::new(),
            })
            .collect();
        let ctx = RequestContext::for_post(p.id, "post", false, terms);
        let header = resolve_slot_status(
            db,
            TemplateType::Header,
            &active_templates,
            &ctx,
            Some(p),
            None,
            &settings,
        );
        let footer = resolve_slot_status(
            db,
            TemplateType::Footer,
            &active_templates,
            &ctx,
            Some(p),
            None,
            &settings,
        );
        let title = if p.title.rendered.trim().is_empty() {
            format!("(Untitled Post #{})", p.id)
        } else {
            p.title.rendered.clone()
        };
        posts.push(PageAssignmentRow {
            target_kind: "post".to_string(),
            post_id: p.id,
            route_key: String::new(),
            title,
            slug: p.slug.clone(),
            permalink: format!("/{}/", p.slug.trim_matches('/')),
            status: p.status.clone(),
            is_front_page: false,
            in_main_menu: false,
            page_template: if p.template.is_empty() {
                "default".to_string()
            } else {
                p.template.clone()
            },
            header,
            footer,
        });
    }

    // 4. Build Coverage Summary per Template
    let mut coverage: HashMap<i64, TemplateCoverageSummary> = HashMap::new();
    for tpl in &all_templates {
        let conditions_summary = format_conditions_summary(&tpl.conditions, &post_titles);
        let mut matched_pages = Vec::new();

        match tpl.template_type {
            TemplateType::Header => {
                for row in &pages {
                    if row.header.template_id == Some(tpl.id) {
                        matched_pages.push(MatchedPageChip {
                            post_id: row.post_id,
                            title: row.title.clone(),
                            permalink: row.permalink.clone(),
                            is_route: false,
                        });
                    }
                }
                for r in &routes {
                    if r.header.template_id == Some(tpl.id)
                        && (r.route_key == "archive"
                            || r.route_key == "not_found"
                            || r.route_key == "all_posts"
                            || pages.is_empty())
                    {
                        matched_pages.push(MatchedPageChip {
                            post_id: 0,
                            title: r.title.clone(),
                            permalink: r.permalink.clone(),
                            is_route: true,
                        });
                    }
                }
            }
            TemplateType::Footer => {
                for row in &pages {
                    if row.footer.template_id == Some(tpl.id) {
                        matched_pages.push(MatchedPageChip {
                            post_id: row.post_id,
                            title: row.title.clone(),
                            permalink: row.permalink.clone(),
                            is_route: false,
                        });
                    }
                }
                for r in &routes {
                    if r.footer.template_id == Some(tpl.id)
                        && (r.route_key == "archive"
                            || r.route_key == "not_found"
                            || r.route_key == "all_posts"
                            || pages.is_empty())
                    {
                        matched_pages.push(MatchedPageChip {
                            post_id: 0,
                            title: r.title.clone(),
                            permalink: r.permalink.clone(),
                            is_route: true,
                        });
                    }
                }
            }
            _ => {
                for r in &routes {
                    let r_ctx = match r.route_key.as_str() {
                        "front_page" => RequestContext::for_front_page(),
                        "all_pages" => RequestContext {
                            post_type: Some("page".to_string()),
                            ..Default::default()
                        },
                        "all_posts" => RequestContext {
                            post_type: Some("post".to_string()),
                            ..Default::default()
                        },
                        "archive" => RequestContext {
                            is_archive: true,
                            ..Default::default()
                        },
                        "not_found" => RequestContext::for_404("/404"),
                        _ => RequestContext::default(),
                    };
                    if let Some(w) =
                        resolve_template_for_type(&active_templates, tpl.template_type, &r_ctx)
                    {
                        if w.id == tpl.id {
                            matched_pages.push(MatchedPageChip {
                                post_id: 0,
                                title: r.title.clone(),
                                permalink: r.permalink.clone(),
                                is_route: true,
                            });
                        }
                    }
                }
            }
        }

        coverage.insert(
            tpl.id,
            TemplateCoverageSummary {
                template_id: tpl.id,
                matched_pages_count: matched_pages.len(),
                matched_pages,
                conditions_summary,
            },
        );
    }

    HeaderFooterMatrixResponse {
        templates: all_templates,
        coverage,
        routes,
        pages,
        posts,
    }
}

/// Atomically persists a Header and/or Footer slot assignment for a Page/Post (`post_id`) or System Route (`route_key`).
pub fn assign_page_or_route_templates(
    db: &Database,
    req: &AssignPageTemplateRequest,
) -> Result<()> {
    let post_id = req.post_id.filter(|&id| id > 0);
    let route_key = req
        .route_key
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let post_type = if let Some(pid) = post_id {
        db.get_post_by_id(pid)
            .ok()
            .flatten()
            .map(|p| p.post_type)
            .unwrap_or_else(|| "page".to_string())
    } else {
        String::new()
    };

    let mut conn = db.conn.lock().unwrap();
    let tx = conn.transaction()?;

    if let Some(ref h_val) = req.header {
        apply_slot_assignment_tx(
            &tx,
            TemplateType::Header,
            post_id,
            &post_type,
            route_key,
            h_val,
        )?;
    }
    if let Some(ref f_val) = req.footer {
        apply_slot_assignment_tx(
            &tx,
            TemplateType::Footer,
            post_id,
            &post_type,
            route_key,
            f_val,
        )?;
    }

    tx.commit()?;
    Ok(())
}

fn apply_slot_assignment_tx(
    tx: &rusqlite::Transaction,
    slot: TemplateType,
    post_id: Option<i64>,
    post_type: &str,
    route_key: Option<&str>,
    raw_mode: &str,
) -> Result<()> {
    let mode = raw_mode.trim();
    let now = Utc::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let slot_str = slot.as_str(); // "header" or "footer"

    // Determine (rule_target_str, target_id_str)
    let (rule_target_str, target_id_str) = if let Some(pid) = post_id {
        let rt = if post_type == "post" { "post" } else { "page" };
        (rt, pid.to_string())
    } else if let Some(rk) = route_key {
        let rt = match rk {
            "front_page" => "front_page",
            "all_pages" => "all_pages",
            "all_posts" => "all_posts",
            "archive" => "archive",
            "not_found" => "not_found",
            _ => "entire_site",
        };
        (rt, String::new())
    } else {
        return Ok(());
    };

    // Helper: delete existing conditions for this exact (slot, target),
    // and if a template loses its last condition, mark it unassigned (`exclude` `entire_site`)
    // so it does not implicitly fall back to matching the entire site.
    let remove_specific_conditions = |tx: &rusqlite::Transaction| -> Result<()> {
        let affected_tpl_ids: Vec<i64> = if let Some(pid) = post_id {
            let mut stmt = tx.prepare(
                r#"SELECT DISTINCT template_id FROM daypress_template_conditions
                   WHERE target_id = ?1
                     AND rule_target IN ('page', 'post')
                     AND template_id IN (SELECT id FROM daypress_templates WHERE template_type = ?2)"#,
            )?;
            let rows = stmt.query_map(params![pid.to_string(), slot_str], |r| r.get(0))?;
            rows.filter_map(|r| r.ok()).collect()
        } else {
            let mut stmt = tx.prepare(
                r#"SELECT DISTINCT template_id FROM daypress_template_conditions
                   WHERE rule_target = ?1 AND target_id = ''
                     AND template_id IN (SELECT id FROM daypress_templates WHERE template_type = ?2)"#,
            )?;
            let rows = stmt.query_map(params![rule_target_str, slot_str], |r| r.get(0))?;
            rows.filter_map(|r| r.ok()).collect()
        };

        if let Some(pid) = post_id {
            tx.execute(
                r#"DELETE FROM daypress_template_conditions
                   WHERE target_id = ?1
                     AND rule_target IN ('page', 'post')
                     AND template_id IN (SELECT id FROM daypress_templates WHERE template_type = ?2)"#,
                params![pid.to_string(), slot_str],
            )?;
        } else {
            tx.execute(
                r#"DELETE FROM daypress_template_conditions
                   WHERE rule_target = ?1 AND target_id = ''
                     AND template_id IN (SELECT id FROM daypress_templates WHERE template_type = ?2)"#,
                params![rule_target_str, slot_str],
            )?;
        }

        for tid in affected_tpl_ids {
            let rem_count: i64 = tx
                .query_row(
                    "SELECT COUNT(*) FROM daypress_template_conditions WHERE template_id = ?1",
                    params![tid],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            if rem_count == 0 {
                tx.execute(
                    r#"INSERT INTO daypress_template_conditions
                       (template_id, condition_type, rule_target, target_id, priority, created_at)
                       VALUES (?1, 'exclude', 'entire_site', '', 10, ?2)"#,
                    params![tid, now],
                )?;
            }
        }
        Ok(())
    };

    // Helper: set or clear postmeta / option override
    let set_override_meta = |tx: &rusqlite::Transaction, val: Option<&str>| -> Result<()> {
        if let Some(pid) = post_id {
            let meta_key = if slot == TemplateType::Header {
                "_np_header_override"
            } else {
                "_np_footer_override"
            };
            tx.execute(
                "DELETE FROM postmeta WHERE post_id = ?1 AND meta_key = ?2",
                params![pid, meta_key],
            )?;
            if let Some(v) = val {
                tx.execute(
                    "INSERT INTO postmeta (post_id, meta_key, meta_value) VALUES (?1, ?2, ?3)",
                    params![pid, meta_key, v],
                )?;
            }
        } else if let Some(rk) = route_key {
            let opt_name = format!("_np_{}_route_{}", slot_str, rk);
            tx.execute(
                "DELETE FROM options WHERE option_name = ?1",
                params![opt_name],
            )?;
            if let Some(v) = val {
                tx.execute(
                    "INSERT INTO options (option_name, option_value) VALUES (?1, ?2)",
                    params![opt_name, v],
                )?;
            }
        }
        Ok(())
    };

    if mode.is_empty() || mode.eq_ignore_ascii_case("inherit") {
        set_override_meta(tx, None)?;
        remove_specific_conditions(tx)?;
    } else if mode.eq_ignore_ascii_case("none") || mode.eq_ignore_ascii_case("hidden") {
        set_override_meta(tx, Some("none"))?;
        remove_specific_conditions(tx)?;
    } else if mode.eq_ignore_ascii_case("fallback") || mode.eq_ignore_ascii_case("theme_fallback") {
        set_override_meta(tx, Some("fallback"))?;
        remove_specific_conditions(tx)?;
    } else {
        let tpl_id_str = mode.strip_prefix("template:").unwrap_or(mode).trim();
        if let Ok(tpl_id) = tpl_id_str.parse::<i64>() {
            // Check if tpl_id had 0 conditions before (meaning implicit EntireSite)
            let prev_total_conds: i64 = tx
                .query_row(
                    "SELECT COUNT(*) FROM daypress_template_conditions WHERE template_id = ?1",
                    params![tpl_id],
                    |r| r.get(0),
                )
                .unwrap_or(0);

            set_override_meta(tx, None)?;
            remove_specific_conditions(tx)?;

            // Remove any unassigned sentinel (`exclude` `entire_site`) on the target template
            tx.execute(
                "DELETE FROM daypress_template_conditions WHERE template_id = ?1 AND condition_type = 'exclude' AND rule_target = 'entire_site'",
                params![tpl_id],
            )?;

            // If template previously had 0 conditions (implicit EntireSite) and we are adding a specific rule,
            // preserve its explicit EntireSite base rule so it doesn't disappear from other pages.
            if prev_total_conds == 0 && rule_target_str != "entire_site" {
                tx.execute(
                    r#"INSERT INTO daypress_template_conditions
                       (template_id, condition_type, rule_target, target_id, priority, created_at)
                       VALUES (?1, 'include', 'entire_site', '', 10, ?2)"#,
                    params![tpl_id, now],
                )?;
            }

            // Ensure target template is active & published
            tx.execute(
                "UPDATE daypress_templates SET is_active = 1, status = 'publish', updated_at = ?1 WHERE id = ?2",
                params![now, tpl_id],
            )?;

            let cond_priority = if post_id.is_some()
                || rule_target_str == "front_page"
                || rule_target_str == "not_found"
            {
                100
            } else {
                20
            };

            tx.execute(
                r#"INSERT INTO daypress_template_conditions
                   (template_id, condition_type, rule_target, target_id, priority, created_at)
                   VALUES (?1, 'include', ?2, ?3, ?4, ?5)"#,
                params![tpl_id, rule_target_str, target_id_str, cond_priority, now],
            )?;
        }
    }

    Ok(())
}
