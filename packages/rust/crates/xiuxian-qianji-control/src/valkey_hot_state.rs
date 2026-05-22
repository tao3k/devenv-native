//! Valkey-backed hot scheduling state.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use redis::FromRedisValue;
use tokio::sync::{Mutex, RwLock};

use crate::{
    ActivityId, ActivityTaskLease, ControlError, ControlResult, HotStateLeasedActivityTask,
    HotStateLeasedStep, HotStateSnapshot, HotStateStore, LeaseId, RunId, RunnableActivityTask,
    RunnableStep, StepId, StepLease, TaskQueue, WorkerHeartbeat, WorkerId, WorkerRef,
};

const DEFAULT_KEY_NAMESPACE: &str = "xiuxian:qianji:control";
const ACQUIRE_LEASE_LUA: &str = r"
local pending_key = KEYS[1]
local lease_deadlines_key = KEYS[2]
local payload_prefix = ARGV[1]
local lease_prefix = ARGV[2]
local now_ms = tonumber(ARGV[3])
local lease_id = ARGV[4]
local worker_id = ARGV[5]
local acquired_at_ms = ARGV[6]
local expires_at_ms = ARGV[7]
local ttl_ms = tonumber(ARGV[8])

local expired = redis.call('ZRANGEBYSCORE', lease_deadlines_key, '-inf', now_ms)
for _, entry_id in ipairs(expired) do
  local lease_key = lease_prefix .. entry_id
  local payload_key = payload_prefix .. entry_id
  if redis.call('EXISTS', lease_key) == 0 then
    local priority_score = redis.call('HGET', payload_key, 'priority_score')
    if priority_score then
      redis.call('ZADD', pending_key, priority_score, entry_id)
    end
    redis.call('ZREM', lease_deadlines_key, entry_id)
  else
    local active_expires_at_ms = tonumber(redis.call('HGET', lease_key, 'expires_at_ms') or '0')
    if active_expires_at_ms <= now_ms then
      redis.call('DEL', lease_key)
      local priority_score = redis.call('HGET', payload_key, 'priority_score')
      if priority_score then
        redis.call('ZADD', pending_key, priority_score, entry_id)
      end
      redis.call('ZREM', lease_deadlines_key, entry_id)
    end
  end
end

local candidates = redis.call('ZRANGE', pending_key, 0, -1)
for _, entry_id in ipairs(candidates) do
  local payload_key = payload_prefix .. entry_id
  local payload = redis.call('HGET', payload_key, 'payload')
  if not payload then
    redis.call('ZREM', pending_key, entry_id)
  else
    local not_before_ms = tonumber(redis.call('HGET', payload_key, 'not_before_ms') or '0')
    if not_before_ms <= now_ms then
      local lease_key = lease_prefix .. entry_id
      redis.call('ZREM', pending_key, entry_id)
      redis.call(
        'HSET',
        lease_key,
        'lease_id',
        lease_id,
        'worker_id',
        worker_id,
        'acquired_at_ms',
        acquired_at_ms,
        'expires_at_ms',
        expires_at_ms
      )
      redis.call('PEXPIRE', lease_key, ttl_ms)
      redis.call('ZADD', lease_deadlines_key, expires_at_ms, entry_id)
      return payload
    end
  end
end
return false
";
const CLAIM_ACTIVITY_TASK_LUA: &str = r"
local pending_key = KEYS[1]
local lease_deadlines_key = KEYS[2]
local payload_prefix = ARGV[1]
local lease_prefix = ARGV[2]
local now_ms = tonumber(ARGV[3])
local lease_id = ARGV[4]
local worker_id = ARGV[5]
local acquired_at_ms = ARGV[6]
local expires_at_ms = ARGV[7]
local ttl_ms = tonumber(ARGV[8])
local task_queue_filter = ARGV[9]

