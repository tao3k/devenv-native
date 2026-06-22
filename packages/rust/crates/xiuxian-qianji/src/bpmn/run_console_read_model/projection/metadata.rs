//! Structured metadata helpers for run-console projections.

use serde_json::{Map, Value, json};
use std::collections::BTreeMap;
use xiuxian_qianji_control::{ControlEventKind, ControlEventRecord};

#[derive(Debug, Clone)]
pub(super) struct ActivityStreamFacts {
    pub(super) activity_type: String,
    pub(super) task_queue: String,
    pub(super) metadata: Value,
}

pub(super) fn collect_activity_stream_facts(
    events: &[ControlEventRecord],
) -> BTreeMap<String, ActivityStreamFacts> {
    let mut facts = BTreeMap::new();
    for record in events {
        let ControlEventKind::ActivityScheduled { task } = &record.event.kind else {
            continue;
        };
        facts.insert(
            task.activity_id.as_str().to_owned(),
            ActivityStreamFacts {
                activity_type: task.activity_type.as_str().to_owned(),
                task_queue: task.task_queue.as_str().to_owned(),
                metadata: scheduled_activity_metadata(
                    task.metadata.clone(),
                    task.activity_id.as_str(),
                    task.activity_type.as_str(),
                    task.task_queue.as_str(),
                ),
            },
        );
    }
    facts
}

pub(super) fn metadata_from_event_kind(kind: &ControlEventKind) -> Value {
    match kind {
        ControlEventKind::RunCreated { metadata, .. }
        | ControlEventKind::ToolCallRecorded { metadata, .. } => metadata.clone(),
        ControlEventKind::AgentProposalRecorded { proposal } => {
            let mut metadata = metadata_object(proposal.metadata.clone());
            metadata.insert(
                "proposal_id".to_owned(),
                json!(proposal.proposal_id.as_str()),
            );
            metadata.insert("step_id".to_owned(), json!(proposal.step_id.as_str()));
            metadata.insert("token_id".to_owned(), json!(proposal.token_id.as_str()));
            metadata.insert(
                "proposed_action".to_owned(),
                json!(proposal.proposed_action.as_str()),
            );
            if let Some(tool_name) = &proposal.tool_name {
                metadata.insert("tool_name".to_owned(), json!(tool_name));
            }
            if let Some(confidence_millis) = proposal.confidence_millis {
                metadata.insert("confidence_millis".to_owned(), json!(confidence_millis));
            }
            Value::Object(metadata)
        }
        ControlEventKind::AgentDecisionRecorded { decision } => {
            let mut metadata = metadata_object(decision.metadata.clone());
            metadata.insert(
                "decision_id".to_owned(),
                json!(decision.decision_id.as_str()),
            );
            metadata.insert(
                "proposal_id".to_owned(),
                json!(decision.proposal_id.as_str()),
            );
            metadata.insert("outcome".to_owned(), json!(decision.outcome));
            metadata.insert(
                "reason_code".to_owned(),
                json!(decision.reason_code.as_str()),
            );
            if let Some(activity_id) = &decision.scheduled_activity_id {
                metadata.insert(
                    "scheduled_activity_id".to_owned(),
                    json!(activity_id.as_str()),
                );
            }
            if let Some(checkpoint_seq) = decision.checkpoint_seq {
                metadata.insert("checkpoint_seq".to_owned(), json!(checkpoint_seq));
            }
            Value::Object(metadata)
        }
        ControlEventKind::ActivityScheduled { task } => scheduled_activity_metadata(
            task.metadata.clone(),
            task.activity_id.as_str(),
            task.activity_type.as_str(),
            task.task_queue.as_str(),
        ),
        ControlEventKind::ActivityCompleted { result, .. } => result.metadata.clone(),
        ControlEventKind::ActivityFailed { failure, .. } => failure.metadata.clone(),
        ControlEventKind::WorkerHeartbeatObserved { heartbeat } => heartbeat.metadata.clone(),
        _ => Value::Object(Map::new()),
    }
}

pub(super) fn metadata_from_event_kind_with_facts(
    kind: &ControlEventKind,
    activity_facts: &BTreeMap<String, ActivityStreamFacts>,
) -> Value {
    match kind {
        ControlEventKind::ActivityStarted { activity_id, .. } => {
            activity_facts.get(activity_id.as_str()).map_or_else(
                || metadata_from_event_kind(kind),
                |facts| facts.metadata.clone(),
            )
        }
        ControlEventKind::ActivityCompleted {
            activity_id,
            result,
            ..
        } if metadata_is_empty(&result.metadata) => {
            activity_facts.get(activity_id.as_str()).map_or_else(
                || metadata_from_event_kind(kind),
                |facts| facts.metadata.clone(),
            )
        }
        ControlEventKind::ActivityFailed {
            activity_id,
            failure,
            ..
        } if metadata_is_empty(&failure.metadata) => {
            activity_facts.get(activity_id.as_str()).map_or_else(
                || metadata_from_event_kind(kind),
                |facts| facts.metadata.clone(),
            )
        }
        _ => metadata_from_event_kind(kind),
    }
}

fn scheduled_activity_metadata(
    metadata: Value,
    activity_id: &str,
    activity_type: &str,
    task_queue: &str,
) -> Value {
    let mut metadata = metadata_object(metadata);
    metadata.insert("activity_id".to_owned(), json!(activity_id));
    metadata.insert("activity_type".to_owned(), json!(activity_type));
    metadata.insert("task_queue".to_owned(), json!(task_queue));
    Value::Object(metadata)
}

fn metadata_is_empty(metadata: &Value) -> bool {
    matches!(metadata, Value::Null) || metadata.as_object().is_some_and(Map::is_empty)
}

fn metadata_object(metadata: Value) -> Map<String, Value> {
    match metadata {
        Value::Object(map) => map,
        _ => Map::new(),
    }
}
