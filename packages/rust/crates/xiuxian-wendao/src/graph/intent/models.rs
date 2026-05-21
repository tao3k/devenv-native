//! Data model for graph query intent routing.

/// Structured intent parsed from a user query.
#[derive(Debug, Clone, Default)]
pub struct QueryIntent {
    /// Primary action verb (e.g. "search", "commit", "create")
    pub action: Option<String>,
    /// Target domain or object (e.g. "git", "knowledge", "file")
    pub target: Option<String>,
    /// Context qualifiers (e.g. "python", "async")
    pub context: Vec<String>,
    /// All significant keywords (stop-words removed)
    pub keywords: Vec<String>,
    /// Original query, lower-cased and trimmed
    pub normalized_query: String,
}
