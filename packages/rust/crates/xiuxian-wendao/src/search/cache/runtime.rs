use serde_yaml::Value;
use xiuxian_config_core::{first_non_empty_named_lookup, toml_first_env, trimmed_non_empty};

use crate::settings::{get_setting_string, merged_wendao_settings};
use crate::valkey_common::open_client;

use super::config::SearchPlaneCacheConfig;

const SEARCH_CACHE_VALKEY_URL_SETTING: &str = "search.cache.valkey_url";
const SEARCH_PLANE_VALKEY_URL_ENV: &str = "XIUXIAN_WENDAO_SEARCH_PLANE_VALKEY_URL";
const KNOWLEDGE_VALKEY_URL_ENV: &str = "XIUXIAN_WENDAO_KNOWLEDGE_VALKEY_URL";
const VALKEY_URL_ENV: &str = "VALKEY_URL";
const REDIS_URL_ENV: &str = "REDIS_URL";

#[derive(Debug, Clone)]
pub(crate) struct SearchPlaneCacheRuntime {
    pub(crate) client: Option<redis::Client>,
    #[cfg(test)]
    pub(crate) valkey_url: Option<String>,
    pub(crate) config: SearchPlaneCacheConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Resolved cache connection target for diagnostics and startup health checks.
pub struct SearchPlaneCacheConnectionTarget {
    /// Effective Valkey URL selected from settings or environment.
    pub valkey_url: String,
    /// Effective cache timeout and TTL configuration.
    pub config: SearchPlaneCacheConfig,
}

pub(crate) fn resolve_search_plane_cache_runtime() -> SearchPlaneCacheRuntime {
    let settings = merged_wendao_settings();
    resolve_search_plane_cache_runtime_with_lookup(&settings, &|name| std::env::var(name).ok())
}

/// Resolve the cache connection target without opening the runtime cache client.
///
/// # Errors
///
/// Returns a message when an explicitly configured Valkey URL is invalid.
pub fn resolve_search_plane_cache_connection_target()
-> Result<SearchPlaneCacheConnectionTarget, String> {
    let settings = merged_wendao_settings();
    resolve_search_plane_cache_connection_target_with_lookup(&settings, &|name| {
        std::env::var(name).ok()
    })
}

fn resolve_search_plane_cache_runtime_with_lookup(
    settings: &Value,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> SearchPlaneCacheRuntime {
    let config = SearchPlaneCacheConfig::from_settings_and_env(settings, lookup);
    let valkey_url = toml_first_env!(
        settings,
        SEARCH_CACHE_VALKEY_URL_SETTING,
        lookup,
        [
            SEARCH_PLANE_VALKEY_URL_ENV,
            KNOWLEDGE_VALKEY_URL_ENV,
            VALKEY_URL_ENV,
            REDIS_URL_ENV
        ],
        get_setting_string,
        |raw| {
            let trimmed = raw.trim();
            open_client(trimmed).ok().map(|_| trimmed.to_string())
        }
    );
    SearchPlaneCacheRuntime {
        client: valkey_url
            .as_ref()
            .and_then(|value| open_client(value.as_str()).ok()),
        #[cfg(test)]
        valkey_url,
        config,
    }
}

fn resolve_search_plane_cache_connection_target_with_lookup(
    settings: &Value,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<SearchPlaneCacheConnectionTarget, String> {
    let config = SearchPlaneCacheConfig::from_settings_and_env(settings, lookup);
    if let Some(raw_setting_url) = trimmed_non_empty(get_setting_string(
        settings,
        SEARCH_CACHE_VALKEY_URL_SETTING,
    )) {
        open_client(raw_setting_url.as_str()).map_err(|error| {
            format!("invalid search.cache.valkey_url from wendao.toml: {error}")
        })?;
        return Ok(SearchPlaneCacheConnectionTarget {
            valkey_url: raw_setting_url,
            config,
        });
    }

    let Some((env_name, env_url)) = first_non_empty_named_lookup(
        &[
            SEARCH_PLANE_VALKEY_URL_ENV,
            KNOWLEDGE_VALKEY_URL_ENV,
            VALKEY_URL_ENV,
            REDIS_URL_ENV,
        ],
        lookup,
    ) else {
        return Err(
            "missing search cache valkey url; set search.cache.valkey_url or one of XIUXIAN_WENDAO_SEARCH_PLANE_VALKEY_URL, XIUXIAN_WENDAO_KNOWLEDGE_VALKEY_URL, VALKEY_URL, REDIS_URL".to_string(),
        );
    };
    open_client(env_url.as_str())
        .map_err(|error| format!("invalid search cache valkey url from {env_name}: {error}"))?;
    Ok(SearchPlaneCacheConnectionTarget {
        valkey_url: env_url,
        config,
    })
}

#[cfg(test)]
#[path = "../../../tests/unit/search/cache/runtime.rs"]
mod tests;
