use std::collections::{BTreeMap, BTreeSet};

#[cfg(test)]
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

#[cfg(test)]
#[derive(Debug, thiserror::Error)]
pub(crate) enum KnowledgeSectionBuildError {
    #[error(transparent)]
    Storage(#[from] VectorStoreError),
}
