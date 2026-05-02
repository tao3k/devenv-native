//! Valkey session-gate state types.

use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

use tokio::sync::{Mutex, RwLock, oneshot};

pub(in crate::channels::telegram::session_gate) const DEFAULT_GATE_RETRY_INTERVAL_MS: u64 = 25;

pub(in crate::channels::telegram::session_gate) static NEXT_LEASE_OWNER_ID: AtomicU64 =
    AtomicU64::new(1);

#[derive(Clone)]
pub(in crate::channels::telegram::session_gate) struct ValkeySessionGateBackend {
    pub(super) client: redis::Client,
    pub(super) key_prefix: String,
    pub(super) lease_ttl_ms: u64,
    pub(super) acquire_timeout: Option<Duration>,
    pub(super) retry_interval: Duration,
    pub(super) connection: Arc<RwLock<Option<redis::aio::MultiplexedConnection>>>,
    pub(super) reconnect_lock: Arc<Mutex<()>>,
}

pub(in crate::channels::telegram::session_gate) struct DistributedLeaseGuard {
    pub(super) backend: Arc<ValkeySessionGateBackend>,
    pub(super) lock_key: String,
    pub(super) owner_token: String,
    pub(super) stop_tx: Option<oneshot::Sender<()>>,
}
