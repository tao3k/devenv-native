use crate::qianji_cli::bpmn_cli::deps::{QianjiBpmnWorkflowInstancesRequest, QianjiRuntimeEnv};
use crate::qianji_cli::bpmn_cli::render;
use crate::qianji_cli::bpmn_cli::types::{BpmnCliOutput, BpmnInstancesCliCommand};

use super::control_service::workflow_control_service;

pub(crate) async fn run_bpmn_instances_command(
    command: &BpmnInstancesCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    run_bpmn_instances_command_with_runtime_env(command, None).await
}

async fn run_bpmn_instances_command_with_runtime_env(
    command: &BpmnInstancesCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let control_service = workflow_control_service(runtime_env, None);
    let instances_request = QianjiBpmnWorkflowInstancesRequest {
        checkpoint_backend: command.checkpoint_backend.clone(),
    };

    let report = control_service
        .list_workflow_instances(&instances_request)
        .await?;
    Ok(render::render_bpmn_instances_output(command, &report))
}
