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
    if request.headers().contains_key(header::AUTHORIZATION) {
        return unauthorized_response(
            security.unauthorized_code.as_ref(),
            "raw public Authorization header is not accepted",
        );
    }

    let Some(service_identity) = header_str(&request, WENDAO_INTERNAL_SERVICE_IDENTITY_HEADER)
    else {
        return unauthorized_response(
            security.unauthorized_code.as_ref(),
            "missing internal service identity",
        );
    };
    let Some(protocol) = header_str(&request, WENDAO_PUBLIC_PROTOCOL_HEADER) else {
        return unauthorized_response(
            security.unauthorized_code.as_ref(),
            "missing public protocol",
        );
    };
    let Some(surface) = PublicProtocolSurface::from_protocol(protocol) else {
        return unauthorized_response(
            security.unauthorized_code.as_ref(),
            "unknown public protocol",
        );
    };
    let Some(scope) = header_str(&request, WENDAO_AUTH_SCOPE_HEADER) else {
        return unauthorized_response(security.unauthorized_code.as_ref(), "missing auth scope");
    };
    if scope != surface.scope() {
        return unauthorized_response(security.unauthorized_code.as_ref(), "auth scope mismatch");
    }
    let Some(signed_principal) = header_str(&request, WENDAO_SIGNED_PRINCIPAL_HEADER) else {
        return unauthorized_response(
            security.unauthorized_code.as_ref(),
            "missing signed principal",
        );
    };
    if !security
        .verifier
        .verify_signed_principal(surface, service_identity, signed_principal)
    {
        return unauthorized_response(
            security.unauthorized_code.as_ref(),
            "invalid signed principal",
        );
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
