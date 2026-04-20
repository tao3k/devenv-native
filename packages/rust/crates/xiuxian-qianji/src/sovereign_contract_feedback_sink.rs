//! Sovereign sinks for persisting contract-feedback knowledge entries into Wendao surfaces.

use async_trait::async_trait;
use std::fmt;
use std::sync::{Mutex, MutexGuard, PoisonError};
use xiuxian_wendao::storage::KnowledgeStorage;
use xiuxian_wendao_core::KnowledgeEntry;

/// Sink trait for persisting contract-feedback knowledge entries.
#[async_trait]
pub trait ContractFeedbackKnowledgeSink: Send + Sync + fmt::Debug {
    /// Persist one batch of contract-feedback knowledge entries.
    ///
    /// # Errors
    ///
    /// Returns an error string when the target persistence surface cannot store one or more
    /// entries.
    async fn persist_entries(&self, entries: &[KnowledgeEntry]) -> Result<Vec<String>, String>;
}

/// Wendao-backed sink that stores contract-feedback entries through `KnowledgeStorage`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeStorageContractFeedbackSink {
    storage_path: String,
    table_name: String,
}

impl KnowledgeStorageContractFeedbackSink {
    /// Create a new storage-backed contract-feedback sink.
    #[must_use]
    pub fn new(storage_path: impl Into<String>, table_name: impl Into<String>) -> Self {
        Self {
            storage_path: storage_path.into(),
            table_name: table_name.into(),
        }
    }

    /// Return the storage namespace path used when constructing `KnowledgeStorage`.
    #[must_use]
    pub fn storage_path(&self) -> &str {
        &self.storage_path
    }

    /// Return the logical table name used for persisted entries.
    #[must_use]
    pub fn table_name(&self) -> &str {
        &self.table_name
    }
}

#[async_trait]
impl ContractFeedbackKnowledgeSink for KnowledgeStorageContractFeedbackSink {
    async fn persist_entries(&self, entries: &[KnowledgeEntry]) -> Result<Vec<String>, String> {
        let storage_path = self.storage_path.clone();
        let table_name = self.table_name.clone();
        let entries = entries.to_vec();

        tokio::task::spawn_blocking(move || {
            let storage = KnowledgeStorage::new(&storage_path, &table_name);
            let mut persisted_ids = Vec::with_capacity(entries.len());

            for entry in entries {
                storage.upsert(&entry).map_err(|error| {
                    format!("failed to persist knowledge entry {}: {error}", entry.id)
                })?;
                persisted_ids.push(entry.id);
            }

            Ok(persisted_ids)
        })
        .await
        .map_err(|error| format!("contract feedback persistence task failed: {error}"))?
    }
}

/// In-memory contract-feedback sink used by focused tests.
#[derive(Debug, Default)]
pub struct InMemoryContractFeedbackSink {
    entries: Mutex<Vec<KnowledgeEntry>>,
}

impl InMemoryContractFeedbackSink {
    /// Create a new in-memory sink.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn entries_guard(&self) -> MutexGuard<'_, Vec<KnowledgeEntry>> {
        self.entries.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Return all persisted entries.
    #[must_use]
    pub fn entries(&self) -> Vec<KnowledgeEntry> {
        self.entries_guard().clone()
    }

    /// Return the current number of persisted entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries_guard().len()
    }

    /// Return whether the sink has no persisted entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries_guard().is_empty()
    }
}

#[async_trait]
impl ContractFeedbackKnowledgeSink for InMemoryContractFeedbackSink {
    async fn persist_entries(&self, entries: &[KnowledgeEntry]) -> Result<Vec<String>, String> {
        let mut guard = self.entries_guard();
        guard.extend(entries.iter().cloned());
        Ok(entries.iter().map(|entry| entry.id.clone()).collect())
    }
}

#[cfg(test)]
#[path = "../tests/unit/sovereign/contract_feedback_sink.rs"]
mod tests;
