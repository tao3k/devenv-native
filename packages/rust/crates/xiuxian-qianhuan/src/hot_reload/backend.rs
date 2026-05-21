//! Hot-reload version backend abstractions and in-memory implementations.

use anyhow::Result;
use std::collections::BTreeMap;
use std::sync::Mutex;

/// Version backend abstraction used to synchronize reload state across
/// multiple runtime instances.
pub trait HotReloadVersionBackend: Send + Sync {
    /// Reads the current version for a target identifier.
    ///
    /// Returning `None` means the target has never been versioned.
    ///
    /// # Errors
    ///
    /// Returns an error when the backend read operation fails.
    fn read_version(&self, target_id: &str) -> Result<Option<u64>>;

    /// Atomically increments and returns the new version for a target
    /// identifier.
    ///
    /// # Errors
    ///
    /// Returns an error when the backend update operation fails.
    fn bump_version(&self, target_id: &str) -> Result<u64>;
}

/// In-memory version backend for local testing and single-process usage.
#[derive(Default)]
pub struct InMemoryHotReloadVersionBackend {
    versions: Mutex<BTreeMap<String, u64>>,
}

/// Hot-reload target identifier used by test and local synchronization helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HotReloadTargetId(String);

impl HotReloadTargetId {
    /// Creates a hot-reload target identifier.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl AsRef<str> for HotReloadTargetId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl InMemoryHotReloadVersionBackend {
    /// Sets an explicit version value.
    ///
    /// This utility is intended for tests that emulate remote version bumps.
    ///
    /// # Errors
    ///
    /// Returns an error when the in-memory lock is poisoned.
    pub fn set_version(&self, target_id: &HotReloadTargetId, version: u64) -> Result<()> {
        let mut guard = self
            .versions
            .lock()
            .map_err(|_| anyhow::anyhow!("hot reload version backend lock poisoned"))?;
        guard.insert(target_id.as_ref().to_string(), version);
        Ok(())
    }
}

impl HotReloadVersionBackend for InMemoryHotReloadVersionBackend {
    fn read_version(&self, target_id: &str) -> Result<Option<u64>> {
        let guard = self
            .versions
            .lock()
            .map_err(|_| anyhow::anyhow!("hot reload version backend lock poisoned"))?;
        Ok(guard.get(target_id).copied())
    }

    fn bump_version(&self, target_id: &str) -> Result<u64> {
        let mut guard = self
            .versions
            .lock()
            .map_err(|_| anyhow::anyhow!("hot reload version backend lock poisoned"))?;
        let next = guard.get(target_id).copied().unwrap_or(0).saturating_add(1);
        guard.insert(target_id.to_string(), next);
        Ok(next)
    }
}

/// Valkey-backed version store for cross-process hot-reload synchronization.
pub struct ValkeyHotReloadVersionBackend {
    client: redis::Client,
    key_prefix: String,
}

impl ValkeyHotReloadVersionBackend {
    const DEFAULT_KEY_PREFIX: &str = "xiuxian_hot_reload";

    /// Creates a Valkey backend from connection URL and optional key prefix.
    ///
    /// # Errors
    ///
    /// Returns an error when URL parsing fails.
    pub fn new(valkey_url: &str, key_prefix: Option<&str>) -> Result<Self> {
        let client = redis::Client::open(valkey_url)
            .map_err(|error| anyhow::anyhow!("invalid valkey url: {error}"))?;
        let resolved_prefix = key_prefix
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(Self::DEFAULT_KEY_PREFIX)
            .to_string();
        Ok(Self {
            client,
            key_prefix: resolved_prefix,
        })
    }

    fn key_for_target(&self, target_id: &str) -> String {
        format!("{}:version:{target_id}", self.key_prefix)
    }
}

impl HotReloadVersionBackend for ValkeyHotReloadVersionBackend {
    fn read_version(&self, target_id: &str) -> Result<Option<u64>> {
        let mut conn = self
            .client
            .get_connection()
            .map_err(|error| anyhow::anyhow!("failed to connect valkey: {error}"))?;
        let key = self.key_for_target(target_id);
        let value: Option<u64> = redis::cmd("GET")
            .arg(&key)
            .query(&mut conn)
            .map_err(|error| {
                anyhow::anyhow!("failed to read valkey hot reload version: {error}")
            })?;
        Ok(value)
    }

    fn bump_version(&self, target_id: &str) -> Result<u64> {
        let mut conn = self
            .client
            .get_connection()
            .map_err(|error| anyhow::anyhow!("failed to connect valkey: {error}"))?;
        let key = self.key_for_target(target_id);
        let value: u64 = redis::cmd("INCR")
            .arg(&key)
            .query(&mut conn)
            .map_err(|error| {
                anyhow::anyhow!("failed to bump valkey hot reload version: {error}")
            })?;
        Ok(value)
    }
}
