use super::data::data_artifact_issue;
use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint_api::LintIssue;

pub(super) fn issue_for_tag(
    source: &BpmnSourceFile,
    tag: &str,
    _parent: Option<&str>,
) -> Option<LintIssue> {
    match tag {
        "dataStore" | "dataStoreReference" => Some(data_artifact_issue(source, tag)),
        _ => None,
    }
}
