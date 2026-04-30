use super::{
    BpmnPackage, BpmnSourceFile, LintIssue, collect_process_metadata, process_loop_risk_issues,
};

pub(in crate::lint::bpmn) fn loop_risk_issues(
    source: &BpmnSourceFile,
    package: &BpmnPackage,
) -> Vec<LintIssue> {
    let metadata_by_process = collect_process_metadata(source);
    package
        .processes
        .iter()
        .flat_map(|process| {
            let metadata = metadata_by_process
                .get(process.key.process_id.as_ref())
                .cloned()
                .unwrap_or_default();
            process_loop_risk_issues(source, process, &metadata)
        })
        .collect()
}
