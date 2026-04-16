use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::cache::{RepositoryAnalysisCacheKey, RepositorySearchQueryCacheKey};

#[cfg(feature = "zhenfa-router")]
use super::storage::decode_analysis_payload_for_revision;
#[cfg(feature = "zhenfa-router")]
use super::storage::valkey_analysis_revision_key;
use super::storage::{
    decode_analysis_payload, decode_search_query_payload, valkey_analysis_key,
    valkey_search_query_key,
};

#[derive(Debug, Clone, Copy)]
pub(crate) enum RepositoryAnalysisValkeyScope<'a> {
    Current(&'a RepositoryAnalysisCacheKey),
    #[cfg(feature = "zhenfa-router")]
    Revision {
        repo_id: &'a str,
        checkout_root: &'a str,
        plugin_ids: &'a [String],
        revision: &'a str,
    },
}

impl<'a> RepositoryAnalysisValkeyScope<'a> {
    pub(crate) fn current(cache_key: &'a RepositoryAnalysisCacheKey) -> Self {
        Self::Current(cache_key)
    }

    #[cfg(feature = "zhenfa-router")]
    pub(crate) fn revision(
        repo_id: &'a str,
        checkout_root: &'a str,
        plugin_ids: &'a [String],
        revision: &'a str,
    ) -> Self {
        Self::Revision {
            repo_id,
            checkout_root,
            plugin_ids,
            revision,
        }
    }

    pub(super) fn storage_key(self, key_prefix: &str) -> String {
        match self {
            Self::Current(cache_key) => valkey_analysis_key(cache_key, key_prefix),
            #[cfg(feature = "zhenfa-router")]
            Self::Revision {
                repo_id,
                checkout_root,
                plugin_ids,
                revision,
            } => valkey_analysis_revision_key(
                repo_id,
                checkout_root,
                plugin_ids,
                revision,
                key_prefix,
            ),
        }
    }

    pub(super) fn decode(self, payload: &str) -> Option<RepositoryAnalysisOutput> {
        match self {
            Self::Current(cache_key) => decode_analysis_payload(cache_key, payload),
            #[cfg(feature = "zhenfa-router")]
            Self::Revision {
                repo_id,
                checkout_root,
                plugin_ids,
                revision,
            } => decode_analysis_payload_for_revision(
                repo_id,
                checkout_root,
                plugin_ids,
                revision,
                payload,
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RepositorySearchQueryValkeyScope<'a> {
    cache_key: &'a RepositorySearchQueryCacheKey,
}

impl<'a> RepositorySearchQueryValkeyScope<'a> {
    pub(crate) fn new(cache_key: &'a RepositorySearchQueryCacheKey) -> Self {
        Self { cache_key }
    }

    pub(super) fn storage_key(self, key_prefix: &str) -> String {
        valkey_search_query_key(self.cache_key, key_prefix)
    }

    pub(super) fn decode<T>(self, payload: &str) -> Option<T>
    where
        T: serde::de::DeserializeOwned,
    {
        decode_search_query_payload(self.cache_key, payload)
    }

    pub(super) fn cache_key(self) -> &'a RepositorySearchQueryCacheKey {
        self.cache_key
    }
}
