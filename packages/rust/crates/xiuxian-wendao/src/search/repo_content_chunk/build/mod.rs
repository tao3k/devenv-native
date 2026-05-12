//! `search::repo_content_chunk::build` owns Wendao search repo content chunk build behavior.

#[path = "orchestration.rs"]
pub(crate) mod orchestration;
#[path = "partitions.rs"]
pub(crate) mod partitions;
#[path = "plan.rs"]
pub(crate) mod plan;
#[cfg(test)]
#[path = "../../../../tests/unit/search/repo_content_chunk/build/mod/mod.rs"]
mod tests;
#[path = "types.rs"]
pub(crate) mod types;
#[path = "write.rs"]
pub(crate) mod write;

#[cfg(feature = "search-runtime")]
pub(crate) use orchestration::publish_repo_content_chunks_incremental_profiled;
pub(crate) use orchestration::{
    publish_repo_content_chunks, publish_repo_content_chunks_incremental,
};
