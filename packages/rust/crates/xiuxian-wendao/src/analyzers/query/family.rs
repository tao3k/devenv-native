//! `analyzers::query::family` owns Wendao analyzers query family behavior.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::projection::{ProjectedPageRecord, ProjectionPageKind};

/// One deterministic page-family context entry ranked by shared stable anchors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProjectedPageFamilyContextEntry {
    /// Shared-anchor score between the center page and this related page.
    pub shared_anchor_score: usize,
    /// Deterministic projected page related to the center page.
    pub page: ProjectedPageRecord,
}

/// One deterministic page-family cluster around a projected page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProjectedPageFamilyCluster {
    /// Diataxis-aligned projected page family for this cluster.
    pub kind: ProjectionPageKind,
    /// Related projected pages in this family ordered by deterministic evidence.
    pub pages: Vec<ProjectedPageFamilyContextEntry>,
}

/// Query for deterministic Stage-2 page-family context around one stable projected page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageFamilyContextQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
    /// Maximum number of related projected pages to return for each page family.
    pub per_kind_limit: usize,
}

/// Deterministic Stage-2 page-family context result for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageFamilyContextResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// The requested center page.
    pub center_page: ProjectedPageRecord,
    /// Related projected pages grouped by projected page family.
    pub families: Vec<ProjectedPageFamilyCluster>,
}

/// One deterministic projected page-family search hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ProjectedPageFamilySearchHit {
    /// The matched projected center page.
    pub center_page: ProjectedPageRecord,
    /// Related projected pages grouped by page family around the matched page.
    pub families: Vec<ProjectedPageFamilyCluster>,
}

/// Query for deterministic Stage-2 page-family cluster search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageFamilySearchQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// User-provided projected-page search string for center pages.
    pub query: String,
    /// Optional projected-page family filter applied to center pages.
    pub kind: Option<ProjectionPageKind>,
    /// Maximum number of center-page hits to return.
    pub limit: usize,
    /// Maximum number of related projected pages to return for each page family.
    pub per_kind_limit: usize,
}

/// Deterministic Stage-2 page-family cluster search result for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageFamilySearchResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// Matching center pages with grouped deterministic family clusters.
    pub hits: Vec<ProjectedPageFamilySearchHit>,
}

/// Query for deterministic Stage-2 singular page-family cluster lookup.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageFamilyClusterQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
    /// Requested projected-page family for the returned cluster.
    pub kind: ProjectionPageKind,
    /// Maximum number of related projected pages to return in the requested family cluster.
    pub limit: usize,
}

/// Deterministic Stage-2 singular page-family cluster lookup result for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageFamilyClusterResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// The requested center page.
    pub center_page: ProjectedPageRecord,
    /// The requested related projected page family.
    pub family: ProjectedPageFamilyCluster,
}
