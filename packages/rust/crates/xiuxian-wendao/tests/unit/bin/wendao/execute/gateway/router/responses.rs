use super::*;

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
