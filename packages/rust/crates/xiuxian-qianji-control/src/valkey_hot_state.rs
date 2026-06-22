//! Valkey-backed hot scheduling state.
//!
//! Qianji owns the workflow-domain payloads and lease semantics. The Valkey
//! command surface is delegated to `xiuxian-db-store` structured queue
//! primitives so this crate does not parse composite keys or hand-roll storage
//! scripts.

use std::sync::atomic::{AtomicU64, Ordering};

use crate::{
    ActivityId, ActivityTaskLease, ControlError, ControlResult, HotStateLeasedActivityTask,
    HotStateLeasedStep, HotStateSnapshot, HotStateStore, LeaseId, RunId, RunnableActivityTask,
    RunnableStep, StepId, StepLease, TaskQueue, WorkerHeartbeat, WorkerId, WorkerRef,
};
use xiuxian_db_store::{
    ValkeyClient, ValkeyLeaseId, ValkeyLeaseScriptResult, ValkeyQueueEntryId, ValkeyQueueKeys,
    ValkeyStoreConfig as DbValkeyStoreConfig, ValkeyStoreError, ValkeyStructuredClaimFilter,
    ValkeyStructuredClaimRequest, ValkeyStructuredQueue, ValkeyStructuredQueueEntry,
    ValkeyStructuredQueueLeaseRef, ValkeyWorkerId,
};

const DEFAULT_KEY_NAMESPACE: &str = "xiuxian:qianji:control";
const TASK_QUEUE_FIELD: &str = "task_queue";
const RUN_ID_FIELD: &str = "run_id";

/// Valkey key namespace for Qianji control hot state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValkeyKeyNamespace(String);

impl ValkeyKeyNamespace {
    /// Creates a non-empty key namespace.
    ///
    /// # Errors
    ///
    /// Returns [`ControlError::BlankId`] when the namespace is empty.
    pub fn new(value: impl Into<String>) -> ControlResult<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(ControlError::BlankId {
                field: "valkey_key_namespace",
            });
        }
        Ok(Self(value))
    }

    /// Borrows the namespace string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Default for ValkeyKeyNamespace {
    fn default() -> Self {
        Self(DEFAULT_KEY_NAMESPACE.to_owned())
    }
}

/// Valkey hot-state adapter configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValkeyHotStateConfig {
    redis_url: String,
    namespace: ValkeyKeyNamespace,
    db_store_config: DbValkeyStoreConfig,
    step_queue_keys: ValkeyQueueKeys,
    activity_queue_keys: ValkeyQueueKeys,
}

impl ValkeyHotStateConfig {
    /// Creates a config from a Valkey URL.
    ///
    /// # Errors
    ///
    /// Returns [`ControlError::BlankId`] when the URL is empty.
    pub fn new(redis_url: impl Into<String>) -> ControlResult<Self> {
        let redis_url = redis_url.into();
        if redis_url.trim().is_empty() {
            return Err(ControlError::BlankId { field: "redis_url" });
        }
        let namespace = ValkeyKeyNamespace::default();
        let db_store_config =
            db_store_config(&redis_url, namespace.as_str()).map_err(control_error)?;
        let step_queue_keys = step_queue_keys(namespace.as_str()).map_err(control_error)?;
        let activity_queue_keys = activity_queue_keys(namespace.as_str()).map_err(control_error)?;
        Ok(Self {
            redis_url,
            namespace,
            db_store_config,
            step_queue_keys,
            activity_queue_keys,
        })
    }

    /// Sets a custom key namespace.
    ///
    /// # Errors
    ///
    /// Returns [`ControlError::BlankId`] when the namespace is empty.
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> ControlResult<Self> {
        self.namespace = ValkeyKeyNamespace::new(namespace)?;
        self.db_store_config = db_store_config(self.redis_url.as_str(), self.namespace.as_str())
            .map_err(control_error)?;
        self.step_queue_keys = step_queue_keys(self.namespace.as_str()).map_err(control_error)?;
        self.activity_queue_keys =
            activity_queue_keys(self.namespace.as_str()).map_err(control_error)?;
        Ok(self)
    }

