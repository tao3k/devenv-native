use super::evidence::{augment_evidence, root_context};
use super::snapshot_count::{
    snapshot_business_knowledge_model_count, snapshot_input_data_count,
    snapshot_item_definition_count, snapshot_knowledge_source_count,
};
use crate::dmn_model_api::DmnDocumentSnapshot;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn unsupported_decision_service_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    LintIssue::from_parts(
        "dmn.unsupported_decision_service",
        "DMN file contains decision service definitions but no executable decisions",
        format!(
            "Source '{source_id}' contains top-level `<decisionService>` definitions, but no executable `<decision>` elements."
        ),
        format!(
            "The bounded DMN evaluator still requires at least one executable `<decision>` in the source. This crate can now route one same-source `decisionService` as a thin local alias to one or more executable `outputDecision` targets, but metadata-only decision-service documents remain non-executable and their direct references are preserved primarily as snapshot evidence.{}",
            root_context(snapshot)
        ),
        vec![
            "If the source should be executable in this slice, expose at least one `<decision>` with exactly one nested `<decisionTable>`.".to_string(),
            "Do not invent decision-table rules from a `decisionService` contract unless the underlying decision logic is explicitly available.".to_string(),
            "Preserve any reported `outputDecision`, `encapsulatedDecision`, `inputDecision`, and `inputData` hrefs under `document_root.decision_services` when generating repair patches or adapter payloads.".to_string(),
            "If the file is intentionally metadata-only, preserve it as a non-executable DMN artifact and report unsupported `decisionService` execution.".to_string(),
        ],
        format!(
            "Inspect DMN source '{source_id}' and do not fabricate decision-table logic from `<decisionService>` metadata. Preserve reported `document_root.decision_services` references, then either expose the underlying executable `<decision>` elements with bounded `<decisionTable>` content, or keep the file non-executable and report that `decisionService` execution is unsupported in this slice."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
            }),
            snapshot,
            None,
        ),
    )
}

pub(super) fn unsupported_input_data_artifact_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let input_data_count = snapshot_input_data_count(snapshot);
    let noun = if input_data_count == 1 {
        "artifact"
    } else {
        "artifacts"
    };
    LintIssue::from_parts(
        "dmn.unsupported_input_data_artifact",
        "DMN file contains input-data artifacts but no executable decisions",
        format!(
            "Source '{source_id}' contains {input_data_count} top-level `<inputData>` {noun}, but no executable `<decision>` elements."
        ),
        format!(
            "The bounded DMN evaluator treats top-level input-data declarations as metadata only; they do not become executable decision logic without at least one local `<decision>`.{}",
            root_context(snapshot)
        ),
        vec![
            "If the source should be executable in this slice, add at least one `<decision>` with exactly one nested `<decisionTable>` that consumes the existing input-data surface.".to_string(),
            "Do not invent outputs, rules, or decision-table clauses only from `<inputData>` metadata unless the missing decision logic is explicit and lossless.".to_string(),
            "If the file is intentionally metadata-only, preserve it as a non-executable DMN artifact and report unsupported input-data-only execution.".to_string(),
        ],
        format!(
            "Inspect DMN source '{source_id}' and do not fabricate decision-table logic just from top-level `<inputData>` metadata. Either add one bounded executable `<decision>` with explicit local `<decisionTable>` logic, or keep the file non-executable and report unsupported input-data-only execution."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "input_data_count": input_data_count,
            }),
            snapshot,
            None,
        ),
    )
}

pub(super) fn unsupported_item_definition_document_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let item_definition_count = snapshot_item_definition_count(snapshot);
    let noun = if item_definition_count == 1 {
        "definition"
    } else {
        "definitions"
    };
    LintIssue::from_parts(
        "dmn.unsupported_item_definition_document",
        "DMN file contains item definitions but no executable decisions",
        format!(
            "Source '{source_id}' contains {item_definition_count} top-level `<itemDefinition>` {noun}, but no executable `<decision>` elements."
        ),
        format!(
            "The bounded DMN evaluator treats top-level item definitions as non-executable type metadata only; they do not become executable decision logic without at least one local `<decision>`.{}",
            root_context(snapshot)
        ),
        vec![
            "If the source should be executable in this slice, add at least one `<decision>` with exactly one nested `<decisionTable>` that explicitly consumes the existing type model.".to_string(),
            "Do not translate `<itemDefinition>` structures into guessed rules, outputs, or decision-table rows unless the missing local decision logic is explicit and lossless.".to_string(),
            "If the file is intentionally a shared type-model document, preserve it as a non-executable DMN document and report unsupported item-definition-only execution.".to_string(),
        ],
        format!(
            "Inspect DMN source '{source_id}' and do not fabricate decision-table logic just from top-level `<itemDefinition>` metadata. Either add one bounded executable `<decision>` with explicit local `<decisionTable>` logic that uses the existing type model, or keep the file non-executable and report unsupported item-definition-only execution."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "item_definition_count": item_definition_count,
            }),
            snapshot,
            None,
        ),
    )
}

