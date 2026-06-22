use super::{BpmnSourceFile, LintIssue, collect_process_contracts};

pub(in crate::lint::bpmn) fn undeclared_gateway_condition_output_issues(
    source: &BpmnSourceFile,
) -> Vec<LintIssue> {
    collect_process_contracts(source)
        .into_iter()
        .flat_map(|process| process.undeclared_gateway_condition_output_issues(source))
        .collect()
}
