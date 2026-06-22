use serde_json::json;

use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint::bpmn::document_surface::SNAPSHOT_EVIDENCE_LIMIT;
use crate::lint_api::LintIssue;

use super::model::DiBooleanViolation;
use super::scan::collect_boolean_violations;

pub(in crate::lint::bpmn::document_surface) fn diagram_boolean_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    let violations = collect_boolean_violations(source)?;
    if violations.is_empty() {
        return None;
    }

    let source_id = &source.source_id;
    let evidence = json!({
        "source_id": source_id,
        "element": "BPMNDiagram",
        "deferred_family": "diagram",
        "invalid_boolean_count": violations.len(),
        "invalid_booleans": violations
            .iter()
            .take(SNAPSHOT_EVIDENCE_LIMIT)
            .map(DiBooleanViolation::evidence)
            .collect::<Vec<_>>(),
        "invalid_booleans_truncated": violations.len() > SNAPSHOT_EVIDENCE_LIMIT,
    });

    Some(LintIssue::from_parts(
        "bpmn.invalid_di_boolean",
        "BPMN diagram interchange metadata uses an invalid boolean value",
        format!(
            "Source '{source_id}' contains BPMN DI or DC boolean attributes outside the XML Schema boolean lexical values."
        ),
        "Native BPMN DI preserves standard diagram interchange, and boolean-valued diagram attributes must use standard XML boolean literals so interchange tools can round-trip them without silent coercion.",
        vec![
            "Use `true`, `false`, `1`, or `0` for BPMN DI and DC boolean attributes.".to_string(),
            "Check `BPMNShape` display flags such as `isHorizontal`, `isExpanded`, `isMarkerVisible`, and `isMessageVisible`.".to_string(),
            "Check `dc:Font` style flags such as `isBold`, `isItalic`, `isUnderline`, and `isStrikeThrough`.".to_string(),
            "Remove the boolean-valued display hint if the diagram does not need it.".to_string(),
        ],
        format!(
            "Repair BPMN source '{source_id}' so every BPMN DI and DC boolean-valued diagram attribute uses one of `true`, `false`, `1`, or `0`. Preserve diagram metadata for interchange, but do not rely on non-standard boolean spellings."
        ),
        evidence,
    ))
}
