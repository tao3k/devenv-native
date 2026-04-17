mod lookup;
#[cfg(test)]
#[path = "../../../../tests/unit/search/repo_content_chunk/query/mod.rs"]
mod tests;

pub(crate) use lookup::{
    RepoContentChunkCandidate, RepoContentChunkSearchError, RepoContentChunkSearchFilters,
    search_repo_content_chunks_with_filters,
};
#[cfg(test)]
pub(crate) use lookup::{
    build_repo_content_detail_sql, build_repo_content_stage1_sql, candidate_path_key,
    compare_candidates, retained_window,
};
