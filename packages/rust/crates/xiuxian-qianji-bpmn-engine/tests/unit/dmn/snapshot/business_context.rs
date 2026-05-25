use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::snapshot_dmn_source;

#[test]
fn dmn_snapshot_counts_top_level_organization_units_without_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-organization-unit-20191111.dmn",
    ))
    .must("organization-unit-only DMN source should still produce a document snapshot");

    assert_eq!(snapshot.root.import_count, 0);
    assert_eq!(snapshot.root.item_definition_count, 0);
    assert_eq!(snapshot.root.input_data_count, 0);
    assert_eq!(snapshot.root.knowledge_source_count, 0);
    assert_eq!(snapshot.root.business_knowledge_model_count, 0);
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.root.organization_unit_count, 1);
    assert_eq!(snapshot.root.organization_units.len(), 1);
    assert_eq!(snapshot.root.performance_indicator_count, 0);
    assert!(snapshot.root.performance_indicators.is_empty());
    assert_eq!(snapshot.root.text_annotation_count, 0);
    assert_eq!(snapshot.root.association_count, 0);
    assert_eq!(snapshot.root.element_collection_count, 0);
    assert_eq!(snapshot.root.group_count, 0);
    assert_eq!(snapshot.root.dmndi_count, 0);
    let organization_unit = &snapshot.root.organization_units[0];
    assert_eq!(
        organization_unit.organization_unit_id.as_deref(),
        Some("OrganizationUnit_credit_risk")
    );
    assert_eq!(
        organization_unit.name.as_deref(),
        Some("Credit Risk Committee")
    );
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn dmn_snapshot_counts_top_level_performance_indicators_without_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-performance-indicator-20191111.dmn",
    ))
    .must("performance-indicator-only DMN source should still produce a document snapshot");

    assert_eq!(snapshot.root.import_count, 0);
    assert_eq!(snapshot.root.item_definition_count, 0);
    assert_eq!(snapshot.root.input_data_count, 0);
    assert_eq!(snapshot.root.knowledge_source_count, 0);
    assert_eq!(snapshot.root.business_knowledge_model_count, 0);
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.root.organization_unit_count, 0);
    assert!(snapshot.root.organization_units.is_empty());
    assert_eq!(snapshot.root.performance_indicator_count, 1);
    assert_eq!(snapshot.root.performance_indicators.len(), 1);
    assert_eq!(snapshot.root.text_annotation_count, 0);
    assert_eq!(snapshot.root.association_count, 0);
    assert_eq!(snapshot.root.element_collection_count, 0);
    assert_eq!(snapshot.root.group_count, 0);
    assert_eq!(snapshot.root.dmndi_count, 0);
    let performance_indicator = &snapshot.root.performance_indicators[0];
    assert_eq!(
        performance_indicator.performance_indicator_id.as_deref(),
        Some("PerformanceIndicator_auto_adjudication_rate")
    );
    assert_eq!(
        performance_indicator.name.as_deref(),
        Some("Auto Adjudication Rate")
    );
    assert!(snapshot.decisions.is_empty());
}
