use crate::transport::{
    GRAPH_NEIGHBORS_ROUTE, SEARCH_ATTACHMENTS_ROUTE, SEARCH_AUTOCOMPLETE_ROUTE,
    SEARCH_DEFINITION_ROUTE, SEARCH_INTENT_ROUTE, SEARCH_KNOWLEDGE_ROUTE, SEARCH_REFERENCES_ROUTE,
    SEARCH_SYMBOLS_ROUTE, TOPOLOGY_3D_ROUTE, VFS_CONTENT_ROUTE, VFS_RESOLVE_ROUTE, VFS_SCAN_ROUTE,
    WendaoFlightService,
};
use serde_json::json;

use super::{
    assert_studio_flight_snapshot, build_service, fetch_flight_info, first_ticket,
    make_gateway_state_with_search_routes, populate_attachment_headers,
    populate_autocomplete_headers, populate_definition_headers, populate_graph_neighbors_headers,
    populate_search_headers, populate_topology_3d_headers, populate_vfs_content_headers,
    populate_vfs_resolve_headers, populate_vfs_scan_headers,
};

async fn snapshot_search_route_contract(
    service: &WendaoFlightService,
    route: &str,
    query_text: &str,
    limit: usize,
) -> serde_json::Value {
    let (descriptor_path, flight_info) = fetch_flight_info(service, route, |metadata| {
        populate_search_headers(metadata, query_text, limit);
    })
    .await;
    let ticket = first_ticket(&flight_info, route);

    json!({
        "route": route,
        "descriptorPath": descriptor_path,
        "query": query_text,
        "limit": limit,
        "ticket": ticket,
        "endpointCount": flight_info.endpoint.len(),
        "schemaLength": flight_info.schema.len(),
    })
}

async fn snapshot_attachment_route_contract(
    service: &WendaoFlightService,
    query_text: &str,
    limit: usize,
) -> serde_json::Value {
    let (descriptor_path, flight_info) =
        fetch_flight_info(service, SEARCH_ATTACHMENTS_ROUTE, |metadata| {
            populate_attachment_headers(metadata, query_text, limit);
        })
        .await;
    let ticket = first_ticket(&flight_info, SEARCH_ATTACHMENTS_ROUTE);

    json!({
        "route": SEARCH_ATTACHMENTS_ROUTE,
        "descriptorPath": descriptor_path,
        "query": query_text,
        "limit": limit,
        "extFilters": ["png"],
        "kindFilters": ["image"],
        "ticket": ticket,
        "endpointCount": flight_info.endpoint.len(),
        "schemaLength": flight_info.schema.len(),
    })
}

async fn snapshot_definition_route_contract(
    service: &WendaoFlightService,
    query_text: &str,
    source_path: &str,
    source_line: usize,
) -> serde_json::Value {
    let (descriptor_path, flight_info) =
        fetch_flight_info(service, SEARCH_DEFINITION_ROUTE, |metadata| {
            populate_definition_headers(metadata, query_text, source_path, source_line);
        })
        .await;
    let ticket = first_ticket(&flight_info, SEARCH_DEFINITION_ROUTE);

    json!({
        "route": SEARCH_DEFINITION_ROUTE,
        "descriptorPath": descriptor_path,
        "query": query_text,
        "sourcePath": source_path,
        "sourceLine": source_line,
        "ticket": ticket,
        "endpointCount": flight_info.endpoint.len(),
        "schemaLength": flight_info.schema.len(),
    })
}

async fn snapshot_autocomplete_route_contract(
    service: &WendaoFlightService,
    prefix: &str,
    limit: usize,
) -> serde_json::Value {
    let (descriptor_path, flight_info) =
        fetch_flight_info(service, SEARCH_AUTOCOMPLETE_ROUTE, |metadata| {
            populate_autocomplete_headers(metadata, prefix, limit);
        })
        .await;
    let ticket = first_ticket(&flight_info, SEARCH_AUTOCOMPLETE_ROUTE);

    json!({
        "route": SEARCH_AUTOCOMPLETE_ROUTE,
        "descriptorPath": descriptor_path,
        "prefix": prefix,
        "limit": limit,
        "ticket": ticket,
        "endpointCount": flight_info.endpoint.len(),
        "schemaLength": flight_info.schema.len(),
    })
}

