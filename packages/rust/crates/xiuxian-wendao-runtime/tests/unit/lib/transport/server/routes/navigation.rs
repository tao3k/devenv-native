use std::sync::Arc;

use arrow_flight::flight_service_server::FlightService;
use tonic::Request;

use crate::transport::{GRAPH_NEIGHBORS_ROUTE, VFS_CONTENT_ROUTE, VFS_RESOLVE_ROUTE};

use crate::tests::transport::server::assertions::{
    must_ok, parse_json, route_descriptor, ticket_string,
};
use crate::tests::transport::server::fixtures::build_service_with_route_providers;
use crate::tests::transport::server::providers::{
    RecordingGraphNeighborsProvider, RecordingVfsContentProvider, RecordingVfsResolveProvider,
};
use crate::tests::transport::server::request_headers::{
    populate_schema_and_graph_neighbors_headers, populate_schema_and_vfs_content_headers,
    populate_schema_and_vfs_resolve_headers,
};

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_vfs_resolve_provider() {
    let provider = Arc::new(RecordingVfsResolveProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.vfs_resolve = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(VFS_RESOLVE_ROUTE));
    populate_schema_and_vfs_resolve_headers(request.metadata_mut(), "main/docs/index.md");

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "VFS resolve route should resolve through the pluggable provider",
    )
    .into_inner();
    let ticket = ticket_string(&flight_info, "VFS resolve route should emit one ticket");
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");

    assert_eq!(ticket, VFS_RESOLVE_ROUTE);
    assert_eq!(app_metadata["path"], "main/docs/index.md");
    assert_eq!(provider.call_count(), 1);
    assert_eq!(
        provider.recorded_request(),
        Some("main/docs/index.md".to_string())
    );
}

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_vfs_content_provider() {
    let provider = Arc::new(RecordingVfsContentProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.vfs_content = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(VFS_CONTENT_ROUTE));
    populate_schema_and_vfs_content_headers(request.metadata_mut(), "main/docs/index.md");

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "VFS content route should resolve through the pluggable provider",
    )
    .into_inner();
    let ticket = ticket_string(&flight_info, "VFS content route should emit one ticket");
    let app_metadata = parse_json(&flight_info.app_metadata, "app_metadata should decode");

    assert_eq!(ticket, VFS_CONTENT_ROUTE);
    assert_eq!(app_metadata["path"], "main/docs/index.md");
    assert_eq!(app_metadata["contentType"], "text/plain");
    assert_eq!(provider.call_count(), 1);
    assert_eq!(
        provider.recorded_request(),
        Some("main/docs/index.md".to_string())
    );
}

#[tokio::test]
async fn wendao_flight_service_get_flight_info_uses_graph_neighbors_provider() {
    let provider = Arc::new(RecordingGraphNeighborsProvider::default());
    let service = build_service_with_route_providers(|route_providers| {
        route_providers.graph_neighbors = Some(provider.clone());
    });
    let mut request = Request::new(route_descriptor(GRAPH_NEIGHBORS_ROUTE));
    populate_schema_and_graph_neighbors_headers(
        request.metadata_mut(),
        "kernel/docs/index.md",
        Some("incoming"),
        Some("3"),
        Some("25"),
    );

    let flight_info = must_ok(
        service.get_flight_info(request).await,
        "graph-neighbors route should resolve through the pluggable provider",
    )
    .into_inner();
    let ticket = ticket_string(&flight_info, "graph-neighbors route should emit one ticket");

    assert_eq!(ticket, GRAPH_NEIGHBORS_ROUTE);
    assert_eq!(
        provider.recorded_request(),
        Some((
            "kernel/docs/index.md".to_string(),
            "incoming".to_string(),
            3,
            25,
        ))
    );
    assert_eq!(provider.call_count(), 1);
}
