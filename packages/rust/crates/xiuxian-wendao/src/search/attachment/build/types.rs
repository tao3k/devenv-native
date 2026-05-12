//! `search::attachment::build::types` owns Wendao attachment build types behavior.

use std::collections::{BTreeMap, BTreeSet};

use crate::search::SearchFileFingerprint;
use crate::search::contracts::AttachmentSearchHit;

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
/// `AttachmentBuildError` public enum boundary for Wendao.

#[cfg(any(test, feature = "test-support"))]
#[derive(Debug, thiserror::Error)]
pub enum AttachmentBuildError {
    #[error(transparent)]
    Storage(#[from] xiuxian_db_store::VectorStoreError),
}
