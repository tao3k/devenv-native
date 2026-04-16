use std::collections::BTreeSet;

use crate::analyzers::RegisteredRepository;
use crate::repo_index::types::{RepoIndexEntryStatus, RepoIndexPhase};

use crate::repo_index::state::coordinator::RepoIndexCoordinator;
use crate::repo_index::state::fingerprint::{fingerprint, fingerprint_id, timestamp_now};
use crate::repo_index::state::task::{RepoIndexTask, RepoIndexTaskPriority};

impl RepoIndexCoordinator {
    pub(crate) fn sync_repositories(&self, repositories: Vec<RegisteredRepository>) -> Vec<String> {
        self.hydrate_repositories_from_search_plane(repositories.as_slice());
        let active_ids = repositories
            .iter()
            .map(|repository| repository.id.clone())
            .collect::<BTreeSet<_>>();
        self.prune_removed(&active_ids);

        let mut enqueued = Vec::new();
        for repository in repositories {
            let repo_fingerprint = fingerprint(&repository);
            let existing = self
                .fingerprints
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(&repository.id)
                .cloned();
            if existing.as_deref() != Some(repo_fingerprint.as_str())
                && self.enqueue_repository(
                    repository,
                    false,
                    true,
                    repo_fingerprint.clone(),
                    RepoIndexTaskPriority::Background,
                )
            {
                enqueued.push(fingerprint_id(&repo_fingerprint));
            }
        }

        enqueued
    }

    pub(crate) fn ensure_repositories_enqueued(
        &self,
        repositories: Vec<RegisteredRepository>,
        refresh: bool,
    ) -> Vec<String> {
        let mut enqueued = Vec::new();
        for repository in repositories {
            let repo_fingerprint = fingerprint(&repository);
            if self.enqueue_repository(
                repository,
                refresh,
                refresh,
                repo_fingerprint.clone(),
                RepoIndexTaskPriority::Interactive,
            ) {
                enqueued.push(fingerprint_id(&repo_fingerprint));
            }
        }
        enqueued
    }

    fn prune_removed(&self, active_ids: &BTreeSet<String>) {
        let removed_repo_ids = self
            .statuses
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .keys()
            .filter(|repo_id| !active_ids.contains(*repo_id))
            .cloned()
            .collect::<Vec<_>>();
        self.statuses
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|repo_id, _| active_ids.contains(repo_id));
        self.fingerprints
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|repo_id, _| active_ids.contains(repo_id));
        self.queued_or_active
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|repo_id| active_ids.contains(repo_id));
        self.active_repo_ids
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|repo_id| active_ids.contains(repo_id));
        self.pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|task| active_ids.contains(&task.repository.id));
        for repo_id in removed_repo_ids {
            self.search_plane.clear_repo_publications(repo_id.as_str());
        }
        self.refresh_status_snapshot();
    }

    pub(crate) fn enqueue_repository(
        &self,
        repository: RegisteredRepository,
        refresh: bool,
        force: bool,
        repo_fingerprint: String,
        priority: RepoIndexTaskPriority,
    ) -> bool {
        self.enqueue_task(
            RepoIndexTask {
                repository,
                refresh,
                fingerprint: repo_fingerprint,
                priority,
                retry_count: 0,
            },
            force,
        )
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn enqueue_task(&self, task: RepoIndexTask, force: bool) -> bool {
        let repo_id = task.repository.id.clone();
        let incoming_priority = task.priority;
        let incoming_refresh = task.refresh;
        let incoming_retry_count = task.retry_count;
        let incoming_repository = task.repository;
        let incoming_fingerprint = task.fingerprint;
        let existing_status = self
            .statuses
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&repo_id)
            .cloned();
        let existing_fingerprint = self
            .fingerprints
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&repo_id)
            .cloned();
        let is_same_fingerprint =
            existing_fingerprint.as_deref() == Some(incoming_fingerprint.as_str());
        let already_queued_or_active = self
            .queued_or_active
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .contains(&repo_id);

        if already_queued_or_active {
            let mut updated_existing_task = false;
            let mut pending = self
                .pending
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if let Some(index) = pending
                .iter()
                .position(|task| task.repository.id == repo_id)
                && let Some(mut task) = pending.remove(index)
            {
                let fingerprint_changed = task.fingerprint != incoming_fingerprint;
                updated_existing_task = fingerprint_changed
                    || incoming_refresh
                    || matches!(incoming_priority, RepoIndexTaskPriority::Interactive)
                        && !matches!(task.priority, RepoIndexTaskPriority::Interactive);
                if fingerprint_changed {
                    self.fingerprints
                        .write()
                        .unwrap_or_else(std::sync::PoisonError::into_inner)
                        .insert(repo_id.clone(), incoming_fingerprint.clone());
                }
                task.priority = match (task.priority, incoming_priority) {
                    (RepoIndexTaskPriority::Interactive, _)
                    | (_, RepoIndexTaskPriority::Interactive) => RepoIndexTaskPriority::Interactive,
                    _ => RepoIndexTaskPriority::Background,
                };
                task.refresh |= incoming_refresh;
                task.repository = incoming_repository;
                task.fingerprint = incoming_fingerprint;
                task.retry_count = if fingerprint_changed {
                    0
                } else {
                    incoming_retry_count
                };
                match task.priority {
                    RepoIndexTaskPriority::Interactive => pending.push_front(task),
                    RepoIndexTaskPriority::Background => pending.push_back(task),
                }
            }
            drop(pending);
            if updated_existing_task {
                self.refresh_status_snapshot();
                self.notify.notify_one();
            }
            return updated_existing_task;
        }

        if !force
            && is_same_fingerprint
            && let Some(ref status) = existing_status
            && matches!(
                status.phase,
                RepoIndexPhase::Ready | RepoIndexPhase::Unsupported | RepoIndexPhase::Failed
            )
        {
            return false;
        }

        self.fingerprints
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(repo_id.clone(), incoming_fingerprint.clone());
        self.queued_or_active
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(repo_id.clone());
        self.statuses
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(
                repo_id.clone(),
                RepoIndexEntryStatus {
                    repo_id: repo_id.clone(),
                    phase: RepoIndexPhase::Queued,
                    queue_position: None,
                    last_error: None,
                    last_revision: existing_status
                        .as_ref()
                        .and_then(|status| status.last_revision.clone()),
                    updated_at: Some(timestamp_now()),
                    attempt_count: 0,
                },
            );
        self.refresh_status_snapshot();
        let task = RepoIndexTask {
            repository: incoming_repository,
            refresh: incoming_refresh,
            fingerprint: incoming_fingerprint,
            priority: incoming_priority,
            retry_count: incoming_retry_count,
        };
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match task.priority {
            RepoIndexTaskPriority::Interactive => pending.push_front(task),
            RepoIndexTaskPriority::Background => pending.push_back(task),
        }
        drop(pending);
        self.notify.notify_one();
        true
    }
}
