use std::collections::{BTreeMap, BTreeSet};

use crate::gateway::studio::types::AstSearchHit;
use crate::search::SearchFileFingerprint;

#[derive(Debug, Clone, Default)]
pub(crate) struct LocalSymbolPartitionBuildPlan {
    pub(crate) replaced_paths: BTreeSet<String>,
    pub(crate) changed_hits: Vec<AstSearchHit>,
}

#[derive(Debug, Clone)]
pub(crate) struct LocalSymbolBuildPlan {
    pub(crate) base_epoch: Option<u64>,
    pub(crate) file_fingerprints: BTreeMap<String, SearchFileFingerprint>,
    pub(crate) partitions: BTreeMap<String, LocalSymbolPartitionBuildPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalSymbolWriteResult {
    pub(crate) row_count: u64,
    pub(crate) fragment_count: u64,
}

#[cfg(test)]
#[derive(Debug, thiserror::Error)]
pub(crate) enum LocalSymbolBuildError {
    #[error("local symbol build was not started for fingerprint `{0}`")]
    BuildRejected(String),
    #[error(transparent)]
    Storage(#[from] xiuxian_db_store::VectorStoreError),
}
