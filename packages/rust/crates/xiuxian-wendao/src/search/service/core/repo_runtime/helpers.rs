//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
use std::collections::BTreeMap;

use crate::repo_index::{RepoIndexPhase, RepoIndexStatusResponse};
use crate::search::service::core::types::{RepoRuntimeState, SearchPlaneService};
use crate::search::{SearchCorpusKind, SearchRepoCorpusRecord, SearchRepoRuntimeRecord};

impl SearchPlaneService {
    pub(crate) fn repo_runtime_records(
        repo_status: &RepoIndexStatusResponse,
    ) -> Vec<SearchRepoRuntimeRecord> {
        repo_status
            .repos
            .iter()
            .map(SearchRepoRuntimeRecord::from_status)
            .collect()
    }

    pub(crate) fn next_repo_runtime_states(
        repo_status: &RepoIndexStatusResponse,
    ) -> BTreeMap<String, RepoRuntimeState> {
        repo_status
            .repos
            .iter()
            .map(|status| {
                (
                    status.repo_id.clone(),
                    RepoRuntimeState::from_status(status),
                )
            })
            .collect()
    }

    pub(crate) fn repo_runtime_delta(
        &self,
        runtime_records: &[SearchRepoRuntimeRecord],
        next_runtime: &BTreeMap<String, RepoRuntimeState>,
    ) -> (Vec<SearchRepoRuntimeRecord>, Vec<String>) {
        let current_runtime = self.current_repo_runtime_states();
        let removed_repo_ids = current_runtime
            .keys()
            .filter(|repo_id| !next_runtime.contains_key(*repo_id))
            .cloned()
            .collect::<Vec<_>>();
        let updated_records = runtime_records
            .iter()
            .filter(|status| {
                current_runtime.get(status.repo_id.as_str())
                    != next_runtime.get(status.repo_id.as_str())
            })
            .cloned()
            .collect::<Vec<_>>();
        (updated_records, removed_repo_ids)
    }

    pub(crate) fn apply_repo_runtime_to_memory(
        &self,
        runtime_records: &[SearchRepoRuntimeRecord],
        removed_repo_ids: &[String],
    ) {
        let mut current_records = self
            .repo_corpus_records
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        for repo_id in removed_repo_ids {
            for corpus in [
                SearchCorpusKind::RepoEntity,
                SearchCorpusKind::RepoContentChunk,
            ] {
                current_records.remove(&(corpus, repo_id.clone()));
            }
        }
        for runtime in runtime_records {
            Self::upsert_repo_runtime_records(&mut current_records, runtime);
        }
    }

    fn upsert_repo_runtime_records(
        current_records: &mut BTreeMap<(SearchCorpusKind, String), SearchRepoCorpusRecord>,
        runtime: &SearchRepoRuntimeRecord,
    ) {
        for corpus in [
            SearchCorpusKind::RepoEntity,
            SearchCorpusKind::RepoContentChunk,
        ] {
            let key = (corpus, runtime.repo_id.clone());
            match current_records.get_mut(&key) {
                Some(record) => {
                    record.runtime = Some(runtime.clone());
                }
                None => {
                    current_records.insert(
                        key,
                        SearchRepoCorpusRecord::new(
                            corpus,
                            runtime.repo_id.clone(),
                            Some(runtime.clone()),
                            None,
                        ),
                    );
                }
            }
        }
    }

    pub(crate) fn current_repo_runtime_states(&self) -> BTreeMap<String, RepoRuntimeState> {
        let mut runtime = BTreeMap::new();
        for record in self
            .repo_corpus_records
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .values()
        {
            if let Some(runtime_record) = record.runtime.as_ref() {
                runtime
                    .entry(record.repo_id.clone())
                    .or_insert_with(|| RepoRuntimeState::from_record(runtime_record));
            }
        }
        runtime
    }

    pub(crate) fn runtime_record_from_state(
        repo_id: &str,
        state: &RepoRuntimeState,
    ) -> SearchRepoRuntimeRecord {
        SearchRepoRuntimeRecord {
            repo_id: repo_id.to_string(),
            phase: state.phase,
            last_revision: state.last_revision.clone(),
            last_error: state.last_error.clone(),
            updated_at: state.updated_at.clone(),
        }
    }

    pub(crate) fn repo_search_publication_state_from_records(
        entity_record: Option<&SearchRepoCorpusRecord>,
        content_record: Option<&SearchRepoCorpusRecord>,
    ) -> crate::search::service::core::types::RepoSearchPublicationState {
        let entity_published = entity_record
            .and_then(|record| record.publication.as_ref())
            .is_some();
        let content_published = content_record
            .and_then(|record| record.publication.as_ref())
            .is_some();
        let runtime = entity_record
            .and_then(|record| record.runtime.as_ref())
            .or_else(|| content_record.and_then(|record| record.runtime.as_ref()))
            .map(RepoRuntimeState::from_record);
        let availability = if entity_published || content_published {
            crate::search::service::core::types::RepoSearchAvailability::Searchable
        } else if matches!(
            runtime.as_ref().map(|state| state.phase),
            Some(RepoIndexPhase::Unsupported | RepoIndexPhase::Failed)
        ) {
            crate::search::service::core::types::RepoSearchAvailability::Skipped
        } else {
            crate::search::service::core::types::RepoSearchAvailability::Pending
        };
        crate::search::service::core::types::RepoSearchPublicationState {
            entity_published,
            content_published,
            availability,
        }
    }
}
