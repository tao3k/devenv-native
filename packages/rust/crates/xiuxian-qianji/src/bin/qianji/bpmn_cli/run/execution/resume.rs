use crate::bpmn_cli::deps::{
    QianjiBpmnPreparedWorkflowResume, QianjiBpmnWorkflowControlError,
    QianjiBpmnWorkflowControlService, QianjiRuntimeEnv, SchedulerAgentIdentity,
};
use crate::bpmn_cli::types::{
    BpmnCliOutput, BpmnEventPollCliCommand, BpmnExecutionRenderContext, BpmnResumeCliCommand,
    BpmnTaskCompleteCliCommand,
};
use crate::bpmn_cli::{host, render};

use super::request::build_bpmn_workflow_resume_request;
use super::request::build_bpmn_workflow_task_complete_request;
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
    let control_service = workflow_control_service(runtime_env, scheduler_identity);
    let task_complete_request = build_bpmn_workflow_task_complete_request(command)?;
    let resume_request = build_bpmn_workflow_task_complete_resume_request(command);

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
            run_prepared_bpmn_task_complete_command(
                command,
                &control_service,
                &task_complete_request,
                &host_context,
            )
            .await
        }
        Err(QianjiBpmnWorkflowControlError::CheckpointMissing { .. }) => {
            Ok(render::render_bpmn_task_complete_missing_output(command))
        }
        Err(error) => Err(error.into()),
    }
}

async fn run_prepared_bpmn_task_complete_command(
    command: &BpmnTaskCompleteCliCommand,
    control_service: &crate::bpmn_cli::deps::QianjiBpmnWorkflowControlService,
    task_complete_request: &crate::bpmn_cli::deps::QianjiBpmnWorkflowTaskCompleteRequest,
    host_context: &crate::bpmn_cli::types::BpmnCliHostBridgeContext,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    match control_service
        .complete_workflow_task(task_complete_request, &host_context.host)
        .await
    {
        Ok(report) => {
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

            Ok(render::render_bpmn_task_complete_output(
                command,
                &report.execution.session,
                &report.execution.outcome,
                &render_context,
            ))
        }
        Err(QianjiBpmnWorkflowControlError::CheckpointMissing { .. }) => {
            Ok(render::render_bpmn_task_complete_missing_output(command))
        }
        Err(error) => Err(error.into()),
    }
}

fn build_bpmn_workflow_task_complete_resume_request(
    command: &BpmnTaskCompleteCliCommand,
) -> crate::bpmn_cli::deps::QianjiBpmnWorkflowResumeRequest {
    crate::bpmn_cli::deps::QianjiBpmnWorkflowResumeRequest {
        bpmn_path: command.bpmn_path.clone(),
        dmn_paths: command.dmn_paths.clone(),
        instance_id: command.instance_id.clone(),
        checkpoint_backend: command.checkpoint_backend.clone(),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResumeLikeRenderMode {
    Resume,
    EventPoll,
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
            run_prepared_bpmn_resume_like_command(command, &control_service, prepared, render_mode)
                .await
        }
        Err(QianjiBpmnWorkflowControlError::CheckpointMissing { .. }) => Ok(match render_mode {
            ResumeLikeRenderMode::Resume => render::render_bpmn_resume_missing_output(command),
            ResumeLikeRenderMode::EventPoll => {
                render::render_bpmn_event_poll_missing_output(command)
            }
        }),
        Err(error) => Err(error.into()),
    }
}

async fn run_prepared_bpmn_resume_like_command(
    command: &BpmnResumeCliCommand,
    control_service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowResume,
    render_mode: ResumeLikeRenderMode,
) -> Result<BpmnCliOutput, Box<dyn std::error::Error>> {
    let host_context = host::build_bpmn_cli_host_bridge(
        &prepared.package,
        prepared.execution_request.process_id.as_str(),
        command.host_fixture_path.as_deref(),
        command.event_fixture_path.as_deref(),
    )?;
    let report = if command.external_host && command.continue_until_human_boundary {
        control_service
            .resume_prepared_workflow_until_human_boundary(
                prepared,
                &host_context.host,
                true,
                trace_observer(command.trace_stream),
            )
            .await?
    } else if command.external_host {
        control_service
            .resume_prepared_workflow_until_host_boundary(
                prepared,
                &host_context.host,
                false,
                trace_observer(command.trace_stream),
            )
            .await?
    } else if command.trace_stream {
        control_service
            .start_prepared_workflow_with_trace_observer(
                prepared,
                &host_context.host,
                trace_observer(true),
            )
            .await?
    } else {
        control_service
            .resume_prepared_workflow(prepared, &host_context.host)
            .await?
    };
    if command.trace_stream {
        for line in render::render_bpmn_pending_host_work_stream_lines(&report.execution.session) {
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
    })
}

fn trace_observer(
    enabled: bool,
) -> impl Fn(&crate::bpmn_cli::deps::QianjiBpmnSession, &[crate::bpmn_cli::deps::BpmnExecutionTraceEvent])
{
    move |session, events| {
        if enabled {
            for line in render::render_bpmn_execution_trace_stream_lines(session, events) {
                println!("{line}");
            }
        }
    }
}
