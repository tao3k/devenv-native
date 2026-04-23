use super::super::super::fixture_source;
use super::support::{assert_label_bounds, assert_shape_bounds, assert_waypoint};
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::snapshot_dmn_source;

#[test]
fn dmn_snapshot_classifies_decision_service_documents() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-decision-service-is-collapsed-20180521.dmn",
    ))
    .must("decision-service DMN source should still produce a document snapshot");

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
    assert_eq!(snapshot.root.decision_service_count, 1);
    assert_eq!(snapshot.root.decision_services.len(), 1);
    let decision_service = &snapshot.root.decision_services[0];
    assert_eq!(
        decision_service.decision_service_id.as_deref(),
        Some("DecisionService_1")
    );
    assert_eq!(decision_service.name.as_deref(), Some("Decision Service 1"));
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
    assert_eq!(diagram.diagram_id.as_deref(), Some("DRD1"));
    assert_eq!(diagram.shape_count, 1);
    assert_eq!(diagram.edge_count, 0);
    assert_eq!(diagram.shapes.len(), 1);
    assert_eq!(diagram.edges.len(), 0);
    assert_eq!(
        diagram.shapes[0].shape_id.as_deref(),
        Some("_DMNShape_DecisionService_1")
    );
    assert_eq!(
        diagram.shapes[0].dmn_element_ref.as_deref(),
        Some("DecisionService_1")
    );
    assert_eq!(diagram.shapes[0].is_listed_input_data, None);
    assert_eq!(diagram.shapes[0].is_collapsed, Some(false));
    assert_label_bounds(
        diagram.shapes[0].label.as_ref(),
        Some("354"),
        Some("96"),
        Some("197"),
        Some("18"),
    );
    let Some(divider_line) = diagram.shapes[0].decision_service_divider_line.as_ref() else {
        panic!("decision-service shape should preserve divider-line placeholders");
    };
    assert_eq!(divider_line.waypoints.len(), 2);
    assert_waypoint(&divider_line.waypoints[0], Some("0"), Some("210"));
    assert_waypoint(&divider_line.waypoints[1], Some("906"), Some("210"));
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn dmn_snapshot_preserves_decision_service_reference_placeholders() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-decision-service-references-20180521.dmn",
    ))
    .must("decision-service reference DMN source should produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("http://www.omg.org/spec/DMN/20180521/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20180521")
    );
    assert_eq!(snapshot.root.decision_service_count, 1);
    assert_eq!(snapshot.root.decision_services.len(), 1);
    let decision_service = &snapshot.root.decision_services[0];
    assert_eq!(
        decision_service.decision_service_id.as_deref(),
        Some("DecisionService_credit")
    );
    assert_eq!(
        decision_service.name.as_deref(),
        Some("Credit Decision Service")
    );
    assert_eq!(
        decision_service
            .output_decisions
            .iter()
            .map(|reference| (reference.reference_kind.as_str(), reference.href.as_deref()))
            .collect::<Vec<_>>(),
        vec![("outputDecision", Some("#Decision_approval"))]
    );
    assert_eq!(
        decision_service
            .encapsulated_decisions
            .iter()
            .map(|reference| (reference.reference_kind.as_str(), reference.href.as_deref()))
            .collect::<Vec<_>>(),
        vec![("encapsulatedDecision", Some("#Decision_risk_score"))]
    );
    assert_eq!(
        decision_service
            .input_decisions
            .iter()
            .map(|reference| (reference.reference_kind.as_str(), reference.href.as_deref()))
            .collect::<Vec<_>>(),
        vec![("inputDecision", Some("#Decision_prior_risk"))]
    );
    assert_eq!(
        decision_service
            .input_data
            .iter()
            .map(|reference| (reference.reference_kind.as_str(), reference.href.as_deref()))
            .collect::<Vec<_>>(),
        vec![("inputData", Some("#InputData_application"))]
    );
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn dmn_snapshot_preserves_versioned_listed_input_data_shape_metadata() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-listed-input-data-shapes-20191111.dmn",
    ))
    .must("versioned listed-input-shape DMN source should produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.input_data_count, 2);
    assert_eq!(snapshot.root.dmndi_count, 1);
    assert_eq!(snapshot.root.dmndi_blocks.len(), 1);
    let dmndi = &snapshot.root.dmndi_blocks[0];
    assert_eq!(dmndi.diagrams.len(), 1);
    let diagram = &dmndi.diagrams[0];
    assert_eq!(diagram.diagram_id.as_deref(), Some("DRD1"));
    assert_eq!(diagram.shape_count, 3);
    assert_eq!(diagram.edge_count, 0);
    assert_eq!(diagram.shapes.len(), 3);
    assert_eq!(diagram.edges.len(), 0);
    assert_eq!(diagram.shapes[0].is_listed_input_data, None);
    assert_shape_bounds(
        &diagram.shapes[0],
        Some("1"),
        Some("1"),
        Some("114"),
        Some("38"),
    );
    assert_eq!(diagram.shapes[0].label, None);
    assert_eq!(
        diagram.shapes[1].shape_id.as_deref(),
        Some("_DMNShape_InputData_1")
    );
    assert_eq!(
        diagram.shapes[1].dmn_element_ref.as_deref(),
        Some("InputData_1")
    );
    assert_eq!(diagram.shapes[1].is_listed_input_data, Some(true));
    assert_shape_bounds(
        &diagram.shapes[1],
        Some("1"),
        Some("39"),
        Some("114"),
        Some("19"),
    );
    assert_eq!(diagram.shapes[1].label, None);
    assert_eq!(
        diagram.shapes[2].shape_id.as_deref(),
        Some("_DMNShape_InputData_2")
    );
    assert_eq!(
        diagram.shapes[2].dmn_element_ref.as_deref(),
        Some("InputData_2")
    );
    assert_eq!(diagram.shapes[2].is_listed_input_data, Some(true));
    assert_shape_bounds(
        &diagram.shapes[2],
        Some("1"),
        Some("58"),
        Some("114"),
        Some("19"),
    );
    assert_eq!(diagram.shapes[2].label, None);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_1");
    assert_eq!(snapshot.decisions[0].required_input_count, 2);
}

