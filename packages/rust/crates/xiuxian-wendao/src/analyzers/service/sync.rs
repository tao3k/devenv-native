//! Repository synchronization functions.

use std::path::Path;

use chrono::Utc;
use xiuxian_git_repo::{
    LocalCheckoutMetadata, MaterializedRepo, RepoDriftState,
    RepoLifecycleState as SubstrateLifecycleState, RepoSourceKind as SubstrateSourceKind, SyncMode,
    discover_checkout_metadata,
};

use super::load_registered_repository;
use crate::analyzers::RegisteredRepository;
use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::resolve_registered_repository_source;
use crate::analyzers::{
    RepoSourceKind, RepoSyncDriftState, RepoSyncFreshnessSummary, RepoSyncHealthState,
    RepoSyncLifecycleSummary, RepoSyncMode, RepoSyncQuery, RepoSyncResult, RepoSyncRevisionSummary,
    RepoSyncStalenessState, RepoSyncState, RepoSyncStatusSummary,
};

/// Build a repository synchronization result from resolved source state.
#[must_use]
pub(crate) fn build_repo_sync(
    query: &RepoSyncQuery,
    repository: &RegisteredRepository,
    source: &MaterializedRepo,
    metadata: Option<LocalCheckoutMetadata>,
) -> RepoSyncResult {
    let metadata = metadata.unwrap_or_default();
    let checked_at = Utc::now();
    let source_kind = repo_source_kind(source.source_kind);
    let mirror_state = repo_sync_state(source.mirror_state);
    let checkout_state = repo_sync_state(source.checkout_state);
    let checked_at_string = checked_at.to_rfc3339();
    let last_fetched_at = source.last_fetched_at.clone();
    let mirror_revision = source.mirror_revision.clone();
    let tracking_revision = source.tracking_revision.clone();
    let upstream_url = repository.url.clone().or(metadata.remote_url);
    let drift_state = repo_sync_drift_state(source.drift_state);
    let health_state = repo_sync_health_state(source);
    let staleness_state = repo_sync_staleness_state(source, checked_at);
    let revision = metadata.revision;
    let status_summary = repo_sync_status_summary(
        source_kind,
        mirror_state,
        checkout_state,
        checked_at_string.as_str(),
        last_fetched_at.as_deref(),
        mirror_revision.as_deref(),
        tracking_revision.as_deref(),
        drift_state,
        health_state,
        staleness_state,
        revision.as_deref(),
    );

    RepoSyncResult {
        repo_id: query.repo_id.clone(),
        mode: query.mode,
        source_kind,
        refresh: repository.refresh,
        mirror_state,
        checkout_state,
        checkout_path: source.checkout_root.display().to_string(),
        mirror_path: source
            .mirror_root
            .as_ref()
            .map(|path| path.display().to_string()),
        checked_at: checked_at_string,
        last_fetched_at,
        mirror_revision,
        tracking_revision,
        upstream_url,
        drift_state,
        health_state,
        staleness_state,
        status_summary,
        revision,
    }
}

/// Load configuration, synchronize one repository source, and return source state.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when configuration loading or repository
/// source preparation fails.
pub fn repo_sync_from_config(
    query: &RepoSyncQuery,
    config_path: Option<&Path>,
    cwd: &Path,
) -> Result<RepoSyncResult, RepoIntelligenceError> {
    let repository = load_registered_repository(&query.repo_id, config_path, cwd)?;
    repo_sync_for_registered_repository(query, &repository, cwd)
}

/// Synchronize one already-resolved registered repository and return source state.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when repository source preparation fails.
pub fn repo_sync_for_registered_repository(
    query: &RepoSyncQuery,
    repository: &RegisteredRepository,
    cwd: &Path,
) -> Result<RepoSyncResult, RepoIntelligenceError> {
    let source =
        resolve_registered_repository_source(repository, cwd, checkout_sync_mode(query.mode))?;
    let metadata = discover_checkout_metadata(&source.checkout_root);
    Ok(build_repo_sync(query, repository, &source, metadata))
}

fn checkout_sync_mode(mode: RepoSyncMode) -> SyncMode {
    match mode {
        RepoSyncMode::Ensure => SyncMode::Ensure,
        RepoSyncMode::Refresh => SyncMode::Refresh,
        RepoSyncMode::Status => SyncMode::Status,
    }
}

fn repo_source_kind(state: SubstrateSourceKind) -> RepoSourceKind {
    match state {
        SubstrateSourceKind::LocalCheckout => RepoSourceKind::LocalCheckout,
        SubstrateSourceKind::ManagedRemote => RepoSourceKind::ManagedRemote,
    }
}

