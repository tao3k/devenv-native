//! Cache runtime configuration records for Wendao host state.

use crate::config::constants::DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX;

/// Resolved cache runtime settings for link-graph state backed by Valkey.
#[derive(Debug, Clone)]
pub struct LinkGraphCacheRuntimeConfig {
    /// Resolved Valkey connection URL.
    pub valkey_url: String,
    /// Resolved key prefix used for cache entries.
    pub key_prefix: String,
    /// Optional TTL applied to cache entries, in seconds.
    pub ttl_seconds: Option<u64>,
}

impl LinkGraphCacheRuntimeConfig {
    /// Build a cache runtime record from normalized cache settings.
    #[must_use]
    pub fn from_parts(
        valkey_url: &str,
        key_prefix: Option<&str>,
        ttl_seconds: Option<u64>,
    ) -> Self {
        let resolved_url = valkey_url.trim().to_string();
        let resolved_prefix = key_prefix
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or(DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX)
            .to_string();
        Self {
            valkey_url: resolved_url,
            key_prefix: resolved_prefix,
            ttl_seconds: ttl_seconds.filter(|value| *value > 0),
        }
    }
}
