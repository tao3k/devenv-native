//! `search::fuzzy::options` owns Wendao search fuzzy options behavior.

/// Shared fuzzy-search configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuzzySearchOptions {
    /// Maximum edit distance allowed for a match.
    pub max_distance: u8,
    /// Number of leading characters that must match exactly.
    pub prefix_length: usize,
    /// Whether adjacent transpositions count as one edit.
    pub transposition: bool,
}

impl FuzzySearchOptions {
    /// Create a new fuzzy-search configuration.
    #[must_use]
    pub const fn new(max_distance: u8, prefix_length: usize, transposition: bool) -> Self {
        Self {
            max_distance,
            prefix_length,
            transposition,
        }
    }

    /// Default option profile for symbol-like identifiers.
    #[must_use]
    pub const fn symbol_search() -> Self {
        Self::new(1, 1, true)
    }

    /// Default option profile for CamelCase-style symbol abbreviations.
    #[must_use]
    pub const fn camel_case_symbol() -> Self {
        Self::new(1, 0, true)
    }

    /// Default option profile for document-like titles and phrases.
    #[must_use]
    pub const fn document_search() -> Self {
        Self::new(2, 0, true)
    }

    /// Default option profile for path/title discovery.
    #[must_use]
    pub const fn path_search() -> Self {
        Self::document_search()
    }
}

impl Default for FuzzySearchOptions {
    fn default() -> Self {
        Self::symbol_search()
    }
}
