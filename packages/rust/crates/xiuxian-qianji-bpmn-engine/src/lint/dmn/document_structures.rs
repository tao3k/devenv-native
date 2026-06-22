use super::evidence::{augment_evidence, dmndi_metadata_context, root_context};
use super::snapshot_count::{
    snapshot_association_count, snapshot_dmndi_count, snapshot_element_collection_count,
    snapshot_group_count,
};
use crate::dmn_model_api::DmnDocumentSnapshot;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn unsupported_association_document_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let association_count = snapshot_association_count(snapshot);
    let noun = if association_count == 1 {
        "association"
    } else {
        "associations"
    };
    LintIssue::from_parts(
        "dmn.unsupported_association_document",
        "DMN file contains associations but no executable decisions",
        format!(
            "Source '{source_id}' contains {association_count} top-level `<association>` {noun}, but no executable `<decision>` elements."
        ),
        format!(
            "The bounded DMN evaluator treats top-level associations as document-structure metadata only; they do not become executable decision logic without at least one local `<decision>`.{}",
            root_context(snapshot)
        ),
        vec![
            "If the source should be executable in this slice, add at least one `<decision>` with exactly one nested `<decisionTable>` and keep cross-element association metadata separate from decision logic.".to_string(),
            "Do not invent dependencies, routing, or decision-table clauses just from `<association>` links unless the missing local decision logic is explicit and lossless.".to_string(),
            "If the file is intentionally metadata-only, preserve it as a non-executable DMN document and report unsupported association-only execution.".to_string(),
        ],
        format!(
            "Inspect DMN source '{source_id}' and do not fabricate decision-table logic just from top-level `<association>` metadata. Either add one bounded executable `<decision>` with explicit local `<decisionTable>` logic, or keep the file non-executable and report unsupported association-only execution."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "association_count": association_count,
            }),
            snapshot,
            None,
        ),
    )
}

pub(super) fn unsupported_element_collection_document_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let element_collection_count = snapshot_element_collection_count(snapshot);
    let noun = if element_collection_count == 1 {
        "collection"
    } else {
        "collections"
    };
    LintIssue::from_parts(
        "dmn.unsupported_element_collection_document",
        "DMN file contains element collections but no executable decisions",
        format!(
            "Source '{source_id}' contains {element_collection_count} top-level `<elementCollection>` {noun}, but no executable `<decision>` elements."
        ),
        format!(
            "The bounded DMN evaluator treats top-level element collections as structural metadata only; they do not become executable decision logic without at least one local `<decision>`.{}",
            root_context(snapshot)
        ),
        vec![
            "If the source should be executable in this slice, add at least one `<decision>` with exactly one nested `<decisionTable>` and keep structural grouping metadata separate from decision logic.".to_string(),
            "Do not invent grouped members, outputs, or decision-table clauses just from `<elementCollection>` metadata unless the missing local decision logic is explicit and lossless.".to_string(),
            "If the file is intentionally metadata-only, preserve it as a non-executable DMN document and report unsupported element-collection-only execution.".to_string(),
        ],
        format!(
            "Inspect DMN source '{source_id}' and do not fabricate decision-table logic just from top-level `<elementCollection>` metadata. Either add one bounded executable `<decision>` with explicit local `<decisionTable>` logic, or keep the file non-executable and report unsupported element-collection-only execution."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "element_collection_count": element_collection_count,
            }),
            snapshot,
            None,
        ),
    )
}

pub(super) fn unsupported_group_document_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let group_count = snapshot_group_count(snapshot);
    let noun = if group_count == 1 { "group" } else { "groups" };
    LintIssue::from_parts(
        "dmn.unsupported_group_document",
        "DMN file contains group artifacts but no executable decisions",
        format!(
            "Source '{source_id}' contains {group_count} top-level `<group>` {noun}, but no executable `<decision>` elements."
        ),
        format!(
            "The bounded DMN evaluator treats top-level groups as non-executable structural metadata only; they do not become executable decision logic without at least one local `<decision>`.{}",
            root_context(snapshot)
        ),
        vec![
            "If the source should be executable in this slice, add at least one `<decision>` with exactly one nested `<decisionTable>` and keep visual grouping metadata separate from decision logic.".to_string(),
            "Do not invent rules, grouped behavior, or decision-table clauses just from `<group>` metadata unless the missing local decision logic is explicit and lossless.".to_string(),
            "If the file is intentionally structure-only, preserve it as a non-executable DMN document and report unsupported group-only execution.".to_string(),
        ],
        format!(
            "Inspect DMN source '{source_id}' and do not fabricate decision-table logic just from top-level `<group>` metadata. Either add one bounded executable `<decision>` with explicit local `<decisionTable>` logic, or keep the file non-executable and report unsupported group-only execution."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "group_count": group_count,
            }),
            snapshot,
            None,
        ),
    )
}

pub(super) fn unsupported_dmndi_document_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let dmndi_count = snapshot_dmndi_count(snapshot);
    let noun = if dmndi_count == 1 { "block" } else { "blocks" };
    LintIssue::from_parts(
        "dmn.unsupported_dmndi_document",
        "DMN file contains diagram interchange metadata but no executable decisions",
        format!(
            "Source '{source_id}' contains {dmndi_count} top-level `<dmndi:DMNDI>` {noun}, but no executable `<decision>` elements.{}",
            dmndi_metadata_context(snapshot)
        ),
        format!(
            "The bounded DMN evaluator treats top-level DMNDI blocks as diagram-interchange metadata only; they do not become executable decision logic without at least one local `<decision>`.{}",
            root_context(snapshot)
        ),
        vec![
            "If the source should be executable in this slice, add at least one `<decision>` with exactly one nested `<decisionTable>` and keep DMNDI diagram metadata separate from decision logic.".to_string(),
            "Do not invent rules, shapes, or decision-table clauses just from `<dmndi:DMNDI>` content unless the missing local decision logic is explicit and lossless.".to_string(),
            "If the file is intentionally diagram-only, preserve it as a non-executable DMN document and report unsupported DMNDI-only execution.".to_string(),
        ],
        format!(
            "Inspect DMN source '{source_id}' and do not fabricate decision-table logic just from top-level `<dmndi:DMNDI>` metadata. Either add one bounded executable `<decision>` with explicit local `<decisionTable>` logic, or keep the file non-executable and report unsupported DMNDI-only execution."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "dmndi_count": dmndi_count,
            }),
            snapshot,
            None,
        ),
    )
}