fn repo_sync_state(state: SubstrateLifecycleState) -> RepoSyncState {
    match state {
        SubstrateLifecycleState::NotApplicable => RepoSyncState::NotApplicable,
        SubstrateLifecycleState::Missing => RepoSyncState::Missing,
        SubstrateLifecycleState::Validated => RepoSyncState::Validated,
        SubstrateLifecycleState::Observed => RepoSyncState::Observed,
        SubstrateLifecycleState::Created => RepoSyncState::Created,
        SubstrateLifecycleState::Reused => RepoSyncState::Reused,
        SubstrateLifecycleState::Refreshed => RepoSyncState::Refreshed,
    }
}

fn repo_sync_drift_state(state: RepoDriftState) -> RepoSyncDriftState {
    match state {
        RepoDriftState::NotApplicable => RepoSyncDriftState::NotApplicable,
        RepoDriftState::Unknown => RepoSyncDriftState::Unknown,
        RepoDriftState::InSync => RepoSyncDriftState::InSync,
        RepoDriftState::Ahead => RepoSyncDriftState::Ahead,
        RepoDriftState::Behind => RepoSyncDriftState::Behind,
        RepoDriftState::Diverged => RepoSyncDriftState::Diverged,
    }
}

fn repo_sync_health_state(source: &MaterializedRepo) -> RepoSyncHealthState {
    match source.source_kind {
        SubstrateSourceKind::LocalCheckout => RepoSyncHealthState::Healthy,
        SubstrateSourceKind::ManagedRemote => {
            if matches!(source.mirror_state, SubstrateLifecycleState::Missing)
                || matches!(source.checkout_state, SubstrateLifecycleState::Missing)
            {
                return RepoSyncHealthState::MissingAssets;
            }

            match source.drift_state {
                RepoDriftState::NotApplicable | RepoDriftState::InSync => {
                    RepoSyncHealthState::Healthy
                }
                RepoDriftState::Ahead => RepoSyncHealthState::HasLocalCommits,
                RepoDriftState::Behind => RepoSyncHealthState::NeedsRefresh,
                RepoDriftState::Diverged => RepoSyncHealthState::Diverged,
                RepoDriftState::Unknown => RepoSyncHealthState::Unknown,
            }
        }
    }
}

fn repo_sync_staleness_state(
    source: &MaterializedRepo,
    checked_at: chrono::DateTime<Utc>,
) -> RepoSyncStalenessState {
    match source.source_kind {
        SubstrateSourceKind::LocalCheckout => RepoSyncStalenessState::NotApplicable,
        SubstrateSourceKind::ManagedRemote => {
            let Some(last_fetched_at) = source.last_fetched_at.as_deref() else {
                return RepoSyncStalenessState::Unknown;
            };
            let Ok(last_fetched_at) = chrono::DateTime::parse_from_rfc3339(last_fetched_at) else {
                return RepoSyncStalenessState::Unknown;
            };
            let age = checked_at.signed_duration_since(last_fetched_at.with_timezone(&Utc));
            if age < chrono::Duration::zero() {
                return RepoSyncStalenessState::Unknown;
            }
            if age < chrono::Duration::hours(1) {
                RepoSyncStalenessState::Fresh
            } else if age < chrono::Duration::hours(24) {
                RepoSyncStalenessState::Aging
            } else {
                RepoSyncStalenessState::Stale
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn repo_sync_status_summary(
    source_kind: RepoSourceKind,
    mirror_state: RepoSyncState,
    checkout_state: RepoSyncState,
    checked_at: &str,
    last_fetched_at: Option<&str>,
    mirror_revision: Option<&str>,
    tracking_revision: Option<&str>,
    drift_state: RepoSyncDriftState,
    health_state: RepoSyncHealthState,
    staleness_state: RepoSyncStalenessState,
    checkout_revision: Option<&str>,
) -> RepoSyncStatusSummary {
    let lifecycle = RepoSyncLifecycleSummary {
        source_kind,
        mirror_state,
        checkout_state,
        mirror_ready: !matches!(
            mirror_state,
            RepoSyncState::Missing | RepoSyncState::NotApplicable
        ),
        checkout_ready: !matches!(checkout_state, RepoSyncState::Missing),
    };
    let freshness = RepoSyncFreshnessSummary {
        checked_at: checked_at.to_string(),
        last_fetched_at: last_fetched_at.map(str::to_string),
        staleness_state,
    };
    let revisions = RepoSyncRevisionSummary {
        checkout_revision: checkout_revision.map(str::to_string),
        mirror_revision: mirror_revision.map(str::to_string),
        tracking_revision: tracking_revision.map(str::to_string),
        aligned_with_mirror: checkout_revision.is_some() && checkout_revision == mirror_revision,
    };

    RepoSyncStatusSummary {
        lifecycle,
        freshness,
        revisions,
        health_state,
        drift_state,
        attention_required: !matches!(health_state, RepoSyncHealthState::Healthy)
            || matches!(
                staleness_state,
                RepoSyncStalenessState::Stale | RepoSyncStalenessState::Unknown
            ),
    }
}
