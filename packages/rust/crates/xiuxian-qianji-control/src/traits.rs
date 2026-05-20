//! Storage and gate traits for the control plane.

use crate::{
    ActivityQueueProjection, ControlEvent, ControlEventRecord, ControlResult,
    CostInventoryProjection, GateResult, HotStateSnapshot, RunId, RunOperatorSummary,
    RunRecoveryPlan, RunRecoverySnapshot, RunView, RunnableStep, SignalInventoryProjection,
    StepLease, StepView, TaskQueue, TimerInventoryProjection, WorkerHeartbeat, WorkerId, WorkerRef,
};

/// Durable append-only event ledger.
pub trait ControlLedger: Send + Sync {
    /// Appends one event and returns the stored record.
    ///
    /// # Errors
    ///
    /// Returns a store-specific control error when the event cannot be
    /// persisted.
    fn append_event(&self, event: ControlEvent) -> ControlResult<ControlEventRecord>;

    /// Loads all event records for one run.
    ///
    /// # Errors
    ///
    /// Returns a store-specific control error when records cannot be loaded.
    fn load_events(&self, run_id: &RunId) -> ControlResult<Vec<ControlEventRecord>>;

    /// Loads and replays one run view.
    ///
    /// # Errors
    ///
    /// Returns a control error when records cannot be loaded or replayed.
    fn load_run_view(&self, run_id: &RunId) -> ControlResult<RunView> {
        crate::replay_run_view(self.load_events(run_id)?)
    }

    /// Loads and projects one run recovery plan from durable history.
    ///
    /// # Errors
    ///
    /// Returns a control error when records cannot be loaded, replayed, or
    /// projected through recovery retry-policy evaluation.
    fn load_recovery_plan(&self, run_id: &RunId, now_ms: u64) -> ControlResult<RunRecoveryPlan> {
        Ok(self
            .load_run_view(run_id)?
            .recovery_view(now_ms)?
            .recovery_plan())
    }

    /// Loads one run recovery snapshot from durable history.
    ///
    /// # Errors
    ///
    /// Returns a control error when records cannot be loaded, replayed, or
    /// projected through recovery retry-policy evaluation.
    fn load_recovery_snapshot(
        &self,
        run_id: &RunId,
        now_ms: u64,
    ) -> ControlResult<RunRecoverySnapshot> {
        Ok(RunRecoverySnapshot::from_view(
            self.load_run_view(run_id)?.recovery_view(now_ms)?,
        ))
    }

    /// Loads a read-only scheduled activity queue projection from durable
    /// history.
    ///
    /// # Errors
    ///
    /// Returns a control error when records cannot be loaded or replayed.
    fn load_activity_queue_projection(
        &self,
        run_id: &RunId,
        task_queue: Option<&TaskQueue>,
    ) -> ControlResult<ActivityQueueProjection> {
        Ok(ActivityQueueProjection::from_view(
            &self.load_run_view(run_id)?,
            task_queue,
        ))
    }

    /// Loads a read-only durable timer inventory projection from durable
    /// history.
    ///
    /// # Errors
    ///
    /// Returns a control error when records cannot be loaded or replayed.
    fn load_timer_inventory_projection(
        &self,
        run_id: &RunId,
    ) -> ControlResult<TimerInventoryProjection> {
        Ok(TimerInventoryProjection::from_view(
            &self.load_run_view(run_id)?,
        ))
    }

    /// Loads a read-only durable signal inventory projection from durable
    /// history.
    ///
    /// # Errors
    ///
    /// Returns a control error when records cannot be loaded.
    fn load_signal_inventory_projection(
        &self,
        run_id: &RunId,
    ) -> ControlResult<SignalInventoryProjection> {
        Ok(SignalInventoryProjection::from_records(
            run_id.clone(),
            &self.load_events(run_id)?,
        ))
    }

    /// Loads a read-only durable cost inventory projection from durable
    /// history.
    ///
    /// # Errors
    ///
    /// Returns a control error when records cannot be loaded.
    fn load_cost_inventory_projection(
        &self,
        run_id: &RunId,
    ) -> ControlResult<CostInventoryProjection> {
        Ok(CostInventoryProjection::from_records(
            run_id.clone(),
            &self.load_events(run_id)?,
        ))
    }

    /// Loads a compact operator summary from durable history.
    ///
    /// # Errors
    ///
    /// Returns a control error when records cannot be loaded, replayed, or
    /// projected through recovery retry-policy evaluation.
    fn load_operator_summary(
        &self,
        run_id: &RunId,
        observed_at_ms: u64,
    ) -> ControlResult<RunOperatorSummary> {
        RunOperatorSummary::from_records(run_id.clone(), &self.load_events(run_id)?, observed_at_ms)
    }
}

/// Hot scheduling state for queues, leases, and heartbeats.
#[async_trait::async_trait]
pub trait HotStateStore: Send + Sync {
    /// Enqueues one runnable step.
    ///
    /// # Errors
    ///
    /// Returns a store-specific control error when enqueue fails.
    async fn enqueue_step(&self, step: RunnableStep) -> ControlResult<()>;

    /// Acquires one runnable step lease for a worker.
    ///
    /// # Errors
    ///
    /// Returns a store-specific control error when acquisition fails.
    async fn acquire_lease(
        &self,
        worker: WorkerRef,
        now_ms: u64,
        lease_ttl_ms: u64,
    ) -> ControlResult<Option<StepLease>>;

    /// Renews a lease if the caller still owns it.
    ///
    /// # Errors
    ///
    /// Returns a store-specific control error when renewal fails.
    async fn renew_lease(
        &self,
        lease: &StepLease,
        now_ms: u64,
        lease_ttl_ms: u64,
    ) -> ControlResult<bool>;

    /// Releases a lease if the caller still owns it.
    ///
    /// # Errors
    ///
    /// Returns a store-specific control error when release fails.
    async fn release_lease(&self, lease: &StepLease) -> ControlResult<bool>;

    /// Reclaims an expired lease and makes the step runnable again.
    ///
    /// # Errors
    ///
    /// Returns a store-specific control error when reclaim fails.
    async fn reclaim_expired_lease(&self, lease: &StepLease, now_ms: u64) -> ControlResult<bool>;

    /// Records one worker heartbeat.
    ///
    /// # Errors
    ///
    /// Returns a store-specific control error when heartbeat fails.
    async fn heartbeat(&self, heartbeat: WorkerHeartbeat) -> ControlResult<()>;

    /// Loads a worker heartbeat.
    ///
    /// # Errors
    ///
    /// Returns a store-specific control error when load fails.
    async fn load_heartbeat(&self, worker_id: &WorkerId) -> ControlResult<Option<WorkerHeartbeat>>;

    /// Loads a read-only snapshot of hot queue, lease, and heartbeat state.
    ///
    /// # Errors
    ///
    /// Returns a store-specific control error when snapshot loading fails.
    async fn load_snapshot(&self, observed_at_ms: u64) -> ControlResult<HotStateSnapshot>;
}

/// Deterministic evidence gate.
pub trait EvidenceGate: Send + Sync {
    /// Evaluates one step view.
    fn evaluate(&self, step: &StepView) -> GateResult;
}
