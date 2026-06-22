use std::collections::BTreeSet;

use crate::bpmn_model_api::{
    BpmnDiagramSnapshot, BpmnLabelSnapshot, BpmnPlaneSnapshot, BpmnShapeSnapshot,
};

use super::model::{DiReferenceScope, DiReferenceTarget, DiReferenceViolation};

pub(super) fn collect_local_reference_violations(
    violations: &mut Vec<DiReferenceViolation>,
    diagram_id: Option<&str>,
    diagram: &BpmnDiagramSnapshot,
    plane: &BpmnPlaneSnapshot,
) {
    let label_style_ids = diagram
        .label_styles
        .iter()
        .filter_map(|style| style.style_id.as_deref())
        .collect::<BTreeSet<_>>();
    let di_element_ids = plane_di_element_ids(plane);
    let shape_ids = plane_shape_ids(plane);
    let scope = DiReferenceScope {
        diagram_id,
        plane_id: plane.plane_id.as_deref(),
    };

    for shape in &plane.shapes {
        collect_shape_reference(violations, scope, shape, &shape_ids);
        collect_label_style_violation(violations, scope, shape.label.as_ref(), &label_style_ids);
    }
    for edge in &plane.edges {
        collect_edge_reference(
            violations,
            scope,
            edge.edge_id.clone(),
            "sourceElement",
            edge.source_element.as_deref(),
            &di_element_ids,
        );
        collect_edge_reference(
            violations,
            scope,
            edge.edge_id.clone(),
            "targetElement",
            edge.target_element.as_deref(),
            &di_element_ids,
        );
        collect_label_style_violation(violations, scope, edge.label.as_ref(), &label_style_ids);
    }
}

fn collect_shape_reference(
    violations: &mut Vec<DiReferenceViolation>,
    scope: DiReferenceScope<'_>,
    shape: &BpmnShapeSnapshot,
    shape_ids: &BTreeSet<&str>,
) {
    collect_reference(
        violations,
        scope,
        DiReferenceTarget {
            element: "BPMNShape",
            element_id: shape.shape_id.as_deref().map(str::to_string),
            attribute: "choreographyActivityShape",
        },
        shape.choreography_activity_shape.as_deref(),
        shape_ids,
    );
}

fn collect_edge_reference(
    violations: &mut Vec<DiReferenceViolation>,
    scope: DiReferenceScope<'_>,
    edge_id: Option<String>,
    attribute: &'static str,
    reference: Option<&str>,
    di_element_ids: &BTreeSet<&str>,
) {
    collect_reference(
        violations,
        scope,
        DiReferenceTarget {
            element: "BPMNEdge",
            element_id: edge_id,
            attribute,
        },
        reference,
        di_element_ids,
    );
}

fn collect_label_style_violation(
    violations: &mut Vec<DiReferenceViolation>,
    scope: DiReferenceScope<'_>,
    label: Option<&BpmnLabelSnapshot>,
    label_style_ids: &BTreeSet<&str>,
) {
    let Some(label) = label else {
        return;
    };
    collect_reference(
        violations,
        scope,
        DiReferenceTarget {
            element: "BPMNLabel",
            element_id: label.label_id.clone(),
            attribute: "labelStyle",
        },
        label.label_style.as_deref(),
        label_style_ids,
    );
}

fn collect_reference(
    violations: &mut Vec<DiReferenceViolation>,
    scope: DiReferenceScope<'_>,
    target: DiReferenceTarget,
    reference: Option<&str>,
    valid_ids: &BTreeSet<&str>,
) {
    let Some(reference) = reference else {
        return;
    };
    if valid_ids.contains(reference) {
        return;
    }
    violations.push(DiReferenceViolation::local(scope, target, reference));
}

fn plane_di_element_ids(plane: &BpmnPlaneSnapshot) -> BTreeSet<&str> {
    let mut ids = BTreeSet::new();
    if let Some(plane_id) = plane.plane_id.as_deref() {
        ids.insert(plane_id);
    }
    ids.extend(
        plane
            .shapes
            .iter()
            .filter_map(|shape| shape.shape_id.as_deref()),
    );
    ids.extend(
        plane
            .edges
            .iter()
            .filter_map(|edge| edge.edge_id.as_deref()),
    );
    ids
}

fn plane_shape_ids(plane: &BpmnPlaneSnapshot) -> BTreeSet<&str> {
    plane
        .shapes
        .iter()
        .filter_map(|shape| shape.shape_id.as_deref())
        .collect()
}
