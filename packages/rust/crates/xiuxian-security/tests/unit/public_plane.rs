//! Tests for shared public-plane security primitives.

use std::sync::Arc;

use xiuxian_security::{
    PublicPlaneRateLimiter, PublicProtocolSurface, PublicSurfacePolicy, SignedPrincipalSigner,
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
}