local expired = redis.call('ZRANGEBYSCORE', lease_deadlines_key, '-inf', now_ms)
for _, entry_id in ipairs(expired) do
  local lease_key = lease_prefix .. entry_id
  local payload_key = payload_prefix .. entry_id
  if redis.call('EXISTS', lease_key) == 0 then
    local priority_score = redis.call('HGET', payload_key, 'priority_score')
    if priority_score then
      redis.call('ZADD', pending_key, priority_score, entry_id)
    end
    redis.call('ZREM', lease_deadlines_key, entry_id)
  else
    local active_expires_at_ms = tonumber(redis.call('HGET', lease_key, 'expires_at_ms') or '0')
    if active_expires_at_ms <= now_ms then
      redis.call('DEL', lease_key)
      local priority_score = redis.call('HGET', payload_key, 'priority_score')
      if priority_score then
        redis.call('ZADD', pending_key, priority_score, entry_id)
      end
      redis.call('ZREM', lease_deadlines_key, entry_id)
    end
  end
end

local candidates = redis.call('ZRANGE', pending_key, 0, -1)
for _, entry_id in ipairs(candidates) do
  local payload_key = payload_prefix .. entry_id
  local payload = redis.call('HGET', payload_key, 'payload')
  if not payload then
    redis.call('ZREM', pending_key, entry_id)
  else
    local not_before_ms = tonumber(redis.call('HGET', payload_key, 'not_before_ms') or '0')
    local task_queue = redis.call('HGET', payload_key, 'task_queue') or ''
    if not_before_ms <= now_ms and (task_queue_filter == '' or task_queue == task_queue_filter) then
      local lease_key = lease_prefix .. entry_id
      redis.call('ZREM', pending_key, entry_id)
      redis.call(
        'HSET',
        lease_key,
        'lease_id',
        lease_id,
        'worker_id',
        worker_id,
        'acquired_at_ms',
        acquired_at_ms,
        'expires_at_ms',
        expires_at_ms
      )
      redis.call('PEXPIRE', lease_key, ttl_ms)
      redis.call('ZADD', lease_deadlines_key, expires_at_ms, entry_id)
      return payload
    end
  end
end
return false
";
const RENEW_LEASE_LUA: &str = r"
local lease_key = KEYS[1]
local lease_deadlines_key = KEYS[2]
local lease_id = ARGV[1]
local worker_id = ARGV[2]
local expires_at_ms = ARGV[3]
local ttl_ms = tonumber(ARGV[4])
local entry_id = ARGV[5]

if redis.call('EXISTS', lease_key) == 0 then
  return 0
end
if redis.call('HGET', lease_key, 'lease_id') ~= lease_id or redis.call('HGET', lease_key, 'worker_id') ~= worker_id then
  return -1
end
redis.call('HSET', lease_key, 'expires_at_ms', expires_at_ms)
redis.call('PEXPIRE', lease_key, ttl_ms)
redis.call('ZADD', lease_deadlines_key, expires_at_ms, entry_id)
return 1
";
const RELEASE_LEASE_LUA: &str = r"
local lease_key = KEYS[1]
local lease_deadlines_key = KEYS[2]
local lease_id = ARGV[1]
local worker_id = ARGV[2]
local entry_id = ARGV[3]

if redis.call('EXISTS', lease_key) == 0 then
  return 0
end
if redis.call('HGET', lease_key, 'lease_id') ~= lease_id or redis.call('HGET', lease_key, 'worker_id') ~= worker_id then
  return -1
end
redis.call('DEL', lease_key)
redis.call('ZREM', lease_deadlines_key, entry_id)
return 1
";
const RECLAIM_EXPIRED_LEASE_LUA: &str = r"
local pending_key = KEYS[1]
local lease_key = KEYS[2]
local lease_deadlines_key = KEYS[3]
local payload_key = KEYS[4]
local lease_id = ARGV[1]
local worker_id = ARGV[2]
local entry_id = ARGV[3]
local now_ms = tonumber(ARGV[4])

if redis.call('EXISTS', lease_key) == 0 then
  return 0
end
if redis.call('HGET', lease_key, 'lease_id') ~= lease_id or redis.call('HGET', lease_key, 'worker_id') ~= worker_id then
  return -1
