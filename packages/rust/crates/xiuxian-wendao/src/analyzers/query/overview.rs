//! `analyzers::query::overview` owns Wendao analyzers query overview behavior.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Query for repository overview data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoOverviewQuery {
    /// Repository identifier to summarize.
    pub repo_id: String,
}

/// Minimal repository overview response for the MVP surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoOverviewResult {
    /// Repository identifier.
    pub repo_id: String,
    /// Primary display name.
    pub display_name: String,
    /// Optional revision string.
    pub revision: Option<String>,
    /// Count of normalized modules.
    pub module_count: usize,
    /// Count of normalized symbols.
    pub symbol_count: usize,
    /// Count of normalized examples.
    pub example_count: usize,
    /// Count of normalized docs.
    pub doc_count: usize,
    /// Optional repository-level hierarchical URI for path mapping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hierarchical_uri: Option<String>,
    /// Optional repository hierarchy segments for breadcrumbs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hierarchy: Option<Vec<String>>,
}
