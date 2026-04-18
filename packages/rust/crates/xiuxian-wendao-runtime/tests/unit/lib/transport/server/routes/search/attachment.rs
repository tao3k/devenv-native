use std::sync::Arc;

use arrow_flight::flight_service_server::FlightService;
use tonic::Request;

use crate::transport::SEARCH_ATTACHMENTS_ROUTE;

use super::super::super::assertions::{
    must_err, must_ok, parse_json, route_descriptor, ticket_string,
};
use super::super::super::fixtures::build_service_with_route_providers;
use super::super::super::providers::{RecordingAttachmentSearchProvider, RecordingSearchProvider};
use super::super::super::request_headers::populate_schema_and_attachment_search_headers;

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_attachment_search_provider() {
    let provider = Arc::new(RecordingAttachmentSearchProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.attachment_search = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(SEARCH_ATTACHMENTS_ROUTE));
    populate_schema_and_attachment_search_headers(
        request.metadata_mut(),
        "image",
        "4",
        Some("png,jpg"),
        Some("image,screenshot"),
        Some("true"),
    );

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "attachment-search route should resolve through the pluggable provider",
    )
    .into_inner();
    let ticket = ticket_string(
        &flight_info,
        "attachment-search route should emit one ticket",
    );
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");

    assert_eq!(ticket, SEARCH_ATTACHMENTS_ROUTE);
    assert_eq!(app_metadata["query"], "image");
    assert_eq!(app_metadata["selectedScope"], "attachments");
    assert_eq!(
        provider.recorded_request(),
        Some((
            "image".to_string(),
            4,
            vec!["jpg".to_string(), "png".to_string()],
            vec!["image".to_string(), "screenshot".to_string()],
            true,
        ))
    );
}

#[tokio::test]
async fn wendao_flight_service_rejects_unconfigured_attachment_search_route() {
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.search = Some(Arc::new(RecordingSearchProvider::default()));
    });
    let mut request = Request::new(route_descriptor(SEARCH_ATTACHMENTS_ROUTE));
    populate_schema_and_attachment_search_headers(
        request.metadata_mut(),
        "image",
        "4",
        Some("png"),
        Some("image"),
        Some("false"),
    );

    let error = must_err(
        service.get_flight_info(request).await,
        "unconfigured attachment-search route should fail",
    );

    assert_eq!(error.code(), tonic::Code::Unimplemented);
    assert_eq!(
        error.message(),
        "attachment-search Flight route `/search/attachments` is not configured for this runtime host"
    );
}
