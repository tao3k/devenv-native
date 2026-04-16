use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::analyzers::RepositoryRefreshPolicy;

/// Query for repository source synchronization.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RepoSyncQuery {
    /// Repository identifier to synchronize.
    pub repo_id: String,
    /// Synchronization mode applied to the repository source lifecycle.
    #[serde(default)]
    pub mode: RepoSyncMode,
}

/// Synchronization mode for repository source preparation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RepoSyncMode {
    /// Prepare the repository source while respecting the configured refresh policy.
    #[default]
    Ensure,
    /// Force a remote refresh for managed repositories before returning source state.
    Refresh,
    /// Inspect repository source state without creating or refreshing managed assets.
    Status,
}

/// Source kind resolved for one repository.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum RepoSourceKind {
    /// A user-provided local checkout path.
    #[default]
    LocalCheckout,
    /// A managed checkout materialized from an upstream remote.
    ManagedRemote,
}

/// Lifecycle status reported for one repository source phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum RepoSyncState {
    /// No lifecycle phase was required for this repository source.
    #[default]
    NotApplicable,
    /// The lifecycle asset is expected but does not currently exist.
    Missing,
    /// An existing local checkout was validated without materialization.
    Validated,
    /// An existing lifecycle asset was observed without mutation.
    Observed,
    /// A new lifecycle asset was created.
    Created,
    /// An existing lifecycle asset was reused without refresh.
    Reused,
    /// An existing lifecycle asset was refreshed in place.
    Refreshed,
}

/// Drift summary between the managed mirror and managed checkout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum RepoSyncDriftState {
    /// Drift does not apply to this repository source kind.
    #[default]
    NotApplicable,
    /// Drift could not be determined from the currently available local metadata.
    Unknown,
    /// Mirror and checkout currently point at the same revision.
    InSync,
    /// The checkout has local commits ahead of the tracked mirror state.
    Ahead,
    /// The checkout is behind the current mirror state.
    Behind,
    /// The checkout and mirror both moved away from their last common tracked state.
    Diverged,
}

/// High-level health summary for one repository source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum RepoSyncHealthState {
    /// The repository source is ready for analysis and does not currently need action.
    #[default]
    Healthy,
    /// One or more managed source assets are missing from the local cache.
    MissingAssets,
    /// The managed checkout is behind the current mirror state and should be refreshed.
    NeedsRefresh,
    /// The managed checkout has local commits ahead of the tracked mirror state.
    HasLocalCommits,
    /// The managed checkout and managed mirror have diverged.
    Diverged,
    /// Health could not be determined from the currently available local metadata.
    Unknown,
}

/// Freshness summary for the managed mirror fetch timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum RepoSyncStalenessState {
    /// Freshness does not apply to this repository source kind.
    #[default]
    NotApplicable,
    /// Freshness could not be determined from the currently available local metadata.
    Unknown,
    /// The managed mirror was fetched within the last hour.
    Fresh,
    /// The managed mirror was fetched within the last day, but not within the last hour.
    Aging,
    /// The managed mirror has not been fetched in more than one day.
    Stale,
}

/// Grouped lifecycle view for one repository source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct RepoSyncLifecycleSummary {
    /// Resolved source kind.
    pub source_kind: RepoSourceKind,
    /// Lifecycle status for mirror preparation.
    pub mirror_state: RepoSyncState,
    /// Lifecycle status for checkout preparation.
    pub checkout_state: RepoSyncState,
    /// Whether a mirror asset is currently available for managed repositories.
    pub mirror_ready: bool,
    /// Whether a working checkout is currently available locally.
    pub checkout_ready: bool,
}

/// Grouped freshness view for one repository source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct RepoSyncFreshnessSummary {
    /// Observation timestamp for this sync or status operation.
    pub checked_at: String,
    /// Last local fetch timestamp observed from the managed mirror cache.
    pub last_fetched_at: Option<String>,
    /// Freshness summary derived from the local mirror fetch timestamp.
    pub staleness_state: RepoSyncStalenessState,
}

/// Grouped revision view for one repository source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct RepoSyncRevisionSummary {
    /// Active checkout revision after synchronization.
    pub checkout_revision: Option<String>,
    /// Active revision observed from the managed mirror branch or HEAD.
    pub mirror_revision: Option<String>,
    /// Last fetched remote-tracking revision observed from the managed checkout.
    pub tracking_revision: Option<String>,
    /// Whether the active checkout revision matches the managed mirror revision.
    pub aligned_with_mirror: bool,
}

/// Grouped status view for one repository source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct RepoSyncStatusSummary {
    /// Lifecycle view of the repository source.
    pub lifecycle: RepoSyncLifecycleSummary,
    /// Freshness view of the repository source.
    pub freshness: RepoSyncFreshnessSummary,
    /// Revision view of the repository source.
    pub revisions: RepoSyncRevisionSummary,
    /// High-level health summary derived from lifecycle and drift state.
    pub health_state: RepoSyncHealthState,
    /// Drift summary between the managed mirror and the working checkout.
    pub drift_state: RepoSyncDriftState,
    /// Whether the repository source likely needs operator attention.
    pub attention_required: bool,
}

/// Repository source synchronization result.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
pub struct RepoSyncResult {
    /// Repository identifier.
    pub repo_id: String,
    /// Synchronization mode that was applied.
    pub mode: RepoSyncMode,
    /// Resolved source kind.
    pub source_kind: RepoSourceKind,
    /// Refresh policy applied to the repository source.
    pub refresh: RepositoryRefreshPolicy,
    /// Lifecycle status for mirror preparation.
    pub mirror_state: RepoSyncState,
    /// Lifecycle status for checkout preparation.
    pub checkout_state: RepoSyncState,
    /// Absolute path to the working checkout used for analysis.
    pub checkout_path: String,
    /// Absolute path to the managed mirror, when remote materialization is used.
    pub mirror_path: Option<String>,
    /// Observation timestamp for this sync or status operation.
    pub checked_at: String,
    /// Last local fetch timestamp observed from the managed mirror cache.
    pub last_fetched_at: Option<String>,
    /// Active revision observed from the managed mirror branch or HEAD.
    pub mirror_revision: Option<String>,
    /// Last fetched remote-tracking revision observed from the managed checkout.
    pub tracking_revision: Option<String>,
    /// Upstream URL declared by configuration or discovered from the checkout.
    pub upstream_url: Option<String>,
    /// Drift summary between the managed mirror and the working checkout.
    pub drift_state: RepoSyncDriftState,
    /// High-level health summary derived from lifecycle and drift state.
    pub health_state: RepoSyncHealthState,
    /// Freshness summary derived from the local mirror fetch timestamp.
    pub staleness_state: RepoSyncStalenessState,
    /// Grouped status summary for agent-facing consumption.
    pub status_summary: RepoSyncStatusSummary,
    /// Active checkout revision after synchronization.
    pub revision: Option<String>,
}
