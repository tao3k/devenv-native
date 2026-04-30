use serde_json::json;

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint::bpmn::document_surface::shared::SNAPSHOT_EVIDENCE_LIMIT;
use crate::lint_api::LintIssue;

use super::model::DiNamespaceViolation;
use super::scan::collect_namespace_violations;

pub(in crate::lint::bpmn::document_surface) fn diagram_namespace_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    let violations = collect_namespace_violations(source)?;
    if violations.is_empty() {
        return None;
    }

    let source_id = &source.source_id;
    let evidence = json!({
        "source_id": source_id,
        "element": "BPMNDiagram",
        "deferred_family": "diagram",
        "invalid_namespace_count": violations.len(),
        "invalid_namespaces": violations
            .iter()
            .take(SNAPSHOT_EVIDENCE_LIMIT)
            .map(DiNamespaceViolation::evidence)
            .collect::<Vec<_>>(),
        "invalid_namespaces_truncated": violations.len() > SNAPSHOT_EVIDENCE_LIMIT,
    });

    Some(LintIssue::new(
        "bpmn.invalid_di_namespace",
        "BPMN diagram interchange metadata uses non-standard XML namespaces",
        format!(
            "Source '{source_id}' contains BPMN DI, DC, or DI diagram-interchange elements bound to non-standard XML namespace URIs."
        ),
        "Native BPMN diagram interchange requires the standard BPMN DI, DC, and DI namespace URIs. The runtime keeps diagram metadata passive, but malformed namespaces break standard BPMN interchange tooling.",
        vec![
            "Bind BPMN DI elements such as `BPMNDiagram`, `BPMNPlane`, `BPMNShape`, `BPMNEdge`, `BPMNLabel`, and `BPMNLabelStyle` to `http://www.omg.org/spec/BPMN/20100524/DI`.".to_string(),
            "Bind DC elements such as `Bounds` and `Font` to `http://www.omg.org/spec/DD/20100524/DC`.".to_string(),
            "Bind DI waypoint elements to `http://www.omg.org/spec/DD/20100524/DI`.".to_string(),
            "Keep executable behavior in BPMN semantic elements; do not use custom diagram namespaces as a runtime contract.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' so every BPMN DI, DC, and DI diagram-interchange element is bound to the standard OMG namespace URI expected for that element."
        ),
        evidence,
    ))
}
