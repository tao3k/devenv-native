use std::collections::BTreeMap;

use crate::bpmn_model_api::{
    BpmnDiagramSnapshot, BpmnDocumentSnapshot, BpmnEdgeSnapshot, BpmnPlaneSnapshot,
    BpmnShapeSnapshot,
};

use super::model::{DiIdentityOccurrence, DiIdentityScope, DiIdentityViolation};

pub(super) fn duplicate_di_ids(snapshot: &BpmnDocumentSnapshot) -> Vec<DiIdentityViolation> {
    let mut occurrences = BTreeMap::<String, Vec<DiIdentityOccurrence>>::new();
    for diagram in &snapshot.root.diagrams {
        collect_diagram_ids(&mut occurrences, diagram);
    }

    occurrences
        .into_iter()
        .filter_map(|(di_id, occurrences)| {
            if occurrences.len() < 2 {
                return None;
            }
            Some(DiIdentityViolation::new(di_id, occurrences))
        })
        .collect()
}

fn collect_diagram_ids(
    occurrences: &mut BTreeMap<String, Vec<DiIdentityOccurrence>>,
    diagram: &BpmnDiagramSnapshot,
) {
    let diagram_id = diagram.diagram_id.as_deref();
    let diagram_scope = DiIdentityScope {
        diagram_id,
        plane_id: None,
    };
    record_id(
        occurrences,
        diagram_scope,
        "BPMNDiagram",
        diagram_id,
        None,
        None,
    );

    for label_style in &diagram.label_styles {
        record_id(
            occurrences,
            diagram_scope,
            "BPMNLabelStyle",
            label_style.style_id.as_deref(),
            None,
            None,
        );
    }

    if let Some(plane) = diagram.plane.as_ref() {
        collect_plane_ids(occurrences, diagram_id, plane);
    }
}

fn collect_plane_ids(
    occurrences: &mut BTreeMap<String, Vec<DiIdentityOccurrence>>,
    diagram_id: Option<&str>,
    plane: &BpmnPlaneSnapshot,
) {
    let plane_id = plane.plane_id.as_deref();
    let scope = DiIdentityScope {
        diagram_id,
        plane_id,
    };
    record_id(occurrences, scope, "BPMNPlane", plane_id, None, None);

    for shape in &plane.shapes {
        collect_shape_ids(occurrences, scope, shape);
    }
    for edge in &plane.edges {
        collect_edge_ids(occurrences, scope, edge);
    }
}

fn collect_shape_ids(
    occurrences: &mut BTreeMap<String, Vec<DiIdentityOccurrence>>,
    scope: DiIdentityScope<'_>,
    shape: &BpmnShapeSnapshot,
) {
    let shape_id = shape.shape_id.as_deref();
    record_id(occurrences, scope, "BPMNShape", shape_id, None, None);
    if let Some(label) = shape.label.as_ref() {
        record_id(
            occurrences,
            scope,
            "BPMNLabel",
            label.label_id.as_deref(),
            Some("BPMNShape"),
            shape_id,
        );
    }
}

fn collect_edge_ids(
    occurrences: &mut BTreeMap<String, Vec<DiIdentityOccurrence>>,
    scope: DiIdentityScope<'_>,
    edge: &BpmnEdgeSnapshot,
) {
    let edge_id = edge.edge_id.as_deref();
    record_id(occurrences, scope, "BPMNEdge", edge_id, None, None);
    if let Some(label) = edge.label.as_ref() {
        record_id(
            occurrences,
            scope,
            "BPMNLabel",
            label.label_id.as_deref(),
            Some("BPMNEdge"),
            edge_id,
        );
    }
}

fn record_id(
    occurrences: &mut BTreeMap<String, Vec<DiIdentityOccurrence>>,
    scope: DiIdentityScope<'_>,
    element: &'static str,
    element_id: Option<&str>,
    owner_element: Option<&'static str>,
    owner_id: Option<&str>,
) {
    let Some(element_id) = element_id else {
        return;
    };
    occurrences
        .entry(element_id.to_string())
        .or_default()
        .push(DiIdentityOccurrence::new(
            scope,
            element,
            element_id,
            owner_element,
            owner_id,
        ));
}
