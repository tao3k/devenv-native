use std::collections::HashSet;
use std::fs;

use super::types::SearchPlaneService;
use crate::search::cache::SearchPlaneFileFingerprintScope;
use crate::search::{
    RepoContentChunkSearchFilters, SearchCorpusKind, SearchPublicationStorageFormat,
    SearchRepoCorpusRecord, SearchRepoPublicationInput, SearchRepoPublicationRecord,
};

impl SearchPlaneService {
    async fn latest_repo_publication_for_revision(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
        revision: &str,
        require_parquet_query_readable: bool,
    ) -> Option<SearchRepoPublicationRecord> {
        let publication = self
            .repo_corpus_record_for_reads(corpus, repo_id)
            .await
            .and_then(|record| record.publication)
            .filter(|publication| {
                repo_publication_matches_revision(
                    publication,
                    revision,
                    require_parquet_query_readable,
                )
            });
        if let Some(publication) = publication.as_ref() {
            self.cache
                .set_repo_publication_for_revision(corpus, repo_id, publication)
                .await;
        }
        publication
    }

    pub(crate) async fn repo_publication_for_revision(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
        revision: &str,
    ) -> Option<SearchRepoPublicationRecord> {
        let normalized_revision = revision.trim();
        if normalized_revision.is_empty() {
            return None;
        }
        if let Some(publication) =
            self.cached_repo_publication(corpus, repo_id)
                .filter(|publication| {
                    publication.source_revision.as_deref() == Some(normalized_revision)
                })
        {
            return Some(publication);
        }
        if let Some(publication) = self
            .cache
            .get_repo_publication_for_revision(corpus, repo_id, normalized_revision)
            .await
        {
            return Some(publication);
        }
        self.latest_repo_publication_for_revision(corpus, repo_id, normalized_revision, false)
            .await
    }

    pub(crate) async fn readable_repo_publication_for_revision(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
        revision: &str,
    ) -> Option<SearchRepoPublicationRecord> {
        let normalized_revision = revision.trim();
        if normalized_revision.is_empty() {
            return None;
        }
        if let Some(publication) =
            self.cached_repo_publication(corpus, repo_id)
                .filter(|publication| {
                    repo_publication_matches_revision(publication, normalized_revision, true)
                })
        {
            return Some(publication);
        }
        if let Some(publication) = self
            .latest_repo_publication_for_revision(corpus, repo_id, normalized_revision, true)
            .await
        {
            return Some(publication);
        }
        self.repo_publication_for_revision(corpus, repo_id, normalized_revision)
            .await
            .filter(|publication| {
                repo_publication_matches_revision(publication, normalized_revision, true)
            })
    }

    pub(crate) async fn repo_backed_publications_are_current_for_revision(
        &self,
        repo_id: &str,
        revision: &str,
    ) -> bool {
        let (entity_publication, content_publication) = tokio::join!(
            self.readable_repo_publication_for_revision(
                SearchCorpusKind::RepoEntity,
                repo_id,
                revision,
            ),
            self.readable_repo_publication_for_revision(
                SearchCorpusKind::RepoContentChunk,
                repo_id,
                revision,
            )
        );
        entity_publication.is_some() && content_publication.is_some()
    }

    pub(crate) async fn refresh_repo_backed_publications_for_revision(
        &self,
        repo_id: &str,
        revision: &str,
    ) -> bool {
        let normalized_revision = revision.trim();
        if normalized_revision.is_empty() {
            return false;
        }

        let entity_publication = self
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoEntity, repo_id)
            .await
            .and_then(|record| record.publication)
            .filter(SearchRepoPublicationRecord::is_parquet_query_readable);
        let content_publication = self
            .repo_corpus_record_for_reads(SearchCorpusKind::RepoContentChunk, repo_id)
            .await
            .and_then(|record| record.publication)
            .filter(SearchRepoPublicationRecord::is_parquet_query_readable);

        let Some(entity_publication) = entity_publication else {
            return false;
        };
        let Some(content_publication) = content_publication else {
            return false;
        };

