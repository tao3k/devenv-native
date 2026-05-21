//! `analyzers::query::navigation` owns Wendao analyzers query navigation behavior.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::family::ProjectedPageFamilyCluster;
use super::index_tree::ProjectedPageIndexNodeContext;
use super::retrieval::ProjectedRetrievalHit;
use crate::analyzers::projection::{
    ProjectedPageIndexTree, ProjectedPageRecord, ProjectionPageKind,
};

/// Query for deterministic Stage-2 page-centric navigation around one stable projected page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
/// Raw DTO boundary: this public record mirrors serialized Wendao transport fields.
pub struct RepoProjectedPageNavigationQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
    /// Optional stable page-index node identifier.
    pub node_id: Option<String>,
    /// Optional projected-page family to include as a deterministic cluster.
    pub family_kind: Option<ProjectionPageKind>,
    /// Maximum number of related projected pages to return.
    pub related_limit: usize,
    /// Maximum number of related projected pages to return in the requested family cluster.
    pub family_limit: usize,
}

/// Deterministic Stage-2 page-centric navigation bundle for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct RepoProjectedPageNavigationResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// The requested center hit.
    pub center: Option<ProjectedRetrievalHit>,
    /// Related projected pages sharing stable anchors with the center page.
    pub related_pages: Vec<ProjectedPageRecord>,
    /// Optional builder-native node neighborhood when `node_id` is present.
    pub node_context: Option<ProjectedPageIndexNodeContext>,
    /// Builder-native projected page-index tree for the requested page.
    pub tree: Option<ProjectedPageIndexTree>,
    /// Optional deterministic related projected page family for the requested page.
    pub family_cluster: Option<ProjectedPageFamilyCluster>,
}

/// One deterministic projected page-navigation search hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProjectedPageNavigationSearchHit {
    /// Stable ordering score derived from the projected page match.
    pub search_score: u8,
    /// Deterministic page-centric navigation bundle for the matched projected page.
    pub navigation: RepoProjectedPageNavigationResult,
}

/// Query for deterministic projected page-navigation search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageNavigationSearchQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// User-provided projected-page search string for center pages.
    pub query: String,
    /// Optional projected-page family filter applied to center pages.
    pub kind: Option<ProjectionPageKind>,
    /// Optional projected-page family to include as a deterministic cluster for each hit.
    pub family_kind: Option<ProjectionPageKind>,
    /// Maximum number of center-page hits to return.
    pub limit: usize,
    /// Maximum number of related projected pages to return for each matched center page.
    pub related_limit: usize,
    /// Maximum number of related projected pages to return in the requested family cluster.
    pub family_limit: usize,
}

/// Deterministic projected page-navigation search result for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageNavigationSearchResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// Matching center pages with deterministic navigation bundles.
    pub hits: Vec<ProjectedPageNavigationSearchHit>,
}
