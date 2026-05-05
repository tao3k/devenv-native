mod example;
mod module;
mod shared;
mod symbol;

#[cfg(test)]
#[path = "../../../../../tests/unit/analyzers/service/search/ranking/mod.rs"]
mod tests;

pub(crate) use example::ranked_example_matches;
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
pub use example::ranked_example_matches_with_artifacts;
pub(crate) use module::ranked_module_matches;
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
pub use module::ranked_module_matches_with_artifacts;
pub(crate) use shared::{
    EXAMPLE_SEARCH_BUCKETS, MODULE_SEARCH_BUCKETS, RankedSearchRecord, SYMBOL_SEARCH_BUCKETS,
    search_candidate_limit,
};
pub(crate) use symbol::ranked_symbol_matches;
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
pub use symbol::ranked_symbol_matches_with_artifacts;
