use std::collections::{BTreeMap, BTreeSet};

use crate::gateway::studio::types::AttachmentSearchHit;
use crate::search::SearchFileFingerprint;

#[derive(Debug, Clone)]
pub(crate) struct AttachmentBuildPlan {
    pub(crate) base_epoch: Option<u64>,
    pub(crate) file_fingerprints: BTreeMap<String, SearchFileFingerprint>,
    pub(crate) replaced_paths: BTreeSet<String>,
    pub(crate) changed_hits: Vec<AttachmentSearchHit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AttachmentWriteResult {
    pub(crate) row_count: u64,
    pub(crate) fragment_count: u64,
}

#[cfg(test)]
#[derive(Debug, thiserror::Error)]
pub(crate) enum AttachmentBuildError {
    #[error(transparent)]
    Storage(#[from] xiuxian_db_store::VectorStoreError),
}
