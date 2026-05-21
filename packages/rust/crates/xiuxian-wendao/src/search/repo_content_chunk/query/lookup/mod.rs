//! `search::repo_content_chunk::query::lookup` owns Wendao repo content chunk query lookup behavior.

mod candidates;
mod error;
mod execution;
mod filters;
mod helpers;
mod route;
mod scan;

pub(crate) use candidates::RepoContentChunkCandidate;
#[cfg(test)]
pub(crate) use candidates::{candidate_path_key, compare_candidates};
pub use error::RepoContentChunkSearchError;
pub(crate) use filters::RepoContentChunkSearchFilters;
pub(crate) use route::search_repo_content_chunks_with_filters;
#[cfg(test)]
pub(crate) use scan::{
    build_repo_content_detail_sql, build_repo_content_stage1_sql, retained_window,
};
