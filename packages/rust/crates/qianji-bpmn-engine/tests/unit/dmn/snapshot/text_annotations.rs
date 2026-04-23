use super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::snapshot_dmn_source;

#[test]
fn dmn_snapshot_counts_top_level_text_annotations_without_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-text-annotation-20191111.dmn",
    ))
    .must("text-annotation-only DMN source should still produce a document snapshot");

    assert_eq!(snapshot.root.import_count, 0);
    assert_eq!(snapshot.root.item_definition_count, 0);
    assert_eq!(snapshot.root.input_data_count, 0);
    assert_eq!(snapshot.root.knowledge_source_count, 0);
    assert_eq!(snapshot.root.business_knowledge_model_count, 0);
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.root.organization_unit_count, 0);
    assert_eq!(snapshot.root.performance_indicator_count, 0);
    assert_eq!(snapshot.root.text_annotation_count, 1);
    assert_eq!(snapshot.root.text_annotations.len(), 1);
    assert_eq!(snapshot.root.association_count, 0);
    assert_eq!(snapshot.root.element_collection_count, 0);
    assert_eq!(snapshot.root.group_count, 0);
    assert_eq!(snapshot.root.dmndi_count, 0);
    let text_annotation = &snapshot.root.text_annotations[0];
    assert_eq!(
        text_annotation.text_annotation_id.as_deref(),
        Some("TextAnnotation_credit_policy_note")
    );
    assert_eq!(
        text_annotation.text.as_deref(),
        Some("Credit policy note for manual reviewers.")
    );
    assert!(snapshot.decisions.is_empty());
}
