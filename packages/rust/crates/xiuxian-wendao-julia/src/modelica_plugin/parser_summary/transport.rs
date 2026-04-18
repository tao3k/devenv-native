use std::collections::HashMap;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Mutex, OnceLock};

use serde_json::Value;
use xiuxian_wendao_core::{
    capabilities::{ContractVersion, PluginCapabilityBinding, PluginProviderSelector},
    ids::{CapabilityId, PluginId},
    repo_intelligence::{RegisteredRepository, RepoIntelligenceError, RepositoryPluginConfig},
    transport::{PluginTransportEndpoint, PluginTransportKind},
};
use xiuxian_wendao_runtime::transport::{
    DEFAULT_FLIGHT_MAX_IN_FLIGHT_REQUESTS, FLIGHT_SCHEMA_VERSION_METADATA_KEY,
    NegotiatedFlightTransportClient, negotiate_flight_transport_client_from_bindings,
    normalize_flight_route, resolve_default_flight_base_url,
    validate_flight_max_in_flight_requests, validate_flight_schema_version,
    validate_flight_timeout_secs,
};

use arrow::record_batch::RecordBatch;

use super::contract::{
    validate_modelica_parser_summary_request_batches,
    validate_modelica_parser_summary_response_batches,
};
use crate::arrow_metadata::attach_record_batch_metadata;

const MODELICA_PLUGIN_ID: &str = "modelica";
const MODELICA_PARSER_SUMMARY_CAPABILITY_ID: &str = "parser-summary";
const PARSER_SUMMARY_TRANSPORT_KEY: &str = "parser_summary_transport";
const FILE_SUMMARY_TRANSPORT_KEY: &str = "file_summary";
const DEFAULT_MODELICA_HEALTH_ROUTE: &str = "/healthz";
const DEFAULT_WENDAOSEARCH_PARSER_SUMMARY_BASE_URL_FALLBACK: &str = "http://127.0.0.1:41081";
const PARSER_SUMMARY_BASE_URL_ENV: &str = "WENDAO_PARSER_SUMMARY_BASE_URL";
const DEFAULT_PARSER_SUMMARY_TIMEOUT_SECS: u64 = 120;

fn resolve_parser_summary_base_url() -> String {
    std::env::var(PARSER_SUMMARY_BASE_URL_ENV)
        .ok()
        .filter(|v| !v.trim().is_empty())
        .unwrap_or_else(|| DEFAULT_WENDAOSEARCH_PARSER_SUMMARY_BASE_URL_FALLBACK.to_string())
}

pub(crate) const MODELICA_PARSER_SUMMARY_SCHEMA_VERSION: &str = "v3";
pub(crate) const MODELICA_FILE_SUMMARY_ROUTE: &str = "/wendao/code-parser/modelica/file-summary";
pub(crate) const MODELICA_AST_QUERY_ROUTE: &str = "/wendao/code-parser/modelica/ast-query";

static LINKED_MODELICA_PARSER_SUMMARY_BASE_URL: OnceLock<String> = OnceLock::new();
static MODELICA_PARSER_SUMMARY_CLIENT_CACHE: OnceLock<
    Mutex<HashMap<ParserSummaryTransportCacheKey, CachedParserSummaryFlightClient>>,
> = OnceLock::new();
#[cfg(test)]
static NEXT_MODELICA_PARSER_SUMMARY_CLIENT_SLOT: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ParserSummaryRouteKind {
    FileSummary,
    AstQuery,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ParserSummaryTransportCacheKey {
    base_url: String,
    route: String,
    schema_version: String,
    timeout_secs: u64,
    max_in_flight_requests: u64,
}

#[derive(Clone)]
struct CachedParserSummaryFlightClient {
    #[cfg(test)]
    slot_id: usize,
    client: NegotiatedFlightTransportClient,
}

impl ParserSummaryRouteKind {
    pub(crate) fn option_key(self) -> &'static str {
        match self {
            Self::FileSummary => FILE_SUMMARY_TRANSPORT_KEY,
            Self::AstQuery => "ast_query",
        }
    }

    pub(crate) fn route(self) -> &'static str {
        match self {
            Self::FileSummary => MODELICA_FILE_SUMMARY_ROUTE,
            Self::AstQuery => MODELICA_AST_QUERY_ROUTE,
        }
    }
}

