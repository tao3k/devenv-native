//! Replay-derived durable timer inventory projections.

use crate::{RecoveryItemScope, RunId, RunView, TimerStatus, TimerView};

/// One timer inventory item derived from durable replay.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TimerInventoryItem {
    /// Run or step scope.
    pub scope: RecoveryItemScope,
    /// Replayed timer state.
    pub timer: TimerView,
}

/// Replayed timer lifecycle counts for wait-state operators.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct TimerInventorySummary {
    /// Total timers included by the projection.
    pub total: usize,
    /// Timers without a scheduled record.
    pub pending: usize,
    /// Timers waiting to fire.
    pub scheduled: usize,
    /// Timers that already fired.
    pub fired: usize,
}

impl TimerInventorySummary {
    fn record(&mut self, status: TimerStatus) {
        self.total += 1;
        match status {
            TimerStatus::Pending => self.pending += 1,
            TimerStatus::Scheduled => self.scheduled += 1,
            TimerStatus::Fired => self.fired += 1,
        }
    }
}

/// Read-only durable timer projection for run and step wait-state inspection.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct TimerInventoryProjection {
    /// Owning run id.
    pub run_id: RunId,
    /// Timers replayed from run and step scopes.
    #[serde(default)]
    pub items: Vec<TimerInventoryItem>,
    /// Lifecycle counts for all included timers.
    #[serde(default)]
    pub summary: TimerInventorySummary,
}

impl TimerInventoryProjection {
    /// Projects durable timers from a replayed run view.
    #[must_use]
    pub fn from_view(view: &RunView) -> Self {
        let mut items = Vec::new();
        let mut summary = TimerInventorySummary::default();

        for timer in view.timers.values() {
            collect_timer(&mut items, &mut summary, RecoveryItemScope::run(), timer);
        }
        for step in view.steps.values() {
            for timer in step.timers.values() {
                collect_timer(
                    &mut items,
                    &mut summary,
                    RecoveryItemScope::step(step.step_id.clone()),
                    timer,
                );
            }
        }

        Self {
            run_id: view.run_id.clone(),
            items,
            summary,
        }
    }
}

fn collect_timer(
    items: &mut Vec<TimerInventoryItem>,
    summary: &mut TimerInventorySummary,
    scope: RecoveryItemScope,
    timer: &TimerView,
) {
    summary.record(timer.status);
    items.push(TimerInventoryItem {
        scope,
        timer: timer.clone(),
    });
}
