use crate::lint::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_accepts_bounded_literal_expression_decision() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-literal-expression-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn dmn_linter_reports_unsupported_literal_expression_subset_with_fix_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-unsupported-literal-expression-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_literal_expression_subset");
    assert!(issue.title.contains("literal expression"));
    assert!(
        issue
            .why_it_failed
            .contains("variable path, or one whitespace-delimited numeric path operation")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("path + number"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("Do not approximate broader FEEL expressions")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_unsupported_literal_expression")
    );
    assert_eq!(
        issue.evidence["literal_expression"],
        json!("applicant.age * 2")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_table_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["literal_expression_count"],
        json!(1)
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
    assert_eq!(issue.evidence["decision_snapshot"]["list_count"], json!(0));
}

#[test]
fn dmn_linter_accepts_bounded_context_decision() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-context-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn dmn_linter_reports_unsupported_context_entry_with_fix_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-unsupported-context-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_context_expression_subset");
    assert!(issue.title.contains("context entry"));
    assert!(
        issue
            .why_it_failed
            .contains("unnamed result entries appear only as the final entry")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("prior context variable"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not flatten the context into guessed decision-table rules")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_unsupported_context")
    );
    assert_eq!(
        issue.evidence["context_entry_expression"],
        json!("applicant.age * 2")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["literal_expression_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["context_count"],
        json!(1)
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
    assert_eq!(issue.evidence["decision_snapshot"]["list_count"], json!(0));
}

#[test]
fn dmn_linter_reports_unsupported_context_child_with_fix_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source("invalid-context-unsupported-child.dmn"));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_context_child");
    assert!(issue.title.contains("context"));
    assert!(
        issue
            .why_it_failed
            .contains("direct `contextEntry` children")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("variable name"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("Do not flatten context entries")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_invalid_context_child")
    );
    assert_eq!(
        issue.evidence["operation"],
        json!("parse_dmn_context_unsupported_child")
    );
}
