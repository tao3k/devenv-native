//! Deterministic rows for the qianji run-console read model.

use num_traits::ToPrimitive;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use xiuxian_qianji_control::{ControlEventKind, ControlEventRecord, RunId};

/// Shared schema version for qianji run-console data-plane rows.
pub const QIANJI_RUN_CONSOLE_SCHEMA_VERSION: &str = "pi-wendao.qianji.run-console.v1";
/// Schema version for the server-owned durable run stream projection.
pub const QIANJI_CONTROL_RUN_STREAM_SCHEMA_VERSION: &str = "xiuxian_qianji.control.run_stream.v1";
/// Logical Flight route for control-event rows.
pub const QIANJI_RUN_CONSOLE_EVENT_ROUTE: &str = "qianji.control.run-console.events";
/// Logical Flight route for BPMN element-state rows.
pub const QIANJI_RUN_CONSOLE_ELEMENT_STATE_ROUTE: &str =
    "qianji.control.run-console.element-states";

/// One qianji control event projected into the run-console Arrow contract.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QianjiRunConsoleEventRow {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Stable event identifier inside the run-console read model.
    pub event_id: String,
    /// Ledger sequence encoded as an Arrow `Int32`.
    pub sequence: i32,
    /// Stable event-kind name.
    pub kind: String,
    /// Operator-readable event message.
    pub message: String,
    /// Optional BPMN/control step id.
    pub step_id: Option<String>,
    /// Unix timestamp in milliseconds encoded as an Arrow `Float64`.
    pub occurred_at_ms: f64,
}

/// One BPMN element state projected into the run-console Arrow contract.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct QianjiRunConsoleElementStateRow {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Stable BPMN element identifier.
    pub element_id: String,
    /// Server-derived runtime state.
    pub state: QianjiRunConsoleElementState,
    /// Source control event id or sequence.
    pub source_event_id: String,
    /// Operator-readable state message.
    pub message: String,
}

/// Internal projection retained so JSON HTTP and Arrow rows share derivation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QianjiRunConsoleElementProjection {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Stable BPMN element identifier.
    pub element_id: String,
    /// Server-derived runtime state for rendering markers.
    pub state: QianjiRunConsoleElementState,
    /// Control ledger sequence that produced this state.
    pub source_event_sequence: u64,
    /// Control event kind that produced this state.
    pub source_event_kind: String,
    /// Event timestamp in Unix milliseconds.
    pub occurred_at_ms: u64,
    /// Operator-readable state message.
    pub message: String,
    /// Source metadata retained for diagnostics and hover panels.
    #[serde(default)]
    pub metadata: Value,
}

impl QianjiRunConsoleElementProjection {
    /// Convert this projection into the public element-state row contract.
    #[must_use]
    pub(crate) fn to_row(&self) -> QianjiRunConsoleElementStateRow {
        QianjiRunConsoleElementStateRow {
            run_id: self.run_id.clone(),
            element_id: self.element_id.clone(),
            state: self.state,
            source_event_id: self.source_event_sequence.to_string(),
            message: self.message.clone(),
        }
    }
}

/// Durable run-stream source lane rendered by workflow UIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QianjiControlRunStreamSource {
    /// BPMN control-plane or step lifecycle event.
    Bpmn,
    /// LLM activity event.
    Llm,
    /// Agent or subagent proposal/decision event.
    Subagent,
    /// Tool or host-work event.
    Tool,
    /// System lifecycle event.
    System,
}

impl QianjiControlRunStreamSource {
    /// Returns the stable wire value for this source lane.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Bpmn => "bpmn",
            Self::Llm => "llm",
            Self::Subagent => "subagent",
            Self::Tool => "tool",
            Self::System => "system",
        }
    }
}

