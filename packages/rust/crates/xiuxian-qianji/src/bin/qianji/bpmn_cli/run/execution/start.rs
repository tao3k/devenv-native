use crate::bpmn_cli::deps::{QianjiRuntimeEnv, SchedulerAgentIdentity};
use crate::bpmn_cli::types::{
    BpmnCliOutput, BpmnExecutionRenderContext, BpmnRunCliCommand, BpmnStartCliCommand,
};
use crate::bpmn_cli::{host, render};

use super::request::build_bpmn_workflow_start_request;
use crate::bpmn_cli::run::shared::workflow_control_service;

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
