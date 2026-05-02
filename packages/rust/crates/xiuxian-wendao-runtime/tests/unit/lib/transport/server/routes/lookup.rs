use std::sync::Arc;

use arrow_flight::Ticket;
use arrow_flight::flight_service_server::FlightService;
use futures::StreamExt;
use tonic::Request;

use crate::transport::{SEARCH_AUTOCOMPLETE_ROUTE, SEARCH_DEFINITION_ROUTE};

use crate::tests::transport::server::assertions::{
    must_ok, parse_json, route_descriptor, ticket_string,
};
use crate::tests::transport::server::fixtures::build_service_with_route_providers;
use crate::tests::transport::server::providers::{
    RecordingAutocompleteProvider, RecordingDefinitionProvider,
};
use crate::tests::transport::server::request_headers::{
    populate_schema_and_autocomplete_headers, populate_schema_and_definition_headers,
};

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_definition_provider() {
    let provider = Arc::new(RecordingDefinitionProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.definition = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(SEARCH_DEFINITION_ROUTE));
    populate_schema_and_definition_headers(
        request.metadata_mut(),
        "AlphaService",
        Some("src/lib.rs"),
        Some("7"),
    );

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "definition route should resolve through the pluggable provider",
    )
    .into_inner();
    let ticket = ticket_string(&flight_info, "definition route should emit one ticket");
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");

    assert_eq!(ticket, SEARCH_DEFINITION_ROUTE);
    assert_eq!(app_metadata["query"], "AlphaService");
    assert_eq!(app_metadata["candidateCount"], 1);
    assert_eq!(provider.call_count(), 1);
    assert_eq!(
        provider.recorded_request(),
        Some((
            "AlphaService".to_string(),
            Some("src/lib.rs".to_string()),
            Some(7),
        ))
    );
}

#[tokio::test]
async fn wendao_flight_service_do_get_reuses_definition_provider_batch() {
    let provider = Arc::new(RecordingDefinitionProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.definition = Some(provider.clone());
    });
    let mut request = Request::new(Ticket::new(SEARCH_DEFINITION_ROUTE.to_string()));
    populate_schema_and_definition_headers(
        request.metadata_mut(),
        "AlphaService",
        Some("src/lib.rs"),
        Some("7"),
    );

    let frames = must_ok(
        service.do_get(request).await,
        "definition route should stream through the pluggable provider",
    )
    .into_inner()
    .collect::<Vec<_>>()
    .await;

    assert!(!frames.is_empty());
    assert_eq!(provider.call_count(), 1);
}

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_autocomplete_provider() {
    let provider = Arc::new(RecordingAutocompleteProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.autocomplete = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(SEARCH_AUTOCOMPLETE_ROUTE));
    populate_schema_and_autocomplete_headers(request.metadata_mut(), "Alpha", "5");

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "autocomplete route should resolve through the pluggable provider",
    )
    .into_inner();
    let ticket = ticket_string(&flight_info, "autocomplete route should emit one ticket");
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");

    assert_eq!(ticket, SEARCH_AUTOCOMPLETE_ROUTE);
    assert_eq!(app_metadata["prefix"], "Alpha");
    assert_eq!(provider.call_count(), 1);
    assert_eq!(provider.recorded_request(), Some(("Alpha".to_string(), 5)));
}
