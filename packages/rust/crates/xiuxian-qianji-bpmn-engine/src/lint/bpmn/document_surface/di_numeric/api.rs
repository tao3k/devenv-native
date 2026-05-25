use serde_json::json;

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint::bpmn::document_surface::SNAPSHOT_EVIDENCE_LIMIT;
use crate::lint_api::LintIssue;

use super::model::DiNumericViolation;
use super::scan::collect_numeric_violations;

pub(in crate::lint::bpmn::document_surface) fn diagram_numeric_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    let violations = collect_numeric_violations(source)?;
    if violations.is_empty() {
        return None;
    }

    let source_id = &source.source_id;
    let evidence = json!({
        "source_id": source_id,
        "element": "BPMNDiagram",
        "deferred_family": "diagram",
        "invalid_numeric_count": violations.len(),
        "invalid_numerics": violations
            .iter()
            .take(SNAPSHOT_EVIDENCE_LIMIT)
            .map(DiNumericViolation::evidence)
            .collect::<Vec<_>>(),
        "invalid_numerics_truncated": violations.len() > SNAPSHOT_EVIDENCE_LIMIT,
    });

    Some(LintIssue::from_parts(
        "bpmn.invalid_di_numeric",
        "BPMN diagram interchange metadata uses an invalid numeric value",
        format!(
            "Source '{source_id}' contains BPMN DI, DC, or DI numeric attributes that are not finite numeric values."
        ),
        "BPMN DI stays metadata-only in the bounded runtime, but numeric diagram attributes must remain finite numeric values so interchange tools can round-trip coordinates, resolution, and font size without non-standard coercion.",
        vec![
            "Use finite numeric values for BPMN DI, DC, and DI numeric attributes.".to_string(),
            "Check `BPMNDiagram` `resolution`.".to_string(),
            "Check `dc:Bounds` `x`, `y`, `width`, and `height` values.".to_string(),
            "Check `di:waypoint` `x` and `y` values.".to_string(),
            "Check `dc:Font` `size`.".to_string(),
            "Keep executable behavior in BPMN semantic elements; do not encode runtime behavior in diagram coordinates.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' so every BPMN DI, DC, and DI numeric diagram attribute uses a finite numeric value. Preserve diagram metadata for interchange, but do not rely on layout numbers for runtime behavior."
        ),
        evidence,
    ))
}
