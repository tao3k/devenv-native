use crate::qianji_cli::bpmn_cli::deps::{
    QianjiBpmnPreparedWorkflowStart, QianjiBpmnWorkflowControlService,
    QianjiBpmnWorkflowTaskCompleteRequest,
};
use crate::qianji_cli::bpmn_cli::render;
use crate::qianji_cli::bpmn_cli::types::{
    BpmnCliHostBridgeContext, BpmnExecutionRenderContext, BpmnRunCliCommand,
    BpmnTaskCompleteCliCommand,
};

use super::result::{BpmnHostSessionStepResult, build_session_step_result};

pub(super) async fn run_prepared_session_start(
    command: &BpmnRunCliCommand,
    control_service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowStart,
    host_context: &BpmnCliHostBridgeContext,
) -> Result<BpmnHostSessionStepResult, Box<dyn std::error::Error>> {
    let report = if command.host_fixture_path.is_some() {
        control_service
            .start_prepared_workflow_until_human_boundary(
                prepared,
                &host_context.host,
                false,
                |session, events| {
                    if command.trace_stream {
                        for line in
                            render::render_bpmn_execution_trace_stream_lines(session, events)
                        {
                            println!("{line}");
                        }
                    }
                },
            )
            .await?
    } else {
        control_service
            .start_prepared_workflow_until_host_boundary(
                prepared,
                &host_context.host,
                false,
                |session, events| {
                    if command.trace_stream {
                        for line in
                            render::render_bpmn_execution_trace_stream_lines(session, events)
                        {
                            println!("{line}");
                        }
                    }
                },
            )
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

    let command_title = if command.start_at_node_id.is_some() {
        "qianji start-at"
    } else {
        "qianji run"
    };
    Ok(build_session_step_result(
        command_title,
        &report.execution.session,
        &report.execution.outcome,
        &render_context,
    ))
}

pub(super) async fn run_prepared_session_task_complete(
    command: &BpmnTaskCompleteCliCommand,
    request: &QianjiBpmnWorkflowTaskCompleteRequest,
    control_service: &QianjiBpmnWorkflowControlService,
    prepared: QianjiBpmnPreparedWorkflowStart,
    host_context: &BpmnCliHostBridgeContext,
) -> Result<BpmnHostSessionStepResult, Box<dyn std::error::Error>> {
    let report = if command.host_fixture_path.is_some() {
        control_service
            .complete_prepared_workflow_task(prepared, request, &host_context.host)
            .await?
    } else {
        control_service
            .complete_prepared_workflow_task_until_host_boundary(
                prepared,
                request,
                &host_context.host,
            )
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

    Ok(build_session_step_result(
        "qianji task complete",
        &report.execution.session,
        &report.execution.outcome,
        &render_context,
    ))
}
