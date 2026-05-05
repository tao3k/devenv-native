use crate::transport::{GRAPH_NEIGHBORS_ROUTE, TOPOLOGY_3D_ROUTE};

use super::{
    assert_route_ticket, build_service, make_gateway_state_with_docs,
    populate_graph_neighbors_headers, populate_topology_3d_headers,
};

#[tokio::test]
async fn build_studio_search_flight_service_wires_graph_neighbors_routes() {
    let fixture = make_gateway_state_with_docs(&[
        ("docs/alpha.md", "# Alpha\n\nSee [[beta]].\n"),
        ("docs/beta.md", "# Beta\n\nBody.\n"),
    ])
    .await;
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        GRAPH_NEIGHBORS_ROUTE,
        "graph-neighbors route",
        |metadata| {
            populate_graph_neighbors_headers(metadata, "kernel/docs/alpha.md", "both", 1, 20);
        },
    )
    .await;
}

#[tokio::test]
async fn build_studio_search_flight_service_wires_topology_3d_routes() {
    let fixture = make_gateway_state_with_docs(&[
        ("docs/alpha.md", "# Alpha\n\nSee [[beta]].\n"),
        ("docs/beta.md", "# Beta\n\nBody.\n"),
    ])
    .await;
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        TOPOLOGY_3D_ROUTE,
        "topology-3d route",
        populate_topology_3d_headers,
    )
    .await;
}
