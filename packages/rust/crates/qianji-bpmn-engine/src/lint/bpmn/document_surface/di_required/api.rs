use serde_json::json;

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint::bpmn::document_surface::SNAPSHOT_EVIDENCE_LIMIT;
use crate::lint_api::LintIssue;

use super::model::DiRequiredAttributeViolation;
use super::scan::collect_required_attribute_violations;

pub(in crate::lint::bpmn::document_surface) fn diagram_required_attribute_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    let violations = collect_required_attribute_violations(source)?;
    if violations.is_empty() {
        return None;
    }

    let source_id = &source.source_id;
    let evidence = json!({
        "source_id": source_id,
        "element": "BPMNDiagram",
        "deferred_family": "diagram",
        "missing_required_attribute_count": violations.len(),
        "missing_required_attributes": violations
            .iter()
            .take(SNAPSHOT_EVIDENCE_LIMIT)
            .map(DiRequiredAttributeViolation::evidence)
            .collect::<Vec<_>>(),
        "missing_required_attributes_truncated": violations.len() > SNAPSHOT_EVIDENCE_LIMIT,
    });

    Some(LintIssue::new(
        "bpmn.missing_di_required_attribute",
        "BPMN diagram interchange metadata is missing a required attribute",
        format!(
            "Source '{source_id}' contains BPMN DI geometry metadata without required DC or DI attributes."
        ),
        "BPMN DI stays metadata-only in the bounded runtime, but required geometry attributes must be present so interchange tools can round-trip bounds and waypoints without inventing coordinates.",
        vec![
            "Add `x`, `y`, `width`, and `height` to every `dc:Bounds` element.".to_string(),
            "Add `x` and `y` to every `di:waypoint` element.".to_string(),
            "Use finite numeric values for those attributes; geometry quality and positive dimensions are separate concerns.".to_string(),
            "Keep executable behavior in BPMN semantic elements; do not encode runtime behavior in diagram coordinates.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' so every `dc:Bounds` and `di:waypoint` element carries its required attributes. Preserve diagram metadata for interchange, but do not rely on layout coordinates for runtime behavior."
        ),
        evidence,
    ))
}