    /// Borrows the configured Valkey URL.
    #[must_use]
    pub fn redis_url(&self) -> &str {
        self.redis_url.as_str()
    }

    /// Borrows the configured key namespace.
    #[must_use]
    pub fn namespace(&self) -> &ValkeyKeyNamespace {
        &self.namespace
    }

    /// Returns the pending queue key.
    #[must_use]
    pub fn pending_queue_key(&self) -> String {
        format!("{}:pending", self.namespace.as_str())
    }

    /// Returns the lease deadline index key.
    #[must_use]
    pub fn lease_deadlines_key(&self) -> String {
        format!("{}:lease_deadlines", self.namespace.as_str())
    }

    /// Returns the payload key for one step.
    #[must_use]
    pub fn step_payload_key(&self, run_id: &RunId, step_id: &StepId) -> String {
        format!(
            "{}:step:{}",
            self.namespace.as_str(),
            step_entry_id(run_id, step_id)
        )
    }

    /// Returns the lease key for one step.
    #[must_use]
    pub fn lease_key(&self, run_id: &RunId, step_id: &StepId) -> String {
        format!(
            "{}:lease:{}",
            self.namespace.as_str(),
            step_entry_id(run_id, step_id)
        )
    }

    /// Returns the heartbeat key for one worker.
    #[must_use]
    pub fn heartbeat_key(&self, worker_id: &WorkerId) -> String {
        format!(
            "{}:heartbeat:{}",
            self.namespace.as_str(),
            worker_id.as_str()
        )
    }

    /// Returns the worker activity task pending queue key.
    #[must_use]
    pub fn activity_pending_queue_key(&self) -> String {
        format!("{}:activity_pending", self.namespace.as_str())
    }

    /// Returns the worker activity task lease deadline index key.
    #[must_use]
    pub fn activity_lease_deadlines_key(&self) -> String {
        format!("{}:activity_lease_deadlines", self.namespace.as_str())
    }

    /// Returns the worker activity task payload key.
    #[must_use]
    pub fn activity_payload_key(
        &self,
        run_id: &RunId,
        step_id: Option<&StepId>,
        activity_id: &ActivityId,
    ) -> String {
        format!(
            "{}:activity:{}",
            self.namespace.as_str(),
            activity_entry_id(run_id, step_id, activity_id)
        )
    }

    /// Returns the worker activity task lease key.
    #[must_use]
    pub fn activity_lease_key(
        &self,
        run_id: &RunId,
        step_id: Option<&StepId>,
        activity_id: &ActivityId,
    ) -> String {
        format!(
            "{}:activity_lease:{}",
            self.namespace.as_str(),
            activity_entry_id(run_id, step_id, activity_id)
        )
    }

    fn db_store_config(&self) -> DbValkeyStoreConfig {
        self.db_store_config.clone()
    }

    fn step_queue_keys(&self) -> ValkeyQueueKeys {
        self.step_queue_keys.clone()
    }

    fn activity_queue_keys(&self) -> ValkeyQueueKeys {
        self.activity_queue_keys.clone()
    }

    fn heartbeat_key_pattern(&self) -> String {
        format!("{}:heartbeat:*", self.namespace.as_str())
    }
}

fn db_store_config(
    redis_url: &str,
    namespace: &str,
) -> Result<DbValkeyStoreConfig, ValkeyStoreError> {
    DbValkeyStoreConfig::new(redis_url.to_owned())?.with_namespace(namespace.to_owned())
}

fn step_queue_keys(namespace: &str) -> Result<ValkeyQueueKeys, ValkeyStoreError> {
    ValkeyQueueKeys::new(
        format!("{namespace}:pending"),
        format!("{namespace}:lease_deadlines"),
        format!("{namespace}:step:"),
        format!("{namespace}:lease:"),
    )
}

