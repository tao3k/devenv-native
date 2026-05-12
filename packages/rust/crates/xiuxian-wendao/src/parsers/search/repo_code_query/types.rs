//! `parsers::search::repo_code_query::types` owns Wendao search repo code query types behavior.

#[derive(Debug, Default, Clone, PartialEq, Eq)]
/// Parsed filters and residual text for a repository code search query.
pub struct ParsedRepoCodeSearchQuery {
    /// Explicit repository filter, or the caller-provided repository hint.
    pub repo: Option<String>,
    /// Lowercase language filters parsed from `lang:` terms.
    pub language_filters: std::collections::HashSet<String>,
    /// Lowercase repository item kind filters parsed from `kind:` terms.
    pub kind_filters: std::collections::HashSet<String>,
    /// Structural parser pattern parsed from `ast:` or `sg:` terms.
    pub ast_pattern: Option<String>,
    /// Residual full-text search term after filter tokens are removed.
    pub search_term: Option<String>,
}

impl ParsedRepoCodeSearchQuery {
    /// Return the residual full-text search term, if one was parsed.
    #[must_use]
    pub fn search_term(&self) -> Option<&str> {
        self.search_term.as_deref()
    }
}
