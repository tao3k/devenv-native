mod candidates;
mod error;
mod execution;
mod filters;
mod helpers;
mod route;
mod scan;

pub(crate) use candidates::RepoContentChunkCandidate;
pub(crate) use error::RepoContentChunkSearchError;
pub(crate) use filters::RepoContentChunkSearchFilters;
#[cfg(test)]
pub(crate) use helpers::{candidate_path_key, compare_candidates};
pub(crate) use route::search_repo_content_chunks_with_filters;
#[cfg(test)]
pub(crate) use scan::{build_repo_content_stage1_sql, retained_window};
