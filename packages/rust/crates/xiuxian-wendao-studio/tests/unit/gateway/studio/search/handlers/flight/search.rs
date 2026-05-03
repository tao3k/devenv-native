use crate::transport::SEARCH_SYMBOLS_ROUTE;

use super::{
    assert_route_ticket, build_service, make_gateway_state_with_docs, populate_search_headers,
};

#[tokio::test]
async fn build_studio_search_flight_service_wires_search_routes() {
    let fixture = make_gateway_state_with_docs(&[(
        "packages/rust/crates/demo/src/lib.rs",
        "pub struct AlphaService;\npub fn alpha_handler() {}\n",
    )])
    .await;
    let service = build_service(fixture.state.clone());

    assert_route_ticket(&service, SEARCH_SYMBOLS_ROUTE, "search route", |metadata| {
        populate_search_headers(metadata, "alpha", 5);
    })
    .await;
}
