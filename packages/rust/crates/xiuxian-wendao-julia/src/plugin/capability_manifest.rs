use std::sync::Arc;

use arrow::array::{Array, BooleanArray, StringArray, UInt64Array};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use serde_json::Value;
use xiuxian_wendao_core::{
    capabilities::{ContractVersion, PluginCapabilityBinding, PluginProviderSelector},
    ids::{CapabilityId, PluginId},
    repo_intelligence::{RegisteredRepository, RepoIntelligenceError, RepositoryPluginConfig},
    transport::{PluginTransportEndpoint, PluginTransportKind},
};
use xiuxian_wendao_runtime::transport::{
    DEFAULT_FLIGHT_BASE_URL, DEFAULT_FLIGHT_TIMEOUT_SECS, FLIGHT_SCHEMA_VERSION_METADATA_KEY,
    NegotiatedFlightTransportClient, negotiate_flight_transport_client_from_bindings,
    normalize_flight_route, validate_flight_schema_version, validate_flight_timeout_secs,
};

use super::graph_structural::GraphStructuralRouteKind;
use crate::arrow_metadata::attach_record_batch_metadata;
use crate::compatibility::link_graph::{
    JULIA_CAPABILITY_MANIFEST_CAPABILITY_ID, JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID,
    julia_capability_manifest_provider_selector,
};

const JULIA_PLUGIN_CONFIG_ID: &str = "julia";
const CAPABILITY_MANIFEST_TRANSPORT_KEY: &str = "capability_manifest_transport";
const DEFAULT_JULIA_HEALTH_ROUTE: &str = "/healthz";
const ARROW_FLIGHT_TRANSPORT_KIND: &str = "arrow_flight";

/// Canonical Arrow Flight route for Julia capability discovery.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE: &str = "/plugin/capabilities";
/// Draft contract version for the Julia capability-manifest lane.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION: &str = "v0-draft";

/// Request column used to identify the plugin being discovered.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_PLUGIN_ID_COLUMN: &str = "plugin_id";
/// Request column used to pass repository identity into discovery.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_REPOSITORY_ID_COLUMN: &str = "repository_id";
/// Request column used to restrict discovery to one capability family.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_FILTER_COLUMN: &str = "capability_filter";
/// Request column used to decide whether disabled capabilities should be included.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_INCLUDE_DISABLED_COLUMN: &str = "include_disabled";

/// Response column carrying the discovered plugin id.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN: &str = "plugin_id";
/// Response column carrying the discovered capability id.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN: &str = "capability_id";
/// Response column carrying one capability variant or operation tag.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN: &str = "capability_variant";
/// Response column carrying the transport kind.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN: &str = "transport_kind";
/// Response column carrying the remote base URL.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN: &str = "base_url";
/// Response column carrying the route descriptor.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN: &str = "route";
/// Response column carrying the health route.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN: &str = "health_route";
/// Response column carrying the schema version.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN: &str = "schema_version";
/// Response column carrying the timeout in seconds.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN: &str = "timeout_secs";
/// Response column carrying whether the capability is enabled.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN: &str = "enabled";

/// Ordered request columns for the Julia capability-manifest contract.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_REQUEST_COLUMNS: [&str; 4] = [
    JULIA_PLUGIN_CAPABILITY_MANIFEST_PLUGIN_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_REPOSITORY_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_FILTER_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_INCLUDE_DISABLED_COLUMN,
];

/// Ordered response columns for the Julia capability-manifest contract.
pub const JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_COLUMNS: [&str; 10] = [
    JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN,
];

/// One typed request row for the Julia plugin capability-manifest route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JuliaPluginCapabilityManifestRequestRow {
    /// Canonical plugin identifier being discovered.
    pub plugin_id: String,
    /// Repository identity attached to the discovery request.
    pub repository_id: String,
    /// Optional capability-family filter.
    pub capability_filter: Option<String>,
    /// Whether disabled capabilities should be returned.
    pub include_disabled: bool,
}

/// One decoded capability-manifest response row from the Julia plugin service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JuliaPluginCapabilityManifestRow {
    /// Canonical plugin identifier returned by the discovery route.
    pub plugin_id: String,
    /// Stable capability identifier for the returned binding.
    pub capability_id: String,
    /// Optional capability variant or operation tag.
    pub capability_variant: Option<String>,
    /// Transport kind required for this capability.
    pub transport_kind: String,
    /// Base URL for the capability service.
    pub base_url: String,
    /// Route for the capability service.
    pub route: String,
    /// Optional health-check route for the capability service.
    pub health_route: Option<String>,
    /// Schema version negotiated for this capability.
    pub schema_version: String,
    /// Optional timeout in seconds.
    pub timeout_secs: Option<u64>,
    /// Whether this capability is enabled.
    pub enabled: bool,
}

