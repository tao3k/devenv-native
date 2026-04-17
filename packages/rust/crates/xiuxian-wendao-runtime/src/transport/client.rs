use std::time::Duration;

use xiuxian_wendao_core::capabilities::PluginCapabilityBinding;
use xiuxian_wendao_core::transport::PluginTransportKind;

use super::contract::{
    DEFAULT_FLIGHT_SCHEMA_VERSION, resolve_flight_max_in_flight_requests, resolve_flight_timeout,
    validate_flight_schema_version,
};
use super::flight::ArrowFlightTransportClient;

/// Build an Arrow Flight transport client from a generic plugin capability binding.
///
/// # Errors
///
/// Returns an error when the contract version or timeout cannot be translated
/// into a valid Arrow transport configuration, when the binding requests an
/// unsupported runtime transport kind, or when the Flight client cannot be
/// constructed.
pub(crate) fn build_arrow_flight_transport_client_from_binding(
    binding: &PluginCapabilityBinding,
) -> Result<Option<ArrowFlightTransportClient>, String> {
    let Some(base_url) = binding.endpoint.base_url.as_deref() else {
        return Ok(None);
    };

    if binding.transport != PluginTransportKind::ArrowFlight {
        return Err(format!(
            "unsupported plugin transport for Arrow Flight client construction: {:?}",
            binding.transport
        ));
    }

    let Some(route) = binding.endpoint.route.as_deref() else {
        return Err(
            "Arrow Flight client construction requires a route-backed FlightDescriptor path"
                .to_string(),
        );
    };

    let schema_version = normalized_schema_version(binding)?;
    let timeout = normalized_timeout(binding)?;
    let max_in_flight_requests = normalized_max_in_flight_requests(binding)?;

    ArrowFlightTransportClient::new(
        base_url,
        route,
        schema_version,
        timeout,
        max_in_flight_requests,
    )
    .map(Some)
    .map_err(|error| format!("failed to construct Arrow Flight client: {error}"))
}

fn normalized_schema_version(binding: &PluginCapabilityBinding) -> Result<String, String> {
    if binding.contract_version.0.trim().is_empty() {
        return Ok(DEFAULT_FLIGHT_SCHEMA_VERSION.to_string());
    }

    validate_flight_schema_version(binding.contract_version.0.as_str())
        .map_err(|error| format!("invalid plugin transport schema version: {error}"))
}

fn normalized_timeout(binding: &PluginCapabilityBinding) -> Result<Duration, String> {
    resolve_flight_timeout(binding.endpoint.timeout_secs)
        .map_err(|error| format!("invalid plugin transport timeout: {error}"))
}

fn normalized_max_in_flight_requests(binding: &PluginCapabilityBinding) -> Result<usize, String> {
    resolve_flight_max_in_flight_requests(binding.endpoint.max_in_flight_requests)
        .map_err(|error| format!("invalid plugin transport max_in_flight_requests: {error}"))
}

#[cfg(test)]
#[path = "../../tests/unit/transport/client.rs"]
mod tests;
