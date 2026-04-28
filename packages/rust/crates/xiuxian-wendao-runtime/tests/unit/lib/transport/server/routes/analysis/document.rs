use std::sync::Arc;

use arrow_array::{Int32Array, StringArray};
use arrow_flight::flight_service_server::FlightService;
use futures::StreamExt;
use tonic::Request;

use crate::transport::ANALYSIS_DOCUMENT_EXTRACT_ROUTE;

use super::super::super::assertions::{batch_column, must_err, must_ok, route_descriptor};
use super::super::super::fixtures::{build_service_with_route_providers, decode_flight_batches};
use super::super::super::providers::{RecordingDocumentExtractProvider, RecordingSearchProvider};
use super::super::super::request_headers::populate_schema_and_document_extract_headers;

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_document_extract_provider() {
    let provider = Arc::new(RecordingDocumentExtractProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.document_extract = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(ANALYSIS_DOCUMENT_EXTRACT_ROUTE));
    populate_schema_and_document_extract_headers(
        request.metadata_mut(),
        "docs/manual.pdf",
        Some(".cache/document-extract"),
        Some("true"),
        Some("false"),
    );

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "document extraction route should resolve through the dedicated provider",
    )
    .into_inner();

    assert_eq!(flight_info.endpoint.len(), 1);
    assert_eq!(
        provider.recorded_request(),
        Some((
            "docs/manual.pdf".to_string(),
            ".cache/document-extract".to_string(),
            true,
            false,
        ))
    );
    assert_eq!(provider.call_count(), 1);
}

#[tokio::test]
async fn wendao_flight_service_do_get_reuses_cached_document_extract_payload_after_get_flight_info()
{
    let provider = Arc::new(RecordingDocumentExtractProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.document_extract = Some(provider.clone());
    });
    let mut flight_info_request = Request::new(route_descriptor(ANALYSIS_DOCUMENT_EXTRACT_ROUTE));
    populate_schema_and_document_extract_headers(
        flight_info_request.metadata_mut(),
        "docs/manual.pdf",
        Some(".cache/document-extract"),
        Some("yes"),
        Some("no"),
    );
    let flight_info = must_ok(
        service.get_flight_info(flight_info_request).await,
        "document extraction route should resolve through the dedicated provider",
    )
    .into_inner();
    let ticket = flight_info
        .endpoint
        .first()
        .and_then(|endpoint| endpoint.ticket.clone())
        .unwrap_or_else(|| panic!("document extraction route should emit one ticket"));

    let mut do_get_request = Request::new(ticket);
    populate_schema_and_document_extract_headers(
        do_get_request.metadata_mut(),
        "docs/manual.pdf",
        Some(".cache/document-extract"),
        Some("yes"),
        Some("no"),
    );
    let frames = must_ok(
        service.do_get(do_get_request).await,
        "document extraction route should reuse the cached payload",
    )
    .into_inner()
    .collect::<Vec<_>>()
    .await;
    let batches = decode_flight_batches(frames).await;

    assert_eq!(provider.call_count(), 1);
    assert_eq!(batches.len(), 1);
    let source_paths = batch_column::<StringArray>(
        &batches[0],
        "sourcePath",
        "document extraction batch should include sourcePath",
    );
    let page_indexes = batch_column::<Int32Array>(
        &batches[0],
        "pageIndex",
        "document extraction batch should include pageIndex",
    );
    assert_eq!(source_paths.value(0), "docs/manual.pdf");
    assert_eq!(page_indexes.value(0), 0);
}

#[tokio::test]
async fn wendao_flight_service_rejects_unconfigured_document_extract_route() {
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(Arc::new(RecordingSearchProvider::default()));
    });
    let mut request = Request::new(route_descriptor(ANALYSIS_DOCUMENT_EXTRACT_ROUTE));
    populate_schema_and_document_extract_headers(
        request.metadata_mut(),
        "docs/manual.pdf",
        Some(".cache/document-extract"),
        None,
        None,
    );

    let error = must_err(
        service.get_flight_info(request).await,
        "unconfigured document extraction route should fail",
    );

    assert_eq!(error.code(), tonic::Code::Unimplemented);
    assert_eq!(
        error.message(),
        "document extract Flight route `/analysis/document-extract` is not configured for this runtime host"
    );
}
