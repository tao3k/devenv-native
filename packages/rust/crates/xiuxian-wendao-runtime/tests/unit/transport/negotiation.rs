use super::{
    CANONICAL_PLUGIN_TRANSPORT_PREFERENCE_ORDER, negotiate_flight_transport_client_from_bindings,
};
use xiuxian_wendao_core::{
    capabilities::{ContractVersion, PluginCapabilityBinding, PluginProviderSelector},
    ids::{CapabilityId, PluginId},
    transport::{PluginTransportEndpoint, PluginTransportKind},
};

fn sample_binding(
    transport: PluginTransportKind,
    base_url: Option<&str>,
    route: &str,
) -> PluginCapabilityBinding {
    PluginCapabilityBinding {
        selector: PluginProviderSelector {
            capability_id: CapabilityId("rerank".to_string()),
            provider: PluginId("xiuxian-julia-core".to_string()),
        },
        endpoint: PluginTransportEndpoint {
            base_url: base_url.map(ToString::to_string),
            route: Some(route.to_string()),
            health_route: Some("/healthz".to_string()),
            timeout_secs: Some(15),
            max_in_flight_requests: None,
        },
        launch: None,
        transport,
        contract_version: ContractVersion("v2".to_string()),
    }
}

#[test]
fn canonical_transport_preference_order_is_flight_first() {
    assert_eq!(
        CANONICAL_PLUGIN_TRANSPORT_PREFERENCE_ORDER,
        [PluginTransportKind::ArrowFlight]
    );
}

#[test]
fn negotiation_selects_arrow_flight_when_binding_is_materializable() {
    let negotiated = negotiate_flight_transport_client_from_bindings(&[sample_binding(
        PluginTransportKind::ArrowFlight,
        Some("http://127.0.0.1:18080"),
        "/flight",
    )])
    .unwrap_or_else(|error| panic!("transport negotiation should succeed: {error}"))
    .unwrap_or_else(|| panic!("transport negotiation should select the Flight client"));

    assert_eq!(
        negotiated.selection().selected_transport,
        PluginTransportKind::ArrowFlight
    );
    assert_eq!(negotiated.selection().fallback_from, None);
    assert_eq!(negotiated.flight_base_url(), "http://127.0.0.1:18080");
    assert_eq!(negotiated.flight_route(), "/flight");
}

#[test]
fn negotiation_reports_flight_materialization_errors_without_ipc_fallback() {
    let result = negotiate_flight_transport_client_from_bindings(&[sample_binding(
        PluginTransportKind::ArrowFlight,
        Some("http://127.0.0.1:18080"),
        "",
    )]);

    let error = match result {
        Ok(value) => panic!(
            "incomplete Flight bindings should now fail directly: {value_present}",
            value_present = value.is_some()
        ),
        Err(error) => error,
    };

    assert!(
        error.contains("preferred transport ArrowFlight is unavailable"),
        "expected transport negotiation failure context, got: {error}"
    );
    assert!(
        error.contains("failed to construct Arrow Flight client"),
        "expected Arrow Flight client construction error, got: {error}"
    );
    assert!(
        error.contains("at least one descriptor segment"),
        "expected Flight materialization error, got: {error}"
    );
}

#[test]
fn negotiation_returns_none_when_candidate_bindings_are_unconfigured() {
    let negotiated = negotiate_flight_transport_client_from_bindings(&[sample_binding(
        PluginTransportKind::ArrowFlight,
        None,
        "/flight",
    )])
    .unwrap_or_else(|error| panic!("transport negotiation should not fail: {error}"));

    assert!(negotiated.is_none());
}
