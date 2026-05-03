use crate::transport::{VFS_CONTENT_ROUTE, VFS_RESOLVE_ROUTE, VFS_SCAN_ROUTE};

use super::{
    assert_route_ticket, build_service, make_gateway_state_with_docs, populate_vfs_content_headers,
    populate_vfs_resolve_headers, populate_vfs_scan_headers,
};

#[tokio::test]
async fn build_studio_search_flight_service_wires_vfs_resolve_routes() {
    let fixture = make_gateway_state_with_docs(&[(
        "docs/index.md",
        "# Index\n\n- [Overview](overview.md)\n",
    )])
    .await;
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        VFS_RESOLVE_ROUTE,
        "VFS resolve route",
        |metadata| {
            populate_vfs_resolve_headers(metadata, "kernel/docs/index.md");
        },
    )
    .await;
}

#[tokio::test]
async fn build_studio_search_flight_service_wires_vfs_content_routes() {
    let fixture = make_gateway_state_with_docs(&[(
        "docs/index.md",
        "# Index\n\n- [Overview](overview.md)\n",
    )])
    .await;
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        VFS_CONTENT_ROUTE,
        "VFS content route",
        |metadata| {
            populate_vfs_content_headers(metadata, "kernel/docs/index.md");
        },
    )
    .await;
}

#[tokio::test]
async fn build_studio_search_flight_service_wires_vfs_scan_routes() {
    let fixture = make_gateway_state_with_docs(&[(
        "docs/index.md",
        "# Index\n\n- [Overview](overview.md)\n",
    )])
    .await;
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        VFS_SCAN_ROUTE,
        "VFS scan route",
        populate_vfs_scan_headers,
    )
    .await;
}
