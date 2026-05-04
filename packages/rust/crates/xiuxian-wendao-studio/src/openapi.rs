//! Stable OpenAPI contract exports for the Wendao gateway.

pub use crate::contracts::{RouteContract, WENDAO_GATEWAY_ROUTE_CONTRACTS};
#[cfg(feature = "openapi-artifacts")]
pub use xiuxian_wendao_runtime::artifacts::openapi::{
    bundled_wendao_gateway_openapi_document, bundled_wendao_gateway_openapi_path,
    load_bundled_wendao_gateway_openapi_document,
};
