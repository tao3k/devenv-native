//! Storage and gate traits for the control plane.

use crate::{
    ControlEvent, ControlEventRecord, ControlResult, GateResult, RunId, RunRecoveryPlan,
    RunRecoverySnapshot, RunView, RunnableStep, StepLease, StepView, WorkerHeartbeat, WorkerId,
    WorkerRef,
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
}

/// Deterministic evidence gate.
pub trait EvidenceGate: Send + Sync {
    /// Evaluates one step view.
    fn evaluate(&self, step: &StepView) -> GateResult;
}
