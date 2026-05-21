use super::types::{RepoRuntimeState, RepoSearchQueryCacheKeyInput, SearchPlaneService};
use crate::search::service::helpers::{repo_corpus_cache_version, repo_publication_cache_version};
use crate::search::{SearchCorpusKind, SearchPlaneCacheTtl};

impl SearchPlaneService {
    #[must_use]
    pub(crate) fn corpus_active_epoch(&self, corpus: SearchCorpusKind) -> Option<u64> {
        self.coordinator.status_for(corpus).active_epoch
    }

    /// Build an autocomplete cache key for the current local-symbol epoch.
    #[must_use]
    pub fn autocomplete_cache_key(&self, prefix: &str, limit: usize) -> Option<String> {
        let epoch = self.corpus_active_epoch(SearchCorpusKind::LocalSymbol)?;
        self.cache.autocomplete_cache_key(prefix, limit, epoch)
    }

    /// Build a query cache key from active local corpus epochs.
    #[must_use]
    /// Positional boundary: this public API preserves an existing compatibility surface; call-site semantics are documented by parameter names.
    pub fn search_query_cache_key(
        &self,
        scope: &str,
        corpora: &[SearchCorpusKind],
        query: &str,
        limit: usize,
        intent: Option<&str>,
        repo_hint: Option<&str>,
    ) -> Option<String> {
        let epochs = corpora
            .iter()
            .map(|corpus| {
                self.corpus_active_epoch(*corpus)
                    .map(|epoch| (*corpus, epoch))
            })
            .collect::<Option<Vec<_>>>()?;
        self.cache
            .search_query_cache_key(scope, epochs.as_slice(), query, limit, intent, repo_hint)
    }

    /// Build a repository search cache key from local and repo publication versions.
    #[must_use]
    pub async fn repo_search_query_cache_key(
        &self,
        input: RepoSearchQueryCacheKeyInput<'_>,
    ) -> Option<String> {
        let mut versions = self.local_corpus_cache_versions(input.corpora);
        self.extend_repo_corpus_cache_versions(&mut versions, input.repo_ids, input.repo_corpora)
            .await;
        self.cache.search_query_cache_key_from_versions(
            input.scope,
            versions.as_slice(),
            input.query,
            input.limit,
            input.intent,
            input.repo_hint,
        )
    }

    fn local_corpus_cache_versions(&self, corpora: &[SearchCorpusKind]) -> Vec<String> {
        corpora
            .iter()
            .map(|corpus| self.corpus_cache_version(*corpus))
            .collect()
    }

    async fn extend_repo_corpus_cache_versions(
        &self,
        versions: &mut Vec<String>,
        repo_ids: &[String],
        repo_corpora: &[SearchCorpusKind],
    ) {
        let sorted_repo_ids = sorted_repo_ids(repo_ids);
        if sorted_repo_ids.is_empty() {
            versions.push("repo_set:none".to_string());
            return;
        }
        for repo_id in sorted_repo_ids {
            self.extend_one_repo_corpus_cache_versions(versions, repo_id.as_str(), repo_corpora)
                .await;
        }
    }

    async fn extend_one_repo_corpus_cache_versions(
        &self,
        versions: &mut Vec<String>,
        repo_id: &str,
        repo_corpora: &[SearchCorpusKind],
    ) {
        for corpus in repo_corpora {
            versions.push(
                self.repo_corpus_cache_version_for_reads(*corpus, repo_id)
                    .await,
            );
        }
    }

    async fn repo_corpus_cache_version_for_reads(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
    ) -> String {
        let Some(record) = self.repo_corpus_record_for_reads(corpus, repo_id).await else {
            return repo_corpus_cache_version(corpus, repo_id, None);
        };
        let runtime = record.runtime.as_ref().map(RepoRuntimeState::from_record);
        record.publication.as_ref().map_or_else(
            || repo_corpus_cache_version(corpus, repo_id, runtime.as_ref()),
            |publication| repo_publication_cache_version(runtime.as_ref(), publication),
        )
    }

    /// Read a JSON value from the search-plane cache.
    pub async fn cache_get_json<T>(&self, key: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        self.cache.get_json(key).await
    }

    /// Write a JSON value into the search-plane cache.
    pub async fn cache_set_json<T>(&self, key: &str, ttl: SearchPlaneCacheTtl, value: &T)
    where
        T: serde::Serialize,
    {
        self.cache.set_json(key, ttl, value).await;
    }
}

fn sorted_repo_ids(repo_ids: &[String]) -> Vec<String> {
    let mut sorted = repo_ids.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted
}