        self.record_repo_publication_input_with_storage_format(
            SearchCorpusKind::RepoEntity,
            repo_id,
            SearchRepoPublicationInput {
                table_name: entity_publication.table_name.clone(),
                schema_version: entity_publication.schema_version,
                source_revision: Some(normalized_revision.to_string()),
                table_version_id: entity_publication.table_version_id,
                row_count: entity_publication.row_count,
                fragment_count: entity_publication.fragment_count,
                published_at: entity_publication.published_at.clone(),
            },
            entity_publication.storage_format,
        )
        .await;
        self.record_repo_publication_input_with_storage_format(
            SearchCorpusKind::RepoContentChunk,
            repo_id,
            SearchRepoPublicationInput {
                table_name: content_publication.table_name.clone(),
                schema_version: content_publication.schema_version,
                source_revision: Some(normalized_revision.to_string()),
                table_version_id: content_publication.table_version_id,
                row_count: content_publication.row_count,
                fragment_count: content_publication.fragment_count,
                published_at: content_publication.published_at.clone(),
            },
            content_publication.storage_format,
        )
        .await;
        true
    }

    pub(crate) async fn record_repo_publication_input_with_storage_format(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
        input: SearchRepoPublicationInput,
        storage_format: SearchPublicationStorageFormat,
    ) {
        let previous_record = self
            .repo_corpus_records
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(&(corpus, repo_id.to_string()))
            .cloned();
        let mut maintenance =
            self.next_repo_publication_maintenance(previous_record.as_ref(), input.row_count);
        if matches!(storage_format, SearchPublicationStorageFormat::Parquet) {
            maintenance.compaction_pending = false;
            maintenance.compaction_running = false;
            maintenance.publish_count_since_compaction = 0;
        }
        let record = match storage_format {
            SearchPublicationStorageFormat::Lance => {
                SearchRepoPublicationRecord::new(corpus, repo_id, input)
            }
            SearchPublicationStorageFormat::Parquet => {
                SearchRepoPublicationRecord::new_with_storage_format(
                    corpus,
                    repo_id,
                    input,
                    SearchPublicationStorageFormat::Parquet,
                )
            }
        };
        let runtime = self
            .repo_runtime_state(repo_id)
            .map(|state| Self::runtime_record_from_state(repo_id, &state));
        let corpus_record =
            SearchRepoCorpusRecord::new(corpus, repo_id.to_string(), runtime, Some(record.clone()))
                .with_maintenance(Some(maintenance));
        self.repo_corpus_records
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert((corpus, repo_id.to_string()), corpus_record.clone());
        self.persist_local_repo_corpus_record(&corpus_record);
        self.cache.set_repo_corpus_record(&corpus_record).await;
        self.cache
            .set_repo_publication_for_revision(corpus, repo_id, &record)
            .await;
        self.schedule_repo_compaction_if_needed(&corpus_record)
            .await;
    }

    /// Publish repository content chunks for a source revision.
    ///
    /// # Errors
    ///
    /// Returns a vector-store error when the repository content publication
    /// cannot be written.
    pub async fn publish_repo_content_chunks_with_revision(
        &self,
        repo_id: &str,
        documents: &[crate::repo_index::RepoCodeDocument],
        source_revision: Option<&str>,
    ) -> Result<(), xiuxian_db_store::VectorStoreError> {
        crate::search::repo_content_chunk::publish_repo_content_chunks(
            self,
            repo_id,
            documents,
            source_revision,
        )
        .await
    }

    pub(crate) async fn publish_repo_content_chunks_incremental_with_revision(
        &self,
        repo_id: &str,
        changed_documents: &[crate::repo_index::RepoCodeDocument],
        deleted_paths: &std::collections::BTreeSet<String>,
        source_revision: Option<&str>,
    ) -> Result<(), xiuxian_db_store::VectorStoreError> {
        crate::search::repo_content_chunk::publish_repo_content_chunks_incremental(
            self,
            repo_id,
            changed_documents,
            deleted_paths,
            source_revision,
        )
        .await
    }

    /// Search published repository content chunks for test support.
    ///
    /// # Errors
    ///
    /// Returns a repository content chunk search error when the published index
    /// cannot be queried or decoded.
    pub async fn search_repo_content_chunks(
        &self,
        repo_id: &str,
        search_term: &str,
        language_filters: &HashSet<String>,
        limit: usize,
    ) -> Result<
        Vec<crate::search::contracts::SearchHit>,
        crate::search::repo_content_chunk::RepoContentChunkSearchError,
    > {
        let filters = RepoContentChunkSearchFilters::default();
        self.search_repo_content_chunks_with_filters(
            repo_id,
            search_term,
            language_filters,
            &filters,
            limit,
        )
        .await
    }

    pub(crate) async fn search_repo_content_chunks_with_filters(
        &self,
        repo_id: &str,
        search_term: &str,
        language_filters: &HashSet<String>,
        filters: &RepoContentChunkSearchFilters,
        limit: usize,
    ) -> Result<
        Vec<crate::search::contracts::SearchHit>,
        crate::search::repo_content_chunk::RepoContentChunkSearchError,
    > {
        crate::search::repo_content_chunk::search_repo_content_chunks_with_filters(
            self,
            repo_id,
            search_term,
            language_filters,
            filters,
            limit,
        )
        .await
    }

    /// Publish repository entity analysis for a source revision.
    ///
    /// # Errors
    ///
    /// Returns a vector-store error when the repository entity publication
    /// cannot be written.
    pub async fn publish_repo_entities_with_revision(
        &self,
        repo_id: &str,
        analysis: &crate::analyzers::RepositoryAnalysisOutput,
        documents: &[crate::repo_index::RepoCodeDocument],
        source_revision: Option<&str>,
    ) -> Result<(), xiuxian_db_store::VectorStoreError> {
        crate::search::repo_entity::publish_repo_entities(
            self,
            repo_id,
            analysis,
            documents,
            source_revision,
        )
        .await
    }

    pub(crate) fn clear_repo_publications(&self, repo_id: &str) {
        for corpus in [
            SearchCorpusKind::RepoEntity,
            SearchCorpusKind::RepoContentChunk,
        ] {
            self.repo_corpus_records
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .remove(&(corpus, repo_id.to_string()));
            let _ = fs::remove_file(self.repo_corpus_record_json_path(corpus, repo_id));
        }
        let _ = fs::remove_file(self.repo_corpus_snapshot_json_path());
        self.clear_repo_maintenance_for_repo(repo_id);
        #[cfg(any(test, feature = "test-support"))]
        self.cache.clear_repo_shadow_for_tests(repo_id);
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            let cache = self.cache.clone();
            let service = self.clone();
            let repo_id = repo_id.to_string();
            let repo_corpus_records = std::sync::Arc::clone(&self.repo_corpus_records);
            handle.spawn(async move {
                cache
                    .delete_repo_corpus_record(SearchCorpusKind::RepoEntity, repo_id.as_str())
                    .await;
                cache
                    .delete_repo_corpus_record(SearchCorpusKind::RepoContentChunk, repo_id.as_str())
                    .await;
                service
                    .delete_file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
                        SearchCorpusKind::RepoEntity,
                        repo_id.as_str(),
                    ))
                    .await;
                service
                    .delete_file_fingerprints(SearchPlaneFileFingerprintScope::repo_corpus(
                        SearchCorpusKind::RepoContentChunk,
                        repo_id.as_str(),
                    ))
                    .await;
                cache
                    .delete_repo_publication_revision_cache(
                        SearchCorpusKind::RepoEntity,
                        repo_id.as_str(),
                    )
                    .await;
                cache
                    .delete_repo_publication_revision_cache(
                        SearchCorpusKind::RepoContentChunk,
                        repo_id.as_str(),
                    )
                    .await;
                cache.delete_repo_corpus_snapshot().await;
                repo_corpus_records
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .retain(|(_, candidate_repo_id), _| candidate_repo_id != &repo_id);
            });
        }
    }

    #[cfg(any(test, feature = "test-support"))]
    #[doc(hidden)]
    #[must_use]
    pub fn has_published_repo_corpus(&self, corpus: SearchCorpusKind, repo_id: &str) -> bool {
        self.cached_repo_publication(corpus, repo_id).is_some()
    }

    /// Search the published repository entity index for one repository.
    ///
    /// # Errors
    ///
    /// Returns a repository entity search error when the publication cannot be
    /// queried or decoded.
    pub async fn search_repo_entities(
        &self,
        repo_id: &str,
        search_term: &str,
        language_filters: &HashSet<String>,
        kind_filters: &HashSet<String>,
        limit: usize,
    ) -> Result<
        Vec<crate::search::contracts::SearchHit>,
        crate::search::repo_entity::RepoEntitySearchError,
    > {
        crate::search::repo_entity::search_repo_entities(
            self,
            repo_id,
            search_term,
            language_filters,
            kind_filters,
            limit,
        )
        .await
    }

    pub(crate) fn persist_local_repo_corpus_record(&self, record: &SearchRepoCorpusRecord) {
        let path = self.repo_corpus_record_json_path(record.corpus, record.repo_id.as_str());
        let Some(parent) = path.parent() else {
            return;
        };
        let Ok(payload) = serde_json::to_vec(record) else {
            return;
        };
        let _ = fs::create_dir_all(parent);
        let _ = fs::write(path, payload);
    }
}

fn repo_publication_matches_revision(
    publication: &SearchRepoPublicationRecord,
    revision: &str,
    require_parquet_query_readable: bool,
) -> bool {
    publication.source_revision.as_deref() == Some(revision)
        && (!require_parquet_query_readable || publication.is_parquet_query_readable())
}
