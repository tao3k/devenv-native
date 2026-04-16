use std::path::Path;

use chrono::Utc;

use crate::analyzers::{
    RegisteredRepository, RepositoryRef, RepositoryRefreshPolicy,
    repo_sync_for_registered_repository,
};
use crate::analyzers::{
    RepoSourceKind, RepoSyncHealthState, RepoSyncMode, RepoSyncResult, RepoSyncStalenessState,
};
use crate::repo_index::state::coordinator::RepoIndexCoordinator;
use crate::repo_index::state::fingerprint::fingerprint;
use crate::repo_index::types::{RepoIndexEntryStatus, RepoIndexPhase};
use xiuxian_git_repo::{ManagedRemoteProbeStatus, discover_managed_remote_probe_state};

impl RepoIndexCoordinator {
    pub(crate) fn hydrate_repositories_from_search_plane(
        &self,
        repositories: &[RegisteredRepository],
    ) {
        let queued_or_active = self
            .queued_or_active
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let existing_fingerprints = self
            .fingerprints
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let repo_ids = repositories
            .iter()
            .filter(|repository| !queued_or_active.contains(&repository.id))
            .filter(|repository| !existing_fingerprints.contains_key(&repository.id))
            .map(|repository| repository.id.clone())
            .collect::<Vec<_>>();
        if repo_ids.is_empty() {
            return;
        }

        let bootstrap_statuses = self.search_plane.repo_index_bootstrap_statuses(&repo_ids);
        if bootstrap_statuses.is_empty() {
            return;
        }

        let mut statuses = self
            .statuses
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut fingerprints = self
            .fingerprints
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let mut status_changed = false;

        for repository in repositories {
            if queued_or_active.contains(&repository.id)
                || fingerprints.contains_key(&repository.id)
            {
                continue;
            }
            let Some(status) = bootstrap_statuses.get(&repository.id).cloned() else {
                continue;
            };
            if !self.repository_is_safe_to_hydrate(repository, &status) {
                continue;
            }

            fingerprints.insert(repository.id.clone(), fingerprint(repository));
            if statuses.get(&repository.id) != Some(&status) {
                statuses.insert(repository.id.clone(), status);
                status_changed = true;
            }
        }
        drop(fingerprints);
        drop(statuses);

        if status_changed {
            self.refresh_status_snapshot();
        }
    }

    fn repository_is_safe_to_hydrate(
        &self,
        repository: &RegisteredRepository,
        status: &RepoIndexEntryStatus,
    ) -> bool {
        if !persisted_publication_bootstrap_is_searchable(status) {
            return false;
        }
        if repository.url.is_none() {
            return true;
        }

        let Some(sync_result) = self.managed_remote_status(repository) else {
            return true;
        };
        managed_remote_bootstrap_is_safe(
            repository,
            status,
            &sync_result,
            managed_remote_probe_freshness(&sync_result),
            managed_remote_retryable_probe_failure_is_recent(&sync_result),
        ) || persisted_publication_bootstrap_is_searchable(status)
    }

    fn managed_remote_status(&self, repository: &RegisteredRepository) -> Option<RepoSyncResult> {
        repo_sync_for_registered_repository(
            &crate::analyzers::RepoSyncQuery {
                repo_id: repository.id.clone(),
                mode: RepoSyncMode::Status,
            },
            repository,
            self.project_root.as_path(),
        )
        .ok()
    }
}

fn persisted_publication_bootstrap_is_searchable(status: &RepoIndexEntryStatus) -> bool {
    matches!(status.phase, RepoIndexPhase::Ready)
}

fn managed_remote_bootstrap_is_safe(
    repository: &RegisteredRepository,
    status: &RepoIndexEntryStatus,
    sync_result: &RepoSyncResult,
    probe_freshness: Option<RepoSyncStalenessState>,
    recent_retryable_probe_failure: bool,
) -> bool {
    if !matches!(status.phase, RepoIndexPhase::Ready) {
        return false;
    }
    if sync_result.source_kind != RepoSourceKind::ManagedRemote {
        return false;
    }
    if sync_result.health_state != RepoSyncHealthState::Healthy {
        return false;
    }
    if sync_result.revision.as_deref() != status.last_revision.as_deref() {
        return false;
    }

    match repository.refresh {
        RepositoryRefreshPolicy::Manual => true,
        RepositoryRefreshPolicy::Fetch
            if matches!(repository.git_ref, Some(RepositoryRef::Commit(_))) =>
        {
            true
        }
        RepositoryRefreshPolicy::Fetch => {
            matches!(
                sync_result.staleness_state,
                RepoSyncStalenessState::Fresh | RepoSyncStalenessState::Aging
            ) || matches!(
                probe_freshness,
                Some(RepoSyncStalenessState::Fresh | RepoSyncStalenessState::Aging)
            ) || recent_retryable_probe_failure
        }
    }
}

fn managed_remote_probe_freshness(sync_result: &RepoSyncResult) -> Option<RepoSyncStalenessState> {
    let mirror_root = Path::new(sync_result.mirror_path.as_deref()?);
    let probe_state = discover_managed_remote_probe_state(mirror_root)?;
    let success_checked_at = if probe_state.status == ManagedRemoteProbeStatus::Success {
        Some(probe_state.checked_at.as_str())
    } else {
        probe_state.last_success_checked_at.as_deref()
    }?;
    let success_target_revision = if probe_state.status == ManagedRemoteProbeStatus::Success {
        probe_state.target_revision.as_deref()
    } else {
        probe_state.last_success_target_revision.as_deref()
    };
    if success_target_revision != sync_result.revision.as_deref() {
        return None;
    }
    probe_staleness_state(success_checked_at, Utc::now())
}

fn managed_remote_retryable_probe_failure_is_recent(sync_result: &RepoSyncResult) -> bool {
    let Some(mirror_path) = sync_result.mirror_path.as_deref() else {
        return false;
    };
    let Some(probe_state) = discover_managed_remote_probe_state(Path::new(mirror_path)) else {
        return false;
    };
    if probe_state.status != ManagedRemoteProbeStatus::RetryableFailure {
        return false;
    }
    matches!(
        probe_staleness_state(probe_state.checked_at.as_str(), Utc::now()),
        Some(RepoSyncStalenessState::Fresh)
    )
}

fn probe_staleness_state(
    checked_at: &str,
    now: chrono::DateTime<Utc>,
) -> Option<RepoSyncStalenessState> {
    let checked_at = chrono::DateTime::parse_from_rfc3339(checked_at)
        .ok()?
        .with_timezone(&Utc);
    let age = now.signed_duration_since(checked_at);
    if age < chrono::Duration::zero() {
        return None;
    }
    if age < chrono::Duration::hours(1) {
        return Some(RepoSyncStalenessState::Fresh);
    }
    if age < chrono::Duration::hours(24) {
        return Some(RepoSyncStalenessState::Aging);
    }
    Some(RepoSyncStalenessState::Stale)
}

#[cfg(test)]
#[path = "../../../../tests/unit/repo_index/state/coordinator/hydration.rs"]
mod tests;
