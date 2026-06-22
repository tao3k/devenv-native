#[cfg(feature = "zhenfa-router")]
use axum::http::header::CONTENT_LENGTH;

#[cfg(feature = "zhenfa-router")]
use super::{
    AUTHORIZATION, Arc, Body, GATEWAY_FLIGHT_SERVICE_AXUM_PATH, Request, ServiceExt, StatusCode,
    app_state, build_gateway_router, build_gateway_router_with_surface_policy,
};

#[cfg(feature = "zhenfa-router")]
#[tokio::test]
async fn test_gateway_router_requires_bearer_token_for_flight_when_configured() {
    let router = build_gateway_router(
        app_state(None),
        32,
        std::time::Duration::from_secs(15),
        16,
        std::time::Duration::from_secs(30),
        true,
        Some(Arc::<str>::from("wd_test")),
    )
    .unwrap_or_else(|error| panic!("gateway router should build: {error}"));
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/arrow.flight.protocol.FlightService/GetFlightInfo")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer Flight requests: {error}"));

    assert_eq!(
        GATEWAY_FLIGHT_SERVICE_AXUM_PATH,
        "/arrow.flight.protocol.FlightService/{*grpc_method}"
    );
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[cfg(feature = "zhenfa-router")]
#[tokio::test]
async fn test_gateway_router_accepts_configured_bearer_token_for_flight() {
    let router = build_gateway_router(
        app_state(None),
        32,
        std::time::Duration::from_secs(15),
        16,
        std::time::Duration::from_secs(30),
        true,
        Some(Arc::<str>::from("wd_test")),
    )
    .unwrap_or_else(|error| panic!("gateway router should build: {error}"));
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/arrow.flight.protocol.FlightService/GetFlightInfo")
                .header(AUTHORIZATION, "Bearer wd_test")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer Flight requests: {error}"));

    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[cfg(feature = "zhenfa-router")]
#[tokio::test]
async fn test_gateway_router_applies_flight_stream_budget_to_flight_mount() {
    let router = build_gateway_router_with_surface_policy(
        app_state(None),
        32,
        std::time::Duration::from_secs(15),
        16,
        std::time::Duration::from_secs(30),
        512,
        128,
        64 * 1024 * 1024,
        8,
        true,
        Some(Arc::<str>::from("wd_test")),
    )
    .unwrap_or_else(|error| panic!("gateway router should build: {error}"));
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/arrow.flight.protocol.FlightService/GetFlightInfo")
                .header(AUTHORIZATION, "Bearer wd_test")
                .header(CONTENT_LENGTH, "16")
                .body(Body::from("0123456789abcdef"))
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer oversized Flight requests: {error}"));

    assert_eq!(response.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[cfg(feature = "zhenfa-router")]
#[tokio::test]
async fn test_gateway_router_applies_flight_rate_limit_to_flight_mount() {
    let router = build_gateway_router_with_surface_policy(
        app_state(None),
        32,
        std::time::Duration::from_secs(15),
        16,
        std::time::Duration::from_secs(30),
        512,
        1,
        64 * 1024 * 1024,
        1024 * 1024 * 1024,
        true,
        Some(Arc::<str>::from("wd_test")),
    )
    .unwrap_or_else(|error| panic!("gateway router should build: {error}"));
    let first = router
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/arrow.flight.protocol.FlightService/GetFlightInfo")
                .header(AUTHORIZATION, "Bearer wd_test")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer first Flight request: {error}"));
    let second = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/arrow.flight.protocol.FlightService/GetFlightInfo")
                .header(AUTHORIZATION, "Bearer wd_test")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer second Flight request: {error}"));

    assert_ne!(first.status(), StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(second.status(), StatusCode::TOO_MANY_REQUESTS);
}
