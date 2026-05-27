//! Server-owned BPMN execution-graph projection for control runs.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use xiuxian_qianji_control::{ControlEventKind, ControlEventRecord, RunId};

/// HTTP response for one control-ledger execution-graph query.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct QianjiControlExecutionGraphHttpResponse {
    /// Stable control-plane run identifier.
    pub(super) run_id: String,
    /// Number of BPMN elements projected from the control run.
    pub(super) element_count: usize,
    /// Stable BPMN element states keyed by server-recorded element ids.
    #[serde(default)]
    pub(super) elements: Vec<QianjiControlExecutionGraphElementHttpResponse>,
}

impl QianjiControlExecutionGraphHttpResponse {
    pub(super) fn from_events(run_id: &RunId, events: &[ControlEventRecord]) -> Self {
        let mut elements = BTreeMap::new();
        for record in events {
            let Some(element) = element_from_record(record) else {
                continue;
            };
            elements.insert(element.element_id.clone(), element);
        }
        let elements = elements.into_values().collect::<Vec<_>>();
        Self {
            run_id: run_id.as_str().to_owned(),
            element_count: elements.len(),
            elements,
        }
    }
}

/// Server-normalized state for one BPMN element.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(super) struct QianjiControlExecutionGraphElementHttpResponse {
    /// Stable BPMN element identifier.
    pub(super) element_id: String,
    /// Server-derived runtime state for rendering markers.
    pub(super) state: QianjiControlExecutionGraphElementState,
    /// Control ledger sequence that produced this state.
    pub(super) source_event_sequence: u64,
    /// Control event kind that produced this state.
    pub(super) source_event_kind: String,
    /// Event timestamp in Unix milliseconds.
    pub(super) occurred_at_ms: u64,
    /// Operator-readable state message.
    pub(super) message: String,
    /// Source metadata retained for diagnostics and hover panels.
    #[serde(default)]
    pub(super) metadata: Value,
}

/// Server-derived marker state for one BPMN element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum QianjiControlExecutionGraphElementState {
    /// Element has pending or active work.
    Active,
    /// Element completed successfully.
    Completed,
    /// Element is failed, cancelled, or blocked.
    Failed,
}

fn element_from_record(
    record: &ControlEventRecord,
) -> Option<QianjiControlExecutionGraphElementHttpResponse> {
    let element_id = element_id_from_record(record)?;
    let state = state_from_event_kind(&record.event.kind)?;
    let source_event_kind = event_kind_name(&record.event.kind);
    Some(QianjiControlExecutionGraphElementHttpResponse {
        element_id,
        state,
        source_event_sequence: record.sequence,
        source_event_kind: source_event_kind.to_owned(),
        occurred_at_ms: record.event.occurred_at_ms,
        message: source_event_kind.replace('_', " "),
        metadata: metadata_from_event_kind(&record.event.kind),
    })
}

fn element_id_from_record(record: &ControlEventRecord) -> Option<String> {
    record
        .event
        .step_id
        .as_ref()
        .map(|step_id| step_id.as_str().to_owned())
        .or_else(|| element_id_from_event_kind(&record.event.kind))
}

fn element_id_from_event_kind(kind: &ControlEventKind) -> Option<String> {
    match kind {
        ControlEventKind::ToolCallRecorded { metadata, .. } => element_id_from_metadata(metadata),
        ControlEventKind::ActivityScheduled { task } => element_id_from_metadata(&task.metadata)
            .or_else(|| Some(task.activity_id.as_str().to_owned())),
        ControlEventKind::ActivityStarted { activity_id, .. }
        | ControlEventKind::ActivityCompleted { activity_id, .. }
        | ControlEventKind::ActivityFailed { activity_id, .. } => {
            Some(activity_id.as_str().to_owned())
        }
        _ => None,
    }
}

fn element_id_from_metadata(metadata: &Value) -> Option<String> {
    [
        "bpmnElementId",
        "bpmn_element_id",
        "element_id",
        "node_id",
        "activity_id",
    ]
    .iter()
    .find_map(|key| metadata.get(key).and_then(Value::as_str))
    .map(str::to_owned)
}

fn state_from_event_kind(
    kind: &ControlEventKind,
) -> Option<QianjiControlExecutionGraphElementState> {
    match kind {
        ControlEventKind::StepQueued
        | ControlEventKind::StepStarted
        | ControlEventKind::StepWaiting { .. }
        | ControlEventKind::ActivityScheduled { .. }
        | ControlEventKind::ActivityStarted { .. } => {
            Some(QianjiControlExecutionGraphElementState::Active)
        }
        ControlEventKind::StepSucceeded | ControlEventKind::ActivityCompleted { .. } => {
            Some(QianjiControlExecutionGraphElementState::Completed)
        }
        ControlEventKind::StepFailed { .. }
        | ControlEventKind::StepBlocked { .. }
        | ControlEventKind::StepCancelled { .. }
        | ControlEventKind::ActivityFailed { .. } => {
            Some(QianjiControlExecutionGraphElementState::Failed)
        }
        ControlEventKind::ToolCallRecorded { metadata, .. } => state_from_runtime_status(metadata)
            .or(Some(QianjiControlExecutionGraphElementState::Completed)),
        _ => None,
    }
}

