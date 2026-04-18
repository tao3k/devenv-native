use arrow::array::{Float64Array, StringArray};
use arrow::record_batch::RecordBatch;
use xiuxian_wendao_core::{
    capabilities::{ContractVersion, PluginCapabilityBinding},
    repo_intelligence::{
        JULIA_ARROW_ANALYZER_SCORE_COLUMN, JULIA_ARROW_DOC_ID_COLUMN,
        JULIA_ARROW_FINAL_SCORE_COLUMN, JULIA_ARROW_TRACE_ID_COLUMN, RegisteredRepository,
        RepositoryPluginConfig,
    },
    transport::{PluginTransportEndpoint, PluginTransportKind},
};
use xiuxian_wendao_runtime::transport::{
    NegotiatedFlightTransportClient, negotiate_flight_transport_client_from_bindings,
};

use super::{
    DEFAULT_JULIA_HEALTH_ROUTE, JULIA_ARROW_RESPONSE_SCHEMA_VERSION,
    build_flight_transport_binding, build_julia_flight_transport_client,
    process_julia_flight_batches, process_julia_flight_batches_for_repository,
};
use crate::compatibility::link_graph::julia_rerank_provider_selector;
use crate::julia_plugin_test_support::contract::{request_batch, request_batch_with_trace_id};
use crate::julia_plugin_test_support::official_examples::{
    reserve_real_service_port, spawn_real_wendaoanalyzer_linear_blend_service,
    spawn_real_wendaoarrow_bad_response_service, spawn_real_wendaoarrow_metadata_service,
    spawn_real_wendaoarrow_service, wait_for_service_ready, wait_for_service_ready_with_attempts,
};

fn string_column<'a>(batch: &'a RecordBatch, name: &str) -> &'a StringArray {
    let Some(column) = batch
        .column_by_name(name)
        .and_then(|array| array.as_any().downcast_ref::<StringArray>())
    else {
        panic!("missing StringArray column `{name}`");
    };
    column
}

fn float64_column<'a>(batch: &'a RecordBatch, name: &str) -> &'a Float64Array {
    let Some(column) = batch
        .column_by_name(name)
        .and_then(|array| array.as_any().downcast_ref::<Float64Array>())
    else {
        panic!("missing Float64Array column `{name}`");
    };
    column
}

include!("transport/config.rs");
include!("transport/live_roundtrip.rs");

fn test_transport_client(base_url: String, route: &str) -> NegotiatedFlightTransportClient {
    negotiate_flight_transport_client_from_bindings(&[PluginCapabilityBinding {
        selector: julia_rerank_provider_selector(),
        endpoint: PluginTransportEndpoint {
            base_url: Some(base_url),
            route: Some(route.to_string()),
            health_route: Some(DEFAULT_JULIA_HEALTH_ROUTE.to_string()),
            timeout_secs: Some(15),
            max_in_flight_requests: None,
        },
        launch: None,
        transport: PluginTransportKind::ArrowFlight,
        contract_version: ContractVersion(JULIA_ARROW_RESPONSE_SCHEMA_VERSION.to_string()),
    }])
    .unwrap_or_else(|error| panic!("build negotiated Flight transport client: {error}"))
    .unwrap_or_else(|| panic!("negotiated Flight transport client should exist"))
}