/// Register a process-local Modelica parser-summary base URL for linked test
/// hosts that run without explicit repo transport config.
///
/// # Errors
///
/// Returns an error when the process has already been configured with a
/// different base URL.
pub fn set_linked_modelica_parser_summary_base_url_for_tests(
    base_url: impl Into<String>,
) -> Result<(), String> {
    let base_url = base_url.into();
    if base_url.trim().is_empty() {
        return Err("linked Modelica parser-summary base_url must not be blank".to_string());
    }
    if let Some(existing) = LINKED_MODELICA_PARSER_SUMMARY_BASE_URL.get() {
        return if existing == &base_url {
            Ok(())
        } else {
            Err(format!(
                "linked Modelica parser-summary base_url already configured as `{existing}`, cannot replace with `{base_url}`",
            ))
        };
    }

    LINKED_MODELICA_PARSER_SUMMARY_BASE_URL
        .set(base_url)
        .map_err(|_| "failed to store linked Modelica parser-summary base_url".to_string())
}

/// Clear the process-local Modelica parser-summary Flight client cache used by
/// linked test hosts.
pub fn clear_modelica_parser_summary_transport_cache_for_tests() {
    modelica_parser_summary_client_cache()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clear();
}

pub(crate) fn build_modelica_parser_summary_flight_transport_client(
    repository: &RegisteredRepository,
    route_kind: ParserSummaryRouteKind,
) -> Result<NegotiatedFlightTransportClient, RepoIntelligenceError> {
    let binding = build_parser_summary_flight_transport_binding(repository, route_kind)?;
    let cache_key = parser_summary_transport_cache_key(&binding, repository, route_kind)?;
    {
        let cache = modelica_parser_summary_client_cache()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(cached) = cache.get(&cache_key) {
            return Ok(cached.client.clone());
        }
    }
    let negotiated = negotiate_flight_transport_client_from_bindings(&[binding]).map_err(
        |error| RepoIntelligenceError::ConfigLoad {
            message: format!(
                "failed to build Modelica parser-summary Flight transport client for repo `{}` and route `{}`: {error}",
                repository.id,
                route_kind.route(),
            ),
        },
    )?;
    let client =
        negotiated.ok_or_else(|| missing_parser_summary_transport_error(repository, route_kind))?;
    let mut cache = modelica_parser_summary_client_cache()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let cached = cache
        .entry(cache_key)
        .or_insert_with(|| CachedParserSummaryFlightClient {
            #[cfg(test)]
            slot_id: NEXT_MODELICA_PARSER_SUMMARY_CLIENT_SLOT.fetch_add(1, Ordering::Relaxed),
            client: client.clone(),
        });
    Ok(cached.client.clone())
}

pub(crate) async fn process_modelica_parser_summary_flight_batches(
    client: &NegotiatedFlightTransportClient,
    route_kind: ParserSummaryRouteKind,
    batches: &[RecordBatch],
) -> Result<Vec<RecordBatch>, RepoIntelligenceError> {
    validate_modelica_parser_summary_request_batches(batches)?;
    let request_batches = batches
        .iter()
        .map(attach_parser_summary_schema_version_metadata)
        .collect::<Result<Vec<_>, _>>()?;
    let response_batches = client
        .process_batches(&request_batches)
        .await
        .map_err(|error| RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "Modelica parser-summary Flight request for route `{}` failed: {error}",
                route_kind.route(),
            ),
        })?;
    validate_modelica_parser_summary_response_batches(response_batches.as_slice())?;
    Ok(response_batches)
}

