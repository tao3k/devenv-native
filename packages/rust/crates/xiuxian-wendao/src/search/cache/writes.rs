use redis::AsyncCommands;
use serde::Serialize;

use crate::search::cache::{SearchPlaneCache, SearchPlaneCacheTtl};
use crate::search::{
    SearchCorpusKind, SearchManifestRecord, SearchRepoCorpusRecord, SearchRepoPublicationRecord,
};

impl SearchPlaneCache {
    pub(crate) async fn set_json<T>(&self, key: &str, ttl: SearchPlaneCacheTtl, value: &T)
    where
        T: Serialize,
    {
        let ttl_seconds = ttl.as_seconds(&self.config);
        if ttl_seconds == 0 {
            return;
        }
        let Ok(payload) = serde_json::to_string(value) else {
            return;
        };
        #[cfg(any(test, feature = "test-support"))]
        self.shadow
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .generic_json_payloads
            .insert(key.to_string(), payload.clone());
        let Some(client) = self.client.as_ref() else {
            return;
        };
        let Ok(mut connection) = client
            .get_multiplexed_async_connection_with_config(&self.async_connection_config())
            .await
        else {
            return;
        };
        let _: redis::RedisResult<()> = connection.set_ex(key, payload, ttl_seconds).await;
    }

    pub(crate) async fn set_repo_corpus_record(&self, record: &SearchRepoCorpusRecord) {
        #[cfg(any(test, feature = "test-support"))]
        self.shadow
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .repo_corpus_records
            .insert((record.corpus, record.repo_id.clone()), record.clone());
        let Some(client) = self.client.as_ref() else {
            return;
        };
        let Ok(payload) = serde_json::to_string(record) else {
            return;
        };
        let key = self
            .keyspace
            .repo_corpus_record_key(record.corpus, record.repo_id.as_str());
        let Ok(mut connection) = client
            .get_multiplexed_async_connection_with_config(&self.async_connection_config())
            .await
        else {
            return;
        };
        let _: redis::RedisResult<()> = connection.set(key, payload).await;
    }

    pub(crate) async fn set_repo_publication_for_revision(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
        publication: &SearchRepoPublicationRecord,
    ) {
        let Some(revision) = publication.source_revision.as_deref() else {
            return;
        };
        let normalized_revision = revision.trim().to_ascii_lowercase();
        if normalized_revision.is_empty() {
            return;
        }
        #[cfg(any(test, feature = "test-support"))]
        self.shadow
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .repo_publications_by_revision
            .insert(
                (corpus, repo_id.to_string(), normalized_revision.clone()),
                publication.clone(),
            );
        self.retain_repo_publication_revision(corpus, repo_id, normalized_revision.as_str())
            .await;
        let Some(client) = self.client.as_ref() else {
            return;
        };
        let Ok(payload) = serde_json::to_string(publication) else {
            return;
        };
        let key = self.keyspace.repo_publication_revision_key(
            corpus,
            repo_id,
            normalized_revision.as_str(),
        );
        let Ok(mut connection) = client
            .get_multiplexed_async_connection_with_config(&self.async_connection_config())
            .await
        else {
            return;
        };
        let _: redis::RedisResult<()> = connection.set(key, payload).await;
    }

    pub(crate) async fn delete_repo_publication_revision_cache(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
    ) {
        let revisions = self.get_repo_publication_revisions(corpus, repo_id).await;
        for revision in revisions {
            self.delete_repo_publication_for_revision(corpus, repo_id, revision.as_str())
                .await;
        }
        #[cfg(any(test, feature = "test-support"))]
        self.shadow
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .repo_publication_revision_indexes
            .remove(&(corpus, repo_id.to_string()));
        let Some(client) = self.client.as_ref() else {
            return;
        };
        let key = self
            .keyspace
            .repo_publication_revision_index_key(corpus, repo_id);
        let Ok(mut connection) = client
            .get_multiplexed_async_connection_with_config(&self.async_connection_config())
            .await
        else {
            return;
        };
        let _: redis::RedisResult<()> = connection.del(key).await;
    }

