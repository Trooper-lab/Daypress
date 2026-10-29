//! Priority Display Conditions Engine & Specificity Scoring.

use crate::theme_builder::models::{
    ConditionType, RequestContext, RuleTarget, TemplateCondition, TemplateType, ThemeTemplateRecord,
};

pub const SPECIFICITY_ENTIRE_SITE: i32 = 10;
pub const SPECIFICITY_POST_TYPE: i32 = 20;
pub const SPECIFICITY_ARCHIVE: i32 = 20;
pub const SPECIFICITY_TAXONOMY: i32 = 30;
pub const SPECIFICITY_SPECIFIC_ID: i32 = 40;
pub const SPECIFICITY_FRONT_PAGE: i32 = 40;
pub const SPECIFICITY_404: i32 = 40;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionMatch {
    NoMatch,
    Matched { specificity: i32 },
}

impl ConditionMatch {
    pub fn specificity(self) -> Option<i32> {
        match self {
            Self::Matched { specificity } => Some(specificity),
            Self::NoMatch => None,
        }
    }
}

/// Evaluates a single condition rule against a request context.
pub fn evaluate_condition(cond: &TemplateCondition, ctx: &RequestContext) -> ConditionMatch {
    match cond.rule_target {
        RuleTarget::EntireSite => ConditionMatch::Matched {
            specificity: SPECIFICITY_ENTIRE_SITE,
        },
        RuleTarget::FrontPage => {
            if ctx.is_front_page {
                ConditionMatch::Matched {
                    specificity: SPECIFICITY_FRONT_PAGE,
                }
            } else {
                ConditionMatch::NoMatch
            }
        }
        RuleTarget::AllPosts => {
            if ctx.post_type.as_deref() == Some("post") && !ctx.is_archive && !ctx.is_404 {
                ConditionMatch::Matched {
                    specificity: SPECIFICITY_POST_TYPE,
                }
            } else {
                ConditionMatch::NoMatch
            }
        }
        RuleTarget::AllPages => {
            if ctx.post_type.as_deref() == Some("page") && !ctx.is_archive && !ctx.is_404 {
                ConditionMatch::Matched {
                    specificity: SPECIFICITY_POST_TYPE,
                }
            } else {
                ConditionMatch::NoMatch
            }
        }
        RuleTarget::Post => {
            if let Some(post_id) = ctx.post_id {
                let matches_id = cond
                    .target_id
                    .parse::<i64>()
                    .map(|id| id == post_id)
                    .unwrap_or(false);
                if matches_id && ctx.post_type.as_deref() == Some("post") {
                    ConditionMatch::Matched {
                        specificity: SPECIFICITY_SPECIFIC_ID,
                    }
                } else {
                    ConditionMatch::NoMatch
                }
            } else {
                ConditionMatch::NoMatch
            }
        }
        RuleTarget::Page => {
            if let Some(post_id) = ctx.post_id {
                let matches_id = cond
                    .target_id
                    .parse::<i64>()
                    .map(|id| id == post_id)
                    .unwrap_or(false);
                if matches_id && ctx.post_type.as_deref() == Some("page") {
                    ConditionMatch::Matched {
                        specificity: SPECIFICITY_SPECIFIC_ID,
                    }
                } else {
                    ConditionMatch::NoMatch
                }
            } else {
                ConditionMatch::NoMatch
            }
        }
        RuleTarget::Category | RuleTarget::Tag => {
            let target_term_id = cond.target_id.parse::<i64>().ok();
            let target_kind = cond.rule_target.as_str();
            let matches_term = ctx.taxonomy_terms.iter().any(|term| {
                if !term.taxonomy.eq_ignore_ascii_case(target_kind) {
                    return false;
                }
                if let Some(tid) = target_term_id {
                    term.term_id == tid
                } else {
                    term.slug.eq_ignore_ascii_case(&cond.target_id)
                }
            });

            if matches_term {
                ConditionMatch::Matched {
                    specificity: SPECIFICITY_TAXONOMY,
                }
            } else {
                ConditionMatch::NoMatch
            }
        }
        RuleTarget::Author => {
            if let Some(author_id) = ctx.author_id {
                let matches = cond
                    .target_id
                    .parse::<i64>()
                    .map(|id| id == author_id)
                    .unwrap_or(false);
                if matches {
                    ConditionMatch::Matched {
                        specificity: SPECIFICITY_TAXONOMY,
                    }
                } else {
                    ConditionMatch::NoMatch
                }
            } else {
                ConditionMatch::NoMatch
            }
        }
        RuleTarget::Archive => {
            if ctx.is_archive {
                ConditionMatch::Matched {
                    specificity: SPECIFICITY_ARCHIVE,
                }
            } else {
                ConditionMatch::NoMatch
            }
        }
        RuleTarget::NotFound => {
            if ctx.is_404 {
                ConditionMatch::Matched {
                    specificity: SPECIFICITY_404,
                }
            } else {
                ConditionMatch::NoMatch
            }
        }
    }
}

