use crate::bpmn_parse_api::BpmnSourceFile;
use crate::lint_api::LintIssue;

pub(super) fn issue_for_tag(
    _source: &BpmnSourceFile,
    _tag: &str,
    _parent: Option<&str>,
) -> Option<LintIssue> {
    None
}