end
local active_expires_at_ms = tonumber(redis.call('HGET', lease_key, 'expires_at_ms') or '0')
if active_expires_at_ms > now_ms then
  return 0
end
local priority_score = redis.call('HGET', payload_key, 'priority_score')
if not priority_score then
  return 0
end
redis.call('DEL', lease_key)
redis.call('ZREM', lease_deadlines_key, entry_id)
redis.call('ZADD', pending_key, priority_score, entry_id)
return 1
";
const ENQUEUE_ACTIVITY_TASK_LUA: &str = r"
local pending_key = KEYS[1]
local payload_key = KEYS[2]
local lease_key = KEYS[3]
local payload = ARGV[1]
local priority_score = ARGV[2]
local not_before_ms = ARGV[3]
local task_queue = ARGV[4]
local entry_id = ARGV[5]

if redis.call('EXISTS', lease_key) == 1 then
  return 0
end
redis.call(
  'HSET',
  payload_key,
  'payload',
  payload,
  'priority_score',
  priority_score,
  'not_before_ms',
  not_before_ms,
  'task_queue',
  task_queue
)
redis.call('ZADD', pending_key, priority_score, entry_id)
return 1
";

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
        Ok(Self {
            redis_url,
            namespace: ValkeyKeyNamespace::default(),
        })
    }

    /// Sets a custom key namespace.
    ///
    /// # Errors
    ///
    /// Returns [`ControlError::BlankId`] when the namespace is empty.
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> ControlResult<Self> {
        self.namespace = ValkeyKeyNamespace::new(namespace)?;
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

    fn step_payload_prefix(&self) -> String {
        format!("{}:step:", self.namespace.as_str())
    }

    fn lease_prefix(&self) -> String {
        format!("{}:lease:", self.namespace.as_str())
    }

    fn step_payload_key_for_entry(&self, entry_id: &str) -> String {
        format!("{}{entry_id}", self.step_payload_prefix())
    }

    fn lease_key_for_entry(&self, entry_id: &str) -> String {
        format!("{}{entry_id}", self.lease_prefix())
    }

    fn heartbeat_key_pattern(&self) -> String {
        format!("{}:heartbeat:*", self.namespace.as_str())
    }

    fn activity_payload_prefix(&self) -> String {
        format!("{}:activity:", self.namespace.as_str())
    }

    fn activity_lease_prefix(&self) -> String {
        format!("{}:activity_lease:", self.namespace.as_str())
    }

    fn activity_payload_key_for_entry(&self, entry_id: &str) -> String {
        format!("{}{entry_id}", self.activity_payload_prefix())
    }

    fn activity_lease_key_for_entry(&self, entry_id: &str) -> String {
        format!("{}{entry_id}", self.activity_lease_prefix())
    }
}

/// Valkey-backed hot-state store for queues, leases, and heartbeats.
pub struct ValkeyHotStateStore {
    config: ValkeyHotStateConfig,
    connection: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
    reconnect_lock: Arc<Mutex<()>>,
    lease_sequence: AtomicU64,
}

impl ValkeyHotStateStore {
    /// Creates a new Valkey hot-state store from config.
    #[must_use]
    pub fn new(config: ValkeyHotStateConfig) -> Self {
        Self {
            config,
            connection: Arc::new(RwLock::new(None)),
            reconnect_lock: Arc::new(Mutex::new(())),
            lease_sequence: AtomicU64::new(0),
        }
    }

