use std::collections::{BTreeMap, BTreeSet};

use crate::gateway::studio::types::ReferenceSearchHit;
use crate::search::SearchFileFingerprint;

#[derive(Debug, Clone)]
pub(crate) struct ReferenceOccurrenceBuildPlan {
    pub(crate) base_epoch: Option<u64>,
    pub(crate) file_fingerprints: BTreeMap<String, SearchFileFingerprint>,
    pub(crate) replaced_paths: BTreeSet<String>,
    pub(crate) changed_hits: Vec<ReferenceSearchHit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ReferenceOccurrenceWriteResult {
    pub(crate) row_count: u64,
    pub(crate) fragment_count: u64,
}

#[cfg(test)]
#[derive(Debug, thiserror::Error)]
pub(crate) enum ReferenceOccurrenceBuildError {
    #[error(transparent)]
    Storage(#[from] xiuxian_db_store::VectorStoreError),
}
