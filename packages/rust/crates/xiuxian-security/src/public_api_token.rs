//! Opaque public API-token primitives for Gateway-owned admission.

use std::sync::Arc;

use rand::RngCore;
use rand::rngs::OsRng;
use thiserror::Error;

use crate::PublicProtocolSurface;

/// Prefix for production Wendao API tokens.
pub const WENDAO_PUBLIC_API_TOKEN_LIVE_PREFIX: &str = "wd_live";
/// Prefix for non-production Wendao API tokens.
pub const WENDAO_PUBLIC_API_TOKEN_TEST_PREFIX: &str = "wd_test";

const PUBLIC_API_TOKEN_PREFIX_BYTES: usize = 8;
const PUBLIC_API_TOKEN_SECRET_BYTES: usize = 32;
const PUBLIC_API_TOKEN_HASH_CONTEXT: &[u8] = b"wendao-public-api-token-v1\n";

/// Deployment class carried by an opaque public API token.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicApiTokenEnvironment {
    /// Production token.
    Live,
    /// Non-production token.
    Test,
}

impl PublicApiTokenEnvironment {
    /// Stable token prefix used in presented credentials.
    #[must_use]
    pub const fn stable_prefix(self) -> &'static str {
        match self {
            Self::Live => WENDAO_PUBLIC_API_TOKEN_LIVE_PREFIX,
            Self::Test => WENDAO_PUBLIC_API_TOKEN_TEST_PREFIX,
        }
    }

    fn from_stable_prefix(prefix: &str) -> Option<Self> {
        match prefix {
            WENDAO_PUBLIC_API_TOKEN_LIVE_PREFIX => Some(Self::Live),
            WENDAO_PUBLIC_API_TOKEN_TEST_PREFIX => Some(Self::Test),
            _ => None,
        }
    }
}

/// Parsed parts of a presented public API token.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PublicApiTokenParts<'a> {
    environment: PublicApiTokenEnvironment,
    token_prefix: &'a str,
    secret: &'a str,
}

impl<'a> PublicApiTokenParts<'a> {
    /// Parse an opaque Wendao public API token.
    ///
    /// # Errors
    ///
    /// Returns an error when the token prefix, public token fragment, or secret
    /// fragment does not match the Wendao public API-token contract.
    pub fn parse(presented_token: &'a str) -> Result<Self, PublicApiTokenError> {
        let (environment, remainder) = parse_environment_prefix(presented_token)?;
        let Some((public_fragment, secret)) = remainder.split_once('_') else {
            return Err(PublicApiTokenError::InvalidFormat);
        };
        if !is_lower_hex_len(public_fragment, PUBLIC_API_TOKEN_PREFIX_BYTES * 2) {
            return Err(PublicApiTokenError::InvalidTokenPrefix);
        }
        if !is_lower_hex_len(secret, PUBLIC_API_TOKEN_SECRET_BYTES * 2) {
            return Err(PublicApiTokenError::InvalidSecret);
        }
        let token_prefix_end = environment.stable_prefix().len() + 1 + public_fragment.len();
        let token_prefix = &presented_token[..token_prefix_end];
        Ok(Self {
            environment,
            token_prefix,
            secret,
        })
    }

    /// Token environment.
    #[must_use]
    pub const fn environment(&self) -> PublicApiTokenEnvironment {
        self.environment
    }

    /// Public token prefix that may be stored and shown for audits.
    #[must_use]
    pub const fn token_prefix(&self) -> &'a str {
        self.token_prefix
    }

    /// Secret token fragment. Gateway callers should not persist this value.
    #[must_use]
    pub const fn secret(&self) -> &'a str {
        self.secret
    }
}

/// Result of issuing one public API token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IssuedPublicApiToken {
    presented_token: String,
    token_prefix: Arc<str>,
    verifier_hash: Arc<str>,
    environment: PublicApiTokenEnvironment,
}

impl IssuedPublicApiToken {
    /// Full token shown once to the user.
    #[must_use]
    pub fn presented_token(&self) -> &str {
        &self.presented_token
    }

    /// Public token prefix safe for storage and display.
    #[must_use]
    pub fn token_prefix(&self) -> &str {
        self.token_prefix.as_ref()
    }

    /// Non-recoverable verifier hash for persistent storage.
    #[must_use]
    pub fn verifier_hash(&self) -> &str {
        self.verifier_hash.as_ref()
    }

    /// Token environment.
    #[must_use]
    pub const fn environment(&self) -> PublicApiTokenEnvironment {
        self.environment
    }
}

/// Issues and verifies opaque public API tokens.
#[derive(Clone, Debug)]
pub struct PublicApiTokenVerifier {
    verifier_secret: Arc<str>,
}

impl PublicApiTokenVerifier {
    /// Create a token verifier with the Gateway verifier secret.
    ///
    /// # Errors
    ///
    /// Returns an error when the verifier secret is empty.
    pub fn new(verifier_secret: Arc<str>) -> Result<Self, PublicApiTokenError> {
        if verifier_secret.is_empty() {
            return Err(PublicApiTokenError::EmptyVerifierSecret);
        }
        Ok(Self { verifier_secret })
    }

