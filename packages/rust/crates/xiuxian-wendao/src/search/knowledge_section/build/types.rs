//! `search::knowledge_section::build::types` owns Wendao knowledge section build types behavior.

use std::collections::{BTreeMap, BTreeSet};

#[cfg(any(test, feature = "test-support"))]
use xiuxian_db_store::VectorStoreError;

use crate::search::SearchFileFingerprint;
use crate::search::knowledge_section::schema::KnowledgeSectionRow;

#[derive(Debug, Clone)]
pub(super) struct KnowledgeSectionBuildPlan {
    pub(super) base_epoch: Option<u64>,
    pub(super) file_fingerprints: BTreeMap<String, SearchFileFingerprint>,
    pub(super) replaced_paths: BTreeSet<String>,
    pub(super) changed_rows: Vec<KnowledgeSectionRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct KnowledgeSectionWriteResult {
    pub(super) row_count: u64,
    pub(super) fragment_count: u64,
}
/// `KnowledgeSectionBuildError` public enum boundary for Wendao.

#[cfg(any(test, feature = "test-support"))]
#[derive(Debug, thiserror::Error)]
pub enum KnowledgeSectionBuildError {
    #[error(transparent)]
    Storage(#[from] VectorStoreError),
}
