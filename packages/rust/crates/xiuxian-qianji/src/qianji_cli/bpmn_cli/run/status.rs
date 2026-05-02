use crate::qianji_cli::bpmn_cli::deps::{
    QianjiBpmnWorkflowControlError, QianjiBpmnWorkflowStatusRequest, QianjiRuntimeEnv,
    load_bpmn_package_from_files, resolve_cli_path,
};
use crate::qianji_cli::bpmn_cli::render;
use crate::qianji_cli::bpmn_cli::types::{BpmnCliOutput, BpmnStatusCliCommand};

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
    let package = command
        .bpmn_path
        .as_ref()
        .map(|bpmn_path| -> Result<_, Box<dyn std::error::Error>> {
            let resolved_bpmn_path = resolve_cli_path(bpmn_path)?;
            let resolved_dmn_paths = command
                .dmn_paths
                .iter()
                .map(|dmn_path| resolve_cli_path(dmn_path))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(load_bpmn_package_from_files(
                resolved_bpmn_path,
                &resolved_dmn_paths,
            )?)
        })
        .transpose()?;

    match control_service.load_workflow_status(&status_request).await {
        Ok(report) => Ok(render::render_bpmn_status_output(
            command,
            &report,
            package.as_deref(),
        )),
        Err(QianjiBpmnWorkflowControlError::CheckpointMissing { .. }) => {
            Ok(render::render_bpmn_status_missing_output(command))
        }
        Err(error) => Err(error.into()),
    }
}
