use super::{LintDomain, dmn_fixture_source, lint_dmn_source};

#[test]
fn dmn_linter_reports_multiple_decisions_with_llm_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source("invalid-multiple-decisions.dmn"));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    super::assert_lint_json_snapshot("dmn_multiple_decisions_lint_report", &report);
}

#[test]
fn dmn_linter_reports_unsupported_unary_test_with_llm_guidance() {
    let report = lint_dmn_source(&dmn_fixture_source("invalid-unsupported-unary-test.dmn"));

    assert_eq!(report.domain, LintDomain::Dmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "dmn.unsupported_unary_test");
    assert!(
        issue
            .summary
            .contains("date and time(\"2026-04-20T09:00:00Z\")")
    );
    assert!(
        issue
            .why_it_failed
            .contains("date and time(\"YYYY-MM-DDTHH:MM:SS\")")
    );
    assert!(issue.why_it_failed.contains("ISO local datetime ranges"));
    assert!(
        issue
            .repair_guidance
            .iter()
            .any(|step| step.contains("timezone-aware datetime forms"))
    );
    assert!(issue.llm_fix_prompt.contains(
        "date and time(\"2026-01-01T09:00:00\") <= ? < date and time(\"2026-01-01T17:00:00\")"
    ));
}
