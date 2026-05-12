//! `analyzers::cache::query` owns Wendao analyzers cache query behavior.

use std::collections::BTreeMap;
use std::sync::{Mutex, OnceLock};

use crate::analyzers::RepoIntelligenceError;
use crate::analyzers::cache::{
    RepositorySearchQueryCacheKey, RepositorySearchQueryValkeyScope, ValkeyAnalysisCache,
};

type RepositorySearchQueryCache = BTreeMap<RepositorySearchQueryCacheKey, serde_json::Value>;

static REPOSITORY_SEARCH_QUERY_CACHE: OnceLock<Mutex<RepositorySearchQueryCache>> = OnceLock::new();

fn repository_search_query_cache() -> &'static Mutex<RepositorySearchQueryCache> {
    REPOSITORY_SEARCH_QUERY_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Loads a cached repo-search payload if available.
///
/// # Errors
///
/// Returns an error when the in-memory cache lock is poisoned or payload decoding fails.
pub fn load_cached_repository_search_result<T>(
    key: &RepositorySearchQueryCacheKey,
) -> Result<Option<T>, RepoIntelligenceError>
where
    T: serde::de::DeserializeOwned + serde::Serialize,
{
    if let Some(value) = repository_search_query_cache()
        .lock()
        .map_err(|_| RepoIntelligenceError::AnalysisFailed {
            message: "repository search query cache lock is poisoned".to_string(),
        })?
        .get(key)
        .cloned()
        .map(|value| {
            serde_json::from_value(value).map_err(|error| RepoIntelligenceError::AnalysisFailed {
                message: format!("failed to decode cached repository search payload: {error}"),
            })
        })
        .transpose()?
    {
        return Ok(Some(value));
    }

    let Some(valkey_cache) = ValkeyAnalysisCache::new()? else {
        return Ok(None);
    };
    let Some(value) = valkey_cache.get_search_query(RepositorySearchQueryValkeyScope::new(key))
    else {
        return Ok(None);
    };
    let encoded =
        serde_json::to_value(&value).map_err(|error| RepoIntelligenceError::AnalysisFailed {
            message: format!("failed to encode cached repository search payload: {error}"),
        })?;
    repository_search_query_cache()
        .lock()
        .map_err(|_| RepoIntelligenceError::AnalysisFailed {
            message: "repository search query cache lock is poisoned".to_string(),
        })?
        .insert(key.clone(), encoded);
    Ok(Some(value))
}

/// Stores a repo-search payload in the query-result cache.
///
/// # Errors
///
/// Returns an error when the in-memory cache lock is poisoned or payload serialization fails.
pub fn store_cached_repository_search_result<T>(
    key: &RepositorySearchQueryCacheKey,
    value: &T,
) -> Result<(), RepoIntelligenceError>
where
    T: serde::Serialize,
{
    let encoded =
        serde_json::to_value(value).map_err(|error| RepoIntelligenceError::AnalysisFailed {
            message: format!("failed to encode cached repository search payload: {error}"),
        })?;
    repository_search_query_cache()
        .lock()
        .map_err(|_| RepoIntelligenceError::AnalysisFailed {
            message: "repository search query cache lock is poisoned".to_string(),
        })
        .map(|mut cache| {
            cache.insert(key.clone(), encoded);
        })?;
    if let Some(valkey_cache) = ValkeyAnalysisCache::new()? {
        valkey_cache.set_search_query(RepositorySearchQueryValkeyScope::new(key), value);
    }
    Ok(())
}