    async fn run_command<T, F>(&self, operation: &'static str, build: F) -> ControlResult<T>
    where
        T: FromRedisValue + Send,
        F: Fn() -> redis::Cmd,
    {
        let mut last_error: Option<redis::RedisError> = None;
        for _ in 0..2 {
            let mut connection = self.acquire_connection().await?;
            let command = build();
            let result: redis::RedisResult<T> = command.query_async(&mut connection).await;
            match result {
                Ok(value) => return Ok(value),
                Err(error) => {
                    self.invalidate_connection().await;
                    last_error = Some(error);
                }
            }
        }
        Err(ControlError::Storage {
            operation,
            message: last_error.map_or_else(
                || "Valkey command failed unexpectedly".to_owned(),
                |error| error.to_string(),
            ),
        })
    }

    async fn acquire_connection(&self) -> ControlResult<redis::aio::MultiplexedConnection> {
        if let Some(connection) = self.connection.read().await.as_ref().cloned() {
            return Ok(connection);
        }

        let _guard = self.reconnect_lock.lock().await;
        if let Some(connection) = self.connection.read().await.as_ref().cloned() {
            return Ok(connection);
        }

        let client = redis::Client::open(self.config.redis_url()).map_err(|error| {
            ControlError::Storage {
                operation: "open_valkey_hot_state_client",
                message: error.to_string(),
            }
        })?;
        let connection = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|error| ControlError::Storage {
                operation: "connect_valkey_hot_state",
                message: error.to_string(),
            })?;
        {
            let mut guard = self.connection.write().await;
            *guard = Some(connection.clone());
        }
        Ok(connection)
    }

    async fn invalidate_connection(&self) {
        let mut guard = self.connection.write().await;
        *guard = None;
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
        let entry_id = step_entry_id(&step.run_id, &step.step_id);
        let payload_key = self.config.step_payload_key(&step.run_id, &step.step_id);
        let pending_queue_key = self.config.pending_queue_key();
        let priority_score = priority_score(step.priority);
        let _: i64 = self
            .run_command("valkey_hot_state_store_step_payload", || {
                let mut command = redis::cmd("HSET");
                command
                    .arg(&payload_key)
                    .arg("payload")
                    .arg(&payload_json)
                    .arg("priority_score")
                    .arg(priority_score.to_string())
                    .arg("not_before_ms")
                    .arg(step.not_before_ms.to_string());
                command
            })
            .await?;
        let _: i64 = self
            .run_command("valkey_hot_state_enqueue_step", || {
                let mut command = redis::cmd("ZADD");
                command
                    .arg(&pending_queue_key)
                    .arg(priority_score)
                    .arg(&entry_id);
                command
            })
            .await?;
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
        let expires_at_ms = now_ms.saturating_add(lease_ttl_ms);
        let pending_queue_key = self.config.pending_queue_key();
        let lease_deadlines_key = self.config.lease_deadlines_key();
        let payload_prefix = self.config.step_payload_prefix();
        let lease_prefix = self.config.lease_prefix();
        let payload_json: Option<String> = self
            .run_command("valkey_hot_state_acquire_lease", || {
                let mut command = redis::cmd("EVAL");
                command
                    .arg(ACQUIRE_LEASE_LUA)
                    .arg(2)
                    .arg(&pending_queue_key)
                    .arg(&lease_deadlines_key)
                    .arg(&payload_prefix)
                    .arg(&lease_prefix)
                    .arg(now_ms.to_string())
                    .arg(lease_id.as_str())
                    .arg(worker.worker_id.as_str())
                    .arg(now_ms.to_string())
                    .arg(expires_at_ms.to_string())
                    .arg(lease_ttl_ms.to_string());
                command
            })
            .await?;
        let Some(payload_json) = payload_json else {
            return Ok(None);
        };
        let step = decode_runnable_step(&payload_json)?;
        Ok(Some(StepLease {
            lease_id,
            run_id: step.run_id,
            step_id: step.step_id,
            worker_id: worker.worker_id,
            acquired_at_ms: now_ms,
            expires_at_ms,
        }))
    }