    pub(crate) async fn set_corpus_manifest(&self, record: &SearchManifestRecord) {
        #[cfg(any(test, feature = "test-support"))]
        {
            self.shadow
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .corpus_manifests
                .insert(record.corpus, record.clone());
        }
        let Some(client) = self.client.as_ref() else {
            return;
        };
        let Ok(payload) = serde_json::to_string(record) else {
            return;
        };
        let key = self.keyspace.corpus_manifest_key(record.corpus);
        let Ok(mut connection) = client
            .get_multiplexed_async_connection_with_config(&self.async_connection_config())
            .await
        else {
            return;
        };
        let _: redis::RedisResult<()> = connection.set(key, payload).await;
    }
    pub(crate) async fn delete_repo_corpus_record(&self, corpus: SearchCorpusKind, repo_id: &str) {
        #[cfg(any(test, feature = "test-support"))]
        self.shadow
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .repo_corpus_records
            .remove(&(corpus, repo_id.to_string()));
        let Some(client) = self.client.as_ref() else {
            return;
        };
        let key = self.keyspace.repo_corpus_record_key(corpus, repo_id);
        let Ok(mut connection) = client
            .get_multiplexed_async_connection_with_config(&self.async_connection_config())
            .await
        else {
            return;
        };
        let _: redis::RedisResult<()> = connection.del(key).await;
    }

    pub(crate) async fn delete_repo_publication_for_revision(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
        revision: &str,
    ) {
        let normalized_revision = revision.trim().to_ascii_lowercase();
        if normalized_revision.is_empty() {
            return;
        }
        #[cfg(any(test, feature = "test-support"))]
        self.shadow
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .repo_publications_by_revision
            .remove(&(corpus, repo_id.to_string(), normalized_revision.clone()));
        let Some(client) = self.client.as_ref() else {
            return;
        };
        let key = self.keyspace.repo_publication_revision_key(
            corpus,
            repo_id,
            normalized_revision.as_str(),
        );
        let Ok(mut connection) = client
            .get_multiplexed_async_connection_with_config(&self.async_connection_config())
            .await
        else {
            return;
        };
        let _: redis::RedisResult<()> = connection.del(key).await;
    }
    pub(crate) async fn delete_repo_corpus_snapshot(&self) {
        #[cfg(any(test, feature = "test-support"))]
        {
            self.shadow
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .repo_corpus_snapshot = None;
        }
        let Some(client) = self.client.as_ref() else {
            return;
        };
        let key = self.keyspace.repo_corpus_snapshot_key();
        let Ok(mut connection) = client
            .get_multiplexed_async_connection_with_config(&self.async_connection_config())
            .await
        else {
            return;
        };
        let _: redis::RedisResult<()> = connection.del(key).await;
    }

    async fn retain_repo_publication_revision(
        &self,
        corpus: SearchCorpusKind,
        repo_id: &str,
        revision: &str,
    ) {
        let retention = self.config.repo_revision_retention.max(1);
        let current = self.get_repo_publication_revisions(corpus, repo_id).await;
        let (retained, evicted) = updated_repo_publication_revisions(current, revision, retention);
        #[cfg(any(test, feature = "test-support"))]
        {
            let mut shadow = self
                .shadow
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            shadow
                .repo_publication_revision_indexes
                .insert((corpus, repo_id.to_string()), retained.clone());
            for evicted_revision in &evicted {
                shadow.repo_publications_by_revision.remove(&(
                    corpus,
                    repo_id.to_string(),
                    evicted_revision.clone(),
                ));
            }
        }
        let Some(client) = self.client.as_ref() else {
            return;
        };
        let index_key = self
            .keyspace
            .repo_publication_revision_index_key(corpus, repo_id);
        let Ok(mut connection) = client
            .get_multiplexed_async_connection_with_config(&self.async_connection_config())
            .await
        else {
            return;
        };
        let _: redis::RedisResult<()> = connection.del(index_key.as_str()).await;
        if !retained.is_empty() {
            let _: redis::RedisResult<()> =
                connection.rpush(index_key.as_str(), retained.clone()).await;
        }
        drop(connection);
        for evicted_revision in evicted {
            self.delete_repo_publication_for_revision(corpus, repo_id, evicted_revision.as_str())
                .await;
        }
    }
}

fn updated_repo_publication_revisions(
    current: Vec<String>,
    revision: &str,
    retention: usize,
) -> (Vec<String>, Vec<String>) {
    let normalized_revision = revision.trim().to_ascii_lowercase();
    if normalized_revision.is_empty() {
        return (current, Vec::new());
    }
    let mut merged = Vec::with_capacity(current.len().saturating_add(1));
    merged.push(normalized_revision.clone());
    for candidate in current {
        let normalized_candidate = candidate.trim().to_ascii_lowercase();
        if normalized_candidate.is_empty() || normalized_candidate == normalized_revision {
            continue;
        }
        merged.push(normalized_candidate);
    }
    let split_at = retention.max(1).min(merged.len());
    let evicted = merged.split_off(split_at);
    (merged, evicted)
}

#[cfg(test)]
#[path = "../../../tests/unit/search/cache/writes.rs"]
mod tests;
