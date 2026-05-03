use xiuxian_wendao_web::openapi::{
    WENDAO_GATEWAY_ROUTE_CONTRACTS, bundled_wendao_gateway_openapi_path,
    load_bundled_wendao_gateway_openapi_document,
};

#[test]
fn exposes_openapi_document_namespace() {
    let document = load_bundled_wendao_gateway_openapi_document().unwrap_or_else(|error| {
        panic!("bundled Wendao gateway OpenAPI document should parse: {error}")
    });

    assert_eq!(document["openapi"].as_str(), Some("3.1.0"));
    let path = bundled_wendao_gateway_openapi_path();
    assert!(path.is_file(), "bundled OpenAPI path should exist");
}

#[test]
fn exposes_gateway_route_contract_namespace() {
    assert!(
        WENDAO_GATEWAY_ROUTE_CONTRACTS
            .iter()
            .any(|contract| contract.axum_path == "/api/health"),
        "gateway route contracts should expose the health route"
    );
}