pub(crate) async fn process_modelica_parser_summary_flight_batches_for_repository(
    repository: &RegisteredRepository,
    route_kind: ParserSummaryRouteKind,
    batches: &[RecordBatch],
) -> Result<Vec<RecordBatch>, RepoIntelligenceError> {
    let client = build_modelica_parser_summary_flight_transport_client(repository, route_kind)?;
    match process_modelica_parser_summary_flight_batches(&client, route_kind, batches).await {
        Ok(response_batches) => Ok(response_batches),
        Err(error) if parser_summary_transport_error_requires_client_refresh(&error) => {
            evict_modelica_parser_summary_cached_client(repository, route_kind)?;
            let refreshed_client =
                build_modelica_parser_summary_flight_transport_client(repository, route_kind)?;
            process_modelica_parser_summary_flight_batches(&refreshed_client, route_kind, batches)
                .await
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn modelica_parser_summary_timeout_secs_for_repository(
    repository: &RegisteredRepository,
    route_kind: ParserSummaryRouteKind,
) -> Result<u64, RepoIntelligenceError> {
    let binding = build_parser_summary_flight_transport_binding(repository, route_kind)?;
    Ok(binding
        .endpoint
        .timeout_secs
        .unwrap_or(DEFAULT_PARSER_SUMMARY_TIMEOUT_SECS))
}

fn modelica_parser_summary_provider_selector() -> PluginProviderSelector {
    PluginProviderSelector {
        capability_id: CapabilityId(MODELICA_PARSER_SUMMARY_CAPABILITY_ID.to_string()),
        provider: PluginId(MODELICA_PLUGIN_ID.to_string()),
    }
}

fn modelica_parser_summary_client_cache()
-> &'static Mutex<HashMap<ParserSummaryTransportCacheKey, CachedParserSummaryFlightClient>> {
    MODELICA_PARSER_SUMMARY_CLIENT_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn evict_modelica_parser_summary_cached_client(
    repository: &RegisteredRepository,
    route_kind: ParserSummaryRouteKind,
) -> Result<(), RepoIntelligenceError> {
    let binding = build_parser_summary_flight_transport_binding(repository, route_kind)?;
    let cache_key = parser_summary_transport_cache_key(&binding, repository, route_kind)?;
    modelica_parser_summary_client_cache()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .remove(&cache_key);
    Ok(())
}

fn parser_summary_transport_error_requires_client_refresh(error: &RepoIntelligenceError) -> bool {
    matches!(
        error,
        RepoIntelligenceError::AnalysisFailed { message }
            if message.contains("Service was not ready: transport error")
    )
}

fn parser_summary_transport_cache_key(
    binding: &PluginCapabilityBinding,
    repository: &RegisteredRepository,
    route_kind: ParserSummaryRouteKind,
) -> Result<ParserSummaryTransportCacheKey, RepoIntelligenceError> {
    let base_url = binding
        .endpoint
        .base_url
        .clone()
        .ok_or_else(|| missing_parser_summary_transport_error(repository, route_kind))?;
    let route = binding
        .endpoint
        .route
        .clone()
        .ok_or_else(|| missing_parser_summary_transport_error(repository, route_kind))?;
    Ok(ParserSummaryTransportCacheKey {
        base_url,
        route,
        schema_version: binding.contract_version.0.clone(),
        timeout_secs: binding
            .endpoint
            .timeout_secs
            .unwrap_or(DEFAULT_PARSER_SUMMARY_TIMEOUT_SECS),
        max_in_flight_requests: binding
            .endpoint
            .max_in_flight_requests
            .unwrap_or(DEFAULT_FLIGHT_MAX_IN_FLIGHT_REQUESTS as u64),
    })
}

#[cfg(test)]
fn modelica_parser_summary_transport_cache_len_for_tests() -> usize {
    modelica_parser_summary_client_cache()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .len()
}

#[cfg(test)]
fn modelica_parser_summary_transport_slot_id_for_tests(
    repository: &RegisteredRepository,
    route_kind: ParserSummaryRouteKind,
) -> Result<usize, RepoIntelligenceError> {
    let binding = build_parser_summary_flight_transport_binding(repository, route_kind)?;
    let cache_key = parser_summary_transport_cache_key(&binding, repository, route_kind)?;
    modelica_parser_summary_client_cache()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .get(&cache_key)
        .map(|cached| cached.slot_id)
        .ok_or_else(|| RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "Modelica parser-summary cached client missing for repo `{}` and route `{}`",
                repository.id,
                route_kind.route(),
            ),
        })
}

fn build_parser_summary_flight_transport_binding(
    repository: &RegisteredRepository,
    route_kind: ParserSummaryRouteKind,
) -> Result<PluginCapabilityBinding, RepoIntelligenceError> {
    let options = resolve_parser_summary_transport_options(repository, route_kind)?
        .ok_or_else(|| missing_parser_summary_transport_error(repository, route_kind))?;
    if let Some(false) = options.enabled {
        return Err(missing_parser_summary_transport_error(
            repository, route_kind,
        ));
    }

    let route = match options.route {
        Some(route) => {
            normalize_flight_route(route).map_err(|error| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Modelica parser-summary route `{}` is invalid: {error}",
                    repository.id,
                    route_kind.route(),
                ),
            })?
        }
        None => route_kind.route().to_string(),
    };
    let health_route = match options.health_route {
        Some(route) => {
            normalize_flight_route(route).map_err(|error| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Modelica parser-summary health_route for `{}` is invalid: {error}",
                    repository.id,
                    route_kind.route(),
                ),
            })?
        }
        None => DEFAULT_MODELICA_HEALTH_ROUTE.to_string(),
    };
    let schema_version = match options.schema_version {
        Some(schema_version) => validate_flight_schema_version(&schema_version).map_err(
            |error| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Modelica parser-summary schema version for `{}` is invalid: {error}",
                    repository.id,
                    route_kind.route(),
                ),
            },
        )?,
        None => MODELICA_PARSER_SUMMARY_SCHEMA_VERSION.to_string(),
    };
    let timeout_secs = match options.timeout_secs {
        Some(timeout_secs) => validate_flight_timeout_secs(timeout_secs).map_err(|error| {
            RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Modelica parser-summary timeout for `{}` is invalid: {error}",
                    repository.id,
                    route_kind.route(),
                ),
            }
        })?,
        None => DEFAULT_PARSER_SUMMARY_TIMEOUT_SECS,
    };
    let max_in_flight_requests = match options.max_in_flight_requests {
        Some(max_in_flight_requests) => {
            validate_flight_max_in_flight_requests(max_in_flight_requests).map_err(|error| {
                RepoIntelligenceError::ConfigLoad {
                    message: format!(
                        "repo `{}` Modelica parser-summary max_in_flight_requests for `{}` is invalid: {error}",
                        repository.id,
                        route_kind.route(),
                    ),
                }
            })?;
            Some(max_in_flight_requests)
        }
        None => None,
    };

    Ok(PluginCapabilityBinding {
        selector: modelica_parser_summary_provider_selector(),
        endpoint: PluginTransportEndpoint {
            base_url: Some(
                options
                    .base_url
                    .unwrap_or_else(resolve_default_flight_base_url),
            ),
            route: Some(route),
            health_route: Some(health_route),
            timeout_secs: Some(timeout_secs),
            max_in_flight_requests,
        },
        launch: None,
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion(schema_version),
    })
}