fn activity_queue_keys(namespace: &str) -> Result<ValkeyQueueKeys, ValkeyStoreError> {
    ValkeyQueueKeys::new(
        format!("{namespace}:activity_pending"),
        format!("{namespace}:activity_lease_deadlines"),
        format!("{namespace}:activity:"),
        format!("{namespace}:activity_lease:"),
    )
}

/// Valkey-backed hot-state store for queues, leases, and heartbeats.
pub struct ValkeyHotStateStore {
    config: ValkeyHotStateConfig,
    client: ValkeyClient,
    step_queue: ValkeyStructuredQueue,
    activity_queue: ValkeyStructuredQueue,
    lease_sequence: AtomicU64,
}

impl ValkeyHotStateStore {
    /// Creates a new Valkey hot-state store from config.
    #[must_use]
    pub fn new(config: ValkeyHotStateConfig) -> Self {
        let client = ValkeyClient::new(config.db_store_config());
        let step_queue = ValkeyStructuredQueue::new(client.clone(), config.step_queue_keys());
        let activity_queue =
            ValkeyStructuredQueue::new(client.clone(), config.activity_queue_keys());
        Self {
            config,
            client,
            step_queue,
            activity_queue,
            lease_sequence: AtomicU64::new(0),
        }
    }

    fn next_lease_id(&self, worker_id: &WorkerId, now_ms: u64) -> ControlResult<LeaseId> {
        let sequence = self.lease_sequence.fetch_add(1, Ordering::Relaxed) + 1;
        LeaseId::new(format!("valkey-{worker_id}-{now_ms}-{sequence}"))
    }

    fn next_activity_lease_id(&self, worker_id: &WorkerId, now_ms: u64) -> ControlResult<LeaseId> {
        let sequence = self.lease_sequence.fetch_add(1, Ordering::Relaxed) + 1;
        LeaseId::new(format!("valkey-activity-{worker_id}-{now_ms}-{sequence}"))
    }
}

#[async_trait::async_trait]
impl HotStateStore for ValkeyHotStateStore {
    async fn enqueue_step(&self, step: RunnableStep) -> ControlResult<()> {
        let payload_json = encode_runnable_step(&step)?;
        let entry_id = valkey_entry_id(step_entry_id(&step.run_id, &step.step_id))?;
        self.step_queue
            .enqueue(
                ValkeyStructuredQueueEntry::new(entry_id, &payload_json)
                    .with_priority(step.priority)
                    .with_not_before_ms(step.not_before_ms),
            )
            .await
            .map_err(control_error)?;
        Ok(())
    }

    async fn acquire_lease(
        &self,
        worker: WorkerRef,
        now_ms: u64,
        lease_ttl_ms: u64,
    ) -> ControlResult<Option<StepLease>> {
        validate_positive_ttl("validate_valkey_lease_ttl", lease_ttl_ms)?;
        let lease_id = self.next_lease_id(&worker.worker_id, now_ms)?;
        let valkey_worker_id = valkey_worker_id(&worker.worker_id)?;
        let valkey_lease_id = valkey_lease_id(&lease_id)?;
        let claimed = self
            .step_queue
            .claim(ValkeyStructuredClaimRequest::new(
                &valkey_worker_id,
                &valkey_lease_id,
                now_ms,
                lease_ttl_ms,
            ))
            .await
            .map_err(control_error)?;
        let Some(claimed) = claimed else {
            return Ok(None);
        };
        let step = decode_runnable_step(claimed.payload())?;
        Ok(Some(StepLease {
            lease_id,
            run_id: step.run_id,
            step_id: step.step_id,
            worker_id: worker.worker_id,
            acquired_at_ms: claimed.acquired_at_ms(),
            expires_at_ms: claimed.expires_at_ms(),
        }))
    }

