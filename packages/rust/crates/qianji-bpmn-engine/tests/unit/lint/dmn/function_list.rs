use super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_function_definition_decision_with_construct_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-function-definition-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_function_definition_decision");
    assert!(issue.title.contains("function definition logic"));
    assert!(
        issue
            .why_it_failed
            .contains("direct functionDefinition decisions remain placeholder-only")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not inline or approximate function semantics"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not rewrite its direct `<functionDefinition>`")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_function_definition")
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
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["function_definition_count"],
        json!(1)
    );
    assert_eq!(issue.evidence["decision_snapshot"]["list_count"], json!(0));
}

#[test]
fn dmn_linter_reports_list_decision_with_construct_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source("versioned-list-decision-20191111.dmn"));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_list_decision");
    assert!(issue.title.contains("list logic"));
    assert!(
        issue
            .why_it_failed
            .contains("direct list decisions remain placeholder-only")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not flatten list items"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not rewrite its direct `<list>`")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_list")
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
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["function_definition_count"],
        json!(0)
    );
    assert_eq!(issue.evidence["decision_snapshot"]["list_count"], json!(1));
}
