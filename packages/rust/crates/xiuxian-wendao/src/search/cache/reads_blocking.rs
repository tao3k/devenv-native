use std::collections::BTreeMap;

use redis::Commands;
use serde::de::DeserializeOwned;

use crate::search::cache::SearchPlaneCache;
use crate::search::{SearchCorpusKind, SearchManifestRecord, SearchRepoCorpusRecord};

impl SearchPlaneCache {
    fn blocking_connection(&self) -> Option<redis::Connection> {
        let client = self.client.as_ref()?;
        let connection = client
            .get_connection_with_timeout(self.config.connection_timeout)
            .ok()?;
        let _ = connection.set_read_timeout(Some(self.config.response_timeout));
        let _ = connection.set_write_timeout(Some(self.config.response_timeout));
        Some(connection)
    }

    fn get_json_blocking<T>(&self, key: &str) -> Option<T>
    where
        T: DeserializeOwned,
    {
        let mut connection = self.blocking_connection()?;
        let payload: Option<String> = connection.get(key).ok()?;
        serde_json::from_str(payload?.as_str()).ok()
    }

    pub(crate) fn get_repo_corpus_records_blocking(
        &self,
        keys: &[(SearchCorpusKind, String)],
    ) -> BTreeMap<(SearchCorpusKind, String), SearchRepoCorpusRecord> {
        #[cfg(test)]
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
        let Some(mut connection) = self.blocking_connection() else {
            return BTreeMap::new();
        };
        let mut pipeline = redis::pipe();
        for (corpus, repo_id) in keys {
            pipeline.cmd("GET").arg(
                self.keyspace
                    .repo_corpus_record_key(*corpus, repo_id.as_str()),
            );
        }
        let payloads: Vec<Option<String>> = pipeline.query(&mut connection).unwrap_or_default();
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

    pub(crate) fn get_corpus_manifest_blocking(
        &self,
        corpus: SearchCorpusKind,
    ) -> Option<SearchManifestRecord> {
        #[cfg(test)]
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
        self.get_json_blocking(key.as_str())
    }
}
