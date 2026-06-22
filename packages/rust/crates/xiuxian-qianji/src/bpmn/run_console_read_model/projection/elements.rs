//! BPMN element-state row projection.

#[cfg(feature = "run-console-flight")]
use super::QianjiRunConsoleElementStateRow;
use super::{
    QianjiRunConsoleElementProjection, QianjiRunConsoleElementState,
    event_text::{event_kind_name, event_message},
    metadata::metadata_from_event_kind,
};
use serde_json::Value;
use std::collections::BTreeMap;
use xiuxian_qianji_control::{ControlEventKind, ControlEventRecord, RunId};

/// Project control events into latest BPMN element-state rows.
#[must_use]
#[cfg(feature = "run-console-flight")]
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

pub(super) fn element_id_from_event_kind(kind: &ControlEventKind) -> Option<String> {
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
