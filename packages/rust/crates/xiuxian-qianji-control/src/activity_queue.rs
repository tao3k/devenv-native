//! Replay-derived activity queue projections.

use crate::{
    ActivityRetryPolicy, ActivityStatus, ActivityType, ActivityView, ArtifactRef, ControlLedger,
    ControlResult, HotStateStore, IdempotencyKey, RecoveryItemScope, RunId, RunView,
    RunnableActivityTask, StepId, TaskQueue,
};

/// Worker-facing activity task envelope derived from durable replay.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerActivityTask {
    /// Owning run id.
    pub run_id: RunId,
    /// Owning step id when the activity is step-scoped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub step_id: Option<StepId>,
    /// Stable activity id within the run.
    pub activity_id: crate::ActivityId,
    /// Logical activity type.
    pub activity_type: ActivityType,
    /// Typed task queue or dispatch lane.
    pub task_queue: TaskQueue,
    /// Next worker attempt that should be recorded when this task starts.
    pub next_attempt: u32,
    /// Timestamp of the durable schedule event.
    pub scheduled_at_ms: u64,
    /// Optional claim-check input reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input_ref: Option<ArtifactRef>,
    /// Idempotency key supplied by the scheduler.
    pub idempotency_key: IdempotencyKey,
    /// Optional retry policy.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retry_policy: Option<ActivityRetryPolicy>,
    /// Optional execution timeout.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    /// Extension metadata from the scheduled activity task.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

/// One worker-visible activity queue item derived from durable replay.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityQueueItem {
    /// Run or step scope.
    pub scope: RecoveryItemScope,
    /// Replayed scheduled activity state.
    pub activity: ActivityView,
}

impl ActivityQueueItem {
    /// Converts the replay item into a worker-facing activity task envelope.
    #[must_use]
    pub fn worker_task(&self, run_id: &RunId) -> Option<WorkerActivityTask> {
        let task = self.activity.task.as_ref()?;
        Some(WorkerActivityTask {
            run_id: run_id.clone(),
            step_id: step_id_for_scope(&self.scope),
            activity_id: task.activity_id.clone(),
            activity_type: task.activity_type.clone(),
            task_queue: task.task_queue.clone(),
            next_attempt: self.activity.attempt.saturating_add(1).max(1),
            scheduled_at_ms: self.activity.updated_at_ms,
            input_ref: task.input_ref.clone(),
            idempotency_key: task.idempotency_key.clone(),
            retry_policy: task.retry_policy.clone(),
            timeout_ms: task.timeout_ms,
            metadata: task.metadata.clone(),
        })
    }
}

/// Replayed activity lifecycle counts for queue operators.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct ActivityQueueSummary {
    /// Total activities included by the projection filter.
    pub total: usize,
    /// Scheduled activities that have not started.
    pub scheduled: usize,
    /// Activities currently in flight.
    pub in_flight: usize,
    /// Activities that completed successfully.
    pub completed: usize,
    /// Activities that failed.
    pub failed: usize,
}

impl ActivityQueueSummary {
    fn record(&mut self, status: ActivityStatus) {
        self.total += 1;
        match status {
            ActivityStatus::Scheduled => self.scheduled += 1,
            ActivityStatus::Started => self.in_flight += 1,
            ActivityStatus::Completed => self.completed += 1,
            ActivityStatus::Failed => self.failed += 1,
            ActivityStatus::Pending => {}
        }
    }
}

/// Read-only scheduled activity projection for worker queue inspection.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityQueueProjection {
    /// Owning run id.
    pub run_id: RunId,
    /// Optional task queue filter used by the projection.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<TaskQueue>,
    /// Scheduled activities that have not started.
    #[serde(default)]
    pub items: Vec<ActivityQueueItem>,
    /// Worker-facing task envelopes derived from `items`.
    #[serde(default)]
    pub worker_tasks: Vec<WorkerActivityTask>,
    /// Lifecycle counts for activities included by the projection filter.
    #[serde(default)]
    pub summary: ActivityQueueSummary,
}

