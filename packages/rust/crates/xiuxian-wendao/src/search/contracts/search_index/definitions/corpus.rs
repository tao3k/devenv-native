use serde::{Deserialize, Serialize};
use specta::Type;

use super::issues::{SearchIndexIssue, SearchIndexIssueSummary};
use super::lifecycle::SearchIndexPhase;
use super::maintenance::SearchIndexMaintenanceStatus;
use super::status_reason::SearchIndexStatusReason;
use super::telemetry::SearchIndexQueryTelemetry;

/// Current search-plane status for one corpus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct SearchCorpusIndexStatus {
    /// Stable corpus identifier.
    pub corpus: String,
    /// Current lifecycle phase.
    pub phase: SearchIndexPhase,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Published epoch available to readers.
    pub active_epoch: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Staging epoch currently building.
    pub staging_epoch: Option<u64>,
    /// Schema version for the active or in-flight corpus.
    pub schema_version: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Active or in-flight build fingerprint.
    pub fingerprint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Build progress in the range `0.0..=1.0`.
    pub progress: Option<f32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Published row count for the active epoch.
    pub row_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Published fragment count for the active epoch.
    pub fragment_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// RFC3339 timestamp for the active build start.
    pub build_started_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// RFC3339 timestamp for the latest completed build attempt.
    pub build_finished_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// RFC3339 timestamp for the latest status mutation.
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Latest build error for the corpus.
    pub last_error: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Machine-readable issues attached to the current corpus snapshot.
    pub issues: Vec<SearchIndexIssue>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// High-level summary derived from the issue list.
    pub issue_summary: Option<SearchIndexIssueSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Compact status reason that folds phase and issues into one UI-friendly decision.
    pub status_reason: Option<SearchIndexStatusReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Recent bounded-rerank telemetry captured from the last successful query on this corpus.
    pub last_query_telemetry: Option<SearchIndexQueryTelemetry>,
    /// Maintenance view for the corpus.
    pub maintenance: SearchIndexMaintenanceStatus,
}

/// Aggregated search-plane status payload returned by Studio.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type, Default)]
#[serde(rename_all = "camelCase")]
pub struct SearchIndexStatusResponse {
    /// Total number of corpora in the response.
    pub total: usize,
    /// Number of corpora currently idle.
    pub idle: usize,
    /// Number of corpora currently indexing.
    pub indexing: usize,
    /// Number of corpora with ready published epochs.
    pub ready: usize,
    /// Number of corpora with readable but degraded published epochs.
    pub degraded: usize,
    /// Number of corpora whose latest build failed.
    pub failed: usize,
    /// Number of corpora pending compaction.
    pub compaction_pending: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Response-level dominant status reason derived from per-corpus status reasons.
    pub status_reason:
        Option<crate::search::contracts::search_index::SearchIndexAggregateStatusReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Response-level maintenance rollup derived from per-corpus maintenance state.
    pub maintenance_summary:
        Option<crate::search::contracts::search_index::SearchIndexAggregateMaintenanceSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Response-level rollup derived from recent per-corpus bounded query telemetry.
    pub query_telemetry_summary:
        Option<crate::search::contracts::search_index::SearchIndexAggregateQueryTelemetry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Shared repo-backed read pressure derived from the repo-search gate.
    pub repo_read_pressure:
        Option<crate::search::contracts::search_index::SearchIndexRepoReadPressure>,
    /// Ordered per-corpus status rows.
    pub corpora: Vec<SearchCorpusIndexStatus>,
}
