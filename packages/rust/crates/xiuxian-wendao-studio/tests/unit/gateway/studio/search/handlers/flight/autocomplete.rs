use crate::transport::SEARCH_AUTOCOMPLETE_ROUTE;

use super::{
    assert_route_ticket, build_service, make_gateway_state_with_docs, populate_autocomplete_headers,
};

#[tokio::test]
async fn build_studio_search_flight_service_wires_autocomplete_routes() {
    let fixture = make_gateway_state_with_docs(&[(
        "packages/rust/crates/demo/src/lib.rs",
        "pub struct AlphaService;\npub fn alpha_handler() {}\n",
    )])
    .await;
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        SEARCH_AUTOCOMPLETE_ROUTE,
        "autocomplete route",
        |metadata| populate_autocomplete_headers(metadata, "Alpha", 5),
    )
    .await;
}
