use std::collections::HashMap;

#[cfg(any(test, feature = "test-support"))]
use crate::repo_index::types::RepoIndexSnapshot;
use crate::repo_index::types::{RepoIndexEntryStatus, RepoIndexPhase, RepoIndexStatusResponse};
#[cfg(any(test, feature = "test-support"))]
use std::sync::Arc;

use crate::repo_index::state::coordinator::RepoIndexCoordinator;
use crate::repo_index::state::filters::{aggregate_status_response, filter_status_response};
use crate::repo_index::state::fingerprint::timestamp_now;
#[cfg(test)]
use crate::repo_index::state::task::AdaptiveConcurrencyController;
use crate::repo_index::state::task::RepoIndexTask;

impl RepoIndexCoordinator {
    /// Return the latest coordinator status, optionally filtered to one repository.
    /// Primitive boundary: this public API keeps raw Wendao identifier carriers for existing transport and query contracts.
    pub fn status_response(&self, repo_id: Option<&str>) -> RepoIndexStatusResponse {
        let snapshot = self
            .status_snapshot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        filter_status_response(snapshot, repo_id)
    }

    pub(crate) fn bump_status(
        &self,
        repo_id: &str,
        phase: RepoIndexPhase,
        last_revision: Option<String>,
        last_error: Option<String>,
    ) {
        self.record_repo_status(repo_id, phase, last_revision, last_error);
    }

    fn next_attempt_count(&self, repo_id: &str) -> usize {
        self.statuses
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(repo_id)
            .map_or(1, |status| status.attempt_count.saturating_add(1))
    }

    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    pub fn set_snapshot_for_test(&self, _snapshot: &Arc<RepoIndexSnapshot>) {
        let _ = &self.status_snapshot;
    }

    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    pub fn set_status_for_test(&self, status: RepoIndexEntryStatus) {
        self.statuses
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(status.repo_id.clone(), status);
        self.refresh_status_snapshot();
    }

    #[cfg(test)]
    pub(crate) fn set_concurrency_for_test(&self, controller: AdaptiveConcurrencyController) {
        *self
            .concurrency
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = controller;
        self.refresh_status_snapshot();
    }

    #[cfg(test)]
    pub(crate) fn mark_active_for_test(&self, repo_id: &str) {
        self.mark_active(repo_id);
    }

    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    pub fn pending_repo_ids_for_test(&self) -> Vec<String> {
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|task| task.repository.id.clone())
            .collect()
    }

    pub(crate) fn record_repo_status(
        &self,
        repo_id: &str,
        phase: RepoIndexPhase,
        last_revision: Option<String>,
        last_error: Option<String>,
    ) {
        let attempt_count = self.next_attempt_count(repo_id);
        self.statuses
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                repo_id.to_string(),
                RepoIndexEntryStatus {
                    repo_id: repo_id.to_string(),
                    phase,
                    queue_position: None,
                    last_error,
                    last_revision,
                    updated_at: Some(timestamp_now()),
                    attempt_count,
                },
            );
        self.refresh_status_snapshot();
    }

    pub(crate) fn record_failure_status(
        &self,
        repo_id: &str,
        error: &crate::analyzers::RepoIntelligenceError,
        last_revision: Option<String>,
    ) {
        let phase = if matches!(
            error,
            crate::analyzers::RepoIntelligenceError::UnsupportedRepositoryLayout { .. }
        ) {
            RepoIndexPhase::Unsupported
        } else {
            RepoIndexPhase::Failed
        };
        self.record_repo_status(repo_id, phase, last_revision, Some(error.to_string()));
    }

    pub(crate) fn refresh_status_snapshot(&self) {
        let queue_positions = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .enumerate()
            .map(|(index, task): (usize, &RepoIndexTask)| {
                (task.repository.id.clone(), index.saturating_add(1))
            })
            .collect::<HashMap<_, _>>();
        let repos = self
            .statuses
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
            .cloned()
            .map(|mut status| {
                status.queue_position = if matches!(status.phase, RepoIndexPhase::Queued) {
                    queue_positions.get(&status.repo_id).copied()
                } else {
                    None
                };
                status
            })
            .collect::<Vec<_>>();
        let active_repo_ids = self
            .active_repo_ids
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        let concurrency = self
            .concurrency
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .snapshot();
        let snapshot = aggregate_status_response(
            repos,
            active_repo_ids,
            concurrency,
            self.sync_concurrency_limit,
        );
        self.search_plane.synchronize_repo_runtime(&snapshot);
        *self
            .status_snapshot
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = snapshot;
    }
}
