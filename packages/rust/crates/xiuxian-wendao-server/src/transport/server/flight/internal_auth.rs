//! Internal-plane metadata verification for Gateway-mounted Flight routes.

use std::sync::Arc;

use tonic::Status;
use tonic::metadata::MetadataMap;
use xiuxian_security::{
    PublicProtocolSurface, SignedPrincipalVerifier, WENDAO_AUTH_SCOPE_HEADER,
    WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY, WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
    WENDAO_PUBLIC_PROTOCOL_HEADER, WENDAO_SIGNED_PRINCIPAL_HEADER,
};

const AUTHORIZATION_METADATA_KEY: &str = "authorization";

/// Internal service verifier for Wendao Flight requests admitted by Gateway.
#[derive(Clone, Debug)]
pub struct WendaoFlightInternalSecurity {
    verifier: SignedPrincipalVerifier,
    unauthorized_code: Arc<str>,
}

/// Reason a Gateway-mounted Flight request was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WendaoFlightInternalSecurityError {
    /// Raw public bearer authorization reached the Flight internal plane.
    RawPublicAuthorization,
    /// Missing internal service identity metadata.
    MissingInternalServiceIdentity,
    /// Missing original public protocol metadata.
    MissingPublicProtocol,
    /// Unknown original public protocol metadata.
    UnknownPublicProtocol,
    /// Missing auth scope metadata.
    MissingAuthScope,
    /// Auth scope does not match the declared public protocol.
    AuthScopeMismatch,
    /// Missing signed principal metadata.
    MissingSignedPrincipal,
    /// Signed principal failed verification.
    InvalidSignedPrincipal,
}

impl WendaoFlightInternalSecurityError {
    /// Stable error message for Flight adapters.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::RawPublicAuthorization => "raw public Authorization metadata is not accepted",
            Self::MissingInternalServiceIdentity => "missing internal service identity",
            Self::MissingPublicProtocol => "missing public protocol",
            Self::UnknownPublicProtocol => "unknown public protocol",
            Self::MissingAuthScope => "missing auth scope",
            Self::AuthScopeMismatch => "auth scope mismatch",
            Self::MissingSignedPrincipal => "missing signed principal",
            Self::InvalidSignedPrincipal => "invalid signed principal",
        }
    }
}

impl WendaoFlightInternalSecurity {
    /// Create one internal service verifier.
    #[must_use]
    pub fn new(
        expected_service_identity: Arc<str>,
        signing_secret: Arc<str>,
        unauthorized_code: Arc<str>,
    ) -> Self {
        Self {
            verifier: SignedPrincipalVerifier::new(expected_service_identity, signing_secret),
            unauthorized_code,
        }
    }

    /// Create one verifier for Flight requests admitted by Wendao Gateway.
    #[must_use]
    pub fn gateway(signing_secret: Arc<str>, unauthorized_code: Arc<str>) -> Self {
        Self::new(
            Arc::<str>::from(WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY),
            signing_secret,
            unauthorized_code,
        )
    }

    pub(crate) fn verify_metadata(&self, metadata: &MetadataMap) -> Result<(), Status> {
        self.verify_metadata_result(metadata)
            .map_err(|error| self.unauthenticated(error))
    }

    fn verify_metadata_result(
        &self,
        metadata: &MetadataMap,
    ) -> Result<(), WendaoFlightInternalSecurityError> {
        if metadata.contains_key(AUTHORIZATION_METADATA_KEY) {
            return Err(WendaoFlightInternalSecurityError::RawPublicAuthorization);
        }

        let Some(service_identity) =
            metadata_str(metadata, WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER)
        else {
            return Err(WendaoFlightInternalSecurityError::MissingInternalServiceIdentity);
        };
        let Some(protocol) = metadata_str(metadata, WENDAO_PUBLIC_PROTOCOL_HEADER) else {
            return Err(WendaoFlightInternalSecurityError::MissingPublicProtocol);
        };
        let Some(surface) = PublicProtocolSurface::from_protocol(protocol) else {
            return Err(WendaoFlightInternalSecurityError::UnknownPublicProtocol);
        };
        let Some(scope) = metadata_str(metadata, WENDAO_AUTH_SCOPE_HEADER) else {
            return Err(WendaoFlightInternalSecurityError::MissingAuthScope);
        };
        if scope != surface.scope() {
            return Err(WendaoFlightInternalSecurityError::AuthScopeMismatch);
        }
        let Some(signed_principal) = metadata_str(metadata, WENDAO_SIGNED_PRINCIPAL_HEADER) else {
            return Err(WendaoFlightInternalSecurityError::MissingSignedPrincipal);
        };
        if !self
            .verifier
            .verify_signed_principal(surface, service_identity, signed_principal)
        {
            return Err(WendaoFlightInternalSecurityError::InvalidSignedPrincipal);
        }

        Ok(())
    }

    fn unauthenticated(&self, error: WendaoFlightInternalSecurityError) -> Status {
        Status::unauthenticated(format!("{}: {}", self.unauthorized_code, error.message()))
    }
}

fn metadata_str<'a>(metadata: &'a MetadataMap, key: &str) -> Option<&'a str> {
    metadata.get(key).and_then(|value| value.to_str().ok())
}
