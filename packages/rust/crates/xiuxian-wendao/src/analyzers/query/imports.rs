use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::ImportRecord;

/// Query for import lookup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ImportSearchQuery {
    /// Repository identifier to search within.
    pub repo_id: String,
    /// Optional package name filter (find imports of specific package).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub package: Option<String>,
    /// Optional module filter (find imports within specific module).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    /// Maximum number of rows to return.
    pub limit: usize,
}

/// One enriched import-search hit with ranking metadata.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ImportSearchHit {
    /// The normalized import record.
    pub import: ImportRecord,
    /// Optional normalized relevance score (0-1).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub score: Option<f64>,
    /// Optional stable rank in the returned hit set (1-based).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rank: Option<usize>,
}

/// Result set for import lookup.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ImportSearchResult {
    /// Repository identifier searched.
    pub repo_id: String,
    /// Matching import rows.
    pub imports: Vec<ImportRecord>,
    /// Enriched import hits with ranking context.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub import_hits: Vec<ImportSearchHit>,
}