pub(super) fn unsupported_knowledge_source_artifact_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let knowledge_source_count = snapshot_knowledge_source_count(snapshot);
    let noun = if knowledge_source_count == 1 {
        "artifact"
    } else {
        "artifacts"
    };
    LintIssue::from_parts(
        "dmn.unsupported_knowledge_source_artifact",
        "DMN file contains knowledge-source artifacts but no executable decisions",
        format!(
            "Source '{source_id}' contains {knowledge_source_count} top-level `<knowledgeSource>` {noun}, but no executable `<decision>` elements."
        ),
        format!(
            "The bounded DMN evaluator treats top-level knowledge-source declarations as governance metadata only; they do not provide executable decision rules without at least one local `<decision>`.{}",
            root_context(snapshot)
        ),
        vec![
            "If the source should be executable in this slice, add at least one `<decision>` with exactly one nested `<decisionTable>` that explicitly uses the existing knowledge-source surface.".to_string(),
            "Do not fabricate decision-table rules only from `<knowledgeSource>` metadata unless the missing local logic is explicit and lossless.".to_string(),
            "If the file is intentionally metadata-only, preserve it as a non-executable DMN artifact and report unsupported knowledge-source-only execution.".to_string(),
        ],
        format!(
            "Inspect DMN source '{source_id}' and do not fabricate decision-table logic just from top-level `<knowledgeSource>` metadata. Either add one bounded executable `<decision>` with explicit local `<decisionTable>` logic, or keep the file non-executable and report unsupported knowledge-source-only execution."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "knowledge_source_count": knowledge_source_count,
            }),
            snapshot,
            None,
        ),
    )
}

pub(super) fn unsupported_business_knowledge_model_artifact_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    let business_knowledge_model_count = snapshot_business_knowledge_model_count(snapshot);
    let noun = if business_knowledge_model_count == 1 {
        "artifact"
    } else {
        "artifacts"
    };
    LintIssue::from_parts(
        "dmn.unsupported_business_knowledge_model_artifact",
        "DMN file contains business-knowledge-model artifacts but no executable decisions",
        format!(
            "Source '{source_id}' contains {business_knowledge_model_count} top-level `<businessKnowledgeModel>` {noun}, but no executable `<decision>` elements."
        ),
        format!(
            "The bounded DMN evaluator does not execute top-level business-knowledge models directly in this slice; they do not become executable decision logic without at least one local `<decision>`. The bounded parser now preserves direct BKM invocable metadata, including `variable`, `encapsulatedLogic`, and any direct body placeholder, but runtime still does not execute that callable surface.{}",
            root_context(snapshot)
        ),
        vec![
            "If the source should be executable in this slice, add at least one `<decision>` with exactly one nested `<decisionTable>` that explicitly consumes or wraps the existing business-knowledge-model surface.".to_string(),
            "Do not inline or approximate `businessKnowledgeModel` invocable semantics into guessed decision-table rules unless the missing local decision logic is explicit and lossless.".to_string(),
            "Preserve any reported invocable metadata under `document_root.business_knowledge_models`, including `variable`, `encapsulatedLogic`, and direct body placeholders, when generating repair patches or adapter payloads.".to_string(),
            "If the file is intentionally metadata-only, preserve it as a non-executable DMN artifact and report unsupported business-knowledge-model-only execution.".to_string(),
        ],
        format!(
            "Inspect DMN source '{source_id}' and do not fabricate decision-table logic just from top-level `<businessKnowledgeModel>` metadata. Preserve reported `document_root.business_knowledge_models` metadata, including any preserved invocable `variable`, `encapsulatedLogic`, and direct body evidence, then either add one bounded executable `<decision>` with explicit local `<decisionTable>` logic, or keep the file non-executable and report unsupported business-knowledge-model-only execution."
        ),
        augment_evidence(
            json!({
                "source_id": source_id,
                "business_knowledge_model_count": business_knowledge_model_count,
            }),
            snapshot,
            None,
        ),
    )
}
