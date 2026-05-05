//! Repository index-status route contract and metadata validation.

/// Canonical repo-index-status repository metadata header for Wendao Flight requests.
pub const WENDAO_REPO_INDEX_STATUS_REPO_HEADER: &str = "x-wendao-repo-index-status-repo";
/// Stable route for the repo index-status analysis contract.
pub const ANALYSIS_REPO_INDEX_STATUS_ROUTE: &str = "/analysis/repo-index-status";

/// Validate the stable repo index-status request contract.
#[must_use]
pub fn validate_repo_index_status_request(repo_id: Option<&str>) -> Option<String> {
    repo_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}
