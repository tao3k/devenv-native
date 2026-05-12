//! `search::repo_content_chunk::build::types` owns Wendao repo content chunk build types behavior.

use std::time::Duration;

use crate::repo_index::RepoCodeDocument;
use crate::search::{RepoStagedMutationAction, RepoStagedMutationPlan};

pub(crate) const REPO_CONTENT_CHUNK_EXTRACTOR_VERSION: u32 = 2;

pub(crate) type RepoContentChunkBuildAction = RepoStagedMutationAction<Vec<RepoCodeDocument>>;
pub(crate) type RepoContentChunkBuildPlan = RepoStagedMutationPlan<Vec<RepoCodeDocument>>;
/// `RepoContentChunkMutationWriteProfile` public type boundary for Wendao.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepoContentChunkMutationWriteProfile {
    pub touched_partition_count: usize,
    pub untouched_partition_count: usize,
    pub copy_untouched_elapsed: Duration,
    pub load_touched_elapsed: Duration,
    pub filter_replaced_elapsed: Duration,
    pub changed_payload_elapsed: Duration,
    pub write_touched_elapsed: Duration,
    pub write_snapshot_elapsed: Duration,
}
/// `RepoContentChunkFinalizeProfile` public type boundary for Wendao.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepoContentChunkFinalizeProfile {
    pub prewarm: Duration,
    pub record_publication: Duration,
    pub set_fingerprints: Duration,
}
/// `RepoContentChunkIncrementalPublishProfile` public type boundary for Wendao.

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RepoContentChunkIncrementalPublishProfile {
    pub previous_fingerprint_read_elapsed: Duration,
    pub current_record_read_elapsed: Duration,
    pub fingerprint_merge_elapsed: Duration,
    pub plan_elapsed: Duration,
    pub mutation_write: RepoContentChunkMutationWriteProfile,
    pub finalize: RepoContentChunkFinalizeProfile,
}
