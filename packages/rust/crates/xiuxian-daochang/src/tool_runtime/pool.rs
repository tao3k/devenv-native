//! External tool client pool bootstrap.

use anyhow::Result;

use super::bridge::{self, ToolClientPool, ToolPoolConnectConfig};
use super::discover_cache;

/// Build an external tool client pool from URL with runtime-resolved discover-cache wiring.
///
/// # Errors
/// Returns an error when discover-cache initialization or client-pool bootstrap fails.
pub async fn connect_tool_pool(url: &str, config: ToolPoolConnectConfig) -> Result<ToolClientPool> {
    let discover_cache = match discover_cache::discover_cache_from_runtime() {
        Ok(cache) => cache,
        Err(error) => {
            tracing::warn!(
                event = "tool_runtime.pool.discover_cache.init_failed",
                error = %error,
                "discover read-through cache init failed; continuing without cache"
            );
            None
        }
    };
    if let Some(cache) = discover_cache.as_ref() {
        let runtime = cache.runtime_info();
        tracing::info!(
            event = "tool_runtime.pool.discover_cache.enabled",
            backend = runtime.backend,
            ttl_secs = runtime.ttl_secs,
            "discover read-through cache enabled"
        );
    }
    bridge::connect_tool_pool_backend(url, config, discover_cache).await
}
