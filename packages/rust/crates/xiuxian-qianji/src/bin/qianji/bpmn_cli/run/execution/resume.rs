use crate::bpmn_cli::deps::{
    QianjiBpmnWorkflowControlError, QianjiRuntimeEnv, SchedulerAgentIdentity,
};
use crate::bpmn_cli::types::{
    BpmnCliOutput, BpmnEventPollCliCommand, BpmnExecutionRenderContext, BpmnResumeCliCommand,
    BpmnTaskCompleteCliCommand,
};
use crate::bpmn_cli::{host, render};

use super::request::build_bpmn_workflow_resume_request;
use crate::bpmn_cli::run::shared::workflow_control_service;

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
            let report = if command.external_host {
                control_service
                    .resume_prepared_workflow_until_host_boundary(
                        prepared,
                        &host_context.host,
                        matches!(render_mode, ResumeLikeRenderMode::TaskComplete),
                        |session, events| {
                            if command.trace_stream {
                                for line in render::render_bpmn_execution_trace_stream_lines(
                                    session, events,
                                ) {
                                    println!("{line}");
                                }
                            }
                        },
                    )
                    .await?
            } else if command.trace_stream {
                control_service
                    .start_prepared_workflow_with_trace_observer(
                        prepared,
                        &host_context.host,
                        |session, events| {
                            for line in
                                render::render_bpmn_execution_trace_stream_lines(session, events)
                            {
                                println!("{line}");
                            }
                        },
                    )
                    .await?
            } else {
                control_service
                    .resume_prepared_workflow(prepared, &host_context.host)
                    .await?
            };
            if command.trace_stream {
                for line in
                    render::render_bpmn_pending_host_work_stream_lines(&report.execution.session)
                {
                    println!("{line}");
                }
            }

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
