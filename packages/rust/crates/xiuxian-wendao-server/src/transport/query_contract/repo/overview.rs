//! Repository overview route contract and metadata validation.

use crate::transport::query_contract::RepoIdRef;

/// Canonical repo-overview repository metadata header for Wendao Flight requests.
pub const WENDAO_REPO_OVERVIEW_REPO_HEADER: &str = "x-wendao-repo-overview-repo";
/// Stable route for the repo overview analysis contract.
pub const ANALYSIS_REPO_OVERVIEW_ROUTE: &str = "/analysis/repo-overview";

/// Validate the stable repo overview request contract.
///
/// # Errors
///
/// Returns an error when the repository identifier is blank.
pub fn validate_repo_overview_request(repo_id: RepoIdRef<'_>) -> Result<String, String> {
    let normalized_repo_id = repo_id.trim();
    if normalized_repo_id.is_empty() {
        return Err("repo overview repo must not be blank".to_string());
    }
    Ok(normalized_repo_id.to_string())
}
