use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::analyzers::RepositoryAnalysisOutput;
use crate::analyzers::cache::{RepositoryAnalysisCacheKey, RepositorySearchQueryCacheKey};

const ANALYZER_CACHE_SCHEMA_VERSION: &str = "xiuxian_wendao.repo_analysis_cache.v3";
const SEARCH_QUERY_CACHE_SCHEMA_VERSION: &str = "xiuxian_wendao.repo_search_query_cache.v2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ValkeyAnalysisCachePayload {
    schema: String,
    repo_id: String,
    checkout_root: String,
    analysis_identity: String,
    plugin_ids: Vec<String>,
    revision: String,
    cached_at_rfc3339: String,
    analysis: RepositoryAnalysisOutput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ValkeySearchQueryCachePayload {
    schema: String,
    repo_id: String,
    analysis_identity: String,
    revision: String,
    endpoint: String,
    query: String,
    filter: Option<String>,
    max_distance: u8,
    prefix_length: usize,
    transposition: bool,
    limit: usize,
    cached_at_rfc3339: String,
    value: serde_json::Value,
}

pub(super) fn valkey_analysis_key(
    cache_key: &RepositoryAnalysisCacheKey,
    key_prefix: &str,
) -> String {
    let payload = format!(
        "repo:{}|root:{}|analysis:{}|plugins:{}",
        cache_key.repo_id.trim(),
        cache_key.checkout_root.trim(),
        cache_key.analysis_identity.trim(),
        cache_key.plugin_ids.join(","),
    );
    let token = blake3::hash(payload.as_bytes()).to_hex().to_string();
    format!("{key_prefix}:analysis:{token}")
}

pub(super) fn encode_analysis_payload(
    cache_key: &RepositoryAnalysisCacheKey,
    analysis: &RepositoryAnalysisOutput,
) -> Option<String> {
    let revision = cache_key
        .checkout_revision
        .as_deref()
        .or(cache_key.mirror_revision.as_deref())
        .or(cache_key.tracking_revision.as_deref())
        .unwrap_or("unknown");
    serde_json::to_string(&ValkeyAnalysisCachePayload {
        schema: ANALYZER_CACHE_SCHEMA_VERSION.to_string(),
        repo_id: cache_key.repo_id.clone(),
        checkout_root: cache_key.checkout_root.clone(),
        analysis_identity: cache_key.analysis_identity.clone(),
        plugin_ids: cache_key.plugin_ids.clone(),
        revision: revision.to_string(),
        cached_at_rfc3339: Utc::now().to_rfc3339(),
        analysis: analysis.clone(),
    })
    .ok()
}

pub(super) fn valkey_analysis_revision_key(
    repo_id: &str,
    checkout_root: &str,
    plugin_ids: &[String],
    revision: &str,
    key_prefix: &str,
) -> String {
    let payload = format!(
        "repo:{}|root:{}|plugins:{}|revision:{}",
        repo_id.trim(),
        checkout_root.trim(),
        plugin_ids.join(","),
        revision.trim(),
    );
    let token = blake3::hash(payload.as_bytes()).to_hex().to_string();
    format!("{key_prefix}:analysis-revision:{token}")
}

pub(super) fn valkey_search_query_key(
    cache_key: &RepositorySearchQueryCacheKey,
    key_prefix: &str,
) -> String {
    let payload = format!(
        "repo:{}|root:{}|analysis:{}|plugins:{}|endpoint:{}|query:{}|filter:{}|distance:{}|prefix:{}|transpose:{}|limit:{}",
        cache_key.analysis_key.repo_id.trim(),
        cache_key.analysis_key.checkout_root.trim(),
        cache_key.analysis_key.analysis_identity.trim(),
        cache_key.analysis_key.plugin_ids.join(","),
        cache_key.endpoint.trim(),
        cache_key.query.trim(),
        cache_key.filter.as_deref().unwrap_or_default().trim(),
        cache_key.max_distance,
        cache_key.prefix_length,
        cache_key.transposition,
        cache_key.limit,
    );
    let token = blake3::hash(payload.as_bytes()).to_hex().to_string();
    format!("{key_prefix}:search-query:{token}")
}

