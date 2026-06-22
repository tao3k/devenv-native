use axum::http::header::CONTENT_LENGTH;

use super::{
    Body, GatewayPublicProtocolSurface, GatewaySurfacePolicy, GatewaySurfaceSecurity, Request,
    ServiceExt, StatusCode, echo_security_headers, post, with_gateway_surface_security,
};

#[tokio::test]
async fn test_gateway_surface_security_applies_surface_rate_limit() {
    let router = with_gateway_surface_security(
        super::super::Router::new().route("/internal", post(echo_security_headers)),
        GatewaySurfaceSecurity::new(GatewayPublicProtocolSurface::HttpsJsonSse, None)
            .with_policy(GatewaySurfacePolicy::new(1, 1024)),
    );

    let first = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer first request: {error}"));
    let second = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer second request: {error}"));

    assert_eq!(first.status(), StatusCode::OK);
    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
}

#[tokio::test]
async fn test_gateway_surface_security_rejects_declared_stream_budget_overflow() {
    let router = with_gateway_surface_security(
        super::super::Router::new().route("/internal", post(echo_security_headers)),
        GatewaySurfaceSecurity::new(GatewayPublicProtocolSurface::ArrowFlight, None)
            .with_policy(GatewaySurfacePolicy::new(16, 8)),
    );
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal")
                .header(CONTENT_LENGTH, "16")
                .body(Body::from("0123456789abcdef"))
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer oversized request: {error}"));

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
