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
}

impl ActivityQueueProjection {
    /// Projects scheduled-but-not-started activity tasks from a replayed run
    /// view.
    #[must_use]
    pub fn from_view(view: &RunView, task_queue: Option<&TaskQueue>) -> Self {
        let mut items = Vec::new();
        for activity in view.activities.values() {
            push_if_selectable(&mut items, RecoveryItemScope::run(), activity, task_queue);
        }
        for step in view.steps.values() {
            for activity in step.activities.values() {
                push_if_selectable(
                    &mut items,
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
        }
    }
}

fn push_if_selectable(
    items: &mut Vec<ActivityQueueItem>,
    scope: RecoveryItemScope,
    activity: &ActivityView,
    task_queue: Option<&TaskQueue>,
) {
    if activity.status != ActivityStatus::Scheduled {
        return;
    }
    let Some(task) = &activity.task else {
        return;
    };
    if task_queue.is_some_and(|expected| &task.task_queue != expected) {
        return;
    }
    items.push(ActivityQueueItem {
        scope,
        activity: activity.clone(),
    });
}
