use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};

#[test]
fn bpmn_linter_reports_duplicate_di_shape_id_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source("invalid-di-duplicate-shape-id.bpmn"));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.duplicate_di_id");
    assert!(issue.why_it_failed.contains("must remain unique"));
    assert!(issue.llm_fix_prompt.contains("every BPMN DI id is unique"));
    assert_eq!(issue.evidence["duplicate_di_id_count"], 1);
    assert_eq!(
        issue.evidence["duplicate_di_ids"][0]["duplicate_id"],
        "Shape_Duplicate"
    );
    assert_eq!(issue.evidence["duplicate_di_ids"][0]["occurrence_count"], 2);
    assert_eq!(
        issue.evidence["duplicate_di_ids"][0]["occurrences"][0]["element"],
        "BPMNShape"
    );
    assert_eq!(
        issue.evidence["duplicate_di_ids"][0]["occurrences"][1]["element"],
        "BPMNShape"
    );
}

#[test]
fn bpmn_linter_reports_duplicate_di_label_style_id_before_metadata_guidance() {
    let report = lint_bpmn_source(&bpmn_fixture_source(
        "invalid-di-duplicate-label-style-id.bpmn",
    ));

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(!report.ok);
    assert_eq!(report.issues.len(), 1);
    let issue = &report.issues[0];
    assert_eq!(issue.code, "bpmn.duplicate_di_id");
    assert_eq!(issue.evidence["duplicate_di_id_count"], 1);
    assert_eq!(
        issue.evidence["duplicate_di_ids"][0]["duplicate_id"],
        "Style_Duplicate"
    );
    assert_eq!(
        issue.evidence["duplicate_di_ids"][0]["occurrences"][0]["element"],
        "BPMNLabelStyle"
    );
    assert_eq!(
        issue.evidence["duplicate_di_ids"][0]["occurrences"][1]["element"],
        "BPMNLabelStyle"
    );
}