impl JuliaPluginCapabilityManifestRow {
    /// Return the selector described by this manifest row.
    #[must_use]
    pub fn selector(&self) -> PluginProviderSelector {
        PluginProviderSelector {
            capability_id: CapabilityId(self.capability_id.clone()),
            provider: PluginId(self.plugin_id.clone()),
        }
    }

    /// Convert one enabled manifest row into a runtime binding.
    ///
    /// # Errors
    ///
    /// Returns [`RepoIntelligenceError`] when the row contains an unsupported
    /// transport kind or invalid transport settings.
    pub fn to_binding(&self) -> Result<Option<PluginCapabilityBinding>, RepoIntelligenceError> {
        if !self.enabled {
            return Ok(None);
        }

        let transport = parse_transport_kind(&self.transport_kind)?;
        let route = normalize_flight_route(self.route.clone()).map_err(|error| {
            RepoIntelligenceError::AnalysisFailed {
                message: format!(
                    "Julia capability-manifest row `{}` has invalid route `{}`: {error}",
                    self.capability_id, self.route
                ),
            }
        })?;
        let schema_version = validate_flight_schema_version(&self.schema_version).map_err(
            |error| RepoIntelligenceError::AnalysisFailed {
                message: format!(
                    "Julia capability-manifest row `{}` has invalid schema version `{}`: {error}",
                    self.capability_id, self.schema_version
                ),
            },
        )?;
        let timeout_secs = self
            .timeout_secs
            .map(|timeout| {
                validate_flight_timeout_secs(timeout).map_err(|error| {
                    RepoIntelligenceError::AnalysisFailed {
                        message: format!(
                            "Julia capability-manifest row `{}` has invalid timeout `{timeout}`: {error}",
                            self.capability_id
                        ),
                    }
                })
            })
            .transpose()?;
        let health_route = self
            .health_route
            .as_ref()
            .map(|route| {
                normalize_flight_route(route.clone()).map_err(|error| {
                    RepoIntelligenceError::AnalysisFailed {
                        message: format!(
                            "Julia capability-manifest row `{}` has invalid health route `{route}`: {error}",
                            self.capability_id
                        ),
                    }
                })
            })
            .transpose()?;

        Ok(Some(PluginCapabilityBinding {
            selector: self.selector(),
            endpoint: PluginTransportEndpoint {
                base_url: Some(self.base_url.clone()),
                route: Some(route),
                health_route,
                timeout_secs,
                max_in_flight_requests: None,
            },
            launch: None,
            transport,
            contract_version: ContractVersion(schema_version),
        }))
    }
}

/// Build a Julia capability-manifest request batch from typed request rows.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the batch cannot be materialized or
/// violates the staged request contract.
pub fn build_julia_plugin_capability_manifest_request_batch(
    rows: &[JuliaPluginCapabilityManifestRequestRow],
) -> Result<RecordBatch, RepoIntelligenceError> {
    let batch = RecordBatch::try_new(
        julia_plugin_capability_manifest_request_schema(),
        vec![
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.plugin_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.repository_id.as_str())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(StringArray::from(
                rows.iter()
                    .map(|row| row.capability_filter.as_deref())
                    .collect::<Vec<_>>(),
            )),
            Arc::new(BooleanArray::from(
                rows.iter()
                    .map(|row| row.include_disabled)
                    .collect::<Vec<_>>(),
            )),
        ],
    )
    .map_err(|error| manifest_request_error(error.to_string()))?;
    validate_julia_plugin_capability_manifest_request_batches(std::slice::from_ref(&batch))?;
    Ok(batch)
}

/// Validate Julia capability-manifest request batches.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any request batch violates the
/// staged request contract.
pub fn validate_julia_plugin_capability_manifest_request_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        let plugin_id = utf8_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_PLUGIN_ID_COLUMN,
            "request",
        )?;
        let repository_id = utf8_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_REPOSITORY_ID_COLUMN,
            "request",
        )?;
        let _capability_filter = nullable_utf8_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_FILTER_COLUMN,
            "request",
        )?;
        let include_disabled = bool_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_INCLUDE_DISABLED_COLUMN,
            "request",
        )?;

        for row in 0..batch.num_rows() {
            if plugin_id.is_null(row) || plugin_id.value(row).trim().is_empty() {
                return Err(manifest_contract_error(
                    "request",
                    "`plugin_id` must be non-null and non-blank",
                ));
            }
            if repository_id.is_null(row) || repository_id.value(row).trim().is_empty() {
                return Err(manifest_contract_error(
                    "request",
                    "`repository_id` must be non-null and non-blank",
                ));
            }
            if include_disabled.is_null(row) {
                return Err(manifest_contract_error(
                    "request",
                    "`include_disabled` must be non-null",
                ));
            }
        }
    }

    Ok(())
}

