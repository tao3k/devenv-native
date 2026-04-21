use super::super::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::snapshot_dmn_source;

#[test]
fn dmn_snapshot_counts_top_level_item_definitions_without_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-item-definition-20191111.dmn",
    ))
    .must("item-definition-only DMN source should still produce a document snapshot");

    assert_eq!(snapshot.root.import_count, 0);
    assert_eq!(snapshot.root.item_definition_count, 1);
    assert_eq!(snapshot.root.input_data_count, 0);
    assert_eq!(snapshot.root.knowledge_source_count, 0);
    assert_eq!(snapshot.root.business_knowledge_model_count, 0);
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert!(snapshot.decisions.is_empty());
}
