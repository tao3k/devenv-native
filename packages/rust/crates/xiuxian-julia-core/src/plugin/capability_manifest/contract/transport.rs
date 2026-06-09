//! Flight transport and repository preflight for Julia capability manifests.

use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::{
    capabilities::{ContractVersion, PluginCapabilityBinding},
    repo_intelligence::{RegisteredRepository, RepoIntelligenceError, RepositoryPluginConfig},
    transport::{PluginTransportEndpoint, PluginTransportKind},
};
use xiuxian_wendao_runtime::transport::{
    DEFAULT_FLIGHT_TIMEOUT_SECS, FLIGHT_SCHEMA_VERSION_METADATA_KEY,
    NegotiatedFlightTransportClient, negotiate_flight_transport_client_from_bindings,
    normalize_flight_route, resolve_default_flight_base_url, validate_flight_schema_version,
    validate_flight_timeout_secs,
};

use crate::arrow_metadata::attach_record_batch_metadata;
use crate::compatibility::link_graph::{
    JULIA_CAPABILITY_MANIFEST_CAPABILITY_ID, JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID,
    julia_capability_manifest_provider_selector,
};
use crate::plugin::capability_manifest::{
    CAPABILITY_MANIFEST_TRANSPORT_KEY, DEFAULT_JULIA_HEALTH_ROUTE,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE, JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION,
    JULIA_PLUGIN_CONFIG_ID, JuliaPluginCapabilityManifestRequestRow,
    JuliaPluginCapabilityManifestRow,
};
use crate::plugin::graph_structural::GraphStructuralRouteKind;

use super::batch::{
    build_julia_plugin_capability_manifest_request_batch,
    decode_julia_plugin_capability_manifest_rows,
    normalize_julia_plugin_capability_manifest_response_batches,
    validate_julia_plugin_capability_manifest_request_batches,
    validate_julia_plugin_capability_manifest_response_batches,
};
use super::support::{
    bool_option, object_option, panic_payload_message, string_option, u64_option,
};

/// Build a Julia capability-manifest Flight transport client from repository config.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the repository config contains an
/// invalid manifest transport block or cannot be negotiated into a Flight
/// client.
pub fn build_julia_capability_manifest_flight_transport_client(
    repository: &RegisteredRepository,
) -> Result<Option<NegotiatedFlightTransportClient>, RepoIntelligenceError> {
    let Some(binding) = build_capability_manifest_transport_binding(repository)? else {
        return Ok(None);
    };

    negotiate_flight_transport_client_from_bindings(&[binding]).map_err(|error| {
        RepoIntelligenceError::ConfigLoad {
            message: format!(
                "failed to build Julia capability-manifest Flight transport client for repo `{}`: {error}",
                repository.id
            ),
        }
    })
}

/// Send capability-manifest batches through one negotiated Flight client.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the request violates the staged
/// contract, the Flight roundtrip fails, or the response violates the staged
/// response contract.
pub async fn process_julia_capability_manifest_flight_batches(
    client: &NegotiatedFlightTransportClient,
    batches: &[RecordBatch],
) -> Result<Vec<RecordBatch>, RepoIntelligenceError> {
    validate_julia_plugin_capability_manifest_request_batches(batches)?;
    let request_batches = batches
        .iter()
        .map(attach_capability_manifest_schema_version_metadata)
        .collect::<Result<Vec<_>, _>>()?;
    let response_batches = client
        .process_batches(&request_batches)
        .await
        .map_err(|error| RepoIntelligenceError::AnalysisFailed {
            message: format!("Julia capability-manifest Flight request failed: {error}"),
        })?;
    let response_batches =
        normalize_julia_plugin_capability_manifest_response_batches(response_batches.as_slice())?;
    validate_julia_plugin_capability_manifest_response_batches(response_batches.as_slice())?;
    Ok(response_batches)
}

/// Resolve the repository-configured capability-manifest client and roundtrip one request.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the repository does not declare a
/// usable manifest client, the roundtrip fails, or the response violates the
/// staged response contract.
pub async fn process_julia_capability_manifest_flight_batches_for_repository(
    repository: &RegisteredRepository,
    batches: &[RecordBatch],
) -> Result<Vec<RecordBatch>, RepoIntelligenceError> {
    let client = build_julia_capability_manifest_flight_transport_client(repository)?
        .ok_or_else(|| RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "repo `{}` does not declare an enabled Julia capability-manifest Flight transport client",
                repository.id
            ),
        })?;
    process_julia_capability_manifest_flight_batches(&client, batches).await
}

