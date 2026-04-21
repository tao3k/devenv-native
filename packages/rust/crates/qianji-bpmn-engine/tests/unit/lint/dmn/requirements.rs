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
            .contains("required-input references identify upstream data dependencies")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("inline upstream input binding semantics"))
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
            .contains("required-knowledge and business-knowledge-model references")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("business-knowledge-model semantics"))
    );
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
            .contains("required-authority and knowledge-source references")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("`knowledgeSource` metadata"))
    );
    assert!(issue.llm_fix_prompt.contains("<requiredAuthority>"));
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
}