    async fn renew_lease(
        &self,
        lease: &StepLease,
        now_ms: u64,
        lease_ttl_ms: u64,
    ) -> ControlResult<bool> {
        validate_positive_ttl("validate_valkey_lease_ttl", lease_ttl_ms)?;
        let expires_at_ms = now_ms.saturating_add(lease_ttl_ms);
        let lease_key = self.config.lease_key(&lease.run_id, &lease.step_id);
        let lease_deadlines_key = self.config.lease_deadlines_key();
        let entry_id = step_entry_id(&lease.run_id, &lease.step_id);
        let result: i64 = self
            .run_command("valkey_hot_state_renew_lease", || {
                let mut command = redis::cmd("EVAL");
                command
                    .arg(RENEW_LEASE_LUA)
                    .arg(2)
                    .arg(&lease_key)
                    .arg(&lease_deadlines_key)
                    .arg(lease.lease_id.as_str())
                    .arg(lease.worker_id.as_str())
                    .arg(expires_at_ms.to_string())
                    .arg(lease_ttl_ms.to_string())
                    .arg(&entry_id);
                command
            })
            .await?;
        lease_script_result(result, &lease.lease_id, &lease.worker_id)
    }

    async fn release_lease(&self, lease: &StepLease) -> ControlResult<bool> {
        let lease_key = self.config.lease_key(&lease.run_id, &lease.step_id);
        let lease_deadlines_key = self.config.lease_deadlines_key();
        let entry_id = step_entry_id(&lease.run_id, &lease.step_id);
        let result: i64 = self
            .run_command("valkey_hot_state_release_lease", || {
                let mut command = redis::cmd("EVAL");
                command
                    .arg(RELEASE_LEASE_LUA)
                    .arg(2)
                    .arg(&lease_key)
                    .arg(&lease_deadlines_key)
                    .arg(lease.lease_id.as_str())
                    .arg(lease.worker_id.as_str())
                    .arg(&entry_id);
                command
            })
            .await?;
        lease_script_result(result, &lease.lease_id, &lease.worker_id)
    }

    async fn reclaim_expired_lease(&self, lease: &StepLease, now_ms: u64) -> ControlResult<bool> {
        let pending_queue_key = self.config.pending_queue_key();
        let lease_key = self.config.lease_key(&lease.run_id, &lease.step_id);
        let lease_deadlines_key = self.config.lease_deadlines_key();
        let payload_key = self.config.step_payload_key(&lease.run_id, &lease.step_id);
        let entry_id = step_entry_id(&lease.run_id, &lease.step_id);
        let result: i64 = self
            .run_command("valkey_hot_state_reclaim_expired_lease", || {
                let mut command = redis::cmd("EVAL");
                command
                    .arg(RECLAIM_EXPIRED_LEASE_LUA)
                    .arg(4)
                    .arg(&pending_queue_key)
                    .arg(&lease_key)
                    .arg(&lease_deadlines_key)
                    .arg(&payload_key)
                    .arg(lease.lease_id.as_str())
                    .arg(lease.worker_id.as_str())
                    .arg(&entry_id)
                    .arg(now_ms.to_string());
                command
            })
            .await?;
        lease_script_result(result, &lease.lease_id, &lease.worker_id)
    }

    async fn enqueue_activity_task(&self, task: RunnableActivityTask) -> ControlResult<()> {
        let payload_json = encode_runnable_activity_task(&task)?;
        let entry_id = activity_entry_id(
            &task.task.run_id,
            task.task.step_id.as_ref(),
            &task.task.activity_id,
        );
        let payload_key = self.config.activity_payload_key(
            &task.task.run_id,
            task.task.step_id.as_ref(),
            &task.task.activity_id,
        );
        let lease_key = self.config.activity_lease_key(
            &task.task.run_id,
            task.task.step_id.as_ref(),
            &task.task.activity_id,
        );
        let pending_queue_key = self.config.activity_pending_queue_key();
        let priority_score = priority_score(task.priority);
        let _: i64 = self
            .run_command("valkey_hot_state_enqueue_activity_task", || {
                let mut command = redis::cmd("EVAL");
                command
                    .arg(ENQUEUE_ACTIVITY_TASK_LUA)
                    .arg(3)
                    .arg(&pending_queue_key)
                    .arg(&payload_key)
                    .arg(&lease_key)
                    .arg(&payload_json)
                    .arg(priority_score.to_string())
                    .arg(task.not_before_ms.to_string())
                    .arg(task.task.task_queue.as_str())
                    .arg(&entry_id);
                command
            })
            .await?;
        Ok(())
    }

