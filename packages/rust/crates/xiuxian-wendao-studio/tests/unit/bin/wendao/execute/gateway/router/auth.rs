use super::{
    AUTHORIZATION, Arc, Body, CONTENT_TYPE, Request, ServiceExt, StatusCode, app_state,
    build_gateway_router,
};

#[cfg(feature = "zhenfa-router")]
use super::GATEWAY_FLIGHT_SERVICE_AXUM_PATH;

#[tokio::test]
async fn test_gateway_router_keeps_health_route_unauthenticated_when_bearer_configured() {
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
                .method("GET")
                .uri("/api/health")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer health requests: {error}"));

    assert_ne!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_gateway_router_requires_bearer_token_when_configured() {
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
                .uri("/query")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"query_language":"sql","query":"SELECT 1"}"#))
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer shared query requests: {error}"));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_gateway_router_protects_public_responses_with_configured_bearer_token() {
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
                .uri("/v1/responses")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"query_language":"sql","input":"SELECT 1"}"#))
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer public response requests: {error}"));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

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

#[tokio::test]
async fn test_gateway_router_accepts_configured_bearer_token() {
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
                .uri("/query")
                .header(CONTENT_TYPE, "application/json")
                .header(AUTHORIZATION, "Bearer wd_test")
                .body(Body::from(
                    r#"{"query_language":"sql","query":"SELECT sql_table_name FROM wendao_sql_tables ORDER BY sql_table_name LIMIT 1"}"#,
                ))
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer shared query requests: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
}
