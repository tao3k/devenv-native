use super::{LintDomain, assert_lint_json_snapshot, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_invalid_standard_loop_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-standard-loop-missing-limit.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    assert_lint_json_snapshot("bpmn_standard_loop_missing_limit_lint_report", &report);
}

#[test]
fn bpmn_linter_reports_parallel_multi_instance_completion_condition_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-multi-instance-deferred.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_loop_configuration");
    assert!(issue.summary.contains("review"));
    assert!(issue.llm_fix_prompt.contains("completionCondition"));
}

#[test]
fn bpmn_linter_reports_missing_multi_instance_cardinality_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-sequential-multi-instance-missing-cardinality.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_loop_configuration");
    assert!(issue.summary.contains("review"));
    assert!(issue.llm_fix_prompt.contains("loopCardinality"));
    assert!(issue.llm_fix_prompt.contains("loopDataInputRef"));
}

#[test]
fn bpmn_linter_reports_in_place_multi_instance_output_binding_with_llm_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-sequential-multi-instance-in-place-output.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.unsupported_loop_configuration");
    assert!(issue.summary.contains("review"));
    assert!(issue.llm_fix_prompt.contains("loopDataOutputRef"));
    assert!(
        issue
            .llm_fix_prompt
            .contains("different from the input path")
    );
}