    async fn claim_activity_task(
        &self,
        worker: WorkerRef,
        task_queue: Option<&TaskQueue>,
        now_ms: u64,
        lease_ttl_ms: u64,
    ) -> ControlResult<Option<HotStateLeasedActivityTask>> {
        validate_positive_ttl("validate_valkey_activity_task_lease_ttl", lease_ttl_ms)?;
        let lease_id = self.next_activity_lease_id(&worker.worker_id, now_ms)?;
        let expires_at_ms = now_ms.saturating_add(lease_ttl_ms);
        let pending_queue_key = self.config.activity_pending_queue_key();
        let lease_deadlines_key = self.config.activity_lease_deadlines_key();
        let payload_prefix = self.config.activity_payload_prefix();
        let lease_prefix = self.config.activity_lease_prefix();
        let task_queue_filter = task_queue.map_or("", TaskQueue::as_str);
        let payload_json: Option<String> = self
            .run_command("valkey_hot_state_claim_activity_task", || {
                let mut command = redis::cmd("EVAL");
                command
                    .arg(CLAIM_ACTIVITY_TASK_LUA)
                    .arg(2)
                    .arg(&pending_queue_key)
                    .arg(&lease_deadlines_key)
                    .arg(&payload_prefix)
                    .arg(&lease_prefix)
                    .arg(now_ms.to_string())
                    .arg(lease_id.as_str())
                    .arg(worker.worker_id.as_str())
                    .arg(now_ms.to_string())
                    .arg(expires_at_ms.to_string())
                    .arg(lease_ttl_ms.to_string())
                    .arg(task_queue_filter);
                command
            })
            .await?;
        let Some(payload_json) = payload_json else {
            return Ok(None);
        };
        let activity_task = decode_runnable_activity_task(&payload_json)?;
        let lease = ActivityTaskLease {
            lease_id,
            run_id: activity_task.task.run_id.clone(),
            step_id: activity_task.task.step_id.clone(),
            activity_id: activity_task.task.activity_id.clone(),
            worker_id: worker.worker_id,
            acquired_at_ms: now_ms,
            expires_at_ms,
        };
        Ok(Some(HotStateLeasedActivityTask {
            activity_task,
            lease,
        }))
    }

    async fn release_activity_task_lease(&self, lease: &ActivityTaskLease) -> ControlResult<bool> {
        let lease_key = self.config.activity_lease_key(
            &lease.run_id,
            lease.step_id.as_ref(),
            &lease.activity_id,
        );
        let lease_deadlines_key = self.config.activity_lease_deadlines_key();
        let entry_id = activity_entry_id(&lease.run_id, lease.step_id.as_ref(), &lease.activity_id);
        let result: i64 = self
            .run_command("valkey_hot_state_release_activity_task_lease", || {
                let mut command = redis::cmd("EVAL");
                command
                    .arg(RELEASE_LEASE_LUA)
                    .arg(2)
                    .arg(&lease_key)
                    .arg(&lease_deadlines_key)
                    .arg(lease.lease_id.as_str())
                    .arg(lease.worker_id.as_str())
                    .arg(&entry_id);
                command
            })
            .await?;
        lease_script_result(result, &lease.lease_id, &lease.worker_id)
    }

