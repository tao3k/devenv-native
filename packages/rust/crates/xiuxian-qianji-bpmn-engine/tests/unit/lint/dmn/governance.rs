use crate::lint::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_decision_maker_decision_with_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-decision-maker-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_decision_maker_decision");
    assert!(issue.title.contains("decision-maker"));
    assert!(issue.why_it_failed.contains("governance metadata only"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not invent rules"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not fabricate decision-table rules")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_decision_maker")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["allowed_answers_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_maker_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_owner_count"],
        json!(0)
    );
}

#[test]
fn dmn_linter_reports_decision_owner_decision_with_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-decision-owner-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_decision_owner_decision");
    assert!(issue.title.contains("decision-owner"));
    assert!(issue.why_it_failed.contains("governance metadata only"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not invent rules"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not fabricate decision-table rules")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_decision_owner")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["allowed_answers_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_maker_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_owner_count"],
        json!(1)
    );
}

#[test]
fn dmn_linter_reports_mixed_decision_governance_with_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-mixed-decision-governance-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(
        issue.code,
        "dmn.unsupported_mixed_decision_governance_decision"
    );
    assert!(issue.title.contains("maker and owner metadata"));
    assert!(issue.why_it_failed.contains("governance metadata only"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not invent rules"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not fabricate decision-table rules")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_mixed_decision_governance")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["allowed_answers_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_maker_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_owner_count"],
        json!(1)
    );
}
