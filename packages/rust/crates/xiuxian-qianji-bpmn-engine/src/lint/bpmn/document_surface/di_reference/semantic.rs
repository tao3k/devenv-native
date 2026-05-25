use crate::bpmn_model_api::BpmnPlaneSnapshot;
use crate::lint::bpmn::document_surface::di_semantic::SemanticElementIndex;

use super::model::{DiReferenceScope, DiReferenceTarget, DiReferenceViolation};

pub(super) fn collect_semantic_reference_violations(
    violations: &mut Vec<DiReferenceViolation>,
    diagram_id: Option<&str>,
    plane_id: Option<&str>,
    plane: &BpmnPlaneSnapshot,
    semantic_index: &SemanticElementIndex,
) {
    let scope = DiReferenceScope {
        diagram_id,
        plane_id,
    };
    collect_reference(
        violations,
        scope,
        DiReferenceTarget {
            element: "BPMNPlane",
            element_id: plane.plane_id.clone(),
            attribute: "bpmnElement",
        },
        plane.bpmn_element.as_deref(),
        semantic_index,
    );
    for shape in &plane.shapes {
        collect_reference(
            violations,
            scope,
            DiReferenceTarget {
                element: "BPMNShape",
                element_id: shape.shape_id.as_deref().map(str::to_string),
                attribute: "bpmnElement",
            },
            shape.bpmn_element.as_deref(),
            semantic_index,
        );
    }
    for edge in &plane.edges {
        collect_reference(
            violations,
            scope,
            DiReferenceTarget {
                element: "BPMNEdge",
                element_id: edge.edge_id.clone(),
                attribute: "bpmnElement",
            },
            edge.bpmn_element.as_deref(),
            semantic_index,
        );
    }
}

fn collect_reference(
    violations: &mut Vec<DiReferenceViolation>,
    scope: DiReferenceScope<'_>,
    target: DiReferenceTarget,
    reference: Option<&str>,
    semantic_index: &SemanticElementIndex,
) {
    let Some(reference) = reference else {
        return;
    };
    if semantic_index.contains_id(reference) {
        return;
    }
    violations.push(DiReferenceViolation::semantic(scope, target, reference));
}