/// One row in qianji-server's durable run-stream projection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QianjiControlRunStreamRow {
    /// Stable control-plane run identifier.
    pub run_id: String,
    /// Stable row id inside the run stream.
    pub row_id: String,
    /// Ledger sequence that produced this stream row.
    pub sequence: u64,
    /// Stream lane used by UI renderers.
    pub source: QianjiControlRunStreamSource,
    /// Stable control event kind.
    pub kind: String,
    /// Compact operator-readable title.
    pub title: String,
    /// Compact operator-readable message.
    pub message: String,
    /// Activity id when the event belongs to an external activity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activity_id: Option<String>,
    /// Step id when the event belongs to a control step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<String>,
    /// BPMN element id when the event can be pinned to the diagram.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub element_id: Option<String>,
    /// Event timestamp in Unix milliseconds.
    pub occurred_at_ms: u64,
    /// Source metadata retained for diagnostics and richer clients.
    #[serde(default)]
    pub metadata: Value,
}

/// Server-derived marker state for one BPMN element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QianjiRunConsoleElementState {
    /// Element has pending or active work.
    Active,
    /// Element completed successfully.
    Completed,
    /// Element is failed, cancelled, or blocked.
    Failed,
}

impl QianjiRunConsoleElementState {
    /// Return the string used by the JS Arrow row contract.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

/// Project control events into qianji run-console event rows.
///
/// # Errors
///
/// Returns an error when a ledger sequence exceeds the Arrow `Int32` range
/// used by the JavaScript read-model contract.
pub(crate) fn qianji_run_console_event_rows(
    run_id: &RunId,
    events: &[ControlEventRecord],
) -> Result<Vec<QianjiRunConsoleEventRow>, String> {
    events
        .iter()
        .map(|record| event_row_from_record(run_id, record))
        .collect()
}

/// Project control events into the server-owned durable run stream.
#[must_use]
pub(crate) fn qianji_control_run_stream_rows(
    run_id: &RunId,
    events: &[ControlEventRecord],
) -> Vec<QianjiControlRunStreamRow> {
    events
        .iter()
        .map(|record| control_run_stream_row_from_record(run_id, record))
        .collect()
}

/// Project control events into latest BPMN element-state rows.
#[must_use]
pub(crate) fn qianji_run_console_element_state_rows(
    run_id: &RunId,
    events: &[ControlEventRecord],
) -> Vec<QianjiRunConsoleElementStateRow> {
    qianji_run_console_element_projections(run_id, events)
        .iter()
        .map(QianjiRunConsoleElementProjection::to_row)
        .collect()
}

/// Project control events into latest BPMN element projections.
#[must_use]
pub(crate) fn qianji_run_console_element_projections(
    run_id: &RunId,
    events: &[ControlEventRecord],
) -> Vec<QianjiRunConsoleElementProjection> {
    let mut elements = BTreeMap::new();
    for record in events {
        let Some(element) = element_from_record(run_id, record) else {
            continue;
        };
        if let Some(existing) = elements.get(&element.element_id)
            && should_keep_existing_element_state(existing, &element)
        {
            continue;
        }
        elements.insert(element.element_id.clone(), element);
    }
    elements.into_values().collect()
}

fn should_keep_existing_element_state(
    existing: &QianjiRunConsoleElementProjection,
    next: &QianjiRunConsoleElementProjection,
) -> bool {
    matches!(
        existing.state,
        QianjiRunConsoleElementState::Completed | QianjiRunConsoleElementState::Failed
    ) && next.state == QianjiRunConsoleElementState::Active
        && matches!(
            next.source_event_kind.as_str(),
            "step_queued" | "step_started"
        )
}

fn event_row_from_record(
    run_id: &RunId,
    record: &ControlEventRecord,
) -> Result<QianjiRunConsoleEventRow, String> {
    let sequence = i32::try_from(record.sequence).map_err(|_| {
        format!(
            "qianji run-console event sequence {} exceeds Int32 range",
            record.sequence
        )
    })?;
    Ok(QianjiRunConsoleEventRow {
        run_id: run_id.as_str().to_owned(),
        event_id: record.sequence.to_string(),
        sequence,
        kind: event_kind_name(&record.event.kind).to_owned(),
        message: event_message(&record.event.kind),
        step_id: record
            .event
            .step_id
            .as_ref()
            .map(|step_id| step_id.as_str().to_owned()),
        occurred_at_ms: record
            .event
            .occurred_at_ms
            .to_f64()
            .ok_or_else(|| "event timestamp cannot be represented as Float64".to_string())?,
    })
}

fn control_run_stream_row_from_record(
    run_id: &RunId,
    record: &ControlEventRecord,
) -> QianjiControlRunStreamRow {
    let kind = event_kind_name(&record.event.kind);
    let source = stream_source_from_event_kind(&record.event.kind);
    QianjiControlRunStreamRow {
        run_id: run_id.as_str().to_owned(),
        row_id: format!("stream-{}", record.sequence),
        sequence: record.sequence,
        source,
        kind: kind.to_owned(),
        title: stream_title(source, kind),
        message: event_message(&record.event.kind),
        activity_id: activity_id_from_event_kind(&record.event.kind),
        step_id: record
            .event
            .step_id
            .as_ref()
            .map(|step_id| step_id.as_str().to_owned()),
        element_id: element_id_from_record(record),
        occurred_at_ms: record.event.occurred_at_ms,
        metadata: metadata_from_event_kind(&record.event.kind),
    }
}

fn element_from_record(
    run_id: &RunId,
    record: &ControlEventRecord,
) -> Option<QianjiRunConsoleElementProjection> {
    let element_id = element_id_from_record(record)?;
    let state = state_from_event_kind(&record.event.kind)?;
    let source_event_kind = event_kind_name(&record.event.kind);
    Some(QianjiRunConsoleElementProjection {
        run_id: run_id.as_str().to_owned(),
        element_id,
        state,
        source_event_sequence: record.sequence,
        source_event_kind: source_event_kind.to_owned(),
        occurred_at_ms: record.event.occurred_at_ms,
        message: event_message(&record.event.kind),
        metadata: metadata_from_event_kind(&record.event.kind),
    })
}

fn element_id_from_record(record: &ControlEventRecord) -> Option<String> {
    match &record.event.kind {
        ControlEventKind::ToolCallRecorded { .. }
        | ControlEventKind::ActivityScheduled { .. }
        | ControlEventKind::ActivityStarted { .. }
        | ControlEventKind::ActivityCompleted { .. }
        | ControlEventKind::ActivityFailed { .. } => element_id_from_event_kind(&record.event.kind)
            .or_else(|| {
                record
                    .event
                    .step_id
                    .as_ref()
                    .map(|step_id| step_id.as_str().to_owned())
            }),
        _ => record
            .event
            .step_id
            .as_ref()
            .map(|step_id| step_id.as_str().to_owned()),
    }
}

fn stream_title(source: QianjiControlRunStreamSource, kind: &str) -> String {
    format!(
        "{} {}",
        match source {
            QianjiControlRunStreamSource::Bpmn => "BPMN",
            QianjiControlRunStreamSource::Llm => "LLM",
            QianjiControlRunStreamSource::Subagent => "Agent",
            QianjiControlRunStreamSource::Tool => "Tool",
            QianjiControlRunStreamSource::System => "System",
        },
        kind.replace('_', " ")
    )
}

fn activity_id_from_event_kind(kind: &ControlEventKind) -> Option<String> {
    match kind {
        ControlEventKind::ActivityScheduled { task } => Some(task.activity_id.as_str().to_owned()),
        ControlEventKind::ActivityStarted { activity_id, .. }
        | ControlEventKind::ActivityCompleted { activity_id, .. }
        | ControlEventKind::ActivityFailed { activity_id, .. } => {
            Some(activity_id.as_str().to_owned())
        }
        _ => None,
    }
}

fn stream_source_from_event_kind(kind: &ControlEventKind) -> QianjiControlRunStreamSource {
    match kind {
        ControlEventKind::AgentProposalRecorded { .. }
        | ControlEventKind::AgentDecisionRecorded { .. } => QianjiControlRunStreamSource::Subagent,
        ControlEventKind::ToolCallRecorded { .. } => QianjiControlRunStreamSource::Tool,
        ControlEventKind::ActivityScheduled { task } => stream_source_from_activity(
            Some(task.activity_id.as_str()),
            Some(task.activity_type.as_str()),
            Some(task.task_queue.as_str()),
            &task.metadata,
        ),
        ControlEventKind::ActivityStarted { activity_id, .. } => {
            stream_source_from_activity(Some(activity_id.as_str()), None, None, &Value::Null)
        }
        ControlEventKind::ActivityCompleted {
            activity_id,
            result,
            ..
        } => stream_source_from_activity(Some(activity_id.as_str()), None, None, &result.metadata),
        ControlEventKind::ActivityFailed {
            activity_id,
            failure,
            ..
        } => stream_source_from_activity(Some(activity_id.as_str()), None, None, &failure.metadata),
        ControlEventKind::SignalReceived { .. }
        | ControlEventKind::TimerScheduled { .. }
        | ControlEventKind::TimerFired { .. }
        | ControlEventKind::VersionPinned { .. }
        | ControlEventKind::ArtifactAttached { .. }
        | ControlEventKind::EvidenceAttached { .. }
        | ControlEventKind::CostObserved { .. }
        | ControlEventKind::GateEvaluated { .. }
        | ControlEventKind::RecoveryStarted { .. }
        | ControlEventKind::WorkerHeartbeatObserved { .. } => QianjiControlRunStreamSource::System,
        _ => QianjiControlRunStreamSource::Bpmn,
    }
}

fn stream_source_from_activity(
    activity_id: Option<&str>,
    activity_type: Option<&str>,
    task_queue: Option<&str>,
    metadata: &Value,
) -> QianjiControlRunStreamSource {
    if [activity_id, activity_type, task_queue]
        .into_iter()
        .flatten()
        .any(is_llm_text)
        || metadata_contains_token(metadata, "llm")
    {
        return QianjiControlRunStreamSource::Llm;
    }
    if [activity_id, activity_type, task_queue]
        .into_iter()
        .flatten()
        .any(is_subagent_text)
        || metadata_contains_token(metadata, "subagent")
    {
        return QianjiControlRunStreamSource::Subagent;
    }
    QianjiControlRunStreamSource::Tool
}

fn is_llm_text(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value == "llm" || value.starts_with("llm.") || value.contains(".llm")
}

fn is_subagent_text(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.contains("subagent") || value.contains("pi-subagents")
}

fn metadata_contains_token(value: &Value, token: &str) -> bool {
    match value {
        Value::String(text) => text.to_ascii_lowercase().contains(token),
        Value::Array(items) => items
            .iter()
            .any(|item| metadata_contains_token(item, token)),
        Value::Object(map) => map.iter().any(|(key, nested)| {
            key.to_ascii_lowercase().contains(token) || metadata_contains_token(nested, token)
        }),
        Value::Null | Value::Bool(_) | Value::Number(_) => false,
    }
}

fn element_id_from_event_kind(kind: &ControlEventKind) -> Option<String> {
    match kind {
        ControlEventKind::ToolCallRecorded { metadata, .. } => element_id_from_metadata(metadata),
        ControlEventKind::ActivityScheduled { task } => element_id_from_metadata(&task.metadata)
            .or_else(|| element_id_from_control_activity_id(task.activity_id.as_str())),
        ControlEventKind::ActivityStarted { activity_id, .. } => {
            element_id_from_control_activity_id(activity_id.as_str())
        }
        ControlEventKind::ActivityCompleted {
            activity_id,
            result,
            ..
        } => element_id_from_metadata(&result.metadata)
            .or_else(|| element_id_from_control_activity_id(activity_id.as_str())),
        ControlEventKind::ActivityFailed {
            activity_id,
            failure,
            ..
        } => element_id_from_metadata(&failure.metadata)
            .or_else(|| element_id_from_control_activity_id(activity_id.as_str())),
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
    .or_else(|| nested_element_id_from_metadata(metadata, "qianji_bpmn_host_work_activity"))
    .or_else(|| nested_element_id_from_metadata(metadata, "qianji_bpmn_host_work_completion"))
}

fn nested_element_id_from_metadata(metadata: &Value, key: &str) -> Option<String> {
    let nested = metadata.get(key)?;
    [
        "bpmnElementId",
        "bpmn_element_id",
        "element_id",
        "node_id",
        "activityId",
        "activity_id",
    ]
    .iter()
    .find_map(|field| nested.get(field).and_then(Value::as_str))
    .map(str::to_owned)
}

fn element_id_from_control_activity_id(activity_id: &str) -> Option<String> {
    if !activity_id.starts_with("bpmn.") {
        return None;
    }
    let mut parts = activity_id.rsplit('.');
    let repeat_index = parts.next()?;
    if repeat_index.is_empty() || !repeat_index.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    parts
        .next()
        .filter(|element_id| !element_id.is_empty())
        .map(str::to_owned)
}

fn state_from_event_kind(kind: &ControlEventKind) -> Option<QianjiRunConsoleElementState> {
    match kind {
        ControlEventKind::StepQueued
        | ControlEventKind::StepStarted
        | ControlEventKind::StepWaiting { .. }
        | ControlEventKind::ActivityScheduled { .. }
        | ControlEventKind::ActivityStarted { .. } => Some(QianjiRunConsoleElementState::Active),
        ControlEventKind::StepSucceeded | ControlEventKind::ActivityCompleted { .. } => {
            Some(QianjiRunConsoleElementState::Completed)
        }
        ControlEventKind::StepFailed { .. }
        | ControlEventKind::StepBlocked { .. }
        | ControlEventKind::StepCancelled { .. }
        | ControlEventKind::ActivityFailed { .. } => Some(QianjiRunConsoleElementState::Failed),
        ControlEventKind::ToolCallRecorded { metadata, .. } => {
            state_from_runtime_status(metadata).or(Some(QianjiRunConsoleElementState::Completed))
        }
        _ => None,
    }
}

fn state_from_runtime_status(metadata: &Value) -> Option<QianjiRunConsoleElementState> {
    let status = metadata.get("runtimeStatus").and_then(Value::as_str)?;
    match status {
        "queued" | "executing" | "Queued" | "Executing" => {
            Some(QianjiRunConsoleElementState::Active)
        }
        "completed" | "Completed" => Some(QianjiRunConsoleElementState::Completed),
        "failed" | "cancelled" | "Failed" | "Cancelled" => {
            Some(QianjiRunConsoleElementState::Failed)
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

fn event_message(kind: &ControlEventKind) -> String {
    match kind {
        ControlEventKind::PlanRecorded { summary } => summary.clone(),
        ControlEventKind::StepCreated { title, .. } => title.clone(),
        ControlEventKind::ToolCallRecorded { tool_name, .. } => tool_name.clone(),
        ControlEventKind::ActivityScheduled { task } => {
            format!("{} scheduled", task.activity_type.as_str())
        }
        ControlEventKind::ActivityStarted { activity_id, .. } => {
            format!("{} started", activity_id.as_str())
        }
        ControlEventKind::ActivityCompleted { activity_id, .. } => {
            format!("{} completed", activity_id.as_str())
        }
        ControlEventKind::ActivityFailed { failure, .. } => failure.message.clone(),
        ControlEventKind::StepWaiting { reason } => format!("{reason:?}"),
        ControlEventKind::StepFailed { message, .. } | ControlEventKind::RunFailed { message } => {
            message.clone()
        }
        ControlEventKind::StepBlocked { reason }
        | ControlEventKind::StepCancelled { reason }
        | ControlEventKind::RunBlocked { reason }
        | ControlEventKind::RunAborted { reason } => reason.clone(),
        _ => event_kind_name(kind).replace('_', " "),
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
