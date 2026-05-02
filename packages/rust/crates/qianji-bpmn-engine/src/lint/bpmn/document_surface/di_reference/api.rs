use serde_json::json;

use crate::bpmn_model_api::BpmnDocumentSnapshot;
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::bpmn_snapshot_api::snapshot_bpmn_source;
use crate::lint::bpmn::document_surface::SNAPSHOT_EVIDENCE_LIMIT;
use crate::lint::bpmn::document_surface::di_semantic::SemanticElementIndex;
use crate::lint::bpmn::document_surface::summary::diagram_snapshot_summary;
use crate::lint_api::LintIssue;

use super::local::collect_local_reference_violations;
use super::model::DiReferenceViolation;
use super::semantic::collect_semantic_reference_violations;

pub(in crate::lint::bpmn::document_surface) fn diagram_reference_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    let snapshot = snapshot_bpmn_source(source).ok()?;
    let semantic_index = SemanticElementIndex::from_source(source)?;
    let missing_references = missing_di_references(&snapshot, &semantic_index);
    if missing_references.is_empty() {
        return None;
    }

    let source_id = &source.source_id;
    let evidence = json!({
        "source_id": source_id,
        "element": "BPMNDiagram",
        "deferred_family": "diagram",
        "snapshot_available": true,
        "semantic_id_count": semantic_index.len(),
        "invalid_reference_count": missing_references.len(),
        "invalid_references": missing_references
            .iter()
            .take(SNAPSHOT_EVIDENCE_LIMIT)
            .map(DiReferenceViolation::evidence)
            .collect::<Vec<_>>(),
        "invalid_references_truncated": missing_references.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "snapshot": diagram_snapshot_summary(&snapshot),
    });

    Some(LintIssue::new(
        "bpmn.invalid_di_reference",
        "BPMN diagram reference points at a missing element",
        format!(
            "Source '{source_id}' contains BPMN DI references that do not resolve to expected BPMN or DI ids."
        ),
        "BPMN DI stays metadata-only in the bounded runtime, but its semantic and DI-local links must remain traceable for diagram interchange and round-trip compatibility.",
        vec![
            "Retarget each missing DI `bpmnElement` reference to an existing BPMN semantic id in the same source.".to_string(),
            "Retarget each missing DI `sourceElement`, `targetElement`, or `labelStyle` reference to an existing DI id in the same diagram scope.".to_string(),
            "Remove stale DI shapes or edges if the referenced process element was deleted.".to_string(),
            "Keep executable behavior in process flow, events, tasks, gateways, and data mappings rather than in diagram coordinates.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' so every BPMN DI `bpmnElement` reference points at an existing semantic BPMN id and every DI-local reference points at an existing DI id in its diagram scope. Preserve valid diagram metadata, but remove or retarget stale DI elements before relying on the source for interchange."
        ),
        evidence,
    ))
}

fn missing_di_references(
    snapshot: &BpmnDocumentSnapshot,
    semantic_index: &SemanticElementIndex,
) -> Vec<DiReferenceViolation> {
    let mut violations = Vec::new();
    for diagram in &snapshot.root.diagrams {
        let diagram_id = diagram.diagram_id.as_deref();
        let Some(plane) = diagram.plane.as_ref() else {
            continue;
        };
        collect_semantic_reference_violations(
            &mut violations,
            diagram_id,
            plane.plane_id.as_deref(),
            plane,
            semantic_index,
        );
        collect_local_reference_violations(&mut violations, diagram_id, diagram, plane);
    }
    violations
}
