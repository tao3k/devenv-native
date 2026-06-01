mod flight;
mod http;
mod policy;
mod principal;

use super::{
    AUTHORIZATION, Arc, Body, CONTENT_TYPE, GatewayPublicProtocolSurface, GatewaySurfaceSecurity,
    Request, ServiceExt, StatusCode, WENDAO_AUTH_SCOPE_HEADER,
    WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER, WENDAO_PUBLIC_PROTOCOL_HEADER,
    WENDAO_SIGNED_PRINCIPAL_HEADER, app_state, build_gateway_router, post,
    with_gateway_surface_security,
};
use crate::bin_support::wendao::execute::gateway::security::GatewaySurfacePolicy;

#[cfg(feature = "zhenfa-router")]
use super::GATEWAY_FLIGHT_SERVICE_AXUM_PATH;

async fn echo_security_headers(headers: axum::http::HeaderMap) -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({
        "authorization": headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok()),
        "serviceIdentity": headers
            .get(WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER)
            .and_then(|value| value.to_str().ok()),
        "signedPrincipal": headers
            .get(WENDAO_SIGNED_PRINCIPAL_HEADER)
            .and_then(|value| value.to_str().ok()),
        "protocol": headers
            .get(WENDAO_PUBLIC_PROTOCOL_HEADER)
            .and_then(|value| value.to_str().ok()),
        "scope": headers
            .get(WENDAO_AUTH_SCOPE_HEADER)
            .and_then(|value| value.to_str().ok()),
    }))
}
