//! `search::status::snapshot` owns Wendao search status snapshot behavior.

use serde::{Deserialize, Serialize};

use crate::search::status::SearchCorpusStatus;

/// Multi-corpus view returned by the coordinator and status API.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SearchPlaneStatusSnapshot {
    /// Current repo-backed request pressure observed by the shared repo-read gate.
    pub repo_read_pressure: Option<SearchRepoReadPressure>,
    /// Ordered status rows for every search-plane corpus.
    pub corpora: Vec<SearchCorpusStatus>,
}

/// Response-level repo-backed read pressure surfaced alongside corpus status rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SearchRepoReadPressure {
    /// Total shared repo-read budget currently configured for the service.
    pub budget: u32,
    /// Number of repo-read permits currently checked out by in-flight repo queries.
    pub in_flight: u32,
    /// RFC3339 timestamp of the most recent repo-search dispatch observation.
    pub captured_at: Option<String>,
    /// Number of repositories considered by the most recent repo-backed dispatch.
    pub requested_repo_count: Option<u32>,
    /// Number of repositories that were actually searchable in the most recent dispatch.
    pub searchable_repo_count: Option<u32>,
    /// Parallelism cap applied to the most recent repo-backed dispatch.
    pub parallelism: Option<u32>,
    /// Whether the most recent searchable repo set was wider than the shared fan-out budget.
    pub fanout_capped: bool,
}
