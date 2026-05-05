//! Core Zhixing-Heyi service type and constructor.

use super::reminder_queue::ReminderQueueStore;
use crate::Result;
use crate::storage::MarkdownStorage;
use chrono_tz::Tz;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use xiuxian_qianhuan::{ManifestationInterface, PersonaProfile};
use xiuxian_wendao::graph::KnowledgeGraph;
use xiuxian_wendao::skill_runtime::zhixing::{ZhixingIndexSummary, ZhixingWendaoIndexer};
use xiuxian_wendao::sync::IncrementalSyncPolicy;

/// Integration of Zhi (Knowledge) and Xing (Action) - the Heyi orchestrator.
pub struct ZhixingHeyi {
    /// Reference to the Knowledge Graph managed by Wendao.
    pub graph: Arc<KnowledgeGraph>,
    /// Reference to the Manifestation layer managed by Qianhuan.
    pub manifestation: Arc<dyn ManifestationInterface>,
    /// Reference to the markdown file storage engine.
    pub storage: Arc<MarkdownStorage>,
    /// Scope key used for persistent metadata indexing.
    pub scope_key: String,
    /// Configured time zone for local calculations (e.g., "`America/Los_Angeles`").
    pub time_zone: Tz,
    /// Optional Valkey due queue for scalable reminder scheduling.
    pub reminder_queue: Option<ReminderQueueStore>,
    /// Active persona profile for Zhixing response shaping.
    pub active_persona: Option<PersonaProfile>,
}

impl ZhixingHeyi {
    /// Creates a new Heyi orchestrator with explicit time-zone validation.
    ///
    /// # Errors
    /// Returns `Error::Config` when `time_zone_str` is not a valid IANA time zone.
    pub fn new(
        graph: Arc<KnowledgeGraph>,
        manifestation: Arc<dyn ManifestationInterface>,
        storage: Arc<MarkdownStorage>,
        scope_key: String,
        time_zone_str: &str,
    ) -> Result<Self> {
        let time_zone = Tz::from_str(time_zone_str).map_err(|error| {
            crate::Error::Config(format!("Invalid time zone '{time_zone_str}': {error}"))
        })?;
        Ok(Self {
            graph,
            manifestation,
            storage,
            scope_key,
            time_zone,
            reminder_queue: None,
            active_persona: None,
        })
    }

    /// Attach an optional reminder queue backend.
    #[must_use]
    pub fn with_reminder_queue(mut self, reminder_queue: Option<ReminderQueueStore>) -> Self {
        self.reminder_queue = reminder_queue;
        self
    }

    /// Attach active persona profile used for response rendering.
    #[must_use]
    pub fn with_active_persona(mut self, active_persona: Option<PersonaProfile>) -> Self {
        self.active_persona = active_persona;
        self
    }

    /// Synchronizes the graph with local disk state via Wendao domain indexer.
    ///
    /// # Errors
    /// Returns an error when markdown discovery, parsing, or graph operations fail.
    pub fn sync_from_disk(&self) -> Result<ZhixingIndexSummary> {
        let indexer =
            ZhixingWendaoIndexer::new(Arc::clone(&self.graph), self.storage.root_dir.clone());
        let summary = indexer
            .index_all_domain_objects()
            .map_err(|error| crate::Error::Internal(error.to_string()))?;
        log::debug!(
            "Zhixing sync completed for scope={} (journal_documents={}, agenda_documents={}, task_entities={})",
            self.scope_key,
            summary.journal_documents,
            summary.agenda_documents,
            summary.task_entities
        );
        Ok(summary)
    }

    /// Incrementally synchronize one changed path into the graph.
    ///
    /// # Errors
    /// Returns an error when file parsing or graph operations fail.
    pub fn sync_changed_path_from_disk(
        &self,
        changed_path: &Path,
        policy: &IncrementalSyncPolicy,
    ) -> Result<bool> {
        let indexer =
            ZhixingWendaoIndexer::new(Arc::clone(&self.graph), self.storage.root_dir.clone());
        indexer
            .sync_changed_path(changed_path, policy)
            .map_err(|error| crate::Error::Internal(error.to_string()))
    }
}