#[test]
fn dmn_snapshot_preserves_versioned_direct_label_placeholders() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-listed-input-data-shape-labels-20191111.dmn",
    ))
    .must("versioned listed-input label DMN source should produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(snapshot.root.dmndi_count, 1);
    assert_eq!(snapshot.root.dmndi_blocks.len(), 1);
    let diagram = &snapshot.root.dmndi_blocks[0].diagrams[0];
    assert_eq!(diagram.shape_count, 3);
    assert_eq!(diagram.shapes.len(), 3);
    assert!(diagram.shapes[0].label.is_some());
    assert_shape_bounds(
        &diagram.shapes[0],
        Some("1"),
        Some("1"),
        Some("114"),
        Some("38"),
    );
    assert_eq!(
        diagram.shapes[0]
            .label
            .as_ref()
            .and_then(|label| label.label_id.as_deref()),
        None
    );
    assert_label_bounds(
        diagram.shapes[0].label.as_ref(),
        Some("33"),
        Some("14"),
        Some("49"),
        Some("10"),
    );
    assert_eq!(
        diagram.shapes[0]
            .label
            .as_ref()
            .and_then(|label| label.text.as_deref()),
        None
    );
    assert!(diagram.shapes[1].label.is_some());
    assert_eq!(diagram.shapes[1].is_listed_input_data, Some(true));
    assert_shape_bounds(
        &diagram.shapes[1],
        Some("1"),
        Some("39"),
        Some("114"),
        Some("19"),
    );
    assert_label_bounds(
        diagram.shapes[1].label.as_ref(),
        Some("23"),
        Some("44"),
        Some("67"),
        Some("12"),
    );
    assert_eq!(
        diagram.shapes[1]
            .label
            .as_ref()
            .and_then(|label| label.text.as_deref()),
        None
    );
    assert!(diagram.shapes[2].label.is_some());
    assert_eq!(diagram.shapes[2].is_listed_input_data, Some(true));
    assert_shape_bounds(
        &diagram.shapes[2],
        Some("1"),
        Some("58"),
        Some("114"),
        Some("19"),
    );
    assert_label_bounds(
        diagram.shapes[2].label.as_ref(),
        Some("23"),
        Some("60"),
        Some("69"),
        Some("12"),
    );
    assert_eq!(
        diagram.shapes[2]
            .label
            .as_ref()
            .and_then(|label| label.text.as_deref()),
        None
    );
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].required_input_count, 2);
}

