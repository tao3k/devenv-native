use super::super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_invocation_decision_with_construct_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-invocation-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_invocation_decision");
    assert!(issue.title.contains("invocation logic"));
    assert!(
        issue
            .why_it_failed
            .contains("invocation function-expression and binding metadata")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("decision_snapshot.invocations"))
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not inline or fabricate invoked logic"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("do not rewrite its direct `<invocation>`")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["decision_id"],
        json!("Decision_invocation")
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
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["invocations"][0]["invocation_id"],
        json!("invocation_1")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["invocations"][0]["invoked_expression"]["text"],
        json!("scoreCard")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["invocations"][0]["bindings"][0]["parameter"]["name"],
        json!("age")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["invocations"][0]["bindings"][0]["argument"]["text"],
        json!("applicant.age")
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
