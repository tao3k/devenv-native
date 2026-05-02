//! Memory state persistence backends.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use anyhow::Result;

use crate::{EpisodeStore, StoreConfig};

/// Persistence abstraction for memory state (episodes + Q-values).
pub trait MemoryStateStore: Send + Sync {
    /// Backend identifier for logs and metrics.
    fn backend_name(&self) -> &'static str;

    /// Whether startup should fail if loading state fails.
    fn strict_startup(&self) -> bool {
        false
    }

    /// Load state into `store`.
    ///
    /// # Errors
    ///
    /// Returns an error when backend state cannot be loaded or decoded.
    fn load(&self, store: &mut EpisodeStore) -> Result<()>;

    /// Save state from `store`.
    ///
    /// # Errors
    ///
    /// Returns an error when backend state cannot be serialized or persisted.
    fn save(&self, store: &EpisodeStore) -> Result<()>;

    /// Persist one episode Q-value atomically when supported by backend.
    ///
    /// Backends that only support coarse-grained snapshots can keep the default
    /// no-op implementation.
    ///
    /// # Errors
    ///
    /// Returns an error when a backend-specific atomic write fails.
    fn update_q_atomic(&self, _episode_id: &str, _new_q: f32) -> Result<()> {
        Ok(())
    }

    /// Persist one scope-level recall feedback bias atomically when supported by backend.
    ///
    /// Backends that only support coarse-grained snapshots can keep the default
    /// no-op implementation.
    ///
    /// # Errors
    ///
    /// Returns an error when a backend-specific atomic write fails.
    fn update_scope_feedback_bias_atomic(&self, _scope: &str, _new_bias: f32) -> Result<()> {
        Ok(())
    }

    /// Delete one scope-level recall feedback bias atomically when supported by backend.
    ///
    /// # Errors
    ///
    /// Returns an error when a backend-specific atomic delete fails.
    fn clear_scope_feedback_bias_atomic(&self, _scope: &str) -> Result<()> {
        Ok(())
    }
}

/// Local JSON-backed memory state store.
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalMemoryStateStore;

impl LocalMemoryStateStore {
    /// Create a local filesystem-backed memory state store.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl MemoryStateStore for LocalMemoryStateStore {
    fn backend_name(&self) -> &'static str {
        "local"
    }

    fn load(&self, store: &mut EpisodeStore) -> Result<()> {
        store.load_state()
    }

    fn save(&self, store: &EpisodeStore) -> Result<()> {
        store.save_state()
    }
}

/// Build a deterministic Valkey key from prefix + store identity.
#[must_use]
pub fn default_valkey_state_key(prefix: &str, store_config: &StoreConfig) -> String {
    let mut hasher = DefaultHasher::new();
    store_config.path.hash(&mut hasher);
    let path_fingerprint = hasher.finish();
    format!("{prefix}:{path_fingerprint}:{}", store_config.table_name)
}

/// Build deterministic Valkey hash keys for episodes and Q-table fields.
#[must_use]
pub fn default_valkey_state_hash_keys(base_key: &str) -> (String, String) {
    (
        format!("{base_key}:episodes"),
        format!("{base_key}:q_values"),
    )
}

/// Build deterministic Valkey hash key for session recall feedback bias values.
#[must_use]
pub fn default_valkey_recall_feedback_hash_key(base_key: &str) -> String {
    format!("{base_key}:recall_feedback")
}

#[cfg(feature = "valkey")]
mod valkey;

#[cfg(feature = "valkey")]
pub use valkey::ValkeyMemoryStateStore;
