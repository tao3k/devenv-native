//! `search::local_symbol::build::types` owns Wendao local symbol build types behavior.

use std::collections::{BTreeMap, BTreeSet};

use crate::search::SearchFileFingerprint;
use crate::search::contracts::AstSearchHit;

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
/// `LocalSymbolBuildError` public enum boundary for Wendao.

#[cfg(any(test, feature = "test-support"))]
#[derive(Debug, thiserror::Error)]
pub enum LocalSymbolBuildError {
    #[error("local symbol build was not started for fingerprint `{0}`")]
    BuildRejected(String),
    #[error(transparent)]
    Storage(#[from] xiuxian_db_store::VectorStoreError),
}
