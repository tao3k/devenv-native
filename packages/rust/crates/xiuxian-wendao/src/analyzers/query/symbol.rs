//! `analyzers::query::symbol` owns Wendao analyzers query symbol behavior.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::module::RepoBacklinkItem;
use crate::analyzers::SymbolRecord;

/// Query for symbol lookup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SymbolSearchQuery {
    /// Repository identifier to search within.
    pub repo_id: String,
    /// User-provided search string.
    pub query: String,
    /// Maximum number of rows to return.
    pub limit: usize,
}

/// One enriched symbol-search hit with ranking and projection metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
/// Stringly state boundary: this public record preserves serialized catalog tokens from external or stored Wendao data.
pub struct SymbolSearchHit {
    /// The normalized symbol record.
    pub symbol: SymbolRecord,
    /// Optional normalized relevance score (0-1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    /// Optional stable rank in the returned hit set (1-based).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<usize>,
    /// Optional saliency score (0-1) for mixed-source ordering.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saliency_score: Option<f64>,
    /// Optional hierarchical URI for path mapping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hierarchical_uri: Option<String>,
    /// Optional hierarchy segments for breadcrumbs and drawers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hierarchy: Option<Vec<String>>,
    /// Optional implicit backlinks derived from `documents` relations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implicit_backlinks: Option<Vec<String>>,
    /// Optional structured backlink metadata derived from `documents` relations.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implicit_backlink_items: Option<Vec<RepoBacklinkItem>>,
    /// Optional projected-page identifiers that reference this symbol.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection_page_ids: Option<Vec<String>>,
    /// Optional audit status echoed from source records.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audit_status: Option<String>,
    /// Optional verification state derived from audit status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verification_state: Option<String>,
}

/// Result set for symbol lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SymbolSearchResult {
    /// Repository identifier searched.
    pub repo_id: String,
    /// Matching symbol rows.
    pub symbols: Vec<SymbolRecord>,
    /// Enriched symbol hits with ranking/backlink/projection context.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub symbol_hits: Vec<SymbolSearchHit>,
}
