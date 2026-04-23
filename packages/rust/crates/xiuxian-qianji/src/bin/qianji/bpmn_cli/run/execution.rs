use crate::bpmn_cli::deps::{
    QianjiBpmnWorkflowControlError, QianjiBpmnWorkflowResumeRequest,
    QianjiBpmnWorkflowStartRequest, QianjiRuntimeEnv, SchedulerAgentIdentity, invalid_input,
};
use crate::bpmn_cli::types::{
    BpmnCliOutput, BpmnEventPollCliCommand, BpmnExecutionRenderContext, BpmnResumeCliCommand,
    BpmnRunCliCommand, BpmnStartCliCommand, BpmnTaskCompleteCliCommand,
};
use crate::bpmn_cli::{host, render};

use super::shared::workflow_control_service;

pub(crate) async fn run_bpmn_start_command(
    command: &BpmnStartCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let scheduler_identity = SchedulerAgentIdentity::from_env();
    run_bpmn_start_command_with_runtime_env(command, None, Some(&scheduler_identity)).await
}

pub(crate) async fn run_bpmn_run_command(
    command: &BpmnRunCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let scheduler_identity = SchedulerAgentIdentity::from_env();
    run_bpmn_run_command_with_runtime_env(command, None, Some(&scheduler_identity)).await
}

pub(crate) async fn run_bpmn_resume_command(
    command: &BpmnResumeCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let scheduler_identity = SchedulerAgentIdentity::from_env();
    run_bpmn_resume_command_with_runtime_env(command, None, Some(&scheduler_identity)).await
}

pub(crate) async fn run_bpmn_event_poll_command(
    command: &BpmnEventPollCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let scheduler_identity = SchedulerAgentIdentity::from_env();
    run_bpmn_event_poll_command_with_runtime_env(command, None, Some(&scheduler_identity)).await
}

pub(crate) async fn run_bpmn_task_complete_command(
    command: &BpmnTaskCompleteCliCommand,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let scheduler_identity = SchedulerAgentIdentity::from_env();
    run_bpmn_task_complete_command_with_runtime_env(command, None, Some(&scheduler_identity)).await
}

pub(crate) async fn run_bpmn_run_command_with_runtime_env(
    command: &BpmnRunCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
    scheduler_identity: Option<&SchedulerAgentIdentity>,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    run_bpmn_start_like_command_with_runtime_env(command, runtime_env, scheduler_identity, true)
        .await
}

async fn run_bpmn_start_command_with_runtime_env(
    command: &BpmnStartCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
    scheduler_identity: Option<&SchedulerAgentIdentity>,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    run_bpmn_start_like_command_with_runtime_env(command, runtime_env, scheduler_identity, false)
        .await
}

async fn run_bpmn_start_like_command_with_runtime_env(
    command: &BpmnRunCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
    scheduler_identity: Option<&SchedulerAgentIdentity>,
    render_as_run_alias: bool,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let control_service = workflow_control_service(runtime_env, scheduler_identity);
    let start_request = build_bpmn_workflow_start_request(command)?;
    let prepared = control_service.prepare_start_workflow(&start_request)?;
    let host_context = host::build_bpmn_cli_host_bridge(
        &prepared.package,
        command.process_id.as_str(),
        command.host_fixture_path.as_deref(),
        command.event_fixture_path.as_deref(),
    )?;
    let report = control_service
        .start_prepared_workflow(prepared, &host_context.host)
        .await?;

    let render_context = BpmnExecutionRenderContext {
        resolved_bpmn_path: report.resolved_bpmn_path.as_path(),
        resolved_dmn_paths: &report.resolved_dmn_paths,
        checkpoint_store: report.checkpoint_store.as_ref(),
        resolved_host_fixture_path: host_context.resolved_host_fixture_path.as_deref(),
        resolved_event_fixture_path: host_context.resolved_event_fixture_path.as_deref(),
        resumed_from_checkpoint: report.execution.resumed_from_checkpoint,
        checkpoint_saved: report.execution.checkpoint_saved,
        checkpoint_deleted: report.execution.checkpoint_deleted,
    };

    Ok(if render_as_run_alias {
        render::render_bpmn_run_output(
            command,
            &report.execution.session,
            &report.execution.outcome,
            &render_context,
        )
    } else {
        render::render_bpmn_start_output(
            command,
            &report.execution.session,
            &report.execution.outcome,
            &render_context,
        )
    })
}