    /// Issue one opaque token and its persistent verifier metadata.
    #[must_use]
    pub fn issue(&self, environment: PublicApiTokenEnvironment) -> IssuedPublicApiToken {
        let public_fragment = random_hex::<PUBLIC_API_TOKEN_PREFIX_BYTES>();
        let secret = random_hex::<PUBLIC_API_TOKEN_SECRET_BYTES>();
        let presented_token = format!("{}_{public_fragment}_{secret}", environment.stable_prefix());
        let token_prefix =
            Arc::<str>::from(format!("{}_{public_fragment}", environment.stable_prefix()));
        let verifier_hash = Arc::<str>::from(self.derive_hash_unchecked(presented_token.as_str()));
        IssuedPublicApiToken {
            presented_token,
            token_prefix,
            verifier_hash,
            environment,
        }
    }

    /// Derive the persistent verifier hash for one presented token.
    ///
    /// # Errors
    ///
    /// Returns an error when the presented token does not match the public
    /// API-token format.
    pub fn derive_verifier_hash(
        &self,
        presented_token: &str,
    ) -> Result<String, PublicApiTokenError> {
        PublicApiTokenParts::parse(presented_token)?;
        Ok(self.derive_hash_unchecked(presented_token))
    }

    /// Verify a presented token against stored prefix and verifier hash.
    #[must_use]
    pub fn verify_presented_token(
        &self,
        presented_token: &str,
        stored_token_prefix: &str,
        stored_verifier_hash: &str,
    ) -> bool {
        let Ok(parts) = PublicApiTokenParts::parse(presented_token) else {
            return false;
        };
        if parts.token_prefix() != stored_token_prefix {
            return false;
        }
        let candidate_hash = self.derive_hash_unchecked(presented_token);
        constant_time_eq(candidate_hash.as_bytes(), stored_verifier_hash.as_bytes())
    }

    fn derive_hash_unchecked(&self, presented_token: &str) -> String {
        let verifier_key_hash = blake3::hash(self.verifier_secret.as_bytes());
        let mut hasher = blake3::Hasher::new_keyed(verifier_key_hash.as_bytes());
        hasher.update(PUBLIC_API_TOKEN_HASH_CONTEXT);
        hasher.update(presented_token.as_bytes());
        hasher.finalize().to_hex().to_string()
    }
}

/// Public token scopes granted at Gateway admission.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct PublicApiTokenScopeSet {
    scopes: Vec<Arc<str>>,
}

impl PublicApiTokenScopeSet {
    /// Create a scope set from stable scope labels.
    #[must_use]
    pub fn new<I, S>(scopes: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<Arc<str>>,
    {
        Self {
            scopes: scopes.into_iter().map(Into::into).collect(),
        }
    }

    /// Returns true when this token grants the exact scope.
    #[must_use]
    pub fn contains_scope(&self, scope: &str) -> bool {
        self.scopes
            .iter()
            .any(|candidate| candidate.as_ref() == scope)
    }

    /// Returns true when this token grants the requested public protocol
    /// surface.
    #[must_use]
    pub fn allows_surface(&self, surface: PublicProtocolSurface) -> bool {
        self.contains_scope(surface.scope())
    }

    /// Stored scope labels.
    #[must_use]
    pub fn scopes(&self) -> &[Arc<str>] {
        &self.scopes
    }
}

/// Reason a public API token was rejected.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum PublicApiTokenError {
    /// Gateway verifier secret is empty.
    #[error("public API token verifier secret is empty")]
    EmptyVerifierSecret,
    /// Token does not match the expected three-part shape.
    #[error("public API token shape is invalid")]
    InvalidFormat,
    /// Token environment prefix is unknown.
    #[error("public API token environment prefix is invalid")]
    InvalidEnvironment,
    /// Public token prefix fragment is invalid.
    #[error("public API token prefix is invalid")]
    InvalidTokenPrefix,
    /// Secret token fragment is invalid.
    #[error("public API token secret is invalid")]
    InvalidSecret,
}

fn parse_environment_prefix(
    presented_token: &str,
) -> Result<(PublicApiTokenEnvironment, &str), PublicApiTokenError> {
    let Some((candidate, remainder)) = presented_token.split_once('_') else {
        return Err(PublicApiTokenError::InvalidFormat);
    };
    let Some((second, remaining)) = remainder.split_once('_') else {
        return Err(PublicApiTokenError::InvalidFormat);
    };
    let stable_prefix = format!("{candidate}_{second}");
    let Some(environment) = PublicApiTokenEnvironment::from_stable_prefix(stable_prefix.as_str())
    else {
        return Err(PublicApiTokenError::InvalidEnvironment);
    };
    Ok((environment, remaining))
}

fn random_hex<const N: usize>() -> String {
    let mut bytes = [0_u8; N];
    OsRng.fill_bytes(&mut bytes);
    hex_bytes(&bytes)
}

fn hex_bytes(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[usize::from(byte >> 4)] as char);
        output.push(HEX[usize::from(byte & 0x0f)] as char);
    }
    output
}

fn is_lower_hex_len(value: &str, expected_len: usize) -> bool {
    value.len() == expected_len
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
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
