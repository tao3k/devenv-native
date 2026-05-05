use std::sync::Arc;

use arrow_flight::flight_service_server::FlightService;
use tonic::Request;

use crate::transport::{ANALYSIS_REPO_DOC_COVERAGE_ROUTE, ANALYSIS_REPO_OVERVIEW_ROUTE};

use crate::tests::transport::server::assertions::{
    must_err, must_ok, parse_json, route_descriptor, ticket_string,
};
use crate::tests::transport::server::fixtures::build_service_with_route_providers;
use crate::tests::transport::server::providers::{
    RecordingRepoDocCoverageProvider, RecordingRepoOverviewProvider, RecordingSearchProvider,
};
use crate::tests::transport::server::request_headers::{
    populate_schema_and_repo_doc_coverage_headers, populate_schema_and_repo_overview_headers,
};

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_repo_doc_coverage_provider() {
    let provider = Arc::new(RecordingRepoDocCoverageProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.repo_doc_coverage = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(ANALYSIS_REPO_DOC_COVERAGE_ROUTE));
    populate_schema_and_repo_doc_coverage_headers(
        request.metadata_mut(),
        "gateway-sync",
        Some("GatewaySyncPkg"),
    );

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "repo doc coverage route should resolve through the dedicated provider",
    )
    .into_inner();
    let ticket = ticket_string(
        &flight_info,
        "repo doc coverage route should emit one ticket",
    );

    assert_eq!(ticket, ANALYSIS_REPO_DOC_COVERAGE_ROUTE);
    assert_eq!(
        provider.recorded_request(),
        Some((
            "gateway-sync".to_string(),
            Some("GatewaySyncPkg".to_string())
        ))
    );
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");
    assert_eq!(app_metadata["repoId"], "gateway-sync");
    assert_eq!(app_metadata["moduleId"], "GatewaySyncPkg");
    assert_eq!(app_metadata["coveredSymbols"], 3);
    assert_eq!(app_metadata["uncoveredSymbols"], 1);
}

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_repo_overview_provider() {
    let provider = Arc::new(RecordingRepoOverviewProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.repo_overview = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(ANALYSIS_REPO_OVERVIEW_ROUTE));
    populate_schema_and_repo_overview_headers(request.metadata_mut(), "gateway-sync");

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "repo overview route should resolve through the dedicated provider",
    )
    .into_inner();
    let ticket = ticket_string(&flight_info, "repo overview route should emit one ticket");

    assert_eq!(ticket, ANALYSIS_REPO_OVERVIEW_ROUTE);
    assert_eq!(
        provider.recorded_request(),
        Some("gateway-sync".to_string())
    );
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");
    assert_eq!(app_metadata["repoId"], "gateway-sync");
    assert_eq!(app_metadata["displayName"], "Gateway Sync");
    assert_eq!(app_metadata["moduleCount"], 3);
}

#[tokio::test]
async fn wendao_flight_service_rejects_unconfigured_repo_doc_coverage_route() {
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(Arc::new(RecordingSearchProvider::default()));
    });
    let mut request = Request::new(route_descriptor(ANALYSIS_REPO_DOC_COVERAGE_ROUTE));
    populate_schema_and_repo_doc_coverage_headers(
        request.metadata_mut(),
        "gateway-sync",
        Some("GatewaySyncPkg"),
    );

    let error = must_err(
        service.get_flight_info(request).await,
        "unconfigured repo doc coverage route should fail",
    );

    assert_eq!(error.code(), tonic::Code::Unimplemented);
    assert_eq!(
        error.message(),
        "repo doc coverage Flight route `/analysis/repo-doc-coverage` is not configured for this runtime host"
    );
}

#[tokio::test]
async fn wendao_flight_service_rejects_unconfigured_repo_overview_route() {
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(Arc::new(RecordingSearchProvider::default()));
    });
    let mut request = Request::new(route_descriptor(ANALYSIS_REPO_OVERVIEW_ROUTE));
    populate_schema_and_repo_overview_headers(request.metadata_mut(), "gateway-sync");

    let error = must_err(
        service.get_flight_info(request).await,
        "unconfigured repo overview route should fail",
    );

    assert_eq!(error.code(), tonic::Code::Unimplemented);
    assert_eq!(
        error.message(),
        "repo overview Flight route `/analysis/repo-overview` is not configured for this runtime host"
    );
}
