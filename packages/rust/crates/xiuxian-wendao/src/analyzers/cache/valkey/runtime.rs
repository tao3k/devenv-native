use crate::analyzers::RepoIntelligenceError;
use crate::settings::{get_setting_string, merged_wendao_settings};
use crate::valkey_common::{normalize_key_prefix, open_client};
use serde_yaml::Value;
use xiuxian_config_core::{toml_first_env, toml_first_named_string};

const ANALYZER_VALKEY_URL_SETTING: &str = "analyzers.cache.valkey_url";
const ANALYZER_VALKEY_KEY_PREFIX_SETTING: &str = "analyzers.cache.key_prefix";
const ANALYZER_VALKEY_TTL_SETTING: &str = "analyzers.cache.ttl_seconds";
const ANALYZER_VALKEY_URL_ENV: &str = "XIUXIAN_WENDAO_ANALYZER_VALKEY_URL";
const VALKEY_URL_ENV: &str = "VALKEY_URL";
const REDIS_URL_ENV: &str = "REDIS_URL";
const ANALYZER_VALKEY_KEY_PREFIX_ENV: &str = "XIUXIAN_WENDAO_ANALYZER_VALKEY_KEY_PREFIX";
const ANALYZER_VALKEY_TTL_ENV: &str = "XIUXIAN_WENDAO_ANALYZER_VALKEY_TTL_SECS";
const DEFAULT_ANALYZER_VALKEY_KEY_PREFIX: &str = "xiuxian_wendao:repo_analysis";

#[derive(Debug, Clone)]
pub(super) struct ValkeyAnalysisCacheRuntime {
    pub(super) client: Option<redis::Client>,
    pub(super) key_prefix: String,
    pub(super) ttl_seconds: Option<u64>,
}

impl ValkeyAnalysisCacheRuntime {
    #[cfg(all(test, feature = "zhenfa-router"))]
    pub(super) fn for_tests(key_prefix: &str, ttl_seconds: Option<u64>) -> Self {
        Self {
            client: None,
            key_prefix: normalize_key_prefix(key_prefix, DEFAULT_ANALYZER_VALKEY_KEY_PREFIX),
            ttl_seconds,
        }
    }
}

pub(super) fn resolve_valkey_analysis_cache_runtime()
-> Result<Option<ValkeyAnalysisCacheRuntime>, RepoIntelligenceError> {
    let settings = merged_wendao_settings();
    resolve_valkey_analysis_cache_runtime_with_settings_and_lookup(&settings, &|name| {
        std::env::var(name).ok()
    })
}

#[cfg(all(test, feature = "zhenfa-router"))]
pub(super) fn resolve_valkey_analysis_cache_runtime_with_lookup(
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Option<ValkeyAnalysisCacheRuntime>, RepoIntelligenceError> {
    resolve_valkey_analysis_cache_runtime_with_settings_and_lookup(&Value::Null, lookup)
}

#[cfg(all(test, feature = "zhenfa-router"))]
pub(super) fn resolve_valkey_analysis_cache_runtime_with_settings_and_lookup_for_tests(
    settings: &Value,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Option<ValkeyAnalysisCacheRuntime>, RepoIntelligenceError> {
    resolve_valkey_analysis_cache_runtime_with_settings_and_lookup(settings, lookup)
}

fn resolve_valkey_analysis_cache_runtime_with_settings_and_lookup(
    settings: &Value,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Option<ValkeyAnalysisCacheRuntime>, RepoIntelligenceError> {
    let Some(candidate) = toml_first_named_string(
        ANALYZER_VALKEY_URL_SETTING,
        get_setting_string(settings, ANALYZER_VALKEY_URL_SETTING),
        lookup,
        &[ANALYZER_VALKEY_URL_ENV, VALKEY_URL_ENV, REDIS_URL_ENV],
    ) else {
        return Ok(None);
    };
    let client = open_client(candidate.value.as_str()).map_err(|error| {
        RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "invalid analyzer valkey url from {}: {error}",
                candidate.source_name
            ),
        }
    })?;
    let key_prefix = normalize_key_prefix(
        toml_first_env!(
            settings,
            ANALYZER_VALKEY_KEY_PREFIX_SETTING,
            lookup,
            [ANALYZER_VALKEY_KEY_PREFIX_ENV],
            get_setting_string
        )
        .unwrap_or_default()
        .as_str(),
        DEFAULT_ANALYZER_VALKEY_KEY_PREFIX,
    );
    let ttl_seconds = resolve_optional_ttl_seconds_with_settings_and_lookup(settings, lookup)?;
    Ok(Some(ValkeyAnalysisCacheRuntime {
        client: Some(client),
        key_prefix,
        ttl_seconds,
    }))
}

fn resolve_optional_ttl_seconds_with_settings_and_lookup(
    settings: &Value,
    lookup: &dyn Fn(&str) -> Option<String>,
) -> Result<Option<u64>, RepoIntelligenceError> {
    let Some(raw_ttl) = toml_first_env!(
        settings,
        ANALYZER_VALKEY_TTL_SETTING,
        lookup,
        [ANALYZER_VALKEY_TTL_ENV],
        get_setting_string
    ) else {
        return Ok(None);
    };
    let ttl_seconds = raw_ttl.parse::<u64>().map_err(|error| {
        RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "{ANALYZER_VALKEY_TTL_ENV} must be a non-negative integer, got `{raw_ttl}`: {error}"
            ),
        }
    })?;
    Ok((ttl_seconds > 0).then_some(ttl_seconds))
}
