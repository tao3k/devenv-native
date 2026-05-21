//! Settings extraction for Julia link-graph compatibility artifacts.

use serde_yaml::Value;

use super::runtime::LinkGraphJuliaRerankRuntimeConfig;
use crate::{
    JuliaContractMode, JuliaContractPath, JuliaContractRoute, JuliaContractSchemaVersion,
    JuliaContractSecondsU64, JuliaContractUrl,
};

const LINK_GRAPH_JULIA_RERANK_BASE_URL_KEY: &str = "link_graph.retrieval.julia_rerank.base_url";
const LINK_GRAPH_JULIA_RERANK_ROUTE_KEY: &str = "link_graph.retrieval.julia_rerank.route";
const LINK_GRAPH_JULIA_RERANK_HEALTH_ROUTE_KEY: &str =
    "link_graph.retrieval.julia_rerank.health_route";
const LINK_GRAPH_JULIA_RERANK_SCHEMA_VERSION_KEY: &str =
    "link_graph.retrieval.julia_rerank.schema_version";
const LINK_GRAPH_JULIA_RERANK_TIMEOUT_SECS_KEY: &str =
    "link_graph.retrieval.julia_rerank.timeout_secs";
const LINK_GRAPH_JULIA_RERANK_SERVICE_MODE_KEY: &str =
    "link_graph.retrieval.julia_rerank.service_mode";
const LINK_GRAPH_JULIA_RERANK_SEARCH_CONFIG_PATH_KEY: &str =
    "link_graph.retrieval.julia_rerank.search_config_path";
const LINK_GRAPH_JULIA_RERANK_VECTOR_WEIGHT_KEY: &str =
    "link_graph.retrieval.julia_rerank.vector_weight";
const LINK_GRAPH_JULIA_RERANK_SIMILARITY_WEIGHT_KEY: &str =
    "link_graph.retrieval.julia_rerank.similarity_weight";

/// Environment variable that overrides `link_graph.retrieval.julia_rerank.base_url`.
pub const LINK_GRAPH_JULIA_RERANK_BASE_URL_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_JULIA_RERANK_BASE_URL";
/// Environment variable that overrides `link_graph.retrieval.julia_rerank.route`.
pub const LINK_GRAPH_JULIA_RERANK_ROUTE_ENV: &str = "XIUXIAN_WENDAO_LINK_GRAPH_JULIA_RERANK_ROUTE";
/// Environment variable that overrides `link_graph.retrieval.julia_rerank.health_route`.
pub const LINK_GRAPH_JULIA_RERANK_HEALTH_ROUTE_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_JULIA_RERANK_HEALTH_ROUTE";
/// Environment variable that overrides `link_graph.retrieval.julia_rerank.schema_version`.
pub const LINK_GRAPH_JULIA_RERANK_SCHEMA_VERSION_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_JULIA_RERANK_SCHEMA_VERSION";
/// Environment variable that overrides `link_graph.retrieval.julia_rerank.timeout_secs`.
pub const LINK_GRAPH_JULIA_RERANK_TIMEOUT_SECS_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_JULIA_RERANK_TIMEOUT_SECS";
/// Environment variable that overrides `link_graph.retrieval.julia_rerank.service_mode`.
pub const LINK_GRAPH_JULIA_RERANK_SERVICE_MODE_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_JULIA_RERANK_SERVICE_MODE";
/// Environment variable that overrides `link_graph.retrieval.julia_rerank.search_config_path`.
pub const LINK_GRAPH_JULIA_RERANK_SEARCH_CONFIG_PATH_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_JULIA_RERANK_SEARCH_CONFIG_PATH";
/// Environment variable that overrides `link_graph.retrieval.julia_rerank.vector_weight`.
pub const LINK_GRAPH_JULIA_RERANK_VECTOR_WEIGHT_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_JULIA_RERANK_VECTOR_WEIGHT";
/// Environment variable that overrides `link_graph.retrieval.julia_rerank.similarity_weight`.
pub const LINK_GRAPH_JULIA_RERANK_SIMILARITY_WEIGHT_ENV: &str =
    "XIUXIAN_WENDAO_LINK_GRAPH_JULIA_RERANK_SIMILARITY_WEIGHT";

impl LinkGraphJuliaRerankRuntimeConfig {
    /// Resolve the Julia rerank runtime config from Wendao settings and process environment.
    #[must_use]
    pub fn resolve_with_settings(settings: &Value) -> Self {
        Self::resolve_with_env_lookup(settings, |name| std::env::var(name).ok())
    }

