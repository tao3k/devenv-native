//! Flight transport helpers for Julia repository intelligence analysis.

use arrow::record_batch::RecordBatch;
use serde_json::Value;
use xiuxian_wendao_core::{
    capabilities::{ContractVersion, PluginCapabilityBinding},
    repo_intelligence::{RegisteredRepository, RepoIntelligenceError, RepositoryPluginConfig},
    transport::{PluginTransportEndpoint, PluginTransportKind},
};
use xiuxian_wendao_runtime::transport::{
    DEFAULT_FLIGHT_SCHEMA_VERSION, DEFAULT_FLIGHT_TIMEOUT_SECS, FLIGHT_SCHEMA_VERSION_METADATA_KEY,
    NegotiatedFlightTransportClient, negotiate_flight_transport_client_from_bindings,
    normalize_flight_route, resolve_default_flight_base_url,
    validate_flight_max_in_flight_requests, validate_flight_schema_version,
    validate_flight_timeout_secs, validate_plugin_arrow_response_batches,
};

use crate::arrow_metadata::attach_record_batch_metadata;
use crate::compatibility::link_graph::{
    DEFAULT_JULIA_RERANK_FLIGHT_ROUTE, julia_rerank_provider_selector,
};

const JULIA_PLUGIN_ID: &str = "julia-code-parser";
const FLIGHT_TRANSPORT_KEY: &str = "flight_transport";
const DEFAULT_JULIA_HEALTH_ROUTE: &str = "/healthz";
/// Baseline `WendaoArrow` response contract version enforced by this crate.
pub const JULIA_ARROW_RESPONSE_SCHEMA_VERSION: &str = DEFAULT_FLIGHT_SCHEMA_VERSION;

/// Build a Julia Flight transport client from repository plugin config.
///
/// The function looks for a `RepositoryPluginConfig::Config` entry whose `id`
/// is `julia-code-parser`, and then reads either a nested `flight_transport`
/// object or direct transport keys from that plugin's `options`.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the plugin config uses invalid types
/// or cannot be converted into a valid Arrow Flight transport binding.
pub fn build_julia_flight_transport_client(
    repository: &RegisteredRepository,
) -> Result<Option<NegotiatedFlightTransportClient>, RepoIntelligenceError> {
    let Some(binding) = build_flight_transport_binding(repository)? else {
        return Ok(None);
    };

    negotiate_flight_transport_client_from_bindings(&[binding]).map_err(|error| {
        RepoIntelligenceError::ConfigLoad {
            message: format!(
                "failed to build Julia Flight transport client for repo `{}`: {error}",
                repository.id
            ),
        }
    })
}

/// Send Arrow batches to a remote Julia Flight transport and validate the `v1`
/// response contract before returning the decoded response batches.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the Flight roundtrip fails or the
/// decoded response violates the Julia Arrow response contract.
pub async fn process_julia_flight_batches(
    client: &NegotiatedFlightTransportClient,
    batches: &[RecordBatch],
) -> Result<Vec<RecordBatch>, RepoIntelligenceError> {
    let request_batches = batches
        .iter()
        .map(attach_schema_version_metadata)
        .collect::<Result<Vec<_>, _>>()?;
    let response_batches = client
        .process_batches(&request_batches)
        .await
        .map_err(|error| RepoIntelligenceError::AnalysisFailed {
            message: format!("Julia Arrow Flight request failed: {error}"),
        })?;
    validate_plugin_arrow_response_batches(response_batches.as_slice())?;
    Ok(response_batches)
}

/// Resolve the repository's Julia Arrow Flight transport client, perform the
/// remote Flight roundtrip, and validate the `v1` response contract.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the repository does not declare a
/// usable Julia Arrow Flight transport client, the Flight roundtrip fails, or
/// the decoded response violates the Julia response contract.
pub async fn process_julia_flight_batches_for_repository(
    repository: &RegisteredRepository,
    batches: &[RecordBatch],
) -> Result<Vec<RecordBatch>, RepoIntelligenceError> {
    let client = build_julia_flight_transport_client(repository)?.ok_or_else(|| {
        RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "repo `{}` does not declare an enabled Julia Flight transport client",
                repository.id
            ),
        }
    })?;
    process_julia_flight_batches(&client, batches).await
}

