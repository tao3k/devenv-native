//! `analyzers::projection::search` owns Wendao analyzers projection search behavior.

mod heuristic;
#[cfg(feature = "repo-lexical-index")]
mod indexed;
mod lexical;
mod mapping;
mod options;
mod ranking;
mod sort;

#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
pub use indexed::build_projected_page_search_index;
#[cfg(all(feature = "search-runtime", feature = "repo-lexical-index"))]
pub use ranking::build_repo_projected_page_search_with_artifacts;
pub use ranking::{build_repo_projected_page_search, scored_projected_page_matches};

#[allow(unused_imports)]
pub use ranking::build_repo_projected_page_search_with_options;

#[cfg(all(test, feature = "repo-lexical-index", feature = "search-runtime"))]
#[path = "../../../../tests/unit/analyzers/projection/search/mod.rs"]
mod tests;
