use std::fmt::Write as _;

use crate::bpmn_cli::deps::{
    BpmnAdvanceOutcome, BpmnEventKind, BpmnInstanceState, BpmnNodeKind, BpmnPackage,
    BpmnProcessSpec, BpmnTimerKind, BpmnTimerSpec, InstanceLifecycle, NodeRuntimeStatus,
    PendingHostWorkKind, QianjiBpmnCheckpointStore, QianjiBpmnWorkflowCheckpointBackend,
    SuspendReason, WaitKind,
};

pub(super) fn append_bpmn_wait_registrations(
    rendered: &mut String,
    package: &BpmnPackage,
    instance: &BpmnInstanceState,
) {
    if instance.waits.is_empty() {
        return;
    }

    let Some(process) = package.find_process(instance.process.process_id.as_ref()) else {
        return;
    };
    let mut wait_lines = instance
        .waits
        .iter()
        .map(|wait| {
            let wait_id = render_bpmn_wait_node_id(process, wait.node_index);
            let mut line = format!("- {wait_id} | kind={}", bpmn_wait_kind_label(&wait.kind));
            if let Some(event_kind) = wait.event_kind.as_ref() {
                let _ = write!(line, " | event={}", bpmn_event_kind_label(event_kind));
            }
            if let Some(reference) = wait.event_reference.as_ref() {
                let _ = write!(line, " | ref={reference}");
            }
            if let Some(name) = wait.event_name.as_ref() {
                let _ = write!(line, " | name={name}");
            }
            if let Some(timer) = wait.timer.as_ref() {
                let _ = write!(line, " | timer={}", bpmn_timer_spec_label(timer));
            }
            if let Some(blocking_node_index) = wait.blocking_node_index {
                let _ = write!(
                    line,
                    " | blocking={}",
                    render_bpmn_wait_node_id(process, blocking_node_index)
                );
            }
            if let Some(correlation_key) = wait.correlation_key.as_ref() {
                let _ = write!(line, " | correlation={correlation_key}");
            }
            (wait_id, line)
        })
        .collect::<Vec<_>>();
    wait_lines.sort_by(|left, right| left.0.cmp(&right.0));

    let _ = writeln!(rendered, "\n## Wait Registrations");

    if let Some(competition) = instance.event_competition.as_ref() {
        let _ = writeln!(
            rendered,
            "Competition gateway: {}",
            render_bpmn_wait_node_id(process, competition.gateway_node_index)
        );
    }

    let wait_key = wait_lines
        .iter()
        .map(|(wait_id, _)| wait_id.as_str())
        .collect::<Vec<_>>()
        .join("|");
    let _ = writeln!(rendered, "Event fixture key: {wait_key}");

    for (_, line) in wait_lines {
        let _ = writeln!(rendered, "{line}");
    }
}

pub(super) fn bpmn_checkpoint_backend_label(store: &QianjiBpmnCheckpointStore) -> &'static str {
    match store {
        QianjiBpmnCheckpointStore::Valkey { .. } => "runtime_valkey",
        #[cfg(feature = "duckdb")]
        QianjiBpmnCheckpointStore::DuckDb { .. } => "duckdb",
    }
}

pub(super) fn bpmn_checkpoint_backend_selection_label(
    backend: &QianjiBpmnWorkflowCheckpointBackend,
) -> &'static str {
    match backend {
        QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey => "runtime_valkey",
        #[cfg(feature = "duckdb")]
        QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb => "duckdb",
    }
}

pub(super) fn bpmn_outcome_label(outcome: &BpmnAdvanceOutcome) -> &'static str {
    match outcome {
        BpmnAdvanceOutcome::Advanced => "advanced",
        BpmnAdvanceOutcome::BlockedOnHost(_) => "blocked_on_host",
        BpmnAdvanceOutcome::WaitingExternalEvent => "waiting_external_event",
        BpmnAdvanceOutcome::Suspended(_) => "suspended",
        BpmnAdvanceOutcome::Completed => "completed",
        BpmnAdvanceOutcome::Failed(_) => "failed",
    }
}

pub(super) fn bpmn_lifecycle_label(lifecycle: &InstanceLifecycle) -> &'static str {
    match lifecycle {
        InstanceLifecycle::Ready => "ready",
        InstanceLifecycle::Running => "running",
        InstanceLifecycle::Waiting => "waiting",
        InstanceLifecycle::Suspended => "suspended",
        InstanceLifecycle::Completed => "completed",
        InstanceLifecycle::Failed => "failed",
    }
}

pub(super) fn bpmn_suspend_reason_label(reason: &SuspendReason) -> &'static str {
    match reason {
        SuspendReason::HostRequested => "host_requested",
        SuspendReason::ExternalWait => "external_wait",
        SuspendReason::DmnPlaceholder => "dmn_placeholder",
        SuspendReason::ScaffoldBoundary => "scaffold_boundary",
    }
}

pub(super) fn bpmn_wait_kind_label(kind: &WaitKind) -> &'static str {
    match kind {
        WaitKind::ExternalEvent => "external_event",
        WaitKind::UserAction => "user_action",
        WaitKind::Timer => "timer",
    }
}

pub(super) fn bpmn_pending_host_work_kind_label(kind: &PendingHostWorkKind) -> &'static str {
    match kind {
        PendingHostWorkKind::Send => "send",
        PendingHostWorkKind::Service => "service",
        PendingHostWorkKind::Script => "script",
        PendingHostWorkKind::User => "user",
        PendingHostWorkKind::Manual => "manual",
        PendingHostWorkKind::BusinessRule => "business_rule",
    }
}

pub(super) fn bpmn_event_kind_label(kind: &BpmnEventKind) -> &'static str {
    match kind {
        BpmnEventKind::Timer => "timer",
        BpmnEventKind::Message => "message",
        BpmnEventKind::Signal => "signal",
        BpmnEventKind::Error => "error",
        BpmnEventKind::Cancel => "cancel",
        BpmnEventKind::Compensation => "compensation",
        BpmnEventKind::Conditional => "conditional",
    }
}

pub(super) fn bpmn_timer_spec_label(timer: &BpmnTimerSpec) -> String {
    let kind = match timer.kind {
        BpmnTimerKind::Date => "date",
        BpmnTimerKind::Duration => "duration",
        BpmnTimerKind::Cycle => "cycle",
    };
    format!("{kind}:{}", timer.expression)
}

fn render_bpmn_wait_node_id(process: &BpmnProcessSpec, node_index: u32) -> String {
    bpmn_node_id_label(process, node_index)
}

pub(super) fn bpmn_node_id_label(process: &BpmnProcessSpec, node_index: u32) -> String {
    process.nodes.get(node_index as usize).map_or_else(
        || format!("node#{node_index}"),
        |node| node.bpmn_id.to_string(),
    )
}

pub(super) fn node_runtime_status_label(status: &NodeRuntimeStatus) -> &'static str {
    match status {
        NodeRuntimeStatus::Idle => "idle",
        NodeRuntimeStatus::Queued => "queued",
        NodeRuntimeStatus::Executing => "executing",
        NodeRuntimeStatus::Completed => "completed",
        NodeRuntimeStatus::Cancelled => "cancelled",
        NodeRuntimeStatus::Failed => "failed",
    }
}

pub(super) fn bpmn_node_kind_label(kind: &BpmnNodeKind) -> &'static str {
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
