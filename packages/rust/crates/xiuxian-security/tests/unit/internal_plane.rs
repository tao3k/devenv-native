#![cfg(feature = "axum-internal-plane")]

use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::routing::{Router, get};
use tower::ServiceExt;
use xiuxian_security::{
    InternalServicePrincipalHeaders, InternalServiceSecurity, InternalServiceSecurityError,
    PublicProtocolSurface, SignedPrincipalSigner, WENDAO_AUTH_SCOPE_HEADER,
    WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY, WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
    WENDAO_PUBLIC_PROTOCOL_HEADER, WENDAO_SIGNED_PRINCIPAL_HEADER,
    with_internal_service_security,
};

#[tokio::test]
async fn internal_service_security_accepts_gateway_signed_principal() {
    let router = secured_router();

    let response = router
        .oneshot(internal_request(
            PublicProtocolSurface::ArrowFlight,
            "public-token",
            "internal-secret",
            PublicProtocolSurface::ArrowFlight.scope(),
        ))
        .await
        .unwrap_or_else(|error| panic!("router should answer signed internal request: {error}"));

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn internal_service_security_rejects_raw_public_bearer() {
    let router = secured_router();

    let response = router
        .oneshot(
            Request::builder()
                .uri("/internal")
                .header(header::AUTHORIZATION, "Bearer public-token")
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer bearer request: {error}"));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn internal_service_security_rejects_scope_mismatch() {
    let router = secured_router();

    let response = router
        .oneshot(internal_request(
            PublicProtocolSurface::ArrowFlight,
            "public-token",
            "internal-secret",
            PublicProtocolSurface::HttpsJsonSse.scope(),
        ))
        .await
        .unwrap_or_else(|error| panic!("router should answer mismatched scope request: {error}"));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn internal_service_security_rejects_missing_signed_principal() {
    let router = secured_router();
    let surface = PublicProtocolSurface::HttpsJsonSse;

    let response = router
        .oneshot(
            Request::builder()
                .uri("/internal")
                .header(
                    WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
                    WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY,
                )
                .header(WENDAO_PUBLIC_PROTOCOL_HEADER, surface.protocol())
                .header(WENDAO_AUTH_SCOPE_HEADER, surface.scope())
                .body(Body::empty())
                .unwrap_or_else(|error| panic!("request should build: {error}")),
        )
        .await
        .unwrap_or_else(|error| panic!("router should answer unsigned internal request: {error}"));

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[test]
fn internal_service_security_verifies_header_values_without_axum_request() {
    let security = internal_service_security();
    let signed_principal = signed_principal(PublicProtocolSurface::ArrowFlight, "internal-secret");

    let result = security.verify_headers(InternalServicePrincipalHeaders {
        authorization_present: false,
        service_identity: Some(WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY),
        protocol: Some(PublicProtocolSurface::ArrowFlight.protocol()),
        scope: Some(PublicProtocolSurface::ArrowFlight.scope()),
        signed_principal: Some(signed_principal.as_str()),
    });

    assert_eq!(result, Ok(()));
}

#[test]
fn internal_service_security_rejects_header_values_with_public_authorization() {
    let security = internal_service_security();
    let signed_principal = signed_principal(PublicProtocolSurface::HttpsJsonSse, "internal-secret");

    let result = security.verify_headers(InternalServicePrincipalHeaders {
        authorization_present: true,
        service_identity: Some(WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY),
        protocol: Some(PublicProtocolSurface::HttpsJsonSse.protocol()),
        scope: Some(PublicProtocolSurface::HttpsJsonSse.scope()),
        signed_principal: Some(signed_principal.as_str()),
    });

    assert_eq!(
        result,
        Err(InternalServiceSecurityError::RawPublicAuthorization)
    );
}

fn secured_router() -> Router {
    let router = Router::new().route("/internal", get(|| async { "ok" }));
    with_internal_service_security(router, internal_service_security())
}

fn internal_service_security() -> InternalServiceSecurity {
    InternalServiceSecurity::gateway(
        Arc::<str>::from("internal-secret"),
        Arc::<str>::from("INTERNAL_REQUIRED"),
    )
}

fn signed_principal(surface: PublicProtocolSurface, signing_secret: &str) -> String {
    SignedPrincipalSigner::new(
        Arc::<str>::from(WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY),
        Arc::<str>::from(signing_secret),
    )
    .sign_user_token(surface, "public-token")
}

fn internal_request(
    surface: PublicProtocolSurface,
    public_token: &str,
    signing_secret: &str,
    scope: &str,
) -> Request<Body> {
    let signed_principal = signed_principal_for_token(surface, signing_secret, public_token);

    Request::builder()
        .uri("/internal")
        .header(
            WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
            WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY,
        )
        .header(WENDAO_PUBLIC_PROTOCOL_HEADER, surface.protocol())
        .header(WENDAO_AUTH_SCOPE_HEADER, scope)
        .header(WENDAO_SIGNED_PRINCIPAL_HEADER, signed_principal)
        .body(Body::empty())
        .unwrap_or_else(|error| panic!("request should build: {error}"))
}

fn signed_principal_for_token(
    surface: PublicProtocolSurface,
    signing_secret: &str,
    public_token: &str,
) -> String {
    SignedPrincipalSigner::new(
        Arc::<str>::from(WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY),
        Arc::<str>::from(signing_secret),
    )
    .sign_user_token(surface, public_token)
}
