use arrow::array::StringArray;
use xiuxian_wendao_core::transport::PluginTransportKind;

use super::fake_flight::spawn_ontology_quality_flight_service;
use super::support::sample_request_batches;
use super::{
    WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ROUTE,
    WendaoGraphOntologyReadModelQualityFlightBindingOptions,
    build_wendaograph_ontology_read_model_quality_flight_binding,
    roundtrip_wendaograph_ontology_read_model_quality_with_binding,
};

#[tokio::test]
async fn ontology_read_model_quality_roundtrip_uses_runtime_flight_exchange() {
    let (base_url, server) = spawn_ontology_quality_flight_service().await;
    let binding = build_wendaograph_ontology_read_model_quality_flight_binding(
        WendaoGraphOntologyReadModelQualityFlightBindingOptions {
            base_url,
            health_route: None,
            timeout_secs: Some(5),
            max_in_flight_requests: Some(1),
        },
    )
    .unwrap_or_else(|error| panic!("build ontology quality binding: {error}"));

    let roundtrip = roundtrip_wendaograph_ontology_read_model_quality_with_binding(
        &binding,
        &sample_request_batches(),
    )
    .await
    .unwrap_or_else(|error| panic!("ontology quality roundtrip should succeed: {error:?}"))
    .unwrap_or_else(|| panic!("binding should negotiate a runtime transport"));

    assert_eq!(
        roundtrip.selection.selected_transport,
        PluginTransportKind::ArrowFlight
    );
    assert_eq!(roundtrip.selection.fallback_from, None);
    assert_eq!(roundtrip.response_batches.len(), 1);
    assert_eq!(roundtrip.response_batches[0].num_rows(), 1);
    assert_eq!(
        binding.endpoint.route.as_deref(),
        Some(WENDAO_GRAPH_ONTOLOGY_READ_MODEL_QUALITY_ROUTE)
    );

    let check_ids = roundtrip.response_batches[0]
        .column_by_name("check_id")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .unwrap_or_else(|| panic!("response check_id column should decode as Utf8"));
    let statuses = roundtrip.response_batches[0]
        .column_by_name("status")
        .and_then(|column| column.as_any().downcast_ref::<StringArray>())
        .unwrap_or_else(|| panic!("response status column should decode as Utf8"));
    assert_eq!(check_ids.value(0), "object_graph_component_count");
    assert_eq!(statuses.value(0), "pass");

    server.abort();
}
