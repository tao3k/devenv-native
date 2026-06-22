use crate::lint::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_invocation_decision_without_local_bkm_target() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-invocation-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_invocation_contract");
    assert!(issue.title.contains("local callable subset"));
    assert!(
        issue
            .why_it_failed
            .contains("same-source top-level `businessKnowledgeModel`")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("same-source top-level `businessKnowledgeModel`"))
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("Do not fabricate or inline broader BKM semantics"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("keep its direct `<invocation>` honest")
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

#[test]
fn dmn_linter_accepts_supported_local_bkm_invocation_contract() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-local-bkm-invocation-runtime-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(report.ok);
    assert!(report.issues.is_empty());
}

#[test]
fn dmn_linter_reports_invocation_outside_declared_required_knowledge_contract() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-outside-required-knowledge-runtime-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_invocation_contract");
    assert!(issue.why_it_failed.contains("requiredKnowledge"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| { step.contains("requiredKnowledge") })
    );
}
