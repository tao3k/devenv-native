use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex, RwLock};

use crate::analyzers::PluginRegistry;
use crate::repo_index::types::{RepoIndexEntryStatus, RepoIndexStatusResponse};
use crate::search::SearchPlaneService;
use tokio::sync::{Notify, Semaphore};

use super::handle::RepoIndexRuntimeHandle;
use crate::repo_index::state::task::{
    AdaptiveConcurrencyController, RepoIndexTask, repo_index_sync_concurrency_limit,
};

/// Background coordinator that keeps configured repositories indexed for search.
pub struct RepoIndexCoordinator {
    pub(crate) project_root: PathBuf,
    pub(crate) plugin_registry: Arc<PluginRegistry>,
    pub(crate) search_plane: SearchPlaneService,
    pub(crate) statuses: Arc<RwLock<BTreeMap<String, RepoIndexEntryStatus>>>,
    pub(crate) fingerprints: Arc<RwLock<HashMap<String, String>>>,
    pub(crate) queued_or_active: Arc<RwLock<HashSet<String>>>,
    pub(crate) active_repo_ids: Arc<RwLock<Vec<String>>>,
    pub(crate) status_snapshot: Arc<Mutex<RepoIndexStatusResponse>>,
    pub(super) pending: Arc<Mutex<VecDeque<RepoIndexTask>>>,
    pub(crate) notify: Arc<Notify>,
    pub(super) concurrency: Arc<Mutex<AdaptiveConcurrencyController>>,
    pub(crate) sync_concurrency_limit: usize,
    pub(crate) sync_permits: Arc<Semaphore>,
    pub(crate) started: AtomicBool,
    pub(super) runtime_handle: Mutex<Option<RepoIndexRuntimeHandle>>,
}

impl RepoIndexCoordinator {
    #[must_use]
    #[doc(hidden)]
    pub fn new(
        project_root: PathBuf,
        plugin_registry: Arc<PluginRegistry>,
        search_plane: SearchPlaneService,
    ) -> Self {
        let sync_concurrency_limit = repo_index_sync_concurrency_limit();
        Self {
            project_root,
            plugin_registry,
            search_plane,
            statuses: Arc::new(RwLock::new(BTreeMap::new())),
            fingerprints: Arc::new(RwLock::new(HashMap::new())),
            queued_or_active: Arc::new(RwLock::new(HashSet::new())),
            active_repo_ids: Arc::new(RwLock::new(Vec::new())),
            status_snapshot: Arc::new(Mutex::new(RepoIndexStatusResponse::default())),
            pending: Arc::new(Mutex::new(VecDeque::new())),
            notify: Arc::new(Notify::new()),
            concurrency: Arc::new(Mutex::new(AdaptiveConcurrencyController::new())),
            sync_concurrency_limit,
            sync_permits: Arc::new(Semaphore::new(sync_concurrency_limit)),
            started: AtomicBool::new(false),
            runtime_handle: Mutex::new(None),
        }
    }
}
