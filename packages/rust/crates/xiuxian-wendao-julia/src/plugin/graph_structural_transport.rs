use arrow::record_batch::RecordBatch;
use serde_json::Value;
use xiuxian_wendao_core::{
    capabilities::{ContractVersion, PluginCapabilityBinding},
    repo_intelligence::{RegisteredRepository, RepoIntelligenceError, RepositoryPluginConfig},
    transport::{PluginTransportEndpoint, PluginTransportKind},
};
use xiuxian_wendao_runtime::transport::{
    DEFAULT_FLIGHT_BASE_URL, DEFAULT_FLIGHT_TIMEOUT_SECS, FLIGHT_SCHEMA_VERSION_METADATA_KEY,
    NegotiatedFlightTransportClient, negotiate_flight_transport_client_from_bindings,
    normalize_flight_route, validate_flight_schema_version, validate_flight_timeout_secs,
};

use super::capability_manifest::discover_julia_graph_structural_binding_from_manifest_for_repository;
use super::graph_structural::{
    GraphStructuralRouteKind, validate_graph_structural_filter_request_batch,
    validate_graph_structural_filter_response_batch,
    validate_graph_structural_rerank_request_batch,
    validate_graph_structural_rerank_response_batch,
};
use crate::arrow_metadata::attach_record_batch_metadata;
use crate::compatibility::link_graph::julia_graph_structural_provider_selector;

const JULIA_PLUGIN_ID: &str = "julia";
const GRAPH_STRUCTURAL_TRANSPORT_KEY: &str = "graph_structural_transport";
const STRUCTURAL_RERANK_TRANSPORT_KEY: &str = "structural_rerank";
const CONSTRAINT_FILTER_TRANSPORT_KEY: &str = "constraint_filter";
const DEFAULT_JULIA_HEALTH_ROUTE: &str = "/healthz";

/// Build a Julia Flight transport client for one graph-structural route kind.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the repository config contains an
/// invalid graph-structural transport block or cannot be negotiated into a
/// Flight client.
pub fn build_graph_structural_flight_transport_client(
    repository: &RegisteredRepository,
    route_kind: GraphStructuralRouteKind,
) -> Result<Option<NegotiatedFlightTransportClient>, RepoIntelligenceError> {
    let Some(binding) = build_graph_structural_flight_transport_binding(repository, route_kind)?
    else {
        return Ok(None);
    };

    negotiate_flight_transport_client_from_bindings(&[binding]).map_err(|error| {
        RepoIntelligenceError::ConfigLoad {
            message: format!(
                "failed to build Julia graph-structural Flight transport client for repo `{}` and route `{}`: {error}",
                repository.id,
                route_kind.route(),
            ),
        }
    })
}

/// Validate one staged graph-structural request batch set.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any batch violates the staged
/// graph-structural request contract for the selected route kind.
pub fn validate_graph_structural_request_batches(
    route_kind: GraphStructuralRouteKind,
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        validate_graph_structural_request_batch(route_kind, batch)
            .map_err(|error| graph_structural_contract_error(route_kind, "request", error))?;
    }
    Ok(())
}

/// Validate one staged graph-structural response batch set.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any batch violates the staged
/// graph-structural response contract for the selected route kind.
pub fn validate_graph_structural_response_batches(
    route_kind: GraphStructuralRouteKind,
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        validate_graph_structural_response_batch(route_kind, batch)
            .map_err(|error| graph_structural_contract_error(route_kind, "response", error))?;
    }
    Ok(())
}

