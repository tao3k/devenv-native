//! Shared public-plane security primitives for Gateway-owned protocol surfaces.

use std::sync::Arc;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

/// Header carrying the internal service identity after public auth succeeds.
pub const WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER: &str = "x-wendao-internal-service-identity";
/// Header carrying the signed user principal for internal service calls.
pub const WENDAO_SIGNED_PRINCIPAL_HEADER: &str = "x-wendao-signed-principal";
/// Header carrying the public protocol surface that admitted the request.
pub const WENDAO_PUBLIC_PROTOCOL_HEADER: &str = "x-wendao-public-protocol";
/// Header carrying the scope granted by the public protocol surface.
pub const WENDAO_AUTH_SCOPE_HEADER: &str = "x-wendao-auth-scope";
/// Canonical internal service identity asserted by Wendao Gateway.
pub const WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY: &str = "wendao-gateway";
/// Shared internal principal signing secret environment variable.
pub const XIUXIAN_INTERNAL_PRINCIPAL_SECRET_ENV: &str = "XIUXIAN_INTERNAL_PRINCIPAL_SECRET";

/// Public protocol surfaces owned by the Gateway boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicProtocolSurface {
    /// HTTPS JSON and Server-Sent Events surface for OpenAI-like `/v1/*` APIs.
    HttpsJsonSse,
    /// Arrow Flight surface for high-throughput batch and stream APIs.
    ArrowFlight,
}

impl PublicProtocolSurface {
    /// Scope required for this public protocol surface.
    #[must_use]
    pub const fn scope(self) -> &'static str {
        match self {
            Self::HttpsJsonSse => "gateway:https-json-sse",
            Self::ArrowFlight => "gateway:arrow-flight",
        }
    }

    /// Stable protocol label for audit and internal service propagation.
    #[must_use]
    pub const fn protocol(self) -> &'static str {
        match self {
            Self::HttpsJsonSse => "https-json-sse",
            Self::ArrowFlight => "arrow-flight",
        }
    }

    /// Parse a stable protocol label.
    #[must_use]
    pub fn from_protocol(protocol: &str) -> Option<Self> {
        match protocol {
            "https-json-sse" => Some(Self::HttpsJsonSse),
            "arrow-flight" => Some(Self::ArrowFlight),
            _ => None,
        }
    }
}

/// Per-surface public boundary policy.
#[derive(Clone, Debug)]
pub struct PublicSurfacePolicy {
    rate_limit_per_second: u64,
    stream_budget_bytes: usize,
}

impl PublicSurfacePolicy {
    /// Create one public-surface policy.
    #[must_use]
    pub const fn new(rate_limit_per_second: u64, stream_budget_bytes: usize) -> Self {
        Self {
            rate_limit_per_second,
            stream_budget_bytes,
        }
    }

    /// Maximum admitted requests per second for this surface.
    #[must_use]
    pub const fn rate_limit_per_second(&self) -> u64 {
        self.rate_limit_per_second
    }

    /// Maximum admitted request stream bytes for this surface.
    #[must_use]
    pub const fn stream_budget_bytes(&self) -> usize {
        self.stream_budget_bytes
    }
}

/// Small in-process fixed-window limiter for per-surface admission.
#[derive(Debug)]
pub struct PublicPlaneRateLimiter {
    limit_per_second: u64,
    window: Mutex<PublicPlaneRateWindow>,
}

impl PublicPlaneRateLimiter {
    /// Create one fixed-window limiter.
    #[must_use]
    pub fn new(limit_per_second: u64) -> Self {
        Self {
            limit_per_second: limit_per_second.max(1),
            window: Mutex::new(PublicPlaneRateWindow::default()),
        }
    }

