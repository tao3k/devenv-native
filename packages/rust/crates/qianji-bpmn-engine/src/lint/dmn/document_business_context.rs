use super::evidence::{augment_evidence, root_context};
use super::snapshot_count::{
    snapshot_organization_unit_count, snapshot_performance_indicator_count,
    snapshot_text_annotation_count,
};
use crate::dmn_model_api::DmnDocumentSnapshot;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn unsupported_organization_unit_document_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let organization_unit_count = snapshot_organization_unit_count(snapshot);
    let noun = if organization_unit_count == 1 {
        "element"
    } else {
        "elements"
    };
    LintIssue::from_parts(
        "dmn.unsupported_organization_unit_document",
        "DMN file contains organization-unit business context but no executable decisions",
        format!(
            "Source '{source_id}' contains {organization_unit_count} top-level `<organizationUnit>` {noun}, but no executable `<decision>` elements."
        ),
        format!(
            "The bounded DMN evaluator treats top-level organization-unit declarations as governance metadata only; they do not become executable decision logic without at least one local `<decision>`.{}",
            root_context(snapshot)
        ),
        vec![
            "If the source should be executable in this slice, add at least one `<decision>` with exactly one nested `<decisionTable>` and keep organization ownership metadata separate from decision logic.".to_string(),
            "Do not invent approval rules, routing, or decision-table clauses just from `<organizationUnit>` metadata unless the missing local decision logic is explicit and lossless.".to_string(),
            "If the file is intentionally governance-only, preserve it as a non-executable DMN document and report unsupported organization-unit-only execution.".to_string(),
        ],
        format!(
            "Inspect DMN source '{source_id}' and do not fabricate decision-table logic just from top-level `<organizationUnit>` metadata. Either add one bounded executable `<decision>` with explicit local `<decisionTable>` logic, or keep the file non-executable and report unsupported organization-unit-only execution."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "organization_unit_count": organization_unit_count,
            }),
            snapshot,
            None,
        ),
    )
}

pub(super) fn unsupported_performance_indicator_document_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let performance_indicator_count = snapshot_performance_indicator_count(snapshot);
    let noun = if performance_indicator_count == 1 {
        "element"
    } else {
        "elements"
    };
    LintIssue::from_parts(
        "dmn.unsupported_performance_indicator_document",
        "DMN file contains performance-indicator business context but no executable decisions",
        format!(
            "Source '{source_id}' contains {performance_indicator_count} top-level `<performanceIndicator>` {noun}, but no executable `<decision>` elements."
        ),
        format!(
            "The bounded DMN evaluator treats top-level performance indicators as monitoring metadata only; they do not become executable decision logic without at least one local `<decision>`.{}",
            root_context(snapshot)
        ),
        vec![
            "If the source should be executable in this slice, add at least one `<decision>` with exactly one nested `<decisionTable>` and keep performance monitoring metadata separate from decision logic.".to_string(),
            "Do not invent thresholds, targets, or decision-table clauses just from `<performanceIndicator>` metadata unless the missing local decision logic is explicit and lossless.".to_string(),
            "If the file is intentionally governance-only, preserve it as a non-executable DMN document and report unsupported performance-indicator-only execution.".to_string(),
        ],
        format!(
            "Inspect DMN source '{source_id}' and do not fabricate decision-table logic just from top-level `<performanceIndicator>` metadata. Either add one bounded executable `<decision>` with explicit local `<decisionTable>` logic, or keep the file non-executable and report unsupported performance-indicator-only execution."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "performance_indicator_count": performance_indicator_count,
            }),
            snapshot,
            None,
        ),
    )
}

pub(super) fn unsupported_text_annotation_document_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let text_annotation_count = snapshot_text_annotation_count(snapshot);
    let noun = if text_annotation_count == 1 {
        "annotation"
    } else {
        "annotations"
    };
    LintIssue::from_parts(
        "dmn.unsupported_text_annotation_document",
        "DMN file contains text annotations but no executable decisions",
        format!(
            "Source '{source_id}' contains {text_annotation_count} top-level `<textAnnotation>` {noun}, but no executable `<decision>` elements."
        ),
        format!(
            "The bounded DMN evaluator treats top-level text annotations as descriptive metadata only; they do not become executable decision logic without at least one local `<decision>`.{}",
            root_context(snapshot)
        ),
        vec![
            "If the source should be executable in this slice, add at least one `<decision>` with exactly one nested `<decisionTable>` and keep descriptive annotations separate from decision logic.".to_string(),
            "Do not invent rules, outputs, or decision-table clauses just from `<textAnnotation>` prose unless the missing local decision logic is explicit and lossless.".to_string(),
            "If the file is intentionally documentation-only, preserve it as a non-executable DMN document and report unsupported text-annotation-only execution.".to_string(),
        ],
        format!(
            "Inspect DMN source '{source_id}' and do not fabricate decision-table logic just from top-level `<textAnnotation>` metadata. Either add one bounded executable `<decision>` with explicit local `<decisionTable>` logic, or keep the file non-executable and report unsupported text-annotation-only execution."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "text_annotation_count": text_annotation_count,
            }),
            snapshot,
            None,
        ),
    )
}
