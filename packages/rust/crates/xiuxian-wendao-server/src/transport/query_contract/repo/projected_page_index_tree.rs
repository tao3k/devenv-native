//! Projected page-index tree route contract and metadata validation.

use crate::transport::query_contract::{PageIdRef, RepoIdRef};

/// Canonical projected page-index tree repository metadata header for Wendao
/// Flight requests.
pub const WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_REPO_HEADER: &str =
    "x-wendao-repo-projected-page-index-tree-repo";
/// Canonical projected page-index tree page metadata header for Wendao Flight
/// requests.
pub const WENDAO_REPO_PROJECTED_PAGE_INDEX_TREE_PAGE_ID_HEADER: &str =
    "x-wendao-repo-projected-page-index-tree-page-id";
/// Stable route for the repo projected page-index tree analysis contract.
pub const ANALYSIS_REPO_PROJECTED_PAGE_INDEX_TREE_ROUTE: &str =
    "/analysis/repo-projected-page-index-tree";

/// Normalized projected page-index tree request metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoProjectedPageIndexTreeRequest {
    /// Normalized repository identifier.
    pub repo_id: String,
    /// Normalized projected page identifier.
    pub page_id: String,
}

impl PartialEq<(String, String)> for RepoProjectedPageIndexTreeRequest {
    fn eq(&self, other: &(String, String)) -> bool {
        self.repo_id == other.0 && self.page_id == other.1
    }
}

/// Validate the stable projected page-index tree request contract.
///
/// # Errors
///
/// Returns an error when the repository identifier or page identifier is
/// blank.
pub fn validate_repo_projected_page_index_tree_request(
    repo_id: RepoIdRef<'_>,
    page_id: PageIdRef<'_>,
) -> Result<RepoProjectedPageIndexTreeRequest, String> {
    let normalized_repo_id = repo_id.trim();
    if normalized_repo_id.is_empty() {
        return Err("repo projected page-index tree repo must not be blank".to_string());
    }
    let normalized_page_id = page_id.trim();
    if normalized_page_id.is_empty() {
        return Err("repo projected page-index tree page id must not be blank".to_string());
    }
    Ok(RepoProjectedPageIndexTreeRequest {
        repo_id: normalized_repo_id.to_string(),
        page_id: normalized_page_id.to_string(),
    })
}
