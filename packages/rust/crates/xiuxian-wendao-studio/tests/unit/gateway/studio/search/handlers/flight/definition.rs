use crate::transport::SEARCH_DEFINITION_ROUTE;

use super::{
    assert_route_ticket, build_service, make_gateway_state_with_docs, populate_definition_headers,
};

#[tokio::test]
async fn build_studio_search_flight_service_wires_definition_routes() {
    let fixture = make_gateway_state_with_docs(&[
        (
            "packages/rust/crates/demo/src/lib.rs",
            "pub fn build_service() {\n    let _service = AlphaService::new();\n}\n",
        ),
        (
            "packages/rust/crates/demo/src/service.rs",
            "pub struct AlphaService {\n    ready: bool,\n}\n",
        ),
    ])
    .await;
    let service = build_service(fixture.state.clone());

    assert_route_ticket(
        &service,
        SEARCH_DEFINITION_ROUTE,
        "definition route",
        |metadata| {
            populate_definition_headers(
                metadata,
                "AlphaService",
                "packages/rust/crates/demo/src/lib.rs",
                2,
            );
        },
    )
    .await;
}
