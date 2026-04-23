use super::super::{LintDomain, dmn_fixture_source, lint_dmn_source};
use serde_json::json;

#[test]
fn dmn_linter_reports_required_decision_target_with_specific_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source(
        "versioned-required-decision-dependency-20191111.dmn",
    ));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(
        issue.code,
        "dmn.unsupported_information_requirement_decision"
    );
    assert!(issue.title.contains("another decision"));
    assert!(issue.summary.contains("<requiredDecision>"));
    assert!(issue.why_it_failed.contains(
        "can recurse through direct same-source `<requiredDecision>` edges only after the current decision already contributes local executable rules"
    ));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("upstream decision logic"))
    );
    assert!(issue.llm_fix_prompt.contains("<requiredDecision>"));
    assert_eq!(
        issue.evidence["decision_snapshot"]["information_requirement_count"],
        json!(1)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["required_input_count"],
        json!(0)
    );
    assert_eq!(
        issue.evidence["decision_snapshot"]["required_decision_count"],
        json!(1)
    );
}
