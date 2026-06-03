use std::sync::Arc;
use std::time::SystemTime;

use axum::Json;
use axum::body::{Body, to_bytes};
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::routing::{Router, post};
use tower::ServiceExt;
use xiuxian_security::{
    PublicApiTokenEnvironment, PublicApiTokenScopeSet, PublicApiTokenVerifier,
    PublicProtocolSurface,
};

use super::{
    GATEWAY_API_TOKEN_PREFIX_ENV, GATEWAY_API_TOKEN_SCOPES_ENV, GATEWAY_API_TOKEN_STATUS_ENV,
    GATEWAY_API_TOKEN_VERIFIER_HASH_ENV, GATEWAY_API_TOKEN_VERIFIER_SECRET_ENV,
    GatewayApiTokenAdmission, GatewayApiTokenRecord, GatewayInMemoryApiTokenStore,
    GatewayPublicProtocolSurface, GatewaySurfaceSecurity, WENDAO_AUTH_SCOPE_HEADER,
    WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY, WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
    WENDAO_PUBLIC_PROTOCOL_HEADER, WENDAO_SIGNED_PRINCIPAL_HEADER,
    gateway_api_token_admission_with_lookup, with_gateway_surface_security,
};

async fn echo_security_headers(headers: HeaderMap) -> Json<serde_json::Value> {
    Json(serde_json::json!({
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

fn token_verifier() -> PublicApiTokenVerifier {
    PublicApiTokenVerifier::new(Arc::<str>::from("gateway-public-token-verifier"))
        .unwrap_or_else(|error| panic!("verifier secret should be accepted: {error}"))
}

fn router_with_token_record(
    verifier: PublicApiTokenVerifier,
    record: GatewayApiTokenRecord,
) -> Router {
    let store = GatewayInMemoryApiTokenStore::default();
    store.insert(record);
    router_with_admission((verifier, Arc::new(store)))
}

fn router_with_admission(admission: GatewayApiTokenAdmission) -> Router {
    let (verifier, repository) = admission;
    with_gateway_surface_security(
        Router::new().route("/internal", post(echo_security_headers)),
        GatewaySurfaceSecurity::new(GatewayPublicProtocolSurface::HttpsJsonSse, None)
            .with_api_token_admission(verifier, repository)
            .with_signing_secret(Some(Arc::<str>::from("internal_secret"))),
    )
}

#[tokio::test]
async fn gateway_https_auth_accepts_public_api_token_and_adds_internal_principal() {
    let verifier = token_verifier();
    let issued = verifier.issue(PublicApiTokenEnvironment::Test);
    let record = GatewayApiTokenRecord::new(
        Arc::<str>::from(issued.token_prefix()),
        Arc::<str>::from(issued.verifier_hash()),
        PublicApiTokenScopeSet::new([Arc::<str>::from(
            PublicProtocolSurface::HttpsJsonSse.scope(),
        )]),
    );
    let router = router_with_token_record(verifier, record);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal")
                .header(
                    AUTHORIZATION,
                    format!("Bearer {}", issued.presented_token()),
                )
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer internal echo requests: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 8192)
        .await
        .unwrap_or_else(|error| panic!("response body should read: {error}"));
    let headers: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|error| panic!("json should parse: {error}"));
    assert_eq!(headers["authorization"], serde_json::Value::Null);
    assert_eq!(headers["scope"], "gateway:https-json-sse");
    assert_eq!(headers["protocol"], "https-json-sse");
    assert_eq!(
        headers["serviceIdentity"],
        WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY
    );
    assert!(
        headers["signedPrincipal"]
            .as_str()
            .is_some_and(|value| value.starts_with("v1:")),
        "{headers}"
    );
}

#[tokio::test]
async fn gateway_https_auth_rejects_public_api_token_without_surface_scope() {
    let verifier = token_verifier();
    let issued = verifier.issue(PublicApiTokenEnvironment::Test);
    let record = GatewayApiTokenRecord::new(
        Arc::<str>::from(issued.token_prefix()),
        Arc::<str>::from(issued.verifier_hash()),
        PublicApiTokenScopeSet::new([Arc::<str>::from(PublicProtocolSurface::ArrowFlight.scope())]),
    );
    let router = router_with_token_record(verifier, record);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal")
                .header(
                    AUTHORIZATION,
                    format!("Bearer {}", issued.presented_token()),
                )
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer rejected requests: {error}"));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn gateway_https_auth_rejects_revoked_public_api_token() {
    let verifier = token_verifier();
    let issued = verifier.issue(PublicApiTokenEnvironment::Test);
    let record = GatewayApiTokenRecord::new(
        Arc::<str>::from(issued.token_prefix()),
        Arc::<str>::from(issued.verifier_hash()),
        PublicApiTokenScopeSet::new([Arc::<str>::from(
            PublicProtocolSurface::HttpsJsonSse.scope(),
        )]),
    )
    .revoked();
    let router = router_with_token_record(verifier, record);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal")
                .header(
                    AUTHORIZATION,
                    format!("Bearer {}", issued.presented_token()),
                )
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer rejected requests: {error}"));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn gateway_https_auth_rejects_expired_public_api_token() {
    let verifier = token_verifier();
    let issued = verifier.issue(PublicApiTokenEnvironment::Test);
    let record = GatewayApiTokenRecord::new(
        Arc::<str>::from(issued.token_prefix()),
        Arc::<str>::from(issued.verifier_hash()),
        PublicApiTokenScopeSet::new([Arc::<str>::from(
            PublicProtocolSurface::HttpsJsonSse.scope(),
        )]),
    )
    .expires_at(SystemTime::UNIX_EPOCH);
    let router = router_with_token_record(verifier, record);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal")
                .header(
                    AUTHORIZATION,
                    format!("Bearer {}", issued.presented_token()),
                )
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer rejected requests: {error}"));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn gateway_static_api_token_seed_rejects_revoked_token() {
    let verifier_secret = "gateway-public-token-verifier";
    let issued = PublicApiTokenVerifier::new(Arc::<str>::from(verifier_secret))
        .unwrap_or_else(|error| panic!("verifier secret should be accepted: {error}"))
        .issue(PublicApiTokenEnvironment::Test);
    let admission = gateway_api_token_admission_with_lookup(&|key| match key {
        GATEWAY_API_TOKEN_VERIFIER_SECRET_ENV => Some(verifier_secret.to_string()),
        GATEWAY_API_TOKEN_PREFIX_ENV => Some(issued.token_prefix().to_string()),
        GATEWAY_API_TOKEN_VERIFIER_HASH_ENV => Some(issued.verifier_hash().to_string()),
        GATEWAY_API_TOKEN_SCOPES_ENV => Some(PublicProtocolSurface::HttpsJsonSse.scope().into()),
        GATEWAY_API_TOKEN_STATUS_ENV => Some("revoked".to_string()),
        _ => None,
    })
    .unwrap_or_else(|| panic!("static token seed should build admission"));
    let router = router_with_admission(admission);

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal")
                .header(
                    AUTHORIZATION,
                    format!("Bearer {}", issued.presented_token()),
                )
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer revoked seed requests: {error}"));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn gateway_https_auth_rejects_malformed_public_api_token() {
    let verifier = token_verifier();
    let router = with_gateway_surface_security(
        Router::new().route("/internal", post(echo_security_headers)),
        GatewaySurfaceSecurity::new(GatewayPublicProtocolSurface::HttpsJsonSse, None)
            .with_api_token_admission(verifier, Arc::new(GatewayInMemoryApiTokenStore::default()))
            .with_signing_secret(Some(Arc::<str>::from("internal_secret"))),
    );

    let response = router
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/internal")
                .header(AUTHORIZATION, "Bearer malformed")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer malformed token requests: {error}"));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