fn attach_parser_summary_schema_version_metadata(
    batch: &RecordBatch,
) -> Result<RecordBatch, RepoIntelligenceError> {
    attach_record_batch_metadata(
        batch,
        [(
            FLIGHT_SCHEMA_VERSION_METADATA_KEY,
            MODELICA_PARSER_SUMMARY_SCHEMA_VERSION,
        )],
    )
    .map_err(|error| RepoIntelligenceError::AnalysisFailed {
        message: format!("failed to attach Modelica parser-summary schema metadata: {error}"),
    })
}

#[derive(Debug, Clone, Default)]
struct ParserSummaryTransportOptions {
    enabled: Option<bool>,
    base_url: Option<String>,
    route: Option<String>,
    health_route: Option<String>,
    schema_version: Option<String>,
    timeout_secs: Option<u64>,
    max_in_flight_requests: Option<u64>,
}

fn missing_parser_summary_transport_error(
    repository: &RegisteredRepository,
    route_kind: ParserSummaryRouteKind,
) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "repo `{}` requires an enabled Modelica parser-summary Flight transport client for route `{}`",
            repository.id,
            route_kind.route(),
        ),
    }
}

fn resolve_parser_summary_transport_options(
    repository: &RegisteredRepository,
    route_kind: ParserSummaryRouteKind,
) -> Result<Option<ParserSummaryTransportOptions>, RepoIntelligenceError> {
    let mut saw_modelica_plugin = false;

    for plugin in &repository.plugins {
        if plugin.id() == MODELICA_PLUGIN_ID {
            saw_modelica_plugin = true;
        }
        let RepositoryPluginConfig::Config { id, options } = plugin else {
            continue;
        };
        if id != MODELICA_PLUGIN_ID {
            continue;
        }

        let Some(transport) = options.get(PARSER_SUMMARY_TRANSPORT_KEY) else {
            continue;
        };
        let transport = object_option(
            transport,
            PARSER_SUMMARY_TRANSPORT_KEY,
            route_kind,
            repository,
        )?;
        let route_override = transport.get(route_kind.option_key());
        let route_override = match route_override {
            Some(value) => Some(object_option(
                value,
                route_kind.option_key(),
                route_kind,
                repository,
            )?),
            None => None,
        };

        return Ok(Some(ParserSummaryTransportOptions {
            enabled: route_override
                .map(|value| bool_option(value, "enabled", repository))
                .transpose()?
                .flatten()
                .or(bool_option(transport, "enabled", repository)?),
            base_url: route_override
                .map(|value| string_option(value, "base_url", repository))
                .transpose()?
                .flatten()
                .or(string_option(transport, "base_url", repository)?),
            route: route_override
                .map(|value| string_option(value, "route", repository))
                .transpose()?
                .flatten()
                .or(string_option(transport, "route", repository)?),
            health_route: route_override
                .map(|value| string_option(value, "health_route", repository))
                .transpose()?
                .flatten()
                .or(string_option(transport, "health_route", repository)?),
            schema_version: route_override
                .map(|value| string_option(value, "schema_version", repository))
                .transpose()?
                .flatten()
                .or(string_option(transport, "schema_version", repository)?),
            timeout_secs: route_override
                .map(|value| u64_option(value, "timeout_secs", repository))
                .transpose()?
                .flatten()
                .or(u64_option(transport, "timeout_secs", repository)?),
            max_in_flight_requests: route_override
                .map(|value| u64_option(value, "max_in_flight_requests", repository))
                .transpose()?
                .flatten()
                .or(u64_option(transport, "max_in_flight_requests", repository)?),
        }));
    }

    if let Some(base_url) = LINKED_MODELICA_PARSER_SUMMARY_BASE_URL.get() {
        return Ok(Some(ParserSummaryTransportOptions {
            enabled: Some(true),
            base_url: Some(base_url.clone()),
            route: None,
            health_route: None,
            schema_version: Some(MODELICA_PARSER_SUMMARY_SCHEMA_VERSION.to_string()),
            timeout_secs: None,
            max_in_flight_requests: None,
        }));
    }

    if saw_modelica_plugin {
        return Ok(Some(ParserSummaryTransportOptions {
            enabled: Some(true),
            base_url: Some(resolve_parser_summary_base_url()),
            route: None,
            health_route: None,
            schema_version: Some(MODELICA_PARSER_SUMMARY_SCHEMA_VERSION.to_string()),
            timeout_secs: None,
            max_in_flight_requests: None,
        }));
    }

    Ok(None)
}

