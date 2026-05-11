//! Enumerated link-graph query labels exposed through schemas.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Neighbor direction relative to the queried note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LinkGraphDirection {
    /// Points to current note.
    Incoming,
    /// Referenced by current note.
    Outgoing,
    /// Reachable from both sides.
    Both,
}

impl LinkGraphDirection {
    /// Parse direction aliases from user/runtime input.
    #[must_use]
    pub fn from_alias(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "to" | "incoming" => Self::Incoming,
            "from" | "outgoing" => Self::Outgoing,
            _ => Self::Both,
        }
    }
}

/// Search strategy used by link-graph search.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LinkGraphMatchStrategy {
    /// Default lexical ranking (token/substring aware).
    Fts,
    /// Path + heading fuzzy ranking (structure-aware).
    #[serde(rename = "path_fuzzy")]
    PathFuzzy,
    /// Exact match on id/stem/title/path/tag.
    Exact,
    /// Regex match on id/stem/title/path/tag.
    Re,
}

impl LinkGraphMatchStrategy {
    /// Parse strategy aliases from user/runtime input.
    #[must_use]
    pub fn from_alias(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "path_fuzzy" | "path-fuzzy" | "pathfuzzy" | "fuzzy" => Self::PathFuzzy,
            "exact" => Self::Exact,
            "re" | "regex" => Self::Re,
            _ => Self::Fts,
        }
    }
}

/// Schema-first sort field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LinkGraphSortField {
    /// Relevance score.
    Score,
    /// Relative path.
    Path,
    /// Display title.
    Title,
    /// File stem.
    Stem,
    /// Created timestamp.
    Created,
    /// Modified timestamp.
    Modified,
    /// Deterministic random key.
    Random,
    /// Document word count.
    WordCount,
}

/// Schema-first sort order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum LinkGraphSortOrder {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

/// Related filter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LinkGraphPprSubgraphMode {
    /// Let engine choose when to partition subgraphs.
    Auto,
    /// Disable subgraph partitioning.
    Disabled,
    /// Force subgraph partitioning.
    Force,
}

/// Result scope for doc/section level retrieval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LinkGraphScope {
    /// Return document-level hits only.
    DocOnly,
    /// Return section-level hits only.
    SectionOnly,
    /// Return both document and section hits.
    Mixed,
}

/// Edge type filters for tree-aware traversal/ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum LinkGraphEdgeType {
    /// Structural links (for example wiki-links, markdown references).
    Structural,
    /// Semantic edges inferred by higher-level processors.
    Semantic,
    /// Provisional edges awaiting promotion.
    Provisional,
    /// Verified edges from trusted workflows.
    Verified,
    /// Property drawer edges: explicit references from property attributes.
    PropertyDrawer,
}
