//! Durable run-stream row projection.

use super::{
    QianjiControlRunStreamKind, QianjiControlRunStreamRow, QianjiControlRunStreamSource,
    elements::element_id_from_event_kind,
    event_text::{event_kind_name, event_message},
    metadata::{
        ActivityStreamFacts, collect_activity_stream_facts, metadata_from_event_kind_with_facts,
    },
};
use serde_json::Value;
use std::collections::BTreeMap;
use xiuxian_qianji_control::{ControlEventKind, ControlEventRecord, RunId};

/// Project control events into the server-owned durable run stream.
#[must_use]
pub fn qianji_control_run_stream_rows(
    run_id: &RunId,
    events: &[ControlEventRecord],
) -> Vec<QianjiControlRunStreamRow> {
    let activity_facts = collect_activity_stream_facts(events);
    events
        .iter()
        .map(|record| control_run_stream_row_from_record(run_id, record, &activity_facts))
        .collect()
}

fn control_run_stream_row_from_record(
    run_id: &RunId,
    record: &ControlEventRecord,
    activity_facts: &BTreeMap<String, ActivityStreamFacts>,
) -> QianjiControlRunStreamRow {
    let kind = event_kind_name(&record.event.kind);
    let source = stream_source_from_event_kind(&record.event.kind, activity_facts);
    QianjiControlRunStreamRow {
        run_id: run_id.as_str().to_owned(),
        row_id: format!("stream-{}", record.sequence),
        sequence: record.sequence,
        source,
        kind: QianjiControlRunStreamKind::new(kind),
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
        metadata: metadata_from_event_kind_with_facts(&record.event.kind, activity_facts),
    }
}

fn element_id_from_record(record: &ControlEventRecord) -> Option<String> {
    element_id_from_event_kind(&record.event.kind).or_else(|| {
        record
            .event
            .step_id
            .as_ref()
            .map(|step_id| step_id.as_str().to_owned())
    })
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
        ControlEventKind::WorkerHeartbeatObserved { heartbeat } => {
            metadata_str(&heartbeat.metadata, "activity_id").map(str::to_owned)
        }
        _ => None,
    }
}

fn stream_source_from_event_kind(
    kind: &ControlEventKind,
    activity_facts: &BTreeMap<String, ActivityStreamFacts>,
) -> QianjiControlRunStreamSource {
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
            let facts = activity_facts.get(activity_id.as_str());
            stream_source_from_activity(
                Some(activity_id.as_str()),
                facts.map(|facts| facts.activity_type.as_str()),
                facts.map(|facts| facts.task_queue.as_str()),
                facts.map_or(&Value::Null, |facts| &facts.metadata),
            )
        }
        ControlEventKind::ActivityCompleted {
            activity_id,
            result,
            ..
        } => {
            let facts = activity_facts.get(activity_id.as_str());
            stream_source_from_activity(
                Some(activity_id.as_str()),
                facts.map(|facts| facts.activity_type.as_str()),
                facts.map(|facts| facts.task_queue.as_str()),
                if result
                    .metadata
                    .as_object()
                    .is_some_and(serde_json::Map::is_empty)
                    || result.metadata.is_null()
                {
                    facts.map_or(&result.metadata, |facts| &facts.metadata)
                } else {
                    &result.metadata
                },
            )
        }
        ControlEventKind::ActivityFailed {
            activity_id,
            failure,
            ..
        } => {
            let facts = activity_facts.get(activity_id.as_str());
            stream_source_from_activity(
                Some(activity_id.as_str()),
                facts.map(|facts| facts.activity_type.as_str()),
                facts.map(|facts| facts.task_queue.as_str()),
                if failure
                    .metadata
                    .as_object()
                    .is_some_and(serde_json::Map::is_empty)
                    || failure.metadata.is_null()
                {
                    facts.map_or(&failure.metadata, |facts| &facts.metadata)
                } else {
                    &failure.metadata
                },
            )
        }
        ControlEventKind::SignalReceived { .. }
        | ControlEventKind::TimerScheduled { .. }
        | ControlEventKind::TimerFired { .. }
        | ControlEventKind::VersionPinned { .. }
        | ControlEventKind::ArtifactAttached { .. }
        | ControlEventKind::EvidenceAttached { .. }
        | ControlEventKind::CostObserved { .. }
        | ControlEventKind::GateEvaluated { .. }
        | ControlEventKind::RecoveryStarted { .. } => QianjiControlRunStreamSource::System,
        ControlEventKind::WorkerHeartbeatObserved { heartbeat } => stream_source_from_activity(
            metadata_str(&heartbeat.metadata, "activity_id"),
            metadata_str(&heartbeat.metadata, "activity_type"),
            metadata_str(&heartbeat.metadata, "task_queue"),
            &heartbeat.metadata,
        ),
        _ => QianjiControlRunStreamSource::Bpmn,
    }
}

fn stream_source_from_activity(
    activity_id: Option<&str>,
    activity_type: Option<&str>,
    task_queue: Option<&str>,
    metadata: &Value,
) -> QianjiControlRunStreamSource {
    if activity_id.is_some_and(is_qianji_llm_activity_id)
        || activity_type.is_some_and(is_llm_route)
        || task_queue.is_some_and(is_llm_route)
        || metadata_describes_llm_activity(metadata)
    {
        return QianjiControlRunStreamSource::Llm;
    }
    if activity_type.is_some_and(is_agent_route)
        || task_queue.is_some_and(is_agent_route)
        || metadata_describes_agent_activity(metadata)
    {
        return QianjiControlRunStreamSource::Subagent;
    }
    QianjiControlRunStreamSource::Tool
}

fn is_qianji_llm_activity_id(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value.starts_with("bpmn-llm-")
}

fn is_llm_route(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    value == "llm" || value.starts_with("llm.")
}

fn is_agent_route(value: &str) -> bool {
    let value = value.to_ascii_lowercase();
    matches!(value.as_str(), "agent" | "subagent" | "pi-subagents")
        || value.starts_with("agent.")
        || value.starts_with("subagent.")
        || value.starts_with("pi-subagents.")
}

fn metadata_describes_llm_activity(value: &Value) -> bool {
    let Value::Object(map) = value else {
        return false;
    };
    if map.contains_key("qianji_llm_activity_request") {
        return true;
    }
    matches!(
        map.get("profile").and_then(Value::as_str),
        Some("bpmn-host-work-llm")
    ) || matches!(
        map.get("executor").and_then(Value::as_str),
        Some("openai-compatible-llm")
    ) || map
        .get("activity_type")
        .and_then(Value::as_str)
        .is_some_and(is_llm_route)
        || map
            .get("task_queue")
            .and_then(Value::as_str)
            .is_some_and(is_llm_route)
}

fn metadata_describes_agent_activity(value: &Value) -> bool {
    let Value::Object(map) = value else {
        return false;
    };
    map.get("activity_type")
        .and_then(Value::as_str)
        .is_some_and(is_agent_route)
        || map
            .get("task_queue")
            .and_then(Value::as_str)
            .is_some_and(is_agent_route)
        || map
            .get("profile")
            .and_then(Value::as_str)
            .is_some_and(is_agent_route)
}

fn metadata_str<'a>(metadata: &'a Value, key: &str) -> Option<&'a str> {
    metadata.get(key).and_then(Value::as_str)
}
