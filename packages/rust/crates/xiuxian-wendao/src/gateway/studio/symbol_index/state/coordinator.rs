use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use tokio::task::JoinHandle;

use crate::gateway::studio::symbol_index::state::{
    fingerprint_projects, maybe_spawn_build, timestamp_now,
};
use crate::gateway::studio::symbol_index::{SymbolIndexPhase, SymbolIndexStatus};
use crate::gateway::studio::types::UiProjectConfig;
use crate::search::{SearchCorpusKind, SearchPlaneService};
use crate::unified_symbol::UnifiedSymbolIndex;

pub(crate) struct SymbolIndexCoordinator {
    pub(crate) project_root: PathBuf,
    pub(crate) config_root: PathBuf,
    pub(crate) search_plane: SearchPlaneService,
    pub(crate) active_fingerprint: Arc<RwLock<Option<String>>>,
    pub(crate) status: Arc<RwLock<SymbolIndexStatus>>,
    pub(crate) spawn_lock: Mutex<()>,
    pub(crate) shutdown_requested: Arc<AtomicBool>,
    pub(crate) build_task: Mutex<Option<JoinHandle<()>>>,
}

impl SymbolIndexCoordinator {
    #[must_use]
    pub(crate) fn new(
        project_root: PathBuf,
        config_root: PathBuf,
        search_plane: SearchPlaneService,
    ) -> Self {
        Self {
            project_root,
            config_root,
            search_plane,
            active_fingerprint: Arc::new(RwLock::new(None)),
            status: Arc::new(RwLock::new(SymbolIndexStatus::default())),
            spawn_lock: Mutex::new(()),
            shutdown_requested: Arc::new(AtomicBool::new(false)),
            build_task: Mutex::new(None),
        }
    }

    pub(crate) fn stop(&self) {
        self.shutdown_requested.store(true, Ordering::SeqCst);
        *self
            .active_fingerprint
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        *self
            .status
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = SymbolIndexStatus {
            phase: SymbolIndexPhase::Idle,
            last_error: None,
            updated_at: Some(timestamp_now()),
        };
        if let Some(build_task) = self
            .build_task
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            build_task.abort();
        }
    }

    pub(crate) fn sync_projects(
        self: &Arc<Self>,
        projects: Vec<UiProjectConfig>,
        index_cache: Arc<RwLock<Option<Arc<UnifiedSymbolIndex>>>>,
    ) {
        if projects.is_empty() {
            *index_cache
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
            *self
                .active_fingerprint
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
            *self
                .status
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = SymbolIndexStatus {
                phase: SymbolIndexPhase::Idle,
                last_error: None,
                updated_at: Some(timestamp_now()),
            };
            return;
        }

        let fingerprint = fingerprint_projects(projects.as_slice());
        let search_fingerprint = self
            .search_plane
            .coordinator()
            .status_for(SearchCorpusKind::LocalSymbol)
            .fingerprint;
        let requested_fingerprint = search_fingerprint.unwrap_or(fingerprint);
        let current_fingerprint = self
            .active_fingerprint
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let current_status = self.status();
        let current_index = index_cache
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();

        if current_fingerprint.as_deref() == Some(requested_fingerprint.as_str())
            && current_index.is_some()
            && matches!(current_status.phase, SymbolIndexPhase::Ready)
        {
            return;
        }

        *index_cache
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        maybe_spawn_build(self, projects, index_cache, requested_fingerprint);
    }

    #[cfg(test)]
    pub(crate) fn ensure_started(
        self: &Arc<Self>,
        projects: Vec<UiProjectConfig>,
        index_cache: Arc<RwLock<Option<Arc<UnifiedSymbolIndex>>>>,
    ) {
        if projects.is_empty() {
            return;
        }

        let fingerprint = self
            .search_plane
            .coordinator()
            .status_for(SearchCorpusKind::LocalSymbol)
            .fingerprint
            .unwrap_or_else(|| fingerprint_projects(projects.as_slice()));
        let current_fingerprint = self
            .active_fingerprint
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let current_status = self.status();
        let current_index = index_cache
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();

        if current_fingerprint.as_deref() == Some(fingerprint.as_str()) {
            if current_index.is_some() && matches!(current_status.phase, SymbolIndexPhase::Ready) {
                return;
            }
            if matches!(current_status.phase, SymbolIndexPhase::Indexing) {
                return;
            }
        }

        maybe_spawn_build(self, projects, index_cache, fingerprint);
    }

    pub(crate) fn status(&self) -> SymbolIndexStatus {
        self.status
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}