/// Validate Julia capability-manifest response batches.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when any response batch violates the
/// staged response contract.
pub fn validate_julia_plugin_capability_manifest_response_batches(
    batches: &[RecordBatch],
) -> Result<(), RepoIntelligenceError> {
    for batch in batches {
        let response_columns = JuliaPluginCapabilityManifestResponseColumns::new(batch)?;
        for row in 0..batch.num_rows() {
            validate_julia_plugin_capability_manifest_response_row(&response_columns, row)?;
        }
    }

    Ok(())
}

struct JuliaPluginCapabilityManifestResponseColumns<'a> {
    plugin_id: &'a StringArray,
    capability_id: &'a StringArray,
    capability_variant: &'a StringArray,
    transport_kind: &'a StringArray,
    base_url: &'a StringArray,
    route: &'a StringArray,
    health_route: &'a StringArray,
    schema_version: &'a StringArray,
    timeout_secs: &'a UInt64Array,
    enabled: &'a BooleanArray,
}

impl<'a> JuliaPluginCapabilityManifestResponseColumns<'a> {
    fn new(batch: &'a RecordBatch) -> Result<Self, RepoIntelligenceError> {
        Ok(Self {
            plugin_id: utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
                "response",
            )?,
            capability_id: utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN,
                "response",
            )?,
            capability_variant: nullable_utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
                "response",
            )?,
            transport_kind: utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
                "response",
            )?,
            base_url: utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
                "response",
            )?,
            route: utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN,
                "response",
            )?,
            health_route: nullable_utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN,
                "response",
            )?,
            schema_version: utf8_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
                "response",
            )?,
            timeout_secs: nullable_u64_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
                "response",
            )?,
            enabled: bool_column(
                batch,
                JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN,
                "response",
            )?,
        })
    }
}

fn validate_julia_plugin_capability_manifest_response_row(
    columns: &JuliaPluginCapabilityManifestResponseColumns<'_>,
    row: usize,
) -> Result<(), RepoIntelligenceError> {
    let _capability_variant = string_value(columns.capability_variant, row);
    validate_non_blank_manifest_response_value(
        columns.plugin_id,
        row,
        "`plugin_id` must be non-null and non-blank",
    )?;
    validate_non_blank_manifest_response_value(
        columns.capability_id,
        row,
        "`capability_id` must be non-null and non-blank",
    )?;
    let transport_kind = validate_non_blank_manifest_response_value(
        columns.transport_kind,
        row,
        "`transport_kind` must be non-null and non-blank",
    )?;
    parse_transport_kind(transport_kind)?;
    validate_non_blank_manifest_response_value(
        columns.base_url,
        row,
        "`base_url` must be non-null and non-blank",
    )?;
    let route = validate_non_blank_manifest_response_value(
        columns.route,
        row,
        "`route` must be non-null and non-blank",
    )?;
    validate_manifest_response_route(route, "`route` must be a normalized Flight route")?;
    if let Some(health_route) = string_value(columns.health_route, row) {
        validate_manifest_response_route(
            health_route,
            "`health_route` must be a normalized Flight route",
        )?;
    }
    let schema_version = validate_non_blank_manifest_response_value(
        columns.schema_version,
        row,
        "`schema_version` must be non-null and non-blank",
    )?;
    validate_flight_schema_version(schema_version).map_err(|error| {
        manifest_contract_error(
            "response",
            format!("`schema_version` must be valid: {error}"),
        )
    })?;
    if let Some(timeout_secs) = u64_value(columns.timeout_secs, row) {
        validate_flight_timeout_secs(timeout_secs).map_err(|error| {
            manifest_contract_error("response", format!("`timeout_secs` must be valid: {error}"))
        })?;
    }
    if columns.enabled.is_null(row) {
        return Err(manifest_contract_error(
            "response",
            "`enabled` must be non-null",
        ));
    }
    Ok(())
}

fn validate_non_blank_manifest_response_value<'a>(
    array: &'a StringArray,
    row: usize,
    error_message: &str,
) -> Result<&'a str, RepoIntelligenceError> {
    let value = string_value(array, row)
        .ok_or_else(|| manifest_contract_error("response", error_message))?;
    if value.trim().is_empty() {
        return Err(manifest_contract_error("response", error_message));
    }
    Ok(value)
}

