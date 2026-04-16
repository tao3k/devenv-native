use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

use dashmap::DashMap;
use tokio::sync::{Semaphore, oneshot};
use tokio::task::JoinHandle;

use crate::repo_index::{RepoIndexEntryStatus, RepoIndexPhase};
use crate::search::coordinator::SearchCompactionReason;
use crate::search::service::core::repeat_work::SearchBuildRepeatWorkTelemetryState;
use crate::search::{
    MarkdownSnapshotEntry, SearchCorpusKind, SearchManifestKeyspace, SearchPlaneCoordinator,
    SearchQueryTelemetry, SearchRepoCorpusRecord, SearchRepoRuntimeRecord, SourceSnapshotEntry,
};
use xiuxian_vector_store::SearchEngineContext;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum RepoMaintenanceTaskKind {
    Prewarm,
    Compaction,
}

pub(crate) type RepoMaintenanceTaskKey =
    (SearchCorpusKind, String, String, RepoMaintenanceTaskKind);
pub(crate) type RepoMaintenanceTaskResult = Result<(), String>;

#[derive(Debug, Clone)]
pub(crate) struct RepoPrewarmTask {
    pub(crate) corpus: SearchCorpusKind,
    pub(crate) repo_id: String,
    pub(crate) table_name: String,
    pub(crate) projected_columns: Vec<String>,
}