    async fn renew_lease(
        &self,
        lease: &StepLease,
        now_ms: u64,
        lease_ttl_ms: u64,
    ) -> ControlResult<bool> {
        validate_positive_ttl("validate_valkey_lease_ttl", lease_ttl_ms)?;
        let entry_id = valkey_entry_id(step_entry_id(&lease.run_id, &lease.step_id))?;
        let lease_id = valkey_lease_id(&lease.lease_id)?;
        let worker_id = valkey_worker_id(&lease.worker_id)?;
        let result = self
            .step_queue
            .renew(
                ValkeyStructuredQueueLeaseRef::new(&lease_id, &entry_id, &worker_id),
                now_ms,
                lease_ttl_ms,
            )
            .await
            .map_err(control_error)?;
        Ok(result == ValkeyLeaseScriptResult::Applied)
    }

    async fn release_lease(&self, lease: &StepLease) -> ControlResult<bool> {
        let entry_id = valkey_entry_id(step_entry_id(&lease.run_id, &lease.step_id))?;
        let lease_id = valkey_lease_id(&lease.lease_id)?;
        let worker_id = valkey_worker_id(&lease.worker_id)?;
        let result = self
            .step_queue
            .release(ValkeyStructuredQueueLeaseRef::new(
                &lease_id, &entry_id, &worker_id,
            ))
            .await
            .map_err(control_error)?;
        Ok(result == ValkeyLeaseScriptResult::Applied)
    }

    async fn reclaim_expired_lease(&self, lease: &StepLease, now_ms: u64) -> ControlResult<bool> {
        let entry_id = valkey_entry_id(step_entry_id(&lease.run_id, &lease.step_id))?;
        let lease_id = valkey_lease_id(&lease.lease_id)?;
        let worker_id = valkey_worker_id(&lease.worker_id)?;
        let result = self
            .step_queue
            .reclaim_expired(
                ValkeyStructuredQueueLeaseRef::new(&lease_id, &entry_id, &worker_id),
                now_ms,
            )
            .await
            .map_err(control_error)?;
        Ok(result == ValkeyLeaseScriptResult::Applied)
    }

    async fn enqueue_activity_task(&self, task: RunnableActivityTask) -> ControlResult<()> {
        let payload_json = encode_runnable_activity_task(&task)?;
        let entry_id = activity_entry_id(
            &task.task.run_id,
            task.task.step_id.as_ref(),
            &task.task.activity_id,
        );
        let entry_id = valkey_entry_id(entry_id)?;
        let indexed_fields = [
            (TASK_QUEUE_FIELD, task.task.task_queue.as_str()),
            (RUN_ID_FIELD, task.task.run_id.as_str()),
        ];
        self.activity_queue
            .enqueue(
                ValkeyStructuredQueueEntry::new(entry_id, &payload_json)
                    .with_priority(task.priority)
                    .with_not_before_ms(task.not_before_ms)
                    .with_fields(&indexed_fields),
            )
            .await
            .map_err(control_error)?;
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
            .await
    }

    async fn claim_activity_task_for_run(
        &self,
        request: crate::RunScopedActivityTaskClaimRequest,
    ) -> ControlResult<Option<HotStateLeasedActivityTask>> {
        self.claim_matching_activity_task(
            request.worker,
            Some(&request.run_id),
            request.task_queue.as_ref(),
            request.now_ms,
            request.lease_ttl_ms,
        )
        .await
    }

    async fn release_activity_task_lease(&self, lease: &ActivityTaskLease) -> ControlResult<bool> {
        let entry_id = valkey_entry_id(activity_entry_id(
            &lease.run_id,
            lease.step_id.as_ref(),
            &lease.activity_id,
        ))?;
        let lease_id = valkey_lease_id(&lease.lease_id)?;
        let worker_id = valkey_worker_id(&lease.worker_id)?;
        let result = self
            .activity_queue
            .release(ValkeyStructuredQueueLeaseRef::new(
                &lease_id, &entry_id, &worker_id,
            ))
            .await
            .map_err(control_error)?;
        Ok(result == ValkeyLeaseScriptResult::Applied)
    }