/// Evaluates all conditions of a template and computes its rank score (priority, specificity, id).
pub fn evaluate_template(
    template: &ThemeTemplateRecord,
    ctx: &RequestContext,
) -> Option<(i32, i32, i64)> {
    if !template.is_active || template.status != "publish" {
        return None;
    }

    let mut has_includes = false;
    let mut max_include_specificity = 0;

    // Phase 1 & 2: Evaluate conditions
    for cond in &template.conditions {
        let is_exclude = cond.condition_type == ConditionType::Exclude;
        let m = evaluate_condition(cond, ctx);

        if is_exclude {
            if m != ConditionMatch::NoMatch {
                // Exclusion matches -> Strictly overrides all inclusions
                return None;
            }
        } else {
            has_includes = true;
            if let Some(spec) = m.specificity() {
                if spec > max_include_specificity {
                    max_include_specificity = spec;
                }
            }
        }
    }

    // If template has conditions configured, at least one include must match
    if has_includes && max_include_specificity == 0 {
        return None;
    }

    // Default template with zero conditions acts as EntireSite default (specificity 10)
    let final_spec = if max_include_specificity > 0 {
        max_include_specificity
    } else {
        SPECIFICITY_ENTIRE_SITE
    };

    Some((template.priority, final_spec, template.id))
}

