//! Repository index route contract and metadata validation.

use crate::transport::query_contract::{RepoIdRef, RequestIdRef};

/// Canonical repo-index repository metadata header for Wendao Flight requests.
pub const WENDAO_REPO_INDEX_REPO_HEADER: &str = "x-wendao-repo-index-repo";
/// Canonical repo-index refresh metadata header for Wendao Flight requests.
pub const WENDAO_REPO_INDEX_REFRESH_HEADER: &str = "x-wendao-repo-index-refresh";
/// Canonical repo-index request identifier metadata header for Wendao Flight requests.
pub const WENDAO_REPO_INDEX_REQUEST_ID_HEADER: &str = "x-wendao-repo-index-request-id";
/// Stable route for the repo index analysis contract.
pub const ANALYSIS_REPO_INDEX_ROUTE: &str = "/analysis/repo-index";

/// Validate the stable repo index request contract.
///
/// # Errors
///
/// Returns an error when the optional refresh flag is not a canonical boolean
/// or when the request identifier is blank.
pub fn validate_repo_index_request(
    repo_id: Option<RepoIdRef<'_>>,
    refresh: Option<&str>,
    request_id: RequestIdRef<'_>,
) -> Result<(Option<String>, bool, String), String> {
    let normalized_repo_id = repo_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let normalized_refresh = match refresh
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("false")
    {
        "true" => true,
        "false" => false,
        other => return Err(format!("unsupported repo index refresh flag `{other}`")),
    };
    let normalized_request_id = request_id.trim();
    if normalized_request_id.is_empty() {
        return Err("repo index request id must not be blank".to_string());
    }
    Ok((
        normalized_repo_id,
        normalized_refresh,
        normalized_request_id.to_string(),
    ))
}
