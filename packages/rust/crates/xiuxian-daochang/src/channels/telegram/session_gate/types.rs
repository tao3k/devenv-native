//! Telegram session gate state and lease types.

use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex as StdMutex;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use tokio::sync::{Mutex, OwnedMutexGuard};

use super::config::{SessionGateBackendMode, SessionGateRuntimeConfig};
use super::valkey::{DistributedLeaseGuard, ValkeySessionGateBackend};

/// Session-scoped foreground gate.
///
/// Calls sharing the same session id are serialized, while unrelated session
/// ids can proceed concurrently.
#[derive(Clone)]
pub struct SessionGate {
    pub(super) inner: Arc<StdMutex<HashMap<String, Arc<SessionGateEntry>>>>,
    pub(super) backend: SessionGateBackend,
}

#[derive(Default)]
pub(super) struct SessionGateEntry {
    pub(super) lock: Arc<Mutex<()>>,
    pub(super) permits: AtomicUsize,
}

/// Active session-gate guard that releases local or valkey-backed leases on drop.
pub struct SessionGuard {
    pub(super) _distributed_lease: Option<DistributedLeaseGuard>,
    pub(super) _lock_guard: OwnedMutexGuard<()>,
    pub(super) _permit: SessionPermit,
}

pub(super) struct SessionPermit {
    pub(super) session_id: String,
    pub(super) inner: Arc<StdMutex<HashMap<String, Arc<SessionGateEntry>>>>,
    pub(super) entry: Arc<SessionGateEntry>,
}

#[derive(Clone)]
pub(super) enum SessionGateBackend {
    Memory,
    Valkey(Arc<ValkeySessionGateBackend>),
}

impl Drop for SessionPermit {
    fn drop(&mut self) {
        let previous = self.entry.permits.fetch_sub(1, Ordering::AcqRel);
        debug_assert!(previous > 0, "session gate permit underflow");
        if previous != 1 {
            return;
        }

        let mut map = self
            .inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let should_remove = map
            .get(&self.session_id)
            .is_some_and(|current| Arc::ptr_eq(current, &self.entry))
            && self.entry.permits.load(Ordering::Acquire) == 0;
        if should_remove {
            map.remove(&self.session_id);
        }
    }
}

impl SessionGate {
    /// Build an in-memory gate with no distributed coordination backend.
    #[must_use]
    pub fn new_memory() -> Self {
        Self {
            inner: Arc::new(StdMutex::new(HashMap::new())),
            backend: SessionGateBackend::Memory,
        }
    }

    /// Build the gate from the current runtime environment and config.
    ///
    /// # Errors
    ///
    /// Returns an error when session-gate configuration is invalid.
    pub fn from_env() -> Result<Self> {
        let runtime_config = SessionGateRuntimeConfig::from_env()?;
        let backend = match runtime_config.backend_mode {
            SessionGateBackendMode::Auto | SessionGateBackendMode::Memory => {
                SessionGateBackend::Memory
            }
            SessionGateBackendMode::Valkey => {
                let backend = runtime_config
                    .valkey_url
                    .as_deref()
                    .map(|valkey_url| {
                        ValkeySessionGateBackend::new(
                            valkey_url,
                            runtime_config.key_prefix.as_str(),
                            runtime_config.lease_ttl_secs,
                            runtime_config.acquire_timeout_secs,
                        )
                    })
                    .transpose()?
                    .map(Arc::new);
                match backend {
                    Some(backend) => SessionGateBackend::Valkey(backend),
                    None => SessionGateBackend::Memory,
                }
            }
        };

        Ok(Self {
            inner: Arc::new(StdMutex::new(HashMap::new())),
            backend,
        })
    }

    /// Build a Valkey-backed gate for integration tests.
    ///
    /// # Errors
    ///
    /// Returns an error when the provided Valkey URL is invalid.
    pub fn new_with_valkey_for_test(
        valkey_url: &str,
        key_prefix: &str,
        lease_ttl_secs: u64,
        acquire_timeout_secs: Option<u64>,
    ) -> Result<Self> {
        let backend = ValkeySessionGateBackend::new(
            valkey_url,
            key_prefix,
            lease_ttl_secs,
            acquire_timeout_secs,
        )?;
        Ok(Self {
            inner: Arc::new(StdMutex::new(HashMap::new())),
            backend: SessionGateBackend::Valkey(Arc::new(backend)),
        })
    }

    /// Report the active backend label.
    #[must_use]
    pub fn backend_name(&self) -> &'static str {
        match self.backend {
            SessionGateBackend::Memory => "memory",
            SessionGateBackend::Valkey(_) => "valkey",
        }
    }

    /// Acquire the session gate for one logical session.
    ///
    /// # Errors
    ///
    /// Returns an error when distributed lease acquisition fails.
    pub async fn acquire(&self, session_id: impl AsRef<str>) -> Result<SessionGuard> {
        let session_id = session_id.as_ref();
        let distributed_lease = match &self.backend {
            SessionGateBackend::Memory => None,
            SessionGateBackend::Valkey(backend) => Some(backend.acquire_lease(session_id).await?),
        };

        let entry = {
            let mut guard = self
                .inner
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            guard
                .entry(session_id.to_string())
                .or_insert_with(|| Arc::new(SessionGateEntry::default()))
                .clone()
        };

        entry.permits.fetch_add(1, Ordering::AcqRel);
        let lock_guard = entry.lock.clone().lock_owned().await;
        Ok(SessionGuard {
            _distributed_lease: distributed_lease,
            _lock_guard: lock_guard,
            _permit: SessionPermit {
                session_id: session_id.to_string(),
                inner: Arc::clone(&self.inner),
                entry,
            },
        })
    }

    /// Return the number of active session entries tracked in memory.
    #[doc(hidden)]
    #[must_use]
    pub fn active_sessions(&self) -> usize {
        self.inner
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }
}

impl Default for SessionGate {
    fn default() -> Self {
        Self::new_memory()
    }
}
