use crate::qianji_cli::bpmn_cli::deps::{
    QianjiBpmnWorkflowResumeRequest, QianjiBpmnWorkflowStartRequest,
    QianjiBpmnWorkflowTaskCompleteRequest, QianjiBpmnWorkflowTaskCompletionKind,
    QianjiBpmnWorkflowTaskCompletionPayload, invalid_input,
};
use crate::qianji_cli::bpmn_cli::types::{
    BpmnResumeCliCommand, BpmnRunCliCommand, BpmnTaskCompleteCliCommand, BpmnTaskCompleteCliKind,
};

pub(crate) fn build_bpmn_workflow_start_request(
    command: &BpmnRunCliCommand,
) -> Result<QianjiBpmnWorkflowStartRequest, Box<dyn std::error::Error>> {
    Ok(QianjiBpmnWorkflowStartRequest {
        bpmn_path: command.bpmn_path.clone(),
        dmn_paths: command.dmn_paths.clone(),
        process_id: command.process_id.clone().into(),
        instance_id: command.instance_id.clone().into(),
        initial_variables: parse_bpmn_cli_initial_variables(command.context_json.as_deref())?,
        start_at_node_id: command.start_at_node_id.clone().map(Into::into),
        checkpoint_backend: command.checkpoint_backend.clone(),
    })
}

pub(crate) fn build_bpmn_workflow_resume_request(
    command: &BpmnResumeCliCommand,
) -> QianjiBpmnWorkflowResumeRequest {
    QianjiBpmnWorkflowResumeRequest {
        bpmn_path: command.bpmn_path.clone(),
        dmn_paths: command.dmn_paths.clone(),
        instance_id: command.instance_id.clone().into(),
        checkpoint_backend: command.checkpoint_backend.clone(),
    }
}

pub(crate) fn build_bpmn_workflow_task_complete_request(
    command: &BpmnTaskCompleteCliCommand,
) -> Result<QianjiBpmnWorkflowTaskCompleteRequest, Box<dyn std::error::Error>> {
    Ok(QianjiBpmnWorkflowTaskCompleteRequest {
        bpmn_path: command.bpmn_path.clone(),
        dmn_paths: command.dmn_paths.clone(),
        instance_id: command.instance_id.clone().into(),
        checkpoint_backend: command.checkpoint_backend.clone(),
        completion: QianjiBpmnWorkflowTaskCompletionPayload {
            token_id: command.token_id,
            process_id: command.process_id.clone().into(),
            activity_id: command.activity_id.clone().into(),
            kind: match command.kind {
                BpmnTaskCompleteCliKind::Task => QianjiBpmnWorkflowTaskCompletionKind::Task,
                BpmnTaskCompleteCliKind::Send => QianjiBpmnWorkflowTaskCompletionKind::Send,
                BpmnTaskCompleteCliKind::Service => QianjiBpmnWorkflowTaskCompletionKind::Service,
                BpmnTaskCompleteCliKind::Script => QianjiBpmnWorkflowTaskCompletionKind::Script,
                BpmnTaskCompleteCliKind::User => QianjiBpmnWorkflowTaskCompletionKind::User,
                BpmnTaskCompleteCliKind::Manual => QianjiBpmnWorkflowTaskCompletionKind::Manual,
            },
            data: parse_bpmn_cli_data_json(command.data_json.as_str())?,
            claimant: command.claimant.clone(),
        },
        continue_until_human_boundary: command.continue_until_human_boundary,
    })
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

fn parse_bpmn_cli_data_json(
    raw_data: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    serde_json::from_str(raw_data)
        .map_err(|error| {
            invalid_input(format!(
                "failed to parse `--data-json` as valid JSON: {error}"
            ))
        })
        .map_err(Into::into)
}
