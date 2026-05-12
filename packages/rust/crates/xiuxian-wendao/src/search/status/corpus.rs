//! `search::status::corpus` owns Wendao search status corpus behavior.

use serde::{Deserialize, Serialize};

use crate::search::SearchCorpusKind;
use crate::search::status::{
    SearchCorpusIssue, SearchCorpusIssueSummary, SearchCorpusStatusReason, SearchMaintenanceStatus,
    SearchPlanePhase, SearchQueryTelemetry,
};

/// Per-corpus status snapshot for API and orchestration layers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchCorpusStatus {
    /// Corpus this status row describes.
    pub corpus: SearchCorpusKind,
    /// Current build/publish phase.
    pub phase: SearchPlanePhase,
    /// Last published epoch available to readers.
    pub active_epoch: Option<u64>,
    /// Current staging epoch being built, if any.
    pub staging_epoch: Option<u64>,
    /// Active schema version expected by the builder and reader.
    pub schema_version: u32,
    /// Fingerprint of the currently active or in-flight build.
    pub fingerprint: Option<String>,
    /// Build progress in the range `0.0..=1.0` while indexing.
    pub progress: Option<f32>,
    /// Published row count for the active epoch.
    pub row_count: Option<u64>,
    /// Published fragment count for the active epoch.
    pub fragment_count: Option<u64>,
    /// RFC3339 timestamp for the current build start.
    pub build_started_at: Option<String>,
    /// RFC3339 timestamp for the latest completed build attempt.
    pub build_finished_at: Option<String>,
    /// RFC3339 timestamp for the latest status mutation.
    pub updated_at: Option<String>,
    /// Last recorded build error, if any.
    pub last_error: Option<String>,
    /// Machine-readable issues attached to the current corpus snapshot.
    pub issues: Vec<SearchCorpusIssue>,
    /// High-level summary derived from the issue list.
    pub issue_summary: Option<SearchCorpusIssueSummary>,
    /// Compact status reason that folds phase and issues into one UI-friendly decision.
    pub status_reason: Option<SearchCorpusStatusReason>,
    /// Recent bounded-rerank telemetry captured from the last successful query on this corpus.
    pub last_query_telemetry: Option<SearchQueryTelemetry>,
    /// Background maintenance state for the corpus.
    pub maintenance: SearchMaintenanceStatus,
}

impl SearchCorpusStatus {
    /// Build an empty status row for a corpus.
    #[must_use]
    pub fn new(corpus: SearchCorpusKind) -> Self {
        Self {
            corpus,
            phase: SearchPlanePhase::Idle,
            active_epoch: None,
            staging_epoch: None,
            schema_version: corpus.schema_version(),
            fingerprint: None,
            progress: None,
            row_count: None,
            fragment_count: None,
            build_started_at: None,
            build_finished_at: None,
            updated_at: None,
            last_error: None,
            issues: Vec::new(),
            issue_summary: None,
            status_reason: None,
            last_query_telemetry: None,
            maintenance: SearchMaintenanceStatus::default(),
        }
    }
}
