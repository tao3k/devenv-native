use std::fmt::Write as _;

use crate::bpmn_cli::deps::{
    BpmnAdvanceOutcome, BpmnExecutionTraceEvent, BpmnExecutionTraceEventKind, BpmnNodeKind,
    BpmnProcessSpec, NodeRuntimeStatus, Path, PendingHostWorkRequest, QianjiBpmnSession,
    build_pending_host_work_requests,
};
use crate::bpmn_cli::types::{
    BpmnCliOutput, BpmnEventPollCliCommand, BpmnExecutionRenderContext, BpmnResumeCliCommand,
    BpmnRunCliCommand, BpmnStartCliCommand, BpmnTaskCompleteCliCommand,
};

use super::support::{
    append_bpmn_wait_registrations, bpmn_checkpoint_backend_label,
    bpmn_checkpoint_backend_selection_label, bpmn_lifecycle_label, bpmn_outcome_label,
    bpmn_pending_host_work_kind_label, bpmn_suspend_reason_label,
};

pub(crate) fn render_bpmn_start_output(
    command: &BpmnStartCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Start",
        command.process_id.as_str(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}

pub(crate) fn render_bpmn_run_output(
    command: &BpmnRunCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Run",
        command.process_id.as_str(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}

pub(crate) fn render_bpmn_resume_output(
    command: &BpmnResumeCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Resume",
        session.instance().process.process_id.as_ref(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}

pub(crate) fn render_bpmn_event_poll_output(
    command: &BpmnEventPollCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Event Poll",
        session.instance().process.process_id.as_ref(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}

pub(crate) fn render_bpmn_task_complete_output(
    command: &BpmnTaskCompleteCliCommand,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    render_bpmn_execution_output(
        "BPMN Task Complete",
        session.instance().process.process_id.as_ref(),
        command.instance_id.as_str(),
        session,
        outcome,
        render_context,
    )
}

pub(crate) fn render_bpmn_resume_missing_output(command: &BpmnResumeCliCommand) -> BpmnCliOutput {
    BpmnCliOutput {
        rendered: format!(
            "# BPMN Resume\n\nSource: {}\nInstance: {}\nCheckpoint backend: {}\nCheckpoint status: missing\n",
            command.bpmn_path.display(),
            command.instance_id,
            bpmn_checkpoint_backend_selection_label(&command.checkpoint_backend),
        ),
        exit_code: 1,
    }
}

pub(crate) fn render_bpmn_event_poll_missing_output(
    command: &BpmnEventPollCliCommand,
) -> BpmnCliOutput {
    BpmnCliOutput {
        rendered: format!(
            "# BPMN Event Poll\n\nSource: {}\nInstance: {}\nCheckpoint backend: {}\nCheckpoint status: missing\n",
            command.bpmn_path.display(),
            command.instance_id,
            bpmn_checkpoint_backend_selection_label(&command.checkpoint_backend),
        ),
        exit_code: 1,
    }
}

pub(crate) fn render_bpmn_task_complete_missing_output(
    command: &BpmnTaskCompleteCliCommand,
) -> BpmnCliOutput {
    BpmnCliOutput {
        rendered: format!(
            "# BPMN Task Complete\n\nSource: {}\nInstance: {}\nCheckpoint backend: {}\nCheckpoint status: missing\n",
            command.bpmn_path.display(),
            command.instance_id,
            bpmn_checkpoint_backend_selection_label(&command.checkpoint_backend),
        ),
        exit_code: 1,
    }
}

fn render_bpmn_execution_output(
    title: &str,
    process_id: &str,
    instance_id: &str,
    session: &QianjiBpmnSession,
    outcome: &BpmnAdvanceOutcome,
    render_context: &BpmnExecutionRenderContext<'_>,
) -> BpmnCliOutput {
    let checkpoint_backend = render_context
        .checkpoint_store
        .map_or("none", bpmn_checkpoint_backend_label);
    let checkpoint_source = if render_context.resumed_from_checkpoint {
        "resumed"
    } else {
        "fresh"
    };
    let checkpoint_saved_label = if render_context.checkpoint_saved {
        "yes"
    } else {
        "no"
    };
    let checkpoint_deleted_label = if render_context.checkpoint_deleted {
        "yes"
    } else {
        "no"
    };
    let host_fixture = render_context.resolved_host_fixture_path.map_or_else(
        || "none".to_string(),
        |path: &Path| path.display().to_string(),
    );
    let event_fixture = render_context.resolved_event_fixture_path.map_or_else(
        || "none".to_string(),
        |path: &Path| path.display().to_string(),
    );
    let variables = serde_json::to_string_pretty(&session.instance().variables)
        .unwrap_or_else(|error| format!("{{\"serialization_error\":\"{error}\"}}"));
    let mut rendered = format!(
        "# {title}\n\nSource: {}\nProcess: {}\nInstance: {}\nPackage: {}\nOutcome: {}\nLifecycle: {}\nCheckpoint backend: {}\nCheckpoint source: {}\nCheckpoint saved: {}\nCheckpoint deleted: {}\nHost fixture: {}\nEvent fixture: {}\nDMN sources: {}\nSequence: {}\nActive tokens: {}\nPending host work: {}\nWait registrations: {}\n",
        render_context.resolved_bpmn_path.display(),
        process_id,
        instance_id,
        session.package().package_id,
        bpmn_outcome_label(outcome),
        bpmn_lifecycle_label(&session.instance().lifecycle),
        checkpoint_backend,
        checkpoint_source,
        checkpoint_saved_label,
        checkpoint_deleted_label,
        host_fixture,
        event_fixture,
        render_context.resolved_dmn_paths.len(),
        session.instance().sequence,
        session.instance().active_tokens.len(),
        session.instance().pending_host_work.len(),
        session.instance().waits.len(),
    );

    if !render_context.resolved_dmn_paths.is_empty() {
        let _ = writeln!(rendered, "\n## DMN Sources");
        for path in render_context.resolved_dmn_paths {
            let _ = writeln!(rendered, "- {}", path.display());
        }
    }

    if let Some(reason) = session.instance().suspend_reason.as_ref() {
        let _ = writeln!(
            rendered,
            "\nSuspend reason: {}",
            bpmn_suspend_reason_label(reason)
        );
    }

    if let BpmnAdvanceOutcome::Failed(message) = outcome {
        let _ = writeln!(rendered, "\nFailure: {message}");
    }

    append_bpmn_wait_registrations(&mut rendered, session.package(), session.instance());
    append_bpmn_pending_host_work(&mut rendered, session);

    let trace = render_bpmn_execution_trace(session);
    let _ = writeln!(rendered, "\n## Trace");
    let _ = writeln!(rendered, "```json");
    let _ = writeln!(rendered, "{trace}");
    let _ = writeln!(rendered, "```");

    let _ = writeln!(rendered, "\n## Variables");
    let _ = writeln!(rendered, "```json");
    let _ = writeln!(rendered, "{variables}");
    let _ = writeln!(rendered, "```");

    BpmnCliOutput {
        rendered,
        exit_code: if matches!(outcome, BpmnAdvanceOutcome::Failed(_)) {
            2
        } else {
            0
        },
    }
}

fn append_bpmn_pending_host_work(rendered: &mut String, session: &QianjiBpmnSession) {
    if session.instance().pending_host_work.is_empty() {
        return;
    }

    let Some(process) = session
        .package()
        .find_process(session.instance().process.process_id.as_ref())
    else {
        return;
    };

    let _ = writeln!(rendered, "\n## Pending Host Work");
    for work in &session.instance().pending_host_work {
        let node_id = process.nodes.get(work.node_index as usize).map_or_else(
            || format!("node#{}", work.node_index),
            |node| node.bpmn_id.to_string(),
        );
        let mut line = format!(
            "- {node_id} | token#{} | kind={}",
            work.token_id,
            bpmn_pending_host_work_kind_label(&work.kind)
        );
        if let Some(process_id) = work.process_id.as_ref() {
            let _ = write!(line, " | process={process_id}");
        }
        if let Some(work_id) = work.work_id.as_ref() {
            let _ = write!(line, " | work_id={work_id}");
        }
        let _ = writeln!(rendered, "{line}");
    }
}

fn render_bpmn_execution_trace(session: &QianjiBpmnSession) -> String {
    serde_json::to_string_pretty(&bpmn_execution_trace_values(
        session,
        &session.instance().trace,
    ))
    .unwrap_or_else(|error| format!("[{{\"serialization_error\":\"{error}\"}}]"))
}

pub(crate) fn render_bpmn_execution_trace_stream_lines(
    session: &QianjiBpmnSession,
    events: &[BpmnExecutionTraceEvent],
) -> Vec<String> {
    bpmn_execution_trace_values(session, events)
        .into_iter()
        .filter_map(|event| serde_json::to_string(&event).ok())
        .map(|event| format!("@@QIANJI_TRACE {event}"))
        .collect()
}

pub(crate) fn render_bpmn_pending_host_work_stream_lines(
    session: &QianjiBpmnSession,
) -> Vec<String> {
    if session.instance().pending_host_work.is_empty() {
        return Vec::new();
    }

    let Some(process) = session
        .package()
        .find_process(session.instance().process.process_id.as_ref())
    else {
        return Vec::new();
    };

    let Ok(requests) = build_pending_host_work_requests(session.instance()) else {
        return Vec::new();
    };

    requests
        .into_iter()
        .filter_map(|request| {
            let value = pending_host_work_request_stream_value(process, request);
            serde_json::to_string(&value)
                .ok()
                .map(|payload| format!("@@QIANJI_HOST_WORK {payload}"))
        })
        .collect()
}

fn pending_host_work_request_stream_value(
    process: &BpmnProcessSpec,
    request: PendingHostWorkRequest,
) -> serde_json::Value {
    match request {
        PendingHostWorkRequest::Send(request) => serde_json::json!({
            "kind": "send",
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "message_reference": request.message_reference,
            "message_name": request.message_name,
        }),
        PendingHostWorkRequest::Service(request) => serde_json::json!({
            "kind": "service",
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "repeat": request.repeat,
        }),
        PendingHostWorkRequest::Script(request) => serde_json::json!({
            "kind": "script",
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "repeat": request.repeat,
            "script_format": request.script_format,
            "script_body": request.script_body,
        }),
        PendingHostWorkRequest::User(request) => serde_json::json!({
            "kind": "user",
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "repeat": request.repeat,
        }),
        PendingHostWorkRequest::Manual(request) => serde_json::json!({
            "kind": "manual",
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "variables": request.variables,
            "repeat": request.repeat,
        }),
        PendingHostWorkRequest::BusinessRule(request) => serde_json::json!({
            "kind": "business_rule",
            "node_id": pending_host_work_request_node_id(process, request.node_index),
            "node_index": request.node_index,
            "token_id": request.token_id,
            "evaluation": request.evaluation,
            "repeat": request.repeat,
        }),
    }
}

fn pending_host_work_request_node_id(process: &BpmnProcessSpec, node_index: u32) -> String {
    process.nodes.get(node_index as usize).map_or_else(
        || format!("node#{node_index}"),
        |node| node.bpmn_id.to_string(),
    )
}

fn bpmn_execution_trace_values(
    session: &QianjiBpmnSession,
    events: &[BpmnExecutionTraceEvent],
) -> Vec<serde_json::Value> {
    events
        .iter()
        .map(|event| {
            let process = session
                .package()
                .find_process_position(event.process.process_id.as_ref())
                .map(|(_, process)| process);
            match &event.kind {
                BpmnExecutionTraceEventKind::NodeStatus => {
                    let node = node_by_optional_index(process, event.node_index);
                    let node_id = node_id_by_index(node, event.node_index);
                    serde_json::json!({
                        "sequence": event.sequence,
                        "kind": "node_status",
                        "process_id": event.process.process_id.as_ref(),
                        "node_id": node_id,
                        "node_kind": node.map(|node| bpmn_node_kind_label(&node.kind)),
                        "status": event.status.as_ref().map_or("unknown", node_runtime_status_label),
                    })
                }
                BpmnExecutionTraceEventKind::FlowTake => {
                    let (source_id, target_id) =
                        flow_endpoint_ids(process, event.edge_index, event.node_index);
                    serde_json::json!({
                        "sequence": event.sequence,
                        "kind": "flow_take",
                        "process_id": event.process.process_id.as_ref(),
                        "source_id": source_id,
                        "target_id": target_id,
                    })
                }
            }
        })
        .collect::<Vec<_>>()
}

fn node_by_optional_index(
    process: Option<&BpmnProcessSpec>,
    node_index: Option<qianji_bpmn_engine::BpmnNodeIndex>,
) -> Option<&qianji_bpmn_engine::BpmnNodeSpec> {
    let node_index = node_index?;
    process.and_then(|process| node_by_index(process, node_index))
}

fn node_id_by_index(
    node: Option<&qianji_bpmn_engine::BpmnNodeSpec>,
    node_index: Option<qianji_bpmn_engine::BpmnNodeIndex>,
) -> String {
    let Some(node_index) = node_index else {
        return String::new();
    };
    node.map_or_else(|| node_index.to_string(), |node| node.bpmn_id.to_string())
}

fn flow_endpoint_ids(
    process: Option<&BpmnProcessSpec>,
    edge_index: Option<u32>,
    fallback_target_node_index: Option<qianji_bpmn_engine::BpmnNodeIndex>,
) -> (String, String) {
    let fallback_target = fallback_target_node_index
        .map(|index| index.to_string())
        .unwrap_or_default();
    let (Some(process), Some(edge_index)) = (process, edge_index) else {
        return (String::new(), fallback_target);
    };
    let Some(edge) = edge_by_index(process, edge_index) else {
        return (String::new(), fallback_target);
    };
    let source_id = node_by_index(process, edge.from)
        .map_or_else(|| edge.from.to_string(), |node| node.bpmn_id.to_string());
    let target_id = node_by_index(process, edge.to)
        .map_or_else(|| edge.to.to_string(), |node| node.bpmn_id.to_string());
    (source_id, target_id)
}

fn node_by_index(
    process: &BpmnProcessSpec,
    node_index: qianji_bpmn_engine::BpmnNodeIndex,
) -> Option<&qianji_bpmn_engine::BpmnNodeSpec> {
    process.nodes.get(node_index as usize)
}

fn edge_by_index(
    process: &BpmnProcessSpec,
    edge_index: u32,
) -> Option<&qianji_bpmn_engine::BpmnEdgeSpec> {
    process.edges.get(edge_index as usize)
}

fn node_runtime_status_label(status: &NodeRuntimeStatus) -> &'static str {
    match status {
        NodeRuntimeStatus::Idle => "idle",
        NodeRuntimeStatus::Queued => "queued",
        NodeRuntimeStatus::Executing => "executing",
        NodeRuntimeStatus::Completed => "completed",
        NodeRuntimeStatus::Cancelled => "cancelled",
        NodeRuntimeStatus::Failed => "failed",
    }
}

fn bpmn_node_kind_label(kind: &BpmnNodeKind) -> &'static str {
    match kind {
        BpmnNodeKind::StartEvent => "start_event",
        BpmnNodeKind::EndEvent => "end_event",
        BpmnNodeKind::IntermediateThrowEvent => "intermediate_throw_event",
        BpmnNodeKind::IntermediateCatchEvent => "intermediate_catch_event",
        BpmnNodeKind::BoundaryEvent => "boundary_event",
        BpmnNodeKind::SendTask => "send_task",
        BpmnNodeKind::ReceiveTask => "receive_task",
        BpmnNodeKind::ServiceTask => "service_task",
        BpmnNodeKind::ScriptTask => "script_task",
        BpmnNodeKind::UserTask => "user_task",
        BpmnNodeKind::ManualTask => "manual_task",
        BpmnNodeKind::BusinessRuleTask => "business_rule_task",
        BpmnNodeKind::Gateway => "gateway",
        BpmnNodeKind::SubProcess => "sub_process",
    }
}
