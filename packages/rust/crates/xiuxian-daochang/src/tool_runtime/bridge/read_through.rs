//! Valkey read-through cache for `skill.discover` tool calls.

use std::collections::BTreeMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result};
use redis::Client as ValkeyClient;
use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::tool_runtime::types::ToolRuntimeCallResult;

/// Snapshot of the read-through discover-cache counters.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ToolDiscoverCacheStatsSnapshot {
    /// Total discover-cache lookups attempted.
    pub requests_total: u64,
    /// Number of discover-cache hits.
    pub cache_hits: u64,
    /// Number of discover-cache misses.
    pub cache_misses: u64,
    /// Number of successful cache writes.
    pub cache_writes: u64,
    /// Cache hit rate expressed as a percentage in the `0..=100` range.
    pub hit_rate_pct: f64,
}

/// Configuration for the Valkey-backed discover read-through cache.
#[derive(Clone, Debug)]
pub struct ToolDiscoverCacheConfig {
    /// Valkey connection string.
    pub valkey_url: String,
    /// Prefix used for discover-cache keys.
    pub key_prefix: String,
    /// Cache TTL in seconds.
    pub ttl_secs: u64,
}

/// Runtime metadata for the discover-cache backend currently in use.
#[derive(Clone, Debug)]
pub struct ToolDiscoverCacheRuntimeInfo {
    /// Backend name, for example `valkey`.
    pub backend: &'static str,
    /// Effective TTL in seconds.
    pub ttl_secs: u64,
}

#[derive(Debug)]
struct ToolDiscoverCacheStats {
    requests_total: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
    cache_writes: AtomicU64,
}

impl Default for ToolDiscoverCacheStats {
    fn default() -> Self {
        Self {
            requests_total: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
            cache_writes: AtomicU64::new(0),
        }
    }
}

/// Valkey-backed read-through cache for `skill.discover` tool calls.
#[derive(Debug)]
pub struct ToolDiscoverReadThroughCache {
    client: ValkeyClient,
    key_prefix: String,
    ttl_secs: u64,
    stats: Arc<ToolDiscoverCacheStats>,
}

impl ToolDiscoverReadThroughCache {
    /// Builds a discover-cache client from runtime configuration.
    ///
    /// # Errors
    ///
    /// Returns an error when the Valkey client cannot be constructed from the
    /// provided URL.
    pub fn from_config(config: ToolDiscoverCacheConfig) -> Result<Self> {
        let client = ValkeyClient::open(config.valkey_url.clone())
            .with_context(|| format!("open discover cache valkey client: {}", config.valkey_url))?;
        Ok(Self {
            client,
            key_prefix: config.key_prefix,
            ttl_secs: config.ttl_secs,
            stats: Arc::new(ToolDiscoverCacheStats::default()),
        })
    }

    #[must_use]
    /// Returns static runtime metadata about the discover-cache backend.
    pub fn runtime_info(&self) -> ToolDiscoverCacheRuntimeInfo {
        ToolDiscoverCacheRuntimeInfo {
            backend: "valkey",
            ttl_secs: self.ttl_secs,
        }
    }

    #[must_use]
    /// Returns a point-in-time snapshot of discover-cache counters.
    pub fn stats_snapshot(&self) -> ToolDiscoverCacheStatsSnapshot {
        let requests_total = self.stats.requests_total.load(Ordering::Relaxed);
        let cache_hits = self.stats.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.stats.cache_misses.load(Ordering::Relaxed);
        let cache_writes = self.stats.cache_writes.load(Ordering::Relaxed);
        let hit_rate_pct = if requests_total == 0 {
            0.0
        } else {
            ratio_pct(cache_hits, requests_total)
        };
        ToolDiscoverCacheStatsSnapshot {
            requests_total,
            cache_hits,
            cache_misses,
            cache_writes,
            hit_rate_pct,
        }
    }

    /// Looks up a cached `skill.discover` result by normalized tool arguments.
    ///
    /// # Errors
    ///
    /// Returns an error when the cache key cannot be serialized, Valkey cannot
    /// be reached, or a cached payload cannot be decoded.
    pub async fn lookup(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
    ) -> Result<Option<ToolRuntimeCallResult>> {
        self.stats.requests_total.fetch_add(1, Ordering::Relaxed);
        let key = self.cache_key(tool_name, arguments)?;
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .context("connect discover cache valkey")?;
        let payload: Option<String> = redis::cmd("GET")
            .arg(&key)
            .query_async(&mut connection)
            .await
            .with_context(|| format!("discover cache GET {key}"))?;
        if let Some(payload) = payload {
            self.stats.cache_hits.fetch_add(1, Ordering::Relaxed);
            let result = serde_json::from_str(&payload)
                .with_context(|| format!("decode discover cache payload for {key}"))?;
            Ok(Some(result))
        } else {
            self.stats.cache_misses.fetch_add(1, Ordering::Relaxed);
            Ok(None)
        }
    }

    /// Stores a successful `skill.discover` result in the read-through cache.
    ///
    /// # Errors
    ///
    /// Returns an error when the cache key cannot be serialized, the result
    /// payload cannot be encoded, or Valkey write operations fail.
    pub async fn store(
        &self,
        tool_name: &str,
        arguments: &serde_json::Value,
        result: &ToolRuntimeCallResult,
    ) -> Result<()> {
        let key = self.cache_key(tool_name, arguments)?;
        let payload = serde_json::to_string(result)
            .with_context(|| format!("encode discover cache payload for {key}"))?;
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .context("connect discover cache valkey")?;
        let _: () = redis::cmd("SETEX")
            .arg(&key)
            .arg(self.ttl_secs)
            .arg(payload)
            .query_async(&mut connection)
            .await
            .with_context(|| format!("discover cache SETEX {key}"))?;
        self.stats.cache_writes.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    fn cache_key(&self, tool_name: &str, arguments: &serde_json::Value) -> Result<String> {
        let normalized = normalize_json(arguments);
        let canonical =
            serde_json::to_string(&normalized).context("serialize discover cache key")?;
        let mut digest = Sha256::new();
        digest.update(tool_name.as_bytes());
        digest.update([0]);
        digest.update(canonical.as_bytes());
        let digest_hex = hex::encode(digest.finalize());
        Ok(format!("{}:{tool_name}:{digest_hex}", self.key_prefix))
    }
}

fn normalize_json(value: &serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => {
            let sorted: BTreeMap<String, serde_json::Value> = map
                .iter()
                .map(|(key, value)| (key.clone(), normalize_json(value)))
                .collect();
            let normalized = sorted.into_iter().collect();
            serde_json::Value::Object(normalized)
        }
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.iter().map(normalize_json).collect())
        }
        _ => value.clone(),
    }
}

fn ratio_pct(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        return 0.0;
    }
    let basis_points = numerator.saturating_mul(10_000) / denominator;
    f64::from(u32::try_from(basis_points).unwrap_or(10_000)) / 100.0
}
