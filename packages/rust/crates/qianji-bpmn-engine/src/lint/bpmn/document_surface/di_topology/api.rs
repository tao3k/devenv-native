use serde_json::json;

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::bpmn_snapshot_api::snapshot_bpmn_source;
use crate::lint::bpmn::document_surface::SNAPSHOT_EVIDENCE_LIMIT;
use crate::lint::bpmn::document_surface::summary::diagram_snapshot_summary;
use crate::lint_api::LintIssue;

use super::model::DiTopologyViolation;
use super::scan::collect_topology_violations;

pub(in crate::lint::bpmn::document_surface) fn diagram_topology_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    let violations = collect_topology_violations(source);
    if violations.is_empty() {
        return None;
    }

    let snapshot = snapshot_bpmn_source(source).ok();
    let source_id = &source.source_id;
    let evidence = json!({
        "source_id": source_id,
        "element": "BPMNDiagram",
        "deferred_family": "diagram",
        "snapshot_available": snapshot.is_some(),
        "invalid_topology_count": violations.len(),
        "invalid_topology": violations
            .iter()
            .take(SNAPSHOT_EVIDENCE_LIMIT)
            .map(DiTopologyViolation::evidence)
            .collect::<Vec<_>>(),
        "invalid_topology_truncated": violations.len() > SNAPSHOT_EVIDENCE_LIMIT,
        "snapshot": snapshot.as_ref().map(diagram_snapshot_summary),
    });

    Some(LintIssue::from_parts(
        "bpmn.invalid_di_plane_topology",
        "BPMN diagram interchange plane topology is invalid",
        format!(
            "Source '{source_id}' contains BPMN DI diagram-plane metadata that is not shaped for stable interchange."
        ),
        "BPMN DI stays metadata-only in the bounded runtime, but every BPMN diagram should contain exactly one direct BPMN plane so diagram metadata can be preserved and audited without ambiguous container structure.",
        vec![
            "Add exactly one direct `bpmndi:BPMNPlane` child to each `bpmndi:BPMNDiagram`.".to_string(),
            "Merge or split extra planes into separate `bpmndi:BPMNDiagram` elements before relying on the source for interchange.".to_string(),
            "Move orphan `bpmndi:BPMNPlane` elements under the owning `bpmndi:BPMNDiagram`.".to_string(),
            "Keep executable behavior in process flow, events, tasks, gateways, and data mappings rather than in diagram container metadata.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' so every BPMN DI diagram has exactly one direct `BPMNPlane`, and every `BPMNPlane` is a direct child of its owning `BPMNDiagram`. Preserve diagram metadata for interchange, but do not rely on layout containers for runtime behavior."
        ),
        evidence,
    ))
}
