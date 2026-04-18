use std::sync::Arc;

use arrow_flight::flight_service_server::FlightService;
use tonic::Request;

use crate::transport::{ANALYSIS_REPO_INDEX_STATUS_ROUTE, ANALYSIS_REPO_SYNC_ROUTE};

use super::super::super::assertions::{
    must_err, must_ok, parse_json, route_descriptor, ticket_string,
};
use super::super::super::fixtures::build_service_with_route_providers;
use super::super::super::providers::{
    RecordingRepoIndexStatusProvider, RecordingRepoSyncProvider, RecordingSearchProvider,
};
use super::super::super::request_headers::{
    populate_schema_and_repo_index_status_headers, populate_schema_and_repo_sync_headers,
};

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_repo_index_status_provider() {
    let provider = Arc::new(RecordingRepoIndexStatusProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.repo_index_status = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(ANALYSIS_REPO_INDEX_STATUS_ROUTE));
    populate_schema_and_repo_index_status_headers(request.metadata_mut(), Some("gateway-sync"));

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "repo index status route should resolve through the dedicated provider",
    )
    .into_inner();
    let ticket = ticket_string(
        &flight_info,
        "repo index status route should emit one ticket",
    );

    assert_eq!(ticket, ANALYSIS_REPO_INDEX_STATUS_ROUTE);
    assert_eq!(
        provider.recorded_request(),
        Some(Some("gateway-sync".to_string()))
    );
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");
    assert_eq!(app_metadata["total"], 3);
    assert_eq!(app_metadata["targetConcurrency"], 2);
}

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_repo_sync_provider() {
    let provider = Arc::new(RecordingRepoSyncProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.repo_sync = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(ANALYSIS_REPO_SYNC_ROUTE));
    populate_schema_and_repo_sync_headers(request.metadata_mut(), "gateway-sync", Some("status"));

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "repo sync route should resolve through the dedicated provider",
    )
    .into_inner();
    let ticket = ticket_string(&flight_info, "repo sync route should emit one ticket");

    assert_eq!(ticket, ANALYSIS_REPO_SYNC_ROUTE);
    assert_eq!(
        provider.recorded_request(),
        Some(("gateway-sync".to_string(), "status".to_string()))
    );
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");
    assert_eq!(app_metadata["repoId"], "gateway-sync");
    assert_eq!(app_metadata["mode"], "status");
    assert_eq!(app_metadata["healthState"], "healthy");
}

#[tokio::test]
async fn wendao_flight_service_rejects_unconfigured_repo_index_status_route() {
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(Arc::new(RecordingSearchProvider::default()));
    });
    let mut request = Request::new(route_descriptor(ANALYSIS_REPO_INDEX_STATUS_ROUTE));
    populate_schema_and_repo_index_status_headers(request.metadata_mut(), None);

    let error = must_err(
        service.get_flight_info(request).await,
        "unconfigured repo index status route should fail",
    );

    assert_eq!(error.code(), tonic::Code::Unimplemented);
    assert_eq!(
        error.message(),
        "repo index status Flight route `/analysis/repo-index-status` is not configured for this runtime host"
    );
}

#[tokio::test]
async fn wendao_flight_service_rejects_unconfigured_repo_sync_route() {
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(Arc::new(RecordingSearchProvider::default()));
    });
    let mut request = Request::new(route_descriptor(ANALYSIS_REPO_SYNC_ROUTE));
    populate_schema_and_repo_sync_headers(request.metadata_mut(), "gateway-sync", Some("status"));

    let error = must_err(
        service.get_flight_info(request).await,
        "unconfigured repo sync route should fail",
    );

    assert_eq!(error.code(), tonic::Code::Unimplemented);
    assert_eq!(
        error.message(),
        "repo sync Flight route `/analysis/repo-sync` is not configured for this runtime host"
    );
}