fn validate_manifest_response_route(
    route: &str,
    prefix: &str,
) -> Result<(), RepoIntelligenceError> {
    normalize_flight_route(route)
        .map_err(|error| manifest_contract_error("response", format!("{prefix}: {error}")))?;
    Ok(())
}

/// Decode response batches from the Julia capability-manifest route.
///
/// # Errors
///
/// Returns [`RepoIntelligenceError`] when the staged response contract is
/// violated.
pub fn decode_julia_plugin_capability_manifest_rows(
    batches: &[RecordBatch],
) -> Result<Vec<JuliaPluginCapabilityManifestRow>, RepoIntelligenceError> {
    validate_julia_plugin_capability_manifest_response_batches(batches)?;

    let mut rows = Vec::new();
    for batch in batches {
        let plugin_id = utf8_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
            "response",
        )?;
        let capability_id = utf8_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN,
            "response",
        )?;
        let capability_variant = nullable_utf8_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
            "response",
        )?;
        let transport_kind = utf8_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
            "response",
        )?;
        let base_url = utf8_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
            "response",
        )?;
        let route = utf8_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN,
            "response",
        )?;
        let health_route = nullable_utf8_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN,
            "response",
        )?;
        let schema_version = utf8_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
            "response",
        )?;
        let timeout_secs = nullable_u64_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
            "response",
        )?;
        let enabled = bool_column(
            batch,
            JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN,
            "response",
        )?;

        for row in 0..batch.num_rows() {
            rows.push(JuliaPluginCapabilityManifestRow {
                plugin_id: plugin_id.value(row).to_string(),
                capability_id: capability_id.value(row).to_string(),
                capability_variant: string_value(capability_variant, row).map(str::to_string),
                transport_kind: transport_kind.value(row).to_string(),
                base_url: base_url.value(row).to_string(),
                route: route.value(row).to_string(),
                health_route: string_value(health_route, row).map(str::to_string),
                schema_version: schema_version.value(row).to_string(),
                timeout_secs: u64_value(timeout_secs, row),
                enabled: enabled.value(row),
            });
        }
    }

    Ok(rows)
}

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

pub(crate) fn fetch_julia_plugin_capability_manifest_rows_blocking_for_repository(
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
            plugin_id: expected_plugin_id.clone(),
            repository_id: repository.id.clone(),
            capability_filter: None,
            include_disabled: true,
        }],
    )?;

    if rows.is_empty() {
        return Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "repo `{}` Julia capability-manifest preflight returned no rows",
                repository.id
            ),
        });
    }

    if let Some(row) = rows.iter().find(|row| row.plugin_id != expected_plugin_id) {
        return Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "repo `{}` Julia capability-manifest preflight returned provider `{}` but expected `{}`",
                repository.id, row.plugin_id, expected_plugin_id
            ),
        });
    }

    if !rows
        .iter()
        .any(|row| row.capability_id == JULIA_CAPABILITY_MANIFEST_CAPABILITY_ID)
    {
        return Err(RepoIntelligenceError::AnalysisFailed {
            message: format!(
                "repo `{}` Julia capability-manifest preflight did not advertise capability `{}`",
                repository.id, JULIA_CAPABILITY_MANIFEST_CAPABILITY_ID
            ),
        });
    }

    Ok(Some(rows))
}

