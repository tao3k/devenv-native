//! Repository sync route contract and metadata validation.

/// Canonical repo-sync repository metadata header for Wendao Flight requests.
pub const WENDAO_REPO_SYNC_REPO_HEADER: &str = "x-wendao-repo-sync-repo";
/// Canonical repo-sync mode metadata header for Wendao Flight requests.
pub const WENDAO_REPO_SYNC_MODE_HEADER: &str = "x-wendao-repo-sync-mode";
/// Stable route for the repo sync analysis contract.
pub const ANALYSIS_REPO_SYNC_ROUTE: &str = "/analysis/repo-sync";

/// Validate the stable repo sync request contract.
///
/// # Errors
///
/// Returns an error when the repository identifier is blank or when the sync
/// mode is unsupported.
pub fn validate_repo_sync_request(
    repo_id: &str,
    mode: Option<&str>,
) -> Result<(String, String), String> {
    let normalized_repo_id = repo_id.trim();
    if normalized_repo_id.is_empty() {
        return Err("repo sync repo must not be blank".to_string());
    }
    let normalized_mode = match mode
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("ensure")
    {
        "ensure" | "refresh" | "status" => mode
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("ensure")
            .to_string(),
        other => return Err(format!("unsupported repo sync mode `{other}`")),
    };
    Ok((normalized_repo_id.to_string(), normalized_mode))
}
