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
- Opaque public API-token generation, parsing, verifier hashing, and
  scope-set helpers for Gateway-owned token admission
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
    PublicApiTokenEnvironment, PublicApiTokenScopeSet, PublicApiTokenVerifier,
    PublicPlaneRateLimiter, PublicProtocolSurface, SignedPrincipalSigner, SignedPrincipalVerifier,
};

let limiter = PublicPlaneRateLimiter::new(128);
assert!(limiter.allow());

let token_verifier = PublicApiTokenVerifier::new(
    Arc::<str>::from("gateway-token-verifier-secret"),
)?;
let issued = token_verifier.issue(PublicApiTokenEnvironment::Live);
assert!(token_verifier.verify_presented_token(
    issued.presented_token(),
    issued.token_prefix(),
    issued.verifier_hash(),
));

let scopes = PublicApiTokenScopeSet::new([
    Arc::<str>::from(PublicProtocolSurface::ArrowFlight.scope()),
]);
assert!(scopes.allows_surface(PublicProtocolSurface::ArrowFlight));

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

This crate does not own public HTTP or Flight routing, public user storage, or
login sessions. Gateway crates validate browser sessions or public API tokens at
the public boundary, then use these primitives to sign and propagate an
internal principal. Internal services such as Qianji and Wendao should verify
internal service identity and signed-principal metadata, not directly accept
user bearer tokens.

Public API tokens are opaque credentials. Gateway may show the full token once,
but durable storage should keep only the public token prefix, verifier hash,
owner metadata, scopes, budget profile, and lifecycle timestamps.

The optional `axum-internal-plane` feature only provides reusable Axum
middleware for that internal service verification step. It does not make this
crate a server, a public route owner, or a token authority.

## Control-Plane Authority

Public API-token lifecycle facts must stay in a durable control-plane
authority, not in an analytical read-model. Production deployments should use a
PostgreSQL-compatible control store as the first supported authority for users,
organizations, projects, API-key metadata, verifier hashes, scopes, roles,
quotas, expiration, revocation, billing authority, and outbox events. A managed
AuthN/AuthZ service may be introduced later as an adapter around that contract,
but it is not the primary storage authority for this lane.

Gateway may keep a small process-local token cache for local development or hot
admission, and Valkey may cache short-lived token/session/rate-limit state, but
neither cache is the final truth for revoke or expire decisions. DuckDB,
DuckLake, and Arrow-SQL surfaces are appropriate for Wendao read-models,
ontology materialization, evidence, benchmark history, projection caches, and
append-only audit projections. They must not become the sole authority for
public API tokens or external user membership.

The intended production split is:

- Gateway terminates external tokens and emits signed internal principals.
- A PostgreSQL-compatible control-plane authority stores small, strongly
  consistent auth facts.
- Valkey handles hot cache, rate-limit counters, stream budgets, nonce, and
  replay guards.
- DuckDB or DuckLake stores queryable read-model and audit projections that can
  be rebuilt from control-plane events.

## License

Apache-2.0