pub(super) fn encode_search_query_payload<T>(
    cache_key: &RepositorySearchQueryCacheKey,
    value: &T,
) -> Option<String>
where
    T: serde::Serialize,
{
    let revision = cache_key
        .analysis_key
        .checkout_revision
        .as_deref()
        .or(cache_key.analysis_key.mirror_revision.as_deref())
        .or(cache_key.analysis_key.tracking_revision.as_deref())
        .unwrap_or("unknown");
    let encoded_value = serde_json::to_value(value).ok()?;
    serde_json::to_string(&ValkeySearchQueryCachePayload {
        schema: SEARCH_QUERY_CACHE_SCHEMA_VERSION.to_string(),
        repo_id: cache_key.analysis_key.repo_id.clone(),
        analysis_identity: cache_key.analysis_key.analysis_identity.clone(),
        revision: revision.to_string(),
        endpoint: cache_key.endpoint.clone(),
        query: cache_key.query.clone(),
        filter: cache_key.filter.clone(),
        max_distance: cache_key.max_distance,
        prefix_length: cache_key.prefix_length,
        transposition: cache_key.transposition,
        limit: cache_key.limit,
        cached_at_rfc3339: Utc::now().to_rfc3339(),
        value: encoded_value,
    })
    .ok()
}

pub(super) fn decode_analysis_payload(
    cache_key: &RepositoryAnalysisCacheKey,
    payload: &str,
) -> Option<RepositoryAnalysisOutput> {
    let decoded = serde_json::from_str::<ValkeyAnalysisCachePayload>(payload).ok()?;
    if decoded.schema != ANALYZER_CACHE_SCHEMA_VERSION {
        return None;
    }
    if decoded.repo_id != cache_key.repo_id
        || decoded.analysis_identity != cache_key.analysis_identity
    {
        return None;
    }
    Some(decoded.analysis)
}

#[cfg(feature = "search-runtime")]
pub(super) fn decode_analysis_payload_for_revision(
    repo_id: &str,
    checkout_root: &str,
    plugin_ids: &[String],
    revision: &str,
    payload: &str,
) -> Option<RepositoryAnalysisOutput> {
    let decoded = serde_json::from_str::<ValkeyAnalysisCachePayload>(payload).ok()?;
    if decoded.schema != ANALYZER_CACHE_SCHEMA_VERSION {
        return None;
    }
    if decoded.repo_id != repo_id
        || decoded.checkout_root != checkout_root
        || decoded.plugin_ids != plugin_ids
        || decoded.revision != revision
    {
        return None;
    }
    Some(decoded.analysis)
}

pub(super) fn decode_search_query_payload<T>(
    cache_key: &RepositorySearchQueryCacheKey,
    payload: &str,
) -> Option<T>
where
    T: serde::de::DeserializeOwned,
{
    let decoded = serde_json::from_str::<ValkeySearchQueryCachePayload>(payload).ok()?;
    if decoded.schema != SEARCH_QUERY_CACHE_SCHEMA_VERSION {
        return None;
    }
    if decoded.repo_id != cache_key.analysis_key.repo_id
        || decoded.analysis_identity != cache_key.analysis_key.analysis_identity
        || decoded.endpoint != cache_key.endpoint
        || decoded.query != cache_key.query
        || decoded.filter != cache_key.filter
        || decoded.max_distance != cache_key.max_distance
        || decoded.prefix_length != cache_key.prefix_length
        || decoded.transposition != cache_key.transposition
        || decoded.limit != cache_key.limit
    {
        return None;
    }
    serde_json::from_value(decoded.value).ok()
}
