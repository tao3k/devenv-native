//! `search::status::reason` owns Wendao search status reason behavior.

use serde::{Deserialize, Serialize};

/// UI-friendly severity for one corpus status reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchCorpusStatusSeverity {
    /// Informational lifecycle state.
    Info,
    /// Non-blocking but inconsistent or degraded state.
    Warning,
    /// Blocking state that prevents reliable reads.
    Error,
}

/// Suggested next action for one corpus status reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchCorpusStatusAction {
    /// Wait for the in-flight work to finish.
    Wait,
    /// Retry or restart the failed corpus build.
    RetryBuild,
    /// Trigger repo resync/publication rebuild.
    ResyncRepo,
    /// Inspect upstream repo-index sync failures.
    InspectRepoSync,
}

/// Stable machine-readable reason attached to one corpus status row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchCorpusStatusReasonCode {
    /// The corpus is indexing for the first time and has no readable publication yet.
    WarmingUp,
    /// The corpus is indexing for the first time, and the staging epoch has already been prewarmed.
    Prewarming,
    /// The corpus is refreshing while an older publication remains readable.
    Refreshing,
    /// Background compaction is actively running for the readable publication.
    Compacting,
    /// Background compaction has been scheduled for the readable publication.
    CompactionPending,
    /// The latest build failed.
    BuildFailed,
    /// A repo reported ready but no published state exists for this corpus.
    PublishedManifestMissing,
    /// Published state exists, but it does not record the source revision.
    PublishedRevisionMissing,
    /// Published state exists, but it points at a different source revision.
    PublishedRevisionMismatch,
    /// Repo indexing failed while the corpus status was synthesized.
    RepoIndexFailed,
}

/// Compact reason surface that drives UI severity and action semantics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchCorpusStatusReason {
    /// Stable machine-readable reason code.
    pub code: SearchCorpusStatusReasonCode,
    /// UI-facing severity lane for the current reason.
    pub severity: SearchCorpusStatusSeverity,
    /// Suggested next action for the current reason.
    pub action: SearchCorpusStatusAction,
    /// Whether the corpus remains readable despite the current reason.
    pub readable: bool,
}
