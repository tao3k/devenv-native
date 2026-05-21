//! Link-graph filter payloads for structural and relatedness search.

use super::enums::{LinkGraphEdgeType, LinkGraphPprSubgraphMode, LinkGraphScope};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Boolean tag filter.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphTagFilter {
    /// Note must contain ALL tags in this list.
    #[serde(default)]
    pub all: Vec<String>,
    /// Note must contain AT LEAST ONE tag in this list.
    #[serde(default)]
    pub any: Vec<String>,
    /// Note must NOT contain any tag in this list.
    #[serde(default)]
    pub not_tags: Vec<String>,
}

/// Link-traversal filter used for `link_to` and `linked_by` constraints.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphLinkFilter {
    /// One or more seed note stems or identifiers.
    #[serde(default)]
    pub seeds: Vec<String>,
    /// Whether to negate the match.
    #[serde(default)]
    pub negate: bool,
    /// Whether to allow recursive traversal.
    #[serde(default)]
    pub recursive: bool,
    /// Maximum distance for recursive traversal.
    #[serde(default)]
    pub max_distance: Option<usize>,
}

/// Personalized `PageRank` options for related-note discovery.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphRelatedPprOptions {
    /// PPR teleport probability.
    pub alpha: Option<f64>,
    /// Maximum number of power iterations.
    pub max_iter: Option<usize>,
    /// Convergence tolerance.
    pub tol: Option<f64>,
    /// Large-graph subgraph computation strategy.
    pub subgraph_mode: Option<LinkGraphPprSubgraphMode>,
}

/// Related-note discovery filter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphRelatedFilter {
    /// One or more seed note stems or identifiers.
    #[serde(default)]
    pub seeds: Vec<String>,
    /// Maximum hop distance for discovery.
    #[serde(default)]
    pub max_distance: Option<usize>,
    /// Optional PPR refinement parameters.
    #[serde(default)]
    pub ppr: Option<LinkGraphRelatedPprOptions>,
}

/// Structured search filters for link-graph retrieval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkGraphSearchFilters {
    /// Search scope.
    pub scope: Option<LinkGraphScope>,
    /// Max heading level for section matches.
    pub max_heading_level: Option<usize>,
    /// Max hops allowed for recursive tree matching.
    pub max_tree_hops: Option<usize>,
    /// Whether to collapse multiple section matches into a single document hit.
    pub collapse_to_doc: Option<bool>,
    /// Note must be in one of these subdirectories.
    #[serde(default)]
    pub include_paths: Vec<String>,
    /// Note must NOT be in any of these subdirectories.
    #[serde(default)]
    pub exclude_paths: Vec<String>,
    /// Note must contain a direct wikilink/reference to these documents.
    #[serde(default)]
    pub link_to: Option<LinkGraphLinkFilter>,
    /// Note must be referenced by these documents.
    #[serde(default)]
    pub linked_by: Option<LinkGraphLinkFilter>,
    /// Discovery of semantically related notes via graph traversal.
    #[serde(default)]
    pub related: Option<LinkGraphRelatedFilter>,
    /// Boolean tag filter.
    #[serde(default)]
    pub tags: Option<LinkGraphTagFilter>,
    /// Note must contain these explicit mentions.
    #[serde(default)]
    pub mentions_of: Vec<String>,
    /// Note must be mentioned by these explicit documents.
    #[serde(default)]
    pub mentioned_by_notes: Vec<String>,
    /// Filter for notes with no incoming or outgoing links.
    #[serde(default)]
    pub orphan: bool,
    /// Filter for notes with no tags.
    #[serde(default)]
    pub tagless: bool,
    /// Filter for documents with missing backlink metadata.
    #[serde(default)]
    pub missing_backlink: bool,
    /// Allowed edge types for structural/semantic traversal.
    #[serde(default)]
    pub edge_types: Vec<LinkGraphEdgeType>,
    /// Optional section cap per document.
    #[serde(default)]
    pub per_doc_section_cap: Option<usize>,
    /// Optional minimum words for section hits.
    #[serde(default)]
    pub min_section_words: Option<usize>,
}