/// Send graph-structural Arrow batches to one remote Julia Flight transport.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the request violates the staged
/// contract, the Flight roundtrip fails, or the response violates the staged
/// graph-structural response contract.
pub async fn process_graph_structural_flight_batches(
    client: &NegotiatedFlightTransportClient,
    route_kind: GraphStructuralRouteKind,
    batches: &[RecordBatch],
) -> Result<Vec<RecordBatch>, RepoIntelligenceError> {
    validate_graph_structural_request_batches(route_kind, batches)?;
    let request_batches = batches
        .iter()
        .map(|batch| attach_graph_structural_schema_version_metadata(batch, route_kind))
        .collect::<Result<Vec<_>, _>>()?;
    let response_batches = client
        .process_batches(&request_batches)
        .await
        .map_err(|error| RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "Julia graph-structural Flight request for route `{}` failed: {error}",
                route_kind.route()
            ),
        })?;
    validate_graph_structural_response_batches(route_kind, response_batches.as_slice())?;
    Ok(response_batches)
}

/// Resolve the repository's graph-structural Julia Flight transport client,
/// perform the remote Flight roundtrip, and validate the staged response
/// contract.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the repository does not declare a
/// usable graph-structural client, the roundtrip fails, or the response
/// violates the staged contract.
pub async fn process_graph_structural_flight_batches_for_repository(
    repository: &RegisteredRepository,
    route_kind: GraphStructuralRouteKind,
    batches: &[RecordBatch],
) -> Result<Vec<RecordBatch>, RepoIntelligenceError> {
    let client =
        build_graph_structural_flight_transport_client(repository, route_kind)?.ok_or_else(
            || RepoIntelligenceError::AnalysisFailed {
                message: format!(
                    "repo `{}` does not declare an enabled Julia graph-structural Flight transport client for route `{}`",
                    repository.id,
                    route_kind.route()
                ),
            },
        )?;
    process_graph_structural_flight_batches(&client, route_kind, batches).await
}

fn build_graph_structural_flight_transport_binding(
    repository: &RegisteredRepository,
    route_kind: GraphStructuralRouteKind,
) -> Result<Option<PluginCapabilityBinding>, RepoIntelligenceError> {
    let Some(options) = resolve_graph_structural_transport_options(repository, route_kind)? else {
        return discover_julia_graph_structural_binding_from_manifest_for_repository(
            repository, route_kind,
        );
    };
    build_graph_structural_flight_transport_binding_from_options(repository, route_kind, options)
}

fn build_graph_structural_flight_transport_binding_from_options(
    repository: &RegisteredRepository,
    route_kind: GraphStructuralRouteKind,
    options: GraphStructuralTransportOptions,
) -> Result<Option<PluginCapabilityBinding>, RepoIntelligenceError> {
    if let Some(false) = options.enabled {
        return Ok(None);
    }

    let route = match options.route {
        Some(route) => {
            normalize_flight_route(route).map_err(|error| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Julia graph-structural route `{}` is invalid: {error}",
                    repository.id,
                    route_kind.route()
                ),
            })?
        }
        None => route_kind.route().to_string(),
    };
    let health_route = match options.health_route {
        Some(route) => {
            normalize_flight_route(route).map_err(|error| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Julia graph-structural health_route for `{}` is invalid: {error}",
                    repository.id,
                    route_kind.route()
                ),
            })?
        }
        None => DEFAULT_JULIA_HEALTH_ROUTE.to_string(),
    };
    let schema_version = match options.schema_version {
        Some(schema_version) => validate_flight_schema_version(&schema_version).map_err(
            |error| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Julia graph-structural schema version for `{}` is invalid: {error}",
                    repository.id,
                    route_kind.route()
                ),
            },
        )?,
        None => route_kind.schema_version().to_string(),
    };
    let timeout_secs = match options.timeout_secs {
        Some(timeout_secs) => validate_flight_timeout_secs(timeout_secs).map_err(|error| {
            RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Julia graph-structural timeout for `{}` is invalid: {error}",
                    repository.id,
                    route_kind.route()
                ),
            }
        })?,
        None => DEFAULT_FLIGHT_TIMEOUT_SECS,
    };

    Ok(Some(PluginCapabilityBinding {
        selector: julia_graph_structural_provider_selector(),
        endpoint: PluginTransportEndpoint {
            base_url: Some(
                options
                    .base_url
                    .unwrap_or_else(|| DEFAULT_FLIGHT_BASE_URL.to_string()),
            ),
            route: Some(route),
            health_route: Some(health_route),
            timeout_secs: Some(timeout_secs),
        },
        launch: None,
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion(schema_version),
    }))
}

