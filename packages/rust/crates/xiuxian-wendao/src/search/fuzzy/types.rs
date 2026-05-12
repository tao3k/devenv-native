//! `search::fuzzy::types` owns Wendao search fuzzy types behavior.

/// One scored fuzzy match result.
#[derive(Debug, Clone)]
pub struct FuzzyMatch<T> {
    /// Matched item.
    pub item: T,
    /// The candidate text used for matching.
    pub matched_text: String,
    /// Normalized similarity score in `[0.0, 1.0]`.
    pub score: f32,
    /// Edit distance between query and candidate text.
    pub distance: usize,
}

/// A compact score result for one fuzzy comparison.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FuzzyScore {
    /// Normalized similarity score in `[0.0, 1.0]`.
    pub score: f32,
    /// Edit distance between the compared strings.
    pub distance: usize,
}

/// Shared fuzzy matcher abstraction.
pub trait FuzzyMatcher<T> {
    /// Error type emitted by the matcher.
    type Error;

    /// Search for fuzzy matches for one query.
    ///
    /// # Errors
    ///
    /// Returns the matcher-specific error when the underlying fuzzy-search
    /// backend cannot execute the query.
    fn search(&self, query: &str, limit: usize) -> Result<Vec<FuzzyMatch<T>>, Self::Error>;
}
