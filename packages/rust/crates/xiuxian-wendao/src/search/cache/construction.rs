use redis::AsyncConnectionConfig;

use crate::search::SearchManifestKeyspace;

use super::config::SearchPlaneCacheConfig;
use super::runtime::resolve_search_plane_cache_runtime;
use super::types::SearchPlaneCache;
#[cfg(any(test, feature = "test-support"))]
use super::types::TestCacheShadow;
#[cfg(any(test, feature = "test-support"))]
use std::sync::{Arc, RwLock};

impl SearchPlaneCache {
    pub(crate) fn from_runtime(keyspace: SearchManifestKeyspace) -> Self {
        let runtime = resolve_search_plane_cache_runtime();
        Self::new(runtime.client, runtime.config, keyspace)
    }

    pub(crate) fn disabled(keyspace: SearchManifestKeyspace) -> Self {
        Self::new(None, SearchPlaneCacheConfig::default(), keyspace)
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn for_tests(keyspace: SearchManifestKeyspace) -> Self {
        Self::for_tests_with_config(keyspace, SearchPlaneCacheConfig::default())
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn for_tests_with_config(
        keyspace: SearchManifestKeyspace,
        config: SearchPlaneCacheConfig,
    ) -> Self {
        Self::new(
            Some(
                redis::Client::open("redis://127.0.0.1/")
                    .unwrap_or_else(|error| panic!("client: {error}")),
            ),
            config,
            keyspace,
        )
    }

    fn new(
        client: Option<redis::Client>,
        config: SearchPlaneCacheConfig,
        keyspace: SearchManifestKeyspace,
    ) -> Self {
        Self {
            client,
            config,
            keyspace,
            #[cfg(any(test, feature = "test-support"))]
            shadow: Arc::new(RwLock::new(TestCacheShadow::default())),
        }
    }

    pub(crate) fn async_connection_config(&self) -> AsyncConnectionConfig {
        AsyncConnectionConfig::new()
            .set_connection_timeout(Some(self.config.connection_timeout))
            .set_response_timeout(Some(self.config.response_timeout))
    }

    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn clear_repo_shadow_for_tests(&self, repo_id: &str) {
        let mut shadow = self
            .shadow
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        shadow
            .repo_corpus_records
            .retain(|(_, candidate_repo_id), _| candidate_repo_id != repo_id);
        if let Some(snapshot) = shadow.repo_corpus_snapshot.as_mut() {
            snapshot.records.retain(|record| record.repo_id != repo_id);
            if snapshot.records.is_empty() {
                shadow.repo_corpus_snapshot = None;
            }
        }
        shadow
            .repo_corpus_file_fingerprints
            .retain(|(_, candidate_repo_id), _| candidate_repo_id != repo_id);
        shadow
            .repo_publications_by_revision
            .retain(|(_, candidate_repo_id, _), _| candidate_repo_id != repo_id);
        shadow
            .repo_publication_revision_indexes
            .retain(|(_, candidate_repo_id), _| candidate_repo_id != repo_id);
    }
}
