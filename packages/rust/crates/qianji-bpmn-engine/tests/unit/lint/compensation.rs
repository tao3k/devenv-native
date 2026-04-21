use super::{LintDomain, assert_lint_json_snapshot, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_missing_compensation_handler_marker_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-transaction-compensation-missing-marker.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_compensation_configuration");
    assert!(issue.summary.contains("tx_refund"));
    assert!(issue.why_it_failed.contains("isForCompensation"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("isForCompensation"))
    );
    assert!(issue.llm_fix_prompt.contains("compensateEventDefinition"));
    assert_lint_json_snapshot("bpmn_compensation_missing_marker_lint_report", &report);
}
