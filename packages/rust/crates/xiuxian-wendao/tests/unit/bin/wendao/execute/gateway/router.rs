use std::net::SocketAddr;

use axum::body::{Body, to_bytes};
use axum::http::header::CONTENT_TYPE;
use axum::http::{Request, StatusCode};
use axum::routing::Router;
use tower::ServiceExt;

use crate::execute::gateway::command::{GATEWAY_FLIGHT_SERVICE_AXUM_PATH, build_gateway_router};

use super::support::app_state;

#[tokio::test]
async fn test_gateway_server_bind() {
    let app: Router = build_gateway_router(app_state(None), 32, std::time::Duration::from_secs(15))
        .unwrap_or_else(|error| panic!("gateway router should build: {error}"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(addr).await;
    assert!(listener.is_ok(), "Should be able to bind to random port");

    let _ = app;
}

#[cfg(feature = "julia")]
#[tokio::test]
async fn test_gateway_router_mounts_flight_service_on_same_listener() {
    let router = build_gateway_router(app_state(None), 32, std::time::Duration::from_secs(15))
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
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_gateway_router_mounts_shared_query_route() {
    let router = build_gateway_router(app_state(None), 32, std::time::Duration::from_secs(15))
        .unwrap_or_else(|error| panic!("gateway router should build: {error}"));
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/query")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"query_language":"sql","query":"SELECT sql_table_name FROM wendao_sql_tables ORDER BY sql_table_name LIMIT 1"}"#,
                ))
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer shared query requests: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("response body should decode: {error}"));
    let payload: serde_json::Value = serde_json::from_slice(body.as_ref())
        .unwrap_or_else(|error| panic!("response body should be valid json: {error}"));
    assert_eq!(payload["query_language"], serde_json::json!("sql"));
    assert!(
        payload["payload"]["metadata"]["registeredTableCount"]
            .as_u64()
            .is_some_and(|count| count >= 1),
        "shared query payload should expose SQL surface metadata"
    );
}
