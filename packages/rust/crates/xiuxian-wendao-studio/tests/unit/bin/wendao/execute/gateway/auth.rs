use std::sync::Arc;

use axum::Json;
use axum::body::{Body, to_bytes};
use axum::http::header::AUTHORIZATION;
use axum::http::{HeaderMap, Request, StatusCode};
use axum::routing::{Router, post};
use serde_json::json;
use tower::ServiceExt;
use xiuxian_security::{PublicApiTokenScopeSet, PublicApiTokenVerifier, PublicProtocolSurface};

use super::{GatewayAuthIssuer, gateway_auth_allowed_scopes_with_lookup, gateway_auth_router};
use crate::bin_support::wendao::execute::gateway::security::{
    GatewayApiTokenRepositoryHandle, GatewayInMemoryApiTokenStore, GatewayPublicProtocolSurface,
    GatewaySurfaceSecurity, WENDAO_AUTH_SCOPE_HEADER, with_gateway_surface_security,
};
use crate::contracts::routes::API_AUTH_TOKENS_AXUM_PATH;

async fn echo_scope(headers: HeaderMap) -> Json<serde_json::Value> {
    Json(json!({
        "scope": headers
            .get(WENDAO_AUTH_SCOPE_HEADER)
            .and_then(|value| value.to_str().ok()),
    }))
}

fn verifier() -> PublicApiTokenVerifier {
    PublicApiTokenVerifier::new(Arc::<str>::from("gateway-public-token-verifier"))
        .unwrap_or_else(|error| panic!("verifier secret should be accepted: {error}"))
}

fn auth_issuer(
    verifier: PublicApiTokenVerifier,
    repository: GatewayApiTokenRepositoryHandle,
    allowed_scopes: PublicApiTokenScopeSet,
) -> GatewayAuthIssuer {
    GatewayAuthIssuer::new(
        verifier,
        repository,
        Arc::<str>::from("alice"),
        Arc::<str>::from("correct horse battery staple"),
        allowed_scopes,
        128,
    )
}

#[tokio::test]
async fn gateway_auth_tokens_issue_api_token_that_can_call_protected_https() {
    let verifier = verifier();
    let store = GatewayInMemoryApiTokenStore::default();
    let app = Router::new()
        .merge(gateway_auth_router(auth_issuer(
            verifier.clone(),
            Arc::new(store.clone()),
            PublicApiTokenScopeSet::new([
                Arc::<str>::from(PublicProtocolSurface::HttpsJsonSse.scope()),
                Arc::<str>::from(PublicProtocolSurface::ArrowFlight.scope()),
            ]),
        )))
        .merge(with_gateway_surface_security(
            Router::new().route("/protected", post(echo_scope)),
            GatewaySurfaceSecurity::new(GatewayPublicProtocolSurface::HttpsJsonSse, None)
                .with_api_token_admission(verifier, Arc::new(store))
                .with_signing_secret(Some(Arc::<str>::from("internal_secret"))),
        ));

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(API_AUTH_TOKENS_AXUM_PATH)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "alice",
                        "password": "correct horse battery staple",
                        "scopes": [PublicProtocolSurface::HttpsJsonSse.scope()],
                        "environment": "test",
                    })
                    .to_string(),
                ))
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("auth route should answer: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), 8192)
        .await
        .unwrap_or_else(|error| panic!("response body should read: {error}"));
    let issued: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|error| panic!("json should parse: {error}"));
    let token = issued["token"]
        .as_str()
        .unwrap_or_else(|| panic!("issued token should be present: {issued}"));

    let protected = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/protected")
                .header(AUTHORIZATION, format!("Bearer {token}"))
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("protected route should answer: {error}"));

    assert_eq!(protected.status(), StatusCode::OK);
    let body = to_bytes(protected.into_body(), 8192)
        .await
        .unwrap_or_else(|error| panic!("response body should read: {error}"));
    let headers: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or_else(|error| panic!("json should parse: {error}"));
    assert_eq!(
        headers["scope"],
        PublicProtocolSurface::HttpsJsonSse.scope()
    );
}

#[tokio::test]
async fn gateway_auth_tokens_reject_scope_outside_allowed_surface_set() {
    let app = gateway_auth_router(auth_issuer(
        verifier(),
        Arc::new(GatewayInMemoryApiTokenStore::default()),
        PublicApiTokenScopeSet::new([Arc::<str>::from(
            PublicProtocolSurface::HttpsJsonSse.scope(),
        )]),
    ));

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(API_AUTH_TOKENS_AXUM_PATH)
                .header("content-type", "application/json")
                .body(Body::from(
                    json!({
                        "username": "alice",
                        "password": "correct horse battery staple",
                        "scopes": [PublicProtocolSurface::ArrowFlight.scope()],
                    })
                    .to_string(),
                ))
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("auth route should answer: {error}"));

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[test]
fn gateway_auth_allowed_scopes_default_to_both_public_surfaces() {
    let scopes = gateway_auth_allowed_scopes_with_lookup(&|_| None);

    assert!(scopes.allows_surface(PublicProtocolSurface::HttpsJsonSse));
    assert!(scopes.allows_surface(PublicProtocolSurface::ArrowFlight));
}

#[test]
fn gateway_auth_allowed_scopes_do_not_expand_invalid_explicit_config() {
    let scopes = gateway_auth_allowed_scopes_with_lookup(&|key| match key {
        "XIUXIAN_WENDAO_GATEWAY_AUTH_ALLOWED_SCOPES" => Some("not-a-scope".to_string()),
        _ => None,
    });

    assert!(!scopes.allows_surface(PublicProtocolSurface::HttpsJsonSse));
    assert!(!scopes.allows_surface(PublicProtocolSurface::ArrowFlight));
}
