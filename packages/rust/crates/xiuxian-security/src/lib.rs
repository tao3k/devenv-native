//! xiuxian-security - Security Scanner & Sandbox for Omni `DevEnv`
//!
//! ## Modules
//!
//! - `security`: Secret scanning and permission gatekeeper
//! - `sandbox`: Isolated execution environment for harvested skills
//!
//! ## Features
//!
//! - O(n) linear-time regex matching via `RegexSet`
//! - Pre-compiled patterns at startup (Lazy static)
//! - Zero-copy scanning for large files
//! - Fail-fast on first detected secret
//! - Docker/NsJail sandboxing for safe test execution
//!
//! Patterns follow ODF-REP Security Standards.

mod permissions;
mod public_plane;
mod sandbox;
mod scanner;

pub use permissions::PermissionGatekeeper;
pub use public_plane::{
    PublicPlaneRateLimiter, PublicProtocolSurface, PublicSurfacePolicy, SignedPrincipalSigner,
    SignedPrincipalVerifier, WENDAO_AUTH_SCOPE_HEADER, WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY,
    WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER, WENDAO_PUBLIC_PROTOCOL_HEADER,
    WENDAO_SIGNED_PRINCIPAL_HEADER, XIUXIAN_INTERNAL_PRINCIPAL_SECRET_ENV,
};
pub use sandbox::{SandboxConfig, SandboxError, SandboxMode, SandboxResult, SandboxRunner};
pub use scanner::{SecretScanner, SecurityViolation};