impl RepoPrewarmTask {
    #[must_use]
    pub(crate) fn task_key(&self) -> RepoMaintenanceTaskKey {
        (
            self.corpus,
            self.repo_id.clone(),
            self.table_name.clone(),
            RepoMaintenanceTaskKind::Prewarm,
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RepoCompactionTask {
    pub(crate) corpus: SearchCorpusKind,
    pub(crate) repo_id: String,
    pub(crate) publication_id: String,
    pub(crate) table_name: String,
    pub(crate) row_count: u64,
    pub(crate) reason: SearchCompactionReason,
}

impl RepoCompactionTask {
    #[must_use]
    pub(crate) fn task_key(&self) -> RepoMaintenanceTaskKey {
        (
            self.corpus,
            self.repo_id.clone(),
            self.publication_id.clone(),
            RepoMaintenanceTaskKind::Compaction,
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) enum RepoMaintenanceTask {
    Prewarm(RepoPrewarmTask),
    Compaction(RepoCompactionTask),
}

impl RepoMaintenanceTask {
    #[must_use]
    pub(crate) fn task_key(&self) -> RepoMaintenanceTaskKey {
        match self {
            Self::Prewarm(task) => task.task_key(),
            Self::Compaction(task) => task.task_key(),
        }
    }

    #[must_use]
    pub(crate) fn repo_id(&self) -> &str {
        match self {
            Self::Prewarm(task) => task.repo_id.as_str(),
            Self::Compaction(task) => task.repo_id.as_str(),
        }
    }
}

pub(crate) struct QueuedRepoMaintenanceTask {
    pub(crate) task: RepoMaintenanceTask,
    pub(crate) enqueue_sequence: u64,
}

#[derive(Default)]
pub(crate) struct RepoMaintenanceRuntime {
    pub(crate) in_flight: BTreeSet<RepoMaintenanceTaskKey>,
    pub(crate) waiters:
        BTreeMap<RepoMaintenanceTaskKey, Vec<oneshot::Sender<RepoMaintenanceTaskResult>>>,
    pub(crate) queue: VecDeque<QueuedRepoMaintenanceTask>,
    pub(crate) next_enqueue_sequence: u64,
    pub(crate) shutdown_requested: bool,
    pub(crate) worker_running: bool,
    pub(crate) worker_handle: Option<JoinHandle<()>>,
    pub(crate) active_task: Option<RepoMaintenanceTaskKey>,
}

#[derive(Default)]
pub(crate) struct LocalMaintenanceRuntime {
    pub(crate) shutdown_requested: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RepoSearchDispatchRuntime {
    pub(crate) captured_at: Option<String>,
    pub(crate) requested_repo_count: u32,
    pub(crate) searchable_repo_count: u32,
    pub(crate) parallelism: u32,
    pub(crate) fanout_capped: bool,
}

pub(crate) type MarkdownSnapshotEntryCell = Arc<OnceLock<Arc<MarkdownSnapshotEntry>>>;
pub(crate) type SourceSnapshotEntryCell = Arc<OnceLock<Arc<SourceSnapshotEntry>>>;

/// Project-scoped entrypoint for the search-plane domain.
#[derive(Clone)]
pub struct SearchPlaneService {
    pub(super) project_root: PathBuf,
    pub(super) storage_root: PathBuf,
    pub(super) manifest_keyspace: SearchManifestKeyspace,
    pub(super) coordinator: Arc<SearchPlaneCoordinator>,
    pub(super) datafusion_query_engine: SearchEngineContext,
    pub(super) cache: crate::search::cache::SearchPlaneCache,
    pub(crate) repo_search_read_concurrency_limit: usize,
    pub(crate) repo_search_read_permits: Arc<Semaphore>,
    pub(crate) repo_search_dispatch: Arc<Mutex<RepoSearchDispatchRuntime>>,
    pub(crate) repo_runtime_generation: Arc<AtomicU64>,
    pub(crate) local_maintenance: Arc<Mutex<LocalMaintenanceRuntime>>,
    pub(crate) repo_maintenance: Arc<Mutex<RepoMaintenanceRuntime>>,
    pub(crate) query_telemetry:
        Arc<RwLock<std::collections::BTreeMap<SearchCorpusKind, SearchQueryTelemetry>>>,
    pub(crate) parquet_query_engine: Arc<OnceLock<crate::duckdb::ParquetQueryEngine>>,
    pub(crate) markdown_snapshot_entries: Arc<DashMap<String, MarkdownSnapshotEntryCell>>,
    pub(crate) source_snapshot_entries: Arc<DashMap<String, SourceSnapshotEntryCell>>,
    pub(crate) repeat_work_telemetry: Arc<RwLock<SearchBuildRepeatWorkTelemetryState>>,
    pub(super) repo_corpus_records:
        Arc<RwLock<std::collections::BTreeMap<(SearchCorpusKind, String), SearchRepoCorpusRecord>>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RepoRuntimeState {
    pub(crate) phase: RepoIndexPhase,
    pub(crate) last_revision: Option<String>,
    pub(crate) last_error: Option<String>,
    pub(crate) updated_at: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RepoSearchAvailability {
    Searchable,
    Pending,
    Skipped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RepoSearchPublicationState {
    pub(crate) entity_published: bool,
    pub(crate) content_published: bool,
    pub(crate) availability: RepoSearchAvailability,
}

pub(crate) struct RepoSearchQueryCacheKeyInput<'a> {
    pub(crate) scope: &'a str,
    pub(crate) corpora: &'a [SearchCorpusKind],
    pub(crate) repo_corpora: &'a [SearchCorpusKind],
    pub(crate) repo_ids: &'a [String],
    pub(crate) query: &'a str,
    pub(crate) limit: usize,
    pub(crate) intent: Option<&'a str>,
    pub(crate) repo_hint: Option<&'a str>,
}

impl RepoSearchPublicationState {
    #[must_use]
    pub(crate) fn is_searchable(self) -> bool {
        matches!(self.availability, RepoSearchAvailability::Searchable)
    }
}

impl RepoRuntimeState {
    pub(super) fn from_status(status: &RepoIndexEntryStatus) -> Self {
        Self {
            phase: status.phase,
            last_revision: status.last_revision.clone(),
            last_error: status.last_error.clone(),
            updated_at: status.updated_at.clone(),
        }
    }

    pub(super) fn from_record(record: &SearchRepoRuntimeRecord) -> Self {
        Self {
            phase: record.phase,
            last_revision: record.last_revision.clone(),
            last_error: record.last_error.clone(),
            updated_at: record.updated_at.clone(),
        }
    }

    pub(super) fn as_status(&self, repo_id: &str) -> RepoIndexEntryStatus {
        RepoIndexEntryStatus {
            repo_id: repo_id.to_string(),
            phase: self.phase,
            queue_position: None,
            last_error: self.last_error.clone(),
            last_revision: self.last_revision.clone(),
            updated_at: self.updated_at.clone(),
            attempt_count: 0,
        }
    }
}