#[test]
fn dmn_snapshot_preserves_versioned_direct_label_text_payload() {
    let snapshot = snapshot_dmn_source(&fixture_source("versioned-shape-label-text-20191111.dmn"))
        .must("versioned shape-label-text DMN source should produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(snapshot.root.dmndi_count, 1);
    let diagram = &snapshot.root.dmndi_blocks[0].diagrams[0];
    assert_eq!(diagram.shape_count, 1);
    assert_shape_bounds(
        &diagram.shapes[0],
        Some("200"),
        Some("200"),
        Some("180"),
        Some("80"),
    );
    assert_label_bounds(diagram.shapes[0].label.as_ref(), None, None, None, None);
    assert_eq!(
        diagram.shapes[0]
            .label
            .as_ref()
            .and_then(|label| label.text.as_deref()),
        Some("Deci-\nsion\n1")
    );
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_1");
}

#[test]
fn dmn_snapshot_preserves_versioned_direct_edge_waypoints() {
    let snapshot = snapshot_dmn_source(&fixture_source("versioned-edge-waypoints-20191111.dmn"))
        .must("versioned edge-waypoint DMN source should produce a document snapshot");

    assert_eq!(
        snapshot.root.model_namespace_uri.as_deref(),
        Some("https://www.omg.org/spec/DMN/20191111/MODEL/")
    );
    assert_eq!(
        snapshot.root.model_version_hint.as_deref(),
        Some("20191111")
    );
    assert_eq!(snapshot.root.input_data_count, 1);
    assert_eq!(snapshot.root.dmndi_count, 1);
    assert_eq!(snapshot.root.dmndi_blocks.len(), 1);
    let diagram = &snapshot.root.dmndi_blocks[0].diagrams[0];
    assert_eq!(diagram.diagram_id.as_deref(), Some("diagram_eligibility"));
    assert_eq!(diagram.shape_count, 2);
    assert_eq!(diagram.edge_count, 1);
    assert_eq!(diagram.shapes.len(), 2);
    assert_eq!(diagram.edges.len(), 1);
    assert_shape_bounds(
        &diagram.shapes[0],
        Some("120"),
        Some("80"),
        Some("180"),
        Some("80"),
    );
    assert_eq!(
        diagram.edges[0].edge_id.as_deref(),
        Some("edge_requirement_1")
    );
    assert_eq!(
        diagram.edges[0].dmn_element_ref.as_deref(),
        Some("Requirement_1")
    );
    assert_eq!(diagram.edges[0].waypoints.len(), 2);
    assert_waypoint(&diagram.edges[0].waypoints[0], Some("210"), Some("240"));
    assert_waypoint(&diagram.edges[0].waypoints[1], Some("210"), Some("160"));
    assert_eq!(diagram.edges[0].label, None);
    assert_eq!(snapshot.decisions.len(), 1);
    assert_eq!(snapshot.decisions[0].decision_id, "Decision_1");
    assert_eq!(snapshot.decisions[0].information_requirement_count, 1);
    assert_eq!(snapshot.decisions[0].required_input_count, 1);
}
