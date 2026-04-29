use crate::bpmn_cli::deps::{
    BpmnAdvanceOutcome, BpmnEventKind, BpmnNodeKind, BpmnProcessSpec, BpmnTimerKind, BpmnTimerSpec,
    InstanceLifecycle, NodeRuntimeStatus, PendingHostWorkKind, QianjiBpmnCheckpointStore,
    QianjiBpmnWorkflowCheckpointBackend, SuspendReason, WaitKind,
};

pub(in crate::bpmn_cli::render) fn bpmn_checkpoint_backend_label(
    store: &QianjiBpmnCheckpointStore,
) -> &'static str {
    match store {
        QianjiBpmnCheckpointStore::Valkey { .. } => "runtime_valkey",
        #[cfg(feature = "duckdb")]
        QianjiBpmnCheckpointStore::DuckDb { .. } => "duckdb",
    }
}

pub(in crate::bpmn_cli::render) fn bpmn_checkpoint_backend_selection_label(
    backend: &QianjiBpmnWorkflowCheckpointBackend,
) -> &'static str {
    match backend {
        QianjiBpmnWorkflowCheckpointBackend::RuntimeValkey => "runtime_valkey",
        #[cfg(feature = "duckdb")]
        QianjiBpmnWorkflowCheckpointBackend::LocalDuckDb => "duckdb",
    }
}

pub(in crate::bpmn_cli::render) fn bpmn_outcome_label(
    outcome: &BpmnAdvanceOutcome,
) -> &'static str {
    match outcome {
        BpmnAdvanceOutcome::Advanced => "advanced",
        BpmnAdvanceOutcome::BlockedOnHost(_) => "blocked_on_host",
        BpmnAdvanceOutcome::WaitingExternalEvent => "waiting_external_event",
        BpmnAdvanceOutcome::Suspended(_) => "suspended",
        BpmnAdvanceOutcome::Completed => "completed",
        BpmnAdvanceOutcome::Failed(_) => "failed",
    }
}

pub(in crate::bpmn_cli::render) fn bpmn_lifecycle_label(
    lifecycle: &InstanceLifecycle,
) -> &'static str {
    match lifecycle {
        InstanceLifecycle::Ready => "ready",
        InstanceLifecycle::Running => "running",
        InstanceLifecycle::Waiting => "waiting",
        InstanceLifecycle::Suspended => "suspended",
        InstanceLifecycle::Completed => "completed",
        InstanceLifecycle::Failed => "failed",
    }
}

pub(in crate::bpmn_cli::render) fn bpmn_suspend_reason_label(
    reason: &SuspendReason,
) -> &'static str {
    match reason {
        SuspendReason::HostRequested => "host_requested",
        SuspendReason::ExternalWait => "external_wait",
        SuspendReason::DmnPlaceholder => "dmn_placeholder",
        SuspendReason::ScaffoldBoundary => "scaffold_boundary",
    }
}

pub(in crate::bpmn_cli::render) fn bpmn_wait_kind_label(kind: &WaitKind) -> &'static str {
    match kind {
        WaitKind::ExternalEvent => "external_event",
        WaitKind::UserAction => "user_action",
        WaitKind::Conditional => "conditional",
        WaitKind::Timer => "timer",
    }
}

pub(in crate::bpmn_cli::render) fn bpmn_pending_host_work_kind_label(
    kind: &PendingHostWorkKind,
) -> &'static str {
    match kind {
        PendingHostWorkKind::Send => "send",
        PendingHostWorkKind::Service => "service",
        PendingHostWorkKind::Script => "script",
        PendingHostWorkKind::User => "user",
        PendingHostWorkKind::Manual => "manual",
        PendingHostWorkKind::BusinessRule => "business_rule",
    }
}

pub(in crate::bpmn_cli::render) fn bpmn_event_kind_label(kind: &BpmnEventKind) -> &'static str {
    match kind {
        BpmnEventKind::Timer => "timer",
        BpmnEventKind::Message => "message",
        BpmnEventKind::Signal => "signal",
        BpmnEventKind::Error => "error",
        BpmnEventKind::Cancel => "cancel",
        BpmnEventKind::Compensation => "compensation",
        BpmnEventKind::Conditional => "conditional",
        BpmnEventKind::Terminate => "terminate",
    }
}

pub(in crate::bpmn_cli::render) fn bpmn_timer_spec_label(timer: &BpmnTimerSpec) -> String {
    let kind = match timer.kind {
        BpmnTimerKind::Date => "date",
        BpmnTimerKind::Duration => "duration",
        BpmnTimerKind::Cycle => "cycle",
    };
    format!("{kind}:{}", timer.expression)
}

pub(in crate::bpmn_cli::render) fn bpmn_node_id_label(
    process: &BpmnProcessSpec,
    node_index: u32,
) -> String {
    process.nodes.get(node_index as usize).map_or_else(
        || format!("node#{node_index}"),
        |node| node.bpmn_id.to_string(),
    )
}

pub(in crate::bpmn_cli::render) fn node_runtime_status_label(
    status: &NodeRuntimeStatus,
) -> &'static str {
    match status {
        NodeRuntimeStatus::Idle => "idle",
        NodeRuntimeStatus::Queued => "queued",
        NodeRuntimeStatus::Executing => "executing",
        NodeRuntimeStatus::Completed => "completed",
        NodeRuntimeStatus::Cancelled => "cancelled",
        NodeRuntimeStatus::Failed => "failed",
    }
}

pub(in crate::bpmn_cli::render) fn bpmn_node_kind_label(kind: &BpmnNodeKind) -> &'static str {
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
