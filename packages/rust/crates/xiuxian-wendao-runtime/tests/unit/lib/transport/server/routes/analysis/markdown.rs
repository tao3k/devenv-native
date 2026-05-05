use std::sync::Arc;

use arrow_flight::flight_service_server::FlightService;
use futures::StreamExt;
use tonic::Request;

use crate::transport::ANALYSIS_MARKDOWN_ROUTE;

use crate::tests::transport::server::assertions::{
    must_err, must_ok, parse_json, route_descriptor, ticket_string,
};
use crate::tests::transport::server::fixtures::{
    build_service_with_route_providers, decode_flight_batches,
};
use crate::tests::transport::server::providers::{
    RecordingMarkdownAnalysisProvider, RecordingSearchProvider,
};
use crate::tests::transport::server::request_headers::populate_schema_and_markdown_analysis_headers;

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_markdown_analysis_provider() {
    let provider = Arc::new(RecordingMarkdownAnalysisProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.markdown_analysis = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(ANALYSIS_MARKDOWN_ROUTE));
    populate_schema_and_markdown_analysis_headers(request.metadata_mut(), "docs/analysis.md");

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "markdown analysis route should resolve through the dedicated provider",
    )
    .into_inner();
    let ticket = ticket_string(
        &flight_info,
        "markdown analysis route should emit one ticket",
    );

    assert_eq!(ticket, ANALYSIS_MARKDOWN_ROUTE);
    assert_eq!(
        provider.recorded_request(),
        Some("docs/analysis.md".to_string())
    );
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");
    assert_eq!(app_metadata["path"], "docs/analysis.md");
    assert_eq!(app_metadata["documentHash"], "fp:markdown");
    assert_eq!(app_metadata["nodeCount"], 1);
    assert_eq!(app_metadata["edgeCount"], 0);
    assert_eq!(provider.call_count(), 1);
}

#[tokio::test]
async fn wendao_flight_service_do_get_reuses_cached_markdown_analysis_payload_after_get_flight_info()
 {
    let provider = Arc::new(RecordingMarkdownAnalysisProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.markdown_analysis = Some(provider.clone());
    });
    let mut flight_info_request = Request::new(route_descriptor(ANALYSIS_MARKDOWN_ROUTE));
    populate_schema_and_markdown_analysis_headers(
        flight_info_request.metadata_mut(),
        "docs/analysis.md",
    );
    let flight_info = must_ok(
        service.get_flight_info(flight_info_request).await,
        "markdown analysis route should resolve through the dedicated provider",
    )
    .into_inner();
    let ticket = flight_info
        .endpoint
        .first()
        .and_then(|endpoint| endpoint.ticket.clone())
        .unwrap_or_else(|| panic!("markdown analysis route should emit one ticket"));

    let mut do_get_request = Request::new(ticket);
    populate_schema_and_markdown_analysis_headers(
        do_get_request.metadata_mut(),
        "docs/analysis.md",
    );
    let frames = must_ok(
        service.do_get(do_get_request).await,
        "markdown analysis route should reuse the cached payload",
    )
    .into_inner()
    .collect::<Vec<_>>()
    .await;
    let batches = decode_flight_batches(frames).await;

    assert_eq!(provider.call_count(), 1);
    assert_eq!(
        provider.recorded_request(),
        Some("docs/analysis.md".to_string())
    );
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].num_rows(), 1);
}

#[tokio::test]
async fn wendao_flight_service_rejects_unconfigured_markdown_analysis_route() {
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(Arc::new(RecordingSearchProvider::default()));
    });
    let mut request = Request::new(route_descriptor(ANALYSIS_MARKDOWN_ROUTE));
    populate_schema_and_markdown_analysis_headers(request.metadata_mut(), "docs/analysis.md");

    let error = must_err(
        service.get_flight_info(request).await,
        "unconfigured markdown analysis route should fail",
    );

    assert_eq!(error.code(), tonic::Code::Unimplemented);
    assert_eq!(
        error.message(),
        "markdown analysis Flight route `/analysis/markdown` is not configured for this runtime host"
    );
}
