#[cfg(feature = "duckdb")]
use super::duckdb;

use crate::BpmnOrchestrationError;
use crate::runtime_config::QianjiRuntimeCheckpointConfig;
use qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, delete_checkpoint, delete_checkpoint_as_owner, load_checkpoint,
    release_checkpoint_lease, renew_checkpoint_lease, save_checkpoint, save_checkpoint_as_owner,
    try_acquire_checkpoint_lease,
};
#[cfg(feature = "duckdb")]
use std::collections::HashMap;
#[cfg(feature = "duckdb")]
use std::path::PathBuf;
#[cfg(feature = "duckdb")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "duckdb")]
use xiuxian_db_store::qianji_bpmn::QianjiBpmnDuckDbDataStore;

#[cfg(feature = "duckdb")]
use duckdb::{
    cache_duckdb_checkpoint, cache_duckdb_checkpoints, cached_duckdb_checkpoint,
    cached_duckdb_checkpoint_for_store, hydrate_duckdb_latest_cache,
    remove_cached_duckdb_checkpoint, with_duckdb_workflow_state_store,
};

/// Host-owned checkpoint store facade for BPMN runtime sessions.
#[derive(Clone)]
pub enum QianjiBpmnCheckpointStore {
    /// Valkey-backed distributed checkpoint storage.
    Valkey {
        /// Resolved Valkey connection URL.
        url: String,
    },
    /// Local no-server `DuckDB` workflow-state snapshot storage.
    #[cfg(feature = "duckdb")]
    DuckDb {
        /// Filesystem path to the `DuckDB` workflow-state database.
        path: PathBuf,
        /// Lazily opened local workflow-state store reused across checkpoint operations.
        store: Arc<Mutex<Option<QianjiBpmnDuckDbDataStore>>>,
        /// Same-process latest-checkpoint cache for hot save/load loops.
        latest_cache: Arc<Mutex<HashMap<String, BpmnCheckpointEnvelope>>>,
        /// Whether the latest cache has been hydrated from compacted `DuckDB` state.
        latest_cache_hydrated: Arc<Mutex<bool>>,
    },
}

impl std::fmt::Debug for QianjiBpmnCheckpointStore {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Valkey { url } => formatter.debug_struct("Valkey").field("url", url).finish(),
            #[cfg(feature = "duckdb")]
            Self::DuckDb { path, .. } => formatter
                .debug_struct("DuckDb")
                .field("path", path)
                .finish_non_exhaustive(),
        }
    }
}

impl PartialEq for QianjiBpmnCheckpointStore {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Valkey { url: left }, Self::Valkey { url: right }) => left == right,
            #[cfg(feature = "duckdb")]
            (Self::DuckDb { path: left, .. }, Self::DuckDb { path: right, .. }) => left == right,
            #[cfg(feature = "duckdb")]
            (Self::Valkey { .. }, Self::DuckDb { .. })
            | (Self::DuckDb { .. }, Self::Valkey { .. }) => false,
        }
    }
}

impl Eq for QianjiBpmnCheckpointStore {}

