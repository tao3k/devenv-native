use std::sync::atomic::Ordering;

use crate::repo_index::RepoIndexStatusResponse;
use crate::search::service::core::types::SearchPlaneService;
use crate::search::{SearchCorpusKind, SearchRepoRuntimeRecord};
use futures::stream::{self, StreamExt};

impl SearchPlaneService {
    fn advance_repo_runtime_generation(&self) -> u64 {
        self.repo_runtime_generation.fetch_add(1, Ordering::Relaxed) + 1
    }

    fn repo_runtime_generation_is_current(&self, generation: u64) -> bool {
        self.repo_runtime_generation.load(Ordering::Relaxed) == generation
    }

    fn prepare_repo_runtime_refresh(
        &self,
        repo_status: &RepoIndexStatusResponse,
    ) -> Option<(u64, Vec<String>, Vec<SearchRepoRuntimeRecord>)> {
        let runtime_records = Self::repo_runtime_records(repo_status);
        let next_runtime = Self::next_repo_runtime_states(repo_status);
        let (updated_records, removed_repo_ids) =
            self.repo_runtime_delta(runtime_records.as_slice(), &next_runtime);
        self.apply_repo_runtime_to_memory(runtime_records.as_slice(), removed_repo_ids.as_slice());
        if updated_records.is_empty() && removed_repo_ids.is_empty() {
            return None;
        }
        let generation = self.advance_repo_runtime_generation();
        Some((generation, removed_repo_ids, runtime_records))
    }

    pub(crate) fn synchronize_repo_runtime(&self, repo_status: &RepoIndexStatusResponse) {
        let Some((generation, removed_repo_ids, runtime_records)) =
            self.prepare_repo_runtime_refresh(repo_status)
        else {
            return;
        };
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let service = self.clone();
            handle.spawn(async move {
                service
                    .refresh_repo_runtime_cache(generation, removed_repo_ids, runtime_records)
                    .await;
            });
        }
    }

    async fn refresh_repo_runtime_cache(
        &self,
        generation: u64,
        removed_repo_ids: Vec<String>,
        runtime_records: Vec<crate::search::SearchRepoRuntimeRecord>,
    ) {
        if !self.repo_runtime_generation_is_current(generation) {
            return;
        }
        if !self
            .delete_removed_repo_runtime_records(generation, removed_repo_ids.as_slice())
            .await
        {
            return;
        }
        if !self
            .refresh_repo_corpus_records(generation, runtime_records.as_slice())
            .await
        {
            return;
        }
        if !self.repo_runtime_generation_is_current(generation) {
            return;
        }
        self.synchronize_repo_corpus_statuses_from_runtime().await;
    }

    async fn delete_removed_repo_runtime_records(
        &self,
        generation: u64,
        removed_repo_ids: &[String],
    ) -> bool {
        stream::iter(removed_repo_ids)
            .then(|repo_id| async move {
                if !self.repo_runtime_generation_is_current(generation) {
                    return false;
                }
                self.delete_removed_repo_runtime_record_for_repo(repo_id)
                    .await;
                true
            })
            .all(|deleted| async move { deleted })
            .await
    }

    async fn delete_removed_repo_runtime_record_for_repo(&self, repo_id: &str) {
        stream::iter([
            SearchCorpusKind::RepoEntity,
            SearchCorpusKind::RepoContentChunk,
        ])
        .then(|corpus| async move {
            self.repo_corpus_records
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(&(corpus, repo_id.to_string()));
            self.cache.delete_repo_corpus_record(corpus, repo_id).await;
            let _ = std::fs::remove_file(self.repo_corpus_record_json_path(corpus, repo_id));
        })
        .collect::<Vec<_>>()
        .await;
    }

    async fn refresh_repo_corpus_records(
        &self,
        generation: u64,
        runtime_records: &[crate::search::SearchRepoRuntimeRecord],
    ) -> bool {
        for runtime in runtime_records {
            for corpus in [
                SearchCorpusKind::RepoEntity,
                SearchCorpusKind::RepoContentChunk,
            ] {
                if !self.repo_runtime_generation_is_current(generation) {
                    return false;
                }
                let existing_record = self
                    .repo_corpus_record_for_reads(corpus, runtime.repo_id.as_str())
                    .await;
                if !self.repo_runtime_generation_is_current(generation) {
                    return false;
                }
                let publication = existing_record
                    .as_ref()
                    .and_then(|record| record.publication.clone())
                    .or_else(|| self.cached_repo_publication(corpus, runtime.repo_id.as_str()));
                let maintenance = existing_record
                    .as_ref()
                    .and_then(|record| record.maintenance.clone());
                let record = crate::search::SearchRepoCorpusRecord::new(
                    corpus,
                    runtime.repo_id.clone(),
                    Some(runtime.clone()),
                    publication,
                )
                .with_maintenance(maintenance);
                self.repo_corpus_records
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .insert((corpus, runtime.repo_id.clone()), record.clone());
                self.persist_local_repo_corpus_record(&record);
                self.cache.set_repo_corpus_record(&record).await;
            }
        }
        true
    }

    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    #[must_use]
    pub fn advance_repo_runtime_generation_for_test(&self) -> u64 {
        self.advance_repo_runtime_generation()
    }

    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    pub async fn refresh_repo_runtime_cache_for_test(
        &self,
        generation: u64,
        runtime_records: Vec<crate::search::SearchRepoRuntimeRecord>,
    ) {
        self.refresh_repo_runtime_cache(generation, Vec::new(), runtime_records)
            .await;
    }

    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    pub async fn synchronize_repo_runtime_for_test(&self, repo_status: &RepoIndexStatusResponse) {
        let Some((generation, removed_repo_ids, runtime_records)) =
            self.prepare_repo_runtime_refresh(repo_status)
        else {
            return;
        };
        self.refresh_repo_runtime_cache(generation, removed_repo_ids, runtime_records)
            .await;
    }
}
