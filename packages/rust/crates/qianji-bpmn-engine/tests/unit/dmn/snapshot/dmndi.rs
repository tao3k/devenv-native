use crate::dmn::fixture_source;
use crate::test_support::MustExt as _;
use qianji_bpmn_engine::{
    DmnBoundsSnapshot, DmnLabelSnapshot, DmnWaypointSnapshot, snapshot_dmn_source,
};

fn assert_bounds(
    bounds: Option<&DmnBoundsSnapshot>,
    x: Option<&str>,
    y: Option<&str>,
    width: Option<&str>,
    height: Option<&str>,
) {
    assert_eq!(bounds.and_then(|bounds| bounds.x.as_deref()), x);
    assert_eq!(bounds.and_then(|bounds| bounds.y.as_deref()), y);
    assert_eq!(bounds.and_then(|bounds| bounds.width.as_deref()), width);
    assert_eq!(bounds.and_then(|bounds| bounds.height.as_deref()), height);
}

fn assert_label_bounds(
    label: Option<&DmnLabelSnapshot>,
    x: Option<&str>,
    y: Option<&str>,
    width: Option<&str>,
    height: Option<&str>,
) {
    assert_bounds(
        label.and_then(|label| label.bounds.as_ref()),
        x,
        y,
        width,
        height,
    );
}

fn assert_waypoint(waypoint: &DmnWaypointSnapshot, x: Option<&str>, y: Option<&str>) {
    assert_eq!(waypoint.x.as_deref(), x);
    assert_eq!(waypoint.y.as_deref(), y);
}

