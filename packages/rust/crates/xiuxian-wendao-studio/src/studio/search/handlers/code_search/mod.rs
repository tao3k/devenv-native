//! Coordinates the Studio search handlers code search branch and keeps its child modules behind one documented reasoning-tree boundary.

mod content;
#[path = "execution/mod.rs"]
mod execution;
mod query;
mod response_mapping;

#[cfg(test)]
pub(crate) use content::{
    CODE_CONTENT_EXCLUDE_GLOBS, is_supported_code_extension, parse_content_search_line,
    path_matches_language_filters, truncate_content_search_snippet,
};
pub(crate) use execution::build_code_search_response;
#[cfg(test)]
pub(crate) use execution::{
    build_code_search_cache_key, build_code_search_response_with_budget,
    build_repo_content_search_hits, build_repo_entity_search_hits,
};
#[cfg(test)]
pub(crate) use query::{
    RepoSearchResultLimits, infer_repo_hint_from_query, infer_repo_hint_from_repositories,
    query_uses_redundant_repo_seed, repo_search_result_limits, repo_wide_code_search_timeout,
};
#[cfg(test)]
pub(crate) use response_mapping::{repo_navigation_target, symbol_search_hit_to_search_hit};
