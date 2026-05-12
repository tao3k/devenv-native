//! `search::tantivy::document` owns Wendao search tantivy document behavior.

/// One shared search document stored in Tantivy.
#[derive(Debug, Clone, PartialEq, Eq)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct SearchDocument {
    /// Stable identifier used to map search hits back into domain records.
    pub id: String,
    /// Primary title or symbol name.
    pub title: String,
    /// Domain-specific kind label.
    pub kind: String,
    /// Stable path or location for the record.
    pub path: String,
    /// Coarse search scope such as repo or source.
    pub scope: String,
    /// Secondary namespace such as crate or document identifier.
    pub namespace: String,
    /// Additional searchable terms.
    pub terms: Vec<String>,
}

/// One fuzzy/exact search hit rehydrated from shared Tantivy documents.
#[derive(Debug, Clone, PartialEq)]
pub struct SearchDocumentHit {
    /// Stable identifier used to rehydrate domain records from caller-owned lookups.
    pub id: String,
    /// Best-matching stored field when available.
    pub matched_field: Option<SearchDocumentMatchField>,
    /// Best-matching text fragment when available.
    pub matched_text: Option<String>,
    /// Tantivy/reranked score for the hit.
    pub score: f32,
    /// Edit distance for fuzzy hits. Exact and prefix hits use `0`.
    pub distance: usize,
}

/// Stored search field labels used by shared hit rehydration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchDocumentMatchField {
    /// The document title.
    Title,
    /// The document path.
    Path,
    /// The document namespace.
    Namespace,
    /// Additional document terms.
    Terms,
}
