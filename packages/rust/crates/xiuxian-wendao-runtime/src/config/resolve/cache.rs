//! Cache configuration resolver for runtime host behavior.

use crate::config::LinkGraphCacheRuntimeConfig;
use crate::config::constants::{
    DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX, LINK_GRAPH_CACHE_VALKEY_URL_ENV,
    LINK_GRAPH_VALKEY_KEY_PREFIX_ENV, LINK_GRAPH_VALKEY_TTL_SECONDS_ENV,
};
use crate::settings::get_setting_string;
use serde_yaml::Value;
use xiuxian_config_core::{toml_first_env, toml_first_named_string};

const LINK_GRAPH_CACHE_VALKEY_URL_SETTING: &str = "link_graph.cache.valkey_url";
const LINK_GRAPH_CACHE_KEY_PREFIX_SETTING: &str = "link_graph.cache.key_prefix";
const LINK_GRAPH_CACHE_TTL_SECONDS_SETTING: &str = "link_graph.cache.ttl_seconds";

/// Resolve runtime cache configuration from merged settings and environment.
///
/// # Errors
///
/// Returns an error when no Valkey URL can be resolved from config or env.
pub fn resolve_link_graph_cache_runtime_with_settings(
    settings: &Value,
) -> Result<LinkGraphCacheRuntimeConfig, String> {
    resolve_link_graph_cache_runtime_with_settings_and_lookup(settings, &|name| {
        std::env::var(name).ok()
    })
}

fn resolve_link_graph_cache_runtime_with_settings_and_lookup(
    settings: &Value,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<LinkGraphCacheRuntimeConfig, String> {
    let valkey_url = toml_first_named_string(
        LINK_GRAPH_CACHE_VALKEY_URL_SETTING,
        get_setting_string(settings, LINK_GRAPH_CACHE_VALKEY_URL_SETTING),
        lookup,
        &[LINK_GRAPH_CACHE_VALKEY_URL_ENV],
    )
    .map(|(_, value)| value)
    .ok_or_else(|| {
        format!(
            "link_graph cache valkey url is required (set {LINK_GRAPH_CACHE_VALKEY_URL_SETTING} or {LINK_GRAPH_CACHE_VALKEY_URL_ENV})"
        )
    })?;

    let key_prefix = toml_first_env!(
        settings,
        LINK_GRAPH_CACHE_KEY_PREFIX_SETTING,
        lookup,
        [LINK_GRAPH_VALKEY_KEY_PREFIX_ENV],
        get_setting_string
    )
    .unwrap_or_else(|| DEFAULT_LINK_GRAPH_VALKEY_KEY_PREFIX.to_string());

    let ttl_seconds = toml_first_env!(
        settings,
        LINK_GRAPH_CACHE_TTL_SECONDS_SETTING,
        lookup,
        [LINK_GRAPH_VALKEY_TTL_SECONDS_ENV],
        get_setting_string,
        xiuxian_config_core::parse_positive::<u64>
    );

    Ok(LinkGraphCacheRuntimeConfig::from_parts(
        &valkey_url,
        Some(&key_prefix),
        ttl_seconds,
    ))
}

#[cfg(test)]
#[path = "../../../tests/unit/config/resolve/cache.rs"]
mod tests;
