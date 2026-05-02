use std::sync::Arc;

use arrow_flight::flight_service_server::FlightService;
use tonic::Request;

use crate::transport::SEARCH_AST_ROUTE;

use crate::tests::transport::server::assertions::{
    must_err, must_ok, parse_json, route_descriptor, ticket_string,
};
use crate::tests::transport::server::fixtures::build_service_with_route_providers;
use crate::tests::transport::server::providers::{
    RecordingAstSearchProvider, RecordingSearchProvider,
};
use crate::tests::transport::server::request_headers::populate_schema_and_search_headers;

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_ast_search_provider() {
    let provider = Arc::new(RecordingAstSearchProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.ast_search = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(SEARCH_AST_ROUTE));
    populate_schema_and_search_headers(request.metadata_mut(), "symbol", "6");

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "AST route should resolve through the dedicated provider",
    )
    .into_inner();
    let ticket = ticket_string(&flight_info, "AST route should emit one ticket");
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");

    assert_eq!(ticket, SEARCH_AST_ROUTE);
    assert_eq!(app_metadata["query"], "symbol");
    assert_eq!(app_metadata["selectedScope"], "definitions");
    assert_eq!(provider.recorded_request(), Some(("symbol".to_string(), 6)));
}

#[tokio::test]
async fn wendao_flight_service_rejects_unconfigured_ast_search_route() {
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(Arc::new(RecordingSearchProvider::default()));
    });
    let mut request = Request::new(route_descriptor(SEARCH_AST_ROUTE));
    populate_schema_and_search_headers(request.metadata_mut(), "symbol", "6");

    let error = must_err(
        service.get_flight_info(request).await,
        "unconfigured AST route should fail",
    );

    assert_eq!(error.code(), tonic::Code::Unimplemented);
    assert_eq!(
        error.message(),
        "AST-search Flight route `/search/ast` is not configured for this runtime host"
    );
}
