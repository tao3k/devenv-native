//! `parsers::search::repo_code_query` owns Wendao parsers search repo code query behavior.

mod api;
mod types;

pub use self::api::{
    REPO_CODE_SEARCH_BACKEND_PREFIXES, REPO_CODE_SEARCH_KIND_FILTER_VALUES,
    REPO_CODE_SEARCH_PREFIX_ALIASES, REPO_CODE_SEARCH_STRUCTURAL_PREFIXES,
    parse_repo_code_search_query, parse_repo_code_search_query_with_repo_hint,
};
pub use self::types::ParsedRepoCodeSearchQuery;

#[cfg(test)]
#[path = "../../../../tests/unit/parsers/search/repo_code_query.rs"]
mod tests;
