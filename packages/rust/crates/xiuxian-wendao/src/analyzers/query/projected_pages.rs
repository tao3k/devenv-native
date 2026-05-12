//! `analyzers::query::projected_pages` owns Wendao analyzers query projected pages behavior.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::projection::{ProjectedPageRecord, ProjectionPageKind};

/// Query for deterministic projected pages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPagesQuery {
    /// Repository identifier to project.
    pub repo_id: String,
}

/// Deterministic projected-page result set for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPagesResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// Deterministic projected pages derived from repository truth.
    pub pages: Vec<ProjectedPageRecord>,
}

/// Query for deterministic projected-page lookup by stable page identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// Stable projected page identifier.
    pub page_id: String,
}

/// Deterministic projected-page lookup result for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// The requested deterministic projected page.
    pub page: ProjectedPageRecord,
}

/// Query for deterministic projected-page retrieval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageSearchQuery {
    /// Repository identifier to project.
    pub repo_id: String,
    /// User-provided projected-page search string.
    pub query: String,
    /// Optional projected-page family filter.
    pub kind: Option<ProjectionPageKind>,
    /// Maximum number of projected pages to return.
    pub limit: usize,
}

/// Deterministic projected-page search result set for one repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoProjectedPageSearchResult {
    /// Repository identifier projected.
    pub repo_id: String,
    /// Matching deterministic projected pages.
    pub pages: Vec<ProjectedPageRecord>,
}
