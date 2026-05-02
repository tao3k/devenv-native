use crate::qianji_cli::bpmn_cli::deps::{
    QianjiBpmnWorkflowControlError, QianjiBpmnWorkflowInterruptRequest, QianjiRuntimeEnv,
    SchedulerAgentIdentity,
};
use crate::qianji_cli::bpmn_cli::render;
use crate::qianji_cli::bpmn_cli::types::{BpmnCliOutput, BpmnInterruptCliCommand};

use super::shared::workflow_control_service;

pub(crate) async fn run_bpmn_interrupt_command(
    command: &BpmnInterruptCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let scheduler_identity = SchedulerAgentIdentity::from_env();
    run_bpmn_interrupt_command_with_runtime_env(command, None, Some(&scheduler_identity)).await
}

async fn run_bpmn_interrupt_command_with_runtime_env(
    command: &BpmnInterruptCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
    scheduler_identity: Option<&SchedulerAgentIdentity>,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let control_service = workflow_control_service(runtime_env, scheduler_identity);
    let interrupt_request = QianjiBpmnWorkflowInterruptRequest {
        instance_id: command.instance_id.clone(),
        checkpoint_backend: command.checkpoint_backend.clone(),
    };

    match control_service.interrupt_workflow(&interrupt_request).await {
        Ok(report) => Ok(render::render_bpmn_interrupt_output(command, &report)),
        Err(QianjiBpmnWorkflowControlError::CheckpointMissing { .. }) => {
            Ok(render::render_bpmn_interrupt_missing_output(command))
        }
        Err(error) => Err(error.into()),
    }
}
