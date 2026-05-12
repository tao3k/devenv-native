//! `search::coordinator::types` owns Wendao search coordinator types behavior.

use crate::search::{SearchCorpusKind, SearchCorpusStatus, SearchPlaneStatusSnapshot};

/// Reason that triggered a background compaction request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchCompactionReason {
    /// Publish count crossed the maintenance threshold.
    PublishThreshold,
    /// Row-count drift crossed the maintenance threshold.
    RowDeltaRatio,
}

impl SearchCompactionReason {
    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::PublishThreshold => "publish_threshold",
            Self::RowDeltaRatio => "row_delta_ratio",
        }
    }
}

/// Single build token for one corpus epoch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchBuildLease {
    /// Corpus being built.
    pub corpus: SearchCorpusKind,
    /// Fingerprint bound to the in-flight build.
    pub fingerprint: String,
    /// Staging epoch assigned to this build.
    pub epoch: u64,
    /// Schema version expected by this build.
    pub schema_version: u32,
}

/// Result of attempting to start a background build.
#[derive(Debug, Clone, PartialEq)]
pub enum BeginBuildDecision {
    /// A new staging build has been leased.
    Started(SearchBuildLease),
    /// The requested fingerprint is already published and ready.
    AlreadyReady(SearchCorpusStatus),
    /// The requested fingerprint is already being indexed.
    AlreadyIndexing(SearchCorpusStatus),
}

#[derive(Debug, Clone)]
pub(crate) struct SearchCorpusRuntime {
    pub(crate) status: SearchCorpusStatus,
    pub(crate) next_epoch: u64,
    pub(crate) last_compacted_row_count: Option<u64>,
}

impl SearchCorpusRuntime {
    pub(crate) fn new(corpus: SearchCorpusKind) -> Self {
        Self {
            status: SearchCorpusStatus::new(corpus),
            next_epoch: 1,
            last_compacted_row_count: None,
        }
    }
}

pub(crate) fn snapshot_from_state(
    state: &std::collections::BTreeMap<SearchCorpusKind, SearchCorpusRuntime>,
) -> SearchPlaneStatusSnapshot {
    let corpora = SearchCorpusKind::ALL
        .iter()
        .filter_map(|corpus| state.get(corpus).map(|runtime| runtime.status.clone()))
        .collect();
    SearchPlaneStatusSnapshot {
        repo_read_pressure: None,
        corpora,
    }
}
