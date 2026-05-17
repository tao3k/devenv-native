//! In-memory control-plane stores for tests and first-slice integration.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::{
    ControlError, ControlEvent, ControlEventRecord, ControlLedger, ControlResult, HotStateStore,
    LeaseId, RunId, RunnableStep, StepId, StepLease, WorkerHeartbeat, WorkerId, WorkerRef,
};

/// In-memory append-only event ledger.
#[derive(Debug, Default)]
pub struct InMemoryControlLedger {
    state: Mutex<LedgerState>,
}

#[derive(Debug, Default)]
struct LedgerState {
    next_sequence: u64,
    records: Vec<ControlEventRecord>,
}

impl InMemoryControlLedger {
    /// Creates an empty in-memory ledger.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl ControlLedger for InMemoryControlLedger {
    fn append_event(&self, event: ControlEvent) -> ControlResult<ControlEventRecord> {
        let mut guard = lock(&self.state, "ledger_state")?;
        guard.next_sequence += 1;
        let record = ControlEventRecord {
            sequence: guard.next_sequence,
            event,
        };
        guard.records.push(record.clone());
        Ok(record)
    }

    fn load_events(&self, run_id: &RunId) -> ControlResult<Vec<ControlEventRecord>> {
        let guard = lock(&self.state, "ledger_state")?;
        Ok(guard
            .records
            .iter()
            .filter(|record| &record.event.run_id == run_id)
            .cloned()
            .collect())
    }
}

/// In-memory hot scheduling state.
#[derive(Debug, Default)]
pub struct InMemoryHotStateStore {
    state: Mutex<HotState>,
}

#[derive(Debug, Default)]
struct HotState {
    queue: Vec<RunnableStep>,
    leases: HashMap<StepKey, ActiveLease>,
    heartbeats: HashMap<WorkerId, WorkerHeartbeat>,
    next_lease_sequence: u64,
}

#[derive(Debug, Clone)]
struct ActiveLease {
    step: RunnableStep,
    lease: StepLease,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StepKey {
    run_id: RunId,
    step_id: StepId,
}

impl InMemoryHotStateStore {
    /// Creates an empty in-memory hot-state store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait::async_trait]
impl HotStateStore for InMemoryHotStateStore {
    async fn enqueue_step(&self, step: RunnableStep) -> ControlResult<()> {
        let mut guard = lock(&self.state, "hot_state")?;
        guard.queue.push(step);
        Ok(())
    }

    async fn acquire_lease(
        &self,
        worker: WorkerRef,
        now_ms: u64,
        lease_ttl_ms: u64,
    ) -> ControlResult<Option<StepLease>> {
        let mut guard = lock(&self.state, "hot_state")?;
        remove_expired_leases(&mut guard, now_ms);
        let Some(index) = next_runnable_index(&guard.queue, now_ms) else {
            return Ok(None);
        };
        let step = guard.queue.remove(index);
        guard.next_lease_sequence += 1;
        let lease = StepLease {
            lease_id: LeaseId::new(format!("lease-{}", guard.next_lease_sequence))?,
            run_id: step.run_id.clone(),
            step_id: step.step_id.clone(),
            worker_id: worker.worker_id,
            acquired_at_ms: now_ms,
            expires_at_ms: now_ms.saturating_add(lease_ttl_ms),
        };
        guard.leases.insert(
            step_key(&lease),
            ActiveLease {
                step,
                lease: lease.clone(),
            },
        );
        Ok(Some(lease))
    }

    async fn renew_lease(
        &self,
        lease: &StepLease,
        now_ms: u64,
        lease_ttl_ms: u64,
    ) -> ControlResult<bool> {
        let mut guard = lock(&self.state, "hot_state")?;
        let key = step_key(lease);
        let Some(active) = guard.leases.get_mut(&key) else {
            return Ok(false);
        };
        if active.lease.lease_id != lease.lease_id || active.lease.worker_id != lease.worker_id {
            return Err(ControlError::LeaseNotOwned {
                lease_id: lease.lease_id.clone(),
                worker_id: lease.worker_id.clone(),
            });
        }
        if !active.lease.is_active_at(now_ms) {
            if let Some(expired) = guard.leases.remove(&key) {
                guard.queue.push(expired.step);
            }
            return Ok(false);
        }
        active.lease.expires_at_ms = now_ms.saturating_add(lease_ttl_ms);
        Ok(true)
    }

    async fn release_lease(&self, lease: &StepLease) -> ControlResult<bool> {
        let mut guard = lock(&self.state, "hot_state")?;
        let key = step_key(lease);
        let Some(active) = guard.leases.get(&key) else {
            return Ok(false);
        };
        if active.lease.lease_id != lease.lease_id || active.lease.worker_id != lease.worker_id {
            return Err(ControlError::LeaseNotOwned {
                lease_id: lease.lease_id.clone(),
                worker_id: lease.worker_id.clone(),
            });
        }
        guard.leases.remove(&key);
        Ok(true)
    }

    async fn heartbeat(&self, heartbeat: WorkerHeartbeat) -> ControlResult<()> {
        let mut guard = lock(&self.state, "hot_state")?;
        guard
            .heartbeats
            .insert(heartbeat.worker_id.clone(), heartbeat);
        Ok(())
    }

    async fn load_heartbeat(&self, worker_id: &WorkerId) -> ControlResult<Option<WorkerHeartbeat>> {
        let guard = lock(&self.state, "hot_state")?;
        Ok(guard.heartbeats.get(worker_id).cloned())
    }
}

fn remove_expired_leases(state: &mut HotState, now_ms: u64) {
    let expired = state
        .leases
        .iter()
        .filter(|(_, active)| !active.lease.is_active_at(now_ms))
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    for key in expired {
        if let Some(active) = state.leases.remove(&key) {
            state.queue.push(active.step);
        }
    }
}

fn next_runnable_index(queue: &[RunnableStep], now_ms: u64) -> Option<usize> {
    queue
        .iter()
        .enumerate()
        .filter(|(_, step)| step.not_before_ms <= now_ms)
        .max_by_key(|(index, step)| (step.priority, std::cmp::Reverse(*index)))
        .map(|(index, _)| index)
}

fn step_key(lease: &StepLease) -> StepKey {
    StepKey {
        run_id: lease.run_id.clone(),
        step_id: lease.step_id.clone(),
    }
}

fn lock<'a, T>(
    mutex: &'a Mutex<T>,
    lock_name: &'static str,
) -> ControlResult<std::sync::MutexGuard<'a, T>> {
    mutex.lock().map_err(|error| ControlError::LockPoisoned {
        lock_name,
        message: error.to_string(),
    })
}
