use super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_information_requirement_decision_with_construct_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-listed-input-data-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(
        issue.code,
        "dmn.unsupported_information_requirement_decision"
    );
    assert!(issue.title.contains("required input data"));
    assert!(issue.summary.contains("<requiredInput>"));
    assert!(
        issue
            .why_it_failed
            .contains("can only bind an already-supplied local input-data alias")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("same-source input-data alias bind"))
    );
    assert!(issue.llm_fix_prompt.contains("<requiredInput>"));
    assert_eq!(
        issue.evidence["decision_snapshot"]["information_requirement_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["required_input_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["requirement_references"][0]["requirement_kind"],
        json!("informationRequirement")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["requirement_references"][0]["reference_kind"],
        json!("requiredInput")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["requirement_references"][0]["href"],
        json!("#InputData_1")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["required_decision_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["knowledge_requirement_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["authority_requirement_count"],
        json!(0)
    );
}

#[test]
fn dmn_linter_reports_knowledge_requirement_decision_with_construct_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-knowledge-requirement-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_knowledge_requirement_decision");
    assert!(issue.title.contains("required knowledge"));
    assert!(issue.summary.contains("<requiredKnowledge>"));
    assert!(
        issue
            .why_it_failed
            .contains("bounded parser now preserves one invocable `variable` / `encapsulatedLogic` placeholder contract")
    );
    assert!(issue.repair_guidance.iter().any(|step| {
        step.contains("runtime still does not execute preserved BKM invocable metadata")
    }));
    assert!(issue.llm_fix_prompt.contains("<requiredKnowledge>"));
    assert_eq!(
        issue.evidence["decision_snapshot"]["information_requirement_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["required_input_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["required_decision_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["knowledge_requirement_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["required_knowledge_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["requirement_references"][0]["requirement_kind"],
        json!("knowledgeRequirement")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["requirement_references"][0]["reference_kind"],
        json!("requiredKnowledge")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["requirement_references"][0]["href"],
        json!("#BKM_policy_source")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["authority_requirement_count"],
        json!(0)
    );
}

#[test]
fn dmn_linter_reports_authority_requirement_decision_with_construct_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-authority-requirement-decision-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_authority_requirement_decision");
    assert!(issue.title.contains("required authority"));
    assert!(issue.summary.contains("<requiredAuthority>"));
    assert!(
        issue
            .why_it_failed
            .contains("required-authority, knowledge-source, and any authority-linked decision or input references")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("<requiredDecision>"))
    );
    assert!(issue.llm_fix_prompt.contains("<authorityRequirement>"));
    assert_eq!(
        issue.evidence["decision_snapshot"]["information_requirement_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["required_input_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["required_decision_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["knowledge_requirement_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["required_knowledge_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["authority_requirement_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["required_authority_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["requirement_references"][0]["requirement_kind"],
        json!("authorityRequirement")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["requirement_references"][0]["reference_kind"],
        json!("requiredAuthority")
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["requirement_references"][0]["href"],
        json!("#KnowledgeSource_policy")
    );
}

#[test]
fn dmn_linter_reports_authority_requirement_decision_with_mixed_reference_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-authority-requirement-mixed-references-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_authority_requirement_decision");
    assert!(issue.title.contains("required authority"));
    assert!(issue.summary.contains("<requiredAuthority>"));
    assert!(issue.why_it_failed.contains(
        "authority-linked decision or input references still do not provide local executable rules"
    ));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("<requiredInput>"))
    );
    assert!(issue.llm_fix_prompt.contains("<authorityRequirement>"));
    assert_eq!(
        issue.evidence["decision_snapshot"]["authority_requirement_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["required_authority_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["required_decision_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["required_input_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["requirement_references"],
        json!([
            {
                "requirement_kind": "authorityRequirement",
                "reference_kind": "requiredAuthority",
                "href": "#KnowledgeSource_policy"
            },
            {
                "requirement_kind": "authorityRequirement",
                "reference_kind": "requiredDecision",
                "href": "#Decision_upstream"
            },
            {
                "requirement_kind": "authorityRequirement",
                "reference_kind": "requiredInput",
                "href": "#InputData_customer"
            }
        ])
    );
}