fn attach_graph_structural_schema_version_metadata(
    batch: &RecordBatch,
    route_kind: GraphStructuralRouteKind,
) -> Result<RecordBatch, RepoIntelligenceError> {
    attach_record_batch_metadata(
        batch,
        [(
            FLIGHT_SCHEMA_VERSION_METADATA_KEY,
            route_kind.schema_version(),
        )],
    )
    .map_err(|error| RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "failed to attach Julia graph-structural schema metadata for route `{}`: {error}",
            route_kind.route()
        ),
    })
}

fn validate_graph_structural_request_batch(
    route_kind: GraphStructuralRouteKind,
    batch: &RecordBatch,
) -> Result<(), String> {
    match route_kind {
        GraphStructuralRouteKind::StructuralRerank => {
            validate_graph_structural_rerank_request_batch(batch)
        }
        GraphStructuralRouteKind::ConstraintFilter => {
            validate_graph_structural_filter_request_batch(batch)
        }
    }
}

fn validate_graph_structural_response_batch(
    route_kind: GraphStructuralRouteKind,
    batch: &RecordBatch,
) -> Result<(), String> {
    match route_kind {
        GraphStructuralRouteKind::StructuralRerank => {
            validate_graph_structural_rerank_response_batch(batch)
        }
        GraphStructuralRouteKind::ConstraintFilter => {
            validate_graph_structural_filter_response_batch(batch)
        }
    }
}

fn graph_structural_contract_error(
    route_kind: GraphStructuralRouteKind,
    direction: &str,
    message: impl Into<String>,
) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "Julia graph-structural {direction} contract `{}` for route `{}` violated: {}",
            route_kind.schema_version(),
            route_kind.route(),
            message.into()
        ),
    }
}

#[derive(Debug, Default)]
struct GraphStructuralTransportOptions {
    enabled: Option<bool>,
    base_url: Option<String>,
    route: Option<String>,
    health_route: Option<String>,
    schema_version: Option<String>,
    timeout_secs: Option<u64>,
}

fn resolve_graph_structural_transport_options(
    repository: &RegisteredRepository,
    route_kind: GraphStructuralRouteKind,
) -> Result<Option<GraphStructuralTransportOptions>, RepoIntelligenceError> {
    for plugin in &repository.plugins {
        let RepositoryPluginConfig::Config { id, options } = plugin else {
            continue;
        };
        if id != JULIA_PLUGIN_ID {
            continue;
        }

        let Some(transport) = options.get(GRAPH_STRUCTURAL_TRANSPORT_KEY) else {
            continue;
        };
        let transport = object_option(
            transport,
            GRAPH_STRUCTURAL_TRANSPORT_KEY,
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

        return Ok(Some(GraphStructuralTransportOptions {
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
        }));
    }

    Ok(None)
}

fn object_option<'a>(
    value: &'a Value,
    field: &str,
    route_kind: GraphStructuralRouteKind,
    repository: &RegisteredRepository,
) -> Result<&'a Value, RepoIntelligenceError> {
    if value.is_object() {
        return Ok(value);
    }

    Err(RepoIntelligenceError::ConfigLoad {
        message: format!(
            "repo `{}` Julia graph-structural field `{field}` for route `{}` must be an object",
            repository.id,
            route_kind.route()
        ),
    })
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

impl GraphStructuralRouteKind {
    fn option_key(self) -> &'static str {
        match self {
            Self::StructuralRerank => STRUCTURAL_RERANK_TRANSPORT_KEY,
            Self::ConstraintFilter => CONSTRAINT_FILTER_TRANSPORT_KEY,
        }
    }
}

#[cfg(test)]
#[path = "../../tests/unit/plugin/graph_structural_transport.rs"]
mod tests;
