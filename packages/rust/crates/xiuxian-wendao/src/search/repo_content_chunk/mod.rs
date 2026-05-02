#[path = "build/mod.rs"]
mod build;
#[path = "query/mod.rs"]
mod query;
mod schema;

#[cfg(feature = "search-runtime")]
pub(crate) use build::partitions::{
    repo_content_chunk_partition_count_for_document_count, repo_content_chunk_partition_id_for_path,
};
#[cfg(any(test, feature = "performance"))]
pub(crate) use build::plan::repo_content_chunk_file_fingerprints;
#[cfg(feature = "search-runtime")]
pub(crate) use build::publish_repo_content_chunks_incremental_profiled;
#[cfg(feature = "search-runtime")]
pub(crate) use build::types::RepoContentChunkIncrementalPublishProfile;
pub(crate) use build::{publish_repo_content_chunks, publish_repo_content_chunks_incremental};
pub(crate) use query::{
    RepoContentChunkCandidate, RepoContentChunkSearchError, RepoContentChunkSearchFilters,
    search_repo_content_chunks_with_filters,
};
