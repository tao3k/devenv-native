//! Shared lexical fuzzy-search utilities.

mod buffers;
mod case;
mod distance;
mod matcher;
mod options;
mod scoring;
mod types;

pub use distance::{
    edit_distance, levenshtein_distance, passes_prefix_requirement, shared_prefix_len,
};
pub use matcher::LexicalMatcher;
pub use options::FuzzySearchOptions;
pub use scoring::{normalized_score, score_candidate};
pub use types::{FuzzyMatch, FuzzyMatcher, FuzzyScore};

#[cfg(feature = "repo-lexical-index")]
pub(crate) use scoring::score_candidate_with_query_chars;

#[cfg(test)]
#[path = "../../../tests/unit/search/fuzzy/mod.rs"]
mod tests;
