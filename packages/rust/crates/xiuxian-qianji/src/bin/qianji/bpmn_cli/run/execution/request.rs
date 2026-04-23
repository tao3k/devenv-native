use crate::bpmn_cli::deps::{
    QianjiBpmnWorkflowResumeRequest, QianjiBpmnWorkflowStartRequest, invalid_input,
};
use crate::bpmn_cli::types::{BpmnResumeCliCommand, BpmnRunCliCommand};

pub(super) fn build_bpmn_workflow_start_request(
    command: &BpmnRunCliCommand,
) -> Result<QianjiBpmnWorkflowStartRequest, Box<dyn std::error::Error>> {
    Ok(QianjiBpmnWorkflowStartRequest {
        bpmn_path: command.bpmn_path.clone(),
        dmn_paths: command.dmn_paths.clone(),
        process_id: command.process_id.clone(),
        instance_id: command.instance_id.clone(),
        initial_variables: parse_bpmn_cli_initial_variables(command.context_json.as_deref())?,
        checkpoint_backend: command.checkpoint_backend.clone(),
    })
}

pub(super) fn build_bpmn_workflow_resume_request(
    command: &BpmnResumeCliCommand,
) -> QianjiBpmnWorkflowResumeRequest {
    QianjiBpmnWorkflowResumeRequest {
        bpmn_path: command.bpmn_path.clone(),
        dmn_paths: command.dmn_paths.clone(),
        instance_id: command.instance_id.clone(),
        checkpoint_backend: command.checkpoint_backend.clone(),
    }
}

fn parse_bpmn_cli_initial_variables(
    raw_context: Option<&str>,
) -> Result<Option<serde_json::Value>, Box<dyn std::error::Error>> {
    raw_context
        .map(|raw| {
            serde_json::from_str(raw).map_err(|error| {
                invalid_input(format!(
                    "failed to parse `--context-json` as valid JSON: {error}"
                ))
            })
        })
        .transpose()
        .map_err(Into::into)
}
