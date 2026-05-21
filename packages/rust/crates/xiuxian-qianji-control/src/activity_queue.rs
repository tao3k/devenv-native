//! Replay-derived activity queue projections.

use crate::{ActivityStatus, ActivityView, RecoveryItemScope, RunId, RunView, TaskQueue};

/// One worker-visible activity queue item derived from durable replay.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ActivityQueueItem {
    /// Run or step scope.
    pub scope: RecoveryItemScope,
    /// Replayed scheduled activity state.
    pub activity: ActivityView,
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
    /// Lifecycle counts for activities included by the projection filter.
    #[serde(default)]
    pub summary: ActivityQueueSummary,
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
        Self {
            run_id: view.run_id.clone(),
            task_queue: task_queue.cloned(),
            items,
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
