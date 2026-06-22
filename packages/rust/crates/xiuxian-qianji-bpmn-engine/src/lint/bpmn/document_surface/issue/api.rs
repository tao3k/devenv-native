use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint_api::LintIssue;

pub(in crate::lint::bpmn::document_surface) fn issue_for_tag(
    source: &BpmnSourceFile,
    tag: &str,
    parent: Option<&str>,
) -> Option<LintIssue> {
    super::dispatch::issue_for_tag(source, tag, parent)
}

pub(in crate::lint::bpmn::document_surface) fn resource_role_metadata_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    super::metadata::resource_role_metadata_issue(source)
}

pub(in crate::lint::bpmn::document_surface) fn flow_element_metadata_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    super::metadata::flow_element_metadata_issue(source)
}

pub(in crate::lint::bpmn::document_surface) fn io_set_lifecycle_issue(
    source: &BpmnSourceFile,
) -> Option<LintIssue> {
    super::data::io_set_lifecycle_issue(source)
}
