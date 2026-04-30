use serde_json::json;

use crate::bpmn_model_api::{BpmnDocumentSnapshot, BpmnPlaneSnapshot};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::bpmn_snapshot_api::snapshot_bpmn_source;
use crate::lint::bpmn::document_surface::shared::SNAPSHOT_EVIDENCE_LIMIT;
use crate::lint::bpmn::document_surface::summary::diagram_snapshot_summary;
use crate::lint_api::LintIssue;

use super::model::DiAnchorViolation;

pub(in crate::lint::bpmn::document_surface) fn diagram_anchor_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    let snapshot = snapshot_bpmn_source(source).ok()?;
    let violations = missing_di_anchors(&snapshot);
    if violations.is_empty() {
        return None;
    }

    let source_id = &source.source_id;
    let evidence = json!({
        "source_id": source_id,
        "element": "BPMNDiagram",
        "deferred_family": "diagram",
        "snapshot_available": true,
        "missing_anchor_count": violations.len(),
        "missing_anchors": violations
            .iter()
            .take(SNAPSHOT_EVIDENCE_LIMIT)
            .map(DiAnchorViolation::evidence)
            .collect::<Vec<_>>(),
        "missing_anchors_truncated": violations.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "snapshot": diagram_snapshot_summary(&snapshot),
    });

    Some(LintIssue::new(
        "bpmn.missing_di_semantic_anchor",
        "BPMN diagram interchange metadata is missing semantic anchors",
        format!(
            "Source '{source_id}' contains BPMN DI elements without `bpmnElement` semantic anchors."
        ),
        "BPMN DI stays metadata-only in the bounded runtime, but each plane, shape, and edge should declare which BPMN semantic element it represents so diagram metadata can be traced without deriving runtime behavior from layout.",
        vec![
            "Add `bpmnElement` to each `bpmndi:BPMNPlane` and point it at the owning process, collaboration, choreography, or other diagram root semantic id.".to_string(),
            "Add `bpmnElement` to each `bpmndi:BPMNShape` and point it at the BPMN node or artifact it displays.".to_string(),
            "Add `bpmnElement` to each `bpmndi:BPMNEdge` and point it at the BPMN flow or association it displays.".to_string(),
            "Keep executable behavior in process flow, events, tasks, gateways, and data mappings rather than in diagram anchors.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' so every BPMN DI plane, shape, and edge has a `bpmnElement` anchor pointing at the semantic BPMN element it displays. Preserve diagram metadata for interchange, but do not rely on layout anchors for runtime behavior."
        ),
        evidence,
    ))
}

fn missing_di_anchors(snapshot: &BpmnDocumentSnapshot) -> Vec<DiAnchorViolation> {
    let mut violations = Vec::new();
    for diagram in &snapshot.root.diagrams {
        let diagram_id = diagram.diagram_id.as_deref();
        let Some(plane) = diagram.plane.as_ref() else {
            continue;
        };
        collect_plane_anchor_violations(&mut violations, diagram_id, plane);
    }
    violations
}

fn collect_plane_anchor_violations(
    violations: &mut Vec<DiAnchorViolation>,
    diagram_id: Option<&str>,
    plane: &BpmnPlaneSnapshot,
) {
    let plane_id = plane.plane_id.as_deref();
    if plane.bpmn_element.is_none() {
        violations.push(DiAnchorViolation::plane(diagram_id, plane_id));
    }
    for shape in &plane.shapes {
        if shape.bpmn_element.is_none() {
            violations.push(DiAnchorViolation::shape(
                diagram_id,
                plane_id,
                shape.shape_id.as_deref(),
            ));
        }
    }
    for edge in &plane.edges {
        if edge.bpmn_element.is_none() {
            violations.push(DiAnchorViolation::edge(
                diagram_id,
                plane_id,
                edge.edge_id.as_deref(),
            ));
        }
    }
}
