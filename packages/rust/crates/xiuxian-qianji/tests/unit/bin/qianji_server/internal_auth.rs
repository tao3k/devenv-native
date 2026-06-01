use std::sync::Arc;

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::Value;
use std::path::PathBuf;
use tower::util::ServiceExt;
use xiuxian_security::{
    PublicProtocolSurface, SignedPrincipalSigner, WENDAO_AUTH_SCOPE_HEADER,
    WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY, WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
    WENDAO_PUBLIC_PROTOCOL_HEADER, WENDAO_SIGNED_PRINCIPAL_HEADER,
};

use super::support::must_ok;
use crate::qianji_server_cli::cli::QianjiServerServeCommand;
use crate::qianji_server_cli::run::build_qianji_server_router_with_internal_security;
use crate::qianji_server_cli::security::{
    QianjiInternalServiceSecurity, require_qianji_internal_service_security_with_lookup,
};

#[test]
fn qianji_server_internal_security_requires_startup_secret() {
    let error = require_qianji_internal_service_security_with_lookup(&|_| None)
        .expect_err("qianji-server startup should require internal principal secret");

    assert!(
        error
            .to_string()
            .contains("XIUXIAN_QIANJI_INTERNAL_PRINCIPAL_SECRET"),
        "{error}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_internal_security_leaves_health_open() {
    let router = secured_router();

    let response = router
        .oneshot(get("/healthz"))
        .await
        .unwrap_or_else(|error| panic!("health route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_internal_security_rejects_missing_principal() {
    let router = secured_router();

    let response = router
        .oneshot(get("/flowhub/scenarios"))
        .await
        .unwrap_or_else(|error| panic!("flowhub route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert_eq!(body["code"], "QIANJI_INTERNAL_PRINCIPAL_REQUIRED");
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_internal_security_rejects_raw_public_bearer() {
    let router = secured_router();

    let response = router
        .oneshot(
            Request::builder()
                .uri("/flowhub/scenarios")
                .header(header::AUTHORIZATION, "Bearer public-token")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("flowhub route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = response_json(response).await;
    assert!(
        body["error"]
            .as_str()
            .is_some_and(|message| message.contains("Authorization")),
        "unexpected auth error: {body}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_internal_security_accepts_gateway_signed_principal() {
    let router = secured_router();

    let response = router
        .oneshot(internal_get("/flowhub/scenarios", "internal-secret"))
        .await
        .unwrap_or_else(|error| panic!("flowhub route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
    let body = response_json(response).await;
    assert_eq!(body["passed"], false);
}

#[tokio::test(flavor = "current_thread")]
async fn qianji_server_internal_security_rejects_bad_signature() {
    let router = secured_router();

    let response = router
        .oneshot(internal_get("/flowhub/scenarios", "wrong-secret"))
        .await
        .unwrap_or_else(|error| panic!("flowhub route should respond: {error}"));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

fn secured_router() -> Router {
    let command = QianjiServerServeCommand {
        bind_addr: None,
        flight_bind_addr: None,
        valkey_url: Some("not-a-valkey-url".to_string()),
        require_valkey_ready: None,
        flowhub_root: Some(PathBuf::from("missing-flowhub-internal-auth")),
        control_ledger_path: None,
    };
    must_ok(
        build_qianji_server_router_with_internal_security(
            &command,
            Some(QianjiInternalServiceSecurity::gateway(
                Arc::<str>::from("internal-secret"),
                Arc::<str>::from("QIANJI_INTERNAL_PRINCIPAL_REQUIRED"),
            )),
        ),
        "secured qianji-server router should build",
    )
}

fn get(uri: &str) -> Request<Body> {
    Request::builder()
        .uri(uri)
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("GET request should build: {error}"))
}

fn internal_get(uri: &str, signing_secret: &str) -> Request<Body> {
    let surface = PublicProtocolSurface::HttpsJsonSse;
    let signed_principal = SignedPrincipalSigner::new(
        Arc::<str>::from(WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY),
        Arc::<str>::from(signing_secret),
    )
    .sign_user_token(surface, "public-token");

    Request::builder()
        .uri(uri)
        .header(
            WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
            WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY,
        )
        .header(WENDAO_PUBLIC_PROTOCOL_HEADER, surface.protocol())
        .header(WENDAO_AUTH_SCOPE_HEADER, surface.scope())
        .header(WENDAO_SIGNED_PRINCIPAL_HEADER, signed_principal)
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("internal GET request should build: {error}"))
}

async fn response_json(response: axum::response::Response) -> Value {
    let body = to_bytes(response.into_body(), 1024 * 1024)
        .await
        .unwrap_or_else(|error| panic!("response body should read: {error}"));
    serde_json::from_slice(&body)
        .unwrap_or_else(|error| panic!("response body should decode as JSON: {error}"))
}
