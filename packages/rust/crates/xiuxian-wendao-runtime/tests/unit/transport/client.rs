use super::build_arrow_flight_transport_client_from_binding;
use crate::transport::DEFAULT_FLIGHT_SCHEMA_VERSION;
use xiuxian_wendao_core::{
    capabilities::{ContractVersion, PluginCapabilityBinding, PluginProviderSelector},
    ids::{CapabilityId, PluginId},
    transport::{PluginTransportEndpoint, PluginTransportKind},
};

fn sample_binding(base_url: Option<&str>) -> PluginCapabilityBinding {
    PluginCapabilityBinding {
        selector: PluginProviderSelector {
            capability_id: CapabilityId("rerank".to_string()),
            provider: PluginId("xiuxian-julia-core".to_string()),
        },
        endpoint: PluginTransportEndpoint {
            base_url: base_url.map(ToString::to_string),
            route: Some("/rerank".to_string()),
            health_route: Some("/healthz".to_string()),
            timeout_secs: Some(15),
            max_in_flight_requests: None,
        },
        launch: None,
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion("v2".to_string()),
    }
}

#[test]
fn flight_transport_client_builder_requires_a_route_descriptor() {
    let result = build_arrow_flight_transport_client_from_binding(&PluginCapabilityBinding {
        transport: PluginTransportKind::ArrowFlight,
        endpoint: PluginTransportEndpoint {
            route: None,
            ..sample_binding(Some("http://127.0.0.1:18080")).endpoint
        },
        ..sample_binding(Some("http://127.0.0.1:18080"))
    });
    let Err(error) = result else {
        panic!("Arrow Flight construction should require an explicit route");
    };

    assert!(error.contains("FlightDescriptor"));
}

#[test]
fn flight_transport_client_builder_uses_flight_defaults_for_schema_and_timeout() {
    let mut binding = sample_binding(Some("http://127.0.0.1:18080"));
    binding.endpoint.timeout_secs = None;
    let client = build_arrow_flight_transport_client_from_binding(&PluginCapabilityBinding {
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion(String::new()),
        ..binding
    })
    .unwrap_or_else(|error| panic!("flight transport builder should succeed: {error}"))
    .unwrap_or_else(|| panic!("flight transport client should exist"));

    assert_eq!(client.base_url(), "http://127.0.0.1:18080");
    assert_eq!(client.route(), "/rerank");
    assert_eq!(client.schema_version(), DEFAULT_FLIGHT_SCHEMA_VERSION);
    assert_eq!(client.timeout().as_secs(), 10);
    assert_eq!(client.max_in_flight_requests(), 32);
}

#[test]
fn flight_transport_client_builder_uses_explicit_in_flight_budget() {
    let mut binding = sample_binding(Some("http://127.0.0.1:18080"));
    binding.endpoint.max_in_flight_requests = Some(3);
    let client = build_arrow_flight_transport_client_from_binding(&binding)
        .unwrap_or_else(|error| panic!("flight transport builder should succeed: {error}"))
        .unwrap_or_else(|| panic!("flight transport client should exist"));

    assert_eq!(client.max_in_flight_requests(), 3);
}

#[test]
fn flight_transport_client_builder_rejects_zero_in_flight_budget() {
    let mut binding = sample_binding(Some("http://127.0.0.1:18080"));
    binding.endpoint.max_in_flight_requests = Some(0);
    let Err(error) = build_arrow_flight_transport_client_from_binding(&binding) else {
        panic!("zero in-flight budget should fail");
    };

    assert!(error.contains("max_in_flight_requests"));
}
