//! Tests for Gateway public API-token primitives.

use std::sync::Arc;

use xiuxian_security::{
    PublicApiTokenEnvironment, PublicApiTokenError, PublicApiTokenParts, PublicApiTokenScopeSet,
    PublicApiTokenVerifier, PublicProtocolSurface, WENDAO_PUBLIC_API_TOKEN_LIVE_PREFIX,
};

#[test]
fn public_api_token_verifier_rejects_empty_verifier_secret() {
    let result = PublicApiTokenVerifier::new(Arc::<str>::from(""));

    assert_eq!(result.err(), Some(PublicApiTokenError::EmptyVerifierSecret));
}

#[test]
fn public_api_token_issue_returns_presented_token_prefix_and_verifier_hash() {
    let verifier = PublicApiTokenVerifier::new(Arc::<str>::from("gateway-token-secret"))
        .unwrap_or_else(|error| panic!("verifier secret should be accepted: {error}"));

    let issued = verifier.issue(PublicApiTokenEnvironment::Live);
    let parsed = PublicApiTokenParts::parse(issued.presented_token())
        .unwrap_or_else(|error| panic!("issued token should parse: {error}"));

    assert_eq!(issued.environment(), PublicApiTokenEnvironment::Live);
    assert_eq!(parsed.environment(), PublicApiTokenEnvironment::Live);
    assert_eq!(parsed.token_prefix(), issued.token_prefix());
    assert!(
        issued
            .presented_token()
            .starts_with(WENDAO_PUBLIC_API_TOKEN_LIVE_PREFIX)
    );
    assert_eq!(issued.verifier_hash().len(), blake3::OUT_LEN * 2);
    assert_ne!(issued.verifier_hash(), issued.presented_token());
    assert_ne!(parsed.secret(), issued.verifier_hash());
}

#[test]
fn public_api_token_verifies_without_storing_raw_secret() {
    let verifier = PublicApiTokenVerifier::new(Arc::<str>::from("gateway-token-secret"))
        .unwrap_or_else(|error| panic!("verifier secret should be accepted: {error}"));
    let issued = verifier.issue(PublicApiTokenEnvironment::Live);

    assert!(verifier.verify_presented_token(
        issued.presented_token(),
        issued.token_prefix(),
        issued.verifier_hash(),
    ));
    assert!(!verifier.verify_presented_token(
        "wd_live_0000000000000000_0000000000000000000000000000000000000000000000000000000000000000",
        issued.token_prefix(),
        issued.verifier_hash(),
    ));
    assert!(!verifier.verify_presented_token(
        issued.presented_token(),
        "wd_live_ffffffffffffffff",
        issued.verifier_hash(),
    ));
}

#[test]
fn public_api_token_parse_rejects_malformed_tokens() {
    assert_eq!(
        PublicApiTokenParts::parse("wd_live_missing_secret").err(),
        Some(PublicApiTokenError::InvalidTokenPrefix)
    );
    assert_eq!(
        PublicApiTokenParts::parse("wd_live_0000000000000000_nope").err(),
        Some(PublicApiTokenError::InvalidSecret)
    );
    assert_eq!(
        PublicApiTokenParts::parse("unknown_0000000000000000_secret").err(),
        Some(PublicApiTokenError::InvalidEnvironment)
    );
}

#[test]
fn public_api_token_scopes_are_surface_specific() {
    let https_only = PublicApiTokenScopeSet::new([Arc::<str>::from(
        PublicProtocolSurface::HttpsJsonSse.scope(),
    )]);
    let flight_only =
        PublicApiTokenScopeSet::new([Arc::<str>::from(PublicProtocolSurface::ArrowFlight.scope())]);

    assert!(https_only.allows_surface(PublicProtocolSurface::HttpsJsonSse));
    assert!(!https_only.allows_surface(PublicProtocolSurface::ArrowFlight));
    assert!(flight_only.allows_surface(PublicProtocolSurface::ArrowFlight));
    assert!(!flight_only.allows_surface(PublicProtocolSurface::HttpsJsonSse));
    assert_eq!(https_only.scopes().len(), 1);
}
