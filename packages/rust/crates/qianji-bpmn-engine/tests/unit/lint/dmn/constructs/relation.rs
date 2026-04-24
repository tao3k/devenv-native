use super::super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_accepts_bounded_relation_decision() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-relation-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn dmn_linter_reports_unsupported_relation_cell_with_fix_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-unsupported-relation-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_relation_expression_subset");
    assert!(issue.title.contains("relation cell"));
    assert!(
        issue
            .why_it_failed
            .contains("every row matches the relation column count")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("column order"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not flatten the relation into guessed decision-table rules")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_unsupported_relation")
    );
    assert_eq!(
        issue.evidence["relation_cell_expression"],
        json!("applicant.age * 2")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["literal_expression_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["context_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["invocation_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["relation_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["function_definition_count"],
        json!(0)
    );
    assert_eq!(issue.evidence["decision_snapshot"]["list_count"], json!(0));
}

#[test]
fn dmn_linter_reports_unsupported_relation_child_with_fix_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "invalid-relation-unsupported-child.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_relation_child");
    assert!(issue.title.contains("relation"));
    assert!(
        issue
            .why_it_failed
            .contains("direct `row` children whose cells")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("row cell"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("Do not flatten relation rows")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_invalid_relation_child")
    );
    assert_eq!(
        issue.evidence["operation"],
        json!("parse_dmn_relation_unsupported_child")
    );
}
