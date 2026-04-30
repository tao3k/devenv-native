use serde_json::json;

use crate::bpmn_model_api::{BpmnDocumentSnapshot, BpmnPlaneSnapshot};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::bpmn_snapshot_api::snapshot_bpmn_source;
use crate::lint::bpmn::document_surface::shared::SNAPSHOT_EVIDENCE_LIMIT;
use crate::lint::bpmn::document_surface::summary::diagram_snapshot_summary;
use crate::lint_api::LintIssue;

use super::model::DiCompletenessViolation;

pub(in crate::lint::bpmn::document_surface) fn diagram_completeness_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    let snapshot = snapshot_bpmn_source(source).ok()?;
    let violations = incomplete_di_surfaces(&snapshot);
    if violations.is_empty() {
        return None;
    }

    let source_id = &source.source_id;
    let evidence = json!({
        "source_id": source_id,
        "element": "BPMNDiagram",
        "deferred_family": "diagram",
        "snapshot_available": true,
        "incomplete_surface_count": violations.len(),
        "incomplete_surfaces": violations
            .iter()
            .take(SNAPSHOT_EVIDENCE_LIMIT)
            .map(DiCompletenessViolation::evidence)
            .collect::<Vec<_>>(),
        "incomplete_surfaces_truncated": violations.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "snapshot": diagram_snapshot_summary(&snapshot),
    });

    Some(LintIssue::new(
        "bpmn.incomplete_di_surface",
        "BPMN diagram interchange metadata is incomplete",
        format!(
            "Source '{source_id}' contains BPMN DI elements without the minimum layout payload needed for stable interchange."
        ),
        "BPMN DI stays metadata-only in the bounded runtime, but shapes, edges, and label styles should carry their direct interchange payloads so editors can round-trip the diagram without deriving runtime behavior from coordinates or labels.",
        vec![
            "Add direct `dc:Bounds` metadata to every BPMN DI `BPMNShape`.".to_string(),
            "Add at least two direct `di:waypoint` entries to every BPMN DI `BPMNEdge`.".to_string(),
            "Add a direct `dc:Font` child to every `BPMNLabelStyle` entry.".to_string(),
            "Keep executable behavior in process flow, events, tasks, gateways, and data mappings rather than in diagram coordinates.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' so each BPMN DI shape has `dc:Bounds`, each BPMN DI edge has at least two `di:waypoint` entries, and each `BPMNLabelStyle` has a direct `dc:Font` child. Preserve diagram metadata for interchange, but do not move executable behavior into layout coordinates or labels."
        ),
        evidence,
    ))
}

fn incomplete_di_surfaces(snapshot: &BpmnDocumentSnapshot) -> Vec<DiCompletenessViolation> {
    let mut violations = Vec::new();
    for diagram in &snapshot.root.diagrams {
        let diagram_id = diagram.diagram_id.as_deref();
        let Some(plane) = diagram.plane.as_ref() else {
            continue;
        };
        collect_plane_violations(&mut violations, diagram_id, plane);
    }
    collect_label_style_violations(&mut violations, snapshot);
    violations
}

fn collect_plane_violations(
    violations: &mut Vec<DiCompletenessViolation>,
    diagram_id: Option<&str>,
    plane: &BpmnPlaneSnapshot,
) {
    let plane_id = plane.plane_id.as_deref();
    for shape in &plane.shapes {
        if shape.bounds.is_none() {
            violations.push(DiCompletenessViolation::shape_bounds(
                diagram_id,
                plane_id,
                shape.shape_id.as_deref(),
            ));
        }
    }
    for edge in &plane.edges {
        if edge.waypoints.len() < 2 {
            violations.push(DiCompletenessViolation::edge_waypoints(
                diagram_id,
                plane_id,
                edge.edge_id.as_deref(),
                edge.waypoints.len(),
            ));
        }
    }
}

fn collect_label_style_violations(
    violations: &mut Vec<DiCompletenessViolation>,
    snapshot: &BpmnDocumentSnapshot,
) {
    for diagram in &snapshot.root.diagrams {
        let diagram_id = diagram.diagram_id.as_deref();
        for label_style in &diagram.label_styles {
            if label_style.font.is_none() {
                violations.push(DiCompletenessViolation::label_style_font(
                    diagram_id,
                    label_style.style_id.as_deref(),
                ));
            }
        }
    }
}