    async fn reclaim_expired_activity_task_lease(
        &self,
        lease: &ActivityTaskLease,
        now_ms: u64,
    ) -> ControlResult<bool> {
        let entry_id = valkey_entry_id(activity_entry_id(
            &lease.run_id,
            lease.step_id.as_ref(),
            &lease.activity_id,
        ))?;
        let lease_id = valkey_lease_id(&lease.lease_id)?;
        let worker_id = valkey_worker_id(&lease.worker_id)?;
        let result = self
            .activity_queue
            .reclaim_expired(
                ValkeyStructuredQueueLeaseRef::new(&lease_id, &entry_id, &worker_id),
                now_ms,
            )
            .await
            .map_err(control_error)?;
        Ok(result == ValkeyLeaseScriptResult::Applied)
    }

    async fn heartbeat(&self, heartbeat: WorkerHeartbeat) -> ControlResult<()> {
        let payload_json = encode_heartbeat(&heartbeat)?;
        let ttl_ms = heartbeat
            .expires_at_ms
            .saturating_sub(heartbeat.observed_at_ms);
        validate_positive_ttl("validate_valkey_worker_heartbeat_ttl", ttl_ms)?;
        let heartbeat_key = self.config.heartbeat_key(&heartbeat.worker_id);
        self.client
            .set_string_with_ttl(&heartbeat_key, &payload_json, ttl_ms)
            .await
            .map_err(control_error)?;
        Ok(())
    }

    async fn load_heartbeat(&self, worker_id: &WorkerId) -> ControlResult<Option<WorkerHeartbeat>> {
        let heartbeat_key = self.config.heartbeat_key(worker_id);
        self.client
            .get_string(&heartbeat_key)
            .await
            .map_err(control_error)?
            .map(|payload| decode_heartbeat(&payload))
            .transpose()
    }

    async fn load_snapshot(&self, observed_at_ms: u64) -> ControlResult<HotStateSnapshot> {
        self.load_hot_state_snapshot(observed_at_ms).await
    }
}

impl ValkeyHotStateStore {
    async fn claim_matching_activity_task(
        &self,
        worker: WorkerRef,
        run_id: Option<&RunId>,
        task_queue: Option<&TaskQueue>,
        now_ms: u64,
        lease_ttl_ms: u64,
    ) -> ControlResult<Option<HotStateLeasedActivityTask>> {
        validate_positive_ttl("validate_valkey_activity_task_lease_ttl", lease_ttl_ms)?;
        let lease_id = self.next_activity_lease_id(&worker.worker_id, now_ms)?;
        let filters = activity_claim_filters(run_id, task_queue);
        let valkey_worker_id = valkey_worker_id(&worker.worker_id)?;
        let valkey_lease_id = valkey_lease_id(&lease_id)?;
        let claimed = self
            .activity_queue
            .claim(
                ValkeyStructuredClaimRequest::new(
                    &valkey_worker_id,
                    &valkey_lease_id,
                    now_ms,
                    lease_ttl_ms,
                )
                .with_filters(&filters),
            )
            .await
            .map_err(control_error)?;
        let Some(claimed) = claimed else {
            return Ok(None);
        };
        let activity_task = decode_runnable_activity_task(claimed.payload())?;
        let lease = ActivityTaskLease {
            lease_id,
            run_id: activity_task.task.run_id.clone(),
            step_id: activity_task.task.step_id.clone(),
            activity_id: activity_task.task.activity_id.clone(),
            worker_id: worker.worker_id,
            acquired_at_ms: claimed.acquired_at_ms(),
            expires_at_ms: claimed.expires_at_ms(),
        };
        Ok(Some(HotStateLeasedActivityTask {
            activity_task,
            lease,
        }))
    }