pub(crate) fn graph_structural_binding_from_capability_manifest_rows(
    rows: &[JuliaPluginCapabilityManifestRow],
    route_kind: GraphStructuralRouteKind,
) -> Result<Option<PluginCapabilityBinding>, RepoIntelligenceError> {
    let mut matching_rows = rows.iter().filter(|row| {
        row.capability_id == JULIA_GRAPH_STRUCTURAL_CAPABILITY_ID
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

fn build_capability_manifest_transport_binding(
    repository: &RegisteredRepository,
) -> Result<Option<PluginCapabilityBinding>, RepoIntelligenceError> {
    let Some(options) = resolve_capability_manifest_transport_options(repository)? else {
        return Ok(None);
    };

    if let Some(false) = options.enabled {
        return Ok(None);
    }

    let route = match options.route {
        Some(route) => {
            normalize_flight_route(route).map_err(|error| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Julia capability-manifest route is invalid: {error}",
                    repository.id
                ),
            })?
        }
        None => JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE.to_string(),
    };
    let health_route = match options.health_route {
        Some(route) => {
            normalize_flight_route(route).map_err(|error| RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Julia capability-manifest health_route is invalid: {error}",
                    repository.id
                ),
            })?
        }
        None => DEFAULT_JULIA_HEALTH_ROUTE.to_string(),
    };
    let schema_version =
        match options.schema_version {
            Some(schema_version) => validate_flight_schema_version(&schema_version).map_err(
                |error| RepoIntelligenceError::ConfigLoad {
                    message: format!(
                        "repo `{}` Julia capability-manifest schema version is invalid: {error}",
                        repository.id
                    ),
                },
            )?,
            None => JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION.to_string(),
        };
    let timeout_secs = match options.timeout_secs {
        Some(timeout_secs) => validate_flight_timeout_secs(timeout_secs).map_err(|error| {
            RepoIntelligenceError::ConfigLoad {
                message: format!(
                    "repo `{}` Julia capability-manifest timeout is invalid: {error}",
                    repository.id
                ),
            }
        })?,
        None => DEFAULT_FLIGHT_TIMEOUT_SECS,
    };

    Ok(Some(PluginCapabilityBinding {
        selector: julia_capability_manifest_provider_selector(),
        endpoint: PluginTransportEndpoint {
            base_url: Some(
                options
                    .base_url
                    .unwrap_or_else(|| DEFAULT_FLIGHT_BASE_URL.to_string()),
            ),
            route: Some(route),
            health_route: Some(health_route),
            timeout_secs: Some(timeout_secs),
            max_in_flight_requests: None,
        },
        launch: None,
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion(schema_version),
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

fn parse_transport_kind(value: &str) -> Result<PluginTransportKind, RepoIntelligenceError> {
    match value {
        ARROW_FLIGHT_TRANSPORT_KIND => Ok(PluginTransportKind::ArrowFlight),
        other => Err(manifest_contract_error(
            "response",
            format!("unsupported `transport_kind` `{other}`"),
        )),
    }
}

fn panic_payload_message(panic_payload: &Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = panic_payload.downcast_ref::<String>() {
        return message.clone();
    }
    if let Some(message) = panic_payload.downcast_ref::<&'static str>() {
        return (*message).to_string();
    }
    "unknown panic payload".to_string()
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

fn julia_plugin_capability_manifest_request_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_PLUGIN_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_REPOSITORY_ID_COLUMN,
            DataType::Utf8,
            false,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_FILTER_COLUMN,
            DataType::Utf8,
            true,
        ),
        Field::new(
            JULIA_PLUGIN_CAPABILITY_MANIFEST_INCLUDE_DISABLED_COLUMN,
            DataType::Boolean,
            false,
        ),
    ]))
}

fn utf8_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
    direction: &str,
) -> Result<&'a StringArray, RepoIntelligenceError> {
    batch
        .column_by_name(name)
        .and_then(|array| array.as_any().downcast_ref::<StringArray>())
        .ok_or_else(|| {
            manifest_contract_error(direction, format!("missing required Utf8 column `{name}`"))
        })
}

fn nullable_utf8_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
    direction: &str,
) -> Result<&'a StringArray, RepoIntelligenceError> {
    utf8_column(batch, name, direction)
}

fn bool_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
    direction: &str,
) -> Result<&'a BooleanArray, RepoIntelligenceError> {
    batch
        .column_by_name(name)
        .and_then(|array| array.as_any().downcast_ref::<BooleanArray>())
        .ok_or_else(|| {
            manifest_contract_error(
                direction,
                format!("missing required Boolean column `{name}`"),
            )
        })
}

fn nullable_u64_column<'a>(
    batch: &'a RecordBatch,
    name: &str,
    direction: &str,
) -> Result<&'a UInt64Array, RepoIntelligenceError> {
    batch
        .column_by_name(name)
        .and_then(|array| array.as_any().downcast_ref::<UInt64Array>())
        .ok_or_else(|| {
            manifest_contract_error(
                direction,
                format!("missing required UInt64 column `{name}`"),
            )
        })
}

fn string_value(array: &StringArray, row: usize) -> Option<&str> {
    (!array.is_null(row)).then(|| array.value(row))
}

fn u64_value(array: &UInt64Array, row: usize) -> Option<u64> {
    (!array.is_null(row)).then(|| array.value(row))
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

fn manifest_request_error(message: impl Into<String>) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "Julia capability-manifest request contract `{JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION}` violated: {}",
            message.into()
        ),
    }
}

fn manifest_contract_error(direction: &str, message: impl Into<String>) -> RepoIntelligenceError {
    RepoIntelligenceError::AnalysisFailed {
        message: format!(
            "Julia capability-manifest {direction} contract `{JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION}` violated: {}",
            message.into()
        ),
    }
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
#[path = "../../tests/unit/plugin/capability_manifest.rs"]
mod tests;
