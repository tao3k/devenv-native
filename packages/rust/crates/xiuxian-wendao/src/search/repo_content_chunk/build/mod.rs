pub(crate) mod orchestration;
pub(crate) mod partitions;
pub(crate) mod plan;
#[cfg(test)]
#[path = "../../../../tests/unit/search/repo_content_chunk/build/mod.rs"]
mod tests;
pub(crate) mod types;
pub(crate) mod write;

#[cfg(any(test, feature = "performance"))]
pub(crate) use orchestration::publish_repo_content_chunks_incremental_profiled;
pub(crate) use orchestration::{
    publish_repo_content_chunks, publish_repo_content_chunks_incremental,
};
