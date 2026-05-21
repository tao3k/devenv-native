//! `search::coordinator::state` owns Wendao search coordinator state behavior.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, RwLock};

use crate::search::{
    SearchCorpusKind, SearchCorpusStatus, SearchMaintenancePolicy, SearchManifestKeyspace,
    SearchPlaneStatusSnapshot,
};

use super::types::{SearchCorpusRuntime, snapshot_from_state};

/// In-memory, per-corpus build coordinator for the Studio search plane.
pub struct SearchPlaneCoordinator {
    pub(crate) project_root: PathBuf,
    pub(crate) storage_root: PathBuf,
    pub(crate) manifest_keyspace: SearchManifestKeyspace,
    pub(crate) maintenance_policy: SearchMaintenancePolicy,
    pub(crate) state: Arc<RwLock<BTreeMap<SearchCorpusKind, SearchCorpusRuntime>>>,
    pub(crate) spawn_lock: Mutex<()>,
}

impl SearchPlaneCoordinator {
    /// Construct a coordinator for one project-local search plane.
    #[must_use]
    pub fn new(
        project_root: PathBuf,
        storage_root: PathBuf,
        manifest_keyspace: SearchManifestKeyspace,
        maintenance_policy: SearchMaintenancePolicy,
    ) -> Self {
        let state = SearchCorpusKind::ALL
            .into_iter()
            .map(|corpus| (corpus, SearchCorpusRuntime::new(corpus)))
            .collect();
        Self {
            project_root,
            storage_root,
            manifest_keyspace,
            maintenance_policy,
            state: Arc::new(RwLock::new(state)),
            spawn_lock: Mutex::new(()),
        }
    }

    /// Absolute project root associated with this coordinator.
    #[must_use]
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Root directory that stores per-corpus Lance datasets.
    #[must_use]
    pub fn storage_root(&self) -> &Path {
        &self.storage_root
    }

    /// Valkey namespace used for manifests, leases, and short-lived caches.
    #[must_use]
    pub fn manifest_keyspace(&self) -> &SearchManifestKeyspace {
        &self.manifest_keyspace
    }

    /// Maintenance policy that decides when compaction should run.
    #[must_use]
    pub fn maintenance_policy(&self) -> &SearchMaintenancePolicy {
        &self.maintenance_policy
    }

    /// Snapshot ordered status rows for every corpus.
    #[must_use]
    pub fn status(&self) -> SearchPlaneStatusSnapshot {
        let state = self
            .state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        snapshot_from_state(&state)
    }

    /// Read the current status for one corpus.
    #[must_use]
    pub fn status_for(&self, corpus: SearchCorpusKind) -> SearchCorpusStatus {
        self.state
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&corpus)
            .map_or_else(
                || SearchCorpusStatus::new(corpus),
                |runtime| runtime.status.clone(),
            )
    }

    /// Replace the runtime status for a corpus from an external publisher.
    pub fn replace_status(&self, status: SearchCorpusStatus) {
        let mut state = self
            .state
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let runtime = state
            .entry(status.corpus)
            .or_insert_with(|| SearchCorpusRuntime::new(status.corpus));
        runtime.next_epoch = runtime
            .next_epoch
            .max(status.active_epoch.unwrap_or_default().saturating_add(1))
            .max(status.staging_epoch.unwrap_or_default().saturating_add(1));
        runtime.status = status;
    }
}

pub(crate) fn timestamp_now() -> String {
    chrono::Utc::now().to_rfc3339()
}
