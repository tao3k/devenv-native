//! Discover tool-call read-through cache runtime wiring.
//!
//! Core cache behavior is provided by the shared tool-runtime backend; this
//! file only resolves runtime settings/environment for xiuxian-daochang.

use std::sync::Arc;

use anyhow::Result;
use xiuxian_macros::env_non_empty;

use super::bridge::{ToolDiscoverCacheConfig, ToolDiscoverReadThroughCache};
use crate::config::load_runtime_settings;
use crate::env_parse::{parse_bool_from_env, parse_positive_u64_from_env, resolve_valkey_url_env};

const DEFAULT_DISCOVER_CACHE_KEY_PREFIX: &str = "xiuxian-daochang:discover";
const DEFAULT_DISCOVER_CACHE_TTL_SECS: u64 = 30;
const MAX_DISCOVER_CACHE_TTL_SECS: u64 = 3_600;

/// Build discover cache from env + runtime settings.
///
/// Returns `Ok(None)` when cache is disabled or no valkey url is configured.
pub(super) fn discover_cache_from_runtime() -> Result<Option<Arc<ToolDiscoverReadThroughCache>>> {
    let Some(config) = resolve_discover_cache_config() else {
        return Ok(None);
    };
    let cache = ToolDiscoverReadThroughCache::from_config(config)?;
    Ok(Some(Arc::new(cache)))
}

fn resolve_discover_cache_config() -> Option<ToolDiscoverCacheConfig> {
    let settings = load_runtime_settings();

    let enabled = parse_bool_from_env("XIUXIAN_DAOCHANG_TOOL_DISCOVER_CACHE_ENABLED")
        .or(settings.tool_runtime.discover_cache_enabled)
        .unwrap_or(true);
    if !enabled {
        return None;
    }

    let valkey_url = settings
        .session
        .valkey_url
        .as_deref()
        .map(str::trim)
        .map(str::to_string)
        .filter(|value| !value.is_empty())
        .or_else(resolve_valkey_url_env)?;

    let key_prefix = env_non_empty!("XIUXIAN_DAOCHANG_TOOL_DISCOVER_CACHE_KEY_PREFIX")
        .or_else(|| {
            settings
                .tool_runtime
                .discover_cache_key_prefix
                .as_deref()
                .map(str::trim)
                .map(str::to_string)
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| DEFAULT_DISCOVER_CACHE_KEY_PREFIX.to_string());

    let ttl_secs = parse_positive_u64_from_env("XIUXIAN_DAOCHANG_TOOL_DISCOVER_CACHE_TTL_SECS")
        .or(settings
            .tool_runtime
            .discover_cache_ttl_secs
            .filter(|value| *value > 0))
        .unwrap_or(DEFAULT_DISCOVER_CACHE_TTL_SECS)
        .clamp(1, MAX_DISCOVER_CACHE_TTL_SECS);

    Some(ToolDiscoverCacheConfig {
        valkey_url,
        key_prefix,
        ttl_secs,
    })
}
