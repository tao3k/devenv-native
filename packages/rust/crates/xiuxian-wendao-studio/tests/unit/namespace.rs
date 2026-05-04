#[cfg(feature = "openapi-artifacts")]
use xiuxian_wendao_studio::openapi::{
    bundled_wendao_gateway_openapi_path, load_bundled_wendao_gateway_openapi_document,
};

#[cfg(feature = "openapi-artifacts")]
#[test]
fn exposes_openapi_document_namespace() {
    let document = load_bundled_wendao_gateway_openapi_document().unwrap_or_else(|error| {
        panic!("bundled Wendao gateway OpenAPI document should parse: {error}")
    });

    assert_eq!(document["openapi"].as_str(), Some("3.1.0"));
    let path = bundled_wendao_gateway_openapi_path();
    assert!(path.is_file(), "bundled OpenAPI path should exist");
}

#[cfg(feature = "contracts")]
#[test]
fn exposes_gateway_route_contract_namespace() {
    use xiuxian_wendao_studio::openapi::WENDAO_GATEWAY_ROUTE_CONTRACTS;

    assert!(
        WENDAO_GATEWAY_ROUTE_CONTRACTS
            .iter()
            .any(|contract| contract.axum_path == "/api/health"),
        "gateway route contracts should expose the health route"
    );
}

#[cfg(feature = "studio")]
#[test]
fn exposes_studio_namespace_when_enabled() {
    assert_eq!(
        std::any::type_name::<xiuxian_wendao_studio::studio::StudioState>(),
        "xiuxian_wendao_studio::studio::router::state::types::StudioState"
    );
}