    async fn load_hot_state_snapshot(
        &self,
        observed_at_ms: u64,
    ) -> ControlResult<HotStateSnapshot> {
        let pending_entries = self
            .step_queue
            .pending_entries()
            .await
            .map_err(control_error)?;
        let lease_entries = self
            .step_queue
            .lease_entries()
            .await
            .map_err(control_error)?;
        let activity_pending_entries = self
            .activity_queue
            .pending_entries()
            .await
            .map_err(control_error)?;
        let activity_lease_entries = self
            .activity_queue
            .lease_entries()
            .await
            .map_err(control_error)?;
        let heartbeat_keys = self
            .client
            .keys(&self.config.heartbeat_key_pattern())
            .await
            .map_err(control_error)?;

        let mut snapshot = HotStateSnapshot::new(observed_at_ms);
        self.append_pending_steps(&mut snapshot, pending_entries)
            .await?;
        self.append_leased_steps(&mut snapshot, lease_entries)
            .await?;
        self.append_pending_activity_tasks(&mut snapshot, activity_pending_entries)
            .await?;
        self.append_leased_activity_tasks(&mut snapshot, activity_lease_entries)
            .await?;
        self.append_heartbeats(&mut snapshot, heartbeat_keys)
            .await?;
        sort_hot_state_snapshot(&mut snapshot);
        Ok(snapshot)
    }

    async fn append_pending_steps(
        &self,
        snapshot: &mut HotStateSnapshot,
        entries: Vec<ValkeyQueueEntryId>,
    ) -> ControlResult<()> {
        for entry_id in entries {
            if let Some(step) = self.load_step_payload_by_entry(&entry_id).await? {
                snapshot.pending_steps.push(step);
            }
        }
        Ok(())
    }

    async fn append_leased_steps(
        &self,
        snapshot: &mut HotStateSnapshot,
        entries: Vec<ValkeyQueueEntryId>,
    ) -> ControlResult<()> {
        for entry_id in entries {
            let Some(step) = self.load_step_payload_by_entry(&entry_id).await? else {
                continue;
            };
            let lease_hash = self
                .step_queue
                .load_lease_hash(&entry_id)
                .await
                .map_err(control_error)?;
            if let Some(lease) = decode_lease_hash(&step, &lease_hash)? {
                snapshot
                    .leased_steps
                    .push(HotStateLeasedStep { step, lease });
            }
        }
        Ok(())
    }

    async fn append_pending_activity_tasks(
        &self,
        snapshot: &mut HotStateSnapshot,
        entries: Vec<ValkeyQueueEntryId>,
    ) -> ControlResult<()> {
        for entry_id in entries {
            if let Some(activity_task) = self.load_activity_payload_by_entry(&entry_id).await? {
                snapshot.pending_activity_tasks.push(activity_task);
            }
        }
        Ok(())
    }

    async fn append_leased_activity_tasks(
        &self,
        snapshot: &mut HotStateSnapshot,
        entries: Vec<ValkeyQueueEntryId>,
    ) -> ControlResult<()> {
        for entry_id in entries {
            let Some(activity_task) = self.load_activity_payload_by_entry(&entry_id).await? else {
                continue;
            };
            let lease_hash = self
                .activity_queue
                .load_lease_hash(&entry_id)
                .await
                .map_err(control_error)?;
            if let Some(lease) = decode_activity_lease_hash(&activity_task, &lease_hash)? {
                snapshot
                    .leased_activity_tasks
                    .push(HotStateLeasedActivityTask {
                        activity_task,
                        lease,
                    });
            }
        }
        Ok(())
    }

    async fn append_heartbeats(
        &self,
        snapshot: &mut HotStateSnapshot,
        keys: Vec<String>,
    ) -> ControlResult<()> {
        for heartbeat_key in keys {
            if let Some(payload_json) = self
                .client
                .get_string(&heartbeat_key)
                .await
                .map_err(control_error)?
            {
                snapshot
                    .worker_heartbeats
                    .push(decode_heartbeat(&payload_json)?);
            }
        }
        Ok(())
    }

    async fn load_step_payload_by_entry(
        &self,
        entry_id: &ValkeyQueueEntryId,
    ) -> ControlResult<Option<RunnableStep>> {
        self.step_queue
            .load_payload(entry_id)
            .await
            .map_err(control_error)?
            .map(|payload| decode_runnable_step(&payload))
            .transpose()
    }