    /// Returns true when the current request is admitted.
    #[must_use]
    pub fn allow(&self) -> bool {
        if self.limit_per_second == u64::MAX {
            return true;
        }
        let now_second = current_epoch_second();
        let Ok(mut window) = self.window.lock() else {
            return false;
        };
        if window.epoch_second != now_second {
            window.epoch_second = now_second;
            window.count = 0;
        }
        if window.count >= self.limit_per_second {
            return false;
        }
        window.count += 1;
        true
    }
}

#[derive(Debug, Default)]
struct PublicPlaneRateWindow {
    epoch_second: u64,
    count: u64,
}

fn current_epoch_second() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

/// Signs a verified public user principal for internal service calls.
#[derive(Clone, Debug)]
pub struct SignedPrincipalSigner {
    service_identity: Arc<str>,
    signing_secret: Arc<str>,
}

impl SignedPrincipalSigner {
    /// Create one signer for the Gateway-to-internal plane.
    #[must_use]
    pub fn new(service_identity: Arc<str>, signing_secret: Arc<str>) -> Self {
        Self {
            service_identity,
            signing_secret,
        }
    }

    /// Sign a presented public bearer token for one admitted protocol surface.
    #[must_use]
    pub fn sign_user_token(&self, surface: PublicProtocolSurface, presented_token: &str) -> String {
        let token_hash = blake3::hash(presented_token.as_bytes()).to_hex();
        let signature = signed_principal_signature(
            self.signing_secret.as_ref(),
            self.service_identity.as_ref(),
            surface,
            token_hash.as_str(),
        );
        format!("v1:{token_hash}:{signature}")
    }
}

/// Verifies Gateway-issued signed principals for internal service adapters.
#[derive(Clone, Debug)]
pub struct SignedPrincipalVerifier {
    expected_service_identity: Arc<str>,
    signing_secret: Arc<str>,
}

impl SignedPrincipalVerifier {
    /// Create one verifier for an internal service boundary.
    #[must_use]
    pub fn new(expected_service_identity: Arc<str>, signing_secret: Arc<str>) -> Self {
        Self {
            expected_service_identity,
            signing_secret,
        }
    }

    /// Verify a Gateway-issued signed principal without the raw public token.
    #[must_use]
    pub fn verify_signed_principal(
        &self,
        surface: PublicProtocolSurface,
        service_identity: &str,
        signed_principal: &str,
    ) -> bool {
        if service_identity != self.expected_service_identity.as_ref() {
            return false;
        }

        let Some((token_hash, signature)) = parse_signed_principal(signed_principal) else {
            return false;
        };
        let expected_signature = signed_principal_signature(
            self.signing_secret.as_ref(),
            service_identity,
            surface,
            token_hash,
        );
        constant_time_eq(signature.as_bytes(), expected_signature.as_bytes())
    }
}

fn parse_signed_principal(signed_principal: &str) -> Option<(&str, &str)> {
    let rest = signed_principal.strip_prefix("v1:")?;
    let (token_hash, signature) = rest.split_once(':')?;
    if token_hash.len() != blake3::OUT_LEN * 2 || signature.len() != blake3::OUT_LEN * 2 {
        return None;
    }
    if !token_hash.bytes().all(|byte| byte.is_ascii_hexdigit())
        || !signature.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return None;
    }
    Some((token_hash, signature))
}

fn signed_principal_signature(
    signing_secret: &str,
    service_identity: &str,
    surface: PublicProtocolSurface,
    token_hash: &str,
) -> String {
    let signing_key_hash = blake3::hash(signing_secret.as_bytes());
    let mut hasher = blake3::Hasher::new_keyed(signing_key_hash.as_bytes());
    hasher.update(service_identity.as_bytes());
    hasher.update(b"\n");
    hasher.update(surface.protocol().as_bytes());
    hasher.update(b"\n");
    hasher.update(surface.scope().as_bytes());
    hasher.update(b"\n");
    hasher.update(token_hash.as_bytes());
    hasher.finalize().to_hex().to_string()
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0;
    for (left_byte, right_byte) in left.iter().zip(right) {
        diff |= left_byte ^ right_byte;
    }
    diff == 0
}
