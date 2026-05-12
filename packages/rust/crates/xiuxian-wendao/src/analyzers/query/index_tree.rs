//! `analyzers::query::index_tree` owns Wendao analyzers query index tree behavior.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::projection::{ProjectedPageIndexTree, ProjectionPageKind};

/// Deterministic local context around one projected page-index node hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct ProjectedPageIndexNodeContext {
    /// Ancestor nodes ordered from root to immediate parent.
    pub ancestors: Vec<ProjectedPageIndexNodeHit>,
    /// Previous sibling node within the same parent scope.
    pub previous_sibling: Option<ProjectedPageIndexNodeHit>,
    /// Next sibling node within the same parent scope.
    pub next_sibling: Option<ProjectedPageIndexNodeHit>,
    /// Direct child nodes under the requested node.
    pub children: Vec<ProjectedPageIndexNodeHit>,
}

/// Query for deterministic projected page-index trees.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageIndexTreesQuery {
    /// Repository identifier to project.
    pub repo_id: String,
}

/// Query for deterministic projected page-index tree lookup by stable page identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageIndexTreeQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
}

/// Deterministic projected page-index tree lookup result for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct RepoProjectedPageIndexTreeResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// The requested deterministic projected page-index tree.
    pub tree: Option<ProjectedPageIndexTree>,
}

/// Query for deterministic projected page-index node lookup by stable identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::struct_field_names)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct RepoProjectedPageIndexNodeQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
    /// Stable page-index node identifier.
    pub node_id: String,
}

/// One deterministic section-level retrieval hit inside a projected page-index tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct ProjectedPageIndexNodeHit {
    /// Owning repository identifier.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
    /// Human-readable projected page title.
    pub page_title: String,
    /// Diataxis-aligned projected page family.
    pub page_kind: ProjectionPageKind,
    /// Virtual markdown path used for parsing the projected page.
    pub path: String,
    /// Parsed document identifier as seen by the markdown parser.
    pub doc_id: String,
    /// Stable page-index node identifier.
    pub node_id: String,
    /// Human-readable node title.
    pub node_title: String,
    /// Structural path carried by the page-index builder.
    pub structural_path: Vec<String>,
    /// Inclusive 1-based source line range.
    pub line_range: (usize, usize),
    /// Node text payload after optional thinning.
    pub text: String,
}

/// Deterministic projected page-index node lookup result for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct RepoProjectedPageIndexNodeResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// The requested deterministic projected page-index node hit.
    pub hit: Option<ProjectedPageIndexNodeHit>,
}

/// Query for deterministic projected page-index tree retrieval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageIndexTreeSearchQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// User-provided search string.
    pub query: String,
    /// Optional projected-page family filter.
    pub kind: Option<ProjectionPageKind>,
    /// Maximum number of projected page-index node hits to return.
    pub limit: usize,
}

/// Deterministic projected page-index tree retrieval result set for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageIndexTreeSearchResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// Matching deterministic section-level hits.
    pub hits: Vec<ProjectedPageIndexNodeHit>,
}

/// Deterministic projected page-index tree result set for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageIndexTreesResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// Deterministic projected page-index trees derived from repository truth.
    pub trees: Vec<ProjectedPageIndexTree>,
}
