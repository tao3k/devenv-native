use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::ModuleRecord;

/// Query for module lookup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ModuleSearchQuery {
    /// Repository identifier to search within.
    pub repo_id: String,
    /// User-provided search string.
    pub query: String,
    /// Maximum number of rows to return.
    pub limit: usize,
}

/// Structured backlink metadata derived from relation records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoBacklinkItem {
    /// Stable backlink identifier (typically a doc id).
    pub id: String,
    /// Optional display title of the backlink source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Optional repository-relative path of the backlink source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Optional relation kind label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// One enriched module-search hit with ranking and projection metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ModuleSearchHit {
    /// The normalized module record.
    pub module: ModuleRecord,
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
    /// Optional projected-page identifiers that reference this module.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection_page_ids: Option<Vec<String>>,
}

/// Result set for module lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ModuleSearchResult {
    /// Repository identifier searched.
    pub repo_id: String,
    /// Matching module rows.
    pub modules: Vec<ModuleRecord>,
    /// Enriched module hits with ranking/backlink/projection context.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub module_hits: Vec<ModuleSearchHit>,
}
