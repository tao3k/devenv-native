use super::{
    AUTHORIZATION, Arc, Body, CONTENT_TYPE, Request, ServiceExt, StatusCode, app_state,
    build_gateway_router,
};

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
