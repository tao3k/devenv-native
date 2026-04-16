use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::projection::{
    ProjectedMarkdownDocument, ProjectedPageIndexDocument, ProjectionPageKind,
};
use crate::analyzers::{
    RepoProjectedPageFamilyClusterResult, RepoProjectedPageFamilyContextResult,
    RepoProjectedPageFamilySearchResult, RepoProjectedPageIndexNodeResult,
    RepoProjectedPageIndexTreeResult, RepoProjectedPageIndexTreeSearchResult,
    RepoProjectedPageIndexTreesResult, RepoProjectedPageNavigationResult,
    RepoProjectedPageNavigationSearchResult, RepoProjectedPageResult,
    RepoProjectedPageSearchResult, RepoProjectedRetrievalContextResult,
    RepoProjectedRetrievalHitResult, RepoProjectedRetrievalResult,
};

/// Docs-facing query for deterministic projected-page search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsSearchQuery {
    /// Repository identifier to search.
    pub repo_id: String,
    /// User-provided projected-page search string.
    pub query: String,
    /// Optional projected-page family filter.
    pub kind: Option<ProjectionPageKind>,
    /// Maximum number of projected pages to return.
    pub limit: usize,
}

/// Docs-facing deterministic projected-page search result.
pub type DocsSearchResult = RepoProjectedPageSearchResult;

/// Docs-facing query for deterministic mixed projected retrieval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsRetrievalQuery {
    /// Repository identifier to search.
    pub repo_id: String,
    /// User-provided retrieval search string.
    pub query: String,
    /// Optional projected-page family filter.
    pub kind: Option<ProjectionPageKind>,
    /// Maximum number of mixed retrieval hits to return.
    pub limit: usize,
}

/// Docs-facing deterministic mixed projected retrieval result.
pub type DocsRetrievalResult = RepoProjectedRetrievalResult;

/// Docs-facing query for deterministic mixed projected retrieval context around one stable hit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsRetrievalContextQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
    /// Optional stable page-index node identifier.
    pub node_id: Option<String>,
    /// Maximum number of related projected pages to return.
    pub related_limit: usize,
}

/// Docs-facing deterministic mixed projected retrieval-context result.
pub type DocsRetrievalContextResult = RepoProjectedRetrievalContextResult;

/// Docs-facing query for deterministic mixed projected retrieval hit reopening.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsRetrievalHitQuery {
    /// Repository identifier to project.
    pub repo: String,
    /// Stable projected page identifier.
    pub page: String,
    /// Optional stable page-index node identifier.
    pub node: Option<String>,
}

/// Docs-facing deterministic mixed projected retrieval-hit result.
pub type DocsRetrievalHitResult = RepoProjectedRetrievalHitResult;

/// Docs-facing query for deterministic projected-page lookup by stable page identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPageQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
}

/// Docs-facing deterministic projected-page lookup result.
pub type DocsPageResult = RepoProjectedPageResult;

/// Docs-facing query for deterministic projected markdown documents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsMarkdownDocumentsQuery {
    /// Repository identifier to project.
    pub repo_id: String,
}

/// Docs-facing deterministic projected markdown documents result set for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsMarkdownDocumentsResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// Deterministic projected markdown documents derived from repository truth.
    pub documents: Vec<ProjectedMarkdownDocument>,
}

/// Docs-facing query for deterministic projected page-index tree lookup by stable page identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPageIndexTreeQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
}

/// Docs-facing deterministic projected page-index tree lookup result.
pub type DocsPageIndexTreeResult = RepoProjectedPageIndexTreeResult;

/// Docs-facing query for deterministic projected page-index trees.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPageIndexTreesQuery {
    /// Repository identifier to project.
    pub repo_id: String,
}

/// Docs-facing deterministic projected page-index trees result.
pub type DocsPageIndexTreesResult = RepoProjectedPageIndexTreesResult;

/// Docs-facing query for deterministic projected page-index documents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPageIndexDocumentsQuery {
    /// Repository identifier to project.
    pub repo_id: String,
}

/// Docs-facing deterministic projected page-index documents result set for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPageIndexDocumentsResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// Parsed page-index-ready documents derived from repository truth.
    pub documents: Vec<ProjectedPageIndexDocument>,
}

/// Docs-facing query for deterministic projected page-index tree search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsPageIndexTreeSearchQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// User-provided page-index tree search string.
    pub query: String,
    /// Optional projected-page family filter applied to candidate pages.
    pub kind: Option<ProjectionPageKind>,
    /// Maximum number of page-index tree hits to return.
    pub limit: usize,
}

/// Docs-facing deterministic projected page-index tree search result.
pub type DocsPageIndexTreeSearchResult = RepoProjectedPageIndexTreeSearchResult;

/// Docs-facing query for deterministic projected page-index node lookup by stable identifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[allow(clippy::struct_field_names)]
pub struct DocsPageIndexNodeQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
    /// Stable page-index node identifier.
    pub node_id: String,
}

/// Docs-facing deterministic projected page-index node lookup result.
pub type DocsPageIndexNodeResult = RepoProjectedPageIndexNodeResult;

/// Docs-facing query for deterministic projected-page family context around one stable page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsFamilyContextQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
    /// Maximum number of related projected pages to return for each page family.
    pub per_kind_limit: usize,
}

/// Docs-facing deterministic projected-page family-context result.
pub type DocsFamilyContextResult = RepoProjectedPageFamilyContextResult;

/// Docs-facing query for deterministic projected-page family search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsFamilySearchQuery {
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

/// Docs-facing deterministic projected-page family-search result.
pub type DocsFamilySearchResult = RepoProjectedPageFamilySearchResult;

/// Docs-facing query for deterministic projected-page family cluster around one stable page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsFamilyClusterQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
    /// Requested projected-page family for the returned cluster.
    pub kind: ProjectionPageKind,
    /// Maximum number of related projected pages to return in the requested family cluster.
    pub limit: usize,
}

/// Docs-facing deterministic projected-page family-cluster result.
pub type DocsFamilyClusterResult = RepoProjectedPageFamilyClusterResult;

/// Docs-facing query for deterministic projected-page navigation around one stable page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsNavigationQuery {
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

/// Docs-facing deterministic projected-page navigation bundle.
pub type DocsNavigationResult = RepoProjectedPageNavigationResult;

/// Docs-facing query for deterministic projected-page navigation search.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DocsNavigationSearchQuery {
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

/// Docs-facing deterministic projected-page navigation search result.
pub type DocsNavigationSearchResult = RepoProjectedPageNavigationSearchResult;
