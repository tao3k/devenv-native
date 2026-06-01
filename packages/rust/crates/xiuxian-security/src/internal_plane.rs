//! Shared Axum middleware for internal service principal verification.

use std::sync::Arc;

use axum::Json;
use axum::extract::{Request, State};
use axum::http::StatusCode;
use axum::http::header;
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::Router;

use crate::public_plane::{
    PublicProtocolSurface, SignedPrincipalVerifier, WENDAO_AUTH_SCOPE_HEADER,
    WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY, WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER,
    WENDAO_PUBLIC_PROTOCOL_HEADER, WENDAO_SIGNED_PRINCIPAL_HEADER,
};

/// Internal service boundary verifier for Gateway-issued principals.
#[derive(Clone, Debug)]
pub struct InternalServiceSecurity {
    verifier: SignedPrincipalVerifier,
    unauthorized_code: Arc<str>,
}

/// Header values required at an internal service boundary.
#[derive(Clone, Copy, Debug, Default)]
pub struct InternalServicePrincipalHeaders<'a> {
    /// Whether the raw public `Authorization` header is still present.
    pub authorization_present: bool,
    /// Gateway internal service identity header value.
    pub service_identity: Option<&'a str>,
    /// Original public protocol surface value.
    pub protocol: Option<&'a str>,
    /// Gateway-issued auth scope value.
    pub scope: Option<&'a str>,
    /// Gateway-issued signed principal value.
    pub signed_principal: Option<&'a str>,
}

/// Reason an internal service principal was rejected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InternalServiceSecurityError {
    /// Raw public bearer authorization reached the internal plane.
    RawPublicAuthorization,
    /// Missing internal service identity header.
    MissingInternalServiceIdentity,
    /// Missing original public protocol header.
    MissingPublicProtocol,
    /// Unknown original public protocol header.
    UnknownPublicProtocol,
    /// Missing auth scope header.
    MissingAuthScope,
    /// Auth scope does not match the declared public protocol.
    AuthScopeMismatch,
    /// Missing signed principal header.
    MissingSignedPrincipal,
    /// Signed principal failed verification.
    InvalidSignedPrincipal,
}

impl InternalServiceSecurityError {
    /// Stable error message for HTTP and Flight adapters.
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::RawPublicAuthorization => "raw public Authorization header is not accepted",
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

impl InternalServiceSecurity {
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

    /// Create one verifier for requests admitted by Wendao Gateway.
    #[must_use]
    pub fn gateway(signing_secret: Arc<str>, unauthorized_code: Arc<str>) -> Self {
        Self::new(
            Arc::<str>::from(WENDAO_GATEWAY_INTERNAL_SERVICE_IDENTITY),
            signing_secret,
            unauthorized_code,
        )
    }

    /// Verify internal service headers issued by the public Gateway.
    ///
    /// # Errors
    ///
    /// Returns [`InternalServiceSecurityError`] when any required internal
    /// header is missing, mismatched, or has an invalid signed principal.
    pub fn verify_headers(
        &self,
        headers: InternalServicePrincipalHeaders<'_>,
    ) -> Result<(), InternalServiceSecurityError> {
        if headers.authorization_present {
            return Err(InternalServiceSecurityError::RawPublicAuthorization);
        }

        let Some(service_identity) = headers.service_identity else {
            return Err(InternalServiceSecurityError::MissingInternalServiceIdentity);
        };
        let Some(protocol) = headers.protocol else {
            return Err(InternalServiceSecurityError::MissingPublicProtocol);
        };
        let Some(surface) = PublicProtocolSurface::from_protocol(protocol) else {
            return Err(InternalServiceSecurityError::UnknownPublicProtocol);
        };
        let Some(scope) = headers.scope else {
            return Err(InternalServiceSecurityError::MissingAuthScope);
        };
        if scope != surface.scope() {
            return Err(InternalServiceSecurityError::AuthScopeMismatch);
        }
        let Some(signed_principal) = headers.signed_principal else {
            return Err(InternalServiceSecurityError::MissingSignedPrincipal);
        };
        if !self
            .verifier
            .verify_signed_principal(surface, service_identity, signed_principal)
        {
            return Err(InternalServiceSecurityError::InvalidSignedPrincipal);
        }

        Ok(())
    }
}

/// Wrap an Axum router with internal service principal verification.
#[must_use]
pub fn with_internal_service_security<S>(
    router: Router<S>,
    security: InternalServiceSecurity,
) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    router.route_layer(middleware::from_fn_with_state(
        security,
        require_internal_service_security,
    ))
}

async fn require_internal_service_security(
    State(security): State<InternalServiceSecurity>,
    request: Request,
    next: Next,
) -> Response {
    if let Err(error) = security.verify_headers(InternalServicePrincipalHeaders {
        authorization_present: request.headers().contains_key(header::AUTHORIZATION),
        service_identity: header_str(&request, WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER),
        protocol: header_str(&request, WENDAO_PUBLIC_PROTOCOL_HEADER),
        scope: header_str(&request, WENDAO_AUTH_SCOPE_HEADER),
        signed_principal: header_str(&request, WENDAO_SIGNED_PRINCIPAL_HEADER),
    }) {
        return unauthorized_response(security.unauthorized_code.as_ref(), error.message());
    }

    next.run(request).await
}

fn header_str<'a>(request: &'a Request, name: &str) -> Option<&'a str> {
    request
        .headers()
        .get(name)
        .and_then(|value| value.to_str().ok())
}

fn unauthorized_response(code: &str, error: &str) -> Response {
    (
        StatusCode::UNAUTHORIZED,
        Json(serde_json::json!({
            "code": code,
            "error": error,
        })),
    )
        .into_response()
}
