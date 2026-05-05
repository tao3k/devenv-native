/// Effective repo-index runtime policy values for diagnostics and performance
/// probes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RepoIndexPolicyDebugSnapshot {
    /// Maximum duration allowed for one repository analysis attempt.
    pub analysis_timeout_secs: u64,
    /// Maximum duration allowed for one repository synchronization attempt.
    pub sync_timeout_secs: u64,
    /// Number of sync retry attempts allowed before failing the repository.
    pub sync_retry_budget: usize,
}

/// Return the effective repo-index policy values used by the current process.
#[must_use]
pub fn repo_index_policy_debug_snapshot() -> RepoIndexPolicyDebugSnapshot {
    RepoIndexPolicyDebugSnapshot {
        analysis_timeout_secs: super::state::repo_index_analysis_timeout().as_secs(),
        sync_timeout_secs: super::state::repo_index_sync_timeout().as_secs(),
        sync_retry_budget: super::state::repo_index_sync_requeue_attempt_limit(),
    }
}
