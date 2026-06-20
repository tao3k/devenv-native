use std::collections::BTreeMap;

use redis::AsyncCommands;
use serde::de::DeserializeOwned;

#[cfg(any(test, feature = "test-support"))]
#[cfg(test)]
use crate::search::SearchManifestRecord;
use crate::search::cache::SearchPlaneCache;
use crate::search::{SearchCorpusKind, SearchRepoCorpusRecord, SearchRepoPublicationRecord};

impl SearchPlaneCache {
    pub(crate) async fn get_json<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        #[cfg(any(test, feature = "test-support"))]
        if let Some(payload) = self
            .shadow
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .generic_json_payloads
            .get(key)
            .cloned()
        {
            return serde_json::from_str(payload.as_str()).ok();
        }
        let mut connection = self.shared_async_connection().await?;
        let payload: Option<String> = connection.get(key).await.ok()?;
        serde_json::from_str(payload?.as_str()).ok()
    }

    pub(crate) async fn get_repo_corpus_record(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
    ) -> Option<SearchRepoCorpusRecord> {
        #[cfg(any(test, feature = "test-support"))]
        if let Some(record) = self
            .shadow
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .repo_corpus_records
            .get(&(corpus, repo_id.to_string()))
            .cloned()
        {
            return Some(record);
        }
        let key = self.keyspace.repo_corpus_record_key(corpus, repo_id);
        self.get_json(key.as_str()).await
    }

    pub(crate) async fn get_repo_corpus_records(
        &self,
        keys: &[(SearchCorpusKind, String)],
    ) -> BTreeMap<(SearchCorpusKind, String), SearchRepoCorpusRecord> {
        #[cfg(any(test, feature = "test-support"))]
        {
            let shadow = self
                .shadow
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !shadow.repo_corpus_records.is_empty() {
                return keys
                    .iter()
                    .filter_map(|(corpus, repo_id)| {
                        shadow
                            .repo_corpus_records
                            .get(&(*corpus, repo_id.clone()))
                            .cloned()
                            .map(|record| ((*corpus, repo_id.clone()), record))
                    })
                    .collect();
            }
        }
        if keys.is_empty() {
            return BTreeMap::new();
        }
        let Some(mut connection) = self.shared_async_connection().await else {
            return BTreeMap::new();
        };
        let mut pipeline = redis::pipe();
        for (corpus, repo_id) in keys {
            pipeline.cmd("GET").arg(
                self.keyspace
                    .repo_corpus_record_key(*corpus, repo_id.as_str()),
            );
        }
        let payloads: Vec<Option<String>> = pipeline
            .query_async(&mut connection)
            .await
            .unwrap_or_default();
        keys.iter()
            .cloned()
            .zip(payloads)
            .filter_map(|((corpus, repo_id), payload)| {
                let record =
                    serde_json::from_str::<SearchRepoCorpusRecord>(payload?.as_str()).ok()?;
                Some(((corpus, repo_id), record))
            })
            .collect()
    }

    pub(crate) async fn get_repo_publication_for_revision(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
        revision: &str,
    ) -> Option<SearchRepoPublicationRecord> {
        let normalized_revision = revision.trim().to_ascii_lowercase();
        if normalized_revision.is_empty() {
            return None;
        }
        #[cfg(any(test, feature = "test-support"))]
        if let Some(record) = self
            .shadow
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .repo_publications_by_revision
            .get(&(corpus, repo_id.to_string(), normalized_revision.clone()))
            .cloned()
        {
            return Some(record);
        }
        let key = self.keyspace.repo_publication_revision_key(
            corpus,
            repo_id,
            normalized_revision.as_str(),
        );
        self.get_json(key.as_str()).await
    }

    pub(crate) async fn get_repo_publication_revisions(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
    ) -> Vec<String> {
        #[cfg(any(test, feature = "test-support"))]
        if let Some(revisions) = self
            .shadow
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .repo_publication_revision_indexes
            .get(&(corpus, repo_id.to_string()))
            .cloned()
        {
            return revisions;
        }
        let Some(mut connection) = self.shared_async_connection().await else {
            return Vec::new();
        };
        let key = self
            .keyspace
            .repo_publication_revision_index_key(corpus, repo_id);
        connection.lrange(key, 0, -1).await.unwrap_or_default()
    }

    #[cfg(test)]
    pub(crate) async fn get_corpus_manifest(
        &self,
        corpus: SearchCorpusKind,
    ) -> Option<SearchManifestRecord> {
        #[cfg(any(test, feature = "test-support"))]
        if let Some(record) = self
            .shadow
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .corpus_manifests
            .get(&corpus)
            .cloned()
        {
            return Some(record);
        }
        let key = self.keyspace.corpus_manifest_key(corpus);
        self.get_json(key.as_str()).await
    }
}
