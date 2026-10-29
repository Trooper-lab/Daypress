//! Adversarial Stress Tests for Daypress Theme Builder Display Conditions Engine.
//!
//! Evaluates conflict resolution, tie-breaking, malformed conditions, and edge cases.

use daypress_ast::DaypressLayout;
use daypress_server::theme_builder::conditions::ConditionMatch;
use daypress_server::theme_builder::models::*;
use daypress_server::theme_builder::ThemeTemplateRepository;
use daypress_server::theme_builder::{
    evaluate_condition, evaluate_template, resolve_template_for_type,
};
use daypress_server::Database;

fn make_tpl(id: i64, priority: i32, conditions: Vec<TemplateCondition>) -> ThemeTemplateRecord {
    ThemeTemplateRecord {
        id,
        title: format!("Adversarial Template {}", id),
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

fn cond(t: ConditionType, r: RuleTarget, target_id: &str, priority: i32) -> TemplateCondition {
    TemplateCondition {
        id: None,
        template_id: 1,
        condition_type: t,
        rule_target: r,
        target_id: target_id.to_string(),
        priority,
        created_at: None,
    }
}

// =========================================================================
// 1. Conflict Resolution & Exclusion Stress Tests
// =========================================================================

#[test]
fn test_conflict_intra_template_exclusion_overrides_high_priority_inclusion() {
    let ctx = RequestContext::for_post(42, "post", false, vec![]);

    // Template A has priority 100 with EntireSite inclusion, but ALSO an exclusion for post 42
    let tpl_a = make_tpl(
        1,
        100,
        vec![
            cond(ConditionType::Include, RuleTarget::EntireSite, "", 10),
            cond(ConditionType::Exclude, RuleTarget::Post, "42", 10),
        ],
    );

    let eval_a = evaluate_template(&tpl_a, &ctx);
    assert!(
        eval_a.is_none(),
        "Intra-template exclusion must strictly disqualify Template A on post 42 despite priority 100"
    );

    // On post 43, Template A must match cleanly
    let ctx_other = RequestContext::for_post(43, "post", false, vec![]);
    let eval_other = evaluate_template(&tpl_a, &ctx_other);
    assert_eq!(
        eval_other,
        Some((100, 10, 1)),
        "Template A must match post 43 with priority 100 and EntireSite specificity 10"
    );
}

#[test]
fn test_conflict_cross_template_exclusion_isolation() {
    let ctx = RequestContext::for_post(42, "post", false, vec![]);

    // Template A: Global Site Header (priority 100, EntireSite)
    let tpl_a = make_tpl(
        1,
        100,
        vec![cond(ConditionType::Include, RuleTarget::EntireSite, "", 10)],
    );

    // Template B: Specialized Post Header with exclusion for post 42 (priority 10, AllPosts, Exclude Post 42)
    let tpl_b = make_tpl(
        2,
        10,
        vec![
            cond(ConditionType::Include, RuleTarget::AllPosts, "", 20),
            cond(ConditionType::Exclude, RuleTarget::Post, "42", 10),
        ],
    );

    let templates = vec![tpl_a, tpl_b];

    // On Post 42: Template B is disqualified by its exclusion. Template A remains valid as fallback!
    let winner = resolve_template_for_type(&templates, TemplateType::Header, &ctx);
    assert!(
        winner.is_some(),
        "Fallback Template A must resolve on post 42"
    );
    assert_eq!(
        winner.unwrap().id,
        1,
        "Template B's exclusion rule must NOT disqualify Template A; Template B only disqualifies itself"
    );

    // On Post 43: Template B matches AllPosts (specificity 20) vs Template A (specificity 10)
    // BUT Template A has priority 100 vs Template B priority 10!
    let ctx_43 = RequestContext::for_post(43, "post", false, vec![]);
    let winner_43 = resolve_template_for_type(&templates, TemplateType::Header, &ctx_43).unwrap();
    assert_eq!(
        winner_43.id, 1,
        "User priority 100 on Template A overrides Template B's higher specificity (20)"
    );
}

#[test]
fn test_conflict_specificity_wins_when_priority_equal() {
    let ctx = RequestContext::for_post(42, "post", false, vec![]);

    // Both templates have equal priority 10
    let tpl_general = make_tpl(
        1,
        10,
        vec![cond(ConditionType::Include, RuleTarget::EntireSite, "", 10)],
    );
    let tpl_specific = make_tpl(
        2,
        10,
        vec![cond(ConditionType::Include, RuleTarget::Post, "42", 10)],
    );

    let winner =
        resolve_template_for_type(&[tpl_general, tpl_specific], TemplateType::Header, &ctx)
            .unwrap();
    assert_eq!(
        winner.id,
        2,
        "With equal priority (10), Specific Post ID (specificity 40) must beat EntireSite (specificity 10)"
    );
}

// =========================================================================
// 2. Tie-Breaking Stress Tests (Equal Priority & Specificity)
// =========================================================================

#[test]
fn test_tie_breaking_equal_priority_and_specificity_highest_id_wins() {
    let ctx = RequestContext::for_front_page();

    // 5 templates with identical priority (10) and specificity (FrontPage = 40)
    let ids = vec![5, 99, 12, 3, 42];
    let templates: Vec<ThemeTemplateRecord> = ids
        .into_iter()
        .map(|id| {
            make_tpl(
                id,
                10,
                vec![cond(ConditionType::Include, RuleTarget::FrontPage, "", 10)],
            )
        })
        .collect();

    let winner = resolve_template_for_type(&templates, TemplateType::Header, &ctx);
    assert!(winner.is_some());
    assert_eq!(
        winner.unwrap().id,
        99,
        "Tie-breaker must deterministically select the highest template ID (99)"
    );
}

#[test]
fn test_tie_breaking_multi_category_match() {
    let terms = vec![
        TaxonomyTermRef {
            taxonomy: "category".to_string(),
            term_id: 101,
            slug: "news".to_string(),
        },
        TaxonomyTermRef {
            taxonomy: "category".to_string(),
            term_id: 102,
            slug: "tech".to_string(),
        },
    ];
    let ctx = RequestContext::for_post(42, "post", false, terms);

    // Template A matches "news" (spec 30, priority 15, id 50)
    let tpl_a = make_tpl(
        50,
        15,
        vec![cond(
            ConditionType::Include,
            RuleTarget::Category,
            "news",
            15,
        )],
    );

    // Template B matches "tech" (spec 30, priority 15, id 60)
    let tpl_b = make_tpl(
        60,
        15,
        vec![cond(
            ConditionType::Include,
            RuleTarget::Category,
            "tech",
            15,
        )],
    );

    let winner = resolve_template_for_type(&[tpl_a, tpl_b], TemplateType::Header, &ctx).unwrap();
    assert_eq!(
        winner.id,
        60,
        "When post has two matching categories with identical priority and specificity, highest ID (60) wins"
    );
}

// =========================================================================
// 3. Malformed, Empty, and Corrupted Condition Records Stress Tests
// =========================================================================

#[test]
fn test_malformed_target_id_post_and_page() {
    let ctx = RequestContext::for_post(42, "post", false, vec![]);

    // Case 1: Empty target_id
    let cond_empty = cond(ConditionType::Include, RuleTarget::Post, "", 10);
    assert_eq!(
        evaluate_condition(&cond_empty, &ctx),
        ConditionMatch::NoMatch
    );

    // Case 2: Non-numeric alphabetic string
    let cond_alpha = cond(ConditionType::Include, RuleTarget::Post, "forty-two", 10);
    assert_eq!(
        evaluate_condition(&cond_alpha, &ctx),
        ConditionMatch::NoMatch
    );

    // Case 3: Overflow numeric string
    let cond_overflow = cond(
        ConditionType::Include,
        RuleTarget::Post,
        "999999999999999999999999999999",
        10,
    );
    assert_eq!(
        evaluate_condition(&cond_overflow, &ctx),
        ConditionMatch::NoMatch
    );

    // Case 4: Whitespace padded numeric string
    let cond_whitespace = cond(ConditionType::Include, RuleTarget::Post, " 42 ", 10);
    assert_eq!(
        evaluate_condition(&cond_whitespace, &ctx),
        ConditionMatch::NoMatch
    );

    // Case 5: Negative integer
    let cond_neg = cond(ConditionType::Include, RuleTarget::Post, "-42", 10);
    assert_eq!(evaluate_condition(&cond_neg, &ctx), ConditionMatch::NoMatch);

    // Case 6: Unicode emojis and special characters
    let cond_unicode = cond(ConditionType::Include, RuleTarget::Post, "🦀_42_!@#$", 10);
    assert_eq!(
        evaluate_condition(&cond_unicode, &ctx),
        ConditionMatch::NoMatch
    );
}

#[test]
fn test_taxonomy_case_insensitivity_and_kind_isolation() {
    let terms = vec![
        TaxonomyTermRef {
            taxonomy: "category".to_string(),
            term_id: 10,
            slug: "Featured-Stories".to_string(),
        },
        TaxonomyTermRef {
            taxonomy: "tag".to_string(),
            term_id: 20,
            slug: "rust-lang".to_string(),
        },
    ];
    let ctx = RequestContext::for_post(1, "post", false, terms);

    // Matching slug with mixed case
    let cond_cat_slug = cond(
        ConditionType::Include,
        RuleTarget::Category,
        "featured-stories",
        10,
    );
    assert_eq!(
        evaluate_condition(&cond_cat_slug, &ctx),
        ConditionMatch::Matched { specificity: 30 }
    );

    // Matching numeric term_id as string
    let cond_cat_id = cond(ConditionType::Include, RuleTarget::Category, "10", 10);
    assert_eq!(
        evaluate_condition(&cond_cat_id, &ctx),
        ConditionMatch::Matched { specificity: 30 }
    );

    // Taxonomy kind isolation: Category condition with Tag slug must NOT match
    let cond_wrong_kind = cond(
        ConditionType::Include,
        RuleTarget::Category,
        "rust-lang",
        10,
    );
    assert_eq!(
        evaluate_condition(&cond_wrong_kind, &ctx),
        ConditionMatch::NoMatch,
        "Category rule must not match a Tag term with the same slug"
    );

    // Tag condition matching tag
    let cond_tag = cond(ConditionType::Include, RuleTarget::Tag, "rust-lang", 10);
    assert_eq!(
        evaluate_condition(&cond_tag, &ctx),
        ConditionMatch::Matched { specificity: 30 }
    );
}

#[test]
fn test_empty_conditions_and_unmatched_includes() {
    let ctx = RequestContext::for_post(42, "post", false, vec![]);

    // Template with zero conditions acts as EntireSite fallback (specificity 10)
    let tpl_empty = make_tpl(1, 10, vec![]);
    assert_eq!(
        evaluate_template(&tpl_empty, &ctx),
        Some((10, 10, 1)),
        "Zero-condition template must act as default fallback with specificity 10"
    );

    // Template with include condition that DOES NOT match context must return None
    let tpl_unmatched = make_tpl(
        2,
        50,
        vec![cond(ConditionType::Include, RuleTarget::Post, "999", 50)],
    );
    assert!(
        evaluate_template(&tpl_unmatched, &ctx).is_none(),
        "Template with non-matching include condition must be disqualified regardless of priority"
    );

    // Inactive template must return None
    let mut tpl_inactive = make_tpl(
        3,
        100,
        vec![cond(ConditionType::Include, RuleTarget::EntireSite, "", 10)],
    );
    tpl_inactive.is_active = false;
    assert!(
        evaluate_template(&tpl_inactive, &ctx).is_none(),
        "Inactive template must return None"
    );

    // Draft template must return None
    let mut tpl_draft = make_tpl(
        4,
        100,
        vec![cond(ConditionType::Include, RuleTarget::EntireSite, "", 10)],
    );
    tpl_draft.status = "draft".to_string();
    assert!(
        evaluate_template(&tpl_draft, &ctx).is_none(),
        "Draft template must return None"
    );
}

#[test]
fn test_author_condition_evaluation() {
    let mut ctx = RequestContext::for_post(42, "post", false, vec![]);
    ctx.author_id = Some(7);

    let cond_author_match = cond(ConditionType::Include, RuleTarget::Author, "7", 10);
    assert_eq!(
        evaluate_condition(&cond_author_match, &ctx),
        ConditionMatch::Matched { specificity: 30 }
    );

    let cond_author_mismatch = cond(ConditionType::Include, RuleTarget::Author, "8", 10);
    assert_eq!(
        evaluate_condition(&cond_author_mismatch, &ctx),
        ConditionMatch::NoMatch
    );

    let cond_author_malformed = cond(ConditionType::Include, RuleTarget::Author, "seven", 10);
    assert_eq!(
        evaluate_condition(&cond_author_malformed, &ctx),
        ConditionMatch::NoMatch
    );
}

// =========================================================================
// 4. SQLite Transactional Cascading & Dual Condition Integrity
// =========================================================================

#[test]
fn test_sqlite_conditions_cascade_on_template_deletion() {
    let db = Database::new_in_memory().expect("Failed in-memory DB");
    let repo = ThemeTemplateRepository::from_database(&db);

    let created = repo
        .create(&CreateTemplateRequest {
            title: "Cascade Test Header".to_string(),
            template_type: TemplateType::Header,
            status: Some("publish".to_string()),
            layout: None,
            layout_json: None,
            settings: None,
            priority: Some(10),
            is_active: Some(true),
            conditions: Some(vec![
                CreateConditionDto {
                    id: None,
                    condition_type: ConditionType::Include,
                    rule_target: RuleTarget::EntireSite,
                    target_id: String::new(),
                    priority: 10,
                },
                CreateConditionDto {
                    id: None,
                    condition_type: ConditionType::Exclude,
                    rule_target: RuleTarget::Post,
                    target_id: "42".to_string(),
                    priority: 20,
                },
            ]),
        })
        .unwrap();

    // Verify conditions exist
    let conditions = repo.get_conditions(created.id).unwrap();
    assert_eq!(conditions.len(), 2);

    // Delete template with force=true
    let deleted = repo.delete(created.id, true).unwrap();
    assert!(deleted);

    // Verify conditions cascade deleted in SQLite
    let remaining_conds = repo.get_conditions(created.id).unwrap();
    assert!(
        remaining_conds.is_empty(),
        "All condition rows must be deleted via ON DELETE CASCADE in SQLite"
    );
}