fn state_from_runtime_status(metadata: &Value) -> Option<QianjiControlExecutionGraphElementState> {
    let status = metadata.get("runtimeStatus").and_then(Value::as_str)?;
    match status {
        "queued" | "executing" | "Queued" | "Executing" => {
            Some(QianjiControlExecutionGraphElementState::Active)
        }
        "completed" | "Completed" => Some(QianjiControlExecutionGraphElementState::Completed),
        "failed" | "cancelled" | "Failed" | "Cancelled" => {
            Some(QianjiControlExecutionGraphElementState::Failed)
        }
        _ => None,
    }
}

fn metadata_from_event_kind(kind: &ControlEventKind) -> Value {
    match kind {
        ControlEventKind::RunCreated { metadata, .. }
        | ControlEventKind::ToolCallRecorded { metadata, .. } => metadata.clone(),
        ControlEventKind::ActivityScheduled { task } => {
            let mut metadata = metadata_object(task.metadata.clone());
            metadata.insert("activity_id".to_owned(), json!(task.activity_id.as_str()));
            metadata.insert(
                "activity_type".to_owned(),
                json!(task.activity_type.as_str()),
            );
            metadata.insert("task_queue".to_owned(), json!(task.task_queue.as_str()));
            Value::Object(metadata)
        }
        ControlEventKind::ActivityCompleted { result, .. } => result.metadata.clone(),
        ControlEventKind::ActivityFailed { failure, .. } => failure.metadata.clone(),
        _ => Value::Object(Map::new()),
    }
}

fn metadata_object(metadata: Value) -> Map<String, Value> {
    match metadata {
        Value::Object(map) => map,
        _ => Map::new(),
    }
}

fn event_kind_name(kind: &ControlEventKind) -> &'static str {
    match kind {
        ControlEventKind::RunCreated { .. } => "run_created",
        ControlEventKind::RunAdmitted => "run_admitted",
        ControlEventKind::PlanRecorded { .. } => "plan_recorded",
        ControlEventKind::StepCreated { .. } => "step_created",
        ControlEventKind::StepQueued => "step_queued",
        ControlEventKind::StepLeaseAcquired { .. } => "step_lease_acquired",
        ControlEventKind::StepLeaseRenewed { .. } => "step_lease_renewed",
        ControlEventKind::StepLeaseReleased { .. } => "step_lease_released",
        ControlEventKind::StepStarted => "step_started",
        ControlEventKind::StepWaiting { .. } => "step_waiting",
        ControlEventKind::ToolCallRecorded { .. } => "tool_call_recorded",
        ControlEventKind::AgentProposalRecorded { .. } => "agent_proposal_recorded",
        ControlEventKind::AgentDecisionRecorded { .. } => "agent_decision_recorded",
        ControlEventKind::ActivityScheduled { .. } => "activity_scheduled",
        ControlEventKind::ActivityStarted { .. } => "activity_started",
        ControlEventKind::ActivityCompleted { .. } => "activity_completed",
        ControlEventKind::ActivityFailed { .. } => "activity_failed",
        ControlEventKind::SignalReceived { .. } => "signal_received",
        ControlEventKind::TimerScheduled { .. } => "timer_scheduled",
        ControlEventKind::TimerFired { .. } => "timer_fired",
        ControlEventKind::VersionPinned { .. } => "version_pinned",
        ControlEventKind::ArtifactAttached { .. } => "artifact_attached",
        ControlEventKind::EvidenceAttached { .. } => "evidence_attached",
        ControlEventKind::CostObserved { .. } => "cost_observed",
        ControlEventKind::GateEvaluated { .. } => "gate_evaluated",
        ControlEventKind::RecoveryStarted { .. } => "recovery_started",
        ControlEventKind::WorkerHeartbeatObserved { .. } => "worker_heartbeat_observed",
        ControlEventKind::StepSucceeded => "step_succeeded",
        ControlEventKind::StepFailed { .. } => "step_failed",
        ControlEventKind::StepBlocked { .. } => "step_blocked",
        ControlEventKind::StepCancelled { .. } => "step_cancelled",
        ControlEventKind::RunCompleted => "run_completed",
        ControlEventKind::RunFailed { .. } => "run_failed",
        ControlEventKind::RunBlocked { .. } => "run_blocked",
        ControlEventKind::RunAborted { .. } => "run_aborted",
    }
}
