//! Projected retrieval-context route contract and metadata validation.

/// Canonical projected retrieval-context repository metadata header for Wendao
/// Flight requests.
pub const WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_REPO_HEADER: &str =
    "x-wendao-repo-projected-retrieval-context-repo";
/// Canonical projected retrieval-context page metadata header for Wendao
/// Flight requests.
pub const WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_PAGE_ID_HEADER: &str =
    "x-wendao-repo-projected-retrieval-context-page-id";
/// Canonical projected retrieval-context node metadata header for Wendao
/// Flight requests.
pub const WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_NODE_ID_HEADER: &str =
    "x-wendao-repo-projected-retrieval-context-node-id";
/// Canonical projected retrieval-context related-limit metadata header for
/// Wendao Flight requests.
pub const WENDAO_REPO_PROJECTED_RETRIEVAL_CONTEXT_RELATED_LIMIT_HEADER: &str =
    "x-wendao-repo-projected-retrieval-context-related-limit";
/// Stable route for the repo projected retrieval-context analysis contract.
pub const ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE: &str =
    "/analysis/repo-projected-retrieval-context";
/// Stable default related page limit for projected retrieval-context requests.
pub const REPO_PROJECTED_RETRIEVAL_CONTEXT_DEFAULT_RELATED_LIMIT: usize = 5;

/// Validate the stable projected retrieval-context request contract.
///
/// # Errors
///
/// Returns an error when the repository identifier or page identifier is
/// blank, or when the related-limit value is zero.
pub fn validate_repo_projected_retrieval_context_request(
    repo_id: &str,
    page_id: &str,
    node_id: Option<&str>,
    related_limit: Option<usize>,
) -> Result<(String, String, Option<String>, usize), String> {
    let normalized_repo_id = repo_id.trim();
    if normalized_repo_id.is_empty() {
        return Err("repo projected retrieval-context repo must not be blank".to_string());
    }
    let normalized_page_id = page_id.trim();
    if normalized_page_id.is_empty() {
        return Err("repo projected retrieval-context page id must not be blank".to_string());
    }
    let normalized_node_id = node_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let related_limit =
        related_limit.unwrap_or(REPO_PROJECTED_RETRIEVAL_CONTEXT_DEFAULT_RELATED_LIMIT);
    if related_limit == 0 {
        return Err(
            "repo projected retrieval-context related_limit must be greater than zero".to_string(),
        );
    }
    Ok((
        normalized_repo_id.to_string(),
        normalized_page_id.to_string(),
        normalized_node_id,
        related_limit,
    ))
}
