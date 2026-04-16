use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::module::RepoBacklinkItem;
use crate::analyzers::ExampleRecord;

/// Query for example lookup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ExampleSearchQuery {
    /// Repository identifier to search within.
    pub repo_id: String,
    /// User-provided search string.
    pub query: String,
    /// Maximum number of rows to return.
    pub limit: usize,
}

/// One enriched example-search hit with ranking and projection metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExampleSearchHit {
    /// The normalized example record.
    pub example: ExampleRecord,
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
    /// Optional projected-page identifiers that reference this example.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection_page_ids: Option<Vec<String>>,
}

/// Result set for example lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ExampleSearchResult {
    /// Repository identifier searched.
    pub repo_id: String,
    /// Matching example rows.
    pub examples: Vec<ExampleRecord>,
    /// Enriched example hits with ranking/backlink/projection context.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub example_hits: Vec<ExampleSearchHit>,
}
