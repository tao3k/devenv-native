use xiuxian_wendao_core::transport::PluginTransportKind;
use xiuxian_wendao_runtime::transport::negotiate_flight_transport_client_from_bindings;

use super::{
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_CAPABILITY_ID,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_PROVIDER_ID,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ROUTE,
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION,
    WendaoGraphOntologyReadModelQualityFlightBindingOptions,
    build_wendaograph_ontology_read_model_quality_flight_binding,
    wendaograph_ontology_read_model_quality_provider_selector,
};

#[test]
fn ontology_read_model_quality_flight_binding_targets_runtime_negotiation() {
    let selector = wendaograph_ontology_read_model_quality_provider_selector();
    let binding = build_wendaograph_ontology_read_model_quality_flight_binding(
        WendaoGraphOntologyReadModelQualityFlightBindingOptions {
            base_url: "http://127.0.0.1:41082".to_string(),
            health_route: Some("/health".to_string()),
            timeout_secs: Some(30),
            max_in_flight_requests: Some(2),
        },
    )
    .unwrap_or_else(|error| panic!("build ontology quality Flight binding: {error}"));

    assert_eq!(
        selector.capability_id.0,
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_CAPABILITY_ID
    );
    assert_eq!(
        selector.provider.0,
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_PROVIDER_ID
    );
    assert_eq!(binding.selector, selector);
    assert_eq!(
        binding.endpoint.base_url.as_deref(),
        Some("http://127.0.0.1:41082")
    );
    assert_eq!(
        binding.endpoint.route.as_deref(),
        Some(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ROUTE)
    );
    assert_eq!(binding.endpoint.health_route.as_deref(), Some("/health"));
    assert_eq!(binding.endpoint.timeout_secs, Some(30));
    assert_eq!(binding.endpoint.max_in_flight_requests, Some(2));
    assert_eq!(binding.launch, None);
    assert_eq!(binding.transport, PluginTransportKind::ArrowFlight);
    assert_eq!(
        binding.contract_version.0,
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_SCHEMA_VERSION
    );
}

#[test]
fn ontology_read_model_quality_flight_binding_rejects_blank_base_url() {
    let error = build_wendaograph_ontology_read_model_quality_flight_binding(
        WendaoGraphOntologyReadModelQualityFlightBindingOptions {
            base_url: " ".to_string(),
            health_route: None,
            timeout_secs: None,
            max_in_flight_requests: None,
        },
    )
    .expect_err("blank base URL should be rejected");

    assert!(error.contains("base URL"));
}

#[test]
fn ontology_read_model_quality_flight_binding_negotiates_runtime_client() {
    let binding = build_wendaograph_ontology_read_model_quality_flight_binding(
        WendaoGraphOntologyReadModelQualityFlightBindingOptions {
            base_url: "http://127.0.0.1:41082".to_string(),
            health_route: None,
            timeout_secs: Some(30),
            max_in_flight_requests: Some(2),
        },
    )
    .unwrap_or_else(|error| panic!("build ontology quality Flight binding: {error}"));
    let negotiated = negotiate_flight_transport_client_from_bindings(&[binding])
        .unwrap_or_else(|error| panic!("negotiate ontology quality Flight binding: {error}"))
        .expect("ontology quality Flight binding should negotiate a runtime client");

    assert_eq!(
        negotiated.flight_route(),
        WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ROUTE
    );
    assert_eq!(negotiated.flight_base_url(), "http://127.0.0.1:41082");
    assert_eq!(
        negotiated.selection().selected_transport,
        PluginTransportKind::ArrowFlight
    );
    assert_eq!(negotiated.selection().fallback_from, None);
}
