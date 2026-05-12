//! `analyzers::query::retrieval` owns Wendao analyzers query retrieval behavior.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::index_tree::{ProjectedPageIndexNodeContext, ProjectedPageIndexNodeHit};
use crate::analyzers::projection::{ProjectedPageRecord, ProjectionPageKind};

/// Retrieval hit family emitted by deterministic Stage-2 mixed retrieval.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ProjectedRetrievalHitKind {
    /// A projected-page level hit.
    Page,
    /// A builder-native projected page-index node hit.
    PageIndexNode,
}

/// One deterministic Stage-2 mixed retrieval hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProjectedRetrievalHit {
    /// Retrieval hit family.
    pub kind: ProjectedRetrievalHitKind,
    /// Owning projected page record.
    pub page: ProjectedPageRecord,
    /// Optional builder-native projected page-index node hit.
    pub node: Option<ProjectedPageIndexNodeHit>,
}

/// Query for deterministic Stage-2 mixed retrieval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedRetrievalQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// User-provided retrieval search string.
    pub query: String,
    /// Optional projected-page family filter.
    pub kind: Option<ProjectionPageKind>,
    /// Maximum number of mixed retrieval hits to return.
    pub limit: usize,
}

/// Deterministic Stage-2 mixed retrieval result set for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedRetrievalResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// Matching deterministic projected-page and page-index-node hits.
    pub hits: Vec<ProjectedRetrievalHit>,
}

/// Query for deterministic Stage-2 mixed retrieval hit lookup by stable identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::struct_field_names)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct RepoProjectedRetrievalHitQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
    /// Optional stable page-index node identifier.
    pub node_id: Option<String>,
}

/// Deterministic Stage-2 mixed retrieval hit lookup result for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedRetrievalHitResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// The requested deterministic mixed retrieval hit.
    pub hit: ProjectedRetrievalHit,
}

/// Query for deterministic Stage-2 retrieval context around one stable hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct RepoProjectedRetrievalContextQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
    /// Optional stable page-index node identifier.
    pub node_id: Option<String>,
    /// Maximum number of related projected pages to return.
    pub related_limit: usize,
}

/// Deterministic Stage-2 retrieval context result for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedRetrievalContextResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// The requested center hit.
    pub center: ProjectedRetrievalHit,
    /// Related projected pages sharing stable anchors with the center page.
    pub related_pages: Vec<ProjectedPageRecord>,
    /// Optional builder-native node neighborhood when `node_id` is present.
    pub node_context: Option<ProjectedPageIndexNodeContext>,
}
