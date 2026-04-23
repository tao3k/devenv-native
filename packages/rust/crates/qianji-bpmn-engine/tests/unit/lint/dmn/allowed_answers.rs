use super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_allowed_answers_decision_with_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-allowed-answers-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_allowed_answers_decision");
    assert!(issue.title.contains("allowed answers"));
    assert!(
        issue
            .why_it_failed
            .contains("non-executable output metadata only")
    );
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
        json!("Decision_allowed_answers")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["allowed_answers_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_table_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["literal_expression_count"],
        json!(0)
    );
    assert_eq!(issue.evidence["decision_snapshot"]["list_count"], json!(0));
}
