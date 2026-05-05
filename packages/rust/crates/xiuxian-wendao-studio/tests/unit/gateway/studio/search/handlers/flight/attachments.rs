use crate::transport::SEARCH_ATTACHMENTS_ROUTE;

use super::{
    assert_route_ticket, build_service, make_gateway_state_with_attachments,
    populate_attachment_headers,
};

#[tokio::test]
async fn build_studio_search_flight_service_wires_attachment_routes() {
    let fixture = make_gateway_state_with_attachments().await;
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        SEARCH_ATTACHMENTS_ROUTE,
        "attachment route",
        |metadata| {
            populate_attachment_headers(metadata, "topology", 5);
        },
    )
    .await;
}
