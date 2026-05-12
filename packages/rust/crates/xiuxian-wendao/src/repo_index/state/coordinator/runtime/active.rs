//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use crate::repo_index::state::coordinator::RepoIndexCoordinator;

impl RepoIndexCoordinator {
    pub(crate) fn fingerprint_matches(&self, repo_id: &str, fingerprint: &str) -> bool {
        self.fingerprints
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(repo_id)
            .is_some_and(|current| current == fingerprint)
    }

    pub(crate) fn mark_active(&self, repo_id: &str) {
        let mut active_repo_ids = self
            .active_repo_ids
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if active_repo_ids.iter().any(|active| active == repo_id) {
            return;
        }
        active_repo_ids.push(repo_id.to_string());
        drop(active_repo_ids);
        self.refresh_status_snapshot();
    }

    pub(crate) fn release_repo(&self, repo_id: &str) {
        self.active_repo_ids
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .retain(|active| active != repo_id);
        self.queued_or_active
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .remove(repo_id);
        self.refresh_status_snapshot();
    }
}
