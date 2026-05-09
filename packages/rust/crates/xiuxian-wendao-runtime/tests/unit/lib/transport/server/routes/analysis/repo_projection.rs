use std::sync::Arc;

use arrow_array::{Int32Array, StringArray};
use arrow_flight::flight_service_server::FlightService;
use futures::StreamExt;
use tonic::Request;

use crate::transport::ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE;

use crate::tests::transport::server::assertions::{
    batch_column, must_err, must_ok, parse_json, route_descriptor, ticket_string,
};
use crate::tests::transport::server::fixtures::{
    build_service_with_route_providers, decode_flight_batches,
};
use crate::tests::transport::server::providers::{
    RecordingRepoProjectedRetrievalContextProvider, RecordingSearchProvider,
};
use crate::tests::transport::server::request_headers::{
    build_repo_projected_retrieval_context_metadata,
    populate_schema_and_repo_projected_retrieval_context_headers,
};

const REPO_ID: &str = "gateway-sync";
const PAGE_ID: &str = "repo:gateway-sync:projection:reference:doc:docs/solve.md";
const NODE_ID: &str = "repo:gateway-sync:reference:docs/solve.md:12";

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_repo_projected_retrieval_context_provider() {
    let provider = Arc::new(RecordingRepoProjectedRetrievalContextProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.repo_projected_retrieval_context = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(
        ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    ));
    populate_schema_and_repo_projected_retrieval_context_headers(
        request.metadata_mut(),
        REPO_ID,
        PAGE_ID,
        Some(NODE_ID),
        Some("7"),
    );

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "repo projected retrieval-context route should resolve through the dedicated provider",
    )
    .into_inner();
    let ticket = ticket_string(
        &flight_info,
        "repo projected retrieval-context route should emit one ticket",
    );

    assert_eq!(ticket, ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE);
    assert_eq!(
        provider.recorded_request(),
        Some((
            REPO_ID.to_string(),
            PAGE_ID.to_string(),
            Some(NODE_ID.to_string()),
            7,
        ))
    );
    assert_eq!(provider.call_count(), 1);
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");
    assert_eq!(app_metadata["repoId"], REPO_ID);
    assert_eq!(app_metadata["pageId"], PAGE_ID);
    assert_eq!(app_metadata["nodeId"], NODE_ID);
    assert_eq!(app_metadata["relatedCount"], 7);
    assert_eq!(app_metadata["hasNodeContext"], true);
}

#[tokio::test]
async fn wendao_flight_service_do_get_reuses_cached_repo_projected_retrieval_context_payload_after_get_flight_info()
 {
    let provider = Arc::new(RecordingRepoProjectedRetrievalContextProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.repo_projected_retrieval_context = Some(provider.clone());
    });
    let mut flight_info_request = Request::new(route_descriptor(
        ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    ));
    populate_schema_and_repo_projected_retrieval_context_headers(
        flight_info_request.metadata_mut(),
        REPO_ID,
        PAGE_ID,
        Some(NODE_ID),
        Some("7"),
    );
    let flight_info = must_ok(
        service.get_flight_info(flight_info_request).await,
        "repo projected retrieval-context route should resolve through the dedicated provider",
    )
    .into_inner();
    let ticket = flight_info
        .endpoint
        .first()
        .and_then(|endpoint| endpoint.ticket.clone())
        .unwrap_or_else(|| panic!("repo projected retrieval-context route should emit one ticket"));

    let mut do_get_request = Request::new(ticket);
    populate_schema_and_repo_projected_retrieval_context_headers(
        do_get_request.metadata_mut(),
        REPO_ID,
        PAGE_ID,
        Some(NODE_ID),
        Some("7"),
    );
    let frames = must_ok(
        service.do_get(do_get_request).await,
        "repo projected retrieval-context route should reuse the cached payload",
    )
    .into_inner()
    .collect::<Vec<_>>()
    .await;
    let batches = decode_flight_batches(frames).await;

    assert_eq!(provider.call_count(), 1);
    assert_eq!(batches.len(), 1);
    let repo_ids = batch_column::<StringArray>(
        &batches[0],
        "repoId",
        "projected retrieval-context batch should include repoId",
    );
    let page_ids = batch_column::<StringArray>(
        &batches[0],
        "pageId",
        "projected retrieval-context batch should include pageId",
    );
    let node_ids = batch_column::<StringArray>(
        &batches[0],
        "nodeId",
        "projected retrieval-context batch should include nodeId",
    );
    let related_counts = batch_column::<Int32Array>(
        &batches[0],
        "relatedCount",
        "projected retrieval-context batch should include relatedCount",
    );
    assert_eq!(repo_ids.value(0), REPO_ID);
    assert_eq!(page_ids.value(0), PAGE_ID);
    assert_eq!(node_ids.value(0), NODE_ID);
    assert_eq!(related_counts.value(0), 7);
}

#[tokio::test]
async fn wendao_flight_service_rejects_unconfigured_repo_projected_retrieval_context_route() {
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(Arc::new(RecordingSearchProvider::default()));
    });
    let mut request = Request::new(route_descriptor(
        ANALYSIS_REPO_PROJECTED_RETRIEVAL_CONTEXT_ROUTE,
    ));
    *request.metadata_mut() =
        build_repo_projected_retrieval_context_metadata(REPO_ID, PAGE_ID, Some(NODE_ID), Some("7"));

    let error = must_err(
        service.get_flight_info(request).await,
        "unconfigured repo projected retrieval-context route should fail",
    );

    assert_eq!(error.code(), tonic::Code::Unimplemented);
    assert_eq!(
        error.message(),
        "repo projected retrieval-context Flight route `/analysis/repo-projected-retrieval-context` is not configured for this runtime host"
    );
}