impl QianjiBpmnCheckpointStore {
    /// Returns the human-readable checkpoint backend name.
    #[must_use]
    pub fn backend_name(&self) -> &'static str {
        match self {
            Self::Valkey { .. } => "valkey",
            #[cfg(feature = "duckdb")]
            Self::DuckDb { .. } => "duckdb",
        }
    }

    /// Creates one Valkey-backed checkpoint store from resolved runtime config.
    #[must_use]
    pub fn from_runtime_checkpoint_config(config: &QianjiRuntimeCheckpointConfig) -> Self {
        Self::Valkey {
            url: config.valkey_url.clone(),
        }
    }

    /// Creates one Valkey-backed checkpoint store directly from its URL.
    #[must_use]
    pub fn valkey(url: impl Into<String>) -> Self {
        Self::Valkey { url: url.into() }
    }

    /// Creates one local `DuckDB` workflow-state store.
    #[cfg(feature = "duckdb")]
    #[must_use]
    pub fn duckdb(path: impl Into<PathBuf>) -> Self {
        Self::DuckDb {
            path: path.into(),
            store: Arc::new(Mutex::new(None)),
            latest_cache: Arc::new(Mutex::new(HashMap::new())),
            latest_cache_hydrated: Arc::new(Mutex::new(false)),
        }
    }

    /// Loads one checkpoint envelope for the supplied BPMN instance id.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the underlying checkpoint
    /// backend cannot load or decode the checkpoint.
    pub async fn load(
        &self,
        instance_id: &str,
    ) -> Result<Option<BpmnCheckpointEnvelope>, BpmnOrchestrationError> {
        match self {
            Self::Valkey { url } => load_checkpoint(instance_id, url).await.map_err(Into::into),
            #[cfg(feature = "duckdb")]
            Self::DuckDb {
                path,
                store,
                latest_cache,
                latest_cache_hydrated,
            } => {
                if let Some(checkpoint) = cached_duckdb_checkpoint(latest_cache, instance_id)? {
                    return Ok(Some(checkpoint));
                }
                let loaded = with_duckdb_workflow_state_store(path, store, |store| {
                    hydrate_duckdb_latest_cache(latest_cache, latest_cache_hydrated, store)?;
                    let mut loaded = cached_duckdb_checkpoint_for_store(latest_cache, instance_id)?;
                    loaded = newest_duckdb_checkpoint(
                        loaded,
                        store.load_compacted_workflow_state_snapshot(instance_id)?,
                    );
                    loaded = newest_duckdb_checkpoint(
                        loaded,
                        store.load_latest_workflow_state_snapshot(instance_id)?,
                    );
                    loaded =
                        newest_duckdb_checkpoint(loaded, store.load_workflow_state(instance_id)?);
                    Ok(loaded)
                })?;
                if let Some(checkpoint) = loaded.as_ref() {
                    cache_duckdb_checkpoint(latest_cache, checkpoint)?;
                }
                Ok(loaded)
            }
        }
    }

    /// Lists latest checkpoint envelopes known to the configured backend.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the backend cannot enumerate
    /// local checkpoint state or when decoding any stored checkpoint fails.
    pub fn list(&self) -> Result<Vec<BpmnCheckpointEnvelope>, BpmnOrchestrationError> {
        match self {
            Self::Valkey { .. } => Err(BpmnOrchestrationError::CheckpointListUnsupportedBackend {
                backend: self.backend_name().to_string(),
            }),
            #[cfg(feature = "duckdb")]
            Self::DuckDb {
                path,
                store,
                latest_cache,
                latest_cache_hydrated,
            } => {
                let checkpoints = with_duckdb_workflow_state_store(path, store, |store| {
                    store.compact_workflow_state_latest_snapshots()?;
                    store.load_compacted_workflow_state_snapshots()
                })?;
                cache_duckdb_checkpoints(latest_cache, latest_cache_hydrated, &checkpoints)?;
                Ok(checkpoints)
            }
        }
    }

    /// Saves one checkpoint envelope to the configured backend.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the underlying checkpoint
    /// backend cannot persist the checkpoint envelope.
    pub async fn save(
        &self,
        checkpoint: &BpmnCheckpointEnvelope,
    ) -> Result<(), BpmnOrchestrationError> {
        match self {
            Self::Valkey { url } => save_checkpoint(checkpoint, url).await.map_err(Into::into),
            #[cfg(feature = "duckdb")]
            Self::DuckDb {
                path,
                store,
                latest_cache,
                ..
            } => {
                with_duckdb_workflow_state_store(path, store, |store| {
                    store.append_workflow_state_snapshot(checkpoint)?;
                    store.upsert_latest_workflow_state_snapshot(checkpoint)
                })?;
                cache_duckdb_checkpoint(latest_cache, checkpoint)
            }
        }
    }

    /// Saves one checkpoint envelope to Valkey when the caller owns the lease.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the underlying backend cannot
    /// persist the checkpoint or when the checkpoint backend is not Valkey.
    pub async fn save_as_owner(
        &self,
        checkpoint: &BpmnCheckpointEnvelope,
        owner_token: &str,
    ) -> Result<(), BpmnOrchestrationError> {
        match self {
            Self::Valkey { url } => save_checkpoint_as_owner(checkpoint, owner_token, url)
                .await
                .map_err(Into::into),
            #[cfg(feature = "duckdb")]
            Self::DuckDb { .. } => Err(BpmnOrchestrationError::CheckpointLeaseUnsupportedBackend {
                backend: self.backend_name().to_string(),
            }),
        }
    }

    /// Deletes one checkpoint envelope for the supplied BPMN instance id.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the underlying checkpoint
    /// backend cannot delete the checkpoint state.
    pub async fn delete(&self, instance_id: &str) -> Result<(), BpmnOrchestrationError> {
        match self {
            Self::Valkey { url } => delete_checkpoint(instance_id, url)
                .await
                .map_err(Into::into),
            #[cfg(feature = "duckdb")]
            Self::DuckDb {
                path,
                store,
                latest_cache,
                ..
            } => {
                with_duckdb_workflow_state_store(path, store, |store| {
                    store.delete_workflow_state_snapshots(instance_id)?;
                    store.delete_workflow_state(instance_id).map(|_| ())
                })?;
                remove_cached_duckdb_checkpoint(latest_cache, instance_id)
            }
        }
    }

    /// Deletes one checkpoint envelope when the caller owns the Valkey lease.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when the underlying backend cannot
    /// delete the checkpoint or when the checkpoint backend is not Valkey.
    pub async fn delete_as_owner(
        &self,
        instance_id: &str,
        owner_token: &str,
    ) -> Result<(), BpmnOrchestrationError> {
        match self {
            Self::Valkey { url } => delete_checkpoint_as_owner(instance_id, owner_token, url)
                .await
                .map_err(Into::into),
            #[cfg(feature = "duckdb")]
            Self::DuckDb { .. } => Err(BpmnOrchestrationError::CheckpointLeaseUnsupportedBackend {
                backend: self.backend_name().to_string(),
            }),
        }
    }

    /// Tries to acquire one Valkey checkpoint lease for the supplied BPMN
    /// instance id.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when lease acquisition fails or when
    /// the checkpoint backend is not Valkey.
    pub async fn try_acquire_lease(
        &self,
        instance_id: &str,
        owner_token: &str,
        lease_ttl_ms: u64,
    ) -> Result<bool, BpmnOrchestrationError> {
        match self {
            Self::Valkey { url } => {
                try_acquire_checkpoint_lease(instance_id, owner_token, lease_ttl_ms, url)
                    .await
                    .map_err(Into::into)
            }
            #[cfg(feature = "duckdb")]
            Self::DuckDb { .. } => Err(BpmnOrchestrationError::CheckpointLeaseUnsupportedBackend {
                backend: self.backend_name().to_string(),
            }),
        }
    }

    /// Renews one Valkey checkpoint lease for the supplied BPMN instance id.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when lease renewal fails or when the
    /// checkpoint backend is not Valkey.
    pub async fn renew_lease(
        &self,
        instance_id: &str,
        owner_token: &str,
        lease_ttl_ms: u64,
    ) -> Result<bool, BpmnOrchestrationError> {
        match self {
            Self::Valkey { url } => {
                renew_checkpoint_lease(instance_id, owner_token, lease_ttl_ms, url)
                    .await
                    .map_err(Into::into)
            }
            #[cfg(feature = "duckdb")]
            Self::DuckDb { .. } => Err(BpmnOrchestrationError::CheckpointLeaseUnsupportedBackend {
                backend: self.backend_name().to_string(),
            }),
        }
    }

    /// Releases one Valkey checkpoint lease for the supplied BPMN instance id.
    ///
    /// # Errors
    ///
    /// Returns [`BpmnOrchestrationError`] when lease release fails or when the
    /// checkpoint backend is not Valkey.
    pub async fn release_lease(
        &self,
        instance_id: &str,
        owner_token: &str,
    ) -> Result<bool, BpmnOrchestrationError> {
        match self {
            Self::Valkey { url } => release_checkpoint_lease(instance_id, owner_token, url)
                .await
                .map_err(Into::into),
            #[cfg(feature = "duckdb")]
            Self::DuckDb { .. } => Err(BpmnOrchestrationError::CheckpointLeaseUnsupportedBackend {
                backend: self.backend_name().to_string(),
            }),
        }
    }
}

#[cfg(feature = "duckdb")]
fn newest_duckdb_checkpoint(
    left: Option<BpmnCheckpointEnvelope>,
    right: Option<BpmnCheckpointEnvelope>,
) -> Option<BpmnCheckpointEnvelope> {
    match (left, right) {
        (None, None) => None,
        (Some(checkpoint), None) | (None, Some(checkpoint)) => Some(checkpoint),
        (Some(left), Some(right)) => {
            let right_is_newer = right.sequence > left.sequence
                || (right.sequence == left.sequence
                    && right.state.updated_at_ms >= left.state.updated_at_ms);
            Some(if right_is_newer { right } else { left })
        }
    }
}
