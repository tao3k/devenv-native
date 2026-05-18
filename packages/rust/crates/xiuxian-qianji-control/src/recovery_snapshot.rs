//! Read-only recovery management snapshots.

use crate::{RunId, RunRecoveryPlan, RunRecoveryPlanSummary, RunRecoveryView};

/// Read-only recovery state package for management surfaces.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RunRecoverySnapshot {
    /// Run id.
    pub run_id: RunId,
    /// Observation time used to classify timers and leases.
    pub observed_at_ms: u64,
    /// Replay-derived recovery facts.
    pub view: RunRecoveryView,
    /// Ordered declarative recovery actions.
    pub plan: RunRecoveryPlan,
    /// Compact counters over the ordered recovery plan.
    pub summary: RunRecoveryPlanSummary,
}

impl RunRecoverySnapshot {
    /// Builds a snapshot from a replay-derived recovery view.
    #[must_use]
    pub fn from_view(view: RunRecoveryView) -> Self {
        let run_id = view.run_id.clone();
        let observed_at_ms = view.now_ms;
        let plan = view.recovery_plan();
        let summary = plan.summary();
        Self {
            run_id,
            observed_at_ms,
            view,
            plan,
            summary,
        }
    }
}
