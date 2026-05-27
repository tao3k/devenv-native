//! In-memory control-plane stores for tests and first-slice integration.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::{
    ActivityId, ActivityTaskLease, ControlError, ControlEvent, ControlEventRecord, ControlLedger,
    ControlResult, HotStateLeasedActivityTask, HotStateLeasedStep, HotStateSnapshot, HotStateStore,
    LeaseId, RunId, RunnableActivityTask, RunnableStep, StepId, StepLease, TaskQueue,
    WorkerHeartbeat, WorkerId, WorkerRef,
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
    activity_queue: Vec<RunnableActivityTask>,
    activity_leases: HashMap<ActivityTaskKey, ActiveActivityLease>,
    heartbeats: HashMap<WorkerId, WorkerHeartbeat>,
    next_lease_sequence: u64,
    next_activity_lease_sequence: u64,
}

#[derive(Debug, Clone)]
struct ActiveLease {
    step: RunnableStep,
    lease: StepLease,
}

#[derive(Debug, Clone)]
struct ActiveActivityLease {
    activity_task: RunnableActivityTask,
    lease: ActivityTaskLease,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct StepKey {
    run_id: RunId,
    step_id: StepId,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ActivityTaskKey {
    run: RunId,
    step: Option<StepId>,
    activity: ActivityId,
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

    async fn reclaim_expired_lease(&self, lease: &StepLease, now_ms: u64) -> ControlResult<bool> {
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
        if active.lease.is_active_at(now_ms) {
            return Ok(false);
        }
        let Some(expired) = guard.leases.remove(&key) else {
            return Ok(false);
        };
        guard.queue.push(expired.step);
        Ok(true)
    }

    async fn enqueue_activity_task(&self, task: RunnableActivityTask) -> ControlResult<()> {
        let mut guard = lock(&self.state, "hot_state")?;
        let key = activity_task_key_from_task(&task);
        if guard.activity_leases.contains_key(&key) {
            return Ok(());
        }
        guard
            .activity_queue
            .retain(|existing| activity_task_key_from_task(existing) != key);
        guard.activity_queue.push(task);
        Ok(())
    }

    async fn claim_activity_task(
        &self,
        worker: WorkerRef,
        task_queue: Option<&TaskQueue>,
        now_ms: u64,
        lease_ttl_ms: u64,
    ) -> ControlResult<Option<HotStateLeasedActivityTask>> {
        self.claim_matching_activity_task(worker, None, task_queue, now_ms, lease_ttl_ms)
    }

    async fn claim_activity_task_for_run(
        &self,
        worker: WorkerRef,
        run_id: &RunId,
        task_queue: Option<&TaskQueue>,
        now_ms: u64,
        lease_ttl_ms: u64,
    ) -> ControlResult<Option<HotStateLeasedActivityTask>> {
        self.claim_matching_activity_task(worker, Some(run_id), task_queue, now_ms, lease_ttl_ms)
    }

    async fn release_activity_task_lease(&self, lease: &ActivityTaskLease) -> ControlResult<bool> {
        let mut guard = lock(&self.state, "hot_state")?;
        let key = activity_task_key_from_lease(lease);
        let Some(active) = guard.activity_leases.get(&key) else {
            return Ok(false);
        };
        if active.lease.lease_id != lease.lease_id || active.lease.worker_id != lease.worker_id {
            return Err(ControlError::LeaseNotOwned {
                lease_id: lease.lease_id.clone(),
                worker_id: lease.worker_id.clone(),
            });
        }
        guard.activity_leases.remove(&key);
        Ok(true)
    }

    async fn reclaim_expired_activity_task_lease(
        &self,
        lease: &ActivityTaskLease,
        now_ms: u64,
    ) -> ControlResult<bool> {
        let mut guard = lock(&self.state, "hot_state")?;
        let key = activity_task_key_from_lease(lease);
        let Some(active) = guard.activity_leases.get(&key) else {
            return Ok(false);
        };
        if active.lease.lease_id != lease.lease_id || active.lease.worker_id != lease.worker_id {
            return Err(ControlError::LeaseNotOwned {
                lease_id: lease.lease_id.clone(),
                worker_id: lease.worker_id.clone(),
            });
        }
        if active.lease.is_active_at(now_ms) {
            return Ok(false);
        }
        let Some(expired) = guard.activity_leases.remove(&key) else {
            return Ok(false);
        };
        guard.activity_queue.push(expired.activity_task);
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

    async fn load_snapshot(&self, observed_at_ms: u64) -> ControlResult<HotStateSnapshot> {
        let guard = lock(&self.state, "hot_state")?;
        let mut snapshot = HotStateSnapshot::new(observed_at_ms);
        snapshot.pending_steps.extend(guard.queue.iter().cloned());
        snapshot.leased_steps = guard
            .leases
            .values()
            .map(|active| HotStateLeasedStep {
                step: active.step.clone(),
                lease: active.lease.clone(),
            })
            .collect();
        snapshot
            .pending_activity_tasks
            .extend(guard.activity_queue.iter().cloned());
        snapshot.leased_activity_tasks = guard
            .activity_leases
            .values()
            .map(|active| HotStateLeasedActivityTask {
                activity_task: active.activity_task.clone(),
                lease: active.lease.clone(),
            })
            .collect();
        snapshot.worker_heartbeats = guard.heartbeats.values().cloned().collect();
        snapshot
            .pending_steps
            .sort_by(|left, right| hot_step_order(left).cmp(&hot_step_order(right)));
        snapshot.leased_steps.sort_by(|left, right| {
            hot_step_order(&left.step)
                .cmp(&hot_step_order(&right.step))
                .then_with(|| {
                    left.lease
                        .lease_id
                        .as_str()
                        .cmp(right.lease.lease_id.as_str())
                })
        });
        snapshot
            .pending_activity_tasks
            .sort_by(hot_activity_task_order_cmp);
        snapshot.leased_activity_tasks.sort_by(|left, right| {
            hot_activity_task_order_cmp(&left.activity_task, &right.activity_task)
        });
        snapshot
            .worker_heartbeats
            .sort_by(|left, right| left.worker_id.as_str().cmp(right.worker_id.as_str()));
        Ok(snapshot)
    }
}

impl InMemoryHotStateStore {
    fn claim_matching_activity_task(
        &self,
        worker: WorkerRef,
        run_id: Option<&RunId>,
        task_queue: Option<&TaskQueue>,
        now_ms: u64,
        lease_ttl_ms: u64,
    ) -> ControlResult<Option<HotStateLeasedActivityTask>> {
        validate_positive_ttl("in_memory_activity_task_lease_ttl", lease_ttl_ms)?;
        let mut guard = lock(&self.state, "hot_state")?;
        remove_expired_activity_leases(&mut guard, now_ms);
        let Some(index) =
            next_runnable_activity_task_index(&guard.activity_queue, run_id, task_queue, now_ms)
        else {
            return Ok(None);
        };
        let activity_task = guard.activity_queue.remove(index);
        guard.next_activity_lease_sequence += 1;
        let lease = ActivityTaskLease {
            lease_id: LeaseId::new(format!(
                "activity-lease-{}",
                guard.next_activity_lease_sequence
            ))?,
            run_id: activity_task.task.run_id.clone(),
            step_id: activity_task.task.step_id.clone(),
            activity_id: activity_task.task.activity_id.clone(),
            worker_id: worker.worker_id,
            acquired_at_ms: now_ms,
            expires_at_ms: now_ms.saturating_add(lease_ttl_ms),
        };
        let leased = HotStateLeasedActivityTask {
            activity_task,
            lease,
        };
        guard.activity_leases.insert(
            activity_task_key_from_lease(&leased.lease),
            ActiveActivityLease {
                activity_task: leased.activity_task.clone(),
                lease: leased.lease.clone(),
            },
        );
        Ok(Some(leased))
    }
}

fn validate_positive_ttl(operation: &'static str, ttl_ms: u64) -> ControlResult<()> {
    if ttl_ms == 0 {
        return Err(ControlError::Storage {
            operation,
            message: "ttl_ms must be greater than zero".to_owned(),
        });
    }
    Ok(())
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

fn remove_expired_activity_leases(state: &mut HotState, now_ms: u64) {
    let expired = state
        .activity_leases
        .iter()
        .filter(|(_, active)| !active.lease.is_active_at(now_ms))
        .map(|(key, _)| key.clone())
        .collect::<Vec<_>>();
    for key in expired {
        if let Some(active) = state.activity_leases.remove(&key) {
            state.activity_queue.push(active.activity_task);
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

fn next_runnable_activity_task_index(
    queue: &[RunnableActivityTask],
    run_id: Option<&RunId>,
    task_queue: Option<&TaskQueue>,
    now_ms: u64,
) -> Option<usize> {
    queue
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.not_before_ms <= now_ms)
        .filter(|(_, entry)| run_id.is_none_or(|run_id| &entry.task.run_id == run_id))
        .filter(|(_, entry)| task_queue.is_none_or(|queue| &entry.task.task_queue == queue))
        .max_by_key(|(index, entry)| (entry.priority, std::cmp::Reverse(*index)))
        .map(|(index, _)| index)
}

fn hot_step_order(step: &RunnableStep) -> (&str, &str) {
    (step.run_id.as_str(), step.step_id.as_str())
}

fn hot_activity_task_order_cmp(
    left: &RunnableActivityTask,
    right: &RunnableActivityTask,
) -> std::cmp::Ordering {
    activity_task_order_tuple(left).cmp(&activity_task_order_tuple(right))
}

fn activity_task_order_tuple(entry: &RunnableActivityTask) -> (&str, &str, &str) {
    (
        entry.task.run_id.as_str(),
        entry.task.step_id.as_ref().map_or("", StepId::as_str),
        entry.task.activity_id.as_str(),
    )
}

fn step_key(lease: &StepLease) -> StepKey {
    StepKey {
        run_id: lease.run_id.clone(),
        step_id: lease.step_id.clone(),
    }
}

fn activity_task_key_from_lease(lease: &ActivityTaskLease) -> ActivityTaskKey {
    ActivityTaskKey {
        run: lease.run_id.clone(),
        step: lease.step_id.clone(),
        activity: lease.activity_id.clone(),
    }
}

fn activity_task_key_from_task(task: &RunnableActivityTask) -> ActivityTaskKey {
    ActivityTaskKey {
        run: task.task.run_id.clone(),
        step: task.task.step_id.clone(),
        activity: task.task.activity_id.clone(),
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