/// Resolves the winning active template for a given template type.
pub fn resolve_template_for_type(
    templates: &[ThemeTemplateRecord],
    expected_type: TemplateType,
    ctx: &RequestContext,
) -> Option<ThemeTemplateRecord> {
    templates
        .iter()
        .filter(|t| t.template_type == expected_type)
        .filter_map(|t| evaluate_template(t, ctx).map(|score| (score, t.clone())))
        .max_by_key(|(score, _)| *score)
        .map(|(_, t)| t)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme_builder::models::*;
    use daypress_ast::DaypressLayout;

    fn make_test_template(
        id: i64,
        priority: i32,
        conditions: Vec<TemplateCondition>,
    ) -> ThemeTemplateRecord {
        ThemeTemplateRecord {
            id,
            title: format!("Template {}", id),
            template_type: TemplateType::Header,
            status: "publish".to_string(),
            layout: DaypressLayout::default(),
            rendered_html: None,
            settings: TemplateSettings::default(),
            priority,
            is_active: true,
            conditions,
            created_at: "2026-10-23T00:00:00Z".to_string(),
            updated_at: "2026-10-23T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_specificity_hierarchy() {
        let ctx = RequestContext::for_post(42, "post", false, vec![]);

        let t_entire_site = make_test_template(
            1,
            10,
            vec![TemplateCondition {
                id: Some(1),
                template_id: 1,
                condition_type: ConditionType::Include,
                rule_target: RuleTarget::EntireSite,
                target_id: String::new(),
                priority: 10,
                created_at: None,
            }],
        );

        let t_all_posts = make_test_template(
            2,
            10,
            vec![TemplateCondition {
                id: Some(2),
                template_id: 2,
                condition_type: ConditionType::Include,
                rule_target: RuleTarget::AllPosts,
                target_id: String::new(),
                priority: 10,
                created_at: None,
            }],
        );

        let t_specific_post = make_test_template(
            3,
            10,
            vec![TemplateCondition {
                id: Some(3),
                template_id: 3,
                condition_type: ConditionType::Include,
                rule_target: RuleTarget::Post,
                target_id: "42".to_string(),
                priority: 10,
                created_at: None,
            }],
        );

        let templates = vec![t_entire_site, t_all_posts, t_specific_post];
        let winner = resolve_template_for_type(&templates, TemplateType::Header, &ctx)
            .expect("Winner found");
        assert_eq!(
            winner.id, 3,
            "Specific post ID (40) must beat AllPosts (20) and EntireSite (10)"
        );
    }

    #[test]
    fn test_exclusion_strictly_overrides_inclusion() {
        let ctx = RequestContext::for_post(42, "post", false, vec![]);

        let tpl = make_test_template(
            1,
            10,
            vec![
                TemplateCondition {
                    id: Some(1),
                    template_id: 1,
                    condition_type: ConditionType::Include,
                    rule_target: RuleTarget::EntireSite,
                    target_id: String::new(),
                    priority: 10,
                    created_at: None,
                },
                TemplateCondition {
                    id: Some(2),
                    template_id: 1,
                    condition_type: ConditionType::Exclude,
                    rule_target: RuleTarget::Post,
                    target_id: "42".to_string(),
                    priority: 10,
                    created_at: None,
                },
            ],
        );

        let eval = evaluate_template(&tpl, &ctx);
        assert_eq!(eval, None, "Excluded post 42 must disqualify template");

        let ctx_other = RequestContext::for_post(43, "post", false, vec![]);
        let eval_other = evaluate_template(&tpl, &ctx_other);
        assert!(
            eval_other.is_some(),
            "Template must match post 43 via EntireSite"
        );
    }

    #[test]
    fn test_user_priority_override() {
        let ctx = RequestContext::for_post(42, "post", false, vec![]);

        let t_high_priority = make_test_template(
            1,
            50,
            vec![TemplateCondition {
                id: Some(1),
                template_id: 1,
                condition_type: ConditionType::Include,
                rule_target: RuleTarget::EntireSite,
                target_id: String::new(),
                priority: 10,
                created_at: None,
            }],
        );

        let t_specific = make_test_template(
            2,
            10,
            vec![TemplateCondition {
                id: Some(2),
                template_id: 2,
                condition_type: ConditionType::Include,
                rule_target: RuleTarget::Post,
                target_id: "42".to_string(),
                priority: 10,
                created_at: None,
            }],
        );

        let templates = vec![t_high_priority, t_specific];
        let winner = resolve_template_for_type(&templates, TemplateType::Header, &ctx)
            .expect("Winner found");
        assert_eq!(
            winner.id, 1,
            "User priority 50 must override default priority 10 even with lower specificity"
        );
    }

    #[test]
    fn test_tie_break_on_highest_id() {
        let ctx = RequestContext::for_front_page();

        let t_old = make_test_template(
            1,
            10,
            vec![TemplateCondition {
                id: Some(1),
                template_id: 1,
                condition_type: ConditionType::Include,
                rule_target: RuleTarget::FrontPage,
                target_id: String::new(),
                priority: 10,
                created_at: None,
            }],
        );

        let t_new = make_test_template(
            2,
            10,
            vec![TemplateCondition {
                id: Some(2),
                template_id: 2,
                condition_type: ConditionType::Include,
                rule_target: RuleTarget::FrontPage,
                target_id: String::new(),
                priority: 10,
                created_at: None,
            }],
        );

        let templates = vec![t_old, t_new];
        let winner = resolve_template_for_type(&templates, TemplateType::Header, &ctx)
            .expect("Winner found");
        assert_eq!(
            winner.id, 2,
            "Equal priority and specificity must tie-break on highest ID"
        );
    }
}