/// Request for mirroring replay-derived worker activity tasks into hot state.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct WorkerActivityHotStateMirrorRequest {
    /// Owning run id.
    pub run_id: RunId,
    /// Optional queue filter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<TaskQueue>,
    /// Hot-state priority assigned to mirrored tasks.
    #[serde(default)]
    pub priority: i64,
    /// Earliest hot-state claim time for mirrored tasks.
    #[serde(default)]
    pub not_before_ms: u64,
    /// Extension metadata attached to the hot-state mirror entry.
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl WorkerActivityHotStateMirrorRequest {
    /// Creates a mirror request for one run.
    #[must_use]
    pub fn new(run_id: RunId) -> Self {
        Self {
            run_id,
            task_queue: None,
            priority: 0,
            not_before_ms: 0,
            metadata: serde_json::Value::Null,
        }
    }

    /// Filters mirrored tasks to one task queue.
    #[must_use]
    pub fn with_task_queue(mut self, task_queue: TaskQueue) -> Self {
        self.task_queue = Some(task_queue);
        self
    }

    /// Sets hot-state priority for mirrored tasks.
    #[must_use]
    pub const fn with_priority(mut self, priority: i64) -> Self {
        self.priority = priority;
        self
    }

    /// Sets the earliest hot-state claim time for mirrored tasks.
    #[must_use]
    pub const fn with_not_before_ms(mut self, not_before_ms: u64) -> Self {
        self.not_before_ms = not_before_ms;
        self
    }

    /// Sets hot-state mirror metadata.
    #[must_use]
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Result of one worker activity hot-state mirror pass.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct WorkerActivityHotStateMirrorOutcome {
    /// Owning run id.
    pub run_id: RunId,
    /// Optional queue filter used by the mirror pass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task_queue: Option<TaskQueue>,
    /// Number of replay-derived tasks mirrored into hot state.
    pub mirrored_count: usize,
}

/// Mirrors pending replay-derived worker activity tasks into hot state.
///
/// The control ledger remains the durable source of truth. This helper only
/// projects currently scheduled worker tasks into a hot-state polling surface.
///
/// # Errors
///
/// Returns a control error when replay projection or hot-state enqueue fails.
pub async fn mirror_worker_activity_tasks_to_hot_state<L, H>(
    ledger: &L,
    hot_state: &H,
    request: WorkerActivityHotStateMirrorRequest,
) -> ControlResult<WorkerActivityHotStateMirrorOutcome>
where
    L: ControlLedger + ?Sized,
    H: HotStateStore + ?Sized,
{
    let worker_tasks =
        ledger.load_worker_activity_tasks(&request.run_id, request.task_queue.as_ref())?;
    let mirrored_count = worker_tasks.len();
    for task in worker_tasks {
        hot_state
            .enqueue_activity_task(RunnableActivityTask {
                task,
                priority: request.priority,
                not_before_ms: request.not_before_ms,
                metadata: request.metadata.clone(),
            })
            .await?;
    }
    Ok(WorkerActivityHotStateMirrorOutcome {
        run_id: request.run_id,
        task_queue: request.task_queue,
        mirrored_count,
    })
}

impl ActivityQueueProjection {
    /// Projects scheduled-but-not-started activity tasks from a replayed run
    /// view.
    #[must_use]
    pub fn from_view(view: &RunView, task_queue: Option<&TaskQueue>) -> Self {
        let mut items = Vec::new();
        let mut summary = ActivityQueueSummary::default();
        for activity in view.activities.values() {
            collect_activity(
                &mut items,
                &mut summary,
                RecoveryItemScope::run(),
                activity,
                task_queue,
            );
        }
        for step in view.steps.values() {
            for activity in step.activities.values() {
                collect_activity(
                    &mut items,
                    &mut summary,
                    RecoveryItemScope::step(step.step_id.clone()),
                    activity,
                    task_queue,
                );
            }
        }
        let worker_tasks = items
            .iter()
            .filter_map(|item| item.worker_task(&view.run_id))
            .collect();
        Self {
            run_id: view.run_id.clone(),
            task_queue: task_queue.cloned(),
            items,
            worker_tasks,
            summary,
        }
    }
}

fn collect_activity(
    items: &mut Vec<ActivityQueueItem>,
    summary: &mut ActivityQueueSummary,
    scope: RecoveryItemScope,
    activity: &ActivityView,
    task_queue: Option<&TaskQueue>,
) {
    if !matches_task_queue(activity, task_queue) {
        return;
    }
    summary.record(activity.status);
    if activity.status != ActivityStatus::Scheduled {
        return;
    }
    items.push(ActivityQueueItem {
        scope,
        activity: activity.clone(),
    });
}

fn matches_task_queue(activity: &ActivityView, task_queue: Option<&TaskQueue>) -> bool {
    let Some(expected) = task_queue else {
        return true;
    };
    activity
        .task
        .as_ref()
        .is_some_and(|task| &task.task_queue == expected)
}

fn step_id_for_scope(scope: &RecoveryItemScope) -> Option<StepId> {
    match scope {
        RecoveryItemScope::Run => None,
        RecoveryItemScope::Step { step_id } => Some(step_id.clone()),
    }
}