/// Build one manifest request batch, execute the configured Flight roundtrip,
/// and decode the manifest rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the request cannot be materialized,
/// the repository does not declare a usable manifest client, the remote
/// roundtrip fails, or the response violates the staged response contract.
pub async fn fetch_julia_plugin_capability_manifest_rows_for_repository(
    repository: &RegisteredRepository,
    rows: &[JuliaPluginCapabilityManifestRequestRow],
) -> Result<Vec<JuliaPluginCapabilityManifestRow>, RepoIntelligenceError> {
    let batch = build_julia_plugin_capability_manifest_request_batch(rows)?;
    let response_batches =
        process_julia_capability_manifest_flight_batches_for_repository(repository, &[batch])
            .await?;
    decode_julia_plugin_capability_manifest_rows(response_batches.as_slice())
}

pub(crate) fn validate_julia_capability_manifest_preflight_for_repository(
    repository: &RegisteredRepository,
) -> Result<Option<Vec<JuliaPluginCapabilityManifestRow>>, RepoIntelligenceError> {
    if build_julia_capability_manifest_flight_transport_client(repository)?.is_none() {
        return Ok(None);
    }

    let selector = julia_capability_manifest_provider_selector();
    let expected_plugin_id = selector.provider.0;
    let rows = fetch_julia_plugin_capability_manifest_rows_blocking_for_repository(
        repository,
        &[JuliaPluginCapabilityManifestRequestRow {
            plugin_id: expected_plugin_id.clone().into(),
            repository_id: repository.id.clone().into(),
            capability_filter: None,
            include_disabled: true.into(),
        }],
    )?;

    validate_capability_manifest_preflight_rows(repository, &expected_plugin_id, &rows)?;
    Ok(Some(rows))
}

pub(crate) fn graph_structural_binding_from_capability_manifest_rows(
    rows: &[JuliaPluginCapabilityManifestRow],
    route_kind: GraphStructuralRouteKind,
) -> Result<Option<PluginCapabilityBinding>, RepoIntelligenceError> {
    let mut matching_rows = rows.iter().filter(|row| {
        row.capability_id.as_str() == JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID
            && row.capability_variant.as_deref() == Some(route_kind.capability_variant())
    });
    let Some(row) = matching_rows.next() else {
        return Ok(None);
    };
    if matching_rows.next().is_some() {
        return Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "Julia capability-manifest returned multiple graph-structural rows for variant `{}`",
                route_kind.capability_variant()
            ),
        });
    }
    row.to_binding()
}

pub(crate) fn discover_julia_graph_structural_binding_from_manifest_for_repository(
    repository: &RegisteredRepository,
    route_kind: GraphStructuralRouteKind,
) -> Result<Option<PluginCapabilityBinding>, RepoIntelligenceError> {
    let Some(rows) = validate_julia_capability_manifest_preflight_for_repository(repository)?
    else {
        return Ok(None);
    };
    graph_structural_binding_from_capability_manifest_rows(rows.as_slice(), route_kind)
}

fn fetch_julia_plugin_capability_manifest_rows_blocking_for_repository(
    repository: &RegisteredRepository,
    rows: &[JuliaPluginCapabilityManifestRequestRow],
) -> Result<Vec<JuliaPluginCapabilityManifestRow>, RepoIntelligenceError> {
    let repository = repository.clone();
    let rows = rows.to_vec();
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|error| RepoIntelligenceError::AnalysisFailed {
                message: format!(
                    "failed to build Julia capability-manifest preflight runtime for repo `{}`: {error}",
                    repository.id
                ),
            })?;
        runtime.block_on(fetch_julia_plugin_capability_manifest_rows_for_repository(
            &repository,
            &rows,
        ))
    })
    .join()
    .map_err(|panic_payload| RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "Julia capability-manifest preflight thread panicked: {}",
            panic_payload_message(&panic_payload)
        ),
    })?
}

fn validate_capability_manifest_preflight_rows(
    repository: &RegisteredRepository,
    expected_plugin_id: &str,
    rows: &[JuliaPluginCapabilityManifestRow],
) -> Result<(), RepoIntelligenceError> {
    if rows.is_empty() {
        return Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "repo `{}` Julia capability-manifest preflight returned no rows",
                repository.id
            ),
        });
    }

    if let Some(row) = rows
        .iter()
        .find(|row| row.plugin_id.as_str() != expected_plugin_id)
    {
        return Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "repo `{}` Julia capability-manifest preflight returned provider `{}` but expected `{}`",
                repository.id, row.plugin_id, expected_plugin_id
            ),
        });
    }

    if rows
        .iter()
        .any(|row| row.capability_id.as_str() == JULIA_CAPABILITY_MANIFEST_CAPABILITY_ID)
    {
        return Ok(());
    }

    Err(RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "repo `{}` Julia capability-manifest preflight did not advertise capability `{}`",
            repository.id, JULIA_CAPABILITY_MANIFEST_CAPABILITY_ID
        ),
    })
}

