use serde_json::json;

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::bpmn_snapshot_api::snapshot_bpmn_source;
use crate::lint::bpmn::document_surface::shared::SNAPSHOT_EVIDENCE_LIMIT;
use crate::lint::bpmn::document_surface::summary::diagram_snapshot_summary;
use crate::lint_api::LintIssue;

use super::collect::duplicate_di_ids;
use super::model::DiIdentityViolation;

pub(in crate::lint::bpmn::document_surface) fn diagram_identity_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    let snapshot = snapshot_bpmn_source(source).ok()?;
    let duplicate_ids = duplicate_di_ids(&snapshot);
    if duplicate_ids.is_empty() {
        return None;
    }

    let source_id = &source.source_id;
    let evidence = json!({
        "source_id": source_id,
        "element": "BPMNDiagram",
        "deferred_family": "diagram",
        "snapshot_available": true,
        "duplicate_di_id_count": duplicate_ids.len(),
        "duplicate_di_ids": duplicate_ids
            .iter()
            .take(SNAPSHOT_EVIDENCE_LIMIT)
            .map(DiIdentityViolation::evidence)
            .collect::<Vec<_>>(),
        "duplicate_di_ids_truncated": duplicate_ids.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "snapshot": diagram_snapshot_summary(&snapshot),
    });

    Some(LintIssue::new(
        "bpmn.duplicate_di_id",
        "BPMN diagram interchange metadata reuses an id",
        format!(
            "Source '{source_id}' contains duplicate BPMN DI identifiers that make diagram interchange ambiguous."
        ),
        "BPMN DI stays metadata-only in the bounded runtime, but DI identifiers must remain unique so diagram elements, labels, styles, and local references can be traced without ambiguity.",
        vec![
            "Give each BPMN DI `BPMNDiagram`, `BPMNPlane`, `BPMNShape`, `BPMNEdge`, `BPMNLabel`, and `BPMNLabelStyle` a unique `id` value.".to_string(),
            "Retarget DI-local references after renaming duplicated shape, edge, label, or label-style ids.".to_string(),
            "Keep executable behavior in process flow, events, tasks, gateways, and data mappings rather than in diagram identity metadata.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' so every BPMN DI id is unique across the source. Preserve diagram metadata for interchange, but do not rely on duplicated DI ids for runtime behavior."
        ),
        evidence,
    ))
}
