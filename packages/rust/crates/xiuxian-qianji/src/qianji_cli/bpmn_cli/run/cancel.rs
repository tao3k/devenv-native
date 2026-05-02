use crate::qianji_cli::bpmn_cli::deps::{
    QianjiBpmnWorkflowCancelRequest, QianjiBpmnWorkflowControlError, QianjiRuntimeEnv,
    SchedulerAgentIdentity,
};
use crate::qianji_cli::bpmn_cli::render;
use crate::qianji_cli::bpmn_cli::types::{BpmnCancelCliCommand, BpmnCliOutput};

use super::shared::workflow_control_service;

pub(crate) async fn run_bpmn_cancel_command(
    command: &BpmnCancelCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let scheduler_identity = SchedulerAgentIdentity::from_env();
    run_bpmn_cancel_command_with_runtime_env(command, None, Some(&scheduler_identity)).await
}

async fn run_bpmn_cancel_command_with_runtime_env(
    command: &BpmnCancelCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
    scheduler_identity: Option<&SchedulerAgentIdentity>,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let control_service = workflow_control_service(runtime_env, scheduler_identity);
    let cancel_request = QianjiBpmnWorkflowCancelRequest {
        instance_id: command.instance_id.clone(),
        checkpoint_backend: command.checkpoint_backend.clone(),
    };

    match control_service.cancel_workflow(&cancel_request).await {
        Ok(report) => Ok(render::render_bpmn_cancel_output(command, &report)),
        Err(QianjiBpmnWorkflowControlError::CheckpointMissing { .. }) => {
            Ok(render::render_bpmn_cancel_missing_output(command))
        }
        Err(error) => Err(error.into()),
    }
}
