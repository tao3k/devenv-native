use crate::bpmn_cli::deps::{
    QianjiBpmnWorkflowControlError, QianjiBpmnWorkflowTaskClaimPayload,
    QianjiBpmnWorkflowTaskClaimRequest, QianjiBpmnWorkflowTaskReleasePayload,
    QianjiBpmnWorkflowTaskReleaseRequest, QianjiBpmnWorkflowWorklistRequest,
    QianjiBpmnWorkflowWorklistRoutingFilter, QianjiRuntimeEnv, SchedulerAgentIdentity,
};
use crate::bpmn_cli::render;
use crate::bpmn_cli::types::{
    BpmnCliOutput, BpmnTaskClaimCliCommand, BpmnTaskReleaseCliCommand, BpmnTaskWorklistCliCommand,
};

use super::shared::workflow_control_service;

pub(crate) async fn run_bpmn_task_claim_command(
    command: &BpmnTaskClaimCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let scheduler_identity = SchedulerAgentIdentity::from_env();
    run_bpmn_task_claim_command_with_runtime_env(command, None, Some(&scheduler_identity)).await
}

pub(crate) async fn run_bpmn_task_release_command(
    command: &BpmnTaskReleaseCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let scheduler_identity = SchedulerAgentIdentity::from_env();
    run_bpmn_task_release_command_with_runtime_env(command, None, Some(&scheduler_identity)).await
}

pub(crate) async fn run_bpmn_task_worklist_command(
    command: &BpmnTaskWorklistCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    run_bpmn_task_worklist_command_with_runtime_env(command, None).await
}

pub(crate) async fn run_bpmn_task_claim_command_with_runtime_env(
    command: &BpmnTaskClaimCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
    scheduler_identity: Option<&SchedulerAgentIdentity>,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let control_service = workflow_control_service(runtime_env, scheduler_identity);
    let request = build_bpmn_workflow_task_claim_request(command);
    match control_service.claim_workflow_task(&request).await {
        Ok(report) => Ok(render::render_bpmn_task_claim_output(command, &report)),
        Err(QianjiBpmnWorkflowControlError::CheckpointMissing { .. }) => {
            Ok(render::render_bpmn_task_claim_missing_output(command))
        }
        Err(error) => Err(error.into()),
    }
}

pub(crate) async fn run_bpmn_task_release_command_with_runtime_env(
    command: &BpmnTaskReleaseCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
    scheduler_identity: Option<&SchedulerAgentIdentity>,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let control_service = workflow_control_service(runtime_env, scheduler_identity);
    let request = build_bpmn_workflow_task_release_request(command);
    match control_service.release_workflow_task(&request).await {
        Ok(report) => Ok(render::render_bpmn_task_release_output(command, &report)),
        Err(QianjiBpmnWorkflowControlError::CheckpointMissing { .. }) => {
            Ok(render::render_bpmn_task_release_missing_output(command))
        }
        Err(error) => Err(error.into()),
    }
}

pub(crate) async fn run_bpmn_task_worklist_command_with_runtime_env(
    command: &BpmnTaskWorklistCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let control_service = workflow_control_service(runtime_env, None);
    let report = control_service
        .list_workflow_worklist(&QianjiBpmnWorkflowWorklistRequest {
            checkpoint_backend: command.checkpoint_backend.clone(),
            claimant: command.claimant.clone(),
            routing: QianjiBpmnWorkflowWorklistRoutingFilter {
                assignment_resource: command.assignment_resource.clone(),
                lane: command.lane.clone(),
            },
        })
        .await?;
    Ok(render::render_bpmn_task_worklist_output(command, &report))
}

fn build_bpmn_workflow_task_claim_request(
    command: &BpmnTaskClaimCliCommand,
) -> QianjiBpmnWorkflowTaskClaimRequest {
    QianjiBpmnWorkflowTaskClaimRequest {
        instance_id: command.instance_id.clone(),
        checkpoint_backend: command.checkpoint_backend.clone(),
        claim: QianjiBpmnWorkflowTaskClaimPayload {
            token_id: command.token_id,
            process_id: command.process_id.clone(),
            activity_id: command.activity_id.clone(),
            claimant: command.claimant.clone(),
        },
    }
}

fn build_bpmn_workflow_task_release_request(
    command: &BpmnTaskReleaseCliCommand,
) -> QianjiBpmnWorkflowTaskReleaseRequest {
    QianjiBpmnWorkflowTaskReleaseRequest {
        instance_id: command.instance_id.clone(),
        checkpoint_backend: command.checkpoint_backend.clone(),
        release: QianjiBpmnWorkflowTaskReleasePayload {
            token_id: command.token_id,
            process_id: command.process_id.clone(),
            activity_id: command.activity_id.clone(),
            claimant: command.claimant.clone(),
        },
    }
}
