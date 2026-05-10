//! Owns the Studio repo query family surface.

use serde::Deserialize;

use crate::contracts::StudioContractKind;

/// Query parameters for projected page-family context lookup.
#[derive(Debug, Deserialize)]
pub struct RepoProjectedPageFamilyContextApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
    /// The projected page identifier.
    pub page_id: Option<String>,
    /// Maximum number of items per kind to return.
    pub per_kind_limit: Option<usize>,
}

/// Query parameters for projected page-family cluster search.
#[derive(Debug, Deserialize)]
pub struct RepoProjectedPageFamilySearchApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
    /// The search query string.
    pub query: Option<String>,
    /// The projected page kind filter.
    pub kind: Option<StudioContractKind>,
    /// Maximum number of hits to return.
    pub limit: Option<usize>,
    /// Maximum number of items per kind to return.
    pub per_kind_limit: Option<usize>,
}

/// Query parameters for projected page-family cluster lookup.
#[derive(Debug, Deserialize)]
pub struct RepoProjectedPageFamilyClusterApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
    /// The projected page identifier.
    pub page_id: Option<String>,
    /// The projected page kind filter.
    pub kind: Option<StudioContractKind>,
    /// Maximum number of hits to return.
    pub limit: Option<usize>,
}

/// Query parameters for projected page navigation bundle lookup.
#[derive(Debug, Deserialize)]
pub struct RepoProjectedPageNavigationApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
    /// The projected page identifier.
    pub page_id: Option<String>,
    /// The focus node identifier.
    pub node_id: Option<String>,
    /// The family kind filter.
    pub family_kind: Option<StudioContractKind>,
    /// Maximum number of related hits to return.
    pub related_limit: Option<usize>,
    /// Maximum number of family items to return.
    pub family_limit: Option<usize>,
}

/// Query parameters for projected page navigation search.
#[derive(Debug, Deserialize)]
pub struct RepoProjectedPageNavigationSearchApiQuery {
    /// The repository identifier.
    pub repo: Option<String>,
    /// The search query string.
    pub query: Option<String>,
    /// The projected page kind filter.
    pub kind: Option<StudioContractKind>,
    /// The family kind filter.
    pub family_kind: Option<StudioContractKind>,
    /// Maximum number of hits to return.
    pub limit: Option<usize>,
    /// Maximum number of related hits to return.
    pub related_limit: Option<usize>,
    /// Maximum number of family items to return.
    pub family_limit: Option<usize>,
}
