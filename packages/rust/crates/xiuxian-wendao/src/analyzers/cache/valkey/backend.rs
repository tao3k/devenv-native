//! Compatibility path boundary: this module preserves an established Wendao owner path while the API surface is being narrowed.
//! `analyzers::cache::valkey::cache` owns Wendao cache valkey cache behavior.

#[cfg(all(test, feature = "search-runtime"))]
use std::collections::BTreeMap;

use crate::analyzers::RepositoryAnalysisOutput;

use super::{
    RepositoryAnalysisValkeyScope, RepositorySearchQueryValkeyScope, ValkeyAnalysisCacheRuntime,
    encode_analysis_payload, encode_search_query_payload, resolve_valkey_analysis_cache_runtime,
    valkey_analysis_key, valkey_analysis_revision_key,
};
/// `ValkeyAnalysisCache` public type boundary for Wendao.
#[derive(Debug, Clone)]
pub struct ValkeyAnalysisCache {
    runtime: ValkeyAnalysisCacheRuntime,
    #[cfg(all(test, feature = "search-runtime"))]
    shadow: std::sync::Arc<std::sync::RwLock<BTreeMap<String, String>>>,
}

impl ValkeyAnalysisCache {
    /// Creates a new Valkey cache client if configured.
    ///
    /// # Errors
    ///
    /// Returns an error when Valkey runtime configuration is invalid.
    pub fn new() -> Result<Option<Self>, crate::analyzers::RepoIntelligenceError> {
        Ok(resolve_valkey_analysis_cache_runtime()?.map(Self::from_runtime))
    }

    #[cfg(all(test, feature = "zhenfa-router"))]
    pub(crate) fn for_tests(key_prefix: &str, ttl_seconds: Option<u64>) -> Self {
        Self::from_runtime(ValkeyAnalysisCacheRuntime::for_tests(
            key_prefix,
            ttl_seconds,
        ))
    }

    /// Retrieves a cached analysis result for one analysis scope.
    #[must_use]
    pub fn get_analysis(
        &self,
        scope: RepositoryAnalysisValkeyScope<'_>,
    ) -> Option<RepositoryAnalysisOutput> {
        let storage_key = scope.storage_key(self.runtime.key_prefix.as_str());
        let payload = self.load_payload(storage_key.as_str())?;
        scope.decode(payload.as_str())
    }

    /// Stores an analysis result in the cache.
    pub fn set_analysis(
        &self,
        scope: RepositoryAnalysisValkeyScope<'_>,
        analysis: &RepositoryAnalysisOutput,
    ) {
        #[cfg(feature = "search-runtime")]
        let cache_key = match scope {
            RepositoryAnalysisValkeyScope::Current(cache_key) => cache_key,
            RepositoryAnalysisValkeyScope::Revision { .. } => return,
        };
        #[cfg(not(feature = "search-runtime"))]
        let RepositoryAnalysisValkeyScope::Current(cache_key) = scope;
        let storage_key = valkey_analysis_key(cache_key, self.runtime.key_prefix.as_str());
        let revision_key = cache_key.revision().map(|revision| {
            valkey_analysis_revision_key(
                cache_key.repo_id.as_str(),
                cache_key.checkout_root.as_str(),
                cache_key.plugin_ids.as_slice(),
                revision,
                self.runtime.key_prefix.as_str(),
            )
        });
        let Some(payload) = encode_analysis_payload(cache_key, analysis) else {
            return;
        };
        self.store_payload(storage_key.as_str(), payload.as_str());
        if let Some(revision_key) = revision_key {
            self.store_payload(revision_key.as_str(), payload.as_str());
        }
    }

    /// Retrieves a cached repo-search endpoint payload.
    #[must_use]
    pub fn get_search_query<T>(&self, scope: RepositorySearchQueryValkeyScope<'_>) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let storage_key = scope.storage_key(self.runtime.key_prefix.as_str());
        let payload = self.load_payload(storage_key.as_str())?;
        scope.decode(payload.as_str())
    }

    /// Stores one repo-search endpoint payload in the cache.
    pub fn set_search_query<T>(&self, scope: RepositorySearchQueryValkeyScope<'_>, value: &T)
    where
        T: serde::Serialize,
    {
        let storage_key = scope.storage_key(self.runtime.key_prefix.as_str());
        let Some(payload) = encode_search_query_payload(scope.cache_key(), value) else {
            return;
        };
        self.store_payload(storage_key.as_str(), payload.as_str());
    }

    fn from_runtime(runtime: ValkeyAnalysisCacheRuntime) -> Self {
        Self {
            runtime,
            #[cfg(all(test, feature = "search-runtime"))]
            shadow: std::sync::Arc::new(std::sync::RwLock::new(BTreeMap::new())),
        }
    }

    fn load_payload(&self, storage_key: &str) -> Option<String> {
        #[cfg(all(test, feature = "search-runtime"))]
        if self.runtime.client.is_none() {
            return self
                .shadow
                .read()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .get(storage_key)
                .cloned();
        }
        let client = self.runtime.client.as_ref()?;
        let mut connection = client.get_connection().ok()?;
        redis::cmd("GET")
            .arg(storage_key)
            .query::<Option<String>>(&mut connection)
            .ok()?
    }

    fn store_payload(&self, storage_key: &str, payload: &str) {
        #[cfg(all(test, feature = "search-runtime"))]
        if self.runtime.client.is_none() {
            self.shadow
                .write()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(storage_key.to_string(), payload.to_string());
            return;
        }
        let Some(client) = self.runtime.client.as_ref() else {
            return;
        };
        let Ok(mut connection) = client.get_connection() else {
            return;
        };
        if let Some(ttl_seconds) = self.runtime.ttl_seconds {
            let _ = redis::cmd("SETEX")
                .arg(storage_key)
                .arg(ttl_seconds)
                .arg(payload)
                .query::<()>(&mut connection);
            return;
        }
        let _ = redis::cmd("SET")
            .arg(storage_key)
            .arg(payload)
            .query::<()>(&mut connection);
    }
}