    async fn load_activity_payload_by_entry(
        &self,
        entry_id: &ValkeyQueueEntryId,
    ) -> ControlResult<Option<RunnableActivityTask>> {
        self.activity_queue
            .load_payload(entry_id)
            .await
            .map_err(control_error)?
            .map(|payload| decode_runnable_activity_task(&payload))
            .transpose()
    }
}

fn activity_claim_filters<'a>(
    run_id: Option<&'a RunId>,
    task_queue: Option<&'a TaskQueue>,
) -> Vec<ValkeyStructuredClaimFilter<'a>> {
    let mut filters = Vec::with_capacity(2);
    if let Some(task_queue) = task_queue {
        filters.push(ValkeyStructuredClaimFilter::new(
            TASK_QUEUE_FIELD,
            task_queue.as_str(),
        ));
    }
    if let Some(run_id) = run_id {
        filters.push(ValkeyStructuredClaimFilter::new(
            RUN_ID_FIELD,
            run_id.as_str(),
        ));
    }
    filters
}

fn encode_runnable_step(step: &RunnableStep) -> ControlResult<String> {
    serde_json::to_string(step).map_err(|error| ControlError::Codec {
        operation: "encode_valkey_runnable_step",
        message: error.to_string(),
    })
}

fn decode_runnable_step(payload: &str) -> ControlResult<RunnableStep> {
    serde_json::from_str(payload).map_err(|error| ControlError::Codec {
        operation: "decode_valkey_runnable_step",
        message: error.to_string(),
    })
}

fn encode_runnable_activity_task(task: &RunnableActivityTask) -> ControlResult<String> {
    serde_json::to_string(task).map_err(|error| ControlError::Codec {
        operation: "encode_valkey_runnable_activity_task",
        message: error.to_string(),
    })
}

fn decode_runnable_activity_task(payload: &str) -> ControlResult<RunnableActivityTask> {
    serde_json::from_str(payload).map_err(|error| ControlError::Codec {
        operation: "decode_valkey_runnable_activity_task",
        message: error.to_string(),
    })
}

fn encode_heartbeat(heartbeat: &WorkerHeartbeat) -> ControlResult<String> {
    serde_json::to_string(heartbeat).map_err(|error| ControlError::Codec {
        operation: "encode_valkey_worker_heartbeat",
        message: error.to_string(),
    })
}

fn decode_heartbeat(payload: &str) -> ControlResult<WorkerHeartbeat> {
    serde_json::from_str(payload).map_err(|error| ControlError::Codec {
        operation: "decode_valkey_worker_heartbeat",
        message: error.to_string(),
    })
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

fn decode_lease_hash(
    step: &RunnableStep,
    lease_hash: &[(String, String)],
) -> ControlResult<Option<StepLease>> {
    if lease_hash.is_empty() {
        return Ok(None);
    }
    let value = |key: &str| {
        lease_hash
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, value)| value.as_str())
    };
    let lease_id = LeaseId::new(required_hash_value(value("lease_id"), "lease_id")?)?;
    let worker_id = WorkerId::new(required_hash_value(value("worker_id"), "worker_id")?)?;
    let acquired_at_ms = parse_hash_u64(value("acquired_at_ms"), "acquired_at_ms")?;
    let expires_at_ms = parse_hash_u64(value("expires_at_ms"), "expires_at_ms")?;
    Ok(Some(StepLease {
        lease_id,
        run_id: step.run_id.clone(),
        step_id: step.step_id.clone(),
        worker_id,
        acquired_at_ms,
        expires_at_ms,
    }))
}