fn build_capability_manifest_transport_binding(
    repository: &RegisteredRepository,
) -> Result<Option<PluginCapabilityBinding>, RepoIntelligenceError> {
    let Some(options) = resolve_capability_manifest_transport_options(repository)? else {
        return Ok(None);
    };

    if let Some(false) = options.enabled {
        return Ok(None);
    }

    Ok(Some(PluginCapabilityBinding {
        selector: julia_capability_manifest_provider_selector(),
        endpoint: PluginTransportEndpoint {
            base_url: Some(
                options
                    .base_url
                    .unwrap_or_else(resolve_default_flight_base_url),
            ),
            route: Some(resolve_capability_manifest_route(
                repository,
                options.route,
            )?),
            health_route: Some(resolve_capability_manifest_health_route(
                repository,
                options.health_route,
            )?),
            timeout_secs: Some(resolve_capability_manifest_timeout_secs(
                repository,
                options.timeout_secs,
            )?),
            max_in_flight_requests: None,
        },
        launch: None,
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion(resolve_capability_manifest_schema_version(
            repository,
            options.schema_version,
        )?),
    }))
}

fn attach_capability_manifest_schema_version_metadata(
    batch: &RecordBatch,
) -> Result<RecordBatch, RepoIntelligenceError> {
    attach_record_batch_metadata(
        batch,
        [(
            FLIGHT_SCHEMA_VERSION_METADATA_KEY,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION,
        )],
    )
    .map_err(|error| RepoIntelligenceError::AnalysisFailed {
        message: format!("failed to attach Julia capability-manifest schema metadata: {error}"),
    })
}

fn resolve_capability_manifest_transport_options(
    repository: &RegisteredRepository,
) -> Result<Option<CapabilityManifestTransportOptions>, RepoIntelligenceError> {
    for plugin in &repository.plugins {
        let RepositoryPluginConfig::Config { id, options } = plugin else {
            continue;
        };
        if id != JULIA_PLUGIN_CONFIG_ID {
            continue;
        }

        let Some(transport) = options.get(CAPABILITY_MANIFEST_TRANSPORT_KEY) else {
            continue;
        };
        let transport = object_option(transport, CAPABILITY_MANIFEST_TRANSPORT_KEY, repository)?;
        return Ok(Some(CapabilityManifestTransportOptions {
            enabled: bool_option(transport, "enabled", repository)?,
            base_url: string_option(transport, "base_url", repository)?,
            route: string_option(transport, "route", repository)?,
            health_route: string_option(transport, "health_route", repository)?,
            schema_version: string_option(transport, "schema_version", repository)?,
            timeout_secs: u64_option(transport, "timeout_secs", repository)?,
        }));
    }

    Ok(None)
}

#[derive(Debug, Default)]
struct CapabilityManifestTransportOptions {
    enabled: Option<bool>,
    base_url: Option<String>,
    route: Option<String>,
    health_route: Option<String>,
    schema_version: Option<String>,
    timeout_secs: Option<u64>,
}

fn resolve_capability_manifest_route(
    repository: &RegisteredRepository,
    route: Option<String>,
) -> Result<String, RepoIntelligenceError> {
    match route {
        Some(route) => {
            normalize_flight_route(route).map_err(|error| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Julia capability-manifest route is invalid: {error}",
                    repository.id
                ),
            })
        }
        None => Ok(JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE.to_string()),
    }
}

fn resolve_capability_manifest_health_route(
    repository: &RegisteredRepository,
    health_route: Option<String>,
) -> Result<String, RepoIntelligenceError> {
    match health_route {
        Some(route) => {
            normalize_flight_route(route).map_err(|error| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Julia capability-manifest health_route is invalid: {error}",
                    repository.id
                ),
            })
        }
        None => Ok(DEFAULT_JULIA_HEALTH_ROUTE.to_string()),
    }
}

fn resolve_capability_manifest_schema_version(
    repository: &RegisteredRepository,
    schema_version: Option<String>,
) -> Result<String, RepoIntelligenceError> {
    match schema_version {
        Some(schema_version) => validate_flight_schema_version(&schema_version).map_err(|error| {
            RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Julia capability-manifest schema version is invalid: {error}",
                    repository.id
                ),
            }
        }),
        None => Ok(JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION.to_string()),
    }
}

fn resolve_capability_manifest_timeout_secs(
    repository: &RegisteredRepository,
    timeout_secs: Option<u64>,
) -> Result<u64, RepoIntelligenceError> {
    match timeout_secs {
        Some(timeout_secs) => validate_flight_timeout_secs(timeout_secs).map_err(|error| {
            RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Julia capability-manifest timeout is invalid: {error}",
                    repository.id
                ),
            }
        }),
        None => Ok(DEFAULT_FLIGHT_TIMEOUT_SECS),
    }
}
