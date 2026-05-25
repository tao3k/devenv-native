//! Julia plugin capability manifest request and response contract.

mod contract;
mod schema;

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
pub(super) use schema::{
    ARROW_FLIGHT_TRANSPORT_KIND, CAPABILITY_MANIFEST_TRANSPORT_KEY, DEFAULT_JULIA_HEALTH_ROUTE,
    JULIA_PLUGIN_CONFIG_ID,
};
pub use schema::{
    JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_FILTER_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_INCLUDE_DISABLED_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_PLUGIN_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_REPOSITORY_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_REQUEST_COLUMNS,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_COLUMNS,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE, JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
    JuliaPluginCapabilityManifestRequestRow, JuliaPluginCapabilityManifestRow,
};
