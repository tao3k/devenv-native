use super::{LintDomain, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_transaction_cancel_end_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-transaction-cancel-end.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_transaction_configuration");
    assert!(issue.summary.contains("payment_tx"));
    assert!(
        issue
            .summary
            .contains("matching parent interrupting cancel boundary")
    );
    assert!(issue.why_it_failed.contains("both sides of the path"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("boundaryEvent"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("parent interrupting `boundaryEvent`")
    );
}

#[test]
fn bpmn_linter_reports_transaction_error_end_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-transaction-error-end.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_transaction_configuration");
    assert!(issue.summary.contains("payment_tx"));
    assert!(
        issue
            .summary
            .contains("matching parent interrupting error boundary")
    );
    assert!(issue.why_it_failed.contains("errorRef"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("errorRef"))
    );
    assert!(issue.llm_fix_prompt.contains("<bpmn:errorEventDefinition>"));
}

#[test]
fn bpmn_linter_reports_transaction_missing_end_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-transaction-missing-end.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_subprocess_configuration");
    assert!(issue.summary.contains("payment_tx"));
    assert!(issue.summary.contains("transaction body"));
    assert!(issue.why_it_failed.contains("bounded transaction shell"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("<bpmn:endEvent>"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("bounded `<bpmn:transaction>` body")
    );
}

#[test]
fn bpmn_linter_reports_transaction_duplicate_cancel_boundaries_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-transaction-multiple-cancel-boundaries.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_boundary_configuration");
    assert!(issue.title.contains("more than one cancel boundary"));
    assert!(issue.summary.contains("tx_cancel_boundary_b"));
    assert!(
        issue
            .why_it_failed
            .contains("one interrupting cancel boundary")
    );
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("one interrupting cancel boundary"))
    );
    assert!(
        issue
            .llm_fix_prompt
            .contains("second transaction cancel boundary")
    );
}
