use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

#[cfg(any(test, feature = "test-support"))]
use crate::analyzers::RepositoryAnalysisOutput;
use crate::search::SearchFileFingerprint;

/// Lifecycle phase for one background repo-index task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum RepoIndexPhase {
    /// No indexing work is currently scheduled for the repository.
    Idle,
    /// The repository is queued for background processing.
    Queued,
    /// The repository source is being validated.
    Checking,
    /// The repository source is being synchronized.
    Syncing,
    /// Analysis and code-document collection are in progress.
    Indexing,
    /// A usable snapshot is ready for search.
    Ready,
    /// The repository configuration or layout is unsupported.
    Unsupported,
    /// The most recent indexing attempt failed.
    Failed,
}

/// Current index status for one configured repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct RepoIndexEntryStatus {
    /// Stable repository identifier.
    pub repo_id: String,
    /// Current lifecycle phase for the repository.
    pub phase: RepoIndexPhase,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Current queue position when the repository is still pending.
    pub queue_position: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Most recent indexing error when the phase is `Failed`.
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Most recent synchronized revision observed for the repository.
    pub last_revision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Timestamp of the most recent status update.
    pub updated_at: Option<String>,
    /// Number of indexing attempts recorded for the repository.
    pub attempt_count: usize,
}

/// Aggregated repo-index status payload returned to Studio clients.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct RepoIndexStatusResponse {
    /// Total number of repositories in the response.
    pub total: usize,
    /// Number of repositories currently active across all phases.
    pub active: usize,
    /// Number of repositories currently queued.
    pub queued: usize,
    /// Number of repositories currently being validated.
    pub checking: usize,
    /// Number of repositories currently being synchronized.
    pub syncing: usize,
    /// Number of repositories currently being indexed.
    pub indexing: usize,
    /// Number of repositories with ready snapshots.
    pub ready: usize,
    /// Number of repositories classified as unsupported.
    pub unsupported: usize,
    /// Number of repositories whose latest indexing attempt failed.
    pub failed: usize,
    /// Current adaptive target concurrency for repo indexing.
    pub target_concurrency: usize,
    /// Maximum concurrency ceiling derived from host parallelism.
    pub max_concurrency: usize,
    /// Effective remote-sync concurrency limit enforced by the coordinator.
    pub sync_concurrency_limit: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Repository currently being processed, when known.
    pub current_repo_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Active repositories currently in progress.
    pub active_repo_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Per-repository status rows included in the response.
    pub repos: Vec<RepoIndexEntryStatus>,
}

/// Request payload for repo-index enqueue operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct RepoIndexRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Optional repository identifier to target a single repo.
    pub repo: Option<String>,
    #[serde(default)]
    /// Whether to force a refresh even when a repo is already indexed.
    pub refresh: bool,
}

/// Indexed source document captured from a configured repository.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoCodeDocument {
    /// Repository-relative source path.
    pub path: String,
    /// Optional language label inferred for the source path.
    pub language: Option<String>,
    /// Full UTF-8 source contents.
    pub contents: Arc<str>,
    /// File size in bytes at scan time.
    pub size_bytes: u64,
    /// Last-modified timestamp in Unix milliseconds.
    pub modified_unix_ms: u64,
}

impl RepoCodeDocument {
    #[must_use]
    pub(crate) fn to_file_fingerprint(
        &self,
        extractor_version: u32,
        schema_version: u32,
    ) -> SearchFileFingerprint {
        SearchFileFingerprint {
            relative_path: self.path.clone(),
            partition_id: None,
            size_bytes: self.size_bytes,
            modified_unix_ms: self.modified_unix_ms,
            extractor_version,
            schema_version,
            blake3: None,
        }
    }
}

#[cfg(any(test, feature = "test-support"))]
#[allow(dead_code)]
#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct RepoIndexSnapshot {
    #[allow(dead_code)]
    pub repo_id: String,
    #[allow(dead_code)]
    pub analysis: Arc<RepositoryAnalysisOutput>,
}
