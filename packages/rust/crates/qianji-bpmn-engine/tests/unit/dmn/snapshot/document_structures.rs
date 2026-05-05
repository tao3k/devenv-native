use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::snapshot_dmn_source;

#[test]
fn dmn_snapshot_counts_top_level_associations_without_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source("metadata-only-association-20191111.dmn"))
        .must("association-only DMN source should still produce a document snapshot");

    assert_eq!(snapshot.root.import_count, 0);
    assert_eq!(snapshot.root.item_definition_count, 0);
    assert_eq!(snapshot.root.input_data_count, 0);
    assert_eq!(snapshot.root.knowledge_source_count, 0);
    assert_eq!(snapshot.root.business_knowledge_model_count, 0);
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.root.organization_unit_count, 0);
    assert_eq!(snapshot.root.performance_indicator_count, 0);
    assert_eq!(snapshot.root.text_annotation_count, 0);
    assert_eq!(snapshot.root.association_count, 1);
    assert_eq!(snapshot.root.associations.len(), 1);
    assert_eq!(snapshot.root.element_collection_count, 0);
    assert_eq!(snapshot.root.group_count, 0);
    assert_eq!(snapshot.root.dmndi_count, 0);
    let association = &snapshot.root.associations[0];
    assert_eq!(
        association.association_id.as_deref(),
        Some("Association_credit_policy_reference")
    );
    assert_eq!(association.association_direction.as_deref(), Some("One"));
    assert_eq!(
        association.source_ref.as_deref(),
        Some("TextAnnotation_credit_policy_note")
    );
    assert_eq!(
        association.target_ref.as_deref(),
        Some("Decision_credit_policy")
    );
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn dmn_snapshot_counts_top_level_element_collections_without_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-element-collection-20191111.dmn",
    ))
    .must("element-collection-only DMN source should still produce a document snapshot");

    assert_eq!(snapshot.root.import_count, 0);
    assert_eq!(snapshot.root.item_definition_count, 0);
    assert_eq!(snapshot.root.input_data_count, 0);
    assert_eq!(snapshot.root.knowledge_source_count, 0);
    assert_eq!(snapshot.root.business_knowledge_model_count, 0);
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.root.organization_unit_count, 0);
    assert_eq!(snapshot.root.performance_indicator_count, 0);
    assert_eq!(snapshot.root.text_annotation_count, 0);
    assert_eq!(snapshot.root.association_count, 0);
    assert_eq!(snapshot.root.element_collection_count, 1);
    assert_eq!(snapshot.root.element_collections.len(), 1);
    assert_eq!(snapshot.root.group_count, 0);
    assert_eq!(snapshot.root.dmndi_count, 0);
    let element_collection = &snapshot.root.element_collections[0];
    assert_eq!(
        element_collection.element_collection_id.as_deref(),
        Some("ElementCollection_manual_review_bundle")
    );
    assert_eq!(
        element_collection.name.as_deref(),
        Some("Manual Review Bundle")
    );
    assert!(snapshot.decisions.is_empty());
}