fn decode_activity_lease_hash(
    activity_task: &RunnableActivityTask,
    lease_hash: &[(String, String)],
) -> ControlResult<Option<ActivityTaskLease>> {
    if lease_hash.is_empty() {
        return Ok(None);
    }
    let value = |key: &str| {
        lease_hash
            .iter()
            .find(|(candidate, _)| candidate == key)
            .map(|(_, value)| value.as_str())
    };
    let lease_id = LeaseId::new(required_hash_value(value("lease_id"), "lease_id")?)?;
    let worker_id = WorkerId::new(required_hash_value(value("worker_id"), "worker_id")?)?;
    let acquired_at_ms = parse_hash_u64(value("acquired_at_ms"), "acquired_at_ms")?;
    let expires_at_ms = parse_hash_u64(value("expires_at_ms"), "expires_at_ms")?;
    Ok(Some(ActivityTaskLease {
        lease_id,
        run_id: activity_task.task.run_id.clone(),
        step_id: activity_task.task.step_id.clone(),
        activity_id: activity_task.task.activity_id.clone(),
        worker_id,
        acquired_at_ms,
        expires_at_ms,
    }))
}

fn required_hash_value<'a>(value: Option<&'a str>, field: &'static str) -> ControlResult<&'a str> {
    value.ok_or_else(|| ControlError::Codec {
        operation: "decode_valkey_lease_hash",
        message: format!("missing lease hash field `{field}`"),
    })
}

fn parse_hash_u64(value: Option<&str>, field: &'static str) -> ControlResult<u64> {
    required_hash_value(value, field)?
        .parse()
        .map_err(|error: std::num::ParseIntError| ControlError::Codec {
            operation: "decode_valkey_lease_hash",
            message: format!("invalid `{field}`: {error}"),
        })
}

fn valkey_entry_id(value: String) -> ControlResult<ValkeyQueueEntryId> {
    ValkeyQueueEntryId::new(value).map_err(control_error)
}

fn valkey_lease_id(value: &LeaseId) -> ControlResult<ValkeyLeaseId> {
    ValkeyLeaseId::new(value.as_str().to_owned()).map_err(control_error)
}

fn valkey_worker_id(value: &WorkerId) -> ControlResult<ValkeyWorkerId> {
    ValkeyWorkerId::new(value.as_str().to_owned()).map_err(control_error)
}

fn step_entry_id(run_id: &RunId, step_id: &StepId) -> String {
    format!("{}|{}", run_id.as_str(), step_id.as_str())
}

fn activity_entry_id(run_id: &RunId, step_id: Option<&StepId>, activity_id: &ActivityId) -> String {
    match step_id {
        Some(step_id) => format!(
            "{}|{}|{}",
            run_id.as_str(),
            step_id.as_str(),
            activity_id.as_str()
        ),
        None => format!("{}||{}", run_id.as_str(), activity_id.as_str()),
    }
}

fn sort_hot_state_snapshot(snapshot: &mut HotStateSnapshot) {
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

fn control_error(error: ValkeyStoreError) -> ControlError {
    match error {
        ValkeyStoreError::BlankId { field } => ControlError::BlankId { field },
        ValkeyStoreError::NonPositiveTtl { field } => ControlError::Storage {
            operation: "validate_valkey_ttl",
            message: format!("{field} must be greater than zero"),
        },
        ValkeyStoreError::Storage { operation, message } => {
            ControlError::Storage { operation, message }
        }
        ValkeyStoreError::LeaseNotOwned { ownership } => match (
            LeaseId::new(ownership.lease_id().as_str().to_owned()),
            WorkerId::new(ownership.worker_id().as_str().to_owned()),
        ) {
            (Ok(lease_id), Ok(worker_id)) => ControlError::LeaseNotOwned {
                lease_id,
                worker_id,
            },
            (lease_id, worker_id) => ControlError::Storage {
                operation: "decode_valkey_lease_ownership",
                message: format!(
                    "invalid lease ownership ids: lease_id={lease_id:?}, worker_id={worker_id:?}"
                ),
            },
        },
        ValkeyStoreError::UnexpectedLeaseScriptResult { result } => ControlError::Storage {
            operation: "decode_valkey_lease_script_result",
            message: format!("unexpected lease script result {result}"),
        },
    }
}
