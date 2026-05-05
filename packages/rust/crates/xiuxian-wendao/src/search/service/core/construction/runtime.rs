use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use crate::search::service::core::construction::concurrency::repo_search_read_concurrency_limit;
use crate::search::service::core::types::SearchPlaneService;
use crate::search::service::helpers::{default_storage_root, manifest_keyspace_for_project};
use crate::search::{SearchMaintenancePolicy, SearchManifestKeyspace, SearchPlaneCoordinator};
use tokio::sync::Semaphore;
use xiuxian_db_store::{VectorStore, VectorStoreError};

impl SearchPlaneService {
    /// Create a service rooted under project-local `PRJ_DATA_HOME`.
    #[must_use]
    pub fn new(project_root: PathBuf) -> Self {
        let storage_root = default_storage_root(project_root.as_path());
        let manifest_keyspace = manifest_keyspace_for_project(project_root.as_path());
        let cache = crate::search::cache::SearchPlaneCache::from_runtime(manifest_keyspace.clone());
        Self::with_runtime(
            project_root,
            storage_root,
            manifest_keyspace,
            SearchMaintenancePolicy::default(),
            cache,
        )
    }

    /// Create a service with explicit storage root, keyspace, and policy.
    #[must_use]
    pub fn with_paths(
        project_root: PathBuf,
        storage_root: PathBuf,
        manifest_keyspace: SearchManifestKeyspace,
        maintenance_policy: SearchMaintenancePolicy,
    ) -> Self {
        let cache = crate::search::cache::SearchPlaneCache::disabled(manifest_keyspace.clone());
        Self::with_runtime(
            project_root,
            storage_root,
            manifest_keyspace,
            maintenance_policy,
            cache,
        )
    }

    pub(crate) fn with_runtime(
        project_root: PathBuf,
        storage_root: PathBuf,
        manifest_keyspace: SearchManifestKeyspace,
        maintenance_policy: SearchMaintenancePolicy,
        cache: crate::search::cache::SearchPlaneCache,
    ) -> Self {
        let repo_search_read_concurrency_limit = repo_search_read_concurrency_limit();
        let coordinator = Arc::new(SearchPlaneCoordinator::new(
            project_root.clone(),
            storage_root.clone(),
            manifest_keyspace.clone(),
            maintenance_policy,
        ));
        let service = Self {
            project_root,
            storage_root,
            manifest_keyspace,
            coordinator,
            datafusion_query_engine: xiuxian_db_store::SearchEngineContext::new(),
            cache,
            repo_search_read_concurrency_limit,
            repo_search_read_permits: Arc::new(Semaphore::new(repo_search_read_concurrency_limit)),
            repo_search_dispatch: Arc::new(Mutex::new(
                crate::search::service::core::types::RepoSearchDispatchRuntime::default(),
            )),
            repo_runtime_generation: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            local_maintenance: Arc::new(Mutex::new(
                crate::search::service::core::types::LocalMaintenanceRuntime::default(),
            )),
            repo_maintenance: Arc::new(Mutex::new(
                crate::search::service::core::types::RepoMaintenanceRuntime::default(),
            )),
            query_telemetry: Arc::new(RwLock::new(std::collections::BTreeMap::new())),
            parquet_query_engine: Arc::new(OnceLock::new()),
            markdown_snapshot_entries: Arc::new(RwLock::new(std::collections::HashMap::new())),
            source_snapshot_entries: Arc::new(RwLock::new(std::collections::HashMap::new())),
            repeat_work_telemetry: Arc::new(RwLock::new(
                crate::search::service::core::repeat_work::SearchBuildRepeatWorkTelemetryState::default(),
            )),
            repo_corpus_records: Arc::new(RwLock::new(std::collections::BTreeMap::new())),
        };
        service.restore_local_corpus_statuses_from_runtime();
        service
    }

    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    /// Create a search plane with an in-memory test cache.
    pub fn with_test_cache(
        project_root: PathBuf,
        storage_root: PathBuf,
        manifest_keyspace: SearchManifestKeyspace,
        maintenance_policy: SearchMaintenancePolicy,
    ) -> Self {
        let cache = crate::search::cache::SearchPlaneCache::for_tests(manifest_keyspace.clone());
        Self::with_runtime(
            project_root,
            storage_root,
            manifest_keyspace,
            maintenance_policy,
            cache,
        )
    }

    #[cfg(any(test, feature = "test-support"))]
    #[must_use]
    /// Create a search plane with an in-memory test cache and revision retention.
    pub fn with_test_cache_and_revision_retention(
        project_root: PathBuf,
        storage_root: PathBuf,
        manifest_keyspace: SearchManifestKeyspace,
        maintenance_policy: SearchMaintenancePolicy,
        repo_revision_retention: usize,
    ) -> Self {
        let cache_config = crate::search::cache::SearchPlaneCacheConfig {
            repo_revision_retention: repo_revision_retention.max(1),
            ..crate::search::cache::SearchPlaneCacheConfig::default()
        };
        let cache = crate::search::cache::SearchPlaneCache::for_tests_with_config(
            manifest_keyspace.clone(),
            cache_config,
        );
        Self::with_runtime(
            project_root,
            storage_root,
            manifest_keyspace,
            maintenance_policy,
            cache,
        )
    }

    /// Absolute project root for this service.
    #[must_use]
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Root directory that contains all per-corpus stores.
    #[must_use]
    pub fn storage_root(&self) -> &Path {
        &self.storage_root
    }

    /// Valkey namespace used by this service.
    #[must_use]
    pub fn manifest_keyspace(&self) -> &SearchManifestKeyspace {
        &self.manifest_keyspace
    }

    /// Shared coordinator for background build state.
    #[must_use]
    pub fn coordinator(&self) -> Arc<SearchPlaneCoordinator> {
        Arc::clone(&self.coordinator)
    }

    /// Shared `DataFusion` search-engine context for Parquet-backed search-plane reads.
    #[cfg(not(feature = "duckdb"))]
    #[must_use]
    pub(crate) fn datafusion_query_engine(&self) -> &xiuxian_db_store::SearchEngineContext {
        &self.datafusion_query_engine
    }

    /// Open the Lance-backed store for one search corpus.
    ///
    /// # Errors
    ///
    /// Returns an error when the backing store cannot be initialized.
    pub async fn open_store(
        &self,
        corpus: crate::search::SearchCorpusKind,
    ) -> Result<VectorStore, VectorStoreError> {
        let root = self.corpus_root(corpus);
        VectorStore::new(root.to_string_lossy().as_ref(), None).await
    }
}
