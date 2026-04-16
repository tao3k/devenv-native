mod build;
mod query;
mod schema;

#[cfg(any(test, feature = "performance"))]
pub(crate) use build::plan::repo_content_chunk_file_fingerprints;
pub(crate) use build::{publish_repo_content_chunks, publish_repo_content_chunks_incremental};
pub(crate) use query::{
    RepoContentChunkCandidate, RepoContentChunkSearchError, RepoContentChunkSearchFilters,
    search_repo_content_chunks_with_filters,
};
