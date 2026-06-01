//! Tests for shared public-plane security primitives.

use std::sync::Arc;

use xiuxian_security::{
    PublicPlaneRateLimiter, PublicProtocolSurface, PublicSurfacePolicy, SignedPrincipalSigner,
    SignedPrincipalVerifier,
};

#[test]
fn public_protocol_surface_exposes_stable_scope_and_protocol() {
    assert_eq!(
        PublicProtocolSurface::HttpsJsonSse.scope(),
        "gateway:https-json-sse"
    );
    assert_eq!(
        PublicProtocolSurface::HttpsJsonSse.protocol(),
        "https-json-sse"
    );
    assert_eq!(
        PublicProtocolSurface::ArrowFlight.scope(),
        "gateway:arrow-flight"
    );
    assert_eq!(
        PublicProtocolSurface::ArrowFlight.protocol(),
        "arrow-flight"
    );
    assert_eq!(
        PublicProtocolSurface::from_protocol("https-json-sse"),
        Some(PublicProtocolSurface::HttpsJsonSse)
    );
    assert_eq!(
        PublicProtocolSurface::from_protocol("arrow-flight"),
        Some(PublicProtocolSurface::ArrowFlight)
    );
    assert_eq!(PublicProtocolSurface::from_protocol("unknown"), None);
}

#[test]
fn public_surface_policy_exposes_rate_and_stream_budget() {
    let policy = PublicSurfacePolicy::new(32, 4096);

    assert_eq!(policy.rate_limit_per_second(), 32);
    assert_eq!(policy.stream_budget_bytes(), 4096);
}

#[test]
fn public_plane_rate_limiter_rejects_over_limit_requests() {
    let limiter = PublicPlaneRateLimiter::new(1);

    assert!(limiter.allow());
    assert!(!limiter.allow());
}

#[test]
fn signed_principal_signer_is_stable_and_surface_specific() {
    let signer = SignedPrincipalSigner::new(
        Arc::<str>::from("wendao-gateway"),
        Arc::<str>::from("internal-secret"),
    );

    let https = signer.sign_user_token(PublicProtocolSurface::HttpsJsonSse, "user-token");
    let https_again = signer.sign_user_token(PublicProtocolSurface::HttpsJsonSse, "user-token");
    let flight = signer.sign_user_token(PublicProtocolSurface::ArrowFlight, "user-token");

    assert_eq!(https, https_again);
    assert_ne!(https, flight);
    assert!(https.starts_with("v1:"));
    assert_eq!(https.split(':').count(), 3);
}

#[test]
fn signed_principal_verifier_accepts_gateway_principal_without_raw_token() {
    let signer = SignedPrincipalSigner::new(
        Arc::<str>::from("wendao-gateway"),
        Arc::<str>::from("internal-secret"),
    );
    let verifier = SignedPrincipalVerifier::new(
        Arc::<str>::from("wendao-gateway"),
        Arc::<str>::from("internal-secret"),
    );

    let signed = signer.sign_user_token(PublicProtocolSurface::ArrowFlight, "user-token");

    assert!(verifier.verify_signed_principal(
        PublicProtocolSurface::ArrowFlight,
        "wendao-gateway",
        signed.as_str(),
    ));
}

#[test]
fn signed_principal_verifier_rejects_wrong_identity_surface_secret_and_shape() {
    let signer = SignedPrincipalSigner::new(
        Arc::<str>::from("wendao-gateway"),
        Arc::<str>::from("internal-secret"),
    );
    let verifier = SignedPrincipalVerifier::new(
        Arc::<str>::from("wendao-gateway"),
        Arc::<str>::from("internal-secret"),
    );
    let wrong_secret_verifier = SignedPrincipalVerifier::new(
        Arc::<str>::from("wendao-gateway"),
        Arc::<str>::from("other-secret"),
    );

    let signed = signer.sign_user_token(PublicProtocolSurface::ArrowFlight, "user-token");

    assert!(!verifier.verify_signed_principal(
        PublicProtocolSurface::ArrowFlight,
        "other-service",
        signed.as_str(),
    ));
    assert!(!verifier.verify_signed_principal(
        PublicProtocolSurface::HttpsJsonSse,
        "wendao-gateway",
        signed.as_str(),
    ));
    assert!(!wrong_secret_verifier.verify_signed_principal(
        PublicProtocolSurface::ArrowFlight,
        "wendao-gateway",
        signed.as_str(),
    ));
    assert!(!verifier.verify_signed_principal(
        PublicProtocolSurface::ArrowFlight,
        "wendao-gateway",
        "v1:not-a-principal",
    ));
}
