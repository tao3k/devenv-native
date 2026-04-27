use super::super::fingerprint::LinkGraphFingerprint;
use super::CacheLookupOutcome;
use super::schema::{LINK_GRAPH_CACHE_SCHEMA_VERSION, cache_schema_fingerprint};
use super::snapshot::LinkGraphIndexSnapshot;
use crate::link_graph::index::LinkGraphIndex;
use crate::link_graph::runtime_config::LinkGraphCacheRuntimeConfig;
use crate::valkey_common::open_client;
use std::path::Path;

fn valkey_cache_key(slot_key: &str, key_prefix: &str) -> String {
    format!("{key_prefix}:{slot_key}")
}

fn redis_client(valkey_url: &str) -> Result<redis::Client, String> {
    open_client(valkey_url).map_err(|err| format!("invalid valkey url for link-graph cache: {err}"))
}

pub(in crate::link_graph::index::build::cache) fn decode_cached_index_payload(
    raw: &str,
    root: &Path,
    include_dirs: &[String],
    excluded_dirs: &[String],
    fingerprint: &LinkGraphFingerprint,
) -> CacheLookupOutcome {
    let Ok(snapshot) = serde_json::from_str::<LinkGraphIndexSnapshot>(raw) else {
        return CacheLookupOutcome::Miss("payload_parse_error");
    };
    if snapshot.schema_version() != LINK_GRAPH_CACHE_SCHEMA_VERSION {
        return CacheLookupOutcome::Miss("schema_version_mismatch");
    }
    if snapshot.schema_fingerprint() != Some(cache_schema_fingerprint()) {
        return CacheLookupOutcome::Miss("schema_fingerprint_mismatch");
    }
    if snapshot.root() != root {
        return CacheLookupOutcome::Miss("root_mismatch");
    }
    if snapshot.include_dirs() != include_dirs {
        return CacheLookupOutcome::Miss("include_dirs_mismatch");
    }
    if snapshot.excluded_dirs() != excluded_dirs {
        return CacheLookupOutcome::Miss("excluded_dirs_mismatch");
    }
    if snapshot.fingerprint() != fingerprint {
        return CacheLookupOutcome::Miss("content_fingerprint_mismatch");
    }
    CacheLookupOutcome::Hit(Box::new(snapshot.into_index()))
}

pub(in crate::link_graph::index::build::cache) fn encode_cached_index_payload(
    index: &LinkGraphIndex,
    fingerprint: LinkGraphFingerprint,
) -> Result<String, String> {
    let payload = LinkGraphIndexSnapshot::from_index(index, fingerprint);
    serde_json::to_string(&payload)
        .map_err(|e| format!("failed to serialize link-graph cache payload: {e}"))
}

pub(in crate::link_graph::index::build) fn load_cached_index_from_valkey(
    runtime: &LinkGraphCacheRuntimeConfig,
    slot_key: &str,
    root: &Path,
    include_dirs: &[String],
    excluded_dirs: &[String],
    fingerprint: &LinkGraphFingerprint,
) -> Result<CacheLookupOutcome, String> {
    let cache_key = valkey_cache_key(slot_key, &runtime.key_prefix);
    let client = redis_client(runtime.valkey_url.as_str())?;
    let mut conn = client
        .get_connection()
        .map_err(|e| format!("failed to connect valkey for link-graph cache: {e}"))?;
    let raw = redis::cmd("GET")
        .arg(&cache_key)
        .query::<Option<String>>(&mut conn)
        .map_err(|e| format!("failed to GET link-graph cache from valkey: {e}"))?;
    let Some(payload) = raw else {
        return Ok(CacheLookupOutcome::Miss("key_not_found"));
    };
    Ok(decode_cached_index_payload(
        &payload,
        root,
        include_dirs,
        excluded_dirs,
        fingerprint,
    ))
}

pub(in crate::link_graph::index::build) fn save_cached_index_to_valkey(
    index: &LinkGraphIndex,
    runtime: &LinkGraphCacheRuntimeConfig,
    slot_key: &str,
    fingerprint: LinkGraphFingerprint,
) -> Result<(), String> {
    let cache_key = valkey_cache_key(slot_key, &runtime.key_prefix);
    let encoded = encode_cached_index_payload(index, fingerprint)?;
    let client = redis_client(runtime.valkey_url.as_str())?;
    let mut conn = client
        .get_connection()
        .map_err(|e| format!("failed to connect valkey for link-graph cache: {e}"))?;
    if let Some(ttl_seconds) = runtime.ttl_seconds {
        redis::cmd("SETEX")
            .arg(&cache_key)
            .arg(ttl_seconds)
            .arg(&encoded)
            .query::<()>(&mut conn)
            .map_err(|e| format!("failed to SETEX link-graph cache to valkey: {e}"))?;
    } else {
        redis::cmd("SET")
            .arg(&cache_key)
            .arg(&encoded)
            .query::<()>(&mut conn)
            .map_err(|e| format!("failed to SET link-graph cache to valkey: {e}"))?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "../../../../../tests/unit/link_graph/index/build/cache/storage.rs"]
mod tests;
