# xiuxian-security

Shared security primitives for Xiuxian services.

## Overview

`xiuxian-security` owns small reusable security building blocks. Runtime
servers use these primitives through their own transport adapters instead of
duplicating string constants, principal signatures, or rate-limit policy.

## Features

- Secret detection in code
- Security pattern matching
- Permission gate helpers
- Public-plane protocol surface labels for Gateway-owned HTTPS JSON/SSE and
  Arrow Flight entry points
- Internal service identity and signed-principal header constants
- Deterministic signed-principal generation and verification for
  Gateway-to-internal service calls. Signed principals use the stable
  `v1:<token-hash>:<signature>` shape so internal services can verify the
  Gateway-issued principal without seeing the raw public bearer token.
- Optional `axum-internal-plane` middleware that rejects raw public bearer
  headers and verifies Gateway-issued internal service identity, public
  protocol, scope, and signed-principal metadata for internal Axum services.
- Small in-process fixed-window admission policy helpers

## Usage

```rust
use std::sync::Arc;

use xiuxian_security::{
    PublicPlaneRateLimiter, PublicProtocolSurface, SignedPrincipalSigner,
    SignedPrincipalVerifier,
};

let limiter = PublicPlaneRateLimiter::new(128);
assert!(limiter.allow());

let signer = SignedPrincipalSigner::new(
    Arc::<str>::from("wendao-gateway"),
    Arc::<str>::from("internal-secret"),
);
let principal = signer.sign_user_token(
    PublicProtocolSurface::ArrowFlight,
    "verified-user-token",
);
assert!(principal.starts_with("v1:"));

let verifier = SignedPrincipalVerifier::new(
    Arc::<str>::from("wendao-gateway"),
    Arc::<str>::from("internal-secret"),
);
assert!(verifier.verify_signed_principal(
    PublicProtocolSurface::ArrowFlight,
    "wendao-gateway",
    &principal,
));
```

## Boundary

This crate does not own public HTTP or Flight routing. Gateway crates validate
user tokens at the public boundary, then use these primitives to sign and
propagate an internal principal. Internal services such as Qianji and Wendao
should verify internal service identity and signed-principal metadata, not
directly accept user bearer tokens.

The optional `axum-internal-plane` feature only provides reusable Axum
middleware for that internal service verification step. It does not make this
crate a server, a public route owner, or a token authority.

## License

Apache-2.0
