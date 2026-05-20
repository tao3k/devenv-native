//! Run-level operator summary projection.

use crate::{
    ActivityQueueProjection, ActivityQueueSummary, ControlEventRecord, ControlResult,
    CostInventoryProjection, CostInventorySummary, RunId, RunRecoveryPlanSummary,
    RunRecoverySnapshot, RunStatus, RunView, SignalInventoryProjection, SignalInventorySummary,
    TimerInventoryProjection, TimerInventorySummary, replay_run_view,
};

/// Compact run management view assembled from durable history projections.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct RunOperatorSummary {
    /// Run id.
    pub run_id: RunId,
    /// Observation time used for time-dependent recovery counters.
    pub observed_at_ms: u64,
    /// Number of durable events loaded for this run.
    pub event_count: usize,
    /// Current run lifecycle status.
    pub status: RunStatus,
    /// Last replayed run update timestamp.
    pub updated_at_ms: u64,
    /// Number of replayed steps.
    pub steps: usize,
    /// Number of steps with an active lease.
    pub active_leases: usize,
    /// Activity lifecycle counters.
    pub activities: ActivityQueueSummary,
    /// Durable timer lifecycle counters.
    pub timers: TimerInventorySummary,
    /// External signal counters.
    pub signals: SignalInventorySummary,
    /// Cost and token counters.
    pub costs: CostInventorySummary,
    /// Recovery action counters.
    pub recovery: RunRecoveryPlanSummary,
}

impl RunOperatorSummary {
    /// Builds a compact operator summary from one loaded event stream.
    ///
    /// # Errors
    ///
    /// Returns a control error when the event stream cannot be replayed or the
    /// recovery view cannot be projected.
    pub fn from_records(
        run_id: RunId,
        records: &[ControlEventRecord],
        observed_at_ms: u64,
    ) -> ControlResult<Self> {
        let event_count = records.len();
        let view = replay_run_view(records.to_owned())?;
        Self::from_parts(run_id, event_count, records, &view, observed_at_ms)
    }

    fn from_parts(
        run_id: RunId,
        event_count: usize,
        records: &[ControlEventRecord],
        view: &RunView,
        observed_at_ms: u64,
    ) -> ControlResult<Self> {
        let activities = ActivityQueueProjection::from_view(view, None).summary;
        let timers = TimerInventoryProjection::from_view(view).summary;
        let signals = SignalInventoryProjection::from_records(run_id.clone(), records).summary;
        let costs = CostInventoryProjection::from_records(run_id.clone(), records).summary;
        let recovery = RunRecoverySnapshot::from_view(view.recovery_view(observed_at_ms)?).summary;
        let active_leases = view
            .steps
            .values()
            .filter(|step| step.active_lease.is_some())
            .count();

        Ok(Self {
            run_id,
            observed_at_ms,
            event_count,
            status: view.status,
            updated_at_ms: view.updated_at_ms,
            steps: view.steps.len(),
            active_leases,
            activities,
            timers,
            signals,
            costs,
            recovery,
        })
    }
}
