use std::sync::Arc;

use arrow_flight::flight_service_server::FlightService;
use arrow_flight::{FlightData, Ticket};
use futures::StreamExt;
use tonic::Request;

use crate::transport::{RerankScoreWeights, SEARCH_INTENT_ROUTE, WendaoFlightService};

use super::super::super::assertions::{
    must_err, must_ok, must_some, parse_json, route_descriptor, ticket_string,
};
use super::super::super::fixtures::build_service_with_route_providers;
use super::super::super::providers::{RecordingRepoSearchProvider, RecordingSearchProvider};
use super::super::super::request_headers::populate_schema_and_search_headers;

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_search_family_provider() {
    let provider = Arc::new(RecordingSearchProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(SEARCH_INTENT_ROUTE));
    populate_schema_and_search_headers(request.metadata_mut(), "semantic-route", "4");

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "search-family route should resolve through the pluggable provider",
    )
    .into_inner();
    let ticket = ticket_string(&flight_info, "search-family route should emit one ticket");
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");

    assert_eq!(ticket, SEARCH_INTENT_ROUTE);
    assert_eq!(app_metadata["query"], "semantic-route");
    assert_eq!(app_metadata["hitCount"], 1);
    assert_eq!(provider.call_count(), 1);
    assert_eq!(
        provider.recorded_request(),
        Some((
            SEARCH_INTENT_ROUTE.to_string(),
            "semantic-route".to_string(),
            4,
            None,
            None,
        ))
    );
}

#[tokio::test]
async fn wendao_flight_service_do_get_reuses_search_family_provider_batch() {
    let provider = Arc::new(RecordingSearchProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(provider.clone());
    });
    let mut request = Request::new(Ticket::new(SEARCH_INTENT_ROUTE.to_string()));
    populate_schema_and_search_headers(request.metadata_mut(), "semantic-route", "2");

    let frames = must_ok(
        service.do_get(request).await,
        "search-family route should stream through the pluggable provider",
    )
    .into_inner()
    .collect::<Vec<_>>()
    .await;

    assert!(!frames.is_empty());
    assert_eq!(provider.call_count(), 1);
    assert_eq!(
        provider.recorded_request(),
        Some((
            SEARCH_INTENT_ROUTE.to_string(),
            "semantic-route".to_string(),
            2,
            None,
            None,
        ))
    );
}

#[tokio::test]
async fn wendao_flight_service_do_get_reuses_cached_search_family_payload_after_get_flight_info() {
    let provider = Arc::new(RecordingSearchProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(provider.clone());
    });
    let mut flight_info_request = Request::new(route_descriptor(SEARCH_INTENT_ROUTE));
    populate_schema_and_search_headers(flight_info_request.metadata_mut(), "semantic-route", "5");
    let flight_info = must_ok(
        service.get_flight_info(flight_info_request).await,
        "search-family route should resolve through the pluggable provider",
    )
    .into_inner();
    let ticket = must_some(
        flight_info
            .endpoint
            .first()
            .and_then(|endpoint| endpoint.ticket.clone()),
        "search-family route should emit one ticket",
    );

    let mut do_get_request = Request::new(ticket);
    populate_schema_and_search_headers(do_get_request.metadata_mut(), "semantic-route", "5");
    let frames = must_ok(
        service.do_get(do_get_request).await,
        "search-family route should reuse the cached payload",
    )
    .into_inner()
    .collect::<Vec<_>>()
    .await;

    assert!(!frames.is_empty());
    assert_eq!(provider.call_count(), 1);
}

#[tokio::test]
async fn wendao_flight_service_do_get_reuses_cached_search_family_encoded_frames() {
    let provider = Arc::new(RecordingSearchProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(provider.clone());
    });

    let mut first_request = Request::new(Ticket::new(SEARCH_INTENT_ROUTE.to_string()));
    populate_schema_and_search_headers(first_request.metadata_mut(), "semantic-route", "6");
    let first_frames = must_ok(
        service.do_get(first_request).await,
        "first DoGet should resolve through the pluggable provider",
    )
    .into_inner()
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .map(|frame| must_ok(frame, "first DoGet frame should stream successfully"))
    .collect::<Vec<FlightData>>();

    let mut second_request = Request::new(Ticket::new(SEARCH_INTENT_ROUTE.to_string()));
    populate_schema_and_search_headers(second_request.metadata_mut(), "semantic-route", "6");
    let second_frames = must_ok(
        service.do_get(second_request).await,
        "second DoGet should reuse the cached encoded frames",
    )
    .into_inner()
    .collect::<Vec<_>>()
    .await
    .into_iter()
    .map(|frame| must_ok(frame, "second DoGet frame should stream successfully"))
    .collect::<Vec<FlightData>>();

    assert!(!first_frames.is_empty());
    assert_eq!(first_frames, second_frames);
    assert_eq!(provider.call_count(), 1);
}

#[tokio::test]
async fn wendao_flight_service_get_flight_info_reuses_cached_search_family_payload() {
    let provider = Arc::new(RecordingSearchProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(provider.clone());
    });
    let descriptor = route_descriptor(SEARCH_INTENT_ROUTE);

    let mut first_request = Request::new(descriptor.clone());
    populate_schema_and_search_headers(first_request.metadata_mut(), "semantic-route", "5");
    let first_info = must_ok(
        service.get_flight_info(first_request).await,
        "first search-family route request should resolve",
    )
    .into_inner();

    let mut second_request = Request::new(descriptor);
    populate_schema_and_search_headers(second_request.metadata_mut(), "semantic-route", "5");
    let second_info = must_ok(
        service.get_flight_info(second_request).await,
        "second search-family route request should reuse the cached payload",
    )
    .into_inner();

    assert_eq!(provider.call_count(), 1);
    assert_eq!(first_info.total_records, second_info.total_records);
    assert_eq!(first_info.app_metadata, second_info.app_metadata);
}

#[tokio::test]
async fn wendao_flight_service_rejects_unconfigured_search_family_route() {
    let service = must_ok(
        WendaoFlightService::new_with_provider(
            "v2",
            Arc::new(RecordingRepoSearchProvider),
            3,
            RerankScoreWeights::default(),
        ),
        "service should build",
    );
    let mut request = Request::new(route_descriptor(SEARCH_INTENT_ROUTE));
    populate_schema_and_search_headers(request.metadata_mut(), "semantic-route", "4");

    let error = must_err(
        service.get_flight_info(request).await,
        "unconfigured search-family route should fail",
    );

    assert_eq!(error.code(), tonic::Code::Unimplemented);
    assert_eq!(
        error.message(),
        "search Flight route `/search/intent` is not configured for this runtime host"
    );
}