    async fn reclaim_expired_activity_task_lease(
        &self,
        lease: &ActivityTaskLease,
        now_ms: u64,
    ) -> ControlResult<bool> {
        let pending_queue_key = self.config.activity_pending_queue_key();
        let lease_key = self.config.activity_lease_key(
            &lease.run_id,
            lease.step_id.as_ref(),
            &lease.activity_id,
        );
        let lease_deadlines_key = self.config.activity_lease_deadlines_key();
        let payload_key = self.config.activity_payload_key(
            &lease.run_id,
            lease.step_id.as_ref(),
            &lease.activity_id,
        );
        let entry_id = activity_entry_id(&lease.run_id, lease.step_id.as_ref(), &lease.activity_id);
        let result: i64 = self
            .run_command(
                "valkey_hot_state_reclaim_expired_activity_task_lease",
                || {
                    let mut command = redis::cmd("EVAL");
                    command
                        .arg(RECLAIM_EXPIRED_LEASE_LUA)
                        .arg(4)
                        .arg(&pending_queue_key)
                        .arg(&lease_key)
                        .arg(&lease_deadlines_key)
                        .arg(&payload_key)
                        .arg(lease.lease_id.as_str())
                        .arg(lease.worker_id.as_str())
                        .arg(&entry_id)
                        .arg(now_ms.to_string());
                    command
                },
            )
            .await?;
        lease_script_result(result, &lease.lease_id, &lease.worker_id)
    }

    async fn heartbeat(&self, heartbeat: WorkerHeartbeat) -> ControlResult<()> {
        let payload_json = encode_heartbeat(&heartbeat)?;
        let ttl_ms = heartbeat
            .expires_at_ms
            .saturating_sub(heartbeat.observed_at_ms);
        validate_positive_ttl("validate_valkey_worker_heartbeat_ttl", ttl_ms)?;
        let heartbeat_key = self.config.heartbeat_key(&heartbeat.worker_id);
        let _: String = self
            .run_command("valkey_hot_state_worker_heartbeat", || {
                let mut command = redis::cmd("SET");
                command
                    .arg(&heartbeat_key)
                    .arg(&payload_json)
                    .arg("PX")
                    .arg(ttl_ms);
                command
            })
            .await?;
        Ok(())
    }

    async fn load_heartbeat(&self, worker_id: &WorkerId) -> ControlResult<Option<WorkerHeartbeat>> {
        let heartbeat_key = self.config.heartbeat_key(worker_id);
        let payload_json: Option<String> = self
            .run_command("valkey_hot_state_load_worker_heartbeat", || {
                let mut command = redis::cmd("GET");
                command.arg(&heartbeat_key);
                command
            })
            .await?;
        payload_json
            .map(|payload| decode_heartbeat(&payload))
            .transpose()
    }

    async fn load_snapshot(&self, observed_at_ms: u64) -> ControlResult<HotStateSnapshot> {
        self.load_hot_state_snapshot(observed_at_ms).await
    }
}

