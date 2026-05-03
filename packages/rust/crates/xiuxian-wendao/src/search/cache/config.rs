use crate::settings::get_setting_string;
use serde_yaml::Value;
use std::time::Duration;

const QUERY_CACHE_TTL_ENV: &str = "XIUXIAN_WENDAO_SEARCH_PLANE_QUERY_CACHE_TTL_SEC";
const AUTOCOMPLETE_CACHE_TTL_ENV: &str = "XIUXIAN_WENDAO_SEARCH_PLANE_AUTOCOMPLETE_CACHE_TTL_SEC";
const REPO_REVISION_RETENTION_ENV: &str = "XIUXIAN_WENDAO_SEARCH_PLANE_REPO_REVISION_RETENTION";
const CACHE_CONNECTION_TIMEOUT_MS_ENV: &str =
    "XIUXIAN_WENDAO_SEARCH_PLANE_CACHE_CONNECTION_TIMEOUT_MS";
const CACHE_RESPONSE_TIMEOUT_MS_ENV: &str = "XIUXIAN_WENDAO_SEARCH_PLANE_CACHE_RESPONSE_TIMEOUT_MS";
const QUERY_CACHE_TTL_SETTING: &str = "search.cache.query_ttl_seconds";
const AUTOCOMPLETE_CACHE_TTL_SETTING: &str = "search.cache.autocomplete_ttl_seconds";
const REPO_REVISION_RETENTION_SETTING: &str = "search.cache.repo_revision_retention";
const CACHE_CONNECTION_TIMEOUT_MS_SETTING: &str = "search.cache.connection_timeout_ms";
const CACHE_RESPONSE_TIMEOUT_MS_SETTING: &str = "search.cache.response_timeout_ms";

const DEFAULT_QUERY_CACHE_TTL_SEC: u64 = 90;
const DEFAULT_AUTOCOMPLETE_CACHE_TTL_SEC: u64 = 300;
const DEFAULT_REPO_REVISION_RETENTION: usize = 32;
const DEFAULT_CACHE_CONNECTION_TIMEOUT_MS: u64 = 25;
const DEFAULT_CACHE_RESPONSE_TIMEOUT_MS: u64 = 25;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Cache TTL family used by search-plane query caches.
pub enum SearchPlaneCacheTtl {
    /// TTL for normal search query results.
    HotQuery,
    /// TTL for autocomplete result sets.
    Autocomplete,
}

impl SearchPlaneCacheTtl {
    pub(crate) fn as_seconds(self, config: &SearchPlaneCacheConfig) -> u64 {
        match self {
            Self::HotQuery => config.query_ttl_seconds,
            Self::Autocomplete => config.autocomplete_ttl_seconds,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Effective cache configuration for search-plane Valkey interactions.
pub struct SearchPlaneCacheConfig {
    /// Hot query TTL in seconds.
    pub query_ttl_seconds: u64,
    /// Autocomplete TTL in seconds.
    pub autocomplete_ttl_seconds: u64,
    /// Number of repository revisions retained in cache metadata.
    pub repo_revision_retention: usize,
    /// Timeout used while opening the cache connection.
    pub connection_timeout: Duration,
    /// Timeout used while waiting for cache responses.
    pub response_timeout: Duration,
}

impl Default for SearchPlaneCacheConfig {
    fn default() -> Self {
        Self {
            query_ttl_seconds: DEFAULT_QUERY_CACHE_TTL_SEC,
            autocomplete_ttl_seconds: DEFAULT_AUTOCOMPLETE_CACHE_TTL_SEC,
            repo_revision_retention: DEFAULT_REPO_REVISION_RETENTION,
            connection_timeout: Duration::from_millis(DEFAULT_CACHE_CONNECTION_TIMEOUT_MS),
            response_timeout: Duration::from_millis(DEFAULT_CACHE_RESPONSE_TIMEOUT_MS),
        }
    }
}

impl SearchPlaneCacheConfig {
    pub(crate) fn from_settings_and_env(
        settings: &Value,
        lookup: &dyn Fn(&str) -> Option<String>,
    ) -> Self {
        Self {
            query_ttl_seconds: xiuxian_config_core::toml_first_env!(
                settings,
                QUERY_CACHE_TTL_SETTING,
                lookup,
                [QUERY_CACHE_TTL_ENV],
                get_setting_string,
                |raw| raw.trim().parse::<u64>().ok()
            )
            .unwrap_or(DEFAULT_QUERY_CACHE_TTL_SEC),
            autocomplete_ttl_seconds: xiuxian_config_core::toml_first_env!(
                settings,
                AUTOCOMPLETE_CACHE_TTL_SETTING,
                lookup,
                [AUTOCOMPLETE_CACHE_TTL_ENV],
                get_setting_string,
                |raw| raw.trim().parse::<u64>().ok()
            )
            .unwrap_or(DEFAULT_AUTOCOMPLETE_CACHE_TTL_SEC),
            repo_revision_retention: xiuxian_config_core::toml_first_env!(
                settings,
                REPO_REVISION_RETENTION_SETTING,
                lookup,
                [REPO_REVISION_RETENTION_ENV],
                get_setting_string,
                |raw| raw.trim().parse::<u64>().ok()
            )
            .and_then(|value| usize::try_from(value).ok())
            .filter(|value| *value > 0)
            .unwrap_or(DEFAULT_REPO_REVISION_RETENTION),
            connection_timeout: Duration::from_millis(
                xiuxian_config_core::toml_first_env!(
                    settings,
                    CACHE_CONNECTION_TIMEOUT_MS_SETTING,
                    lookup,
                    [CACHE_CONNECTION_TIMEOUT_MS_ENV],
                    get_setting_string,
                    |raw| raw.trim().parse::<u64>().ok()
                )
                .unwrap_or(DEFAULT_CACHE_CONNECTION_TIMEOUT_MS),
            ),
            response_timeout: Duration::from_millis(
                xiuxian_config_core::toml_first_env!(
                    settings,
                    CACHE_RESPONSE_TIMEOUT_MS_SETTING,
                    lookup,
                    [CACHE_RESPONSE_TIMEOUT_MS_ENV],
                    get_setting_string,
                    |raw| raw.trim().parse::<u64>().ok()
                )
                .unwrap_or(DEFAULT_CACHE_RESPONSE_TIMEOUT_MS),
            ),
        }
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/search/cache/config.rs"]
mod tests;
