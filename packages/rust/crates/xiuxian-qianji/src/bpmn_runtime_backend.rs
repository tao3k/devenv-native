#[cfg(feature = "duckdb")]
use super::data_store::{QianjiBpmnDuckDbDataStore, QianjiBpmnDuckDbDataStoreConfig};
use super::error::BpmnOrchestrationError;
use crate::runtime_config::QianjiRuntimeCheckpointConfig;
use qianji_bpmn_engine::{
    BpmnCheckpointEnvelope, delete_checkpoint, delete_checkpoint_as_owner, load_checkpoint,
    release_checkpoint_lease, renew_checkpoint_lease, save_checkpoint, save_checkpoint_as_owner,
    try_acquire_checkpoint_lease,
};
#[cfg(feature = "duckdb")]
use std::path::Path;
#[cfg(any(feature = "duckdb", feature = "sqlite"))]
use std::path::PathBuf;

/// Host-owned checkpoint store facade for BPMN runtime sessions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QianjiBpmnCheckpointStore {
    /// Valkey-backed distributed checkpoint storage.
    Valkey {
        /// Resolved Valkey connection URL.
        url: String,
    },
    /// Lightweight local `SQLite` checkpoint storage.
    #[cfg(feature = "sqlite")]
    Sqlite {
        /// Filesystem path to the `SQLite` checkpoint database.
        path: PathBuf,
    },
    /// Local no-server `DuckDB` workflow-state snapshot storage.
    #[cfg(feature = "duckdb")]
    DuckDb {
        /// Filesystem path to the `DuckDB` workflow-state database.
        path: PathBuf,
    },
}

impl QianjiBpmnCheckpointStore {
    /// Returns the human-readable checkpoint backend name.
    #[must_use]
    pub fn backend_name(&self) -> &'static str {
        match self {
            Self::Valkey { .. } => "valkey",
            #[cfg(feature = "sqlite")]
            Self::Sqlite { .. } => "sqlite",
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

    /// Creates one SQLite-backed checkpoint store.
    #[cfg(feature = "sqlite")]
    #[must_use]
    pub fn sqlite(path: impl Into<PathBuf>) -> Self {
        Self::Sqlite { path: path.into() }
    }

    /// Creates one local `DuckDB` workflow-state store.
    #[cfg(feature = "duckdb")]
    #[must_use]
    pub fn duckdb(path: impl Into<PathBuf>) -> Self {
        Self::DuckDb { path: path.into() }
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
            #[cfg(feature = "sqlite")]
            Self::Sqlite { path } => {
                qianji_bpmn_engine::load_checkpoint_sql(instance_id, path).map_err(Into::into)
            }
            #[cfg(feature = "duckdb")]
            Self::DuckDb { path } => open_duckdb_workflow_state_store(path)?
                .load_workflow_state(instance_id)
                .map_err(Into::into),
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
            #[cfg(feature = "sqlite")]
            Self::Sqlite { path } => {
                qianji_bpmn_engine::save_checkpoint_sql(checkpoint, path).map_err(Into::into)
            }
            #[cfg(feature = "duckdb")]
            Self::DuckDb { path } => open_duckdb_workflow_state_store(path)?
                .upsert_workflow_state(checkpoint)
                .map_err(Into::into),
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
            #[cfg(feature = "sqlite")]
            Self::Sqlite { .. } => Err(BpmnOrchestrationError::CheckpointLeaseUnsupportedBackend {
                backend: self.backend_name().to_string(),
            }),
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
            #[cfg(feature = "sqlite")]
            Self::Sqlite { path } => {
                qianji_bpmn_engine::delete_checkpoint_sql(instance_id, path).map_err(Into::into)
            }
            #[cfg(feature = "duckdb")]
            Self::DuckDb { path } => open_duckdb_workflow_state_store(path)?
                .delete_workflow_state(instance_id)
                .map(|_| ())
                .map_err(Into::into),
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
            #[cfg(feature = "sqlite")]
            Self::Sqlite { .. } => Err(BpmnOrchestrationError::CheckpointLeaseUnsupportedBackend {
                backend: self.backend_name().to_string(),
            }),
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
            #[cfg(feature = "sqlite")]
            Self::Sqlite { .. } => Err(BpmnOrchestrationError::CheckpointLeaseUnsupportedBackend {
                backend: self.backend_name().to_string(),
            }),
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
            #[cfg(feature = "sqlite")]
            Self::Sqlite { .. } => Err(BpmnOrchestrationError::CheckpointLeaseUnsupportedBackend {
                backend: self.backend_name().to_string(),
            }),
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
            #[cfg(feature = "sqlite")]
            Self::Sqlite { .. } => Err(BpmnOrchestrationError::CheckpointLeaseUnsupportedBackend {
                backend: self.backend_name().to_string(),
            }),
            #[cfg(feature = "duckdb")]
            Self::DuckDb { .. } => Err(BpmnOrchestrationError::CheckpointLeaseUnsupportedBackend {
                backend: self.backend_name().to_string(),
            }),
        }
    }
}

#[cfg(feature = "duckdb")]
fn open_duckdb_workflow_state_store(
    path: &Path,
) -> Result<QianjiBpmnDuckDbDataStore, BpmnOrchestrationError> {
    QianjiBpmnDuckDbDataStore::open(QianjiBpmnDuckDbDataStoreConfig::file(path.to_path_buf()))
        .map_err(Into::into)
}
