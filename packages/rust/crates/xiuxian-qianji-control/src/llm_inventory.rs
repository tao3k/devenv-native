//! Replay-derived LLM activity inventory projections.

use crate::{
    ActivityStatus, ActivityTask, ActivityType, ActivityView, ArtifactRef, IdempotencyKey,
    RecoveryItemScope, RunId, RunView, TaskQueue,
};

const LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY: &str = "qianji_llm_activity_request";

/// One replayed LLM activity row for operator and Agent inspection.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LlmActivityInventoryItem {
    /// Run or step scope.
    pub scope: RecoveryItemScope,
    /// Stable activity id within the owning run.
    pub activity_id: crate::ActivityId,
    /// Logical activity type.
    pub activity_type: ActivityType,
    /// Typed task queue or dispatch lane.
    pub task_queue: TaskQueue,
    /// Current replayed lifecycle status.
    pub status: ActivityStatus,
    /// Current replayed attempt number.
    pub attempt: u32,
    /// Last durable lifecycle timestamp for this activity.
    pub updated_at_ms: u64,
    /// Optional claim-check input reference from the scheduled task.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_ref: Option<ArtifactRef>,
    /// Idempotency key supplied by the scheduler.
    pub idempotency_key: IdempotencyKey,
    /// Model extracted from request audit metadata, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// LLM request audit metadata copied from scheduled task metadata.
    #[serde(default)]
    pub request_audit_metadata: serde_json::Value,
}

/// Replayed LLM activity lifecycle and audit-coverage counts.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct LlmActivityInventorySummary {
    /// Total replayed LLM activities.
    pub total: usize,
    /// Scheduled activities that have not started.
    pub scheduled: usize,
    /// Activities currently in flight.
    pub in_flight: usize,
    /// Activities that completed successfully.
    pub completed: usize,
    /// Activities that failed.
    pub failed: usize,
    /// LLM activities that do not carry request audit metadata.
    pub missing_request_audit: usize,
}

impl LlmActivityInventorySummary {
    fn record(&mut self, item: &LlmActivityInventoryItem) {
        self.total += 1;
        if item.request_audit_metadata.is_null() {
            self.missing_request_audit += 1;
        }
        match item.status {
            ActivityStatus::Scheduled => self.scheduled += 1,
            ActivityStatus::Started => self.in_flight += 1,
            ActivityStatus::Completed => self.completed += 1,
            ActivityStatus::Failed => self.failed += 1,
            ActivityStatus::Pending => {}
        }
    }
}

/// Read-only LLM activity inventory projection for one run.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LlmActivityInventoryProjection {
    /// Owning run id.
    pub run_id: RunId,
    /// Replayed LLM activity rows.
    #[serde(default)]
    pub items: Vec<LlmActivityInventoryItem>,
    /// Lifecycle and audit-coverage counts.
    #[serde(default)]
    pub summary: LlmActivityInventorySummary,
}

impl LlmActivityInventoryProjection {
    /// Projects LLM activity inventory rows from a replayed run view.
    #[must_use]
    pub fn from_view(view: &RunView) -> Self {
        let mut items = Vec::new();
        for activity in view.activities.values() {
            collect_llm_activity(&mut items, RecoveryItemScope::run(), activity);
        }
        for step in view.steps.values() {
            for activity in step.activities.values() {
                collect_llm_activity(
                    &mut items,
                    RecoveryItemScope::step(step.step_id.clone()),
                    activity,
                );
            }
        }
        let mut summary = LlmActivityInventorySummary::default();
        for item in &items {
            summary.record(item);
        }
        Self {
            run_id: view.run_id.clone(),
            items,
            summary,
        }
    }
}

fn collect_llm_activity(
    items: &mut Vec<LlmActivityInventoryItem>,
    scope: RecoveryItemScope,
    activity: &ActivityView,
) {
    let Some(task) = activity.task.as_ref() else {
        return;
    };
    if !is_llm_task(task) {
        return;
    }
    let request_audit_metadata = task
        .metadata
        .get(LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY)
        .cloned()
        .unwrap_or(serde_json::Value::Null);
    items.push(LlmActivityInventoryItem {
        scope,
        activity_id: task.activity_id.clone(),
        activity_type: task.activity_type.clone(),
        task_queue: task.task_queue.clone(),
        status: activity.status,
        attempt: activity.attempt,
        updated_at_ms: activity.updated_at_ms,
        input_ref: task.input_ref.clone(),
        idempotency_key: task.idempotency_key.clone(),
        model: request_audit_metadata
            .get("model")
            .and_then(serde_json::Value::as_str)
            .map(ToOwned::to_owned),
        request_audit_metadata,
    });
}

fn is_llm_task(task: &ActivityTask) -> bool {
    task.activity_type.as_str().starts_with("llm.")
        || task.task_queue.as_str().starts_with("llm.")
        || task
            .metadata
            .get(LLM_ACTIVITY_REQUEST_AUDIT_METADATA_KEY)
            .is_some()
}