fn build_flight_transport_binding(
    repository: &RegisteredRepository,
) -> Result<Option<PluginCapabilityBinding>, RepoIntelligenceError> {
    let Some(options) = resolve_transport_options(repository)? else {
        return Ok(None);
    };

    if let Some(false) = bool_option(options, "enabled", repository)? {
        return Ok(None);
    }

    let base_url = string_option(options, "base_url", repository)?
        .unwrap_or_else(resolve_default_flight_base_url);
    let route = match string_option(options, "route", repository)? {
        Some(route) => {
            normalize_flight_route(route).map_err(|error| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Julia flight_transport route is invalid: {error}",
                    repository.id
                ),
            })?
        }
        None => DEFAULT_JULIA_RERANK_FLIGHT_ROUTE.to_string(),
    };
    let health_route = match string_option(options, "health_route", repository)? {
        Some(route) => {
            normalize_flight_route(route).map_err(|error| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Julia flight_transport health_route is invalid: {error}",
                    repository.id
                ),
            })?
        }
        None => DEFAULT_JULIA_HEALTH_ROUTE.to_string(),
    };
    let schema_version = match string_option(options, "schema_version", repository)? {
        Some(schema_version) => {
            validate_flight_schema_version(&schema_version).map_err(|error| {
                RepoIntelligenceError::ConfigLoad {
                    message: format!(
                        "repo `{}` Julia flight_transport schema version is invalid: {error}",
                        repository.id
                    ),
                }
            })?
        }
        None => DEFAULT_FLIGHT_SCHEMA_VERSION.to_string(),
    };
    let timeout_secs = match u64_option(options, "timeout_secs", repository)? {
        Some(timeout_secs) => validate_flight_timeout_secs(timeout_secs).map_err(|error| {
            RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Julia flight_transport timeout is invalid: {error}",
                    repository.id
                ),
            }
        })?,
        None => DEFAULT_FLIGHT_TIMEOUT_SECS,
    };
    let max_in_flight_requests = match u64_option(options, "max_in_flight_requests", repository)? {
        Some(max_in_flight_requests) => {
            validate_flight_max_in_flight_requests(max_in_flight_requests).map_err(|error| {
                RepoIntelligenceError::ConfigLoad {
                    message: format!(
                        "repo `{}` Julia flight_transport max_in_flight_requests is invalid: {error}",
                        repository.id
                    ),
                }
            })?;
            Some(max_in_flight_requests)
        }
        None => None,
    };

    Ok(Some(PluginCapabilityBinding {
        selector: julia_rerank_provider_selector(),
        endpoint: PluginTransportEndpoint {
            base_url: Some(base_url),
            route: Some(route),
            health_route: Some(health_route),
            timeout_secs: Some(timeout_secs),
            max_in_flight_requests,
        },
        launch: None,
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion(schema_version),
    }))
}

fn attach_schema_version_metadata(
    batch: &RecordBatch,
) -> Result<RecordBatch, RepoIntelligenceError> {
    attach_record_batch_metadata(
        batch,
        [(
            FLIGHT_SCHEMA_VERSION_METADATA_KEY,
            JULIA_ARROW_RESPONSE_SCHEMA_VERSION,
        )],
    )
    .map_err(|error| RepoIntelligenceError::AnalysisFailed {
        message: format!("failed to attach Julia Arrow Flight schema metadata: {error}"),
    })
}

fn resolve_transport_options(
    repository: &RegisteredRepository,
) -> Result<Option<&Value>, RepoIntelligenceError> {
    for plugin in &repository.plugins {
        let RepositoryPluginConfig::Config { id, options } = plugin else {
            continue;
        };
        if id != JULIA_PLUGIN_ID {
            continue;
        }

        if let Some(transport) = options.get(FLIGHT_TRANSPORT_KEY) {
            return object_option(transport, FLIGHT_TRANSPORT_KEY, repository).map(Some);
        }
        if contains_transport_keys(options) {
            return object_option(options, "options", repository).map(Some);
        }
    }
    Ok(None)
}

fn contains_transport_keys(value: &Value) -> bool {
    let Some(object) = value.as_object() else {
        return false;
    };
    [
        "enabled",
        "base_url",
        "route",
        "health_route",
        "schema_version",
        "timeout_secs",
        "max_in_flight_requests",
    ]
    .iter()
    .any(|key| object.contains_key(*key))
}

fn object_option<'a>(
    value: &'a Value,
    field: &str,
    repository: &RegisteredRepository,
) -> Result<&'a Value, RepoIntelligenceError> {
    if value.is_object() {
        return Ok(value);
    }

    Err(plugin_config_type_error(repository, field, "an object"))
}

fn string_option(
    value: &Value,
    field: &str,
    repository: &RegisteredRepository,
) -> Result<Option<String>, RepoIntelligenceError> {
    let Some(raw) = value.get(field) else {
        return Ok(None);
    };
    let Some(string) = raw.as_str() else {
        return Err(plugin_config_type_error(repository, field, "a string"));
    };
    Ok(Some(string.to_string()))
}

fn bool_option(
    value: &Value,
    field: &str,
    repository: &RegisteredRepository,
) -> Result<Option<bool>, RepoIntelligenceError> {
    let Some(raw) = value.get(field) else {
        return Ok(None);
    };
    let Some(boolean) = raw.as_bool() else {
        return Err(plugin_config_type_error(repository, field, "a boolean"));
    };
    Ok(Some(boolean))
}

fn u64_option(
    value: &Value,
    field: &str,
    repository: &RegisteredRepository,
) -> Result<Option<u64>, RepoIntelligenceError> {
    let Some(raw) = value.get(field) else {
        return Ok(None);
    };
    let Some(number) = raw.as_u64() else {
        return Err(plugin_config_type_error(
            repository,
            field,
            "an unsigned integer",
        ));
    };
    Ok(Some(number))
}

fn plugin_config_type_error(
    repository: &RegisteredRepository,
    field: &str,
    expected: &str,
) -> RepoIntelligenceError {
    RepoIntelligenceError::ConfigLoad {
        message: format!(
            "repo `{}` Julia plugin field `{field}` must be {expected}",
            repository.id
        ),
    }
}

#[cfg(test)]
#[path = "../../tests/unit/plugin/transport.rs"]
mod tests;