async fn run_bpmn_resume_command_with_runtime_env(
    command: &BpmnResumeCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
    scheduler_identity: Option<&SchedulerAgentIdentity>,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    run_bpmn_resume_like_command_with_runtime_env(
        command,
        runtime_env,
        scheduler_identity,
        ResumeLikeRenderMode::Resume,
    )
    .await
}

async fn run_bpmn_event_poll_command_with_runtime_env(
    command: &BpmnEventPollCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
    scheduler_identity: Option<&SchedulerAgentIdentity>,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    run_bpmn_resume_like_command_with_runtime_env(
        command,
        runtime_env,
        scheduler_identity,
        ResumeLikeRenderMode::EventPoll,
    )
    .await
}

async fn run_bpmn_task_complete_command_with_runtime_env(
    command: &BpmnTaskCompleteCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
    scheduler_identity: Option<&SchedulerAgentIdentity>,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    run_bpmn_resume_like_command_with_runtime_env(
        command,
        runtime_env,
        scheduler_identity,
        ResumeLikeRenderMode::TaskComplete,
    )
    .await
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResumeLikeRenderMode {
    Resume,
    EventPoll,
    TaskComplete,
}

async fn run_bpmn_resume_like_command_with_runtime_env(
    command: &BpmnResumeCliCommand,
    runtime_env: Option<&QianjiRuntimeEnv>,
    scheduler_identity: Option<&SchedulerAgentIdentity>,
    render_mode: ResumeLikeRenderMode,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let control_service = workflow_control_service(runtime_env, scheduler_identity);
    let resume_request = build_bpmn_workflow_resume_request(command);

    match control_service
        .prepare_resume_workflow(&resume_request)
        .await
    {
        Ok(prepared) => {
            let host_context = host::build_bpmn_cli_host_bridge(
                &prepared.package,
                prepared.execution_request.process_id.as_str(),
                command.host_fixture_path.as_deref(),
                command.event_fixture_path.as_deref(),
            )?;
            let report = control_service
                .resume_prepared_workflow(prepared, &host_context.host)
                .await?;

            let render_context = BpmnExecutionRenderContext {
                resolved_bpmn_path: report.resolved_bpmn_path.as_path(),
                resolved_dmn_paths: &report.resolved_dmn_paths,
                checkpoint_store: report.checkpoint_store.as_ref(),
                resolved_host_fixture_path: host_context.resolved_host_fixture_path.as_deref(),
                resolved_event_fixture_path: host_context.resolved_event_fixture_path.as_deref(),
                resumed_from_checkpoint: report.execution.resumed_from_checkpoint,
                checkpoint_saved: report.execution.checkpoint_saved,
                checkpoint_deleted: report.execution.checkpoint_deleted,
            };

            Ok(match render_mode {
                ResumeLikeRenderMode::Resume => render::render_bpmn_resume_output(
                    command,
                    &report.execution.session,
                    &report.execution.outcome,
                    &render_context,
                ),
                ResumeLikeRenderMode::EventPoll => render::render_bpmn_event_poll_output(
                    command,
                    &report.execution.session,
                    &report.execution.outcome,
                    &render_context,
                ),
                ResumeLikeRenderMode::TaskComplete => render::render_bpmn_task_complete_output(
                    command,
                    &report.execution.session,
                    &report.execution.outcome,
                    &render_context,
                ),
            })
        }
        Err(QianjiBpmnWorkflowControlError::CheckpointMissing { .. }) => Ok(match render_mode {
            ResumeLikeRenderMode::Resume => render::render_bpmn_resume_missing_output(command),
            ResumeLikeRenderMode::EventPoll => {
                render::render_bpmn_event_poll_missing_output(command)
            }
            ResumeLikeRenderMode::TaskComplete => {
                render::render_bpmn_task_complete_missing_output(command)
            }
        }),
        Err(error) => Err(error.into()),
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

fn build_bpmn_workflow_start_request(
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

fn build_bpmn_workflow_resume_request(
    command: &BpmnResumeCliCommand,
) -> QianjiBpmnWorkflowResumeRequest {
    QianjiBpmnWorkflowResumeRequest {
        bpmn_path: command.bpmn_path.clone(),
        dmn_paths: command.dmn_paths.clone(),
        instance_id: command.instance_id.clone(),
        checkpoint_backend: command.checkpoint_backend.clone(),
    }
}
