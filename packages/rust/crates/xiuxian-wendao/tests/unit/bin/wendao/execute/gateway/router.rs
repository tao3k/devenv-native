use std::net::SocketAddr;
use std::sync::Arc;

use axum::body::{Body, to_bytes};
use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{Request, StatusCode};
use axum::routing::Router;
use tower::ServiceExt;

use crate::execute::gateway::command::{GATEWAY_FLIGHT_SERVICE_AXUM_PATH, build_gateway_router};

use super::support::app_state;

#[tokio::test]
async fn test_gateway_server_bind() {
    let app: Router = build_gateway_router(
        app_state(None),
        32,
        std::time::Duration::from_secs(15),
        16,
        std::time::Duration::from_secs(30),
        true,
        None,
    )
    .unwrap_or_else(|error| panic!("gateway router should build: {error}"));

    let addr = SocketAddr::from(([127, 0, 0, 1], 0));
    let listener = tokio::net::TcpListener::bind(addr).await;
    assert!(listener.is_ok(), "Should be able to bind to random port");

    let _ = app;
}

#[cfg(feature = "julia")]
#[tokio::test]
async fn test_gateway_router_mounts_flight_service_on_same_listener() {
    let router = build_gateway_router(
        app_state(None),
        32,
        std::time::Duration::from_secs(15),
        16,
        std::time::Duration::from_secs(30),
        true,
        None,
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
    assert_ne!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_gateway_router_mounts_shared_query_route() {
    let router = build_gateway_router(
        app_state(None),
        32,
        std::time::Duration::from_secs(15),
        16,
        std::time::Duration::from_secs(30),
        true,
        None,
    )
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

#[tokio::test]
async fn test_gateway_router_mounts_public_responses_json_route() {
    let router = build_gateway_router(
        app_state(None),
        32,
        std::time::Duration::from_secs(15),
        16,
        std::time::Duration::from_secs(30),
        true,
        None,
    )
    .unwrap_or_else(|error| panic!("gateway router should build: {error}"));
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"query_language":"sql","input":"SELECT sql_table_name FROM wendao_sql_tables ORDER BY sql_table_name LIMIT 1"}"#,
                ))
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer public response requests: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("response body should decode: {error}"));
    let payload: serde_json::Value = serde_json::from_slice(body.as_ref())
        .unwrap_or_else(|error| panic!("response body should be valid json: {error}"));
    assert_eq!(payload["object"], serde_json::json!("response"));
    assert_eq!(payload["status"], serde_json::json!("completed"));
    assert_eq!(
        payload["output"][0]["json"]["query_language"],
        serde_json::json!("sql")
    );
}

#[tokio::test]
async fn test_gateway_router_mounts_public_responses_sse_route() {
    let router = build_gateway_router(
        app_state(None),
        32,
        std::time::Duration::from_secs(15),
        16,
        std::time::Duration::from_secs(30),
        true,
        None,
    )
    .unwrap_or_else(|error| panic!("gateway router should build: {error}"));
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header(CONTENT_TYPE, "application/json")
                .header(ACCEPT, "text/event-stream")
                .body(Body::from(
                    r#"{"query_language":"sql","query":"SELECT sql_table_name FROM wendao_sql_tables ORDER BY sql_table_name LIMIT 1"}"#,
                ))
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer public response requests: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("response body should decode: {error}"));
    let text = String::from_utf8(body.to_vec())
        .unwrap_or_else(|error| panic!("response body should be utf8: {error}"));
    assert!(text.contains("event: response.output_json.delta"));
    assert!(text.contains("event: response.completed"));
}

#[tokio::test]
async fn test_gateway_router_streams_public_responses_when_stream_flag_is_true() {
    let router = build_gateway_router(
        app_state(None),
        32,
        std::time::Duration::from_secs(15),
        16,
        std::time::Duration::from_secs(30),
        true,
        None,
    )
    .unwrap_or_else(|error| panic!("gateway router should build: {error}"));
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(
                    r#"{"query_language":"sql","input":"SELECT sql_table_name FROM wendao_sql_tables ORDER BY sql_table_name LIMIT 1","stream":true}"#,
                ))
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer public response requests: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("text/event-stream")
    );
}

#[tokio::test]
async fn test_gateway_router_rejects_empty_public_response_input() {
    let router = build_gateway_router(
        app_state(None),
        32,
        std::time::Duration::from_secs(15),
        16,
        std::time::Duration::from_secs(30),
        true,
        None,
    )
    .unwrap_or_else(|error| panic!("gateway router should build: {error}"));
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/v1/responses")
                .header(CONTENT_TYPE, "application/json")
                .body(Body::from(r#"{"query_language":"sql","input":"   "}"#))
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer public response requests: {error}"));

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap_or_else(|error| panic!("response body should decode: {error}"));
    let payload: serde_json::Value = serde_json::from_slice(body.as_ref())
        .unwrap_or_else(|error| panic!("response body should be valid json: {error}"));
    assert_eq!(
        payload["code"],
        serde_json::json!("RESPONSE_EXECUTION_FAILED")
    );
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
