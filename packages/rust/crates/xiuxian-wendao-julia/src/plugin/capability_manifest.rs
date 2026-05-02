//! Julia plugin capability manifest request and response contract.

mod contract;

pub use contract::{
    build_julia_capability_manifest_flight_transport_client,
    build_julia_plugin_capability_manifest_request_batch,
    decode_julia_plugin_capability_manifest_rows,
    fetch_julia_plugin_capability_manifest_rows_for_repository,
    process_julia_capability_manifest_flight_batches,
    process_julia_capability_manifest_flight_batches_for_repository,
    validate_julia_plugin_capability_manifest_request_batches,
    validate_julia_plugin_capability_manifest_response_batches,
};
pub(crate) use contract::{
    discover_julia_graph_structural_binding_from_manifest_for_repository,
    validate_julia_capability_manifest_preflight_for_repository,
};

use xiuxian_wendao_core::{
    capabilities::{ContractVersion, PluginCapabilityBinding, PluginProviderSelector},
    ids::{CapabilityId, PluginId},
    repo_intelligence::RepoIntelligenceError,
    transport::PluginTransportEndpoint,
};
use xiuxian_wendao_runtime::transport::{
    normalize_flight_route, validate_flight_schema_version, validate_flight_timeout_secs,
};

use contract::parse_transport_kind;

pub(super) const JULIA_PLUGIN_CONFIG_ID: &str = "julia";
pub(super) const CAPABILITY_MANIFEST_TRANSPORT_KEY: &str = "capability_manifest_transport";
pub(super) const DEFAULT_JULIA_HEALTH_ROUTE: &str = "/healthz";
pub(super) const ARROW_FLIGHT_TRANSPORT_KIND: &str = "arrow_flight";

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
