use super::support::assert_shape_bounds;
use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use xiuxian_qianji_bpmn_engine::{parse_dmn_decision, snapshot_dmn_source};

#[test]
fn dmn_snapshot_captures_versioned_document_metadata() {
    let snapshot = snapshot_dmn_source(&fixture_source("versioned-listed-input-data-20191111.dmn"))
        .must("versioned DMN source should produce a document snapshot");

    assert_eq!(
        snapshot.source_id,
        "versioned-listed-input-data-20191111.dmn"
    );
    assert_eq!(snapshot.root.element_name, "definitions");
    assert_eq!(
        snapshot.root.definitions_id.as_deref(),
        Some("Definitions_versioned_listed_input_data")
    );
    assert_eq!(
        snapshot.root.name.as_deref(),
        Some("Versioned Listed Input Data")
    );
    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.import_count, 0);
    assert_eq!(snapshot.root.item_definition_count, 0);
    assert_eq!(snapshot.root.input_data_count, 1);
    assert_eq!(snapshot.root.knowledge_source_count, 0);
    assert_eq!(snapshot.root.business_knowledge_model_count, 0);
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.root.organization_unit_count, 0);
    assert_eq!(snapshot.root.performance_indicator_count, 0);
    assert_eq!(snapshot.root.text_annotation_count, 0);
    assert_eq!(snapshot.root.association_count, 0);
    assert_eq!(snapshot.root.element_collection_count, 0);
    assert_eq!(snapshot.root.group_count, 0);
    assert_eq!(snapshot.root.dmndi_count, 1);
    assert_eq!(snapshot.root.dmndi_blocks.len(), 1);
    let dmndi = &snapshot.root.dmndi_blocks[0];
    assert_eq!(dmndi.dmndi_id, None);
    assert_eq!(dmndi.diagrams.len(), 1);
    let diagram = &dmndi.diagrams[0];
    assert_eq!(diagram.diagram_id.as_deref(), Some("diagram_eligibility"));
    assert_eq!(diagram.shape_count, 1);
    assert_eq!(diagram.edge_count, 0);
    assert_eq!(diagram.shapes.len(), 1);
    assert_eq!(diagram.edges.len(), 0);
    assert_eq!(
        diagram.shapes[0].shape_id.as_deref(),
        Some("shape_Decision_1")
    );
    assert_eq!(
        diagram.shapes[0].dmn_element_ref.as_deref(),
        Some("Decision_1")
    );
    assert_eq!(diagram.shapes[0].is_listed_input_data, None);
    assert_shape_bounds(
        &diagram.shapes[0],
        Some("120"),
        Some("80"),
        Some("180"),
        Some("80"),
    );
    assert_eq!(diagram.shapes[0].label, None);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_1");
    assert_eq!(
        snapshot.decisions[0].name.as_deref(),
        Some("Eligibility Decision")
    );
    assert_eq!(snapshot.decisions[0].allowed_answers_count, 0);
    assert_eq!(snapshot.decisions[0].decision_maker_count, 0);
    assert_eq!(snapshot.decisions[0].decision_owner_count, 0);
    assert_eq!(snapshot.decisions[0].decision_table_count, 0);
    assert_eq!(snapshot.decisions[0].information_requirement_count, 1);
    assert_eq!(snapshot.decisions[0].knowledge_requirement_count, 0);
    assert_eq!(snapshot.decisions[0].authority_requirement_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 0);
    assert_eq!(snapshot.decisions[0].context_count, 0);
    assert_eq!(snapshot.decisions[0].invocation_count, 0);
    assert_eq!(snapshot.decisions[0].relation_count, 0);
    assert_eq!(snapshot.decisions[0].function_definition_count, 0);
    assert_eq!(snapshot.decisions[0].list_count, 0);
}

#[test]
fn dmn_snapshot_preserves_namespaced_executable_decision_shape() {
    let source = fixture_source("versioned-unique-eligibility-20180521.dmn");
    let snapshot = snapshot_dmn_source(&source)
        .must("versioned executable DMN source should produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("http://www.omg.org/spec/DMN/20180521/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20180521")
    );
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
    assert_eq!(snapshot.root.element_collection_count, 0);
    assert_eq!(snapshot.root.group_count, 0);
    assert_eq!(snapshot.root.dmndi_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "versioned-eligibility");
    assert_eq!(snapshot.decisions[0].allowed_answers_count, 0);
    assert_eq!(snapshot.decisions[0].decision_maker_count, 0);
    assert_eq!(snapshot.decisions[0].decision_owner_count, 0);
    assert_eq!(snapshot.decisions[0].decision_table_count, 1);
    assert_eq!(snapshot.decisions[0].information_requirement_count, 0);
    assert_eq!(snapshot.decisions[0].knowledge_requirement_count, 0);
    assert_eq!(snapshot.decisions[0].authority_requirement_count, 0);
    assert_eq!(snapshot.decisions[0].literal_expression_count, 0);
    assert_eq!(snapshot.decisions[0].context_count, 0);
    assert_eq!(snapshot.decisions[0].invocation_count, 0);
    assert_eq!(snapshot.decisions[0].relation_count, 0);
    assert_eq!(snapshot.decisions[0].function_definition_count, 0);
    assert_eq!(snapshot.decisions[0].list_count, 0);

    let decision = parse_dmn_decision(&source)
        .must("versioned executable DMN source should still parse through the bounded parser");
    assert_eq!(
        decision.decision.decision_id.as_ref(),
        "versioned-eligibility"
    );
}

#[test]
fn dmn_snapshot_counts_top_level_imports() {
    let snapshot =
        snapshot_dmn_source(&fixture_source("unsupported-top-level-import-20191111.dmn"))
            .must("imported DMN source should still produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.import_count, 1);
    assert_eq!(snapshot.root.imports.len(), 1);
    let import = &snapshot.root.imports[0];
    assert_eq!(import.name.as_deref(), Some("Partner Services"));
    assert_eq!(
        import.namespace.as_deref(),
        Some("https://example.com/dmn/partner-services")
    );
    assert_eq!(import.location_uri.as_deref(), Some("partner-services.dmn"));
    assert_eq!(
        import.import_type.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(snapshot.root.item_definition_count, 0);
    assert_eq!(snapshot.root.input_data_count, 0);
    assert_eq!(snapshot.root.knowledge_source_count, 0);
    assert_eq!(snapshot.root.business_knowledge_model_count, 0);
    assert_eq!(snapshot.root.decision_service_count, 0);
    assert_eq!(snapshot.root.organization_unit_count, 0);
    assert_eq!(snapshot.root.performance_indicator_count, 0);
    assert_eq!(snapshot.root.text_annotation_count, 0);
    assert_eq!(snapshot.root.association_count, 0);
    assert_eq!(snapshot.root.element_collection_count, 0);
    assert_eq!(snapshot.root.group_count, 0);
    assert_eq!(snapshot.root.dmndi_count, 0);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_imported_offer");
    assert_eq!(snapshot.decisions[0].allowed_answers_count, 0);
    assert_eq!(snapshot.decisions[0].decision_maker_count, 0);
    assert_eq!(snapshot.decisions[0].decision_owner_count, 0);
    assert_eq!(snapshot.decisions[0].decision_table_count, 1);
}
