//! Coordinates the Studio handlers code search execution branch and keeps its child modules behind one documented reasoning-tree boundary.

mod repo_search;
mod response;

#[cfg(test)]
pub(crate) use repo_search::{build_repo_content_search_hits, build_repo_entity_search_hits};
#[cfg(test)]
pub(crate) use response::build_code_search_cache_key;
pub(crate) use response::build_code_search_response;
#[cfg(test)]
pub(crate) use response::build_code_search_response_with_budget;