#[test]
fn dmn_snapshot_counts_top_level_dmndi_without_decisions() {
    let snapshot = snapshot_dmn_source(&fixture_source("metadata-only-dmndi-20191111.dmn"))
        .must("dmndi-only DMN source should still produce a document snapshot");

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
    assert_eq!(snapshot.root.dmndi_count, 1);
    assert_eq!(snapshot.root.dmndi_blocks.len(), 1);
    let dmndi = &snapshot.root.dmndi_blocks[0];
    assert_eq!(dmndi.dmndi_id, None);
    assert!(dmndi.diagrams.is_empty());
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn dmn_snapshot_preserves_direct_dmndi_diagram_element_metadata() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-dmndi-diagram-elements-20191111.dmn",
    ))
    .must("metadata-rich dmndi-only DMN source should still produce a document snapshot");

    assert_eq!(snapshot.root.dmndi_count, 1);
    assert_eq!(snapshot.root.dmndi_blocks.len(), 1);
    let dmndi = &snapshot.root.dmndi_blocks[0];
    assert_eq!(dmndi.diagrams.len(), 1);
    let diagram = &dmndi.diagrams[0];
    assert_eq!(diagram.diagram_id.as_deref(), Some("diagram_metadata"));
    assert_eq!(diagram.shape_count, 1);
    assert_eq!(diagram.edge_count, 1);
    assert_eq!(diagram.shapes.len(), 1);
    assert_eq!(diagram.edges.len(), 1);
    assert_eq!(diagram.shapes[0].shape_id.as_deref(), Some("shape_input_1"));
    assert_eq!(
        diagram.shapes[0].dmn_element_ref.as_deref(),
        Some("InputData_1")
    );
    assert_eq!(diagram.shapes[0].is_listed_input_data, None);
    assert_bounds(
        diagram.shapes[0].bounds.as_ref(),
        Some("120"),
        Some("80"),
        Some("180"),
        Some("80"),
    );
    assert_eq!(diagram.shapes[0].label, None);
    assert_eq!(
        diagram.edges[0].edge_id.as_deref(),
        Some("edge_requirement_1")
    );
    assert_eq!(
        diagram.edges[0].dmn_element_ref.as_deref(),
        Some("Requirement_1")
    );
    assert_eq!(diagram.edges[0].waypoints.len(), 2);
    assert_waypoint(&diagram.edges[0].waypoints[0], Some("180"), Some("120"));
    assert_waypoint(&diagram.edges[0].waypoints[1], Some("300"), Some("120"));
    assert_eq!(diagram.edges[0].label, None);
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn dmn_snapshot_preserves_decision_service_divider_line_placeholders() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "versioned-decision-service-is-collapsed-20180521.dmn",
    ))
    .must("decision-service divider-line DMN source should still produce a document snapshot");

    let diagram = &snapshot.root.dmndi_blocks[0].diagrams[0];
    let shape = &diagram.shapes[0];
    assert_eq!(
        shape.shape_id.as_deref(),
        Some("_DMNShape_DecisionService_1")
    );
    assert_eq!(shape.dmn_element_ref.as_deref(), Some("DecisionService_1"));
    assert_eq!(shape.is_collapsed, Some(false));
    assert_label_bounds(
        shape.label.as_ref(),
        Some("354"),
        Some("96"),
        Some("197"),
        Some("18"),
    );
    let Some(divider_line) = shape.decision_service_divider_line.as_ref() else {
        panic!("decision-service shape should preserve divider-line placeholders");
    };
    assert_eq!(divider_line.waypoints.len(), 2);
    assert_waypoint(&divider_line.waypoints[0], Some("0"), Some("210"));
    assert_waypoint(&divider_line.waypoints[1], Some("906"), Some("210"));
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn dmn_snapshot_preserves_listed_input_data_shape_metadata() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-dmndi-listed-input-shape-20191111.dmn",
    ))
    .must("listed-input metadata-only DMN source should still produce a document snapshot");

    assert_eq!(snapshot.root.dmndi_count, 1);
    assert_eq!(snapshot.root.dmndi_blocks.len(), 1);
    let dmndi = &snapshot.root.dmndi_blocks[0];
    assert_eq!(dmndi.diagrams.len(), 1);
    let diagram = &dmndi.diagrams[0];
    assert_eq!(diagram.diagram_id.as_deref(), Some("diagram_listed_input"));
    assert_eq!(diagram.shape_count, 1);
    assert_eq!(diagram.edge_count, 0);
    assert_eq!(diagram.shapes.len(), 1);
    assert_eq!(diagram.edges.len(), 0);
    assert_eq!(
        diagram.shapes[0].shape_id.as_deref(),
        Some("shape_input_listed_1")
    );
    assert_eq!(
        diagram.shapes[0].dmn_element_ref.as_deref(),
        Some("InputData_1")
    );
    assert_eq!(diagram.shapes[0].is_listed_input_data, Some(true));
    assert_eq!(diagram.shapes[0].is_collapsed, None);
    assert_bounds(
        diagram.shapes[0].bounds.as_ref(),
        Some("24"),
        Some("18"),
        Some("160"),
        Some("40"),
    );
    assert_eq!(diagram.shapes[0].label, None);
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn dmn_snapshot_preserves_direct_label_bounds_placeholders() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-dmndi-label-bounds-20191111.dmn",
    ))
    .must("dmndi label-bounds DMN source should still produce a document snapshot");

    let diagram = &snapshot.root.dmndi_blocks[0].diagrams[0];
    assert_eq!(
        diagram.shapes[0]
            .label
            .as_ref()
            .and_then(|label| label.label_id.as_deref()),
        Some("shape_label_1")
    );
    assert_label_bounds(
        diagram.shapes[0].label.as_ref(),
        Some("33"),
        Some("14"),
        Some("49"),
        Some("10"),
    );
    assert_eq!(
        diagram.edges[0]
            .label
            .as_ref()
            .and_then(|label| label.label_id.as_deref()),
        Some("edge_label_1")
    );
    assert_label_bounds(
        diagram.edges[0].label.as_ref(),
        Some("300"),
        Some("120"),
        Some("42"),
        Some("12"),
    );
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn dmn_snapshot_preserves_direct_label_placeholders() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-dmndi-label-placeholders-20191111.dmn",
    ))
    .must("dmndi label-placeholder DMN source should still produce a document snapshot");

    assert_eq!(snapshot.root.dmndi_count, 1);
    assert_eq!(snapshot.root.dmndi_blocks.len(), 1);
    let dmndi = &snapshot.root.dmndi_blocks[0];
    assert_eq!(dmndi.diagrams.len(), 1);
    let diagram = &dmndi.diagrams[0];
    assert_eq!(diagram.diagram_id.as_deref(), Some("diagram_labels"));
    assert_eq!(diagram.shape_count, 1);
    assert_eq!(diagram.edge_count, 1);
    assert_eq!(diagram.shapes.len(), 1);
    assert_eq!(diagram.edges.len(), 1);
    assert_eq!(
        diagram.shapes[0]
            .label
            .as_ref()
            .and_then(|label| label.label_id.as_deref()),
        Some("shape_label_1")
    );
    assert_eq!(diagram.shapes[0].bounds, None);
    assert_label_bounds(diagram.shapes[0].label.as_ref(), None, None, None, None);
    assert_eq!(
        diagram.shapes[0]
            .label
            .as_ref()
            .and_then(|label| label.text.as_deref()),
        None
    );
    assert_eq!(
        diagram.edges[0]
            .label
            .as_ref()
            .and_then(|label| label.label_id.as_deref()),
        Some("edge_label_1")
    );
    assert_label_bounds(diagram.edges[0].label.as_ref(), None, None, None, None);
    assert_eq!(
        diagram.edges[0]
            .label
            .as_ref()
            .and_then(|label| label.text.as_deref()),
        None
    );
    assert!(snapshot.decisions.is_empty());
}

#[test]
fn dmn_snapshot_preserves_direct_label_text_payloads() {
    let snapshot = snapshot_dmn_source(&fixture_source(
        "metadata-only-dmndi-label-text-20191111.dmn",
    ))
    .must("dmndi label-text DMN source should still produce a document snapshot");

    let diagram = &snapshot.root.dmndi_blocks[0].diagrams[0];
    assert_eq!(
        diagram.shapes[0]
            .label
            .as_ref()
            .and_then(|label| label.label_id.as_deref()),
        Some("shape_label_1")
    );
    assert_eq!(diagram.shapes[0].bounds, None);
    assert_label_bounds(diagram.shapes[0].label.as_ref(), None, None, None, None);
    assert_eq!(
        diagram.shapes[0]
            .label
            .as_ref()
            .and_then(|label| label.text.as_deref()),
        Some("Shape Label")
    );
    assert_eq!(
        diagram.edges[0]
            .label
            .as_ref()
            .and_then(|label| label.label_id.as_deref()),
        Some("edge_label_1")
    );
    assert_label_bounds(diagram.edges[0].label.as_ref(), None, None, None, None);
    assert_eq!(
        diagram.edges[0]
            .label
            .as_ref()
            .and_then(|label| label.text.as_deref()),
        Some("Edge Label")
    );
    assert!(snapshot.decisions.is_empty());
}
