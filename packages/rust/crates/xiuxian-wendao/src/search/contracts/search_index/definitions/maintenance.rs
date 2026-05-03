use serde::{Deserialize, Serialize};
use specta::Type;

/// Transparent bool wrapper for queued-compaction fairness aging in Studio responses.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(transparent)]
#[specta(transparent)]
pub struct SearchIndexMaintenanceQueueAged(bool);

impl SearchIndexMaintenanceQueueAged {
    /// Returns whether the queued compaction has already crossed the fairness-aging guard.
    #[must_use]
    pub const fn is_aged(self) -> bool {
        self.0
    }
}

impl From<bool> for SearchIndexMaintenanceQueueAged {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl std::ops::Not for SearchIndexMaintenanceQueueAged {
    type Output = bool;

    fn not(self) -> Self::Output {
        !self.0
    }
}

/// Response-level summary derived from per-corpus maintenance state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexAggregateMaintenanceSummary {
    /// Number of corpora currently running a prewarm.
    pub prewarm_running_count: usize,
    /// Number of corpora with queued prewarm backlog.
    pub prewarm_queued_corpus_count: usize,
    /// Largest queued prewarm depth observed across corpora.
    pub max_prewarm_queue_depth: u32,
    /// Number of corpora currently running compaction.
    pub compaction_running_count: usize,
    /// Number of corpora with queued compaction backlog.
    pub compaction_queued_corpus_count: usize,
    /// Largest queued compaction depth observed across corpora.
    pub max_compaction_queue_depth: u32,
    /// Number of corpora whose maintenance still reports compaction pending.
    pub compaction_pending_count: usize,
    /// Number of corpora whose queued compaction has already crossed the fairness aging guard.
    pub aged_compaction_queue_count: usize,
}

/// Response-level repo-backed read pressure derived from the shared repo-read gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexRepoReadPressure {
    /// Total shared repo-read budget currently configured for the service.
    pub budget: u32,
    /// Number of repo-read permits currently checked out by in-flight repo queries.
    pub in_flight: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// RFC3339 timestamp of the most recent repo-search dispatch observation.
    pub captured_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Number of repositories considered by the most recent repo-backed dispatch.
    pub requested_repo_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Number of repositories that were actually searchable in the most recent dispatch.
    pub searchable_repo_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Parallelism cap applied to the most recent repo-backed dispatch.
    pub parallelism: Option<u32>,
    /// Whether the most recent searchable repo set was wider than the shared fan-out budget.
    pub fanout_capped: bool,
}

/// Background maintenance state for one corpus.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexMaintenanceStatus {
    #[serde(default)]
    /// Whether a staging-table prewarm task is actively running.
    pub prewarm_running: bool,
    #[serde(default)]
    /// Number of queued prewarm tasks currently waiting behind the active worker.
    pub prewarm_queue_depth: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// One-based queue position for this corpus when its prewarm is queued in repo maintenance.
    pub prewarm_queue_position: Option<u32>,
    /// Whether the corpus is actively being compacted in the background.
    pub compaction_running: bool,
    #[serde(default)]
    /// Number of queued compaction tasks currently waiting behind the active worker.
    pub compaction_queue_depth: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// One-based queue position for this corpus when its compaction is queued locally.
    pub compaction_queue_position: Option<u32>,
    #[serde(default)]
    /// Whether enqueue-time fairness aging has already promoted this queued compaction task.
    pub compaction_queue_aged: SearchIndexMaintenanceQueueAged,
    /// Whether the corpus should be compacted in the background.
    pub compaction_pending: bool,
    /// Number of publishes since the last compact.
    pub publish_count_since_compaction: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// RFC3339 timestamp of the latest staging-table prewarm.
    pub last_prewarmed_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Epoch identifier of the latest staging-table prewarm.
    pub last_prewarmed_epoch: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// RFC3339 timestamp of the latest compaction.
    pub last_compacted_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Reason recorded for the latest compaction.
    pub last_compaction_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Row count observed when the latest compaction completed.
    pub last_compacted_row_count: Option<u64>,
}