fn object_option<'a>(
    value: &'a Value,
    field: &str,
    route_kind: ParserSummaryRouteKind,
    repository: &RegisteredRepository,
) -> Result<&'a Value, RepoIntelligenceError> {
    if value.is_object() {
        return Ok(value);
    }
    Err(RepoIntelligenceError::ConfigLoad {
        message: format!(
            "repo `{}` Modelica parser-summary field `{field}` for route `{}` must be a table",
            repository.id,
            route_kind.route(),
        ),
    })
}

fn string_option(
    options: &Value,
    field: &str,
    repository: &RegisteredRepository,
) -> Result<Option<String>, RepoIntelligenceError> {
    match options.get(field) {
        None => Ok(None),
        Some(value) => value
            .as_str()
            .map(|value| Some(value.to_string()))
            .ok_or_else(|| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Modelica plugin field `{field}` must be a string",
                    repository.id,
                ),
            }),
    }
}

fn bool_option(
    options: &Value,
    field: &str,
    repository: &RegisteredRepository,
) -> Result<Option<bool>, RepoIntelligenceError> {
    match options.get(field) {
        None => Ok(None),
        Some(value) => value
            .as_bool()
            .map(Some)
            .ok_or_else(|| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Modelica plugin field `{field}` must be a boolean",
                    repository.id,
                ),
            }),
    }
}

fn u64_option(
    options: &Value,
    field: &str,
    repository: &RegisteredRepository,
) -> Result<Option<u64>, RepoIntelligenceError> {
    match options.get(field) {
        None => Ok(None),
        Some(value) => value
            .as_u64()
            .map(Some)
            .ok_or_else(|| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Modelica plugin field `{field}` must be an unsigned integer",
                    repository.id,
                ),
            }),
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/modelica_plugin/parser_summary_transport.rs"]
mod tests;
