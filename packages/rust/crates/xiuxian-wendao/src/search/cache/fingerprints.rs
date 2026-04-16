use std::collections::BTreeMap;

use redis::AsyncCommands;

use super::SearchPlaneCache;
use crate::search::{SearchCorpusKind, SearchFileFingerprint};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SearchPlaneFileFingerprintScope<'a> {
    Corpus(SearchCorpusKind),
    RepoCorpus {
        corpus: SearchCorpusKind,
        repo_id: &'a str,
    },
}

impl<'a> SearchPlaneFileFingerprintScope<'a> {
    pub(crate) const fn corpus(corpus: SearchCorpusKind) -> Self {
        Self::Corpus(corpus)
    }

    pub(crate) const fn repo_corpus(corpus: SearchCorpusKind, repo_id: &'a str) -> Self {
        Self::RepoCorpus { corpus, repo_id }
    }

    fn storage_key(self, cache: &SearchPlaneCache) -> String {
        match self {
            Self::Corpus(corpus) => cache.keyspace.corpus_file_fingerprints_key(corpus),
            Self::RepoCorpus { corpus, repo_id } => cache
                .keyspace
                .repo_corpus_file_fingerprints_key(corpus, repo_id),
        }
    }
}

impl SearchPlaneCache {
    pub(crate) async fn get_file_fingerprints(
        &self,
        scope: SearchPlaneFileFingerprintScope<'_>,
    ) -> Option<BTreeMap<String, SearchFileFingerprint>> {
        #[cfg(test)]
        match scope {
            SearchPlaneFileFingerprintScope::Corpus(corpus) => {
                if let Some(fingerprints) = self
                    .shadow
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .corpus_file_fingerprints
                    .get(&corpus)
                    .cloned()
                {
                    return Some(fingerprints);
                }
            }
            SearchPlaneFileFingerprintScope::RepoCorpus { corpus, repo_id } => {
                if let Some(fingerprints) = self
                    .shadow
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .repo_corpus_file_fingerprints
                    .get(&(corpus, repo_id.to_string()))
                    .cloned()
                {
                    return Some(fingerprints);
                }
            }
        }

        let key = scope.storage_key(self);
        self.get_json(key.as_str()).await
    }

    pub(crate) async fn set_file_fingerprints(
        &self,
        scope: SearchPlaneFileFingerprintScope<'_>,
        fingerprints: &BTreeMap<String, SearchFileFingerprint>,
    ) {
        #[cfg(test)]
        match scope {
            SearchPlaneFileFingerprintScope::Corpus(corpus) => {
                self.shadow
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .corpus_file_fingerprints
                    .insert(corpus, fingerprints.clone());
            }
            SearchPlaneFileFingerprintScope::RepoCorpus { corpus, repo_id } => {
                self.shadow
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .repo_corpus_file_fingerprints
                    .insert((corpus, repo_id.to_string()), fingerprints.clone());
            }
        }

        let Some(client) = self.client.as_ref() else {
            return;
        };
        let Ok(payload) = serde_json::to_string(fingerprints) else {
            return;
        };
        let key = scope.storage_key(self);
        let Ok(mut connection) = client
            .get_multiplexed_async_connection_with_config(&self.async_connection_config())
            .await
        else {
            return;
        };
        let _: redis::RedisResult<()> = connection.set(key, payload).await;
    }

    pub(crate) async fn delete_file_fingerprints(
        &self,
        scope: SearchPlaneFileFingerprintScope<'_>,
    ) {
        #[cfg(test)]
        match scope {
            SearchPlaneFileFingerprintScope::Corpus(corpus) => {
                self.shadow
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .corpus_file_fingerprints
                    .remove(&corpus);
            }
            SearchPlaneFileFingerprintScope::RepoCorpus { corpus, repo_id } => {
                self.shadow
                    .write()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .repo_corpus_file_fingerprints
                    .remove(&(corpus, repo_id.to_string()));
            }
        }

        let Some(client) = self.client.as_ref() else {
            return;
        };
        let key = scope.storage_key(self);
        let Ok(mut connection) = client
            .get_multiplexed_async_connection_with_config(&self.async_connection_config())
            .await
        else {
            return;
        };
        let _: redis::RedisResult<()> = connection.del(key).await;
    }
}
