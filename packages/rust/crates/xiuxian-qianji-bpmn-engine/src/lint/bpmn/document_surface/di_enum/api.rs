use serde_json::json;

use crate::bpmn_model_api::{BpmnDocumentSnapshot, BpmnPlaneSnapshot};
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::bpmn_snapshot_api::snapshot_bpmn_source;
use crate::lint::bpmn::document_surface::SNAPSHOT_EVIDENCE_LIMIT;
use crate::lint::bpmn::document_surface::summary::diagram_snapshot_summary;
use crate::lint_api::LintIssue;

use super::model::DiEnumViolation;

const PARTICIPANT_BAND_KIND_VALUES: &[&str] = &[
    "top_initiating",
    "middle_initiating",
    "bottom_initiating",
    "top_non_initiating",
    "middle_non_initiating",
    "bottom_non_initiating",
];

const MESSAGE_VISIBLE_KIND_VALUES: &[&str] = &["initiating", "non_initiating"];

pub(in crate::lint::bpmn::document_surface) fn diagram_enum_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    let snapshot = snapshot_bpmn_source(source).ok()?;
    let violations = invalid_di_enum_values(&snapshot);
    if violations.is_empty() {
        return None;
    }

    let source_id = &source.source_id;
    let evidence = json!({
        "source_id": source_id,
        "element": "BPMNDiagram",
        "deferred_family": "diagram",
        "snapshot_available": true,
        "invalid_enum_count": violations.len(),
        "invalid_enums": violations
            .iter()
            .take(SNAPSHOT_EVIDENCE_LIMIT)
            .map(DiEnumViolation::evidence)
            .collect::<Vec<_>>(),
        "invalid_enums_truncated": violations.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "snapshot": diagram_snapshot_summary(&snapshot),
    });

    Some(LintIssue::from_parts(
        "bpmn.invalid_di_enum",
        "BPMN diagram interchange metadata uses an invalid enum value",
        format!(
            "Source '{source_id}' contains BPMN DI enum-valued attributes outside the standard BPMNDI schema values."
        ),
        "Native BPMN DI preserves standard diagram interchange, and enum-valued diagram attributes must stay inside the standard BPMNDI vocabulary so interchange tools can round-trip them.",
        vec![
            format!(
                "Use one of `{}` for `BPMNShape` `participantBandKind`.",
                PARTICIPANT_BAND_KIND_VALUES.join("`, `")
            ),
            format!(
                "Use one of `{}` for `BPMNEdge` `messageVisibleKind`.",
                MESSAGE_VISIBLE_KIND_VALUES.join("`, `")
            ),
            "Remove the enum-valued attribute if the diagram does not need that display hint.".to_string(),
            "Keep executable behavior in BPMN semantic elements; do not encode runtime behavior in diagram display hints.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' so each BPMN DI enum-valued attribute uses only the standard BPMNDI values. Preserve diagram metadata for interchange, but remove or retarget non-standard display hints."
        ),
        evidence,
    ))
}

fn invalid_di_enum_values(snapshot: &BpmnDocumentSnapshot) -> Vec<DiEnumViolation> {
    let mut violations = Vec::new();
    for diagram in &snapshot.root.diagrams {
        let diagram_id = diagram.diagram_id.as_deref();
        let Some(plane) = diagram.plane.as_ref() else {
            continue;
        };
        collect_plane_enum_violations(&mut violations, diagram_id, plane);
    }
    violations
}

fn collect_plane_enum_violations(
    violations: &mut Vec<DiEnumViolation>,
    diagram_id: Option<&str>,
    plane: &BpmnPlaneSnapshot,
) {
    let plane_id = plane.plane_id.as_deref();
    for shape in &plane.shapes {
        if let Some(value) = shape.participant_band_kind.as_deref()
            && !PARTICIPANT_BAND_KIND_VALUES.contains(&value)
        {
            violations.push(DiEnumViolation::shape(
                diagram_id,
                plane_id,
                shape.shape_id.as_deref(),
                "participantBandKind",
                value,
                PARTICIPANT_BAND_KIND_VALUES,
            ));
        }
    }
    for edge in &plane.edges {
        if let Some(value) = edge.message_visible_kind.as_deref()
            && !MESSAGE_VISIBLE_KIND_VALUES.contains(&value)
        {
            violations.push(DiEnumViolation::edge(
                diagram_id,
                plane_id,
                edge.edge_id.as_deref(),
                "messageVisibleKind",
                value,
                MESSAGE_VISIBLE_KIND_VALUES,
            ));
        }
    }
}
