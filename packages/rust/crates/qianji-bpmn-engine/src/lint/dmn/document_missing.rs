use super::document_business_context::{
    unsupported_organization_unit_document_issue, unsupported_performance_indicator_document_issue,
    unsupported_text_annotation_document_issue,
};
use super::document_root_artifacts::{
    unsupported_business_knowledge_model_artifact_issue, unsupported_decision_service_issue,
    unsupported_input_data_artifact_issue, unsupported_item_definition_document_issue,
    unsupported_knowledge_source_artifact_issue,
};
use super::document_structures::{
    unsupported_association_document_issue, unsupported_dmndi_document_issue,
    unsupported_element_collection_document_issue, unsupported_group_document_issue,
};
use super::evidence::{augment_evidence, root_context};
use super::snapshot_classify::{
    snapshot_has_decision_service, snapshot_has_only_association,
    snapshot_has_only_business_knowledge_model, snapshot_has_only_dmndi,
    snapshot_has_only_element_collection, snapshot_has_only_group, snapshot_has_only_input_data,
    snapshot_has_only_item_definition, snapshot_has_only_knowledge_source,
    snapshot_has_only_organization_unit, snapshot_has_only_performance_indicator,
    snapshot_has_only_text_annotation,
};
use crate::dmn_model_api::DmnDocumentSnapshot;
use crate::lint_api::LintIssue;
use serde_json::json;

pub(super) fn missing_dmn_decision_issue(
    source_id: &str,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> LintIssue {
    if snapshot_has_decision_service(snapshot) {
        unsupported_decision_service_issue(source_id, snapshot)
    } else if snapshot_has_only_item_definition(snapshot) {
        unsupported_item_definition_document_issue(source_id, snapshot)
    } else if snapshot_has_only_input_data(snapshot) {
        unsupported_input_data_artifact_issue(source_id, snapshot)
    } else if snapshot_has_only_knowledge_source(snapshot) {
        unsupported_knowledge_source_artifact_issue(source_id, snapshot)
    } else if snapshot_has_only_business_knowledge_model(snapshot) {
        unsupported_business_knowledge_model_artifact_issue(source_id, snapshot)
    } else if snapshot_has_only_organization_unit(snapshot) {
        unsupported_organization_unit_document_issue(source_id, snapshot)
    } else if snapshot_has_only_performance_indicator(snapshot) {
        unsupported_performance_indicator_document_issue(source_id, snapshot)
    } else if snapshot_has_only_text_annotation(snapshot) {
        unsupported_text_annotation_document_issue(source_id, snapshot)
    } else if snapshot_has_only_association(snapshot) {
        unsupported_association_document_issue(source_id, snapshot)
    } else if snapshot_has_only_element_collection(snapshot) {
        unsupported_element_collection_document_issue(source_id, snapshot)
    } else if snapshot_has_only_group(snapshot) {
        unsupported_group_document_issue(source_id, snapshot)
    } else if snapshot_has_only_dmndi(snapshot) {
        unsupported_dmndi_document_issue(source_id, snapshot)
    } else {
        LintIssue::new(
            "dmn.missing_decision",
            "DMN file contains no decisions",
            format!("Source '{source_id}' does not contain any `<decision>` elements."),
            format!(
                "The bounded DMN contract still requires at least one `<decision>`, even though one source may now carry multiple bounded decisions.{}",
                root_context(snapshot)
            ),
            vec![
                "Add one `<decision>` element to the file.".to_string(),
                "Keep each decision bounded to exactly one `<decisionTable>` for the current v1 contract.".to_string(),
            ],
            format!(
                "Rewrite DMN source '{source_id}' so it contains at least one `<decision>` element, and keep each decision bounded to exactly one nested `<decisionTable>`."
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
}
