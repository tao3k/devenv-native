//! Julia capability manifest batch, validation, decoding, and transport helpers.

mod batch;
mod support;
mod transport;

#[cfg(test)]
pub(super) use crate::plugin::capability_manifest::{
    JULIA_PLUGIN_CAPABILITY_MANIFEST_BASE_URL_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_CAPABILITY_VARIANT_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ENABLED_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_HEALTH_ROUTE_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_RESPONSE_PLUGIN_ID_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_ROUTE_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_SCHEMA_VERSION_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TIMEOUT_SECS_COLUMN,
    JULIA_PLUGIN_CAPABILITY_MANIFEST_TRANSPORT_KIND_COLUMN,
    JuliaPluginCapabilityManifestRequestRow, JuliaPluginCapabilityManifestRow,
};
pub use batch::{
    build_julia_plugin_capability_manifest_request_batch,
    decode_julia_plugin_capability_manifest_rows,
    validate_julia_plugin_capability_manifest_request_batches,
    validate_julia_plugin_capability_manifest_response_batches,
};
pub(super) use support::parse_transport_kind;
#[cfg(test)]
pub(crate) use transport::graph_structural_binding_from_capability_manifest_rows;
pub use transport::{
    build_julia_capability_manifest_flight_transport_client,
    fetch_julia_plugin_capability_manifest_rows_for_repository,
    process_julia_capability_manifest_flight_batches,
    process_julia_capability_manifest_flight_batches_for_repository,
};
pub(crate) use transport::{
    discover_julia_graph_structural_binding_from_manifest_for_repository,
    validate_julia_capability_manifest_preflight_for_repository,
};

#[cfg(test)]
#[path = "../../../../tests/unit/plugin/capability_manifest.rs"]
mod tests;
