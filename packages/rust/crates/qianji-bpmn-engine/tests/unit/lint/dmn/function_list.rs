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
            .contains("function kind, formal-parameter, and body literal-expression metadata")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("decision_snapshot.function_definitions"))
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
    assert_eq!(
        issue.evidence["decision_snapshot"]["function_definitions"][0]["function_definition_id"],
        json!("function_definition_1")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["function_definitions"][0]["kind"],
        json!("FEEL")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["function_definitions"][0]["parameters"][0]["name"],
        json!("riskScore")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["function_definitions"][0]["parameters"][0]["type_ref"],
        json!("number")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["function_definitions"][0]["body"]["text"],
        json!("riskScore")
    );
    assert_eq!(issue.evidence["decision_snapshot"]["list_count"], json!(0));
}

#[test]
fn dmn_linter_accepts_bounded_list_decision() {
    let report = lint_dmn_source(&dmn_fixture_source("versioned-list-decision-20191111.dmn"));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn dmn_linter_reports_unsupported_list_item_with_fix_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-unsupported-list-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_list_expression_subset");
    assert!(issue.title.contains("list item"));
    assert!(
        issue
            .why_it_failed
            .contains("every direct `literalExpression` item")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("list item ordering"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not flatten the list into guessed decision-table rules")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_unsupported_list")
    );
    assert_eq!(
        issue.evidence["list_item_expression"],
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
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["function_definition_count"],
        json!(0)
    );
    assert_eq!(issue.evidence["decision_snapshot"]["list_count"], json!(1));
}

#[test]
fn dmn_linter_reports_unsupported_list_child_with_fix_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source("invalid-list-unsupported-child.dmn"));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_list_child");
    assert!(
        issue
            .why_it_failed
            .contains("direct lists made of direct `literalExpression` items")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("direct `<literalExpression>"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("Do not flatten nested boxed expressions")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_invalid_list_child")
    );
    assert_eq!(
        issue.evidence["operation"],
        json!("parse_dmn_list_unsupported_child")
    );
}