    /// Resolve the Julia rerank runtime config from Wendao settings and a caller-provided
    /// environment lookup.
    #[must_use]
    pub fn resolve_with_env_lookup<F>(settings: &Value, env_lookup: F) -> Self
    where
        F: Fn(&str) -> Option<String>,
    {
        Self {
            base_url: resolve_optional_string(
                settings,
                LINK_GRAPH_JULIA_RERANK_BASE_URL_KEY,
                LINK_GRAPH_JULIA_RERANK_BASE_URL_ENV,
                &env_lookup,
            )
            .map(JuliaContractUrl::from),
            route: resolve_optional_string(
                settings,
                LINK_GRAPH_JULIA_RERANK_ROUTE_KEY,
                LINK_GRAPH_JULIA_RERANK_ROUTE_ENV,
                &env_lookup,
            )
            .map(JuliaContractRoute::from),
            health_route: resolve_optional_string(
                settings,
                LINK_GRAPH_JULIA_RERANK_HEALTH_ROUTE_KEY,
                LINK_GRAPH_JULIA_RERANK_HEALTH_ROUTE_ENV,
                &env_lookup,
            )
            .map(JuliaContractRoute::from),
            schema_version: resolve_optional_string(
                settings,
                LINK_GRAPH_JULIA_RERANK_SCHEMA_VERSION_KEY,
                LINK_GRAPH_JULIA_RERANK_SCHEMA_VERSION_ENV,
                &env_lookup,
            )
            .map(JuliaContractSchemaVersion::from),
            timeout_secs: resolve_optional_string(
                settings,
                LINK_GRAPH_JULIA_RERANK_TIMEOUT_SECS_KEY,
                LINK_GRAPH_JULIA_RERANK_TIMEOUT_SECS_ENV,
                &env_lookup,
            )
            .as_deref()
            .and_then(parse_positive_u64)
            .map(JuliaContractSecondsU64::from),
            service_mode: resolve_optional_string(
                settings,
                LINK_GRAPH_JULIA_RERANK_SERVICE_MODE_KEY,
                LINK_GRAPH_JULIA_RERANK_SERVICE_MODE_ENV,
                &env_lookup,
            )
            .map(JuliaContractMode::from),
            search_config_path: resolve_optional_string(
                settings,
                LINK_GRAPH_JULIA_RERANK_SEARCH_CONFIG_PATH_KEY,
                LINK_GRAPH_JULIA_RERANK_SEARCH_CONFIG_PATH_ENV,
                &env_lookup,
            )
            .map(JuliaContractPath::from),
            vector_weight: resolve_optional_string(
                settings,
                LINK_GRAPH_JULIA_RERANK_VECTOR_WEIGHT_KEY,
                LINK_GRAPH_JULIA_RERANK_VECTOR_WEIGHT_ENV,
                &env_lookup,
            )
            .as_deref()
            .and_then(parse_positive_f64),
            similarity_weight: resolve_optional_string(
                settings,
                LINK_GRAPH_JULIA_RERANK_SIMILARITY_WEIGHT_KEY,
                LINK_GRAPH_JULIA_RERANK_SIMILARITY_WEIGHT_ENV,
                &env_lookup,
            )
            .as_deref()
            .and_then(parse_positive_f64),
        }
    }
}

fn resolve_optional_string<F>(
    settings: &Value,
    dotted_key: &str,
    env_name: &str,
    env_lookup: &F,
) -> Option<String>
where
    F: Fn(&str) -> Option<String>,
{
    first_non_empty(&[
        get_setting_string(settings, dotted_key),
        env_lookup(env_name),
    ])
}

fn get_setting_string(settings: &Value, dotted_key: &str) -> Option<String> {
    get_setting_value(settings, dotted_key).and_then(setting_value_to_string)
}

fn get_setting_value<'a>(settings: &'a Value, dotted_key: &str) -> Option<&'a Value> {
    let mut cursor = settings;
    for segment in dotted_key.split('.') {
        match cursor {
            Value::Mapping(map) => {
                let key = Value::String(segment.to_string());
                cursor = map.get(&key)?;
            }
            _ => return None,
        }
    }
    Some(cursor)
}

fn setting_value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(value) => Some(value.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        _ => None,
    }
}

fn first_non_empty(values: &[Option<String>]) -> Option<String> {
    values.iter().flatten().find_map(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn parse_positive_u64(raw: &str) -> Option<u64> {
    raw.trim().parse::<u64>().ok().filter(|value| *value > 0)
}

fn parse_positive_f64(raw: &str) -> Option<f64> {
    raw.trim()
        .parse::<f64>()
        .ok()
        .filter(|value| value.is_finite() && *value > 0.0)
}
