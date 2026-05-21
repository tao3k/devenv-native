//! `search::repo_search::dispatch` owns Wendao search repo search dispatch behavior.

use std::collections::BTreeMap;

use crate::search::{RepoSearchAvailability, RepoSearchPublicationState, SearchPlaneService};

#[derive(Debug, Clone)]
#[doc(hidden)]
pub struct RepoSearchTarget {
    pub repo_id: String,
    pub publication_state: RepoSearchPublicationState,
}

#[derive(Debug, Default)]
#[doc(hidden)]
pub struct RepoSearchDispatch {
    pub searchable: Vec<RepoSearchTarget>,
    pub pending: Vec<String>,
    pub skipped: Vec<String>,
}

#[doc(hidden)]
#[must_use]
pub fn collect_repo_search_targets(
    repo_ids: Vec<String>,
    publication_states: &BTreeMap<String, RepoSearchPublicationState>,
) -> RepoSearchDispatch {
    let mut dispatch = RepoSearchDispatch::default();
    for repo_id in repo_ids {
        let publication_state = publication_states.get(repo_id.as_str()).copied().unwrap_or(
            RepoSearchPublicationState {
                entity_published: false,
                content_published: false,
                availability: RepoSearchAvailability::Pending,
            },
        );
        if publication_state.is_searchable() {
            dispatch.searchable.push(RepoSearchTarget {
                repo_id,
                publication_state,
            });
            continue;
        }
        match publication_state.availability {
            RepoSearchAvailability::Skipped => dispatch.skipped.push(repo_id),
            RepoSearchAvailability::Pending => dispatch.pending.push(repo_id),
            RepoSearchAvailability::Searchable => {}
        }
    }
    dispatch
}

#[doc(hidden)]
#[must_use]
pub fn repo_search_parallelism(search_plane: &SearchPlaneService, repo_count: usize) -> usize {
    search_plane.repo_search_parallelism(repo_count)
}
