use super::{
    AUTHORIZATION, Arc, Body, GatewayPublicProtocolSurface, GatewaySurfaceSecurity, Request,
    ServiceExt, StatusCode, echo_security_headers, post, with_gateway_surface_security,
};

#[tokio::test]
async fn test_gateway_https_auth_strips_user_token_and_adds_internal_principal() {
    let router = with_gateway_surface_security(
        super::super::Router::new().route("/internal", post(echo_security_headers)),
        GatewaySurfaceSecurity::new(
            GatewayPublicProtocolSurface::HttpsJsonSse,
            Some(Arc::<str>::from("wd_test")),
        )
        .with_signing_secret(Some(Arc::<str>::from("internal_secret"))),
    );
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal")
                .header(AUTHORIZATION, "Bearer wd_test")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer internal echo requests: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    let body = super::super::to_bytes(response.into_body(), 8192)
        .await
        .unwrap_or_else(|error| panic!("response body should read: {error}"));
    let headers: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|error| panic!("json should parse: {error}"));
    assert_eq!(headers["authorization"], serde_json::Value::Null);
    assert_eq!(headers["scope"], "gateway:https-json-sse");
    assert_eq!(headers["protocol"], "https-json-sse");
    assert_eq!(headers["serviceIdentity"], "wendao-gateway");
    assert!(
        headers["signedPrincipal"]
            .as_str()
            .is_some_and(|value| value.starts_with("v1:")),
        "{headers}"
    );
}

#[cfg(feature = "zhenfa-router")]
#[tokio::test]
async fn test_gateway_flight_auth_strips_user_token_and_adds_flight_scope() {
    let router = with_gateway_surface_security(
        super::super::Router::new().route("/internal", post(echo_security_headers)),
        GatewaySurfaceSecurity::new(
            GatewayPublicProtocolSurface::ArrowFlight,
            Some(Arc::<str>::from("wd_test")),
        )
        .with_signing_secret(Some(Arc::<str>::from("internal_secret"))),
    );
    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal")
                .header(AUTHORIZATION, "Bearer wd_test")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer internal echo requests: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    let body = super::super::to_bytes(response.into_body(), 8192)
        .await
        .unwrap_or_else(|error| panic!("response body should read: {error}"));
    let headers: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|error| panic!("json should parse: {error}"));
    assert_eq!(headers["authorization"], serde_json::Value::Null);
    assert_eq!(headers["scope"], "gateway:arrow-flight");
    assert_eq!(headers["protocol"], "arrow-flight");
    assert_eq!(headers["serviceIdentity"], "wendao-gateway");
    assert!(
        headers["signedPrincipal"]
            .as_str()
            .is_some_and(|value| value.starts_with("v1:")),
        "{headers}"
    );
}