async fn snapshot_vfs_route_contract(
    service: &WendaoFlightService,
    route: &str,
    path: Option<&str>,
) -> serde_json::Value {
    let (descriptor_path, flight_info) =
        fetch_flight_info(service, route, |metadata| match (route, path) {
            (VFS_RESOLVE_ROUTE, Some(path)) => populate_vfs_resolve_headers(metadata, path),
            (VFS_CONTENT_ROUTE, Some(path)) => populate_vfs_content_headers(metadata, path),
            (VFS_SCAN_ROUTE, None) => populate_vfs_scan_headers(metadata),
            _ => panic!("unsupported VFS route contract request for `{route}`"),
        })
        .await;
    let ticket = first_ticket(&flight_info, route);
    let mut snapshot = json!({
        "route": route,
        "descriptorPath": descriptor_path,
        "ticket": ticket,
        "endpointCount": flight_info.endpoint.len(),
        "schemaLength": flight_info.schema.len(),
    });
    if let Some(path) = path {
        snapshot["path"] = json!(path);
    }
    snapshot
}

async fn snapshot_graph_neighbors_route_contract(
    service: &WendaoFlightService,
) -> serde_json::Value {
    let (descriptor_path, flight_info) =
        fetch_flight_info(service, GRAPH_NEIGHBORS_ROUTE, |metadata| {
            populate_graph_neighbors_headers(metadata, "kernel/docs/alpha.md", "both", 1, 20);
        })
        .await;
    let ticket = first_ticket(&flight_info, GRAPH_NEIGHBORS_ROUTE);

    json!({
        "route": GRAPH_NEIGHBORS_ROUTE,
        "descriptorPath": descriptor_path,
        "nodeId": "kernel/docs/alpha.md",
        "direction": "both",
        "hops": 1,
        "limit": 20,
        "ticket": ticket,
        "endpointCount": flight_info.endpoint.len(),
        "schemaLength": flight_info.schema.len(),
    })
}

async fn snapshot_topology_3d_route_contract(service: &WendaoFlightService) -> serde_json::Value {
    let (descriptor_path, flight_info) =
        fetch_flight_info(service, TOPOLOGY_3D_ROUTE, populate_topology_3d_headers).await;
    let ticket = first_ticket(&flight_info, TOPOLOGY_3D_ROUTE);

    json!({
        "route": TOPOLOGY_3D_ROUTE,
        "descriptorPath": descriptor_path,
        "ticket": ticket,
        "endpointCount": flight_info.endpoint.len(),
        "schemaLength": flight_info.schema.len(),
    })
}

#[tokio::test]
async fn build_studio_search_flight_service_snapshots_search_route_contracts() {
    let fixture = make_gateway_state_with_search_routes().await;
    let service = build_service(fixture.state.clone());

    let snapshot = json!([
        snapshot_search_route_contract(&service, SEARCH_INTENT_ROUTE, "alpha", 5).await,
        snapshot_search_route_contract(&service, SEARCH_KNOWLEDGE_ROUTE, "Alpha body", 5).await,
        snapshot_attachment_route_contract(&service, "topology", 5).await,
        snapshot_search_route_contract(&service, SEARCH_REFERENCES_ROUTE, "AlphaService", 5).await,
        snapshot_search_route_contract(&service, SEARCH_SYMBOLS_ROUTE, "alpha", 5).await,
        snapshot_definition_route_contract(
            &service,
            "AlphaService",
            "packages/rust/crates/demo/src/lib.rs",
            2,
        )
        .await,
        snapshot_autocomplete_route_contract(&service, "Alpha", 5).await,
    ]);
    assert_studio_flight_snapshot("search_flight_service_route_contracts", snapshot);
}

#[tokio::test]
async fn build_studio_search_flight_service_snapshots_workspace_route_contracts() {
    let fixture = make_gateway_state_with_search_routes().await;
    let service = build_service(fixture.state.clone());

    let snapshot = json!([
        snapshot_vfs_route_contract(&service, VFS_RESOLVE_ROUTE, Some("kernel/docs/alpha.md"))
            .await,
        snapshot_vfs_route_contract(&service, VFS_CONTENT_ROUTE, Some("kernel/docs/alpha.md"))
            .await,
        snapshot_vfs_route_contract(&service, VFS_SCAN_ROUTE, None).await,
        snapshot_graph_neighbors_route_contract(&service).await,
        snapshot_topology_3d_route_contract(&service).await,
    ]);
    assert_studio_flight_snapshot("workspace_flight_service_route_contracts", snapshot);
}
