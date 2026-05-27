use crate::lint::{LintDomain, bpmn_fixture_source, lint_bpmn_source};
use xiuxian_qianji_bpmn_engine::snapshot_bpmn_source;

#[test]
fn bpmn_linter_preserves_artifact_metadata_surface() {
    let source = bpmn_fixture_source("metadata-artifacts.bpmn");
    let report = lint_bpmn_source(&source);

    assert_eq!(report.domain, LintDomain::Bpmn);
    assert!(
        report.ok,
        "artifact metadata should lint cleanly as passive metadata: {report:?}"
    );
    assert!(report.issues.is_empty());

    let snapshot = snapshot_bpmn_source(&source)
        .unwrap_or_else(|error| panic!("artifact fixture should snapshot: {error}"));
    let collaboration = &snapshot.collaborations[0];
    assert_eq!(collaboration.associations.len(), 1);
    assert_eq!(collaboration.groups.len(), 1);
    assert_eq!(collaboration.text_annotations.len(), 1);
    assert_eq!(
        collaboration.associations[0].source_ref.as_deref(),
        Some("TextAnnotation_Collaboration")
    );
    assert_eq!(
        collaboration.groups[0].category_value_ref.as_deref(),
        Some("CategoryValue_ManualReview")
    );
    assert_eq!(
        collaboration.text_annotations[0].text.as_deref(),
        Some("Review note from collaboration scope")
    );
}
