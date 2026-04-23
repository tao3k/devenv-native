use super::document_missing::missing_dmn_decision_issue;
use super::document_namespace::{
    missing_dmn_attribute_issue, missing_dmn_model_namespace_issue, unsupported_dmn_import_issue,
    unsupported_dmn_model_namespace_issue,
};
use super::document_root::{
    invalid_dmn_root_element_issue, invalid_dmn_xml_issue, missing_dmn_root_issue,
};
use crate::dmn_model_api::DmnDocumentSnapshot;
use crate::error::BpmnEngineError;
use crate::lint_api::LintIssue;

pub(super) fn issue_from_dmn_document_error(
    error: &BpmnEngineError,
    snapshot: Option<&DmnDocumentSnapshot>,
) -> Option<LintIssue> {
    Some(match error {
        BpmnEngineError::InvalidDmnXml { source_id, message } => {
            invalid_dmn_xml_issue(source_id, message)
        }
        BpmnEngineError::MissingDmnRootElement { source_id } => {
            missing_dmn_root_issue(source_id, snapshot)
        }
        BpmnEngineError::UnsupportedDmnRootElement { source_id, element } => {
            invalid_dmn_root_element_issue(source_id, element, snapshot)
        }
        BpmnEngineError::MissingDmnModelNamespace { source_id } => {
            missing_dmn_model_namespace_issue(source_id, snapshot)
        }
        BpmnEngineError::UnsupportedDmnModelNamespace {
            source_id,
            model_namespace_uri,
        } => unsupported_dmn_model_namespace_issue(source_id, model_namespace_uri, snapshot),
        BpmnEngineError::UnsupportedDmnImport { source_id } => {
            unsupported_dmn_import_issue(source_id, snapshot)
        }
        BpmnEngineError::MissingDmnAttribute {
            source_id,
            element,
            attribute,
        } => missing_dmn_attribute_issue(source_id, element, attribute, snapshot),
        BpmnEngineError::MissingDmnDecision { source_id } => {
            missing_dmn_decision_issue(source_id, snapshot)
        }
        _ => return None,
    })
}