impl ValkeyHotStateStore {
    async fn load_hot_state_snapshot(
        &self,
        observed_at_ms: u64,
    ) -> ControlResult<HotStateSnapshot> {
        let pending_entries = self
            .load_sorted_set_entries(
                "valkey_hot_state_snapshot_pending_entries",
                self.config.pending_queue_key(),
            )
            .await?;
        let lease_entries = self
            .load_sorted_set_entries(
                "valkey_hot_state_snapshot_lease_entries",
                self.config.lease_deadlines_key(),
            )
            .await?;
        let activity_pending_entries = self
            .load_sorted_set_entries(
                "valkey_hot_state_snapshot_activity_pending_entries",
                self.config.activity_pending_queue_key(),
            )
            .await?;
        let activity_lease_entries = self
            .load_sorted_set_entries(
                "valkey_hot_state_snapshot_activity_lease_entries",
                self.config.activity_lease_deadlines_key(),
            )
            .await?;
        let heartbeat_keys: Vec<String> = self
            .run_command("valkey_hot_state_snapshot_heartbeat_keys", || {
                let mut command = redis::cmd("KEYS");
                command.arg(self.config.heartbeat_key_pattern());
                command
            })
            .await?;

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

    async fn load_sorted_set_entries(
        &self,
        operation: &'static str,
        key: String,
    ) -> ControlResult<Vec<String>> {
        self.run_command(operation, || {
            let mut command = redis::cmd("ZRANGE");
            command.arg(&key).arg(0).arg(-1);
            command
        })
        .await
    }

    async fn append_pending_steps(
        &self,
        snapshot: &mut HotStateSnapshot,
        entries: Vec<String>,
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
        entries: Vec<String>,
    ) -> ControlResult<()> {
        for entry_id in entries {
            let Some(step) = self.load_step_payload_by_entry(&entry_id).await? else {
                continue;
            };
            let lease_key = self.config.lease_key_for_entry(&entry_id);
            let lease_hash: Vec<(String, String)> = self.load_hash(&lease_key).await?;
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
        entries: Vec<String>,
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
        entries: Vec<String>,
    ) -> ControlResult<()> {
        for entry_id in entries {
            let Some(activity_task) = self.load_activity_payload_by_entry(&entry_id).await? else {
                continue;
            };
            let lease_key = self.config.activity_lease_key_for_entry(&entry_id);
            let lease_hash: Vec<(String, String)> = self.load_hash(&lease_key).await?;
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
            let payload_json: Option<String> = self
                .run_command("valkey_hot_state_snapshot_heartbeat", || {
                    let mut command = redis::cmd("GET");
                    command.arg(&heartbeat_key);
                    command
                })
                .await?;
            if let Some(payload_json) = payload_json {
                snapshot
                    .worker_heartbeats
                    .push(decode_heartbeat(&payload_json)?);
            }
        }
        Ok(())
    }

    async fn load_hash(&self, key: &str) -> ControlResult<Vec<(String, String)>> {
        self.run_command("valkey_hot_state_snapshot_hash", || {
            let mut command = redis::cmd("HGETALL");
            command.arg(key);
            command
        })
        .await
    }

    async fn load_step_payload_by_entry(
        &self,
        entry_id: &str,
    ) -> ControlResult<Option<RunnableStep>> {
        let payload_key = self.config.step_payload_key_for_entry(entry_id);
        let payload_json: Option<String> = self
            .run_command("valkey_hot_state_snapshot_step_payload", || {
                let mut command = redis::cmd("HGET");
                command.arg(&payload_key).arg("payload");
                command
            })
            .await?;
        payload_json
            .map(|payload| decode_runnable_step(&payload))
            .transpose()
    }

    async fn load_activity_payload_by_entry(
        &self,
        entry_id: &str,
    ) -> ControlResult<Option<RunnableActivityTask>> {
        let payload_key = self.config.activity_payload_key_for_entry(entry_id);
        let payload_json: Option<String> = self
            .run_command("valkey_hot_state_snapshot_activity_task_payload", || {
                let mut command = redis::cmd("HGET");
                command.arg(&payload_key).arg("payload");
                command
            })
            .await?;
        payload_json
            .map(|payload| decode_runnable_activity_task(&payload))
            .transpose()
    }
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
        operation: "decode_valkey_step_lease_hash",
        message: format!("missing lease hash field `{field}`"),
    })
}

fn parse_hash_u64(value: Option<&str>, field: &'static str) -> ControlResult<u64> {
    required_hash_value(value, field)?
        .parse()
        .map_err(|error: std::num::ParseIntError| ControlError::Codec {
            operation: "decode_valkey_step_lease_hash",
            message: format!("invalid `{field}`: {error}"),
        })
}

fn lease_script_result(
    result: i64,
    lease_id: &LeaseId,
    worker_id: &WorkerId,
) -> ControlResult<bool> {
    match result {
        1 => Ok(true),
        0 => Ok(false),
        -1 => Err(ControlError::LeaseNotOwned {
            lease_id: lease_id.clone(),
            worker_id: worker_id.clone(),
        }),
        unexpected => Err(ControlError::Storage {
            operation: "decode_valkey_lease_script_result",
            message: format!("unexpected lease script result {unexpected}"),
        }),
    }
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

fn priority_score(priority: i64) -> i64 {
    priority.saturating_neg()
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

fn validate_positive_ttl(operation: &'static str, ttl_ms: u64) -> ControlResult<()> {
    if ttl_ms == 0 {
        return Err(ControlError::Storage {
            operation,
            message: "ttl_ms must be greater than zero".to_owned(),
        });
    }
    Ok(())
}
