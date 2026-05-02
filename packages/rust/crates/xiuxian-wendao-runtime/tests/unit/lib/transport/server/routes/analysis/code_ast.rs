use std::sync::Arc;

use arrow_flight::flight_service_server::FlightService;
use tonic::Request;

use crate::transport::ANALYSIS_CODE_AST_ROUTE;

use crate::tests::transport::server::assertions::{
    must_err, must_ok, parse_json, route_descriptor, ticket_string,
};
use crate::tests::transport::server::fixtures::build_service_with_route_providers;
use crate::tests::transport::server::providers::{
    RecordingCodeAstAnalysisProvider, RecordingSearchProvider,
};
use crate::tests::transport::server::request_headers::populate_schema_and_code_ast_analysis_headers;

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_code_ast_analysis_provider() {
    let provider = Arc::new(RecordingCodeAstAnalysisProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.code_ast_analysis = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(ANALYSIS_CODE_AST_ROUTE));
    populate_schema_and_code_ast_analysis_headers(
        request.metadata_mut(),
        "src/lib.jl",
        "demo",
        Some("7"),
    );

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "code-AST analysis route should resolve through the dedicated provider",
    )
    .into_inner();
    let ticket = ticket_string(
        &flight_info,
        "code-AST analysis route should emit one ticket",
    );

    assert_eq!(ticket, ANALYSIS_CODE_AST_ROUTE);
    assert_eq!(
        provider.recorded_request(),
        Some(("src/lib.jl".to_string(), "demo".to_string(), Some(7)))
    );
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");
    assert_eq!(app_metadata["repoId"], "demo");
    assert_eq!(app_metadata["path"], "src/lib.jl");
    assert_eq!(app_metadata["language"], "julia");
    assert_eq!(app_metadata["nodeCount"], 1);
    assert_eq!(app_metadata["edgeCount"], 0);
    assert_eq!(app_metadata["focusNodeId"], "line:7");
}

#[tokio::test]
async fn wendao_flight_service_rejects_unconfigured_code_ast_analysis_route() {
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(Arc::new(RecordingSearchProvider::default()));
    });
    let mut request = Request::new(route_descriptor(ANALYSIS_CODE_AST_ROUTE));
    populate_schema_and_code_ast_analysis_headers(
        request.metadata_mut(),
        "src/lib.jl",
        "demo",
        Some("7"),
    );

    let error = must_err(
        service.get_flight_info(request).await,
        "unconfigured code-AST analysis route should fail",
    );

    assert_eq!(error.code(), tonic::Code::Unimplemented);
    assert_eq!(
        error.message(),
        "code-AST analysis Flight route `/analysis/code-ast` is not configured for this runtime host"
    );
}
