//! Repository sync route contract and metadata validation.

use crate::transport::query_contract::RepoIdRef;

/// Canonical repo-sync repository metadata header for Wendao Flight requests.
pub const WENDAO_REPO_SYNC_REPO_HEADER: &str = "x-wendao-repo-sync-repo";
/// Canonical repo-sync mode metadata header for Wendao Flight requests.
pub const WENDAO_REPO_SYNC_MODE_HEADER: &str = "x-wendao-repo-sync-mode";
/// Stable route for the repo sync analysis contract.
pub const ANALYSIS_REPO_SYNC_ROUTE: &str = "/analysis/repo-sync";

/// Normalized repo-sync mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoSyncMode {
    /// Ensure the repo is indexed.
    Ensure,
    /// Refresh repo index data.
    Refresh,
    /// Return repo sync status.
    Status,
}

impl RepoSyncMode {
    /// Return the canonical mode token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ensure => "ensure",
            Self::Refresh => "refresh",
            Self::Status => "status",
        }
    }
}

/// Normalized repo-sync request metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepoSyncRequest {
    /// Normalized repository identifier.
    pub repo_id: String,
    /// Normalized sync mode.
    pub mode: RepoSyncMode,
}

impl PartialEq<(String, String)> for RepoSyncRequest {
    fn eq(&self, other: &(String, String)) -> bool {
        self.repo_id == other.0 && self.mode.as_str() == other.1
    }
}

/// Validate the stable repo sync request contract.
///
/// # Errors
///
/// Returns an error when the repository identifier is blank or when the sync
/// mode is unsupported.
pub fn validate_repo_sync_request(
    repo_id: RepoIdRef<'_>,
    mode: Option<&str>,
) -> Result<RepoSyncRequest, String> {
    let normalized_repo_id = repo_id.trim();
    if normalized_repo_id.is_empty() {
        return Err("repo sync repo must not be blank".to_string());
    }
    let normalized_mode = match mode
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("ensure")
    {
        "ensure" => RepoSyncMode::Ensure,
        "refresh" => RepoSyncMode::Refresh,
        "status" => RepoSyncMode::Status,
        other => return Err(format!("unsupported repo sync mode `{other}`")),
    };
    Ok(RepoSyncRequest {
        repo_id: normalized_repo_id.to_string(),
        mode: normalized_mode,
    })
}
