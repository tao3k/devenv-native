use crate::bpmn_cli::deps::{
    QianjiBpmnWorkflowControlError, QianjiBpmnWorkflowStatusRequest, QianjiRuntimeEnv,
};
use crate::bpmn_cli::render;
use crate::bpmn_cli::types::{BpmnCliOutput, BpmnStatusCliCommand};

use super::shared::workflow_control_service;

pub(crate) async fn run_bpmn_status_command(
    command: &BpmnStatusCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    run_bpmn_status_command_with_runtime_env(command, None).await
}

pub(crate) async fn run_bpmn_status_command_with_runtime_env(
    command: &BpmnStatusCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let control_service = workflow_control_service(runtime_env, None);
    let status_request = QianjiBpmnWorkflowStatusRequest {
        instance_id: command.instance_id.clone(),
        checkpoint_backend: command.checkpoint_backend.clone(),
    };

    match control_service.load_workflow_status(&status_request).await {
        Ok(report) => Ok(render::render_bpmn_status_output(command, &report)),
        Err(QianjiBpmnWorkflowControlError::CheckpointMissing { .. }) => {
            Ok(render::render_bpmn_status_missing_output(command))
        }
        Err(error) => Err(error.into()),
    }
}
